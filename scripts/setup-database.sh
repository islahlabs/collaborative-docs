#!/bin/bash

# Collaborative Docs Database Setup Script
# This script sets up PostgreSQL database and user for the collaborative-docs application

set -e

echo "🗄️  Setting up Collaborative Docs Database..."

# Default values
DB_NAME=${DB_NAME:-"collaborative_docs"}
DB_USER=${DB_USER:-"collaborative_user"}
DB_PASSWORD=${DB_PASSWORD:-"collaborative_password"}
DB_HOST=${DB_HOST:-"localhost"}
DB_PORT=${DB_PORT:-"5432"}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if PostgreSQL is running
check_postgres() {
    print_status "Checking PostgreSQL connection..."
    
    if command -v pg_isready &> /dev/null; then
        if pg_isready -h $DB_HOST -p $DB_PORT &> /dev/null; then
            print_status "PostgreSQL is running on $DB_HOST:$DB_PORT"
            return 0
        else
            print_error "PostgreSQL is not running on $DB_HOST:$DB_PORT"
            print_warning "Please start PostgreSQL first:"
            echo "   macOS: brew services start postgresql"
            echo "   Ubuntu: sudo systemctl start postgresql"
            echo "   CentOS: sudo systemctl start postgresql"
            echo "   Arch Linux: sudo systemctl start postgresql"
            return 1
        fi
    else
        print_error "PostgreSQL client (psql) not found"
        print_warning "Please install PostgreSQL first:"
        echo "   macOS: brew install postgresql"
        echo "   Ubuntu: sudo apt-get install postgresql postgresql-contrib"
        echo "   CentOS: sudo yum install postgresql postgresql-server"
        echo "   Arch Linux: sudo pacman -S postgresql"
        return 1
    fi
}

# Create database and user
create_database() {
    print_status "Creating database and user..."
    
    # Connect as postgres superuser and create database/user
    sudo -u postgres psql << EOF
-- Create database
CREATE DATABASE $DB_NAME;

-- Create user
CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';

-- Grant privileges on database
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;
ALTER USER $DB_USER CREATEDB;

-- Exit
\q
EOF
    
    if [ $? -eq 0 ]; then
        print_status "Database '$DB_NAME' and user '$DB_USER' created successfully"
    else
        print_error "Failed to create database and user"
        return 1
    fi
    
    # Now connect to the specific database and grant schema privileges
    print_status "Setting up schema permissions..."
    PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME << EOF
-- Grant privileges on public schema
GRANT ALL PRIVILEGES ON SCHEMA public TO $DB_USER;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO $DB_USER;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO $DB_USER;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO $DB_USER;

-- Grant privileges on future tables/sequences/functions
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON TABLES TO $DB_USER;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON SEQUENCES TO $DB_USER;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON FUNCTIONS TO $DB_USER;

-- Exit
\q
EOF
    
    if [ $? -eq 0 ]; then
        print_status "Schema permissions set up successfully"
    else
        print_error "Failed to set up schema permissions"
        return 1
    fi
}

# Test database connection
test_connection() {
    print_status "Testing database connection..."
    
    # Test connection with the new user
    if PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT version();" &> /dev/null; then
        print_status "Database connection successful"
        return 0
    else
        print_error "Database connection failed"
        return 1
    fi
}

# Run database migrations
run_migrations() {
    print_status "Running database migrations..."
    
    # Check if sqlx-cli is installed
    if ! command -v sqlx &> /dev/null; then
        print_warning "sqlx-cli not found. Installing..."
        cargo install sqlx-cli --no-default-features --features postgres
    fi
    
    # Set DATABASE_URL for migrations
    export DATABASE_URL="postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"
    
    # Run migrations
    cd backend
    if sqlx migrate run; then
        print_status "Database migrations completed successfully"
    else
        print_error "Database migrations failed"
        return 1
    fi
}

