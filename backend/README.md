# Collaborative Docs Backend

A production-ready Rust backend for collaborative document editing with PostgreSQL, built with Axum and following Rust best practices.

## Features

- ✅ **PostgreSQL Database** - Production-ready with proper indexing and transactions
- ✅ **Modular Architecture** - Clean separation of concerns with dedicated modules
- ✅ **Comprehensive Error Handling** - Custom error types with proper HTTP responses
- ✅ **Input Validation** - Request validation with detailed error messages
- ✅ **Structured Logging** - Tracing-based logging for debugging and monitoring
- ✅ **TOML Configuration** - Type-safe configuration with environment overrides
- ✅ **CORS Support** - Configurable CORS for frontend integration
- ✅ **Document History** - Full version history with timestamps and IP tracking
- ✅ **Search Functionality** - PostgreSQL-powered full-text search
- ✅ **Docker Support** - Complete Docker setup for development and production
- ✅ **Comprehensive Testing** - Unit and integration tests

## Project Structure

```
backend/
├── src/
│   ├── main.rs          # Application entry point
│   ├── app.rs           # Router and application setup
│   ├── handlers.rs      # API endpoint handlers
│   ├── database.rs      # Database operations
│   ├── models.rs        # Data structures
│   ├── config.rs        # Configuration management
│   ├── error.rs         # Custom error types
│   └── tests.rs         # Unit tests
├── config/
│   ├── default.toml     # Default configuration
│   ├── development.toml # Development overrides
│   └── production.toml  # Production overrides
├── migrations/          # Database migrations
└── tests/              # Integration tests
```

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

2. **Create the database and user:**
   ```bash
   # Connect to PostgreSQL
   psql -U postgres
   
   # Create database and user
   CREATE DATABASE collaborative_docs;
   CREATE USER collaborative_user WITH PASSWORD 'collaborative_password';
   GRANT ALL PRIVILEGES ON DATABASE collaborative_docs TO collaborative_user;
   \q
   ```

3. **Run migrations:**
   ```bash
   cargo install sqlx-cli
   sqlx database create
   sqlx migrate run
   ```

4. **Start the server:**
   ```bash
   cargo run --bin collaborative-docs-rs
   ```

## Configuration

The application uses a **TOML-first configuration approach** with the following precedence (highest to lowest):

1. **Environment variables** (for overrides)
2. **Configuration files** (`config/default.toml`, `config/{RUN_MODE}.toml`)
3. **Default values** (lowest priority)

### Configuration Files

#### `config/default.toml`
```toml
[server]
host = "0.0.0.0"
port = 3000

[database]
host = "localhost"
port = 5432
username = "postgres"
password = "password"
database = "collaborative_docs"
max_connections = 10
min_connections = 2

[cors]
allowed_origins = [
    "http://localhost:5173",
    "http://localhost:3000"
]
allowed_methods = [
    "GET",
    "POST", 
    "PUT",
    "DELETE"
]
```

#### `config/development.toml`
```toml
[database]
database = "collaborative_docs_dev"

[cors]
allowed_origins = [
    "http://localhost:5173",
    "http://localhost:3000",
    "http://127.0.0.1:5173"
]
```

#### `config/production.toml`
```toml
[database]
password = "CHANGE_ME_IN_PRODUCTION"

[cors]
allowed_origins = [
    "https://yourdomain.com"
]
```

### Environment Variables

You can override any configuration value using environment variables:

```bash
# Set the run mode (determines which TOML file to load)
RUN_MODE=development

# Override individual settings
APP__SERVER__PORT=8080
APP__DATABASE__PASSWORD=my_secure_password

# Use a single DATABASE_URL (for cloud platforms)
DATABASE_URL=postgresql://username:password@host:port/database

# Set logging level
RUST_LOG=info
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/doc` | Create a new document |
| `GET` | `/api/doc/{id}` | Get a document by ID |
| `PUT` | `/api/doc/{id}` | Update a document's content |
| `GET` | `/api/doc/{id}/history` | Get document version history |
| `GET` | `/api/doc/{id}/stats` | Get document statistics |
| `GET` | `/api/search?q=query` | Search documents by content |

### Example Usage

```bash
# Create a document
curl -X POST http://localhost:3000/api/doc

# Get a document
curl http://localhost:3000/api/doc/{document_id}

# Update a document
curl -X PUT http://localhost:3000/api/doc/{document_id} \
  -H "Content-Type: application/json" \
  -d '{"content": "Hello, World!"}'

# Get document history
curl http://localhost:3000/api/doc/{document_id}/history

# Search documents
curl "http://localhost:3000/api/search?q=hello"
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_create_document
```

### Code Structure

The codebase follows Rust best practices with clear separation of concerns:

- **`main.rs`** - Clean entry point, handles startup and logging
- **`app.rs`** - Router configuration and middleware setup
- **`handlers.rs`** - All API endpoint handlers
- **`database.rs`** - Database operations and connection management
- **`models.rs`** - Data structures and validation
- **`config.rs`** - Configuration loading and validation
- **`error.rs`** - Custom error types and HTTP responses
- **`tests.rs`** - Unit tests for all functionality

### Adding New Features

1. **Add new models** in `src/models.rs`
2. **Add database operations** in `src/database.rs`
3. **Add handlers** in `src/handlers.rs`
4. **Add routes** in `src/app.rs`
5. **Add tests** in `src/tests.rs`

## Production Deployment

### Using Docker

```bash
# Build and run with Docker Compose
docker-compose -f docker-compose.prod.yml up -d

# Or build manually
docker build -t collaborative-docs .
docker run -p 3000:3000 collaborative-docs
```

### Environment Variables for Production

```bash
RUN_MODE=production
RUST_LOG=warn
DATABASE_URL=postgresql://user:pass@host:5432/db
```

### Database Setup

```bash
# Create production database
createdb collaborative_docs_prod

# Run migrations
sqlx migrate run

# Optional: Create read-only user for analytics
CREATE USER readonly_user WITH PASSWORD 'readonly_password';
GRANT CONNECT ON DATABASE collaborative_docs_prod TO readonly_user;
GRANT USAGE ON SCHEMA public TO readonly_user;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO readonly_user;
```

## Monitoring and Logging

The application uses structured logging with different levels:

```bash
# Set logging level
RUST_LOG=debug  # Most verbose
RUST_LOG=info   # Default
RUST_LOG=warn   # Production
RUST_LOG=error  # Errors only
```

### Health Checks

```bash
# Check if server is running
curl http://localhost:3000/api/doc

# Check database connectivity
# (The application will log connection status on startup)
```

## Contributing

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Make your changes** following the existing code structure
4. **Add tests** for new functionality
5. **Run tests** (`cargo test`)
6. **Commit your changes** (`git commit -m 'Add amazing feature'`)
7. **Push to the branch** (`git push origin feature/amazing-feature`)
8. **Open a Pull Request**

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 