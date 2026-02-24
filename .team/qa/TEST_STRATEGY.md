# RustCommerce Test Strategy

**Document Version**: 1.0
**Date**: 2026-02-24
**Prepared By**: QA Lead
**Project**: RustCommerce (RCOM-001)
**Status**: Approved

---

## 1. Executive Summary

This document defines the overall test strategy for the RustCommerce e-commerce plugin for RustPress CMS. The strategy follows a testing pyramid approach, balancing thoroughness with execution speed. All testing layers target the P0 (MVP) feature set first, expanding to P1 features as the project matures.

---

## 2. Testing Pyramid

```
                    /\
                   /  \
                  / E2E \          (Playwright)
                 /--------\
                / Security  \      (OWASP ZAP, cargo-audit, custom)
               /-------------\
              / Performance    \   (criterion, k6)
             /------------------\
            /   Integration      \  (sqlx::test, wiremock, axum-test)
           /----------------------\
          /       Unit Tests        \ (cargo test, mockall, vitest)
         /----------------------------\
```

| Layer | Scope | Tools | Target Coverage | Execution Time |
|-------|-------|-------|-----------------|----------------|
| Unit | Individual functions, business logic, models | `cargo test`, `mockall`, `vitest`, `testing-library/react` | 80% | < 2 min |
| Integration | API endpoints, database operations, service interactions | `sqlx::test`, `wiremock`, `axum-test`, `testcontainers` | 60% | < 10 min |
| E2E | Full user flows (checkout, admin operations) | `playwright` | Critical paths | < 15 min |
| Performance | Response times, throughput, concurrency | `criterion`, `k6` | NFR benchmarks | On-demand |
| Security | OWASP Top 10, PCI-DSS, auth/authz | `cargo-audit`, `OWASP ZAP`, custom tests | All attack vectors | On-demand |

---

## 3. Test Layers in Detail

### 3.1 Unit Tests

**Purpose**: Verify individual functions, business logic algorithms, and data transformations in isolation.

**Backend (Rust)**:
- **Framework**: `cargo test` (built-in)
- **Mocking**: `mockall` for trait-based dependency injection
- **Scope**:
  - Cart total calculation (subtotal, discount, tax, shipping, grand total)
  - Tax calculation algorithm (zone matching, priority grouping, compounding)
  - Shipping cost calculation (flat rate, weight-based, price-based tiers)
  - Order status state machine (valid transitions, invalid transitions, side effects)
  - Coupon validation rules (expiry, usage limits, minimum spend, product restrictions)
  - Order number generation (format, uniqueness)
  - Stock status derivation (in-stock, out-of-stock, backorder)
  - Price calculation precision (DECIMAL handling, rounding)
  - Product slug generation (uniqueness, special characters)
  - Input validation (all request DTOs)
  - Error type mapping (domain errors to HTTP responses)

**Frontend (React/TypeScript)**:
- **Framework**: `vitest`
- **DOM Testing**: `@testing-library/react`
- **Scope**:
  - Zustand store actions and selectors
  - Cart state management (add, remove, update, clear)
  - Price formatting and display logic
  - Form validation (product editor, checkout forms, settings)
  - API response transformation
  - Component rendering with various props/states
  - Conditional UI logic (admin permissions, stock status indicators)

**Coverage Target**: 80% line coverage for business logic modules (`services/`, `models/`), 70% for handlers, 80% for frontend stores and utilities.

### 3.2 Integration Tests

**Purpose**: Verify that components work correctly together -- API endpoints with real database, service-to-repository interactions, external API integration.

**Backend (Rust)**:
- **Database Tests**: `sqlx::test` with per-test PostgreSQL databases (automatic migration, rollback)
- **HTTP Tests**: `axum-test` (in-process HTTP testing without network overhead)
- **External API Mocking**: `wiremock` (Stripe API simulation)
- **Containers**: `testcontainers` for PostgreSQL and Redis in CI
- **Scope**:
  - Product CRUD operations against real PostgreSQL
  - Cart persistence and retrieval across sessions
  - Checkout flow with mocked Stripe (PaymentIntent creation, webhook simulation)
  - Order creation from checkout with stock decrement
  - Coupon application with usage tracking
  - Customer address management
  - Category hierarchy queries
  - Cursor-based pagination correctness
  - Full-text search (PostgreSQL `tsvector` / `tsquery`)
  - Rate limiting behavior
  - Authentication middleware (JWT validation, permission checks)
  - Webhook signature verification
  - Idempotency key enforcement
  - Database constraint enforcement (unique SKU, valid status values)

