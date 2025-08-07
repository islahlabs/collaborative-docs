# Deployment Guide

This guide covers deploying the collaborative-docs application in various environments, including port configuration, database setup, and reverse proxy configuration.

## 🚀 Quick Start

### Docker Deployment (Recommended)

```bash
# Clone the repository
git clone https://github.com/islahlabs/collaborative-docs.git
cd collaborative-docs

# Start all services
docker-compose up -d

# Access the application
# Frontend: http://localhost:5173
# Backend API: http://localhost:3000
```

## 🔧 Port Configuration

### Backend Port Configuration

The backend port is configurable through multiple methods:

#### 1. Environment Variables (Highest Priority)
```bash
# Set backend port
export APP__SERVER__PORT=8080

# Or use DATABASE_URL for database
export DATABASE_URL="postgresql://user:pass@localhost:5432/collaborative_docs"
```

#### 2. Configuration Files
Create `backend/config/default.toml`:
```toml
[server]
host = "0.0.0.0"
port = 8080  # Change this to your desired port

[database]
url = "postgresql://user:password@localhost:5432/collaborative_docs"

[cors]
allowed_origins = ["http://localhost:5173", "http://localhost:8080"]
allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
```

#### 3. Docker Environment Variables
```yaml
# docker-compose.yml
services:
  app:
    environment:
      - APP__SERVER__PORT=8080
      - APP__SERVER__HOST=0.0.0.0
```

### Frontend Port Configuration

The frontend dev server port is configurable via environment variables:

```bash
# Set frontend port
export VITE_PORT=3001
export VITE_HOST=localhost

# Start frontend
cd frontend
pnpm dev
```

#### Docker Frontend Port
```yaml
# docker-compose.yml
services:
  frontend:
    environment:
      - VITE_PORT=3001
      - VITE_HOST=0.0.0.0
    ports:
      - "3001:3001"
```

## 🗄️ Database Setup

### Option 1: Docker PostgreSQL (Recommended)

The `docker-compose.yml` automatically sets up PostgreSQL:

```yaml
services:
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: collaborative_docs
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d
```

### Option 2: Manual PostgreSQL Setup

#### 1. Install PostgreSQL

**macOS:**
```bash
brew install postgresql
brew services start postgresql
```

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

**CentOS/RHEL:**
```bash
sudo yum install postgresql postgresql-server
sudo postgresql-setup initdb
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

#### 2. Create Database and User

**Option A: Automated Setup (Recommended)**

Use the provided setup script:

```bash
# Run the setup script
./scripts/setup-database.sh

# Or with custom parameters
./scripts/setup-database.sh \
  --db-name myapp \
  --db-user myuser \
  --db-password mypass \
  --admin-email admin@example.com \
  --admin-password secure123
```

**Option B: Manual Setup**

```bash
# Connect to PostgreSQL as superuser
sudo -u postgres psql

# Create database and user
CREATE DATABASE collaborative_docs;
CREATE USER collaborative_user WITH PASSWORD 'collaborative_password';
GRANT ALL PRIVILEGES ON DATABASE collaborative_docs TO collaborative_user;
ALTER USER collaborative_user CREATEDB;

# Exit PostgreSQL
\q

# Connect to the database as the new user to set up schema permissions
psql -h localhost -U collaborative_user -d collaborative_docs

# Grant schema privileges
GRANT ALL PRIVILEGES ON SCHEMA public TO collaborative_user;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO collaborative_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO collaborative_user;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO collaborative_user;

-- Grant privileges on future tables/sequences/functions
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON TABLES TO collaborative_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON SEQUENCES TO collaborative_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL PRIVILEGES ON FUNCTIONS TO collaborative_user;

-- Exit
\q
```

#### 3. Run Migrations

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Set database URL
export DATABASE_URL="postgresql://collaborative_user:collaborative_password@localhost:5432/collaborative_docs"

# Run migrations
cd backend
sqlx migrate run
```

#### 4. Create Admin User

```bash
# Create an admin user
cargo run --bin create_admin admin@example.com adminpassword123
```

### Option 3: Cloud Database (Production)

For production, use a managed PostgreSQL service:

#### AWS RDS
```bash
# Set environment variables
export DATABASE_URL="postgresql://username:password@your-rds-endpoint:5432/collaborative_docs"
```

#### Google Cloud SQL
```bash
export DATABASE_URL="postgresql://username:password@your-cloudsql-ip:5432/collaborative_docs"
```

#### Heroku Postgres
```bash
# Heroku automatically sets DATABASE_URL
heroku config:get DATABASE_URL
```

## 🌐 Reverse Proxy Configuration

### Nginx Configuration

Create `/etc/nginx/sites-available/collaborative-docs`:

```nginx
server {
    listen 80;
    server_name yourdomain.com www.yourdomain.com;

    # Frontend (static files)
    location / {
        root /var/www/collaborative-docs/frontend/dist;
        try_files $uri $uri/ /index.html;
        
        # Cache static assets
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }

    # Backend API
    location /api/ {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }

    # WebSocket connections
    location /ws/ {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Enable the site:
```bash
sudo ln -s /etc/nginx/sites-available/collaborative-docs /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### Apache Configuration

Create `/etc/apache2/sites-available/collaborative-docs.conf`:

