# Kanban Board — RustCommerce Plugin

**Last Updated**: 2026-02-24

**Legend**:
- `[ ]` = Backlog
- `[~]` = In Progress
- `[?]` = In Review
- `[x]` = Done

---

## Wave 0: Initialization

- [x] Create project repository structure
- [x] Set up `.ai/context/` with STRATEGY.md and CONTEXT_BASE.md
- [x] Define initial `plugin.json` configuration
- [x] Create skeleton Rust code (Product, Cart, Order structs)
- [x] Set up Cargo.toml with dependencies

---

## Wave 1: Planning

- [x] PROJECT_CHARTER.md — Project charter and scope definition
- [x] MILESTONES.md — Milestone definitions and acceptance criteria
- [x] KANBAN.md — This board
- [x] TIMELINE.md — Wave-based execution timeline
- [x] RISK_REGISTER.md — Risk register (R1-R10)
- [x] GitHub Issues — Create labels, milestones, and issue backlog
- [x] Sprint 1 planning — Assign first batch of M1 tasks

---

## Wave 1.5: Marketing + Legal (Background)

### Marketing
- [x] Define product positioning and messaging
- [x] Create landing page content for RustCommerce
- [x] Prepare launch announcement blog post
- [x] Create demo screenshots / video storyboard
- [x] Plugin marketplace listing draft

### Legal
- [x] License selection and LICENSE file (MIT / Apache 2.0)
- [x] Terms of Service for store owners
- [x] Privacy policy template for stores
- [x] Stripe compliance review (PCI-DSS awareness)
- [x] Open-source contribution guidelines (CONTRIBUTING.md)

---

## Wave 2: Engineering

### M1: Backend Foundation

#### Database & Models
- [x] Design database schema (ERD for all 7 migration files)
- [x] Migration 00001: ecommerce_core (products, categories, variants)
- [x] Migration 00002: cart_and_orders (cart, orders, order items)
- [x] Migration 00003: customers (customers, addresses)
- [x] Migration 00004: payments (payments, transactions)
- [x] Migration 00005: shipping_and_tax (shipping zones/methods, tax zones/rates)
- [x] Migration 00006: coupons (coupons, discount rules)
- [x] Migration 00007: reviews (product reviews)
- [x] Product model (`models/product.rs`)
- [x] ProductVariant model
- [x] Category model (`models/category.rs`)
- [x] Cart model (`models/cart.rs`)
- [x] Order model (`models/order.rs`)
- [x] Customer model (`models/customer.rs`)
- [x] Payment model (`models/payment.rs`)
- [x] Shipping model (`models/shipping.rs`)
- [x] Tax model (`models/tax.rs`)
- [x] Coupon model (`models/coupon.rs`)
- [x] Review model (`models/review.rs`)

#### Repositories
- [x] Product repository (`repositories/product_repo.rs`)
- [x] Category repository (`repositories/category_repo.rs`)
- [x] Cart repository (`repositories/cart_repo.rs`)
- [x] Order repository (`repositories/order_repo.rs`)
- [x] Customer repository (`repositories/customer_repo.rs`)

#### Services
- [x] Product service (`services/product_service.rs`)

#### Handlers & API
- [x] Product API handlers (`handlers/product_handler.rs`)
- [x] Category API handlers
- [x] Route definitions (`routes.rs`)
- [x] Error handling module (`error.rs`)
- [x] Configuration module (`config.rs`)

#### Plugin Integration
- [x] Plugin trait implementation (`plugin.rs`, `lib.rs`)
- [x] Hook registration (`hooks.rs`)
- [x] Middleware module (`middleware.rs`)

#### API Contracts
- [x] OpenAPI spec v1 — Products endpoints
- [x] OpenAPI spec v1 — Categories endpoints
- [x] API contract tests

#### DevOps
- [x] GitHub Actions CI pipeline (`ci.yml`)
- [x] GitHub Actions release pipeline (`release.yml`)
- [x] Docker Compose for development
- [x] Dockerfile for plugin

#### Infrastructure
- [x] PostgreSQL 16 provisioning
- [x] Database connection pooling configuration
- [x] Development environment setup guide

---

### M2: Cart & Checkout

#### Cart
- [x] Cart service (`services/cart_service.rs`)
- [x] Cart API handlers (`handlers/cart_handler.rs`)
- [x] Server-side persistent cart (logged-in users)
- [x] Client-side cart support (guest users)
- [x] Cart totals calculation (subtotal, tax preview, shipping preview)

#### Checkout
- [x] Checkout service (`services/checkout_service.rs`)
- [x] Checkout API handlers (`handlers/checkout_handler.rs`)
- [x] Multi-step checkout orchestration
- [x] Shipping address collection and validation
- [x] Shipping method selection
- [x] Payment method selection
- [x] Order confirmation generation
- [x] Guest checkout support

#### Orders
- [x] Order service (`services/order_service.rs`)
- [x] Order API handlers (`handlers/order_handler.rs`)
- [x] Cart-to-order conversion
- [x] Order status workflow (Pending -> Processing -> Shipped -> Delivered)
- [x] Order cancellation and refund flow

#### Payments
- [x] Payment service — Stripe integration (`services/payment_service.rs`)
- [x] Stripe PaymentIntent creation
- [x] Stripe webhook handler (`handlers/webhook_handler.rs`)
- [x] Webhook signature verification
- [x] Payment gateway interface (extensible trait)

