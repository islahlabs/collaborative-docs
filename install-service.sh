#!/bin/bash

# Collaborative Docs Systemd Service Installer with CLI Arguments
set -e

echo "🚀 Installing Collaborative Docs as a systemd service..."

# Default values
DB_USER="collaborative_user"
DB_PASSWORD=""
DB_HOST="localhost"
DB_PORT="5432"
DB_NAME="collaborative_docs"

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -u, --db-user USER        Database username (default: collaborative_user)"
    echo "  -p, --db-password PASS    Database password (required)"
    echo "  -h, --db-host HOST        Database host (default: localhost)"
    echo "  -P, --db-port PORT        Database port (default: 5432)"
    echo "  -d, --db-name NAME        Database name (default: collaborative_docs)"
    echo "  --help                    Show this help message"
    echo ""
    echo "Example:"
    echo "  sudo $0 --db-password mypassword123"
    echo "  sudo $0 -u myuser -p mypass -h db.example.com"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -u|--db-user)
            DB_USER="$2"
            shift 2
            ;;
        -p|--db-password)
            DB_PASSWORD="$2"
            shift 2
            ;;
        -h|--db-host)
            DB_HOST="$2"
            shift 2
            ;;
        -P|--db-port)
            DB_PORT="$2"
            shift 2
            ;;
        -d|--db-name)
            DB_NAME="$2"
            shift 2
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            echo "❌ Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Check if password is provided
if [[ -z "$DB_PASSWORD" ]]; then
    echo "❌ Database password is required!"
    echo "Use -p or --db-password to specify the password"
    show_usage
    exit 1
fi

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo "❌ This script must be run as root (use sudo)"
   exit 1
fi

# Configuration
SERVICE_NAME="collaborative-docs"
SERVICE_USER="collaborative-docs"
SERVICE_GROUP="collaborative-docs"
INSTALL_DIR="/opt/collaborative-docs"
BACKEND_DIR="$INSTALL_DIR/backend"
SERVICE_FILE="collaborative-docs.service"

# Build DATABASE_URL
DATABASE_URL="postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Show configuration
echo "📋 Installation Configuration:"
echo "   Database User: $DB_USER"
echo "   Database Host: $DB_HOST:$DB_PORT"
echo "   Database Name: $DB_NAME"
echo "   Database URL:  $DATABASE_URL"
echo ""

# Create service user and group
print_status "Creating service user and group..."
if ! id "$SERVICE_USER" &>/dev/null; then
    useradd --system --shell /bin/false --home-dir "$INSTALL_DIR" "$SERVICE_USER"
    print_status "Created user: $SERVICE_USER"
else
    print_warning "User $SERVICE_USER already exists"
fi

# Create installation directory
print_status "Creating installation directory..."
mkdir -p "$BACKEND_DIR"
mkdir -p "$BACKEND_DIR/logs"

