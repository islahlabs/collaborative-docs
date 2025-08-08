# Production Deployment Guide for docs.islahlabs.com

This guide covers deploying the collaborative docs application to your production server at `docs.islahlabs.com`.

## Architecture Overview

- **Frontend**: Static files served by Nginx from `/var/www/docs.islahlabs.com`
- **Backend**: Rust API server running on port 3001
- **Database**: PostgreSQL 
- **Reverse Proxy**: Nginx handles SSL, static files, and API proxying

## Backend Setup

### 1. Configure Environment

```bash
cd backend
./setup-production-env.sh
```

Edit the generated `.env` file:
```bash
# Set your actual database URL
DATABASE_URL=postgresql://your_user:your_password@localhost:5432/collaborative_docs

# Set a secure JWT secret (generate with: openssl rand -hex 32)
APP__AUTH__JWT_SECRET=your-super-secret-jwt-key-here
```

### 2. Build and Deploy Backend

```bash
# Build for production
cargo build --release

# Copy binary to production location
sudo cp target/release/collaborative-docs /usr/local/bin/

# Create systemd service (if using systemd)
sudo cp collaborative-docs.service /etc/systemd/system/
sudo systemctl enable collaborative-docs
sudo systemctl start collaborative-docs
```

## Frontend Setup

### 1. Build Frontend

```bash
cd frontend
# Build with production API URL
VITE_API_URL=https://docs.islahlabs.com pnpm run build
```

### 2. Deploy to Nginx

```bash
# Copy built files to web root
sudo cp -r dist/* /var/www/docs.islahlabs.com/
```

## Nginx Configuration

Your current Nginx configuration is already correct:

```nginx
# Serves frontend static files
location / {
    try_files $uri /index.html;
}

# Proxies API requests to backend on port 3001
location /api/ {
    proxy_pass http://127.0.0.1:3001;
    # ... headers
}

# Proxies WebSocket connections
location /ws/ {
    proxy_pass http://127.0.0.1:3001;
    # ... WebSocket headers
}
```

## Environment Variables Summary

### Backend (.env)
- `APP__SERVER__PORT=3001` - Matches Nginx proxy
- `APP__CORS__ALLOWED_ORIGINS=["https://docs.islahlabs.com"]` - Allows frontend
- `RUN_MODE=production` - Uses production config
- `DATABASE_URL=postgresql://...` - Your database connection

### Frontend (build time)
- `VITE_API_URL=https://docs.islahlabs.com` - Points to your domain

## Verification

1. **Backend health check**: `curl https://docs.islahlabs.com/api/doc/health`
2. **Frontend loads**: Visit `https://docs.islahlabs.com`
3. **WebSocket works**: Check browser dev tools for WebSocket connections
4. **CORS works**: No CORS errors in browser console

## Troubleshooting

### CORS Issues
- Ensure backend has `APP__CORS__ALLOWED_ORIGINS=["https://docs.islahlabs.com"]`
- Check backend logs for CORS-related errors

### API Not Found (404)
- Verify backend is running on port 3001
- Check Nginx proxy_pass configuration
- Ensure API routes start with `/api/`

### WebSocket Connection Failed
- Verify WebSocket proxy in Nginx (`/ws/` location)
- Check backend WebSocket handler is working
- Ensure WSS (secure WebSocket) is used for HTTPS sites 