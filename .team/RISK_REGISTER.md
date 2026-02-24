# Risk Register — RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Status**: Active
**Review Frequency**: Each wave boundary

---

## Risk Matrix Legend

**Probability**: H (High) / M (Medium) / L (Low)
**Impact**: H (High) / M (Medium) / L (Low)
**Severity** = Probability x Impact: Critical / Major / Moderate / Minor

| | Impact: High | Impact: Medium | Impact: Low |
|---|---|---|---|
| **Prob: High** | Critical | Major | Moderate |
| **Prob: Medium** | Major | Moderate | Minor |
| **Prob: Low** | Moderate | Minor | Minor |

---

## Active Risks

### R1: Stripe API Integration Complexity

| Field | Detail |
|-------|--------|
| **ID** | R1 |
| **Category** | Technical |
| **Description** | Stripe API integration (PaymentIntents, webhooks, refunds) is complex and has strict security requirements. The `stripe-rust` crate may lack features, have breaking changes, or insufficient documentation for production use cases. Webhook signature verification and idempotency handling add further complexity. |
| **Probability** | **M** (Medium) |
| **Impact** | **H** (High) — Payment is the most critical feature; failure blocks checkout |
| **Severity** | **Major** |
| **Mitigation** | 1. Evaluate `stripe-rust` crate maturity early in M2; if insufficient, plan for raw HTTP client with Stripe REST API. 2. Build a payment gateway abstraction layer so Stripe implementation can be swapped. 3. Use Stripe test mode extensively. 4. Implement comprehensive webhook retry logic. 5. Create a dedicated Stripe integration test suite. |
| **Contingency** | Fall back to raw `reqwest` HTTP calls to Stripe REST API if the Rust crate is inadequate. |
| **Owner** | Backend Lead |
| **Status** | Open |

---

### R2: RustPress Plugin API Stability

| Field | Detail |
|-------|--------|
| **ID** | R2 |
| **Category** | Technical / Dependency |
| **Description** | RustPress core is under active development. The Plugin trait, hook system, AppContext, and database layer may change between versions. Breaking changes in core could require significant rework of the RustCommerce plugin. |
| **Probability** | **M** (Medium) |
| **Impact** | **H** (High) — Breaks plugin loading, hooks, database access |
| **Severity** | **Major** |
| **Mitigation** | 1. Pin RustPress core dependency to a specific commit/tag. 2. Use the Plugin trait abstraction to isolate core dependencies. 3. Monitor RustPress core repository for breaking changes. 4. Maintain a compatibility test that validates plugin loading against core. 5. Establish communication channel with core team for advance notice of API changes. |
| **Contingency** | Fork and pin the specific core version needed; apply compatibility patches as needed. |
| **Owner** | Backend Lead |
| **Status** | Open |

---

### R3: Database Schema Evolution

| Field | Detail |
|-------|--------|
| **ID** | R3 |
| **Category** | Technical |
| **Description** | The e-commerce domain is complex. The initial database schema (7 migrations) may need significant changes as features are implemented and edge cases discovered. Schema migrations on tables with production data are risky and can cause downtime. |
| **Probability** | **H** (High) |
| **Impact** | **M** (Medium) — Schema changes require migration scripts and may break existing queries |
| **Severity** | **Major** |
| **Mitigation** | 1. Invest heavily in schema design during M1 (review against WooCommerce, Medusa, Saleor schemas). 2. Use additive-only migrations where possible (add columns, not remove). 3. All migrations must be reversible. 4. Use feature flags for schema-dependent features. 5. Test migrations against populated databases, not just empty ones. |
| **Contingency** | Create data migration scripts that transform existing data to new schema. Schedule maintenance windows for breaking migrations. |
| **Owner** | Backend Lead / Infrastructure Lead |
| **Status** | Open |

---

### R4: Frontend-Backend Contract Mismatches

