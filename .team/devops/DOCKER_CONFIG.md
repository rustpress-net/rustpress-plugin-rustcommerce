# Docker Configuration -- RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Owner**: DevOps Lead
**Status**: Approved

---

## 1. Overview

This document defines the Docker and container configuration for the RustCommerce plugin across three environments: development, testing, and production. All configurations use Docker Compose for orchestration and follow the principle of environment parity -- the development stack should mirror production as closely as possible.

---

## 2. Development Docker Compose

### 2.1 Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Docker Network: rustpress-dev       │
│                                                      │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────┐  │
│  │PostgreSQL │  │  Redis   │  │   RustPress Core  │  │
│  │  16       │  │  7       │  │  + RustCommerce   │  │
│  │  :5432    │  │  :6379   │  │  plugin  :8080    │  │
│  └──────────┘  └──────────┘  └───────────────────┘  │
│                                                      │
│  ┌──────────────────────────┐                        │
│  │   Adminer (DB UI)        │                        │
│  │   :8081                  │                        │
│  └──────────────────────────┘                        │
└─────────────────────────────────────────────────────┘
```

### 2.2 `docker-compose.dev.yml`

```yaml
version: "3.9"

name: rustcommerce-dev

services:
  # ────────────────────────────────────────────
  # PostgreSQL 16 — Primary data store
  # ────────────────────────────────────────────
  postgres:
    image: postgres:16-alpine
    container_name: rustcommerce-postgres
    restart: unless-stopped
    ports:
      - "5432:5432"
    environment:
      POSTGRES_DB: rustpress
      POSTGRES_USER: rustpress
      POSTGRES_PASSWORD: rustpress_dev
      PGDATA: /var/lib/postgresql/data/pgdata
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./docker/init-db.sql:/docker-entrypoint-initdb.d/01-init.sql:ro
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U rustpress -d rustpress"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 30s
    networks:
      - rustpress-dev

  # ────────────────────────────────────────────
  # Redis 7 — Cache + session store + cart
  # ────────────────────────────────────────────
  redis:
    image: redis:7-alpine
    container_name: rustcommerce-redis
    restart: unless-stopped
    ports:
      - "6379:6379"
    command: redis-server --appendonly yes --maxmemory 256mb --maxmemory-policy allkeys-lru
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - rustpress-dev

  # ────────────────────────────────────────────
  # RustPress Core — CMS platform with plugin
  # ────────────────────────────────────────────
  rustpress:
    build:
      context: ../rustpress-core-base
      dockerfile: Dockerfile
    container_name: rustcommerce-rustpress
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      RUST_LOG: info,rustcommerce=debug,rustpress=debug
      DATABASE_URL: postgres://rustpress:rustpress_dev@postgres:5432/rustpress
      REDIS_URL: redis://redis:6379
      SERVER_HOST: 0.0.0.0
      SERVER_PORT: "8080"
      JWT_SECRET: dev-jwt-secret-do-not-use-in-production
      # RustCommerce-specific
      STRIPE_SECRET_KEY: ${STRIPE_SECRET_KEY:-sk_test_placeholder}
      STRIPE_PUBLISHABLE_KEY: ${STRIPE_PUBLISHABLE_KEY:-pk_test_placeholder}
      STRIPE_WEBHOOK_SECRET: ${STRIPE_WEBHOOK_SECRET:-whsec_test_placeholder}
      RUSTCOMMERCE_STORE_NAME: "Dev Store"
      RUSTCOMMERCE_CURRENCY: USD
    volumes:
      - ../rustpress-plugin-rustcommerce:/app/plugins/rustcommerce:ro
      - rustpress_uploads:/app/uploads
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    networks:
      - rustpress-dev

  # ────────────────────────────────────────────
  # Adminer — Database management UI
  # ────────────────────────────────────────────
  adminer:
    image: adminer:latest
    container_name: rustcommerce-adminer
    restart: unless-stopped
    ports:
      - "8081:8080"
    environment:
      ADMINER_DEFAULT_SERVER: postgres
      ADMINER_DESIGN: dracula
    depends_on:
      - postgres
    networks:
      - rustpress-dev

volumes:
  postgres_data:
    driver: local
  redis_data:
    driver: local
  rustpress_uploads:
    driver: local

networks:
  rustpress-dev:
    driver: bridge
