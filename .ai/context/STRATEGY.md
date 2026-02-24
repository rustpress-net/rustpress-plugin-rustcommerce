# Project Strategy Brief — RustCommerce Plugin

> This strategy defines the full plan for delivering a **production-ready e-commerce plugin** for RustPress CMS. AI agents should read this file together with the `CONTEXT_BASE.md` files from the core repositories to understand the complete picture.

---

## 1. Project Vision

**Project Name**: RustCommerce

**One-Line Vision**: A full-featured, high-performance e-commerce plugin for RustPress CMS that enables any RustPress site to become an online store.

**Problem Statement**: RustPress CMS currently has no functional e-commerce capability. The existing `rustcommerce` code is a skeleton with basic structs and a `plugin.json` configuration — zero actual business logic, no database integration, no API handlers, and no admin UI. Store owners who want to sell products online cannot do so within the RustPress ecosystem.

**Desired Outcome**: A complete, production-ready e-commerce plugin that handles products, categories, inventory, shopping cart, checkout, orders, payments, shipping, taxes, customer accounts, and a full admin dashboard — all built on top of the RustPress plugin architecture using its hook system, database layer, and admin UI framework.

---

## 2. Target Audience

**Primary Users**: Small-to-medium business owners who use RustPress as their CMS and want to add e-commerce functionality without leaving the platform.

**Secondary Users**: RustPress developers and agencies building custom stores for clients.

**User Personas**:
- **Store Admin (Sarah)**: Manages products, processes orders, configures shipping/taxes/payments from the admin dashboard. Needs an intuitive UI with bulk operations.
- **Developer (Marcus)**: Extends RustCommerce with custom payment gateways, shipping providers, or storefront themes. Needs clean APIs, hooks, and documentation.
- **Shopper (Customer)**: Browses products, adds to cart, checks out. Interacts via the storefront (theme-rendered) and REST API.

---

## 3. Core Features (Prioritized)

### Must-Have (P0) — MVP
1. **Product Management** — CRUD for products with title, description, price, SKU, images, categories, tags, stock tracking, product variants (size/color), and product status (draft/published/archived)
2. **Category/Tag System** — Hierarchical product categories and flat tags, leveraging RustPress's existing taxonomy system
3. **Shopping Cart** — Server-side persistent cart (for logged-in users) + client-side cart (guests), add/remove/update quantities, cart totals with tax/shipping preview
4. **Checkout Flow** — Multi-step checkout: shipping address → shipping method → payment → order confirmation. Guest checkout supported.
5. **Order Management** — Order creation, status workflow (Pending → Processing → Shipped → Delivered / Cancelled / Refunded), order history, order detail view, admin order management
6. **Payment Gateway Integration** — Stripe integration as the default payment processor. Extensible gateway interface for adding more.
7. **Inventory Management** — Stock tracking per product/variant, low-stock alerts, backorder configuration
8. **Store Settings** — Currency, tax configuration (flat rate + zone-based), shipping methods (flat rate, free over threshold, weight-based), store policies
9. **REST API** — Full CRUD API for all entities under `/api/v1/rustcommerce/` following RustPress API conventions
10. **Admin Dashboard** — Product editor, order list/detail, customer list, store dashboard with key metrics, settings pages
11. **Database Migrations** — PostgreSQL tables for products, variants, categories, cart, orders, order items, customers, addresses, payments, shipping, taxes, coupons, reviews
12. **Hook Integration** — Fire RustPress hooks on key events (order_created, payment_completed, product_updated, etc.) so other plugins can react

### Should-Have (P1) — Post-MVP
1. **Coupon/Discount System** — Percentage, fixed amount, free shipping, BOGO; auto-apply rules, usage limits
2. **Customer Reviews** — Star ratings + text reviews on products, moderation queue, verified buyer badges
3. **Wishlist** — Save products for later, share wishlist
4. **Search & Filtering** — Product search with faceted filtering (price range, category, attributes, rating)
5. **Email Notifications** — Order confirmation, shipping notification, order status updates (via RustPress email system)
6. **Product Import/Export** — CSV import/export for bulk product management
7. **Store Analytics** — Revenue, orders, average order value, best sellers, conversion funnel

