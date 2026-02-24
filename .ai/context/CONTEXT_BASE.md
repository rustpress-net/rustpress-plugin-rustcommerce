# RustCommerce Plugin — AI Context Document

> **Purpose**: This document teaches an AI agent everything it needs to know about the RustCommerce plugin to implement, extend, or debug it. Read this FIRST, then read the STRATEGY.md in this same folder, then reference the core repo CONTEXT_BASE files only when you need deeper platform details.

---

## 1. What Is RustCommerce?

RustCommerce is an **e-commerce plugin for RustPress CMS** that adds online store capabilities: products, cart, checkout, orders, payments, shipping, taxes, and a full admin dashboard.

**Current State**: Skeleton — basic Rust structs exist (Product, Cart, Order) but no business logic, no database integration, no API handlers, and no admin UI have been implemented yet.

**Target State**: A fully functional e-commerce system as described in `STRATEGY.md`.

---

## 2. Context Dependencies

This plugin depends on two other RustPress repositories. Their `.ai/context/CONTEXT_BASE.md` files contain the detailed context you need:

| Repository | Purpose | CONTEXT_BASE Location |
|------------|---------|----------------------|
| [rustpress-core-base](https://github.com/rustpress-net/rustpress-core-base) | Backend platform: plugin system, hooks, database, API, auth | `.ai/context/CONTEXT_BASE.md` |
| [rustpress-core-admin-ui](https://github.com/rustpress-net/rustpress-core-admin-ui) | Frontend admin dashboard: React UI, design system, plugin UI framework | `.ai/context/CONTEXT_BASE.md` |

**When to read the core CONTEXT_BASE files**:
- When you need to understand the `Plugin` trait or hook system → read core-base CONTEXT_BASE Section 4
- When you need to understand how plugin routes work → read core-base CONTEXT_BASE Section 4.5
- When you need to build admin UI pages → read admin-ui CONTEXT_BASE Section 4 and 6
- When you need design system components → read admin-ui CONTEXT_BASE Section 5

**When to explore the full codebase** (beyond .ai/ folders):
- When you need exact function signatures not documented in CONTEXT_BASE
- When you need to see how an existing plugin (like Visual Queue Manager) implements a specific pattern
- When debugging integration issues between the plugin and core

---

## 3. Current Repository Structure

```
rustpress-plugin-rustcommerce/
├── .ai/
│   └── context/
│       ├── CONTEXT_BASE.md    # THIS FILE
│       └── STRATEGY.md        # Full project strategy and target architecture
├── .github/
│   └── workflows/
│       ├── ci.yml             # CI: check, test, fmt, clippy
│       └── release.yml        # Auto-release with version bumping
├── src/
│   ├── lib.rs                 # Entry point — exports modules, VERSION constant, init()
│   ├── products.rs            # Product struct (id, name, description, price, stock)
│   ├── cart.rs                # CartItem + Cart structs
│   └── orders.rs              # Order struct + OrderStatus enum
├── Cargo.toml                 # name: "rustcommerce", deps: serde, serde_json
├── README.md
├── LICENSE                    # MIT
├── CONTRIBUTING.md            # Branch strategy: development → release → main
└── .gitignore
```

### Current Code

**src/lib.rs**:
```rust
pub mod products;
pub mod cart;
pub mod orders;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn init() {
    println!("RustCommerce v{} initialized", VERSION);
}
```

**src/products.rs**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock: u32,
}
```

**src/cart.rs**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: u64,
    pub quantity: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Cart {
    pub items: Vec<CartItem>,
}
```

**src/orders.rs**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: u64,
    pub items: Vec<CartItem>,
    pub status: OrderStatus,
    pub total: f64,
}
```

**Cargo.toml**:
```toml
[package]
name = "rustcommerce"
version = "1.0.0"
edition = "2021"
description = "E-commerce plugin for RustPress CMS"
license = "MIT"

