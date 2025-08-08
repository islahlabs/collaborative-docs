#!/bin/bash

# Backend Production Environment Setup Script
# This script helps set up environment variables for docs.islahlabs.com deployment

echo "Backend Production Environment Setup"
echo "===================================="

# Create production .env file
cat > .env << 'EOF'
# Production Environment Configuration for docs.islahlabs.com

# Server Configuration
APP__SERVER__HOST=0.0.0.0
APP__SERVER__PORT=3001
RUN_MODE=production
RUST_LOG=info

# CORS Configuration
APP__CORS__ALLOWED_ORIGINS=["https://docs.islahlabs.com"]
APP__CORS__ALLOWED_METHODS=["GET","POST","PUT","DELETE","OPTIONS"]

# Database Configuration
# Set your actual database URL here
# DATABASE_URL=postgresql://username:password@host:port/database

# JWT Secret (CHANGE THIS!)
# APP__AUTH__JWT_SECRET=your-super-secret-jwt-key-here

# Email Configuration (if using email features)
# APP__EMAIL__SMTP_SERVER=smtp.gmail.com
# APP__EMAIL__SMTP_PORT=587
# APP__EMAIL__SMTP_USERNAME=your-email@gmail.com
# APP__EMAIL__SMTP_PASSWORD=your-app-password
# APP__EMAIL__FROM_EMAIL=noreply@docs.islahlabs.com
# APP__EMAIL__BASE_URL=https://docs.islahlabs.com
EOF

echo "✅ Created .env file for production"
echo ""
echo "📝 Next steps:"
echo "1. Edit .env file and set your actual database URL"
echo "2. Set a secure JWT secret"
echo "3. Configure email settings if needed"
echo "4. Run: cargo build --release"
echo "5. Start with: ./target/release/collaborative-docs"
echo ""
echo "🔧 Your backend will run on port 3001 (matching your Nginx config)"
echo "🌐 CORS is configured for https://docs.islahlabs.com" 