# Collaborative Docs Backend

A production-ready Rust backend for collaborative document editing with PostgreSQL.

## Features

- ✅ **PostgreSQL Database** - Production-ready with proper indexing and transactions
- ✅ **Comprehensive Error Handling** - Custom error types with proper HTTP responses
- ✅ **Input Validation** - Request validation with detailed error messages
- ✅ **Structured Logging** - Tracing-based logging for debugging and monitoring
- ✅ **Configuration Management** - Environment-based configuration with multiple sources
- ✅ **CORS Support** - Configurable CORS for frontend integration
- ✅ **Document History** - Full version history with timestamps and IP tracking
- ✅ **Search Functionality** - PostgreSQL-powered full-text search
- ✅ **Docker Support** - Complete Docker setup for development and production
- ✅ **Comprehensive Testing** - Unit and integration tests

## Quick Start

### Using Docker (Recommended)

1. **Start the services:**
   ```bash
   docker-compose up -d
   ```

2. **The API will be available at:**
   ```
   http://localhost:3000
   ```

### Manual Setup

1. **Install PostgreSQL:**
   ```bash
   # macOS
   brew install postgresql
   brew services start postgresql
   
   # Ubuntu
   sudo apt-get install postgresql postgresql-contrib
   sudo systemctl start postgresql
   ```

2. **Set up environment variables:**
   ```bash
   # Copy the example environment file
   cp env.example .env
   
   # Edit .env with your settings
   nano .env
   ```

3. **Create the database:**
   ```bash
   createdb collaborative_docs
   ```

4. **Run migrations:**
   ```bash
   cargo install sqlx-cli
   sqlx database create
   sqlx migrate run
   ```

5. **Start the server:**
   ```bash
   cargo run
   ```

## Configuration

The application supports multiple configuration sources with the following precedence (highest to lowest):

1. **Environment variables** (highest priority)
2. **Configuration files** (`config/default.toml`, `config/{RUN_MODE}.toml`)
3. **Default values** (lowest priority)

### Environment Variables

#### Individual Database Settings
```bash
# Database configuration
APP__DATABASE__HOST=localhost
APP__DATABASE__PORT=5432
APP__DATABASE__USERNAME=postgres
APP__DATABASE__PASSWORD=your_secure_password
APP__DATABASE__DATABASE=collaborative_docs
APP__DATABASE__MAX_CONNECTIONS=10
APP__DATABASE__MIN_CONNECTIONS=2
```

#### Single DATABASE_URL (Alternative)
```bash
# Single database URL (takes precedence over individual settings)
DATABASE_URL=postgresql://username:password@host:port/database
```

#### Server Configuration
```bash
# Server settings
APP__SERVER__HOST=0.0.0.0
APP__SERVER__PORT=3000
```

#### CORS Configuration
```bash
# CORS settings
APP__CORS__ALLOWED_ORIGINS=["http://localhost:5173","https://yourdomain.com"]
APP__CORS__ALLOWED_METHODS=["GET","POST","PUT","DELETE"]
```

#### Application Mode
```bash
# Set application mode
RUN_MODE=development  # or staging, production
RUST_LOG=info        # Logging level
```

### Configuration Files

The application looks for configuration files in the `config/` directory:

- `config/default.toml` - Default configuration
- `config/development.toml` - Development-specific settings
- `config/production.toml` - Production-specific settings

### Production Deployment

#### Using Docker Compose

1. **Set environment variables:**
   ```bash
   export POSTGRES_PASSWORD="your_secure_password"
   export CORS_ORIGINS='["https://yourdomain.com"]'
   ```

2. **Start production services:**
   ```bash
   docker-compose -f docker-compose.prod.yml up -d
   ```

#### Manual Production Setup

1. **Create production environment file:**
   ```bash
   cp env.example .env.production
   ```

2. **Edit production settings:**
   ```bash
   RUN_MODE=production
   APP__DATABASE__PASSWORD=your_secure_password
   APP__CORS__ALLOWED_ORIGINS=["https://yourdomain.com"]
   RUST_LOG=warn
   ```

3. **Start with production config:**
   ```bash
   RUN_MODE=production cargo run
   ```

### Cloud Platform Deployment

For cloud platforms (Heroku, Railway, etc.), use the `DATABASE_URL` environment variable:

```bash
DATABASE_URL=postgresql://username:password@host:port/database
```

The application will automatically parse this URL and configure the database connection.

## API Endpoints

### Create Document
```http
POST /api/doc
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Get Document
```http
GET /api/doc/{id}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "Document content",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### Update Document
```http
PUT /api/doc/{id}
Content-Type: application/json

{
  "content": "Updated content"
}
```

### Get Document History
```http
GET /api/doc/{id}/history
```

**Response:**
```json
[
  {
    "timestamp": "2024-01-01T00:00:00Z",
    "ip_address": "127.0.0.1",
    "content": "Previous content"
  }
]
```

### Get Document Stats
```http
GET /api/doc/{id}/stats
```

**Response:**
```json
{
  "history_count": 5,
  "last_updated": "2024-01-01T00:00:00Z"
}
```

### Search Documents
```http
GET /api/search?q=search_term
```

## Development

### Running Tests
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_test
```

### Database Migrations
```bash
# Create a new migration
sqlx migrate add migration_name

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Configuration Testing
```bash
# Test configuration loading
cargo run -- --config-test

# Run with specific environment
RUN_MODE=development cargo run
```

## Production Deployment

### Docker Deployment
```bash
# Development
docker-compose up -d

# Production
docker-compose -f docker-compose.prod.yml up -d
```

### Manual Deployment
1. Set up PostgreSQL with proper security
2. Configure environment variables
3. Run migrations
4. Start the application with a process manager (systemd, supervisor, etc.)

### Security Considerations
- Use strong database passwords
- Configure proper CORS origins
- Set up rate limiting
- Use HTTPS in production
- Implement authentication/authorization
- Regular database backups
- Use environment variables for secrets
- Validate all configuration values

## Architecture

```
src/
├── main.rs          # Application entry point
├── config.rs        # Configuration management
├── database.rs      # Database operations
├── error.rs         # Custom error types
└── models.rs        # Data structures

config/              # Configuration files
├── default.toml     # Default configuration
├── development.toml # Development settings
└── production.toml  # Production settings

migrations/          # Database migrations
tests/              # Integration tests
```

## Performance

- **Connection Pooling**: SQLx provides efficient connection pooling
- **Indexes**: Proper database indexing for fast queries
- **Transactions**: ACID-compliant operations
- **Async/Await**: Non-blocking I/O operations
- **Configuration Caching**: Efficient configuration loading

## Monitoring

The application includes structured logging with tracing:

```bash
# Set log level
RUST_LOG=info cargo run

# Production logging
RUST_LOG=warn cargo run
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details. 