```

### 2.3 Database Initialization Script (`docker/init-db.sql`)

```sql
-- Create the rustcommerce schema namespace
-- Migrations will create tables, but we set up extensions here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create a test database for running integration tests locally
CREATE DATABASE rustcommerce_test;
GRANT ALL PRIVILEGES ON DATABASE rustcommerce_test TO rustpress;

-- Connect to test DB and enable extensions
\c rustcommerce_test;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
```

---

## 3. Test Docker Compose

### 3.1 Design Goals

- **Ephemeral**: Containers are created, used, and destroyed per test run.
- **Isolated**: No persistent volumes; every run starts with a clean state.
- **Fast**: Uses `tmpfs` for PostgreSQL data to avoid disk I/O.
- **CI-compatible**: Works in GitHub Actions service containers and locally.

### 3.2 `docker-compose.test.yml`

```yaml
version: "3.9"

name: rustcommerce-test

services:
  # ────────────────────────────────────────────
  # Ephemeral PostgreSQL for integration tests
  # ────────────────────────────────────────────
  postgres-test:
    image: postgres:16-alpine
    container_name: rustcommerce-postgres-test
    ports:
      - "5433:5432"
    environment:
      POSTGRES_DB: rustcommerce_test
      POSTGRES_USER: rustcommerce
      POSTGRES_PASSWORD: test_password
      PGDATA: /var/lib/postgresql/data/pgdata
    tmpfs:
      - /var/lib/postgresql/data:rw,size=512m
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U rustcommerce -d rustcommerce_test"]
      interval: 5s
      timeout: 3s
      retries: 10
      start_period: 10s
    networks:
      - rustpress-test

  # ────────────────────────────────────────────
  # Ephemeral Redis for integration tests
  # ────────────────────────────────────────────
  redis-test:
    image: redis:7-alpine
    container_name: rustcommerce-redis-test
    ports:
      - "6380:6379"
    command: redis-server --save "" --appendonly no
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 10
    networks:
      - rustpress-test

networks:
  rustpress-test:
    driver: bridge
```

### 3.3 Running Tests Locally

```bash
# Start ephemeral test infrastructure
docker compose -f docker-compose.test.yml up -d --wait

# Run migrations against test database
DATABASE_URL="postgres://rustcommerce:test_password@localhost:5433/rustcommerce_test" \
  sqlx migrate run

# Run integration tests
DATABASE_URL="postgres://rustcommerce:test_password@localhost:5433/rustcommerce_test" \
REDIS_URL="redis://localhost:6380" \
STRIPE_SECRET_KEY="sk_test_placeholder" \
STRIPE_WEBHOOK_SECRET="whsec_test_placeholder" \
  cargo test --all-features --test '*'

# Tear down (removes containers and ephemeral data)
docker compose -f docker-compose.test.yml down -v
```

---

## 4. Production Dockerfile

### 4.1 Design Goals

- **Multi-stage build**: Separate build and runtime stages for minimal image size.
- **Non-root user**: Run as an unprivileged user in production.
- **Minimal image**: Use `debian:bookworm-slim` as the runtime base (glibc required by Rust binaries).
- **Health check**: Built-in container health check.

### 4.2 `Dockerfile`

```dockerfile
# ============================================================
# Stage 1: Build
# ============================================================
FROM rust:1.82-bookworm AS builder

WORKDIR /build

# Cache dependency compilation
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "pub fn init() {}" > src/lib.rs
RUN cargo build --release --lib 2>/dev/null || true
RUN rm -rf src

# Build actual source
COPY src/ src/
COPY migrations/ migrations/
COPY plugin.toml ./
RUN touch src/lib.rs && cargo build --release --lib

# Install sqlx-cli for migration running
RUN cargo install sqlx-cli --no-default-features --features postgres

# ============================================================
# Stage 2: Runtime
# ============================================================
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r rustcommerce && useradd -r -g rustcommerce rustcommerce

WORKDIR /plugin

# Copy built artifacts
COPY --from=builder /build/target/release/librustcommerce.so ./lib/
COPY --from=builder /build/migrations/ ./migrations/
COPY --from=builder /build/plugin.toml ./
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx

# Copy any static assets (admin UI built files, etc.)
COPY --chown=rustcommerce:rustcommerce admin-ui/dist/ ./admin-ui/dist/ 2>/dev/null || true

