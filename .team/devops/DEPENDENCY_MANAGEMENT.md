# Dependency Management Strategy -- RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Owner**: DevOps Lead
**Status**: Approved

---

## 1. Overview

This document defines the dependency management strategy for the RustCommerce plugin, covering Rust (Cargo) and frontend (npm) dependencies. It establishes version policies, update cadences, security auditing procedures, and the Minimum Supported Rust Version (MSRV) policy.

### Guiding Principles

- **Pin for reproducibility**: Use `Cargo.lock` and `package-lock.json` for deterministic builds.
- **Range for flexibility**: Use semver-compatible version ranges in `Cargo.toml` to receive patch fixes.
- **Audit continuously**: Run `cargo audit` and `npm audit` in every CI pipeline run.
- **Update deliberately**: Dependency updates are intentional, reviewed, and tested -- not automatic merges.

---

## 2. Cargo Dependency Versions

### 2.1 Core Dependencies

These are the primary Cargo dependencies for RustCommerce, with their pinning strategy.

| Crate | Version | Strategy | Rationale |
|-------|---------|----------|-----------|
| `rustpress-core` | `path` or `git` | Track HEAD of main branch | Must stay in sync with core platform |
| `rustpress-database` | `path` or `git` | Track HEAD of main branch | Shared database layer |
| `rustpress-auth` | `path` or `git` | Track HEAD of main branch | Authentication integration |
| `serde` | `"1.0"` | Semver range | Stable API, backward compatible |
| `serde_json` | `"1.0"` | Semver range | Stable API |
| `sqlx` | `"0.8"` | Minor-pinned | Breaking changes possible in 0.x |
| `uuid` | `"1.0"` | Semver range | Stable API |
| `chrono` | `"0.4"` | Minor-pinned | Well-established, seldom breaks |
| `tokio` | `"1"` | Major-pinned | Async runtime, stable API |
| `axum` | `"0.7"` | Minor-pinned | Web framework, 0.x versioning |
| `stripe-rust` | `"0.35"` | Minor-pinned | External API client, frequent updates |
| `thiserror` | `"2.0"` | Semver range | Stable error derive macro |
| `tracing` | `"0.1"` | Minor-pinned | Logging/tracing framework |
| `tracing-subscriber` | `"0.3"` | Minor-pinned | Log output formatting |
| `prometheus` | `"0.13"` | Minor-pinned | Metrics library |
| `rust_decimal` | `"1.0"` | Semver range | Precise monetary calculations |
| `validator` | `"0.18"` | Minor-pinned | Input validation |
| `async-trait` | `"0.1"` | Minor-pinned | Until Rust async traits stabilize fully |

### 2.2 Development Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` (with `test-util`) | `"1"` | Async test runtime |
| `sqlx` (with `runtime-tokio`) | `"0.8"` | Database test utilities |
| `mockall` | `"0.13"` | Mock generation for unit tests |
| `wiremock` | `"0.6"` | HTTP mock server for Stripe API tests |
| `fake` | `"3.0"` | Test data generation |
| `assert_matches` | `"1.5"` | Pattern matching assertions |
| `testcontainers` | `"0.21"` | Ephemeral Docker containers in tests |

### 2.3 Version Pinning Strategy

```toml
# Cargo.toml — Version specification patterns

[dependencies]
# Semver range (^) — default, allows patch and minor updates within major
serde = "1.0"            # Equivalent to ^1.0, accepts 1.0.x and 1.x.0

# Minor-pinned — for 0.x crates where minor bumps can break
sqlx = "0.8"             # Accepts 0.8.x only, not 0.9.0
axum = "0.7"             # Accepts 0.7.x only

# Exact pin — for git/path dependencies (core platform crates)
rustpress-core = { git = "https://github.com/rustpress-net/rustpress-core-base", branch = "main" }

# Feature selection — always explicit
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "rust_decimal"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
```

### 2.4 Cargo.lock Policy

- `Cargo.lock` is **always committed** to version control for this project (it is a deployable plugin, not a library consumed by others).
- CI validates that `Cargo.lock` is up to date (no uncommitted lockfile changes after `cargo check`).
- Lockfile updates happen in dedicated PRs, separate from feature work.

---

## 3. Update Policy

### 3.1 Update Cadence

