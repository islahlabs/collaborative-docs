#!/bin/bash

# Frontend Environment Setup Script
# This script helps set up environment variables for different deployment scenarios

echo "Frontend Environment Setup"
echo "=========================="

# Function to create environment file
create_env_file() {
    local env_type=$1
    local api_url=$2
    local filename=".env.${env_type}"
    
    echo "Creating ${filename}..."
    cat > "${filename}" << EOF
# ${env_type^} environment variables
VITE_API_URL=${api_url}
EOF
    echo "✅ Created ${filename}"
}

# Check command line argument
case "${1:-development}" in
    "development"|"dev")
        create_env_file "development" "http://localhost:3000"
        echo ""
        echo "Development environment configured!"
        echo "Backend should be running on http://localhost:3000"
        ;;
    "production"|"prod")
        create_env_file "production" "https://docs.islahlabs.com"
        echo ""
        echo "Production environment configured!"
        echo "This will connect to https://docs.islahlabs.com"
        ;;
    "local")
        create_env_file "local" "http://localhost:3000"
        echo ""
        echo "Local environment configured!"
        ;;
    *)
        echo "Usage: $0 [development|production|local]"
        echo ""
        echo "Examples:"
        echo "  $0 development    # For local development (default)"
        echo "  $0 production     # For docs.islahlabs.com deployment"
        echo "  $0 local          # For local testing"
        exit 1
        ;;
esac

echo ""
echo "You can also set VITE_API_URL environment variable directly:"
echo "  export VITE_API_URL=https://docs.islahlabs.com"
echo "  pnpm run build" 