[lib]
name = "rustcommerce"
path = "src/lib.rs"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## 4. Existing Plugin Configuration (in rustpress-core-base)

There's a `plugin.json` file at `rustpress-core-base/plugins/rustcommerce/plugin.json` that defines the plugin's integration points. This is the **design contract** — implement what it declares:

### Routes Defined

**Public storefront** (base: `/store`):
- `/` — StoreFront (home)
- `/products` — Product listing
- `/products/:slug` — Product detail
- `/category/:slug` — Category page
- `/cart` — Shopping cart
- `/checkout` — Checkout flow
- `/order/thank-you/:orderId` — Order confirmation
- `/search` — Product search
- `/login`, `/register`, `/forgot-password`, `/reset-password/:token` — Auth pages
- `/account` — My account hub
- `/account/orders`, `/account/orders/:orderId` — Order history
- `/account/addresses` — Address management
- `/account/wishlist` — Wishlist
- `/account/settings` — Account settings

**Admin pages** (base: `/admin/store`):
- `/admin/store` — AdminDashboard
- `/admin/store/products` — AdminProducts
- `/admin/store/orders` — AdminOrders
- `/admin/store/customers` — AdminCustomers
- `/admin/store/templates` — AdminTemplates
- `/admin/store/settings` — AdminSettings

### API Endpoints Defined

All under `/api/v1/`:
| Endpoint | Methods |
|----------|---------|
| `/products` | GET, POST, PUT, DELETE |
| `/orders` | GET, POST, PUT |
| `/customers` | GET, POST, PUT, DELETE |
| `/cart` | GET, POST, PUT, DELETE |
| `/checkout` | POST |
| `/templates` | GET, POST, PUT, DELETE |
| `/payments` | POST |
| `/shipping` | GET, POST |
| `/inventory` | GET, PUT |
| `/analytics` | GET |

### Permissions Defined
- `manage_products`
- `manage_orders`
- `manage_customers`
- `manage_store_settings`
- `manage_store_templates`
- `view_store_reports`
- `manage_api_keys`

### Admin Menu
Position 30, icon: `shopping-cart`, with 6 items (Dashboard, Products, Orders, Customers, Templates, Settings)

### Settings
- Currency: USD ($)
- Tax: enabled
- Shipping: enabled
- Guest checkout: enabled
- Reviews: enabled
- Wishlist: enabled
- Compare: enabled (max 4)

---

## 5. How to Implement This Plugin — Architecture Guide

### 5.1 Backend Architecture (Rust)

Follow the **repository → service → handler** pattern used throughout RustPress:

```
Request → Handler (Axum) → Service (business logic) → Repository (database) → PostgreSQL
                                                    → External APIs (Stripe)
```

**Layer responsibilities**:
- **Models** (`src/models/`): Data structures matching database tables. Use `sqlx::FromRow`, `Serialize`, `Deserialize`, UUID primary keys.
- **Repositories** (`src/repositories/`): Database queries using `sqlx`. One repo per entity. Accept `&PgPool` or `&AppContext`.
- **Services** (`src/services/`): Business logic. Orchestrate repos, validate data, compute totals, call external APIs.
- **Handlers** (`src/handlers/`): Axum handler functions. Parse request, call service, return JSON response.
- **Routes** (`src/routes.rs`): Axum Router combining all handler routes under `/api/v1/rustcommerce/`.

### 5.2 Plugin Trait Implementation