| Field | Detail |
|-------|--------|
| **ID** | R4 |
| **Category** | Technical / Process |
| **Description** | The admin UI (React/TypeScript) and backend (Rust/Axum) are developed in parallel by different agents. API contracts may drift, causing runtime errors, type mismatches, or missing fields. This is especially risky for complex objects like orders and products with variants. |
| **Probability** | **H** (High) |
| **Impact** | **M** (Medium) — Causes integration bugs, delays at M3 |
| **Severity** | **Major** |
| **Mitigation** | 1. Define OpenAPI specs before implementation (API-first approach). 2. Generate TypeScript types from OpenAPI specs. 3. Run contract tests in CI that validate backend responses against OpenAPI. 4. Frontend API client auto-generated from specs. 5. Regular sync between Backend and Frontend leads. |
| **Contingency** | Dedicated integration sprint to resolve contract mismatches before M3 completion. |
| **Owner** | API Architect |
| **Status** | Open |

---

### R5: Checkout Flow Edge Cases

| Field | Detail |
|-------|--------|
| **ID** | R5 |
| **Category** | Business Logic |
| **Description** | The checkout flow has many edge cases: payment failure mid-checkout, stock running out between cart and payment, concurrent purchases of last item, network timeouts during Stripe calls, partial refunds, currency rounding errors, tax calculation discrepancies. |
| **Probability** | **H** (High) |
| **Impact** | **H** (High) — Directly impacts revenue, customer trust, and data integrity |
| **Severity** | **Critical** |
| **Mitigation** | 1. Implement stock reservation with 10-minute TTL (hold stock during checkout). 2. Use database transactions for cart-to-order conversion. 3. Implement idempotency keys for payment requests. 4. Handle all Stripe error codes with appropriate user messages. 5. Use pessimistic locking for inventory updates. 6. E2E tests for every known edge case. 7. Implement order reconciliation job. |
| **Contingency** | Manual order reconciliation process. Admin can manually adjust orders and inventory. |
| **Owner** | Backend Lead / QA Lead |
| **Status** | Open |

---

### R6: Performance Under Load

| Field | Detail |
|-------|--------|
| **ID** | R6 |
| **Category** | Non-Functional |
| **Description** | Product listing queries with joins (variants, categories, images, reviews) and checkout with multiple service calls (inventory check, tax calculation, shipping, payment) may not meet the < 100ms and < 3s targets respectively under 100+ concurrent users. |
| **Probability** | **M** (Medium) |
| **Impact** | **M** (Medium) — Poor performance degrades user experience and conversion |
| **Severity** | **Moderate** |
| **Mitigation** | 1. Implement response caching for product listings and category trees (Redis or in-memory). 2. Database indexes on all query-critical columns (product status, category, price, created_at). 3. Use database connection pooling (sqlx pool). 4. Lazy-load product images and reviews. 5. Paginate all list endpoints. 6. Run load tests early in Wave 3 to identify bottlenecks. |
| **Contingency** | Add read replicas for product queries. Implement CDN for static product data. |
| **Owner** | Infrastructure Lead / Backend Lead |
| **Status** | Open |

---

### R7: Security Vulnerabilities

| Field | Detail |
|-------|--------|
| **ID** | R7 |
| **Category** | Security |
| **Description** | E-commerce applications are high-value targets. Risks include: SQL injection via product search, XSS in product descriptions/reviews, CSRF on checkout forms, IDOR (Insecure Direct Object Reference) on order/customer endpoints, rate limiting bypass on payment endpoints, webhook spoofing. |
| **Probability** | **M** (Medium) |
| **Impact** | **H** (High) — Security breach impacts customer data, payments, and trust |
| **Severity** | **Major** |
| **Mitigation** | 1. Use sqlx compile-time checked queries to prevent SQL injection. 2. Sanitize all user input (product descriptions, reviews, addresses). 3. CSRF tokens on all state-changing forms. 4. Authorization checks on every endpoint (ownership validation). 5. Stripe webhook signature verification. 6. Rate limiting on auth, checkout, and payment endpoints. 7. OWASP Top 10 security audit in Wave 3. 8. Never store credit card data (PCI-DSS compliance via Stripe). |
| **Contingency** | Engage external security auditor if internal audit reveals critical issues. |
| **Owner** | QA Lead / Backend Lead |
| **Status** | Open |

---

### R8: RustPress Admin UI Integration

