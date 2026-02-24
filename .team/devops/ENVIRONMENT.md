# Environment Setup -- RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Owner**: DevOps Lead
**Status**: Approved

---

## 1. Overview

This document describes the complete environment setup required to develop, test, and run the RustCommerce plugin. It covers system prerequisites, toolchain installation, environment variable configuration, local development workflow, and testing with Stripe test mode.

---

## 2. Development Environment Requirements

### 2.1 System Prerequisites

| Requirement | Minimum Version | Recommended | Notes |
|-------------|----------------|-------------|-------|
| **Operating System** | Linux, macOS, Windows 10+ | Ubuntu 22.04 / macOS 14+ | WSL2 recommended on Windows |
| **RAM** | 8 GB | 16 GB | Rust compilation is memory-intensive |
| **Disk Space** | 10 GB free | 20 GB free | Cargo target directory can grow large |
| **CPU** | 4 cores | 8 cores | Faster compilation with more cores |

### 2.2 Required Toolchain

#### Rust

| Tool | Version | Install Command |
|------|---------|----------------|
| Rust (stable) | 1.80+ (MSRV) | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| rustfmt | (bundled) | `rustup component add rustfmt` |
| clippy | (bundled) | `rustup component add clippy` |
| cargo-watch | Latest | `cargo install cargo-watch` |
| cargo-audit | Latest | `cargo install cargo-audit` |
| sqlx-cli | Latest | `cargo install sqlx-cli --no-default-features --features postgres` |

#### Node.js (for admin UI components)

| Tool | Version | Install Command |
|------|---------|----------------|
| Node.js | 20 LTS | Via nvm: `nvm install 20` |
| npm | 10+ | Bundled with Node.js 20 |

#### Infrastructure

| Tool | Version | Install Command |
|------|---------|----------------|
| Docker | 24+ | https://docs.docker.com/get-docker/ |
| Docker Compose | 2.20+ | Bundled with Docker Desktop |
| PostgreSQL client | 16 | `apt install postgresql-client-16` or `brew install postgresql@16` |
| Redis CLI | 7+ | `apt install redis-tools` or `brew install redis` |

#### Optional (Recommended)

| Tool | Purpose | Install |
|------|---------|---------|
| `cargo-tarpaulin` | Code coverage | `cargo install cargo-tarpaulin` |
| `cargo-nextest` | Faster test runner | `cargo install cargo-nextest` |
| `just` | Task runner | `cargo install just` |
| `stripe` CLI | Webhook forwarding | https://stripe.com/docs/stripe-cli |

### 2.3 IDE Setup

**Recommended**: VS Code with the following extensions:

| Extension | Purpose |
|-----------|---------|
| `rust-analyzer` | Rust language server (IntelliSense, diagnostics, formatting) |
| `Even Better TOML` | TOML file support (Cargo.toml) |
| `crates` | Cargo dependency version info |
| `Error Lens` | Inline error display |
| `REST Client` | API endpoint testing |
| `Docker` | Docker file support |
| `ESLint` | Frontend linting |
| `Tailwind CSS IntelliSense` | Tailwind class completion |

**Recommended `settings.json`**:
```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": ["--all-features"],
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

---

## 3. Required Environment Variables

### 3.1 Variable Reference

All environment variables required by the RustCommerce plugin, organized by category.

#### Database

| Variable | Required | Example Value | Description |
|----------|----------|---------------|-------------|
| `DATABASE_URL` | Yes | `postgres://rustpress:rustpress_dev@localhost:5432/rustpress` | Full PostgreSQL connection URL |
| `SQLX_OFFLINE` | No | `true` | When set, sqlx uses cached query metadata instead of a live database for compile-time checking |

#### Cache / Sessions

| Variable | Required | Example Value | Description |
|----------|----------|---------------|-------------|
| `REDIS_URL` | Yes | `redis://localhost:6379` | Redis connection URL |

#### Server

| Variable | Required | Example Value | Description |
|----------|----------|---------------|-------------|
| `SERVER_HOST` | No | `0.0.0.0` | HTTP server bind address (default: `0.0.0.0`) |
| `SERVER_PORT` | No | `8080` | HTTP server port (default: `8080`) |
| `RUST_LOG` | No | `debug,rustcommerce=trace` | Log level filter |
| `RUST_BACKTRACE` | No | `1` | Enable backtraces on panic |

#### Authentication

| Variable | Required | Example Value | Description |
|----------|----------|---------------|-------------|
| `JWT_SECRET` | Yes | `your-secret-key-at-least-32-chars` | JWT signing secret (minimum 32 characters) |

#### Stripe Payment Gateway

| Variable | Required | Example Value | Description |
|----------|----------|---------------|-------------|
| `STRIPE_SECRET_KEY` | Yes | `sk_test_51ABC...` | Stripe secret API key |
| `STRIPE_PUBLISHABLE_KEY` | Yes | `pk_test_51ABC...` | Stripe publishable key (for frontend) |
| `STRIPE_WEBHOOK_SECRET` | Yes | `whsec_abc123...` | Stripe webhook signing secret |

