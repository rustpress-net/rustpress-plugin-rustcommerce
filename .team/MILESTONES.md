# Milestones — RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Status**: Active

---

## Overview

| Milestone | Name | Status | Dependencies |
|-----------|------|--------|--------------|
| **M1** | Backend Foundation | Planned | Wave 0 (Initialization) |
| **M2** | Cart & Checkout | Planned | M1 |
| **M3** | Admin Dashboard | Planned | M1, M2 (partial) |
| **M4** | Storefront & Polish | Planned | M1, M2, M3 |
| **M5** | Testing & Release | Planned | M1, M2, M3, M4 |

---

## M1: Backend Foundation

**Objective**: Establish the core backend infrastructure — database schema, product CRUD, category system, plugin integration, and REST API scaffolding.

### Deliverables

| # | Deliverable | Owner | Status |
|---|------------|-------|--------|
| M1.1 | Database schema design and 7 migration files | Backend | Planned |
| M1.2 | Product models (Product, ProductVariant) | Backend | Planned |
| M1.3 | Product repository (CRUD operations via sqlx) | Backend | Planned |
| M1.4 | Product service (business logic layer) | Backend | Planned |
| M1.5 | Product API handlers (Axum route handlers) | Backend | Planned |
| M1.6 | Category/Tag models and repository | Backend | Planned |
| M1.7 | Category system integration with RustPress taxonomy | Backend | Planned |
| M1.8 | Plugin trait implementation (`RustCommercePlugin`) | Backend | Planned |
| M1.9 | Hook registration for key events | Backend | Planned |
| M1.10 | Route definitions (`routes.rs`) | Backend | Planned |
| M1.11 | Error handling module (`error.rs`) | Backend | Planned |
| M1.12 | Configuration module (`config.rs`) | Backend | Planned |
| M1.13 | OpenAPI specification (v1 — products, categories) | API | Planned |
| M1.14 | CI/CD pipeline (GitHub Actions) | DevOps | Planned |
| M1.15 | Docker development environment | DevOps | Planned |
| M1.16 | Database provisioning (PostgreSQL 16) | Infra | Planned |

### Acceptance Criteria

- [ ] All 7 migration files execute cleanly against a fresh PostgreSQL 16 database
- [ ] Product CRUD operations work end-to-end (create, read, update, delete via API)
- [ ] Product variants can be created and associated with parent products
- [ ] Categories can be created in a hierarchical structure
- [ ] Tags can be assigned to products
- [ ] Plugin loads correctly in RustPress and registers hooks
- [ ] REST API responds under `/api/v1/rustcommerce/products` and `/api/v1/rustcommerce/categories`
- [ ] CI pipeline runs `cargo check`, `cargo clippy`, `cargo test` on every PR
- [ ] Docker compose spins up the plugin alongside RustPress core

### Dependencies

- RustPress core plugin API must be accessible (via git or path dependency)
- PostgreSQL 16 instance available for development
- GitHub Actions runners configured

---

## M2: Cart & Checkout

**Objective**: Implement the complete shopping experience — cart management, checkout flow, order creation, Stripe payment processing, and inventory tracking.

### Deliverables

| # | Deliverable | Owner | Status |
|---|------------|-------|--------|
| M2.1 | Cart models (Cart, CartItem) | Backend | Planned |
| M2.2 | Cart repository and service | Backend | Planned |
| M2.3 | Cart API handlers (add, remove, update, get totals) | Backend | Planned |
| M2.4 | Checkout service (multi-step orchestration) | Backend | Planned |
| M2.5 | Order models (Order, OrderItem, OrderStatus) | Backend | Planned |
| M2.6 | Order repository and service | Backend | Planned |
| M2.7 | Order creation from cart (cart -> order conversion) | Backend | Planned |
| M2.8 | Stripe payment integration (PaymentIntent, webhooks) | Backend | Planned |
| M2.9 | Payment models and service | Backend | Planned |
| M2.10 | Webhook handler for Stripe callbacks | Backend | Planned |
| M2.11 | Inventory/stock tracking service | Backend | Planned |
| M2.12 | Stock reservation during checkout (10-min hold) | Backend | Planned |
| M2.13 | Shipping method selection (flat rate, free threshold, weight) | Backend | Planned |
| M2.14 | Tax calculation service (flat rate + zone-based) | Backend | Planned |
| M2.15 | Customer models (Customer, Address) | Backend | Planned |
| M2.16 | Guest checkout support | Backend | Planned |
| M2.17 | OpenAPI specification (v1 — cart, checkout, orders) | API | Planned |

