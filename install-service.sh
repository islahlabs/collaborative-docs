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
INSTALL_DIR="/path/to/collaborative-docs"
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
cp -r . "$BACKEND_DIR/"
chown -R "$SERVICE_USER:$SERVICE_GROUP" "$INSTALL_DIR"

# Create logs directory with proper permissions
mkdir -p "$BACKEND_DIR/logs"
chown "$SERVICE_USER:$SERVICE_GROUP" "$BACKEND_DIR/logs"
chmod 755 "$BACKEND_DIR/logs"

# Build the application
print_status "Building the application..."
cd "$BACKEND_DIR"
sudo -u "$SERVICE_USER" cargo build --release

# Install systemd service
print_status "Installing systemd service..."
cp "$SERVICE_FILE" /etc/systemd/system/
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