#### Store Configuration

| Variable | Required | Example Value | Description |
|----------|----------|---------------|-------------|
| `RUSTCOMMERCE_STORE_NAME` | No | `My Store` | Display name for the store |
| `RUSTCOMMERCE_CURRENCY` | No | `USD` | Default currency (ISO 4217 code) |
| `RUSTCOMMERCE_TAX_INCLUSIVE` | No | `false` | Whether displayed prices include tax |
| `RUSTCOMMERCE_LOW_STOCK_THRESHOLD` | No | `5` | Quantity that triggers low-stock alerts |
| `RUSTCOMMERCE_CART_EXPIRY_HOURS` | No | `72` | Hours until abandoned carts are purged |
| `RUSTCOMMERCE_STOCK_RESERVATION_MINUTES` | No | `10` | Minutes to hold reserved stock during checkout |

### 3.2 Quick Setup

Create a `.env` file in the project root (this file is gitignored):

```bash
cp .env.example .env
# Edit .env with your Stripe test keys and preferences
```

---

## 4. Local Development Workflow

### 4.1 First-Time Setup

```bash
# 1. Clone the repository
git clone https://github.com/rustpress-net/rustpress-plugin-rustcommerce.git
cd rustpress-plugin-rustcommerce

# 2. Install Rust toolchain and components
rustup update stable
rustup component add rustfmt clippy

# 3. Install development tools
cargo install cargo-watch sqlx-cli cargo-audit

# 4. Copy and configure environment
cp .env.example .env
# Edit .env — add your Stripe test keys (see Section 5)

# 5. Start infrastructure (PostgreSQL + Redis)
docker compose -f docker-compose.dev.yml up -d postgres redis

# 6. Wait for PostgreSQL to be healthy
docker compose -f docker-compose.dev.yml exec postgres pg_isready -U rustpress

# 7. Run database migrations
sqlx migrate run

# 8. Prepare sqlx offline metadata (for compile-time checked queries)
cargo sqlx prepare

# 9. Verify everything compiles
cargo check --all-features

# 10. Run tests
cargo test --all-features
```

### 4.2 Daily Development Loop

```bash
# Start infrastructure if not running
docker compose -f docker-compose.dev.yml up -d postgres redis

# Option A: Manual compile-and-run cycle
cargo build
cargo run  # or run via RustPress core with plugin loaded

# Option B: Auto-recompile on save (recommended)
cargo watch -x check -x test -x run

# Option C: Just check and test on save
cargo watch -x 'check --all-features' -x 'test --all-features'
```

### 4.3 Running the Full Stack

To run RustCommerce alongside RustPress core:

```bash
# Start all services including RustPress core
docker compose -f docker-compose.dev.yml up -d

# Or, run RustPress core from source with plugin path
cd ../rustpress-core-base
PLUGIN_DIR=../rustpress-plugin-rustcommerce cargo run

# Access points:
# - RustPress:      http://localhost:8080
# - Admin UI:       http://localhost:8080/admin
# - Commerce API:   http://localhost:8080/api/v1/rustcommerce/
# - Adminer (DB):   http://localhost:8081
```

### 4.4 Working with Migrations

```bash
# Create a new migration
sqlx migrate add <description>
# e.g., sqlx migrate add add_coupon_usage_limit

# Run pending migrations
sqlx migrate run

# Revert the last migration
sqlx migrate revert

# Check migration status
sqlx migrate info

# After changing any sqlx queries, regenerate offline metadata
cargo sqlx prepare
```

### 4.5 Code Quality Commands

```bash
# Format code
cargo fmt --all

# Run clippy lints
cargo clippy --all-targets --all-features -- -D warnings

# Run security audit
cargo audit

# Run tests with verbose output
cargo test --all-features -- --nocapture

# Run a specific test
cargo test test_create_product -- --nocapture

# Run tests with coverage
cargo tarpaulin --all-features --out Html
```

### 4.6 Useful Aliases

Add these to your shell profile for convenience:

```bash
alias cc='cargo check --all-features'
alias ct='cargo test --all-features'
alias cf='cargo fmt --all'
alias cl='cargo clippy --all-targets --all-features -- -D warnings'
alias cw='cargo watch -x "check --all-features" -x "test --all-features"'
alias cqa='cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features'
```

---

## 5. Testing with Stripe Test Mode

### 5.1 Obtaining Stripe Test Keys

1. Create a Stripe account at https://dashboard.stripe.com/register (free).
2. Navigate to **Developers** > **API keys** in the Stripe dashboard.
3. Ensure you are in **Test mode** (toggle at the top of the dashboard).
4. Copy the following keys:
   - **Publishable key**: Starts with `pk_test_`
   - **Secret key**: Starts with `sk_test_`
5. Add them to your `.env` file.

### 5.2 Setting Up Webhook Forwarding

The Stripe CLI forwards webhook events from Stripe to your local development server.