### Acceptance Criteria

- [ ] A guest user can add products to cart, view cart totals (with tax/shipping preview)
- [ ] A logged-in user has a persistent server-side cart
- [ ] Checkout flow completes: address -> shipping method -> payment -> confirmation
- [ ] Stripe PaymentIntent is created and processed successfully (test mode)
- [ ] Stripe webhooks are received and verified (payment_intent.succeeded, etc.)
- [ ] Order is created with correct line items, totals, shipping, and tax
- [ ] Order status transitions follow the defined workflow
- [ ] Stock is decremented upon successful order placement
- [ ] Stock is reserved during checkout and released if checkout is abandoned (10-min TTL)
- [ ] Low-stock alerts are triggered when inventory falls below threshold
- [ ] Checkout completes end-to-end in < 3 seconds

### Dependencies

- **M1** must be complete (database, product models, plugin infrastructure)
- Stripe test API keys must be configured
- Shipping and tax rate seed data available

---

## M3: Admin Dashboard

**Objective**: Build the full admin UI for store management — dashboard metrics, product editor, order management, customer views, and settings configuration.

### Deliverables

| # | Deliverable | Owner | Status |
|---|------------|-------|--------|
| M3.1 | Store dashboard with revenue/order/customer metrics | Frontend | Planned |
| M3.2 | Revenue chart widget (RevenueChart.tsx) | Frontend | Planned |
| M3.3 | Order status pie chart widget (OrderStatusPie.tsx) | Frontend | Planned |
| M3.4 | Top products widget (TopProducts.tsx) | Frontend | Planned |
| M3.5 | Recent orders widget (RecentOrders.tsx) | Frontend | Planned |
| M3.6 | Product editor (create/edit with variants, images, SEO) | Frontend | Planned |
| M3.7 | Product list with filters and search | Frontend | Planned |
| M3.8 | Order list with status filters | Frontend | Planned |
| M3.9 | Order detail view (items, status, refund action) | Frontend | Planned |
| M3.10 | Customer list view | Frontend | Planned |
| M3.11 | Customer detail view (orders, addresses) | Frontend | Planned |
| M3.12 | General settings page (currency, store info) | Frontend | Planned |
| M3.13 | Payment settings page (Stripe configuration) | Frontend | Planned |
| M3.14 | Shipping settings page (methods, zones) | Frontend | Planned |
| M3.15 | Tax settings page (rates, zones) | Frontend | Planned |
| M3.16 | Zustand commerce store (commerceStore.ts) | Frontend | Planned |
| M3.17 | API client for commerce endpoints (commerceApi.ts) | Frontend | Planned |
| M3.18 | TypeScript type definitions (types/index.ts) | Frontend | Planned |

### Acceptance Criteria

- [ ] Dashboard loads with real-time metrics (revenue, orders, customers, average order value)
- [ ] Admin can create a new product with variants, images, and SEO fields in < 2 minutes
- [ ] Admin can view, filter, and search the product list
- [ ] Admin can view all orders, filter by status, and drill into order details
- [ ] Admin can update order status (Processing, Shipped, Delivered, Cancelled)
- [ ] Admin can initiate refunds from the order detail view
- [ ] Admin can view customer list and individual customer details (with order history)
- [ ] All settings pages save and load configuration correctly
- [ ] Stripe API key can be configured from the payment settings page
- [ ] Admin UI uses RustPress design system components throughout
- [ ] All pages are lazy-loaded and perform well

### Dependencies

- **M1** must be complete (backend API endpoints for products, categories)
- **M2** partially complete (cart/order APIs needed for order management views)
- RustPress admin UI build pipeline available
- Design system components documented

---

## M4: Storefront & Polish

**Objective**: Complete the public-facing API, integrate with the broader RustPress ecosystem via hooks, add post-MVP features (coupons, reviews, email), and optimize performance.

### Deliverables

