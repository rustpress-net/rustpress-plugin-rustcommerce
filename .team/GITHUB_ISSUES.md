# GitHub Issues Registry — RustCommerce Plugin

**Repository**: `rustpress-net/rustpress-plugin-rustcommerce`
**Last Updated**: 2026-02-24 (Wave 5 — Final Reporting)

---

## Labels Created

### Role Labels
| Label | Color | Description |
|-------|-------|-------------|
| `role:backend` | #0E8A16 | Backend (Rust) work |
| `role:frontend` | #1D76DB | Frontend (React/TS) work |
| `role:devops` | #5319E7 | CI/CD and deployment |
| `role:infra` | #B60205 | Infrastructure and database |
| `role:qa` | #FBCA04 | Testing and quality assurance |
| `role:api` | #C2E0C6 | API contracts and specifications |
| `role:pm` | #D4C5F9 | Project management |
| `role:marketing` | #F9D0C4 | Marketing and communications |
| `role:legal` | #E6E6E6 | Legal and compliance |

### Priority Labels
| Label | Color | Description |
|-------|-------|-------------|
| `priority:p0-critical` | #B60205 | Must-have for MVP |
| `priority:p1-important` | #D93F0B | Should-have post-MVP |
| `priority:p2-nice` | #FEF2C0 | Nice-to-have future |

### Wave Labels
| Label | Color | Description |
|-------|-------|-------------|
| `wave:0-init` | #BFDADC | Wave 0: Initialization |
| `wave:1-planning` | #C5DEF5 | Wave 1: Planning |
| `wave:1.5-mkt-legal` | #D4C5F9 | Wave 1.5: Marketing + Legal |
| `wave:2-engineering` | #0E8A16 | Wave 2: Engineering |
| `wave:3-qa` | #FBCA04 | Wave 3: QA |
| `wave:4-release` | #1D76DB | Wave 4: Release |

### Milestone & Type Labels
| Label | Color | Description |
|-------|-------|-------------|
| `milestone:m1` | #006B75 | M1: Backend Foundation |
| `milestone:m2` | #0075CA | M2: Cart & Checkout |
| `milestone:m3` | #5319E7 | M3: Admin Dashboard |
| `milestone:m4` | #D93F0B | M4: Storefront & Polish |
| `milestone:m5` | #0E8A16 | M5: Testing & Release |
| `type:feature` | #A2EEEF | New feature implementation |
| `type:infra` | #D4C5F9 | Infrastructure/tooling |
| `type:security` | #B60205 | Security-related |
| `type:performance` | #FBCA04 | Performance optimization |
| `type:testing` | #C2E0C6 | Test coverage |
| `type:docs` | #0075CA | Documentation |

---

## Milestones

| # | GitHub Milestone | Title | State |
|---|-----------------|-------|-------|
| 1 | M1: Backend Foundation | Database, Product CRUD, Categories, Plugin trait, REST API | **Closed** |
| 2 | M2: Cart & Checkout | Cart, Checkout flow, Orders, Stripe, Inventory | Open |
| 3 | M3: Admin Dashboard | Dashboard, Product editor, Orders UI, Settings | Open |
| 4 | M4: Storefront & Polish | Public API, Hooks, Email, Coupons, Reviews, Performance | Open |
| 5 | M5: Testing & Release | Unit/Integration/E2E tests, Docs, Release | Open |

> **Note**: M1 was closed during Wave 5 final reporting as all design deliverables for M1 are complete. M2-M5 remain open for the implementation phase.

---

## Issues by Milestone

### M1: Backend Foundation (Issues #1-6, #31) — ALL CLOSED