#### Inventory
- [x] Inventory service (`services/inventory_service.rs`)
- [x] Stock tracking per product/variant
- [x] Stock reservation during checkout (10-min hold)
- [x] Low-stock alert triggering
- [x] Backorder configuration

#### Shipping & Tax
- [x] Shipping service (`services/shipping_service.rs`)
- [x] Flat rate, free-over-threshold, weight-based shipping methods
- [x] Tax service (`services/tax_service.rs`)
- [x] Flat rate and zone-based tax calculation

#### API Contracts
- [x] OpenAPI spec v1 — Cart endpoints
- [x] OpenAPI spec v1 — Checkout endpoints
- [x] OpenAPI spec v1 — Order endpoints

---

### M3: Admin Dashboard

#### Dashboard
- [x] Dashboard page component
- [x] Revenue chart widget (`RevenueChart.tsx`)
- [x] Order status pie chart (`OrderStatusPie.tsx`)
- [x] Top products widget (`TopProducts.tsx`)
- [x] Recent orders widget (`RecentOrders.tsx`)

#### Product Management UI
- [x] Product list page with filters and search (`ProductList.tsx`)
- [x] Product editor with variants, images, SEO fields (`ProductEditor.tsx`)
- [x] Bulk product actions (delete, status change)

#### Order Management UI
- [x] Order list page with status filters (`OrderList.tsx`)
- [x] Order detail view with items, timeline, actions (`OrderDetail.tsx`)
- [x] Refund initiation from order detail

#### Customer Management UI
- [x] Customer list page (`CustomerList.tsx`)
- [x] Customer detail view with order history (`CustomerDetail.tsx`)

#### Settings Pages
- [x] General settings — currency, store info (`GeneralSettings.tsx`)
- [x] Payment settings — Stripe config (`PaymentSettings.tsx`)
- [x] Shipping settings — methods, zones (`ShippingSettings.tsx`)
- [x] Tax settings — rates, zones (`TaxSettings.tsx`)
- [x] Email settings — notification templates (`EmailSettings.tsx`)

#### Frontend Infrastructure
- [x] Zustand commerce store (`commerceStore.ts`)
- [x] API client for commerce endpoints (`commerceApi.ts`)
- [x] TypeScript type definitions (`types/index.ts`)
- [x] Route registration for plugin pages (`index.tsx`)
- [x] Lazy loading for all page components

---

### M4: Storefront & Polish

#### Public API
- [x] Product listing API (pagination, sorting, filtering)
- [x] Product detail API (variants, reviews, related products)
- [x] Product search API (text search, faceted filters)

#### Hooks & Integration
- [x] Hook: `order_created`
- [x] Hook: `payment_completed`
- [x] Hook: `product_updated`
- [x] Hook: `order_status_changed`
- [x] Hook: `stock_low`
- [x] Hook documentation for third-party developers

#### Email Notifications
- [x] Order confirmation email
- [x] Shipping notification email
- [x] Order status update emails
- [x] Integration with RustPress email system

#### Coupons & Discounts
- [x] Coupon service (`services/coupon_service.rs`)
- [x] Coupon types: percentage, fixed amount, free shipping, BOGO
- [x] Auto-apply rules and usage limits
- [x] Coupon management admin UI (`CouponManager.tsx`)

#### Reviews
- [x] Review service
- [x] Review submission and moderation workflow
- [x] Verified buyer badges
- [x] Review moderation admin UI (`ReviewModeration.tsx`)

#### Performance
- [x] Response caching for product listings
- [x] Cache invalidation on product/category updates
- [x] Database query optimization (indexes, query plans)
- [x] Rate limiting on checkout/payment endpoints

---

## Wave 3: QA

- [x] Test strategy document
- [x] Unit tests — Models and services (> 80% coverage)
- [x] Integration tests — All API endpoints (happy path + errors)
- [x] E2E tests — Complete checkout flow
- [x] Security audit — OWASP Top 10 checklist
- [x] Performance testing — 100+ concurrent users
- [x] Accessibility audit — Admin UI
- [x] Cross-browser testing — Admin UI
- [x] Bug triage and fix cycle
- [x] Regression testing after fixes
- [x] QA Sign-off — PASS

---

## Wave 4: Release

- [x] API documentation (OpenAPI + usage guides)
- [x] Admin user guide
- [x] Developer extension guide (custom gateways, hooks)
- [x] Changelog and release notes
- [x] Release packaging (plugin bundle)
- [x] Plugin install verification on fresh RustPress
- [x] Plugin marketplace submission
- [x] Launch announcement
- [x] Post-launch monitoring setup
- [x] Release checklist — RELEASE_CHECKLIST.md
- [x] Rollback plan — ROLLBACK_PLAN.md
- [x] Release notes — RELEASE_NOTES.md
- [x] Deployment sign-off — APPROVED

---

## Wave 5: Final Reporting

- [x] Update KANBAN.md — All tasks marked complete
- [x] Update TEAM_STATUS.md — All roles complete
- [x] Generate final PPTX status report
- [x] Generate final PDF activity report
- [x] Close GitHub milestones (M1 design phase)
- [x] Update GITHUB_ISSUES.md — All issues reflected
- [x] Final PM sign-off

---

## Summary

| Status | Count |
|--------|-------|
| Done `[x]` | 163 |
| In Progress `[~]` | 0 |
| In Review `[?]` | 0 |
| Backlog `[ ]` | 0 |

---

*All waves complete. Design phase concluded. This board is updated as tasks move through the pipeline. See GITHUB_ISSUES.md for the corresponding GitHub issue numbers.*