**Frontend (React/TypeScript)**:
- **API Integration**: `vitest` with `msw` (Mock Service Worker) for API mocking
- **Scope**:
  - API client functions against mock endpoints
  - Store hydration from API responses
  - Error handling for API failures (network errors, 4xx, 5xx)
  - Pagination and infinite scroll behavior

**Coverage Target**: 60% for integration test paths. All P0 API endpoints must have at least one happy-path and one error-path integration test.

### 3.3 End-to-End (E2E) Tests

**Purpose**: Verify complete user journeys from browser through API to database and back.

**Framework**: `playwright` (cross-browser: Chromium, Firefox, WebKit)

**Critical Paths Covered**:

| # | Flow | Priority |
|---|------|----------|
| E2E-01 | Customer browses products, adds to cart, completes checkout with Stripe test mode | P0 |
| E2E-02 | Guest checkout (no login) with address entry and payment | P0 |
| E2E-03 | Admin creates a product with variants, images, and categories | P0 |
| E2E-04 | Admin views order list, opens order detail, updates status to shipped | P0 |
| E2E-05 | Admin processes a refund from order detail page | P0 |
| E2E-06 | Customer views order history and order detail | P0 |
| E2E-07 | Admin configures store settings (currency, tax, shipping) | P1 |
| E2E-08 | Customer applies coupon at checkout and sees discount | P1 |
| E2E-09 | Cart merge: guest adds items, logs in, guest cart merges with saved cart | P1 |
| E2E-10 | Admin performs bulk product operations (archive, delete) | P1 |

**Environment**: E2E tests run against a Docker Compose stack with real PostgreSQL, the RustPress server, and Stripe test mode (using Stripe test API keys).

### 3.4 Performance Tests

**Purpose**: Verify that the system meets the non-functional performance requirements defined in the project charter.

**Backend Benchmarks**:
- **Framework**: `criterion` (Rust microbenchmarks)
- **Scope**:
  - Cart total calculation with N items (N = 1, 10, 50, 100)
  - Tax calculation with complex zone matching
  - Product listing serialization
  - Database query execution time for common patterns

**Load Tests**:
- **Framework**: `k6` (JavaScript-based load testing)
- **Scenarios**:

| Scenario | Target | Metric |
|----------|--------|--------|
| Product listing (cached) | < 100ms p95 | Response time |
| Product detail page | < 150ms p95 | Response time |
| Add to cart | < 200ms p95 | Response time |
| Full checkout flow | < 3s end-to-end | Total duration |
| 100 concurrent shoppers browsing | No errors, < 200ms p95 | Throughput + latency |
| 50 concurrent checkouts | No stock inconsistency | Data integrity |
| Admin product list (1000+ products) | < 500ms p95 | Response time |

**Execution**: Performance tests run on-demand, not in every CI build. Scheduled weekly on a dedicated performance environment.

### 3.5 Security Tests

**Purpose**: Verify compliance with OWASP Top 10, PCI-DSS requirements, and the security requirements document (SECURITY_REQUIREMENTS.md).

**Tools and Approaches**:

| Area | Tool / Method | Scope |
|------|---------------|-------|
| Dependency vulnerabilities | `cargo-audit`, `npm audit` | All Rust and JS dependencies |
| OWASP Top 10 scan | OWASP ZAP (automated scan) | All public API endpoints |
| SQL injection | Custom integration tests with malicious inputs | All query parameters and request body fields |
| XSS | Custom tests + ZAP scanner | Product name, description, review text |
| CSRF | Custom integration tests | All state-changing endpoints (POST/PUT/DELETE) |
| Auth bypass | Custom integration tests | All protected endpoints without/with invalid JWT |
| Permission escalation | Custom integration tests | Customer accessing admin endpoints, cross-user data access |
| Stripe webhook forgery | Custom integration tests | Webhook endpoint with forged/expired signatures |
| Rate limiting | Custom integration tests | Exceed rate limits on checkout/payment endpoints |
| PCI-DSS: no card data | Codebase grep + integration test | Verify no raw card data in DB, logs, or responses |
| PII encryption | Database inspection | Verify encrypted customer email, phone, addresses |
| Sensitive data in logs | Log inspection after test runs | Verify no tokens, passwords, card data in logs |