```rust
use rustpress_core::{Plugin, PluginInfo, PluginState, AppContext, Result};
use async_trait::async_trait;

pub struct RustCommercePlugin {
    info: PluginInfo,
    state: std::sync::atomic::AtomicU8, // maps to PluginState
}

impl RustCommercePlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo {
                id: "rustcommerce".into(),
                name: "RustCommerce".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                description: "Full-featured e-commerce plugin for RustPress".into(),
                author: "RustPress".into(),
                license: "MIT".into(),
                homepage: Some("https://rustpress.io/plugins/rustcommerce".into()),
                repository: Some("https://github.com/rustpress-net/rustpress-plugin-rustcommerce".into()),
                tags: vec!["ecommerce".into(), "store".into(), "payments".into()],
                dependencies: vec![],
                min_rustpress_version: Some("0.4.0".into()),
            },
            state: Default::default(),
        }
    }
}

#[async_trait]
impl Plugin for RustCommercePlugin {
    fn info(&self) -> &PluginInfo { &self.info }

    async fn activate(&self, ctx: &AppContext) -> Result<()> {
        // 1. Run database migrations
        // 2. Register hooks (actions + filters)
        // 3. Seed default settings (currency, tax rates, etc.)
        // 4. Register scheduled tasks (e.g., abandoned cart cleanup)
        Ok(())
    }

    async fn deactivate(&self, ctx: &AppContext) -> Result<()> {
        // 1. Remove hooks
        // 2. Cancel scheduled tasks
        // Note: Do NOT drop database tables — data should persist
        Ok(())
    }

    async fn on_startup(&self, ctx: &AppContext) -> Result<()> {
        // 1. Initialize Stripe client
        // 2. Warm product cache
        // 3. Start inventory monitoring
        Ok(())
    }

    async fn on_shutdown(&self, ctx: &AppContext) -> Result<()> {
        // Cleanup resources
        Ok(())
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        // Return JSON schema for plugin settings UI
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "currency": { "type": "string", "default": "USD" },
                "stripe_publishable_key": { "type": "string" },
                "stripe_secret_key": { "type": "string", "format": "password" },
                "tax_enabled": { "type": "boolean", "default": true },
                "shipping_enabled": { "type": "boolean", "default": true },
                "guest_checkout": { "type": "boolean", "default": true }
            }
        }))
    }
}
```

### 5.3 Hook Registration

Register hooks in the `activate()` method:

```rust
async fn activate(&self, ctx: &AppContext) -> Result<()> {
    let hooks = ctx.get::<HookRegistry>()?;

    // Fire action when an order is created
    hooks.add_action("rustcommerce_order_created", callback, Priority::NORMAL, Some("rustcommerce"));

    // Filter post content to add product shortcodes
    hooks.add_filter::<String>("filter_the_content", callback, Priority::NORMAL, Some("rustcommerce"));

    // Listen for user events
    hooks.add_action("user_created", on_user_created_callback, Priority::NORMAL, Some("rustcommerce"));

    Ok(())
}
```

### 5.4 Database Schema (Key Tables)

