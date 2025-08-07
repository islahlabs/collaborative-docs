# Port Configuration Quick Reference

This document provides a quick reference for configuring ports in the collaborative-docs application.

## 🔧 Backend Port Configuration

### Environment Variables (Recommended)
```bash
# Set backend port
export APP__SERVER__PORT=8080

# Set backend host
export APP__SERVER__HOST=0.0.0.0
```

### Configuration Files
Create or edit `backend/config/default.toml`:
```toml
[server]
host = "0.0.0.0"
port = 8080  # Change this to your desired port
```

### Docker Environment Variables
```yaml
# docker-compose.yml
services:
  app:
    environment:
      - APP__SERVER__PORT=8080
      - APP__SERVER__HOST=0.0.0.0
    ports:
      - "8080:8080"  # Map host port to container port
```

## 🎨 Frontend Port Configuration

### Environment Variables
```bash
# Set frontend dev server port
export VITE_PORT=3001

# Set frontend dev server host
export VITE_HOST=localhost
```

### Docker Frontend Port
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

## 🗄️ Database Port Configuration

### Environment Variables
```bash
# Set database port
export APP__DATABASE__PORT=5433

# Or use DATABASE_URL
export DATABASE_URL="postgresql://user:pass@localhost:5433/collaborative_docs"
```

### Docker Database Port
```yaml
# docker-compose.yml
services:
  postgres:
    ports:
      - "5433:5432"  # Map host port 5433 to container port 5432
```

## 🔄 Reverse Proxy Configuration

### Nginx Example
```nginx
# Backend API proxy
location /api/ {
    proxy_pass http://localhost:8080;  # Your backend port
}

# WebSocket proxy
location /ws/ {
    proxy_pass http://localhost:8080;  # Your backend port
}
```

### Apache Example
```apache
# Backend API proxy
ProxyPass /api/ http://localhost:8080/api/
ProxyPassReverse /api/ http://localhost:8080/api/

# WebSocket proxy
ProxyPass /ws/ ws://localhost:8080/ws/
ProxyPassReverse /ws/ ws://localhost:8080/ws/
```

## 📋 Common Port Configurations

### Development (Default)
- **Backend**: 3000
- **Frontend**: 5173
- **Database**: 5432

### Production Example
- **Backend**: 8080
- **Frontend**: 80 (served by nginx)
- **Database**: 5432 (internal)

### Custom Example
- **Backend**: 9000
- **Frontend**: 3001
- **Database**: 5433

## 🔍 Troubleshooting

### Check Port Usage
```bash
# Check what's using a port
sudo lsof -i :3000
sudo lsof -i :5173

# Kill process using port
sudo kill -9 <PID>
```

### Test Port Availability
```bash
# Test if port is available
nc -z localhost 3000
nc -z localhost 5173
```

### Docker Port Mapping
```bash
# Check Docker port mappings
docker port <container_name>

# Example output:
# 3000/tcp -> 0.0.0.0:8080
# 5173/tcp -> 0.0.0.0:3001
```

## 📝 Environment Variables Summary

| Component | Variable | Default | Description |
|-----------|----------|---------|-------------|
| Backend | `APP__SERVER__PORT` | 3000 | Backend server port |
| Backend | `APP__SERVER__HOST` | 0.0.0.0 | Backend server host |
| Frontend | `VITE_PORT` | 5173 | Frontend dev server port |
| Frontend | `VITE_HOST` | localhost | Frontend dev server host |
| Database | `APP__DATABASE__PORT` | 5432 | Database port |
| Database | `DATABASE_URL` | - | Full database URL |

## 🚀 Quick Setup Examples

### Change Backend Port to 8080
```bash
export APP__SERVER__PORT=8080
cd backend && cargo run
```

### Change Frontend Port to 3001
```bash
export VITE_PORT=3001
cd frontend && pnpm dev
```

### Use Custom Database Port
```bash
export APP__DATABASE__PORT=5433
export DATABASE_URL="postgresql://user:pass@localhost:5433/collaborative_docs"
cd backend && cargo run
```

### Docker with Custom Ports
```bash
# Create .env file
echo "APP__SERVER__PORT=8080" > .env
echo "VITE_PORT=3001" >> .env
echo "POSTGRES_PASSWORD=mysecret" >> .env

# Start with custom ports
docker-compose up -d
``` 