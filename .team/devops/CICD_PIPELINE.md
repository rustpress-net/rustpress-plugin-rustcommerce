# CI/CD Pipeline Design -- RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Owner**: DevOps Lead
**Status**: Approved

---

## 1. Overview

This document defines the enhanced CI/CD pipeline for the RustCommerce plugin. It extends the existing `ci.yml` and `release.yml` workflows with additional quality gates, integration testing, frontend CI, and a structured branch strategy.

### Design Principles

- **Fail fast**: Cheapest checks (fmt, clippy) run first; expensive checks (integration tests) run only after basic validation passes.
- **Reproducibility**: All pipeline jobs use pinned tool versions and deterministic caching.
- **Security**: Secrets are scoped to the minimum required jobs. No secrets in CI logs.
- **Parallelism**: Independent jobs run concurrently to minimize wall-clock time.

---

## 2. Branch Strategy

```
feature/* ──> development ──> release/* ──> main
hotfix/*  ──────────────────────────────> main (cherry-pick back to development)
```

| Branch | Purpose | CI Trigger | Deploy Target |
|--------|---------|------------|---------------|
| `feature/*` | Individual feature work | PR checks only | None |
| `development` | Integration branch, all features merged here | Full CI + integration tests | Dev environment |
| `release/*` | Release stabilization (e.g., `release/1.0.0`) | Full CI + integration tests + release candidate build | Staging |
| `main` | Production-ready code only | Full CI + release pipeline | Production |
| `hotfix/*` | Urgent production fixes | Full CI + integration tests | Merged to main directly |

### Merge Rules

- **feature -> development**: Squash merge; requires passing CI and 1 approval.
- **development -> release/\***: Merge commit; release branch cut by DevOps or PM.
- **release/\* -> main**: Merge commit; requires passing all CI + integration tests + QA sign-off.
- **hotfix/\* -> main**: Merge commit; requires passing all CI + DevOps approval. Cherry-pick to development after merge.

### Branch Protection Rules

| Branch | Required Checks | Approvals | Force Push | Delete After Merge |
|--------|----------------|-----------|------------|-------------------|
| `main` | All CI + Integration | 2 | Blocked | N/A |
| `development` | All CI | 1 | Blocked | N/A |
| `release/*` | All CI + Integration | 2 | Blocked | Yes |

---

## 3. CI Pipeline (Enhanced `ci.yml`)

### 3.1 Trigger Configuration

```yaml
on:
  push:
    branches: [main, development, 'release/**']
  pull_request:
    branches: [main, development, 'release/**']
```

### 3.2 Job Graph

```
┌────────────┐  ┌────────────┐  ┌────────────┐  ┌──────────────┐
│  cargo fmt  │  │cargo clippy│  │cargo check │  │ sqlx prepare │
│  (30s)      │  │  (2min)    │  │  (2min)    │  │  check (1m)  │
└──────┬──────┘  └──────┬─────┘  └──────┬─────┘  └──────┬───────┘
       │                │               │                │
       └────────────────┴───────┬───────┴────────────────┘
                                │
                         ┌──────▼──────┐
                         │ cargo test  │
                         │  (unit)     │
                         │  (3min)     │
                         └──────┬──────┘
                                │
                    ┌───────────▼───────────┐
                    │ integration tests     │
                    │ (PostgreSQL service)  │
                    │ (5min)               │
                    └───────────┬───────────┘
                                │
                    ┌───────────▼───────────┐
                    │   security audit      │
                    │   (cargo-audit)       │
                    │   (1min)              │
                    └───────────────────────┘
```

### 3.3 Job Definitions

#### Job 1: Format Check (`fmt`)

```yaml
fmt:
  name: Format
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt
    - run: cargo fmt --all -- --check
```

#### Job 2: Clippy Lint (`clippy`)

```yaml
clippy:
  name: Clippy
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    - uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-clippy-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: ${{ runner.os }}-clippy-
    - run: cargo clippy --all-targets --all-features -- -D warnings
```

#### Job 3: Cargo Check (`check`)

```yaml
check:
  name: Check
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-check-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: ${{ runner.os }}-check-
    - run: cargo check --all-targets --all-features
```

#### Job 4: SQLx Prepare Check (`sqlx-check`)

This validates that the offline query data in `.sqlx/` matches the current queries in source code. This ensures compile-time checked queries remain valid without a live database.

```yaml
sqlx-check:
  name: SQLx Prepare Check
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-sqlx-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: ${{ runner.os }}-sqlx-
    - name: Install sqlx-cli
      run: cargo install sqlx-cli --no-default-features --features postgres
    - name: Check sqlx prepare
      run: cargo sqlx prepare --check
      env:
        SQLX_OFFLINE: true
```

#### Job 5: Unit Tests (`test`)