```sql
-- Products
CREATE TABLE rc_products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    short_description TEXT,
    sku VARCHAR(100) UNIQUE,
    price DECIMAL(10,2) NOT NULL,
    compare_at_price DECIMAL(10,2),
    cost_price DECIMAL(10,2),
    status VARCHAR(20) NOT NULL DEFAULT 'draft', -- draft, published, archived
    product_type VARCHAR(50) DEFAULT 'simple',   -- simple, variable, grouped, digital
    featured BOOLEAN DEFAULT false,
    stock_quantity INTEGER DEFAULT 0,
    stock_status VARCHAR(20) DEFAULT 'in_stock', -- in_stock, out_of_stock, on_backorder
    low_stock_threshold INTEGER DEFAULT 5,
    weight DECIMAL(8,2),
    dimensions_length DECIMAL(8,2),
    dimensions_width DECIMAL(8,2),
    dimensions_height DECIMAL(8,2),
    tax_class VARCHAR(50) DEFAULT 'standard',
    meta JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Product Variants (for variable products: size, color, etc.)
CREATE TABLE rc_product_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    sku VARCHAR(100) UNIQUE,
    name VARCHAR(255) NOT NULL,
    price DECIMAL(10,2) NOT NULL,
    compare_at_price DECIMAL(10,2),
    stock_quantity INTEGER DEFAULT 0,
    attributes JSONB NOT NULL DEFAULT '{}',  -- {"color": "red", "size": "XL"}
    image_url TEXT,
    position INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Product Images
CREATE TABLE rc_product_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    alt_text VARCHAR(255),
    position INTEGER DEFAULT 0,
    is_primary BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Product Categories (leverages RustPress taxonomy system)
CREATE TABLE rc_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    parent_id UUID REFERENCES rc_categories(id),
    image_url TEXT,
    position INTEGER DEFAULT 0,
    product_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE rc_product_categories (
    product_id UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES rc_categories(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, category_id)
);

-- Shopping Cart
CREATE TABLE rc_carts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,                    -- NULL for guest carts
    session_id VARCHAR(255),         -- For guest cart association
    status VARCHAR(20) DEFAULT 'active', -- active, abandoned, converted
    coupon_code VARCHAR(100),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ           -- For cart expiration/cleanup
);

CREATE TABLE rc_cart_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cart_id UUID NOT NULL REFERENCES rc_carts(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES rc_products(id),
    variant_id UUID REFERENCES rc_product_variants(id),
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price DECIMAL(10,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Orders
CREATE TABLE rc_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_number VARCHAR(50) NOT NULL UNIQUE,  -- Human-readable: RC-00001
    user_id UUID,
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    -- pending, confirmed, processing, shipped, delivered, cancelled, refunded
    subtotal DECIMAL(10,2) NOT NULL,
    tax_total DECIMAL(10,2) NOT NULL DEFAULT 0,
    shipping_total DECIMAL(10,2) NOT NULL DEFAULT 0,
    discount_total DECIMAL(10,2) NOT NULL DEFAULT 0,
    grand_total DECIMAL(10,2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    billing_address JSONB NOT NULL,
    shipping_address JSONB NOT NULL,
    shipping_method VARCHAR(100),
    payment_method VARCHAR(100),
    payment_status VARCHAR(30) DEFAULT 'unpaid', -- unpaid, paid, partially_refunded, refunded
    stripe_payment_intent_id VARCHAR(255),
    coupon_code VARCHAR(100),
    customer_note TEXT,
    admin_note TEXT,
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ
);

CREATE TABLE rc_order_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES rc_orders(id) ON DELETE CASCADE,
    product_id UUID NOT NULL,
    variant_id UUID,
    product_name VARCHAR(255) NOT NULL,  -- Snapshot at time of order
    variant_name VARCHAR(255),
    sku VARCHAR(100),
    quantity INTEGER NOT NULL,
    unit_price DECIMAL(10,2) NOT NULL,
    subtotal DECIMAL(10,2) NOT NULL,
    tax_amount DECIMAL(10,2) DEFAULT 0,
    discount_amount DECIMAL(10,2) DEFAULT 0,
    total DECIMAL(10,2) NOT NULL,
    meta JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Customers (extends RustPress users)
CREATE TABLE rc_customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID UNIQUE,            -- Links to RustPress users table
    email VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    phone VARCHAR(50),
    total_orders INTEGER DEFAULT 0,
    total_spent DECIMAL(12,2) DEFAULT 0,
    average_order_value DECIMAL(10,2) DEFAULT 0,
    last_order_at TIMESTAMPTZ,
    notes TEXT,
    meta JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE rc_customer_addresses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES rc_customers(id) ON DELETE CASCADE,
    address_type VARCHAR(20) NOT NULL DEFAULT 'shipping', -- billing, shipping
    is_default BOOLEAN DEFAULT false,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    company VARCHAR(255),
    address_line_1 VARCHAR(255) NOT NULL,
    address_line_2 VARCHAR(255),
    city VARCHAR(100) NOT NULL,
    state VARCHAR(100),
    postal_code VARCHAR(20) NOT NULL,
    country VARCHAR(2) NOT NULL,    -- ISO 3166-1 alpha-2
    phone VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Payments
CREATE TABLE rc_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES rc_orders(id),
    payment_method VARCHAR(50) NOT NULL, -- stripe, paypal, manual
    status VARCHAR(30) NOT NULL DEFAULT 'pending',
    -- pending, processing, completed, failed, cancelled, refunded
    amount DECIMAL(10,2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    transaction_id VARCHAR(255),         -- External payment ID (Stripe charge ID)
    gateway_response JSONB,              -- Raw response from payment gateway
    refund_amount DECIMAL(10,2) DEFAULT 0,
    refund_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Shipping
CREATE TABLE rc_shipping_zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    countries TEXT[] NOT NULL DEFAULT '{}',   -- ISO 3166-1 alpha-2 codes
    regions TEXT[] DEFAULT '{}',
    postal_codes TEXT[] DEFAULT '{}',
    is_default BOOLEAN DEFAULT false,
    position INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE rc_shipping_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    zone_id UUID REFERENCES rc_shipping_zones(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    method_type VARCHAR(50) NOT NULL,  -- flat_rate, free_shipping, weight_based, price_based
    cost DECIMAL(10,2) DEFAULT 0,
    free_threshold DECIMAL(10,2),      -- Free shipping over this amount
    min_weight DECIMAL(8,2),
    max_weight DECIMAL(8,2),
    settings JSONB DEFAULT '{}',
    enabled BOOLEAN DEFAULT true,
    position INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tax
CREATE TABLE rc_tax_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    rate DECIMAL(6,4) NOT NULL,       -- e.g., 0.0825 for 8.25%
    country VARCHAR(2) NOT NULL,
    state VARCHAR(100),
    postal_code VARCHAR(20),
    city VARCHAR(100),
    tax_class VARCHAR(50) DEFAULT 'standard',
    compound BOOLEAN DEFAULT false,
    shipping BOOLEAN DEFAULT false,    -- Apply to shipping?
    priority INTEGER DEFAULT 1,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Coupons
CREATE TABLE rc_coupons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    discount_type VARCHAR(30) NOT NULL, -- percentage, fixed_cart, fixed_product, free_shipping
    discount_value DECIMAL(10,2) NOT NULL,
    minimum_spend DECIMAL(10,2),
    maximum_spend DECIMAL(10,2),
    usage_limit INTEGER,
    usage_count INTEGER DEFAULT 0,
    usage_limit_per_user INTEGER,
    product_ids UUID[] DEFAULT '{}',
    category_ids UUID[] DEFAULT '{}',
    excluded_product_ids UUID[] DEFAULT '{}',
    starts_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Reviews
CREATE TABLE rc_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    customer_id UUID REFERENCES rc_customers(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    title VARCHAR(255),
    content TEXT,
    status VARCHAR(20) DEFAULT 'pending', -- pending, approved, rejected, spam
    verified_purchase BOOLEAN DEFAULT false,
    helpful_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX idx_rc_products_slug ON rc_products(slug);
CREATE INDEX idx_rc_products_status ON rc_products(status);
CREATE INDEX idx_rc_products_sku ON rc_products(sku);
CREATE INDEX idx_rc_orders_user ON rc_orders(user_id);
CREATE INDEX idx_rc_orders_status ON rc_orders(status);
CREATE INDEX idx_rc_orders_number ON rc_orders(order_number);
CREATE INDEX idx_rc_carts_user ON rc_carts(user_id);
CREATE INDEX idx_rc_carts_session ON rc_carts(session_id);
CREATE INDEX idx_rc_customers_email ON rc_customers(email);
CREATE INDEX idx_rc_reviews_product ON rc_reviews(product_id);
CREATE INDEX idx_rc_reviews_status ON rc_reviews(status);
```