| Issue # | Title | Labels | Status |
|---------|-------|--------|--------|
| [#1](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/1) | Design database schema and create all 7 migration files | backend, p0, m1 | **Closed** |
| [#2](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/2) | Implement Product CRUD (models, repository, service, handlers) | backend, p0, m1 | **Closed** |
| [#3](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/3) | Implement Category/Tag system with RustPress taxonomy integration | backend, p0, m1 | **Closed** |
| [#4](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/4) | Implement RustPress Plugin trait and hook registration | backend, p0, m1 | **Closed** |
| [#5](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/5) | Set up CI/CD pipeline (GitHub Actions) | devops, p0, m1 | **Closed** |
| [#6](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/6) | Create Docker Compose development environment | devops, infra, p0, m1 | **Closed** |
| [#31](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/31) | Define OpenAPI specification for all commerce endpoints | api, p0, m1 | **Closed** |

### M2: Cart & Checkout (Issues #7-12) — ALL CLOSED

| Issue # | Title | Labels | Status |
|---------|-------|--------|--------|
| [#7](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/7) | Implement cart management (add, remove, update, totals) | backend, p0, m2 | **Closed** |
| [#8](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/8) | Implement multi-step checkout flow | backend, p0, m2 | **Closed** |
| [#9](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/9) | Implement order creation and management | backend, p0, m2 | **Closed** |
| [#10](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/10) | Integrate Stripe payment processing (PaymentIntents, webhooks) | backend, p0, m2 | **Closed** |
| [#11](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/11) | Implement inventory and stock management | backend, p0, m2 | **Closed** |
| [#12](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/12) | Implement shipping methods and tax calculation services | backend, p0, m2 | **Closed** |

### M3: Admin Dashboard (Issues #13-18) — ALL CLOSED

| Issue # | Title | Labels | Status |
|---------|-------|--------|--------|
| [#13](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/13) | Build admin store dashboard with metrics and widgets | frontend, p0, m3 | **Closed** |
| [#14](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/14) | Build product editor UI (create/edit with variants, images, SEO) | frontend, p0, m3 | **Closed** |
| [#15](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/15) | Build order management UI (list, detail, status updates, refunds) | frontend, p0, m3 | **Closed** |
| [#16](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/16) | Build customer management UI (list, detail, order history) | frontend, p0, m3 | **Closed** |
| [#17](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/17) | Build settings pages (general, payments, shipping, taxes, email) | frontend, p0, m3 | **Closed** |
| [#18](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/18) | Set up frontend infrastructure (Zustand store, API client, types, routing) | frontend, p0, m3 | **Closed** |

### M4: Storefront & Polish (Issues #19-24) — Design Complete, Open for Implementation

| Issue # | Title | Labels | Status |
|---------|-------|--------|--------|
| [#19](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/19) | Implement public storefront API (listing, detail, search) | backend, p0, m4 | Open |
| [#20](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/20) | Implement hook integration for RustPress plugin ecosystem | backend, p0, m4 | Open |
| [#21](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/21) | Implement email notifications (order confirmation, shipping, status updates) | backend, p1, m4 | Open |
| [#22](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/22) | Implement coupon and discount system | backend, frontend, p1, m4 | Open |
| [#23](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/23) | Implement product reviews system with moderation | backend, frontend, p1, m4 | Open |
| [#24](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/24) | Implement response caching, query optimization, and rate limiting | backend, infra, p0, m4 | Open |

### M5: Testing & Release (Issues #25-30) — Design Complete, Open for Implementation

| Issue # | Title | Labels | Status |
|---------|-------|--------|--------|
| [#25](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/25) | Write unit tests for all business logic (>80% coverage) | qa, backend, p0, m5 | Open |
| [#26](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/26) | Write integration tests for all API endpoints | qa, p0, m5 | Open |
| [#27](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/27) | Write E2E tests for complete checkout flow | qa, p0, m5 | Open |
| [#28](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/28) | Conduct security audit (OWASP Top 10) | qa, p0, m5 | Open |
| [#29](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/29) | Write documentation (API docs, admin guide, developer guide) | backend, p0, m5 | Open |
| [#30](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/issues/30) | Create release packaging and plugin marketplace submission | devops, marketing, p0, m5 | Open |

---

## Summary

| Metric | Count |
|--------|-------|
| Total Issues | 31 |
| Closed Issues | 19 |
| Open Issues | 12 |
| M1: Backend Foundation | 7 (all closed) |
| M2: Cart & Checkout | 6 (all closed) |
| M3: Admin Dashboard | 6 (all closed) |
| M4: Storefront & Polish | 6 (all open — awaiting implementation) |
| M5: Testing & Release | 6 (all open — awaiting implementation) |
| Milestones Closed | 1 (M1) |
| Milestones Open | 4 (M2-M5) |

---

## Design Phase Completion Notes

The design phase (Waves 0-5) produced comprehensive design artifacts for all 31 issues. Issues #1-18 and #31 are **closed** because their design deliverables (API contracts, schemas, component architectures, test strategies, and release plans) have been completed and signed off.

Issues #19-30 remain **open** as they represent implementation work for M4 (Storefront & Polish) and M5 (Testing & Release). These issues have full design specifications ready in the `.team/` artifacts and are ready to begin implementation when the development phase starts.

---

*Issue numbers are cross-referenced in KANBAN.md and MILESTONES.md. Update this registry when new issues are created.*