```yaml
test:
  name: Unit Tests
  needs: [fmt, clippy, check, sqlx-check]
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-test-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: ${{ runner.os }}-test-
    - name: Run unit tests
      run: cargo test --all-features --lib --bins
      env:
        RUST_LOG: info
        RUST_BACKTRACE: 1
```

#### Job 6: Integration Tests (`integration-test`)

```yaml
integration-test:
  name: Integration Tests
  needs: [test]
  runs-on: ubuntu-latest
  services:
    postgres:
      image: postgres:16
      ports:
        - 5432:5432
      env:
        POSTGRES_DB: rustcommerce_test
        POSTGRES_USER: rustcommerce
        POSTGRES_PASSWORD: test_password
      options: >-
        --health-cmd "pg_isready -U rustcommerce"
        --health-interval 10s
        --health-timeout 5s
        --health-retries 5
    redis:
      image: redis:7
      ports:
        - 6379:6379
      options: >-
        --health-cmd "redis-cli ping"
        --health-interval 10s
        --health-timeout 5s
        --health-retries 5
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-integration-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: ${{ runner.os }}-integration-
    - name: Install sqlx-cli
      run: cargo install sqlx-cli --no-default-features --features postgres
    - name: Run migrations
      run: sqlx migrate run
      env:
        DATABASE_URL: postgres://rustcommerce:test_password@localhost:5432/rustcommerce_test
    - name: Run integration tests
      run: cargo test --all-features --test '*'
      env:
        DATABASE_URL: postgres://rustcommerce:test_password@localhost:5432/rustcommerce_test
        REDIS_URL: redis://localhost:6379
        STRIPE_SECRET_KEY: sk_test_fake_key_for_ci
        STRIPE_WEBHOOK_SECRET: whsec_test_fake_secret_for_ci
        RUST_LOG: info
        RUST_BACKTRACE: 1
```

#### Job 7: Security Audit (`audit`)

```yaml
audit:
  name: Security Audit
  needs: [test]
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install cargo-audit
      run: cargo install cargo-audit
    - name: Run audit
      run: cargo audit
```

---

## 4. Frontend CI Pipeline

The admin UI components for RustCommerce live within the RustPress admin UI repository, but the plugin may also contain frontend code (type definitions, API client, Zustand stores). When frontend assets exist in this repo, the following pipeline runs.

### 4.1 Trigger

Runs as part of the main CI workflow when changes are detected in `admin-ui/` or any `*.ts`/`*.tsx`/`package.json` files.

```yaml
frontend:
  name: Frontend CI
  runs-on: ubuntu-latest
  # Only run if frontend files changed
  if: |
    contains(github.event.head_commit.modified, 'admin-ui/') ||
    contains(github.event.head_commit.modified, 'package.json')
  defaults:
    run:
      working-directory: admin-ui
  steps:
    - uses: actions/checkout@v4

    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: '20'
        cache: 'npm'
        cache-dependency-path: admin-ui/package-lock.json

    - name: Install dependencies
      run: npm ci

    - name: TypeScript type check
      run: npx tsc --noEmit

    - name: Lint
      run: npx eslint . --ext .ts,.tsx --max-warnings 0

    - name: Build
      run: npm run build

    - name: Test
      run: npm test -- --ci --coverage --passWithNoTests
```

### 4.2 Frontend Quality Gates

| Check | Tool | Threshold |
|-------|------|-----------|
| Type safety | `tsc --noEmit` | Zero errors |
| Linting | ESLint | Zero warnings (`--max-warnings 0`) |
| Build | Vite | Must succeed |
| Unit tests | Vitest / Jest | All pass, coverage reported |

---

## 5. Release Pipeline (Enhanced `release.yml`)

### 5.1 Trigger Configuration

```yaml
on:
  push:
    branches: [main]
    tags:
      - 'v*'
  workflow_dispatch:
    inputs:
      version_bump:
        description: 'Version bump type'
        required: true
        default: 'auto'
        type: choice
        options: [auto, patch, minor, major]
      release_type:
        description: 'Release type'
        required: true
        default: 'auto'
        type: choice
        options: [auto, release, pre-release, draft]
```

### 5.2 Release Job Graph

```
┌──────────────┐     ┌──────────────────┐
│ Determine    │────>│ Run full CI      │
│ Version      │     │ (reuse workflow) │
└──────────────┘     └───────┬──────────┘
                             │
                     ┌───────▼──────────┐
                     │ Build release    │
                     │ artifacts        │
                     └───────┬──────────┘
                             │
                     ┌───────▼──────────┐
                     │ Package plugin   │
                     │ (zip + checksums)│
                     └───────┬──────────┘
                             │
                     ┌───────▼──────────┐
                     │ Create GitHub    │
                     │ Release          │
                     └───────┬──────────┘
                             │
                     ┌───────▼──────────┐
                     │ Publish crate    │
                     │ (crates.io)      │
                     └───────┬──────────┘
                             │
                     ┌───────▼──────────┐
                     │ Cleanup old      │
                     │ pre-releases     │
                     └──────────────────┘
```