### Nice-to-Have (P2) — Future
1. **Multiple Payment Gateways** — PayPal, Square, bank transfer
2. **Digital Products** — Downloadable products with secure download links
3. **Subscriptions** — Recurring payments and subscription products
4. **Multi-Currency** — Automatic currency conversion based on customer location
5. **Product Bundles** — Sell grouped products at a discount
6. **Abandoned Cart Recovery** — Email reminders for incomplete checkouts

---

## 4. Technical Constraints

**Required Tech Stack**:
- **Backend**: Rust (same workspace structure as RustPress core)
- **Frontend**: React 18 + TypeScript + Tailwind CSS (matching the admin UI tech stack)
- **Database**: PostgreSQL 16 (via sqlx, UUID primary keys, timestamps with timezone)
- **State Management**: Zustand 5.0 (matching admin UI pattern)
- **Build**: Cargo for Rust, Vite for frontend

**Hosting/Infrastructure**: Same as RustPress core — Docker-based deployment, runs alongside the main server

**Integrations**:
- RustPress Core Plugin System (Plugin trait, hooks, AppContext)
- RustPress Database (sqlx pool, migration system)
- RustPress Auth (JWT tokens for customer sessions, admin permissions)
- RustPress Admin UI (Zustand store, design system components, route registration)
- Stripe API (payment processing)
- RustPress Media (product images)

**Existing Codebase**:
- `rustpress-plugin-rustcommerce/` — Current repo with skeleton Rust code (Product, Cart, Order structs)
- `rustpress-core-base/plugins/rustcommerce/plugin.json` — Existing config defining routes, permissions, admin menu
- `rustpress-core-base/` — Core platform (see `.ai/context/CONTEXT_BASE.md`)
- `rustpress-core-admin-ui/` — Admin dashboard (see `.ai/context/CONTEXT_BASE.md`)

**Package Manager**: Cargo (Rust) + npm (Frontend)

---

## 5. Non-Functional Requirements

**Performance**:
- API response time < 100ms for product listings (cached)
- Checkout flow complete < 3 seconds end-to-end
- Support 100+ concurrent shoppers without degradation

**Security**:
- PCI-DSS awareness: Never store raw credit card data — delegate to Stripe
- CSRF protection on all checkout forms
- Rate limiting on checkout/payment endpoints
- Input validation on all user-submitted data (prevent SQL injection, XSS)
- Secure webhook verification for payment callbacks

**Scalability**:
- Horizontal scaling via RustPress's existing multi-instance support
- Cart stored server-side (Redis or PostgreSQL) for session portability
- Product listing cache invalidation on product updates

**Availability**:
- Graceful degradation: If payment gateway is down, queue orders for retry
- Stock reservation during checkout (10-minute hold)

**Accessibility**:
- Admin UI: Follow existing RustPress admin UI accessibility patterns
- Storefront: Semantic HTML, ARIA labels, keyboard navigation

---

## 6. Timeline & Milestones

### Milestone 1: Backend Foundation
- Database schema and migrations
- Product CRUD (models, repositories, handlers)
- Category system integration
- Plugin trait implementation and hook registration
- Basic REST API endpoints

### Milestone 2: Cart & Checkout
- Cart management (add, remove, update, totals calculation)
- Checkout flow (address, shipping, payment selection)
- Order creation from cart
- Stripe payment integration
- Inventory stock management

### Milestone 3: Admin Dashboard
- Store dashboard with metrics (revenue, orders, customers)
- Product editor (create/edit with variants, images, SEO)
- Order management (list, detail, status updates, refunds)
- Customer list and detail views
- Settings pages (general, payments, shipping, taxes)

### Milestone 4: Storefront & Polish
- Public storefront API endpoints (product listing, detail, search)
- Hook integration with other RustPress plugins
- Email notifications
- Coupon/discount system
- Reviews system
- Performance optimization and caching

### Milestone 5: Testing & Release
- Unit tests for all business logic
- Integration tests for API endpoints
- E2E tests for checkout flow
- Documentation
- Release packaging

---

## 7. Success Criteria

**Launch Criteria**:
- A customer can browse products, add to cart, complete checkout with Stripe, and receive an order confirmation
- An admin can manage products, view/process orders, and configure store settings
- The plugin activates cleanly in a fresh RustPress installation
- All P0 features are functional and tested
- No security vulnerabilities in OWASP Top 10

