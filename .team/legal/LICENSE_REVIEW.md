# License Compliance Review — RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Prepared By**: Legal/Compliance Attorney
**Project**: RustCommerce (RCOM-001)
**Project License**: MIT

---

## 1. Executive Summary

RustCommerce is licensed under the MIT License, which is one of the most permissive open-source licenses available. This review examines the compatibility of all current and planned dependencies with the MIT license, identifies any potential licensing conflicts, and provides recommendations for ongoing license compliance through automated tooling.

**Overall Assessment**: LOW RISK. All identified dependencies use permissive licenses (MIT, Apache-2.0, or dual MIT/Apache-2.0) that are fully compatible with the project's MIT license.

---

## 2. Project License Analysis

### 2.1 RustCommerce MIT License

The project uses a standard MIT License (see `/LICENSE`). Key characteristics:

- **Permissive**: Allows commercial use, modification, distribution, and private use
- **Conditions**: License and copyright notice must be included in copies
- **Limitations**: No liability, no warranty
- **Compatibility**: Compatible with virtually all open-source licenses as a downstream consumer

### 2.2 MIT License Obligations

As an MIT-licensed project, RustCommerce:
- MUST include the MIT license text and copyright notice in all distributions
- MUST preserve license notices from all dependencies in binary distributions
- SHOULD include a NOTICES or THIRD_PARTY_LICENSES file aggregating dependency licenses

---

## 3. Dependency License Audit

### 3.1 Current Dependencies (Cargo.toml)

| Crate | Version | License | Compatible | Notes |
|-------|---------|---------|------------|-------|
| `serde` | 1.0 | MIT OR Apache-2.0 | YES | Dual-licensed, either applies |
| `serde_json` | 1.0 | MIT OR Apache-2.0 | YES | Dual-licensed, either applies |

### 3.2 Planned Dependencies (from STRATEGY.md)

| Crate | Expected License | Compatible | Notes |
|-------|-----------------|------------|-------|
| `sqlx` | MIT OR Apache-2.0 | YES | Database layer; dual-licensed |
| `axum` | MIT | YES | HTTP framework; MIT licensed |
| `tokio` | MIT | YES | Async runtime; MIT licensed |
| `stripe-rust` | MIT OR Apache-2.0 | YES | Stripe SDK; dual-licensed |
| `uuid` | MIT OR Apache-2.0 | YES | UUID generation; dual-licensed |
| `chrono` | MIT OR Apache-2.0 | YES | Date/time handling; dual-licensed |
| `thiserror` | MIT OR Apache-2.0 | YES | Error handling; dual-licensed |
| `async-trait` | MIT OR Apache-2.0 | YES | Async trait support; dual-licensed |
| `tower` | MIT | YES | Service/middleware layer; MIT licensed |
| `tower-http` | MIT | YES | HTTP-specific middleware; MIT licensed |
| `tracing` | MIT | YES | Logging/instrumentation; MIT licensed |
| `reqwest` | MIT OR Apache-2.0 | YES | HTTP client; dual-licensed |

### 3.3 RustPress Core Dependencies

| Crate | Expected License | Compatible | Notes |
|-------|-----------------|------------|-------|
| `rustpress-core` | MIT | YES | Core platform; MIT licensed |
| `rustpress-database` | MIT | YES | Database layer; MIT licensed |
| `rustpress-auth` | MIT | YES | Authentication; MIT licensed |

### 3.4 Frontend Dependencies

| Package | Expected License | Compatible | Notes |
|---------|-----------------|------------|-------|
| React 18 | MIT | YES | UI library |
| TypeScript | Apache-2.0 | YES | Language/compiler |
| Tailwind CSS | MIT | YES | CSS framework |
| Zustand | MIT | YES | State management |
| Vite | MIT | YES | Build tool |
| Axios | MIT | YES | HTTP client |
| Lucide React | ISC | YES | Icons; ISC is MIT-compatible |

---

## 4. Stripe SDK Licensing — Detailed Review

### 4.1 `stripe-rust` Crate

- **License**: MIT OR Apache-2.0 (dual-licensed)
- **Maintainer**: Community-maintained (not official Stripe SDK)
- **Repository**: https://github.com/arlyon/async-stripe
- **Compatibility**: Fully compatible with MIT

### 4.2 Stripe Terms of Service

While the Stripe Rust SDK is open-source, usage of the Stripe API itself is subject to:

- **Stripe Services Agreement**: Governs API usage, not crate licensing
- **Stripe Acceptable Use Policy**: Restricts what can be sold via Stripe
- **PCI-DSS Compliance**: Required for payment processing (see COMPLIANCE_CHECKLIST.md)

**Recommendation**: Store operators using RustCommerce must agree to Stripe's Terms of Service independently. The plugin should display a notice during payment gateway configuration that the store operator is responsible for compliance with Stripe's terms.

### 4.3 Stripe Trademark Usage

- Do NOT use the Stripe logo without permission
- Referencing "Stripe" as a supported payment gateway in documentation is permissible (nominative fair use)
- Follow Stripe's brand guidelines for any storefront-facing "Powered by Stripe" badges

---

## 5. License Compatibility Matrix

| Dependency License | Compatible with MIT (as consumer)? | Notes |
|-------------------|-----------------------------------|-------|
| MIT | YES | Identical license family |
| Apache-2.0 | YES | Permissive; patent grant is additive |
| MIT OR Apache-2.0 | YES | Dual-licensed; can choose MIT path |
| ISC | YES | Functionally equivalent to MIT |
| BSD-2-Clause | YES | Permissive; compatible |
| BSD-3-Clause | YES | Permissive; compatible |
| MPL-2.0 | CONDITIONAL | File-level copyleft; compatible if source of MPL files is available |
| LGPL-2.1/3.0 | CONDITIONAL | Typically compatible with Rust (static linking nuances); review on case-by-case basis |
| GPL-2.0/3.0 | NO | Copyleft; would require RustCommerce to be GPL. AVOID. |
| AGPL-3.0 | NO | Network copyleft; even more restrictive than GPL. AVOID. |
| SSPL | NO | Not OSI-approved; highly restrictive. AVOID. |