| Category | Frequency | Process |
|----------|-----------|---------|
| **Security patches** | Immediately | Triggered by `cargo audit` findings; hotfix branch |
| **Patch updates** (x.x.PATCH) | Bi-weekly | Batch update, run full test suite |
| **Minor updates** (x.MINOR.0) | Monthly | Review changelog, update individually, full test suite |
| **Major updates** (MAJOR.0.0) | As needed | Dedicated PR with migration plan, extensive testing |
| **RustPress core sync** | Each milestone | Update git rev/branch to latest stable core |

### 3.2 Update Process

#### Automated Detection

```bash
# Check for outdated dependencies
cargo outdated

# Check for security advisories
cargo audit

# Check for outdated npm packages (when frontend exists)
cd admin-ui && npm outdated
```

#### Update Procedure

1. **Create a branch**: `deps/update-YYYY-MM-DD`
2. **Run update commands**:
   ```bash
   # Update all compatible versions
   cargo update

   # Or update a specific crate
   cargo update -p sqlx

   # For major version bumps, edit Cargo.toml manually
   ```
3. **Run full quality pipeline locally**:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   cargo audit
   ```
4. **Review changelog** for each updated crate for breaking changes or deprecations.
5. **Open PR** with a summary of what changed and why.
6. **CI must pass** all quality gates before merge.

### 3.3 Dependabot / Renovate Configuration

If using GitHub Dependabot, the recommended configuration:

```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
      day: "monday"
    open-pull-requests-limit: 10
    reviewers:
      - "rustpress-net/devops"
    labels:
      - "dependencies"
      - "rust"
    # Group minor and patch updates to reduce PR noise
    groups:
      minor-and-patch:
        update-types:
          - "minor"
          - "patch"

  - package-ecosystem: "npm"
    directory: "/admin-ui"
    schedule:
      interval: "weekly"
      day: "monday"
    open-pull-requests-limit: 5
    reviewers:
      - "rustpress-net/frontend"
    labels:
      - "dependencies"
      - "frontend"
```

---

## 4. Security Auditing

### 4.1 cargo-audit Integration

`cargo-audit` checks the RustSec Advisory Database for known vulnerabilities in dependencies.

#### CI Integration

```yaml
# In ci.yml
audit:
  name: Security Audit
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install cargo-audit
      run: cargo install cargo-audit
    - name: Run security audit
      run: cargo audit
    - name: Generate audit report
      if: failure()
      run: cargo audit --json > audit-report.json
    - name: Upload audit report
      if: failure()
      uses: actions/upload-artifact@v4
      with:
        name: cargo-audit-report
        path: audit-report.json
```

#### Local Usage

```bash
# Basic audit
cargo audit

# Audit with fix suggestions
cargo audit fix --dry-run

# Audit and output JSON (for automation)
cargo audit --json

# Ignore a specific advisory (with justification in audit.toml)
cargo audit --ignore RUSTSEC-2024-XXXX
```

### 4.2 Advisory Exceptions

When an advisory cannot be immediately resolved (e.g., no patched version available), document the exception:

```toml
# audit.toml
[advisories]
ignore = [
    # RUSTSEC-2024-XXXX: Explanation of why this is acceptable
    # and when we expect to resolve it.
    # Tracked in issue #123.
]
```

### 4.3 npm Audit

```bash
# Run npm security audit
cd admin-ui && npm audit

# Fix automatically where possible
npm audit fix

# Generate a detailed report
npm audit --json > npm-audit-report.json
```

#### CI Integration for npm

```yaml
frontend-audit:
  name: Frontend Security Audit
  runs-on: ubuntu-latest
  defaults:
    run:
      working-directory: admin-ui
  steps:
    - uses: actions/checkout@v4
    - uses: actions/setup-node@v4
      with:
        node-version: '20'
    - run: npm ci
    - run: npm audit --audit-level=high
