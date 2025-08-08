#!/bin/bash

# Collaborative Docs Systemd Service Installer
set -e

echo "🚀 Installing Collaborative Docs as a systemd service..."

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
# Also copy the service file from project root
cp collaborative-docs.service "$BACKEND_DIR/"
chown -R "$SERVICE_USER:$SERVICE_GROUP" "$INSTALL_DIR"

# Create logs directory with proper permissions
mkdir -p "$BACKEND_DIR/logs"
chown "$SERVICE_USER:$SERVICE_GROUP" "$BACKEND_DIR/logs"
chmod 755 "$BACKEND_DIR/logs"

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

# Set DATABASE_URL for sqlx compilation
export DATABASE_URL="postgresql://user:pass@localhost:5432/collaborative_docs"
print_status "Using DATABASE_URL: $DATABASE_URL"

sudo -u "$SUDO_USER" env DATABASE_URL="$DATABASE_URL" "$CARGO_PATH" build --release

# Change ownership back to service user
print_status "Setting service permissions..."
chown -R "$SERVICE_USER:$SERVICE_GROUP" "$BACKEND_DIR"

# Ensure proper ownership of built files
chown -R "$SERVICE_USER:$SERVICE_GROUP" "$BACKEND_DIR"

# Install systemd service
print_status "Installing systemd service..."
# The service file was copied to the backend directory during installation
SERVICE_FILE_PATH="$BACKEND_DIR/collaborative-docs.service"
if [ -f "$SERVICE_FILE_PATH" ]; then
    cp "$SERVICE_FILE_PATH" /etc/systemd/system/
else
    print_error "❌ Service file not found at $SERVICE_FILE_PATH"
    print_error "Please ensure the service file exists in the project root"
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