# Copy files to installation directory
print_status "Copying application files..."
# Copy backend directory contents to avoid nested structure
cp -r backend/* "$BACKEND_DIR/"
chown -R "$SERVICE_USER:$SERVICE_GROUP" "$INSTALL_DIR"

# Create logs directory with proper permissions
mkdir -p "$BACKEND_DIR/logs"
chown "$SERVICE_USER:$SERVICE_GROUP" "$BACKEND_DIR/logs"
chmod 755 "$BACKEND_DIR/logs"

# Update service file with correct paths and database URL
print_status "Updating service file with provided configuration..."
cat > "$BACKEND_DIR/collaborative-docs.service" << EOF
[Unit]
Description=Collaborative Docs Backend API
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=collaborative-docs
Group=collaborative-docs
WorkingDirectory=$BACKEND_DIR
ExecStart=$BACKEND_DIR/target/release/collaborative-docs-rs
Restart=always
RestartSec=10
Environment=RUST_LOG=info
Environment=RUN_MODE=production
Environment=DATABASE_URL=$DATABASE_URL
Environment=APP__SERVER__PORT=3001
Environment=APP__SERVER__HOST=0.0.0.0

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$BACKEND_DIR/logs

# Resource limits
LimitNOFILE=65536
MemoryMax=1G

[Install]
WantedBy=multi-user.target
EOF

# Build the application
print_status "Building application as current user..."
cd "$BACKEND_DIR"

# Temporarily change ownership to sudo user for building
print_status "Setting up build permissions..."
chown -R "$SUDO_USER:$SUDO_USER" "$BACKEND_DIR"

# Build as regular user using sudo -u with full cargo path
print_status "Building application as user $SUDO_USER..."
CARGO_PATH="/home/$SUDO_USER/.cargo/bin/cargo"
if [ ! -f "$CARGO_PATH" ]; then
    print_error "❌ Cargo not found at $CARGO_PATH"
    print_error "Please ensure Rust is installed for user $SUDO_USER"
    exit 1
fi
print_status "Using cargo at: $CARGO_PATH"
print_status "Building from directory: $(pwd)"
print_status "Using DATABASE_URL: $DATABASE_URL"

sudo -u "$SUDO_USER" env DATABASE_URL="$DATABASE_URL" "$CARGO_PATH" build --release

# Run database migrations
print_status "Running database migrations..."
# Check if sqlx-cli is installed
SQLX_PATH="/home/$SUDO_USER/.cargo/bin/sqlx"
if [ ! -f "$SQLX_PATH" ]; then
    print_status "Installing sqlx-cli..."
    sudo -u "$SUDO_USER" "$CARGO_PATH" install sqlx-cli --no-default-features --features postgres
fi

# Run migrations with the correct DATABASE_URL using full path to sqlx
print_status "Running migrations with sqlx..."
sudo -u "$SUDO_USER" env DATABASE_URL="$DATABASE_URL" "$SQLX_PATH" migrate run

# Change ownership back to service user
print_status "Setting service permissions..."
chown -R "$SERVICE_USER:$SERVICE_GROUP" "$BACKEND_DIR"

# Ensure proper ownership of built files
chown -R "$SERVICE_USER:$SERVICE_GROUP" "$BACKEND_DIR"

# Install systemd service
print_status "Installing systemd service..."
# The service file was created with the correct configuration
SERVICE_FILE_PATH="$BACKEND_DIR/collaborative-docs.service"
if [ -f "$SERVICE_FILE_PATH" ]; then
    cp "$SERVICE_FILE_PATH" /etc/systemd/system/
else
    print_error "❌ Service file not found at $SERVICE_FILE_PATH"
    exit 1
fi
systemctl daemon-reload

# Enable and start the service
print_status "Enabling and starting the service..."
systemctl enable "$SERVICE_NAME"
systemctl start "$SERVICE_NAME"

# Check service status
print_status "Checking service status..."
if systemctl is-active --quiet "$SERVICE_NAME"; then
    print_status "✅ Service is running successfully!"
else
    print_error "❌ Service failed to start"
    systemctl status "$SERVICE_NAME"
    exit 1
fi

# Show service information
echo ""
print_status "Service installation complete!"
echo ""
echo "📋 Service Information:"
echo "   Service Name: $SERVICE_NAME"
echo "   Status: $(systemctl is-active $SERVICE_NAME)"
echo "   Enabled: $(systemctl is-enabled $SERVICE_NAME)"
echo "   Database: $DB_USER@$DB_HOST:$DB_PORT/$DB_NAME"
echo "   Logs: journalctl -u $SERVICE_NAME -f"
echo ""
echo "🔧 Useful Commands:"
echo "   Start:   sudo systemctl start $SERVICE_NAME"
echo "   Stop:    sudo systemctl stop $SERVICE_NAME"
echo "   Restart: sudo systemctl restart $SERVICE_NAME"
echo "   Status:  sudo systemctl status $SERVICE_NAME"
echo "   Logs:    sudo journalctl -u $SERVICE_NAME -f"
echo ""
echo "🌐 API Access:"
echo "   Swagger UI: http://localhost:3001/swagger-ui"
echo "   API Base:   http://localhost:3001"
echo ""
print_status "Installation completed successfully!" 