```apache
<VirtualHost *:80>
    ServerName yourdomain.com
    ServerAlias www.yourdomain.com

    # Frontend
    DocumentRoot /var/www/collaborative-docs/frontend/dist
    
    <Directory /var/www/collaborative-docs/frontend/dist>
        AllowOverride All
        Require all granted
    </Directory>

    # Backend API proxy
    ProxyPreserveHost On
    ProxyPass /api/ http://localhost:3000/api/
    ProxyPassReverse /api/ http://localhost:3000/api/
    
    # WebSocket proxy
    ProxyPass /ws/ ws://localhost:3000/ws/
    ProxyPassReverse /ws/ ws://localhost:3000/ws/

    # Rewrite for SPA
    RewriteEngine On
    RewriteCond %{REQUEST_FILENAME} !-f
    RewriteCond %{REQUEST_FILENAME} !-d
    RewriteRule ^(.*)$ /index.html [QSA,L]
</VirtualHost>
```

Enable modules and site:
```bash
sudo a2enmod proxy
sudo a2enmod proxy_http
sudo a2enmod proxy_wstunnel
sudo a2enmod rewrite
sudo a2ensite collaborative-docs
sudo systemctl reload apache2
```

## 🔒 SSL/HTTPS Configuration

### Let's Encrypt with Certbot

```bash
# Install Certbot
sudo apt-get install certbot python3-certbot-nginx

# Get SSL certificate
sudo certbot --nginx -d yourdomain.com -d www.yourdomain.com

# Auto-renewal
sudo crontab -e
# Add: 0 12 * * * /usr/bin/certbot renew --quiet
```

## 🐳 Production Docker Deployment

### 1. Create Production Environment File

Create `.env.production`:
```bash
# Database
POSTGRES_DB=collaborative_docs
POSTGRES_USER=collaborative_user
POSTGRES_PASSWORD=your_secure_password_here

# Backend
RUN_MODE=production
RUST_LOG=warn
APP__SERVER__PORT=3000
APP__SERVER__HOST=0.0.0.0

# Frontend
VITE_PORT=5173
VITE_HOST=0.0.0.0

# CORS
CORS_ORIGINS=["https://yourdomain.com"]
```

### 2. Build and Deploy

```bash
# Build production images
docker-compose -f docker-compose.prod.yml build

# Deploy
docker-compose -f docker-compose.prod.yml --env-file .env.production up -d
```

### 3. Update Frontend API URL

In production, update the frontend API URL:

```bash
# Set environment variable
export VITE_API_URL=https://yourdomain.com/api

# Or update the frontend configuration
cd frontend
echo "VITE_API_URL=https://yourdomain.com/api" >> .env.production
```

## 📊 Monitoring and Logging

### Backend Logs
```bash
# View backend logs
docker-compose logs -f app

# Or if running manually
tail -f /var/log/collaborative-docs/app.log
```

### Database Monitoring
```bash
# Check database connections
docker-compose exec postgres psql -U postgres -d collaborative_docs -c "SELECT * FROM pg_stat_activity;"

# Check database size
docker-compose exec postgres psql -U postgres -d collaborative_docs -c "SELECT pg_size_pretty(pg_database_size('collaborative_docs'));"
```

## 🔧 Troubleshooting

### Common Issues

#### 1. Port Already in Use
```bash
# Check what's using the port
sudo lsof -i :3000
sudo lsof -i :5173

# Kill the process
sudo kill -9 <PID>
```

#### 2. Database Connection Issues
```bash
# Test database connection
psql -h localhost -U collaborative_user -d collaborative_docs

# Check PostgreSQL status
sudo systemctl status postgresql
```

#### 3. CORS Issues
Make sure your CORS configuration matches your frontend URL:
```toml
[cors]
allowed_origins = ["http://localhost:5173", "https://yourdomain.com"]
```

#### 4. WebSocket Connection Issues
Check that your reverse proxy is properly configured for WebSocket upgrades.

## 📝 Environment Variables Reference

### Backend Environment Variables
| Variable | Default | Description |
|----------|---------|-------------|
| `APP__SERVER__PORT` | 3000 | Backend server port |
| `APP__SERVER__HOST` | 0.0.0.0 | Backend server host |
| `APP__DATABASE__HOST` | localhost | Database host |
| `APP__DATABASE__PORT` | 5432 | Database port |
| `APP__DATABASE__USERNAME` | postgres | Database username |
| `APP__DATABASE__PASSWORD` | password | Database password |
| `APP__DATABASE__DATABASE` | collaborative_docs | Database name |
| `DATABASE_URL` | - | Full database URL (overrides individual settings) |
| `RUN_MODE` | development | Application mode (development/production) |
| `RUST_LOG` | info | Logging level |

### Frontend Environment Variables
| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_PORT` | 5173 | Frontend dev server port |
| `VITE_HOST` | localhost | Frontend dev server host |
| `VITE_API_URL` | http://localhost:3000 | Backend API URL |

### Docker Environment Variables
| Variable | Default | Description |
|----------|---------|-------------|
| `POSTGRES_DB` | collaborative_docs | PostgreSQL database name |
| `POSTGRES_USER` | postgres | PostgreSQL username |
| `POSTGRES_PASSWORD` | password | PostgreSQL password |
| `CORS_ORIGINS` | ["http://localhost:5173"] | Allowed CORS origins | 