### 5.3 Semantic Versioning Strategy

Version bumps follow Conventional Commits:

| Commit Prefix | Version Bump | Example |
|---------------|-------------|---------|
| `fix:`, `fix(scope):` | PATCH (0.0.x) | `fix(cart): correct tax calculation` |
| `feat:`, `feat(scope):` | MINOR (0.x.0) | `feat(checkout): add guest checkout` |
| `feat!:`, `BREAKING CHANGE:` | MAJOR (x.0.0) | `feat!: restructure API routes` |

Pre-release tags follow: `v1.2.3-rc.1`, `v1.2.3-beta.1`

### 5.4 Release Artifacts

Each release publishes:

| Artifact | Contents | Format |
|----------|----------|--------|
| `rustcommerce-v{version}-plugin.zip` | Compiled library + migrations + plugin.toml + assets | ZIP |
| `rustcommerce-v{version}-source.tar.gz` | Source code archive | tar.gz |
| `SHA256SUMS.txt` | SHA-256 checksums for all artifacts | Text |
| `CHANGELOG.md` | Auto-generated from conventional commits | Markdown |

### 5.5 Crate Publishing

```yaml
publish:
  name: Publish to crates.io
  needs: [release]
  if: needs.version.outputs.release_type == 'release'
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Update version in Cargo.toml
      run: |
        sed -i "s/^version = .*/version = \"${{ needs.version.outputs.new_version }}\"/" Cargo.toml
    - name: Publish
      run: cargo publish --token ${{ secrets.CARGO_REGISTRY_TOKEN }}
      env:
        CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

---

## 6. Secrets Management

### Required GitHub Secrets

| Secret | Used In | Purpose |
|--------|---------|---------|
| `CARGO_REGISTRY_TOKEN` | Release pipeline | Publishing to crates.io |
| `STRIPE_SECRET_KEY` (test) | Integration tests | Stripe API test-mode calls |
| `STRIPE_WEBHOOK_SECRET` (test) | Integration tests | Webhook signature verification |
| `CODECOV_TOKEN` | CI pipeline | Coverage upload (optional) |

### Secret Scoping

- `CARGO_REGISTRY_TOKEN`: Only available in the `publish` job on `main` branch.
- Stripe test keys: Only available in `integration-test` job; never exposed to other jobs.
- No secrets are logged; all steps use `::add-mask::` for dynamic values.

---

## 7. Caching Strategy

| Cache Key | Scope | Contents | Estimated Size |
|-----------|-------|----------|----------------|
| `cargo-registry` | Per-OS | `~/.cargo/registry`, `~/.cargo/git` | ~200 MB |
| `cargo-target` | Per-OS + Cargo.lock hash | `target/` | ~500 MB |
| `npm-cache` | Per-OS + lock hash | `~/.npm` | ~100 MB |
| `sqlx-cli` | Per-OS | sqlx-cli binary | ~20 MB |

Cache keys include `hashFiles('**/Cargo.lock')` to invalidate on dependency changes. Restore keys allow partial cache hits when only some dependencies change.

---

## 8. Quality Gates Summary

| Gate | Tool | Blocking? | When |
|------|------|-----------|------|
| Code formatting | `cargo fmt` | Yes | All PRs |
| Linting | `cargo clippy` | Yes | All PRs |
| Compilation | `cargo check` | Yes | All PRs |
| SQLx offline check | `sqlx prepare --check` | Yes | All PRs |
| Unit tests | `cargo test` | Yes | All PRs |
| Integration tests | `cargo test --test` | Yes | PRs to main/release |
| Security audit | `cargo audit` | Advisory (non-blocking) | All PRs |
| Frontend typecheck | `tsc --noEmit` | Yes | When frontend changes |
| Frontend lint | ESLint | Yes | When frontend changes |
| Frontend build | Vite | Yes | When frontend changes |
| Coverage threshold | tarpaulin/llvm-cov | Advisory (80% target) | All PRs |

---

## 9. Notifications

| Event | Channel | Recipients |
|-------|---------|------------|
| CI failure on `main` | GitHub notification + Slack | All team members |
| Release published | GitHub notification + Slack | All team members |
| Security audit finding | GitHub Security tab + Slack | DevOps Lead, Backend Lead |
| PR check failure | GitHub PR status | PR author |

---

## 10. Future Enhancements

- **Coverage enforcement**: Integrate `cargo-tarpaulin` or `cargo-llvm-cov` with a minimum threshold gate.
- **Benchmarking**: Add `cargo bench` with `criterion` to detect performance regressions between releases.
- **E2E tests**: Add Playwright/Cypress pipeline stage for full checkout flow testing against a live test environment.
- **Container image publishing**: Build and push Docker images to GitHub Container Registry on release.
- **Dependency review**: Add `actions/dependency-review-action` for PR-time supply chain checks.