### 5.5 API Route Structure

```rust
// src/routes.rs
use axum::{Router, routing::{get, post, put, delete}};

pub fn commerce_routes(state: AppState) -> Router {
    Router::new()
        // Public storefront API
        .route("/products", get(handlers::product::list_products))
        .route("/products/:id", get(handlers::product::get_product))
        .route("/products/:slug", get(handlers::product::get_product_by_slug))
        .route("/categories", get(handlers::category::list_categories))
        .route("/cart", get(handlers::cart::get_cart))
        .route("/cart/items", post(handlers::cart::add_item))
        .route("/cart/items/:id", put(handlers::cart::update_item))
        .route("/cart/items/:id", delete(handlers::cart::remove_item))
        .route("/checkout", post(handlers::checkout::process_checkout))
        .route("/shipping/methods", get(handlers::shipping::available_methods))

        // Authenticated customer API
        .route("/orders", get(handlers::order::list_my_orders))
        .route("/orders/:id", get(handlers::order::get_my_order))
        .route("/account/addresses", get(handlers::customer::list_addresses))
        .route("/account/addresses", post(handlers::customer::add_address))
        .route("/wishlist", get(handlers::wishlist::get_wishlist))
        .route("/reviews", post(handlers::review::create_review))

        // Admin API (requires manage_* permissions)
        .route("/admin/products", post(handlers::admin::create_product))
        .route("/admin/products/:id", put(handlers::admin::update_product))
        .route("/admin/products/:id", delete(handlers::admin::delete_product))
        .route("/admin/orders", get(handlers::admin::list_orders))
        .route("/admin/orders/:id", put(handlers::admin::update_order_status))
        .route("/admin/customers", get(handlers::admin::list_customers))
        .route("/admin/analytics", get(handlers::admin::store_analytics))
        .route("/admin/settings", get(handlers::admin::get_settings))
        .route("/admin/settings", put(handlers::admin::update_settings))
        .route("/admin/coupons", get(handlers::admin::list_coupons))
        .route("/admin/coupons", post(handlers::admin::create_coupon))
        .route("/admin/inventory", get(handlers::admin::inventory_report))
        .route("/admin/inventory/:id", put(handlers::admin::update_stock))

        // Webhooks
        .route("/webhooks/stripe", post(handlers::webhook::stripe_webhook))

        .with_state(state)
}
```