| # | Deliverable | Owner | Status |
|---|------------|-------|--------|
| M4.1 | Public product listing API (pagination, sorting, filtering) | Backend | Planned |
| M4.2 | Public product detail API (with variants, reviews, related) | Backend | Planned |
| M4.3 | Product search API (text search, faceted filtering) | Backend | Planned |
| M4.4 | Hook integration — fire events for other plugins | Backend | Planned |
| M4.5 | Email notifications (order confirmation, shipping, status) | Backend | Planned |
| M4.6 | Coupon/discount system (percentage, fixed, BOGO, free shipping) | Backend | Planned |
| M4.7 | Coupon management admin UI | Frontend | Planned |
| M4.8 | Product reviews system (ratings, text, moderation) | Backend | Planned |
| M4.9 | Review moderation admin UI | Frontend | Planned |
| M4.10 | Response caching (product listings, category trees) | Backend | Planned |
| M4.11 | Cache invalidation on product/category updates | Backend | Planned |
| M4.12 | Database query optimization (indexes, query plans) | Infra | Planned |
| M4.13 | Rate limiting on checkout/payment endpoints | Backend | Planned |

### Acceptance Criteria

- [ ] Public storefront API returns paginated product listings with filtering
- [ ] Product search returns relevant results with faceted filters (price, category, rating)
- [ ] Hooks fire correctly on: order_created, payment_completed, product_updated, order_status_changed
- [ ] Other RustPress plugins can subscribe to and react to commerce hooks
- [ ] Order confirmation email is sent upon successful checkout
- [ ] Shipping notification email is sent when order status changes to Shipped
- [ ] Coupons can be created, applied at checkout, and validated (expiry, usage limits)
- [ ] Product reviews can be submitted, moderated, and displayed
- [ ] Cached product listing API responds in < 100ms
- [ ] Cache is invalidated when products or categories are modified
- [ ] Checkout/payment endpoints are rate-limited (e.g., 10 requests/min per IP)

### Dependencies

- **M1, M2, M3** must be substantially complete
- RustPress email/notification system accessible
- Redis or equivalent cache layer available

---

## M5: Testing & Release

**Objective**: Achieve production readiness through comprehensive testing, documentation, and release packaging.

### Deliverables

| # | Deliverable | Owner | Status |
|---|------------|-------|--------|
| M5.1 | Unit tests for all models and services | QA / Backend | Planned |
| M5.2 | Integration tests for all API endpoints | QA | Planned |
| M5.3 | E2E tests for complete checkout flow | QA | Planned |
| M5.4 | Security audit (OWASP Top 10 checklist) | QA | Planned |
| M5.5 | Performance/load testing (100+ concurrent users) | QA / Infra | Planned |
| M5.6 | API documentation (OpenAPI + usage guides) | Docs | Planned |
| M5.7 | Admin user guide | Docs | Planned |
| M5.8 | Developer extension guide (custom gateways, hooks) | Docs | Planned |
| M5.9 | Release packaging (plugin bundle, install instructions) | DevOps | Planned |
| M5.10 | Changelog and release notes | PM | Planned |
| M5.11 | Plugin marketplace submission | Marketing | Planned |

### Acceptance Criteria

- [ ] Unit test coverage > 80% for business logic (services layer)
- [ ] All API endpoints have integration tests covering happy path + error cases
- [ ] E2E test simulates full checkout flow: browse -> add to cart -> checkout -> payment -> order confirmation
- [ ] No OWASP Top 10 vulnerabilities found in security audit
- [ ] System handles 100+ concurrent shoppers without degradation (< 200ms p95 response)
- [ ] API documentation is complete and published
- [ ] Admin guide covers all settings and workflow operations
- [ ] Developer guide includes examples for custom payment gateway and hook subscription
- [ ] Plugin installs cleanly on a fresh RustPress instance via the plugin system
- [ ] Release notes accurately describe all features, known issues, and upgrade path

### Dependencies

- **M1, M2, M3, M4** must be complete
- Test environments provisioned (staging with Stripe test mode)
- Documentation tooling available

---

## Milestone Dependency Graph

```
M1: Backend Foundation
 |
 +---> M2: Cart & Checkout
 |      |
 |      +---> M3: Admin Dashboard (partial M2 dependency)
 |             |
 +-------------+---> M4: Storefront & Polish
                      |
                      +---> M5: Testing & Release
```

**Critical Path**: M1 -> M2 -> M3 -> M4 -> M5

Note: M3 can begin in parallel with late M2 work (product editor depends on M1, order management depends on M2). M4 work on hooks and public API can begin once M1 is stable.

---

*Milestones are reviewed and updated at each wave boundary. Status updates are tracked in KANBAN.md.*

---

## Implementation Phase Milestones

> **Note**: The design phase milestones (M1-M5) above are **complete** -- all design artifacts, API contracts, schemas, component architectures, test strategies, and release plans have been produced. The following implementation milestones (IM1-IM5) represent the actual code implementation phase, where the designs are turned into working, compilable, deployable code.