**KPIs**:
1. Checkout completion rate > 80% (once payment info entered)
2. Admin can create a product in < 2 minutes
3. API response time < 100ms for cached product listings
4. Zero payment data stored locally (Stripe handles all sensitive data)

**Definition of Done**: All P0 features implemented, tested (unit + integration), documented, and the plugin can be installed on any RustPress instance via the plugin system.

---

## 8. Reference & Inspiration

**Competitor/Reference Products**:
- WooCommerce (WordPress) — Feature scope reference
- Shopify — UX reference for checkout flow
- Medusa.js — Headless commerce API reference
- Saleor — Modern GraphQL commerce reference

**Technical References**:
- RustPress Core CONTEXT_BASE: `rustpress-core-base/.ai/context/CONTEXT_BASE.md`
- RustPress Admin UI CONTEXT_BASE: `rustpress-core-admin-ui/.ai/context/CONTEXT_BASE.md`
- Existing plugin.json: `rustpress-core-base/plugins/rustcommerce/plugin.json`
- Visual Queue Manager plugin (reference implementation): `rustpress-core-base/plugins/visual-queue-manager/`
- Stripe Rust SDK: `stripe-rust` crate
- Axum framework: `https://docs.rs/axum`

---

## 9. Out of Scope

**Explicitly NOT building**:
1. Custom storefront theme — The plugin provides API endpoints and admin UI only; storefront rendering is handled by themes
2. Custom email template builder — Uses RustPress's existing email/notification system
3. Multi-vendor/marketplace — Single-store only for v1
4. POS (Point of Sale) integration — Online only
5. ERP integration — No SAP, NetSuite, etc. connectors
6. Mobile app — API-first approach allows mobile apps later, but we don't build one now
7. Advanced reporting/BI — Basic store analytics only; advanced analytics via RustAnalytics plugin

---

## 10. Additional Context

### Repository Structure (Target)

The final plugin should have this structure across two locations:

**Backend Plugin** (`rustpress-plugin-rustcommerce/`):
```
rustpress-plugin-rustcommerce/
├── .ai/
│   └── context/
│       ├── CONTEXT_BASE.md     # This plugin's own context doc
│       └── STRATEGY.md         # This strategy document
├── .github/workflows/
│   ├── ci.yml
│   └── release.yml
├── src/
│   ├── lib.rs                  # Plugin entry point, Plugin trait impl
│   ├── plugin.rs               # RustCommercePlugin struct
│   ├── config.rs               # Plugin configuration
│   ├── error.rs                # Plugin-specific errors
│   ├── models/                 # Database models
│   │   ├── mod.rs
│   │   ├── product.rs          # Product, ProductVariant
│   │   ├── category.rs         # ProductCategory
│   │   ├── cart.rs             # Cart, CartItem
│   │   ├── order.rs            # Order, OrderItem, OrderStatus
│   │   ├── customer.rs         # Customer, Address
│   │   ├── payment.rs          # Payment, PaymentMethod
│   │   ├── shipping.rs         # ShippingMethod, ShippingZone
│   │   ├── tax.rs              # TaxRate, TaxZone
│   │   ├── coupon.rs           # Coupon, DiscountType
│   │   └── review.rs           # ProductReview
│   ├── repositories/           # Database access layer
│   │   ├── mod.rs
│   │   ├── product_repo.rs
│   │   ├── category_repo.rs
│   │   ├── cart_repo.rs
│   │   ├── order_repo.rs
│   │   ├── customer_repo.rs
│   │   └── ...
│   ├── services/               # Business logic layer
│   │   ├── mod.rs
│   │   ├── product_service.rs
│   │   ├── cart_service.rs
│   │   ├── checkout_service.rs
│   │   ├── order_service.rs
│   │   ├── payment_service.rs  # Stripe integration
│   │   ├── shipping_service.rs
│   │   ├── tax_service.rs
│   │   ├── inventory_service.rs
│   │   └── coupon_service.rs
│   ├── handlers/               # API route handlers (Axum)
│   │   ├── mod.rs
│   │   ├── product_handler.rs
│   │   ├── cart_handler.rs
│   │   ├── checkout_handler.rs
│   │   ├── order_handler.rs
│   │   ├── customer_handler.rs
│   │   ├── admin_handler.rs    # Admin-specific endpoints
│   │   └── webhook_handler.rs  # Stripe webhooks
│   ├── routes.rs               # Route definitions
│   ├── hooks.rs                # Hook registrations
│   └── middleware.rs           # Plugin-specific middleware
├── migrations/
│   ├── 00001_ecommerce_core.sql      # Products, categories, variants
│   ├── 00002_cart_and_orders.sql     # Cart, orders, order items
│   ├── 00003_customers.sql           # Customers, addresses
│   ├── 00004_payments.sql            # Payments, transactions
│   ├── 00005_shipping_and_tax.sql    # Shipping, tax zones/rates
│   ├── 00006_coupons.sql             # Coupons, discount rules
│   └── 00007_reviews.sql             # Product reviews
├── plugin.toml                       # Plugin manifest
├── Cargo.toml
├── README.md
├── LICENSE
└── CONTRIBUTING.md
```

