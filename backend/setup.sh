#!/bin/bash

# Collaborative Docs Backend Setup Script

set -e

echo "🚀 Setting up Collaborative Docs Backend..."

# Check if .env file exists
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cp env.example .env
    echo "✅ Created .env file. Please edit it with your settings."
else
    echo "✅ .env file already exists."
fi

# Check if PostgreSQL is running
echo "🔍 Checking PostgreSQL connection..."
if command -v pg_isready &> /dev/null; then
    if pg_isready -h localhost -p 5432 &> /dev/null; then
        echo "✅ PostgreSQL is running."
    else
        echo "⚠️  PostgreSQL is not running. Please start it first:"
        echo "   macOS: brew services start postgresql"
        echo "   Ubuntu: sudo systemctl start postgresql"
        exit 1
    fi
else
    echo "⚠️  PostgreSQL client not found. Please install PostgreSQL first."
    exit 1
fi

# Check if database exists
echo "🔍 Checking database..."
if psql -h localhost -U postgres -lqt | cut -d \| -f 1 | grep -qw collaborative_docs; then
    echo "✅ Database 'collaborative_docs' exists."
else
    echo "📝 Creating database..."
    createdb -h localhost -U postgres collaborative_docs
    echo "✅ Database created."
fi

# Check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo "📦 Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Run migrations
echo "🔄 Running database migrations..."
sqlx migrate run

echo "✅ Setup complete!"
echo ""
echo "To start the server:"
echo "  cargo run"
echo ""
echo "To start with Docker:"
echo "  docker-compose up -d"
echo ""
echo "API will be available at: http://localhost:3000" 