### 5.6 Checkout Flow

```
1. Customer adds items to cart           → POST /cart/items
2. Customer views cart with totals       → GET /cart (includes tax + shipping estimate)
3. Customer initiates checkout           → POST /checkout/init
4. Collect shipping address              → POST /checkout/shipping-address
5. Calculate available shipping methods  → GET /shipping/methods?address=...
6. Select shipping method                → POST /checkout/shipping-method
7. Create Stripe PaymentIntent           → POST /checkout/payment-intent
8. Customer completes payment on client  → Stripe.js (frontend)
9. Stripe webhook confirms payment       → POST /webhooks/stripe
10. Order created, confirmation returned → GET /orders/:id
11. Email notification sent to customer  → (via RustPress notification system)
```

### 5.7 Frontend Admin UI Structure

The admin UI lives in `rustpress-core-admin-ui/src/pages/plugins/rustcommerce/`. Follow these patterns:

```typescript
// API client for commerce endpoints
// src/pages/plugins/rustcommerce/api/commerceApi.ts
import api from '@/api/client';

export const commerceApi = {
  // Products
  listProducts: (params?: any) => api.get('/v1/rustcommerce/admin/products', { params }),
  getProduct: (id: string) => api.get(`/v1/rustcommerce/admin/products/${id}`),
  createProduct: (data: any) => api.post('/v1/rustcommerce/admin/products', data),
  updateProduct: (id: string, data: any) => api.put(`/v1/rustcommerce/admin/products/${id}`, data),
  deleteProduct: (id: string) => api.delete(`/v1/rustcommerce/admin/products/${id}`),

  // Orders
  listOrders: (params?: any) => api.get('/v1/rustcommerce/admin/orders', { params }),
  getOrder: (id: string) => api.get(`/v1/rustcommerce/admin/orders/${id}`),
  updateOrderStatus: (id: string, status: string) => api.put(`/v1/rustcommerce/admin/orders/${id}`, { status }),

  // Dashboard
  getAnalytics: (params?: any) => api.get('/v1/rustcommerce/admin/analytics', { params }),

  // Settings
  getSettings: () => api.get('/v1/rustcommerce/admin/settings'),
  updateSettings: (data: any) => api.put('/v1/rustcommerce/admin/settings', data),
};
```