# Set ownership
RUN chown -R rustcommerce:rustcommerce /plugin

USER rustcommerce

# Health check — plugin exposes a health endpoint through RustPress
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:8080/api/v1/rustcommerce/health || exit 1

# Default environment
ENV RUST_LOG=info,rustcommerce=info

# The plugin is loaded by RustPress core, not run standalone
# This entrypoint runs migrations then signals readiness
ENTRYPOINT ["sh", "-c", "sqlx migrate run && echo 'Migrations complete. Plugin ready.' && sleep infinity"]
```

### 4.3 Production Docker Compose (`docker-compose.prod.yml`)

```yaml
version: "3.9"

name: rustcommerce-prod

services:
  postgres:
    image: postgres:16-alpine
    restart: always
    ports:
      - "127.0.0.1:5432:5432"
    environment:
      POSTGRES_DB: ${POSTGRES_DB}
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_prod_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER}"]
      interval: 30s
      timeout: 10s
      retries: 5
    deploy:
      resources:
        limits:
          memory: 1G
          cpus: "1.0"
    networks:
      - rustpress-prod

  redis:
    image: redis:7-alpine
    restart: always
    ports:
      - "127.0.0.1:6379:6379"
    command: >
      redis-server
      --appendonly yes
      --maxmemory 512mb
      --maxmemory-policy allkeys-lru
      --requirepass ${REDIS_PASSWORD}
    volumes:
      - redis_prod_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "-a", "${REDIS_PASSWORD}", "ping"]
      interval: 30s
      timeout: 10s
      retries: 5
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: "0.5"
    networks:
      - rustpress-prod

volumes:
  postgres_prod_data:
    driver: local
  redis_prod_data:
    driver: local

networks:
  rustpress-prod:
    driver: bridge
```

---

## 5. Environment Variable Documentation

### 5.1 Complete Variable Reference

#### Core Infrastructure

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | -- | PostgreSQL connection string. Format: `postgres://user:pass@host:port/dbname` |
| `REDIS_URL` | Yes | -- | Redis connection string. Format: `redis://[:password@]host:port` |
| `SERVER_HOST` | No | `0.0.0.0` | Bind address for the HTTP server |
| `SERVER_PORT` | No | `8080` | Port for the HTTP server |
| `RUST_LOG` | No | `info` | Log level filter (e.g., `info,rustcommerce=debug`) |

#### Authentication

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `JWT_SECRET` | Yes | -- | Secret key for JWT token signing. Must be at least 32 characters. |

#### Stripe Payment Gateway

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `STRIPE_SECRET_KEY` | Yes | -- | Stripe API secret key. Use `sk_test_*` for dev/test. |
| `STRIPE_PUBLISHABLE_KEY` | Yes | -- | Stripe publishable key. Use `pk_test_*` for dev/test. |
| `STRIPE_WEBHOOK_SECRET` | Yes | -- | Stripe webhook endpoint signing secret (`whsec_*`). |

#### RustCommerce Store Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `RUSTCOMMERCE_STORE_NAME` | No | `My Store` | Store display name |
| `RUSTCOMMERCE_CURRENCY` | No | `USD` | Default store currency (ISO 4217) |
| `RUSTCOMMERCE_TAX_INCLUSIVE` | No | `false` | Whether prices include tax |
| `RUSTCOMMERCE_LOW_STOCK_THRESHOLD` | No | `5` | Inventory count that triggers low-stock alerts |
| `RUSTCOMMERCE_CART_EXPIRY_HOURS` | No | `72` | Hours before abandoned carts are cleaned up |
| `RUSTCOMMERCE_STOCK_RESERVATION_MINUTES` | No | `10` | Minutes to hold stock during checkout |

#### PostgreSQL (Docker Compose only)

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `POSTGRES_DB` | Yes | -- | Database name |
| `POSTGRES_USER` | Yes | -- | Database user |
| `POSTGRES_PASSWORD` | Yes | -- | Database password |
| `REDIS_PASSWORD` | No | -- | Redis AUTH password (production only) |

### 5.2 Environment File Templates

#### `.env.development`