---

## IM1: Backend Core Implementation

**Objective**: Implement the complete backend foundation with real, compilable Rust code -- all models, repositories, plugin infrastructure, and core modules.

### Deliverables

| # | Deliverable | Owner | Status |
|---|------------|-------|--------|
| IM1.1 | Update Cargo.toml with all required dependencies | Backend | Planned |
| IM1.2 | Create plugin.toml manifest file | Backend | Planned |
| IM1.3 | Create 7 database migration SQL files with real DDL | Backend | Planned |
| IM1.4 | Implement all model structs (Product, ProductVariant, Category, Cart, Order, Customer, Payment, Shipping, Tax, Coupon, Review) | Backend | Planned |
| IM1.5 | Implement repository layer (CRUD operations via sqlx for all entities) | Backend | Planned |
| IM1.6 | Implement plugin.rs (RustPress Plugin trait) | Backend | Planned |
| IM1.7 | Implement error.rs (error types and conversions) | Backend | Planned |
| IM1.8 | Implement config.rs (plugin configuration) | Backend | Planned |
| IM1.9 | Implement hooks.rs (hook registration and event firing) | Backend | Planned |
| IM1.10 | Implement middleware.rs (auth, logging, CORS) | Backend | Planned |
| IM1.11 | Implement routes.rs (Axum route definitions) | Backend | Planned |

### Acceptance Criteria

- [ ] `cargo build` succeeds with zero errors
- [ ] All model structs use proper types: UUID for IDs, Decimal for money, chrono for timestamps
- [ ] All 7 migration files contain valid PostgreSQL DDL
- [ ] Repository layer compiles with sqlx query macros
- [ ] Plugin trait implementation compiles against RustPress core API
- [ ] All modules are properly wired in lib.rs

### Dependencies

- RustPress core plugin API accessible (via git or path dependency)
- Design artifacts from M1-M5 (all complete)

---

## IM2: Backend Cart & Checkout

**Objective**: Implement the complete shopping experience backend -- cart management, checkout orchestration, order processing, Stripe payment integration, and supporting services.

### Deliverables

| # | Deliverable | Owner | Status |
|---|------------|-------|--------|
| IM2.1 | Cart service and API handlers (add, remove, update, totals) | Backend | Planned |
| IM2.2 | Checkout service (multi-step orchestration) | Backend | Planned |
| IM2.3 | Order service and API handlers (creation, status, refunds) | Backend | Planned |
| IM2.4 | Stripe payment integration (PaymentIntent, webhooks, signature verification) | Backend | Planned |
| IM2.5 | Inventory service (stock tracking, reservation, low-stock alerts) | Backend | Planned |
| IM2.6 | Shipping service (flat rate, free threshold, weight-based) | Backend | Planned |
| IM2.7 | Tax service (flat rate, zone-based calculation) | Backend | Planned |

### Acceptance Criteria

- [ ] All REST API endpoints respond correctly (cart, checkout, orders)
- [ ] Stripe test payments process successfully via PaymentIntent
- [ ] Stripe webhooks are received and verified
- [ ] Cart-to-order conversion works end-to-end
- [ ] Stock is reserved during checkout and decremented on order completion
- [ ] Shipping and tax calculations produce correct totals
- [ ] `cargo build` continues to succeed after all additions

### Dependencies

- **IM1** must be complete (models, repositories, plugin infrastructure)
- Stripe test API keys configured

---

## IM3: Admin Dashboard

**Objective**: Build the full React admin UI for store management -- dashboard metrics, product management, order management, customer views, and all settings pages.

### Deliverables

| # | Deliverable | Owner | Status |
|---|------------|-------|--------|
| IM3.1 | TypeScript type definitions (types/index.ts) | Frontend | Planned |
| IM3.2 | API client for commerce endpoints (commerceApi.ts) | Frontend | Planned |
| IM3.3 | Zustand commerce store (commerceStore.ts) | Frontend | Planned |
| IM3.4 | Dashboard page with metric widgets | Frontend | Planned |
| IM3.5 | ProductList page with filters and search | Frontend | Planned |
| IM3.6 | ProductEditor page (create/edit with variants, images, SEO) | Frontend | Planned |
| IM3.7 | OrderList page with status filters | Frontend | Planned |
| IM3.8 | OrderDetail page (items, timeline, refund action) | Frontend | Planned |
| IM3.9 | CustomerList page | Frontend | Planned |
| IM3.10 | CustomerDetail page (order history, addresses) | Frontend | Planned |
| IM3.11 | GeneralSettings, PaymentSettings, ShippingSettings, TaxSettings, EmailSettings pages | Frontend | Planned |
| IM3.12 | Route registration in App.tsx and lazy loading | Frontend | Planned |