---

## 6. Implementation Order

Follow this order for maximum productivity (each step builds on the previous):

### Phase 1: Foundation
1. Update `Cargo.toml` with rustpress-core dependencies
2. Create `plugin.toml` manifest
3. Implement `RustCommercePlugin` struct with `Plugin` trait
4. Write database migration SQL files
5. Create model structs (matching database tables)

### Phase 2: Product System
6. Product repository (CRUD queries)
7. Product service (business logic, validation)
8. Product API handlers (list, get, create, update, delete)
9. Category system (reuse RustPress taxonomies where possible)
10. Product image management (via RustPress media system)

### Phase 3: Cart & Checkout
11. Cart repository + service + handlers
12. Tax calculation service
13. Shipping method service
14. Stripe payment integration (PaymentIntent flow)
15. Checkout handler (orchestrates cart → order conversion)
16. Webhook handler for Stripe events

### Phase 4: Order Management
17. Order repository + service + handlers
18. Order status workflow (state machine)
19. Inventory stock management (decrement on order, increment on cancel)
20. Customer management (profile, addresses, order history)

### Phase 5: Admin UI
21. Commerce Zustand store
22. Store dashboard with charts
23. Product editor (create/edit form with variants, images)
24. Product list with filtering and bulk actions
25. Order list and detail views
26. Customer list and detail views
27. Settings pages (general, payments, shipping, taxes)

### Phase 6: Polish
28. Coupon/discount system
29. Product reviews
30. Hook integration (fire events for other plugins)
31. Caching for product listings
32. Unit + integration tests
33. Documentation

---

## 7. Key Decisions Already Made

1. **Table prefix**: `rc_` (RustCommerce) — prevents collision with core tables
2. **IDs**: UUID v4 (matching RustPress convention)
3. **Money**: `DECIMAL(10,2)` — never use floats for money
4. **Order numbers**: Human-readable `RC-00001` format
5. **Payment**: Stripe-first (PaymentIntent API), extensible gateway interface
6. **Addresses**: Stored as JSONB on orders (snapshot), normalized in customer_addresses table
7. **Stock**: Managed per product and per variant
8. **State machine**: Order status transitions are enforced (can't go from Shipped → Pending)

---

## 8. Testing Strategy

- **Unit tests**: All services and business logic (tax calculation, cart totals, stock management)
- **Integration tests**: API endpoint tests with test database
- **E2E flow test**: Add to cart → checkout → payment → order confirmation
- **Use `mockall`** for mocking repositories in service tests
- **Use `sqlx::test`** for database integration tests with auto-rollback

---

## 9. Quick Reference

| Need | Where to look |
|------|--------------|
| Plugin trait / how plugins work | Core CONTEXT_BASE Section 4 |
| Hook system (actions/filters) | Core CONTEXT_BASE Section 5 |
| Database models & schema | Core CONTEXT_BASE Section 6 |
| API patterns / Axum handlers | Core CONTEXT_BASE Section 7 |
| Error handling | Core CONTEXT_BASE Section 8 |
| Admin UI plugin system | Admin UI CONTEXT_BASE Section 4 |
| Building plugin pages | Admin UI CONTEXT_BASE Section 6 |
| Design system components | Admin UI CONTEXT_BASE Section 5 |
| Existing plugin examples | Core: `plugins/visual-queue-manager/`, Admin: `src/pages/plugins/visual-queue-manager/` |
| Strategy & feature scope | This repo: `.ai/context/STRATEGY.md` |