# Create admin user
create_admin_user() {
    print_status "Creating admin user..."
    
    # Check if admin email and password are provided
    if [ -z "$ADMIN_EMAIL" ] || [ -z "$ADMIN_PASSWORD" ]; then
        print_warning "Admin email and password not provided. Skipping admin user creation."
        print_warning "You can create an admin user later with:"
        echo "   cargo run --bin create_admin <email> <password>"
        return 0
    fi
    
    # Set DATABASE_URL
    export DATABASE_URL="postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"
    
    # Create admin user
    if cargo run --bin create_admin "$ADMIN_EMAIL" "$ADMIN_PASSWORD"; then
        print_status "Admin user created successfully"
    else
        print_error "Failed to create admin user"
        return 1
    fi
}

# Display configuration
show_config() {
    print_status "Database configuration:"
    echo "   Database Name: $DB_NAME"
    echo "   Database User: $DB_USER"
    echo "   Database Host: $DB_HOST"
    echo "   Database Port: $DB_PORT"
    echo "   Connection URL: postgresql://$DB_USER:****@$DB_HOST:$DB_PORT/$DB_NAME"
    
    if [ ! -z "$ADMIN_EMAIL" ]; then
        echo "   Admin Email: $ADMIN_EMAIL"
    fi
}

# Main execution
main() {
    echo "🚀 Collaborative Docs Database Setup"
    echo "=================================="
    echo ""
    
    # Show current configuration
    show_config
    echo ""
    
    # Check PostgreSQL
    if ! check_postgres; then
        exit 1
    fi
    
    # Check if database already exists
    if psql -h $DB_HOST -p $DB_PORT -U postgres -lqt | cut -d \| -f 1 | grep -qw $DB_NAME; then
        print_warning "Database '$DB_NAME' already exists"
        read -p "Do you want to recreate it? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_status "Dropping existing database..."
            sudo -u postgres psql -c "DROP DATABASE IF EXISTS $DB_NAME;"
            sudo -u postgres psql -c "DROP USER IF EXISTS $DB_USER;"
        else
            print_status "Using existing database"
            if test_connection; then
                run_migrations
                create_admin_user
                print_status "Setup completed successfully!"
                return 0
            else
                print_error "Cannot connect to existing database"
                exit 1
            fi
        fi
    fi
    
    # Create database and user
    if ! create_database; then
        exit 1
    fi
    
    # Test connection
    if ! test_connection; then
        exit 1
    fi
    
    # Run migrations
    if ! run_migrations; then
        exit 1
    fi
    
    # Create admin user
    create_admin_user
    
    print_status "✅ Database setup completed successfully!"
    echo ""
    print_status "Next steps:"
    echo "   1. Start the backend: cd backend && cargo run"
    echo "   2. Start the frontend: cd frontend && pnpm dev"
    echo "   3. Access the application: http://localhost:5173"
    echo ""
    print_status "Environment variables for your application:"
    echo "   export DATABASE_URL=\"postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME\""
    echo "   export APP__DATABASE__HOST=\"$DB_HOST\""
    echo "   export APP__DATABASE__PORT=\"$DB_PORT\""
    echo "   export APP__DATABASE__USERNAME=\"$DB_USER\""
    echo "   export APP__DATABASE__PASSWORD=\"$DB_PASSWORD\""
    echo "   export APP__DATABASE__DATABASE=\"$DB_NAME\""
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --db-name)
            DB_NAME="$2"
            shift 2
            ;;
        --db-user)
            DB_USER="$2"
            shift 2
            ;;
        --db-password)
            DB_PASSWORD="$2"
            shift 2
            ;;
        --db-host)
            DB_HOST="$2"
            shift 2
            ;;
        --db-port)
            DB_PORT="$2"
            shift 2
            ;;
        --admin-email)
            ADMIN_EMAIL="$2"
            shift 2
            ;;
        --admin-password)
            ADMIN_PASSWORD="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --db-name NAME        Database name (default: collaborative_docs)"
            echo "  --db-user USER        Database user (default: collaborative_user)"
            echo "  --db-password PASS    Database password (default: collaborative_password)"
            echo "  --db-host HOST        Database host (default: localhost)"
            echo "  --db-port PORT        Database port (default: 5432)"
            echo "  --admin-email EMAIL   Admin user email"
            echo "  --admin-password PASS Admin user password"
            echo "  --help                Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0"
            echo "  $0 --admin-email admin@example.com --admin-password secure123"
            echo "  $0 --db-name myapp --db-user myuser --db-password mypass"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Run main function
main 