**Execution**: Security tests run on every PR that touches auth, payment, or checkout code. Full security scan runs weekly.

---

## 4. Test Tools Summary

### 4.1 Backend (Rust)

| Tool | Purpose | Crate |
|------|---------|-------|
| `cargo test` | Unit and integration test runner | built-in |
| `mockall` | Trait-based mocking for unit tests | `mockall` |
| `sqlx::test` | Per-test database with migrations | `sqlx` (feature `testing`) |
| `axum-test` | In-process HTTP testing for Axum handlers | `axum-test` |
| `wiremock` | HTTP mock server for Stripe API | `wiremock` |
| `testcontainers` | Ephemeral PostgreSQL/Redis containers for CI | `testcontainers` |
| `criterion` | Microbenchmark framework | `criterion` |
| `cargo-audit` | Dependency vulnerability scanning | `cargo-audit` |
| `cargo-tarpaulin` / `llvm-cov` | Code coverage measurement | `cargo-tarpaulin` or `cargo-llvm-cov` |
| `proptest` | Property-based testing (optional, for edge cases) | `proptest` |

### 4.2 Frontend (React/TypeScript)

| Tool | Purpose | Package |
|------|---------|---------|
| `vitest` | Unit and integration test runner | `vitest` |
| `@testing-library/react` | Component rendering and DOM queries | `@testing-library/react` |
| `@testing-library/user-event` | Simulating user interactions | `@testing-library/user-event` |
| `msw` | API mocking (Mock Service Worker) | `msw` |
| `playwright` | End-to-end browser testing | `@playwright/test` |
| `eslint` | Static analysis / code quality | `eslint` |

### 4.3 Infrastructure

| Tool | Purpose |
|------|---------|
| `k6` | Load and performance testing |
| `OWASP ZAP` | Dynamic security scanning |
| `Docker Compose` | Local test environment orchestration |
| `GitHub Actions` | CI/CD test execution |

---

## 5. Test Environments

### 5.1 Local Development

**Setup**: Docker Compose file providing:
- PostgreSQL 16 (with test database)
- Redis (for session/cache testing)
- Stripe CLI (for webhook forwarding in local development)

**How developers run tests**:
```bash
# Unit tests (fast, no external dependencies)
cargo test --lib

# Integration tests (requires Docker Compose up)
cargo test --test '*'

# Frontend unit tests
npm run test

# E2E tests (requires full stack running)
npx playwright test
```

### 5.2 CI Environment (GitHub Actions)

**Configuration**: GitHub Actions workflow with service containers.

```yaml
services:
  postgres:
    image: postgres:16
    env:
      POSTGRES_DB: rustcommerce_test
      POSTGRES_USER: test
      POSTGRES_PASSWORD: test
    ports:
      - 5432:5432
    options: >-
      --health-cmd pg_isready
      --health-interval 10s
      --health-timeout 5s
      --health-retries 5
  redis:
    image: redis:7
    ports:
      - 6379:6379
```

**CI Pipeline Stages**:

| Stage | Trigger | Tests Run | Max Duration |
|-------|---------|-----------|--------------|
| Lint & Format | Every push | `cargo fmt --check`, `cargo clippy`, `eslint` | 2 min |
| Unit Tests | Every push | `cargo test --lib`, `vitest run` | 5 min |
| Integration Tests | Every push | `cargo test --test '*'` with service containers | 10 min |
| E2E Tests | PR to main | `playwright test` (Chromium only in CI) | 15 min |
| Security Scan | PR to main + weekly | `cargo-audit`, `npm audit` | 3 min |
| Performance | Weekly schedule | `criterion` benchmarks, `k6` load tests | 30 min |
| Coverage Report | Every push | `cargo-llvm-cov`, `vitest --coverage` | 5 min |

### 5.3 Staging Environment

A production-like environment used for:
- Final E2E validation before release
- Performance testing with realistic data volumes
- Stripe test mode integration testing with real Stripe test keys
- Security scanning with OWASP ZAP

