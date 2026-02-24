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