```

### 4.4 Supply Chain Security

| Measure | Tool | Purpose |
|---------|------|---------|
| Lock file verification | `cargo check` / `npm ci` | Ensure deterministic installs |
| Advisory scanning | `cargo audit` / `npm audit` | Known vulnerability detection |
| License compliance | `cargo-deny` | Ensure all deps use approved licenses |
| Dependency review | `actions/dependency-review-action` | PR-time supply chain checks |

#### cargo-deny Configuration

```toml
# deny.toml
[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-DFS-2016",
    "Zlib",
]
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-git = [
    "https://github.com/rustpress-net/rustpress-core-base",
]
```

---

## 5. Minimum Supported Rust Version (MSRV) Policy

### 5.1 MSRV Definition

| Field | Value |
|-------|-------|
| **Current MSRV** | 1.80.0 |
| **Policy** | N-2 (support current stable minus 2 releases) |
| **Declared in** | `Cargo.toml` (`rust-version` field) and `README.md` |

### 5.2 Cargo.toml Declaration

```toml
[package]
name = "rustcommerce"
version = "1.0.0"
edition = "2021"
rust-version = "1.80"  # MSRV
```

### 5.3 MSRV Update Policy

- MSRV is bumped **only in minor or major releases**, never in patch releases.
- MSRV bumps require a changelog entry and a note in the release notes.
- Before bumping MSRV, verify that the target Rust version is available in:
  - Ubuntu LTS package repositories
  - Official Docker `rust:` images
  - Common CI runner images (GitHub Actions `ubuntu-latest`)

### 5.4 CI Verification

```yaml
msrv-check:
  name: MSRV Check
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@master
      with:
        toolchain: "1.80"  # Must match rust-version in Cargo.toml
    - run: cargo check --all-features
```

### 5.5 MSRV Bump Process

1. Update `rust-version` in `Cargo.toml`.
2. Update MSRV check in CI to the new version.
3. Update `README.md` with the new MSRV.
4. Add a changelog entry: `chore: bump MSRV to 1.XX`.
5. Test the build with the exact MSRV toolchain version.

---

## 6. Frontend Dependency Management

### 6.1 Core Frontend Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| `react` | `^18.3` | UI framework |
| `react-dom` | `^18.3` | React DOM renderer |
| `typescript` | `^5.5` | Type safety |
| `zustand` | `^5.0` | State management |
| `axios` | `^1.7` | HTTP client |
| `tailwindcss` | `^3.4` | Utility CSS framework |
| `lucide-react` | `^0.400` | Icon library |
| `vite` | `^5.4` | Build tool |

### 6.2 Frontend Dev Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| `@types/react` | `^18` | React type definitions |
| `@types/react-dom` | `^18` | React DOM types |
| `eslint` | `^9` | Linting |
| `@typescript-eslint/parser` | `^8` | TS ESLint parser |
| `prettier` | `^3` | Code formatting |
| `vitest` | `^2` | Unit testing |

### 6.3 npm Lockfile Policy

- `package-lock.json` is **always committed** to version control.
- CI uses `npm ci` (not `npm install`) for deterministic installs.
- Direct dependency updates go through the same PR review process as Cargo updates.

### 6.4 Node.js Version Policy

- **Target**: Node.js 20 LTS.
- **Enforcement**: `.nvmrc` file in the `admin-ui/` directory.
- **CI**: Pinned via `actions/setup-node@v4` with `node-version: '20'`.

```
# admin-ui/.nvmrc
20
```

---

## 7. Dependency Update Schedule

### Weekly Checklist (Monday)

- [ ] Review Dependabot / Renovate PRs if enabled
- [ ] Run `cargo audit` for new advisories
- [ ] Run `npm audit` for new advisories (if frontend exists)
- [ ] Triage any critical/high severity findings

### Monthly Checklist (First Monday of Month)

- [ ] Run `cargo outdated` and review available updates
- [ ] Run `npm outdated` and review available updates
- [ ] Update minor/patch versions in a batch PR
- [ ] Review cargo-deny output for license compliance
- [ ] Check if MSRV should be bumped

### Per-Release Checklist

- [ ] Freeze dependency updates 1 week before release
- [ ] Run full `cargo audit` and `npm audit`
- [ ] Verify `Cargo.lock` and `package-lock.json` are committed
- [ ] Run `cargo-deny check` for license and ban compliance
- [ ] Document any known advisory exceptions in release notes

---

## 8. Dependency Decision Log

Track major dependency decisions here for future reference.

| Date | Decision | Rationale | Issue |
|------|----------|-----------|-------|
| 2026-02-24 | Use `sqlx` 0.8 for database | Compile-time checked queries, async, PostgreSQL native | -- |
| 2026-02-24 | Use `stripe-rust` for Stripe | Official community crate, active maintenance | -- |
| 2026-02-24 | Use `rust_decimal` for money | Avoids floating-point precision issues in financial calculations | -- |
| 2026-02-24 | Use `axum` 0.7 for HTTP | Matches RustPress core framework, tower ecosystem | -- |
| 2026-02-24 | Use `thiserror` 2.0 for errors | Clean derive-based error types, matches RustPress patterns | -- |
| 2026-02-24 | MSRV set to 1.80 | Supports async trait features, wide CI availability | -- |