| Field | Detail |
|-------|--------|
| **ID** | R8 |
| **Category** | Technical / Dependency |
| **Description** | The admin dashboard is built as a plugin UI within `rustpress-core-admin-ui`. The plugin route registration, design system components, Zustand store patterns, and lazy loading may not work as expected or may require undocumented integration steps. |
| **Probability** | **M** (Medium) |
| **Impact** | **M** (Medium) — Delays M3, may require workarounds |
| **Severity** | **Moderate** |
| **Mitigation** | 1. Study the Visual Queue Manager plugin as a reference implementation. 2. Create a minimal "hello world" admin UI plugin early to validate integration. 3. Document the plugin UI registration process. 4. Use only design system components (no custom CSS that might break). 5. Test admin UI in isolation and integrated modes. |
| **Contingency** | Build admin UI as a standalone React app that communicates with the plugin API directly (bypass admin UI integration). |
| **Owner** | Frontend Lead |
| **Status** | Open |

---

### R9: Scope Creep from E-Commerce Complexity

| Field | Detail |
|-------|--------|
| **ID** | R9 |
| **Category** | Project Management |
| **Description** | E-commerce is a deep domain. Features like tax jurisdictions, international shipping, product variants matrix, coupon stacking rules, and refund workflows can balloon in complexity. Team members may add "just one more feature" that delays delivery. |
| **Probability** | **H** (High) |
| **Impact** | **M** (Medium) — Delays timeline, increases bug surface |
| **Severity** | **Major** |
| **Mitigation** | 1. Strict P0/P1/P2 prioritization (MVP first, enhancements later). 2. All feature requests go through PM triage. 3. "Simple first" implementations (flat-rate tax before zone-based, flat-rate shipping before weight-based). 4. Time-box each milestone. 5. Regular scope review at wave boundaries. 6. Defer P1 features (coupons, reviews, wishlist) to M4 only after M1-M3 are solid. |
| **Contingency** | Cut P1 features from initial release; ship as fast-follow updates. |
| **Owner** | Project Manager |
| **Status** | Open |

---

### R10: Test Environment and Data Complexity

| Field | Detail |
|-------|--------|
| **ID** | R10 |
| **Category** | QA / Infrastructure |
| **Description** | E-commerce testing requires realistic seed data (products with variants, populated carts, orders in various states, customer accounts), Stripe test environment configuration, and complex state setup for edge case testing. Setting up and maintaining test environments is time-consuming. |
| **Probability** | **M** (Medium) |
| **Impact** | **M** (Medium) — Slows testing, may miss edge cases |
| **Severity** | **Moderate** |
| **Mitigation** | 1. Create seed data scripts early (include in M1 deliverables). 2. Use Stripe test mode with test card numbers. 3. Build test fixtures/factories for all entities. 4. Containerize test environment with Docker Compose. 5. Automate test environment setup in CI. 6. Create a test data generator that produces realistic product catalogs. |
| **Contingency** | Manual test data creation; prioritize critical path testing over exhaustive edge cases. |
| **Owner** | QA Lead / Infrastructure Lead |
| **Status** | Open |

---

## Risk Summary

| ID | Risk | Prob | Impact | Severity | Owner |
|----|------|------|--------|----------|-------|
| R1 | Stripe API integration complexity | M | H | Major | Backend Lead |
| R2 | RustPress plugin API stability | M | H | Major | Backend Lead |
| R3 | Database schema evolution | H | M | Major | Backend / Infra |
| R4 | Frontend-backend contract mismatches | H | M | Major | API Architect |
| R5 | Checkout flow edge cases | H | H | **Critical** | Backend / QA |
| R6 | Performance under load | M | M | Moderate | Infra / Backend |
| R7 | Security vulnerabilities | M | H | Major | QA / Backend |
| R8 | RustPress admin UI integration | M | M | Moderate | Frontend Lead |
| R9 | Scope creep from e-commerce complexity | H | M | Major | PM |
| R10 | Test environment and data complexity | M | M | Moderate | QA / Infra |

### Severity Distribution
- **Critical**: 1 (R5)
- **Major**: 5 (R1, R2, R3, R4, R7, R9)
- **Moderate**: 3 (R6, R8, R10)
- **Minor**: 0

---

## Risk Review Log

| Date | Reviewer | Changes |
|------|----------|---------|
| 2026-02-24 | PM | Initial risk register created with 10 risks |

---

*This register is reviewed at each wave boundary. New risks are added as discovered. Closed risks are moved to an archive section.*