---

## 6. Coverage Targets

| Module / Area | Unit Coverage | Integration Coverage |
|---------------|:------------:|:-------------------:|
| `services/cart_service.rs` | 90% | 70% |
| `services/checkout_service.rs` | 85% | 70% |
| `services/order_service.rs` | 85% | 65% |
| `services/payment_service.rs` | 80% | 60% |
| `services/tax_service.rs` | 90% | 60% |
| `services/shipping_service.rs` | 85% | 60% |
| `services/inventory_service.rs` | 85% | 65% |
| `services/coupon_service.rs` | 85% | 60% |
| `services/product_service.rs` | 80% | 60% |
| `handlers/*` | 70% | 60% |
| `repositories/*` | 60% | 70% |
| `models/*` | 80% | N/A |
| Frontend stores | 80% | N/A |
| Frontend components | 70% | N/A |
| **Overall Backend** | **80%** | **60%** |
| **Overall Frontend** | **75%** | **50%** |

---

## 7. Test Data Strategy

### 7.1 Seed Data

A standardized seed data set for integration and E2E tests:

| Entity | Seed Count | Notes |
|--------|:----------:|-------|
| Products (simple) | 20 | Various prices, statuses, stock levels |
| Products (variable) | 5 | With 3-5 variants each |
| Categories | 10 | 3 levels of hierarchy |
| Customers | 5 | Various order histories |
| Orders | 15 | Across all status values |
| Coupons | 5 | One of each discount type |
| Tax Rates | 10 | US states + international |
| Shipping Zones | 3 | Domestic, international, default |
| Shipping Methods | 5 | One of each calculation type |
| Reviews | 10 | Various ratings, some pending moderation |

### 7.2 Test Fixtures

- **Rust**: Factory functions in a `test_helpers` module that create valid entities with sensible defaults and allow field overrides.
- **Frontend**: MSW handlers that return consistent mock data matching the API contract.

### 7.3 Data Isolation

- Each integration test gets its own database transaction (via `sqlx::test`) that is rolled back after the test.
- E2E tests use a dedicated test database that is re-seeded before each test suite run.
- No test data leaks between test runs.

---

## 8. Defect Management

| Severity | Description | Response Time | Fix SLA |
|----------|-------------|:-------------:|:-------:|
| S1 (Critical) | Data loss, payment errors, security breach | Immediate | Same day |
| S2 (Major) | Feature broken, checkout blocked, admin inaccessible | < 4 hours | 1 business day |
| S3 (Minor) | Non-critical feature degraded, cosmetic issue with workaround | < 1 day | Next sprint |
| S4 (Trivial) | Cosmetic only, no functional impact | Best effort | Backlog |

Bug reports follow the template in `BUG_REPORT.md`.

---

## 9. Quality Gates

### 9.1 PR Merge Requirements

All of the following must pass before a PR can merge to `main`:
- All unit tests pass
- All integration tests pass
- Code coverage does not decrease
- No new `cargo-audit` / `npm audit` vulnerabilities
- `cargo clippy` and `eslint` produce no warnings
- Code review approved by at least one team member

### 9.2 Release Requirements

Before any release:
- All E2E tests pass on staging
- Performance benchmarks meet NFR targets
- Security scan produces no high/critical findings
- QA sign-off document completed (see `QA_SIGNOFF.md`)
- All S1/S2 bugs resolved
- Coverage targets met

---

## 10. Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|:----------:|:------:|-----------|
| Stripe API changes break integration | Low | High | Pin Stripe API version, monitor changelog, wiremock tests isolate from real API |
| Flaky integration tests in CI | Medium | Medium | Use `sqlx::test` for DB isolation, retry logic for container startup, deterministic test data |
| Slow E2E tests block development | Medium | Medium | Run E2E only on PR to main, parallelize with Playwright sharding |
| Insufficient test coverage on edge cases | Medium | High | Use `proptest` for property-based testing on calculation functions |
| Performance regression undetected | Low | High | Weekly automated benchmarks with alerting on regression |
| Security vulnerability in dependency | Medium | High | Automated `cargo-audit` in CI, Dependabot alerts enabled |

---

*This test strategy is a living document. It will be updated as the project progresses through milestones and new testing needs are identified.*