### Acceptance Criteria

- [ ] `npm run build` succeeds with zero errors
- [ ] All pages render correctly with RustPress design system components
- [ ] Dashboard displays revenue, orders, customers, and average order value metrics
- [ ] Product editor supports variants, images, and SEO fields
- [ ] Order management supports status updates and refund initiation
- [ ] All settings pages save and load configuration correctly
- [ ] All page components are lazy-loaded

### Dependencies

- **IM1** must be complete (backend API endpoints)
- **IM2** must be complete (cart/order APIs for order management views)
- RustPress admin UI build pipeline and design system available

---

## IM4: Integration & Polish

**Objective**: Complete integration with the RustPress ecosystem, implement post-MVP features (coupons, reviews), and add performance optimizations (caching, search, email notifications).

### Deliverables

| # | Deliverable | Owner | Status |
|---|------------|-------|--------|
| IM4.1 | Hook integration (fire events for order_created, payment_completed, product_updated, etc.) | Backend | Planned |
| IM4.2 | Storefront public API (product listing, detail, search with pagination and facets) | Backend | Planned |
| IM4.3 | Coupon/discount system (percentage, fixed, BOGO, free shipping) | Backend | Planned |
| IM4.4 | Product reviews system (submission, moderation, verified buyer badges) | Backend/Frontend | Planned |
| IM4.5 | Email notifications (order confirmation, shipping, status updates) | Backend | Planned |
| IM4.6 | Response caching and cache invalidation | Backend | Planned |
| IM4.7 | Rate limiting on checkout/payment endpoints | Backend | Planned |

### Acceptance Criteria

- [ ] Hooks fire correctly on: order_created, payment_completed, product_updated, order_status_changed
- [ ] Other RustPress plugins can subscribe to commerce hooks
- [ ] Coupons can be created, applied at checkout, and validated (expiry, usage limits)
- [ ] Product reviews can be submitted, moderated, and displayed with verified buyer badges
- [ ] Order confirmation and shipping notification emails are sent
- [ ] Cached product listing API responds in < 100ms
- [ ] Cache is invalidated when products or categories are modified
- [ ] Checkout/payment endpoints are rate-limited

### Dependencies

- **IM1, IM2, IM3** must be substantially complete
- RustPress email/notification system accessible
- Redis or equivalent cache layer available

---

## IM5: QA, Screenshots & Release

**Objective**: Execute full QA testing with proof of evidence -- local install testing, screenshot capture, QA report generation, README update, and final presentation.

### Deliverables

| # | Deliverable | Owner | Status |
|---|------------|-------|--------|
| IM5.1 | Local install testing (plugin installs on fresh RustPress) | QA | Planned |
| IM5.2 | Execute 20+ test scenarios across all features | QA | Planned |
| IM5.3 | Capture 20+ screenshots as evidence | QA | Planned |
| IM5.4 | Create QA_TESTING_REPORT.md with all results | QA | Planned |
| IM5.5 | Update README.md with embedded screenshots | PM | Planned |
| IM5.6 | Generate final PPTX presentation | PM | Planned |

### Acceptance Criteria

- [ ] Plugin installs cleanly on a fresh RustPress instance
- [ ] 20+ screenshots captured covering all major features
- [ ] QA_TESTING_REPORT.md documents all test scenarios with pass/fail status
- [ ] README.md has embedded screenshots showing the plugin in action
- [ ] All tests pass (unit, integration, E2E)
- [ ] Final PPTX presentation is generated and complete

### Dependencies

- **IM1, IM2, IM3, IM4** must be complete
- Local RustPress instance available for testing
- Screenshot capture tooling available

---

## Implementation Milestone Dependency Graph

```
IM1: Backend Core
 |
 +---> IM2: Cart & Checkout
 |      |
 |      +---> IM3: Admin Dashboard
 |             |
 +-------------+---> IM4: Integration & Polish
                      |
                      +---> IM5: QA & Release
```

**Critical Path**: IM1 -> IM2 -> IM3 -> IM4 -> IM5

Note: IM3 can begin once IM2 is substantially complete (product pages only need IM1, order pages need IM2). IM4 hook integration can begin once IM1 is stable. IM5 requires all prior milestones to be complete.