---

## 6. Potential Risk Areas

### 6.1 Transitive Dependencies

The direct dependencies listed above each pull in their own dependency trees. Risks include:

- **Copyleft contamination**: A transitive dependency licensed under GPL/AGPL could create licensing obligations
- **License ambiguity**: Some crates may lack clear license declarations
- **License changes**: Upstream crates may change licenses in future versions

**Mitigation**: Implement `cargo-deny` for automated transitive dependency license scanning (see Section 8).

### 6.2 OpenSSL Licensing

Some Rust crates (notably `reqwest` and `sqlx`) can optionally depend on OpenSSL, which uses a dual OpenSSL/SSLeay license:

- **Risk**: OpenSSL license has an advertising clause that is technically incompatible with GPL (not relevant here since we are MIT)
- **Recommendation**: Prefer `rustls` (MIT/Apache-2.0/ISC) over OpenSSL where possible. Configure `reqwest` and `sqlx` with `rustls-tls` feature instead of `native-tls`.

### 6.3 Unicode/ICU Libraries

Some text-processing dependencies may include Unicode data files with their own license terms:

- **Risk**: LOW. Unicode license is permissive and compatible with MIT
- **Recommendation**: Include Unicode license notices in THIRD_PARTY_LICENSES if applicable

---

## 7. Compliance Actions Required

### 7.1 Immediate Actions

| # | Action | Priority | Status |
|---|--------|----------|--------|
| 1 | Verify LICENSE file is complete and includes full MIT text | HIGH | DONE (verified) |
| 2 | Update copyright year in LICENSE to 2024-2026 | MEDIUM | TODO |
| 3 | Add SPDX license identifier to all source files | LOW | TODO |
| 4 | Configure `cargo-deny` (see Section 8) | HIGH | TODO |

### 7.2 Pre-Release Actions

| # | Action | Priority | Status |
|---|--------|----------|--------|
| 5 | Generate THIRD_PARTY_LICENSES file from dependency tree | HIGH | TODO |
| 6 | Run full license audit on all transitive dependencies | HIGH | TODO |
| 7 | Verify no GPL/AGPL dependencies in transitive tree | HIGH | TODO |
| 8 | Add license attribution section to README | MEDIUM | TODO |
| 9 | Review frontend npm dependency licenses (`npx license-checker`) | HIGH | TODO |
| 10 | Add Stripe Terms of Service notice to payment settings UI | MEDIUM | TODO |

---

## 8. cargo-deny Configuration

### 8.1 Recommended `deny.toml`

Create a `deny.toml` file in the project root with the following configuration:

```toml
# deny.toml — License and dependency policy for RustCommerce

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
yanked = "warn"
notice = "warn"

[licenses]
unlicensed = "deny"
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-DFS-2016",
    "Unicode-3.0",
    "Zlib",
    "BSL-1.0",
    "CC0-1.0",
    "OpenSSL",
]
deny = [
    "GPL-2.0",
    "GPL-3.0",
    "AGPL-3.0",
    "SSPL-1.0",
    "EUPL-1.1",
    "EUPL-1.2",
]
copyleft = "warn"
allow-osi-fsf-free = "neither"
default = "deny"
confidence-threshold = 0.8

[[licenses.clarify]]
name = "ring"
expression = "MIT AND ISC AND OpenSSL"
license-files = [
    { path = "LICENSE", hash = 0xbd0eed23 },
]

[bans]
multiple-versions = "warn"
wildcards = "allow"
highlight = "all"

[sources]
unknown-registry = "warn"
unknown-git = "warn"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

### 8.2 CI Integration

Add to the CI pipeline:

```yaml
- name: Check licenses
  run: |
    cargo install cargo-deny
    cargo deny check licenses
    cargo deny check advisories
    cargo deny check bans
    cargo deny check sources
```

### 8.3 Frontend License Checking

For the React admin UI, add to `package.json` scripts:

```json
{
  "scripts": {
    "license-check": "npx license-checker --production --failOn 'GPL-2.0;GPL-3.0;AGPL-3.0;SSPL-1.0'"
  }
}
```

---

## 9. THIRD_PARTY_LICENSES File

Before release, generate a comprehensive third-party license file:

```bash
# Rust dependencies
cargo install cargo-about
cargo about generate about.hbs -o THIRD_PARTY_LICENSES_RUST.md

# Node/npm dependencies (frontend)
npx license-checker --production --csv > THIRD_PARTY_LICENSES_NPM.csv
```

This file MUST be included in all binary distributions of RustCommerce.

---

## 10. Ongoing Monitoring Recommendations

1. **Dependabot / Renovate**: Enable automated dependency updates to catch license changes
2. **CI Gate**: `cargo deny check` MUST pass on every PR
3. **Quarterly Review**: Audit the full dependency tree quarterly for license changes
4. **Upstream Monitoring**: Watch for license changes in critical dependencies (`stripe-rust`, `sqlx`, `axum`)
5. **SBOM Generation**: Consider generating Software Bill of Materials (SBOM) in CycloneDX or SPDX format for enterprise consumers

---

*This document should be reviewed and updated whenever dependencies are added or upgraded. Next review date: before Milestone 2 completion.*