```env
# Infrastructure
DATABASE_URL=postgres://rustpress:rustpress_dev@localhost:5432/rustpress
REDIS_URL=redis://localhost:6379
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=debug,rustcommerce=trace

# Auth
JWT_SECRET=dev-jwt-secret-minimum-32-characters-long

# Stripe (test mode)
STRIPE_SECRET_KEY=sk_test_your_test_key_here
STRIPE_PUBLISHABLE_KEY=pk_test_your_test_key_here
STRIPE_WEBHOOK_SECRET=whsec_your_test_webhook_secret

# Store
RUSTCOMMERCE_STORE_NAME=Dev Store
RUSTCOMMERCE_CURRENCY=USD
RUSTCOMMERCE_TAX_INCLUSIVE=false
RUSTCOMMERCE_LOW_STOCK_THRESHOLD=5
```

#### `.env.test`

```env
# Infrastructure (note: port 5433 for test PostgreSQL)
DATABASE_URL=postgres://rustcommerce:test_password@localhost:5433/rustcommerce_test
REDIS_URL=redis://localhost:6380
RUST_LOG=info

# Auth
JWT_SECRET=test-jwt-secret-minimum-32-characters-long

# Stripe (test mode — use fixtures/mocks)
STRIPE_SECRET_KEY=sk_test_fake_key_for_testing
STRIPE_PUBLISHABLE_KEY=pk_test_fake_key_for_testing
STRIPE_WEBHOOK_SECRET=whsec_test_fake_secret_for_testing

# Store
RUSTCOMMERCE_STORE_NAME=Test Store
RUSTCOMMERCE_CURRENCY=USD
```

#### `.env.production` (template -- actual values from secrets manager)

```env
# Infrastructure
DATABASE_URL=postgres://produser:${DB_PASSWORD}@db.internal:5432/rustpress
REDIS_URL=redis://:${REDIS_PASSWORD}@cache.internal:6379
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=info,rustcommerce=info

# Auth
JWT_SECRET=${JWT_SECRET_FROM_VAULT}

# Stripe (live mode)
STRIPE_SECRET_KEY=${STRIPE_LIVE_SECRET_KEY}
STRIPE_PUBLISHABLE_KEY=${STRIPE_LIVE_PUBLISHABLE_KEY}
STRIPE_WEBHOOK_SECRET=${STRIPE_LIVE_WEBHOOK_SECRET}

# Store
RUSTCOMMERCE_STORE_NAME=My Store
RUSTCOMMERCE_CURRENCY=USD
RUSTCOMMERCE_TAX_INCLUSIVE=false
RUSTCOMMERCE_LOW_STOCK_THRESHOLD=10
RUSTCOMMERCE_CART_EXPIRY_HOURS=48
RUSTCOMMERCE_STOCK_RESERVATION_MINUTES=15
```

---

## 6. Docker Commands Reference

### Development

```bash
# Start all development services
docker compose -f docker-compose.dev.yml up -d

# View logs
docker compose -f docker-compose.dev.yml logs -f rustpress

# Restart after code changes
docker compose -f docker-compose.dev.yml restart rustpress

# Stop all services
docker compose -f docker-compose.dev.yml down

# Stop and remove all data (clean slate)
docker compose -f docker-compose.dev.yml down -v
```

### Testing

```bash
# Start test infrastructure, run tests, tear down
docker compose -f docker-compose.test.yml up -d --wait
cargo test --all-features
docker compose -f docker-compose.test.yml down -v
```

### Production

```bash
# Start production stack
docker compose -f docker-compose.prod.yml --env-file .env.production up -d

# Rolling restart (zero-downtime with multiple replicas)
docker compose -f docker-compose.prod.yml up -d --no-deps rustpress

# Backup database
docker exec rustcommerce-postgres pg_dump -U ${POSTGRES_USER} ${POSTGRES_DB} > backup.sql
```

---

## 7. Security Considerations

- **No secrets in images**: All secrets are injected via environment variables at runtime.
- **Non-root containers**: All application containers run as non-root users.
- **Network isolation**: Production services bind to `127.0.0.1` only; external access through a reverse proxy.
- **Image scanning**: Use `docker scout` or Trivy to scan images for vulnerabilities before deployment.
- **Read-only filesystem**: Where possible, mount application code as read-only (`:ro`).
- **.env files**: Never committed to version control. Add `.env*` to `.gitignore` (except `.env.example`).