**Admin UI** (inside `rustpress-core-admin-ui/`):
```
src/pages/plugins/rustcommerce/
├── index.tsx                         # Main entry / route definitions
├── components/
│   ├── Dashboard.tsx                 # Store dashboard with metrics
│   ├── ProductEditor.tsx             # Product create/edit form
│   ├── ProductList.tsx               # Product listing with filters
│   ├── OrderList.tsx                 # Order management table
│   ├── OrderDetail.tsx               # Single order view
│   ├── CustomerList.tsx              # Customer management
│   ├── CustomerDetail.tsx            # Single customer view
│   ├── CouponManager.tsx             # Coupon CRUD
│   ├── ReviewModeration.tsx          # Review moderation queue
│   ├── settings/
│   │   ├── GeneralSettings.tsx       # Currency, store info
│   │   ├── PaymentSettings.tsx       # Stripe config
│   │   ├── ShippingSettings.tsx      # Shipping methods/zones
│   │   ├── TaxSettings.tsx           # Tax rates/zones
│   │   └── EmailSettings.tsx         # Notification templates
│   └── widgets/
│       ├── RevenueChart.tsx          # Revenue over time
│       ├── OrderStatusPie.tsx        # Orders by status
│       ├── TopProducts.tsx           # Best sellers
│       └── RecentOrders.tsx          # Latest orders
├── stores/
│   └── commerceStore.ts             # Zustand store for commerce state
├── api/
│   └── commerceApi.ts               # API client for commerce endpoints
└── types/
    └── index.ts                     # TypeScript types for all commerce entities
```

### Coding Guidelines

**Rust Backend**:
- Follow RustPress core conventions: `async_trait`, `thiserror` errors, `serde` for all models
- All database queries via `sqlx` with compile-time checked queries where possible
- UUID v4 for all entity IDs
- Repository pattern: `models/` → `repositories/` → `services/` → `handlers/`
- Return `Result<T, Error>` from all functions
- Use RustPress's `AppContext` to access database pool, cache, config

**React Frontend**:
- Use design system components from `@/design-system`
- Zustand with `persist` middleware for state
- Lazy loading for all page components
- API calls through the shared Axios client (`@/api/client`)
- Follow existing admin UI naming and file organization conventions
- Lucide icons for all iconography
- Tailwind CSS for styling (no inline styles, no CSS modules)

### Dependencies on Core Repos

The plugin will need to:
1. **Import from `rustpress-core`**: `Plugin` trait, `PluginInfo`, `AppContext`, `HookRegistry`, `Error`, `Result`
2. **Import from `rustpress-database`**: `PgPool`, database types
3. **Import from `rustpress-auth`**: Permission checking, JWT validation
4. **Reference the admin UI**: Plugin UI components are developed in or deployed to the admin-ui

The `Cargo.toml` should declare dependencies on rustpress crates:
```toml
[dependencies]
rustpress-core = { path = "../rustpress-core-base/crates/rustpress-core" }
rustpress-database = { path = "../rustpress-core-base/crates/rustpress-database" }
rustpress-auth = { path = "../rustpress-core-base/crates/rustpress-auth" }
```

Or via git dependency:
```toml
[dependencies]
rustpress-core = { git = "https://github.com/rustpress-net/rustpress-core-base", path = "crates/rustpress-core" }
```
