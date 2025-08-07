# Collaborative Docs

A real-time collaborative document editing platform built with modern web technologies (Rust backend, React + TypeScript frontend). Features live collaboration, version history, and a beautiful user interface.

![Collaborative Docs Screenshot](https://via.placeholder.com/800x400/2563eb/ffffff?text=Collaborative+Docs)

## 🚀 Features

- **Real-time Collaboration** - Live editing with WebSocket-based synchronization
- **Version History** - Complete document history with timestamps and user tracking
- **Beautiful UI** - Modern React interface with Tailwind CSS
- **User Authentication** - Secure login and registration system
- **Document Search** - Full-text search across all documents
- **Responsive Design** - Works on desktop and mobile devices
- **Production Ready** - Docker support, comprehensive testing, and monitoring

## 🏗️ Architecture

This project follows a modern microservices architecture:

```
collaborative-docs/
├── backend/          # Rust API server (Axum + PostgreSQL)
├── frontend/         # React + TypeScript + Vite
└── integration/      # End-to-end integration tests
```

### Tech Stack

**Backend:**
- **Rust** with Axum web framework
- **PostgreSQL** for data persistence
- **WebSocket** for real-time communication
- **JWT** for authentication
- **Docker** for containerization

**Frontend:**
- **React 19** with TypeScript
- **Vite** for fast development
- **Tailwind CSS** for styling
- **Radix UI** for accessible components
- **React Router** for navigation

**Testing:**
- **Jest** for integration tests
- **Vitest** for unit tests
- **Testing Library** for component testing

## 🛠️ Quick Start

### Prerequisites

- **Rust** (latest stable)
- **Node.js** (18+) and **pnpm**
- **PostgreSQL** (13+)
- **Docker** (optional, for containerized setup)

### Option 1: Docker Setup (Recommended)

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/collaborative-docs.git
   cd collaborative-docs
   ```

2. **Start all services:**
   ```bash
   docker-compose up -d
   ```

3. **Access the application:**
   - Frontend: http://localhost:5173
   - Backend API: http://localhost:3000

### Option 2: Manual Setup

1. **Start the backend:**
   ```bash
   cd backend
   cargo run --bin collaborative-docs-rs
   ```

2. **Start the frontend:**
   ```bash
   cd frontend
   pnpm install
   pnpm dev
   ```

3. **Run integration tests (optional):**
   ```bash
   cd integration
   pnpm install
   pnpm test
   ```

## 🚀 Deployment

For detailed deployment instructions, including port configuration, database setup, and reverse proxy configuration, see [DEPLOYMENT.md](DEPLOYMENT.md).

### Port Configuration

- **Backend**: Configurable via `APP__SERVER__PORT` environment variable or TOML config files
- **Frontend**: Configurable via `VITE_PORT` environment variable
- **Database**: Configurable via `APP__DATABASE__PORT` or `DATABASE_URL`

### Database Setup

The application supports multiple database setup options:
- Docker PostgreSQL (automatic with docker-compose)
- Manual PostgreSQL installation
- Cloud database services (AWS RDS, Google Cloud SQL, etc.)

See [DEPLOYMENT.md](DEPLOYMENT.md) for complete database setup instructions.

## 📁 Project Structure

```
collaborative-docs/
├── backend/                    # Rust API server
│   ├── src/
│   │   ├── main.rs           # Application entry point
│   │   ├── app.rs            # Router and middleware
│   │   ├── handlers.rs       # API endpoints
│   │   ├── database.rs       # Database operations
│   │   ├── models.rs         # Data structures
│   │   ├── config.rs         # Configuration
│   │   ├── auth.rs           # Authentication
│   │   ├── websocket.rs      # Real-time communication
│   │   └── crdt.rs           # Conflict resolution
│   ├── migrations/           # Database migrations
│   ├── config/              # Configuration files
│   └── tests/               # Backend tests
├── frontend/                  # React application
│   ├── src/
│   │   ├── components/       # React components
│   │   ├── pages/           # Page components
│   │   ├── services/        # API and WebSocket services
│   │   ├── contexts/        # React contexts
│   │   └── lib/             # Utility functions
│   └── public/              # Static assets
├── integration/               # End-to-end tests
│   ├── src/
│   │   ├── websocket-crdt.test.ts
│   │   └── websocket-deadlock.test.ts
│   └── jest.config.js
└── docker-compose.yml        # Docker orchestration
```

## 🔧 Configuration

### Environment Variables

Copy the example environment file and modify as needed:

```bash
# Copy example environment file
cp env.example .env

# Edit the file with your settings
nano .env
```

The environment file includes configuration for:
- Database connection settings
- Server port and host configuration
- Frontend dev server settings
- CORS configuration
- Docker settings
- Production deployment settings

See `env.example` for all available options and their descriptions.

### Backend Configuration

The backend uses TOML configuration files in `backend/config/`:

- `default.toml` - Base configuration
- `development.toml` - Development overrides
- `production.toml` - Production overrides

## 🧪 Testing

### Backend Tests
```bash
cd backend
cargo test
```

### Frontend Tests
```bash
cd frontend
pnpm test
```

### Integration Tests
```bash
cd integration
pnpm test
```

### All Tests
```bash
# From project root
pnpm test:all
```

## 🚀 Deployment

### Production with Docker

1. **Build and deploy:**
   ```bash
   docker-compose -f docker-compose.prod.yml up -d
   ```

2. **Environment variables for production:**
   ```bash
   RUN_MODE=production
   RUST_LOG=warn
   DATABASE_URL=postgresql://user:pass@host:5432/db
   ```

### Manual Deployment

1. **Backend:**
   ```bash
   cd backend
   cargo build --release
   ./target/release/collaborative-docs-rs
   ```

2. **Frontend:**
   ```bash
   cd frontend
   pnpm build
   # Serve the dist/ directory with your web server
   ```

## 📊 API Documentation

For comprehensive API documentation, see [API_DOCUMENTATION.md](API_DOCUMENTATION.md).

### Interactive Documentation

Access the interactive Swagger UI at:
- **Development**: `http://localhost:3000/swagger-ui`
- **Production**: `https://api.example.com/swagger-ui`

### Quick Reference

#### Authentication Endpoints
| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/auth/signup` | Register a new user |
| `POST` | `/api/auth/login` | Authenticate user |

#### Document Endpoints
| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/doc` | Create a new document |
| `GET` | `/api/doc/{id}` | Get document by ID |
| `PUT` | `/api/doc/{id}` | Update document content |
| `GET` | `/api/doc/{id}/history` | Get document version history |
| `GET` | `/api/doc/{id}/stats` | Get document statistics |
| `GET` | `/api/search?q=query` | Search documents |

#### CRDT Endpoints (Real-time Collaboration)
| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/doc/{id}/crdt/state` | Get CRDT state |
| `POST` | `/api/doc/{id}/crdt/update` | Apply CRDT update |

#### Admin Endpoints
| Method | Endpoint | Description |
|--------|----------|-------------|
| `PUT` | `/api/admin/users/{user_id}/role` | Update user role |

#### WebSocket Endpoints
| Endpoint | Description |
|----------|-------------|
| `ws://localhost:3000/ws/doc/{id}` | Real-time document collaboration |
| `GET /ws/info/{id}` | Get WebSocket connection info |

## 🔒 Security Features

- **JWT Authentication** - Secure token-based authentication
- **Input Validation** - Comprehensive request validation
- **CORS Protection** - Configurable cross-origin policies
- **SQL Injection Prevention** - Parameterized queries
- **Rate Limiting** - Protection against abuse

## 🐛 Troubleshooting

### Common Issues

1. **Database Connection Errors:**
   ```bash
   # Check PostgreSQL is running
   brew services list | grep postgresql
   
   # Create database if needed
   createdb collaborative_docs
   ```

2. **WebSocket Connection Issues:**
   ```bash
   # Check backend is running
   curl http://localhost:3000/api/doc
   
   # Check WebSocket endpoint
   curl http://localhost:3000/ws/info/test-doc
   ```

3. **Frontend Build Errors:**
   ```bash
   # Clear node modules and reinstall
   cd frontend
   rm -rf node_modules pnpm-lock.yaml
   pnpm install
   ```

### Debug Mode

Enable debug logging:

```bash
# Backend
RUST_LOG=debug cargo run

# Frontend
pnpm dev --debug
```

## 🤝 Contributing

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Make your changes** following the existing code style
4. **Add tests** for new functionality
5. **Run all tests** to ensure nothing breaks
6. **Commit your changes** (`git commit -m 'Add amazing feature'`)
7. **Push to the branch** (`git push origin feature/amazing-feature`)
8. **Open a Pull Request**

### Development Guidelines

- **Rust**: Follow Rust best practices and use `cargo fmt` and `cargo clippy`
- **TypeScript**: Use strict TypeScript configuration and ESLint
- **Testing**: Maintain high test coverage for all new features
- **Documentation**: Update README files for any new features

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Axum** - Modern Rust web framework
- **React** - UI library
- **Tailwind CSS** - Utility-first CSS framework
- **PostgreSQL** - Reliable database
- **WebSocket** - Real-time communication protocol

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/collaborative-docs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/collaborative-docs/discussions)
- **Email**: riyad@islahlabs.com

---

**Made with ❤️ by the Collaborative Docs team** 