```bash
# Install Stripe CLI
# macOS: brew install stripe/stripe-cli/stripe
# Linux: see https://stripe.com/docs/stripe-cli#install

# Login to Stripe
stripe login

# Forward webhooks to your local server
stripe listen --forward-to localhost:8080/api/v1/rustcommerce/webhooks/stripe

# The CLI will output a webhook signing secret (whsec_...)
# Add this to your .env as STRIPE_WEBHOOK_SECRET
```

### 5.3 Stripe Test Card Numbers

Use these card numbers in test mode (any future expiry date, any 3-digit CVC):

| Card Number | Scenario |
|-------------|----------|
| `4242 4242 4242 4242` | Successful payment |
| `4000 0000 0000 3220` | Requires 3D Secure authentication |
| `4000 0000 0000 9995` | Declined (insufficient funds) |
| `4000 0000 0000 0002` | Declined (generic) |
| `4000 0025 0000 3155` | Requires authentication (SCA) |
| `4000 0000 0000 3063` | Declined (expired card) |

### 5.4 Testing Webhook Events

```bash
# Trigger a specific webhook event
stripe trigger payment_intent.succeeded

# Trigger a checkout session completed event
stripe trigger checkout.session.completed

# Trigger a charge refunded event
stripe trigger charge.refunded

# List all triggerable events
stripe trigger --list
```

### 5.5 Integration Test Configuration

For automated integration tests, Stripe provides a mock-friendly approach:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Tests use the Stripe test key from environment
    // or fall back to a mock client
    fn get_stripe_client() -> StripeClient {
        if let Ok(key) = std::env::var("STRIPE_SECRET_KEY") {
            if key.starts_with("sk_test_") {
                return StripeClient::new(&key);
            }
        }
        // Use mock client for CI without real Stripe keys
        StripeClient::mock()
    }

    #[tokio::test]
    async fn test_payment_intent_creation() {
        let client = get_stripe_client();
        let intent = client
            .create_payment_intent(5000, "usd")
            .await
            .expect("Failed to create payment intent");
        assert_eq!(intent.amount, 5000);
        assert_eq!(intent.currency, "usd");
    }
}
```

### 5.6 Stripe Test Mode Limitations

| Feature | Test Mode Behavior |
|---------|-------------------|
| Payments | Simulated; no real money moves |
| Webhooks | Must use Stripe CLI for local forwarding |
| Rate limits | More lenient than live mode |
| 3D Secure | Simulated with test card numbers |
| Payout timing | Instant (no settlement delay) |
| Disputes | Can be simulated via the dashboard |

---

## 6. Environment-Specific Configurations

### 6.1 Configuration Matrix

| Setting | Development | Test | Staging | Production |
|---------|------------|------|---------|------------|
| `RUST_LOG` | `debug,rustcommerce=trace` | `info` | `info,rustcommerce=debug` | `info` |
| `DATABASE_URL` | localhost:5432 | localhost:5433 | staging-db:5432 | prod-db:5432 |
| `REDIS_URL` | localhost:6379 | localhost:6380 | staging-redis:6379 | prod-redis:6379 |
| Stripe key prefix | `sk_test_` | `sk_test_` | `sk_test_` | `sk_live_` |
| `JWT_SECRET` | Static dev value | Static test value | Rotated secret | Vault-managed |
| Stock reservation | 10 min | 1 min (fast tests) | 10 min | 15 min |
| Cart expiry | 72 hours | 1 hour | 72 hours | 48 hours |

### 6.2 Switching Environments

```bash
# Load development environment
source .env.development

# Load test environment
source .env.test

# Or use direnv for automatic loading:
# Install direnv, then .envrc will auto-load .env
echo 'dotenv' > .envrc
direnv allow
```

---

## 7. Troubleshooting

### Common Issues

| Problem | Cause | Solution |
|---------|-------|----------|
| `sqlx` compile error: "no database" | `DATABASE_URL` not set or DB not running | Start Docker services; ensure `.env` is loaded |
| `cargo check` fails with sqlx errors | Offline metadata out of date | Run `cargo sqlx prepare` after query changes |
| Stripe webhook 400 errors | Wrong webhook secret | Get fresh secret from `stripe listen` output |
| PostgreSQL connection refused | Docker container not healthy | Run `docker compose up -d postgres` and wait for health check |
| Redis connection refused | Redis container not started | Run `docker compose up -d redis` |
| Slow compilation | Full rebuild triggered | Ensure `target/` is not on a network drive; use `sccache` or `mold` linker |
| `cargo test` hangs | Test waiting for database | Ensure test PostgreSQL is running on port 5433 |
| Permission denied on Docker volumes | UID mismatch | Run `docker compose down -v` and recreate |

### Resetting Local State

```bash
# Nuclear option: reset everything to a clean state
docker compose -f docker-compose.dev.yml down -v   # Stop and remove all containers + volumes
cargo clean                                          # Remove build artifacts
rm -rf .sqlx/                                       # Remove sqlx offline data
docker compose -f docker-compose.dev.yml up -d      # Restart fresh
sqlx migrate run                                    # Re-run migrations
cargo sqlx prepare                                  # Regenerate offline data
cargo build                                         # Full rebuild
```
