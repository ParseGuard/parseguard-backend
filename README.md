# ParseGuard Backend

> Rust/Axum backend for ParseGuard - Operational risk monitoring platform

## 🚀 Tech Stack

- **Framework**: Axum 0.7 (async web framework)
- **Runtime**: Tokio (async runtime)
- **Database**: PostgreSQL with SQLx (compile-time verified queries)
- **Cache**: Redis
- **Auth**: JWT with bcrypt
- **AI**: OLLAMA integration for document parsing
- **Logging**: tracing + tracing-subscriber

## 📋 Prerequisites

- Rust 1.75+ ([Install](https://rustup.rs/))
- Docker & Docker Compose ([Install](https://docs.docker.com/get-docker/))
- OLLAMA ([Install](https://ollama.ai/))

## 🛠️ Quick Start

### 1. Start Database Services

```bash
# Start PostgreSQL and Redis
docker-compose up -d

# Verify services are running
docker-compose ps
```

### 2. Configure Environment

```bash
# Copy example env file
cp .env.example .env

# Edit .env with your settings (defaults work for local dev)
```

### 3. Run Database Migrations

```bash
# Install SQLx CLI (one-time setup)
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run
```

### 4. Start Development Server

```bash
# Run with auto-reload
cargo watch -x run

# Or standard run
cargo run
```

Server will start at `http://localhost:8000`

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Check code without building
cargo check
```

## 📁 Project Structure

```
parseguard-backend/
├── src/
│   ├── main.rs              # Application entry point
│   ├── config.rs            # Environment configuration
│   ├── error.rs             # Error handling
│   ├── api/                 # API route handlers
│   ├── db/                  # Database connection & queries
│   ├── models/              # Data models & DTOs
│   ├── middleware/          # Auth, logging, etc.
│   └── services/            # Business logic & external APIs
├── migrations/              # SQLx database migrations
├── docker-compose.yml       # Local development services
├── Cargo.toml              # Dependencies
└── .env.example            # Environment template
```

## 🔌 API Endpoints

### Health Check

```bash
curl http://localhost:8000/health
```

**Response:**
```json
{
  "status": "healthy",
  "service": "parseguard-backend",
  "version": "0.1.0"
}
```

## 🐳 Docker Commands

```bash
# Start services
docker-compose up -d

# Stop services
docker-compose down

# View logs
docker-compose logs -f

# Restart a service
docker-compose restart postgres

# Clean up volumes (WARNING: deletes data)
docker-compose down -v
```

## 🗄️ Database

### Connecting to PostgreSQL

```bash
# Using psql
docker exec -it parseguard-postgres psql -U postgres -d parseguard

# Using GUI tools
Host: localhost
Port: 5432
Database: parseguard
User: postgres
Password: postgres
```

### Creating Migrations

```bash
# Create a new migration
sqlx migrate add <migration_name>

# Example
sqlx migrate add create_users_table

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

## 🔧 Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://postgres:postgres@localhost:5432/parseguard` |
| `JWT_SECRET` | Secret for JWT signing | (required) |
| `OLLAMA_API_URL` | OLLAMA API endpoint | `http://localhost:11434` |
| `PORT` | Server port | `8000` |
| `RUST_LOG` | Logging level | `info,parseguard_backend=debug` |

## 🤖 OLLAMA Setup

```bash
# Install OLLAMA (if not already installed)
# Visit https://ollama.ai/

# Pull a model for document processing
ollama pull llama3.2

# Verify OLLAMA is running
curl http://localhost:11434/api/tags
```

## 🔗 Related Projects

- [parseguard-client](https://github.com/ParseGuard/parseguard-client) - React Router DOM v7 frontend

## 📄 License

MIT

---

*Built with ⚡ by the ParseGuard team*
