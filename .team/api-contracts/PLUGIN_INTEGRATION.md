# RustCommerce Plugin Integration

**Version**: 1.0.0
**Date**: 2026-02-24
**Status**: Approved

---

## Table of Contents

1. [Plugin Trait Implementation](#1-plugin-trait-implementation)
2. [Hook Registrations](#2-hook-registrations)
3. [Route Registration](#3-route-registration)
4. [Migration Integration](#4-migration-integration)
5. [Cache Strategy](#5-cache-strategy)
6. [Event Bus Integration](#6-event-bus-integration)
7. [Background Jobs](#7-background-jobs)
8. [Plugin Configuration](#8-plugin-configuration)
9. [Dependency Management](#9-dependency-management)
10. [Admin UI Integration](#10-admin-ui-integration)

---

## 1. Plugin Trait Implementation

### 1.1 RustCommercePlugin Struct

```rust
// src/plugin.rs

use rustpress_core::{
    Plugin, PluginInfo, PluginState, AppContext, Result, Error,
};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU8, Ordering};

pub struct RustCommercePlugin {
    info: PluginInfo,
    state: AtomicU8,
}

impl RustCommercePlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo {
                id: "rustcommerce".into(),
                name: "RustCommerce".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                description: "Full-featured e-commerce plugin for RustPress CMS".into(),
                author: "RustPress".into(),
                license: "MIT".into(),
                homepage: Some("https://rustpress.io/plugins/rustcommerce".into()),
                repository: Some("https://github.com/rustpress-net/rustpress-plugin-rustcommerce".into()),
                tags: vec![
                    "ecommerce".into(),
                    "store".into(),
                    "payments".into(),
                    "stripe".into(),
                    "shopping-cart".into(),
                ],
                dependencies: vec![],
                min_rustpress_version: Some("0.4.0".into()),
            },
            state: AtomicU8::new(PluginState::Inactive as u8),
        }
    }

    fn set_state(&self, state: PluginState) {
        self.state.store(state as u8, Ordering::SeqCst);
    }
}

impl Default for RustCommercePlugin {
    fn default() -> Self {
        Self::new()
    }
}
```

### 1.2 Plugin Trait Implementation

```rust
#[async_trait]
impl Plugin for RustCommercePlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn state(&self) -> PluginState {
        PluginState::from(self.state.load(Ordering::SeqCst))
    }

    fn is_compatible(&self) -> bool {
        // Check Rust edition, database compatibility, etc.
        true
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "currency": {
                    "type": "string",
                    "default": "USD",
                    "label": "Store Currency",
                    "description": "Default currency for product prices and orders"
                },
                "currency_symbol": {
                    "type": "string",
                    "default": "$",
                    "label": "Currency Symbol"
                },
                "stripe_publishable_key": {
                    "type": "string",
                    "label": "Stripe Publishable Key",
                    "description": "Your Stripe publishable API key (pk_live_...)"
                },
                "stripe_secret_key": {
                    "type": "string",
                    "format": "password",
                    "label": "Stripe Secret Key",
                    "description": "Your Stripe secret API key (sk_live_...)"
                },
                "stripe_webhook_secret": {
                    "type": "string",
                    "format": "password",
                    "label": "Stripe Webhook Secret",
                    "description": "Webhook signing secret from Stripe Dashboard (whsec_...)"
                },
                "tax_enabled": {
                    "type": "boolean",
                    "default": true,
                    "label": "Enable Tax Calculation"
                },
                "shipping_enabled": {
                    "type": "boolean",
                    "default": true,
                    "label": "Enable Shipping"
                },
                "guest_checkout": {
                    "type": "boolean",
                    "default": true,
                    "label": "Allow Guest Checkout"
                },
                "reviews_enabled": {
                    "type": "boolean",
                    "default": true,
                    "label": "Enable Product Reviews"
                },
                "hold_stock_minutes": {
                    "type": "integer",
                    "default": 10,
                    "label": "Stock Hold Duration (minutes)",
                    "description": "How long to reserve stock during checkout"
                },
                "low_stock_threshold": {
                    "type": "integer",
                    "default": 5,
                    "label": "Low Stock Threshold",
                    "description": "Trigger low stock alert when quantity falls to this level"
                }
            },
            "required": ["currency"]
        }))
    }

    async fn activate(&self, ctx: &AppContext) -> Result<()> {
        self.set_state(PluginState::Activating);
        tracing::info!("Activating RustCommerce plugin v{}", self.info.version);

        // Step 1: Run database migrations
        let pool = ctx.get::<sqlx::PgPool>()?;
        self.run_migrations(pool).await?;

        // Step 2: Register permissions with RustPress auth
        self.register_permissions(ctx).await?;

        // Step 3: Register hooks (actions and filters)
        self.register_hooks(ctx).await?;

        // Step 4: Seed default store settings
        self.seed_default_settings(pool).await?;

        // Step 5: Seed default shipping zone
        self.seed_default_shipping(pool).await?;

        // Step 6: Register scheduled jobs
        self.register_scheduled_jobs(ctx).await?;

        self.set_state(PluginState::Active);
        tracing::info!("RustCommerce plugin activated successfully");
        Ok(())
    }

    async fn deactivate(&self, ctx: &AppContext) -> Result<()> {
        self.set_state(PluginState::Deactivating);
        tracing::info!("Deactivating RustCommerce plugin");

        // Step 1: Remove hooks
        let hooks = ctx.get::<HookRegistry>()?;
        hooks.remove_plugin_hooks("rustcommerce");

        // Step 2: Cancel scheduled jobs
        let jobs = ctx.get::<JobScheduler>()?;
        jobs.cancel_plugin_jobs("rustcommerce").await?;

        // Note: Do NOT drop tables or delete data.
        // Data persists across activate/deactivate cycles.

        self.set_state(PluginState::Inactive);
        tracing::info!("RustCommerce plugin deactivated");
        Ok(())
    }

    async fn on_startup(&self, ctx: &AppContext) -> Result<()> {
        tracing::info!("RustCommerce starting up...");

        // Step 1: Initialize Stripe client
        self.initialize_stripe_client(ctx).await?;

        // Step 2: Warm product listing cache
        self.warm_product_cache(ctx).await?;

        // Step 3: Run any pending stock reservation cleanups
        let pool = ctx.get::<sqlx::PgPool>()?;
        crate::services::inventory_service::cleanup_expired_reservations(pool).await?;

        tracing::info!("RustCommerce startup complete");
        Ok(())
    }

    async fn on_shutdown(&self, ctx: &AppContext) -> Result<()> {
        tracing::info!("RustCommerce shutting down...");

        // Flush any pending cache writes
        if let Ok(cache) = ctx.get::<CacheManager>() {
            cache.flush_namespace("rustcommerce").await?;
        }

        tracing::info!("RustCommerce shutdown complete");
        Ok(())
    }
}
```

### 1.3 Helper Methods on RustCommercePlugin

```rust
impl RustCommercePlugin {
    async fn run_migrations(&self, pool: &sqlx::PgPool) -> Result<()> {
        let migrations_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("migrations");

        // Use RustPress migration runner with plugin-specific version table
        let migrator = rustpress_database::Migrator::new(
            pool,
            &migrations_dir,
            "rc_migrations",  // Plugin-specific migration tracking table
        );
        migrator.run_pending().await?;

        Ok(())
    }

    async fn register_permissions(&self, ctx: &AppContext) -> Result<()> {
        let auth_service = ctx.get::<AuthService>()?;

        let permissions = vec![
            ("manage_products", "Create, edit, and delete products, categories, and inventory"),
            ("manage_orders", "View and manage customer orders, process refunds"),
            ("manage_customers", "View and manage customer accounts"),
            ("manage_store_settings", "Configure store settings, payments, shipping, and tax"),
            ("manage_store_templates", "Upload and manage storefront templates"),
            ("view_store_reports", "View store analytics and reports"),
            ("manage_api_keys", "Create and manage store API keys"),
        ];

        for (slug, description) in permissions {
            auth_service.register_capability(slug, description).await?;
        }

        // Grant all to administrator
        auth_service.grant_capabilities_to_role("administrator", &[
            "manage_products", "manage_orders", "manage_customers",
            "manage_store_settings", "manage_store_templates",
            "view_store_reports", "manage_api_keys",
        ]).await?;

        // Grant subset to editor
        auth_service.grant_capabilities_to_role("editor", &[
            "manage_products", "manage_orders", "view_store_reports",
        ]).await?;

        Ok(())
    }

    async fn seed_default_settings(&self, pool: &sqlx::PgPool) -> Result<()> {
        let defaults = vec![
            ("store_name", json!("My Store"), "general"),
            ("currency", json!("USD"), "general"),
            ("currency_symbol", json!("$"), "general"),
            ("currency_position", json!("before"), "general"),
            ("guest_checkout_enabled", json!(true), "general"),
            ("order_number_prefix", json!("RC-"), "orders"),
            ("order_number_sequence", json!(0), "orders"),
            ("hold_stock_minutes", json!(10), "inventory"),
            ("low_stock_threshold_default", json!(5), "inventory"),
            ("manage_stock", json!(true), "inventory"),
            ("tax_enabled", json!(true), "tax"),
            ("prices_include_tax", json!(false), "tax"),
            ("shipping_enabled", json!(true), "shipping"),
            ("weight_unit", json!("kg"), "shipping"),
            ("dimension_unit", json!("cm"), "shipping"),
            ("reviews_enabled", json!(true), "reviews"),
            ("review_auto_approve", json!(false), "reviews"),
        ];

        for (key, value, group) in defaults {
            sqlx::query!(
                "INSERT INTO rc_store_settings (key, value, group_name)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (key) DO NOTHING",
                key, value, group
            ).execute(pool).await?;
        }

        Ok(())
    }

    async fn seed_default_shipping(&self, pool: &sqlx::PgPool) -> Result<()> {
        // Create default "Rest of World" shipping zone
        sqlx::query!(
            "INSERT INTO rc_shipping_zones (name, countries, is_default, position)
             SELECT 'Rest of World', '{}'::text[], true, 99
             WHERE NOT EXISTS (SELECT 1 FROM rc_shipping_zones WHERE is_default = true)"
        ).execute(pool).await?;

        Ok(())
    }

    async fn initialize_stripe_client(&self, ctx: &AppContext) -> Result<()> {
        let pool = ctx.get::<sqlx::PgPool>()?;
        let secret_key: Option<String> = sqlx::query_scalar!(
            "SELECT value::text FROM rc_store_settings WHERE key = 'stripe_secret_key'"
        ).fetch_optional(pool).await?;

        if let Some(key) = secret_key {
            let key = key.trim_matches('"'); // Remove JSON string quotes
            let client = stripe::Client::new(key);
            ctx.insert(client);
            tracing::info!("Stripe client initialized");
        } else {
            tracing::warn!("Stripe secret key not configured - payment processing unavailable");
        }

        Ok(())
    }

    async fn warm_product_cache(&self, ctx: &AppContext) -> Result<()> {
        let pool = ctx.get::<sqlx::PgPool>()?;
        let cache = ctx.get::<CacheManager>()?;

        // Pre-cache the first page of published products
        let products = sqlx::query_as!(Product,
            "SELECT * FROM rc_products WHERE status = 'published'
             ORDER BY created_at DESC LIMIT 100"
        ).fetch_all(pool).await?;

        for product in &products {
            cache.set(
                &format!("rc:product:{}", product.id),
                &serde_json::to_string(product)?,
                Duration::from_secs(300), // 5 minute TTL
            ).await?;
        }

        // Cache category tree
        let categories = sqlx::query_as!(Category,
            "SELECT * FROM rc_categories ORDER BY position ASC"
        ).fetch_all(pool).await?;

        cache.set(
            "rc:categories:tree",
            &serde_json::to_string(&categories)?,
            Duration::from_secs(600), // 10 minute TTL
        ).await?;

        tracing::info!("Product cache warmed: {} products, {} categories",
            products.len(), categories.len());

        Ok(())
    }

    async fn register_scheduled_jobs(&self, ctx: &AppContext) -> Result<()> {
        let jobs = ctx.get::<JobScheduler>()?;

        // Cart cleanup: every 5 minutes
        jobs.schedule(
            "rc_cart_cleanup",
            "*/5 * * * *",
            "rustcommerce",
            Box::new(|ctx| Box::pin(async move {
                crate::jobs::cart_cleanup::run(ctx).await
            })),
        ).await?;

        // Abandoned cart notifications: every hour
        jobs.schedule(
            "rc_abandoned_cart_notify",
            "0 * * * *",
            "rustcommerce",
            Box::new(|ctx| Box::pin(async move {
                crate::jobs::abandoned_cart_notify::run(ctx).await
            })),
        ).await?;

        // Low stock report: daily at 8am
        jobs.schedule(
            "rc_low_stock_report",
            "0 8 * * *",
            "rustcommerce",
            Box::new(|ctx| Box::pin(async move {
                crate::jobs::low_stock_report::run(ctx).await
            })),
        ).await?;

        Ok(())
    }
}
```

### 1.4 Plugin Registration in main.rs

The plugin is registered in the RustPress core `main.rs`:

```rust
// In rustpress-core-base/src/main.rs

use rustcommerce::RustCommercePlugin;

// ...

let mut plugin_loader = PluginLoader::new(&plugins_dir);

// Register RustCommerce plugin factory
plugin_loader.register_factory("rustcommerce", || {
    Arc::new(RustCommercePlugin::new())
});

// Discover, load, and activate
plugin_loader.discover().await?;
let plugin_manager = PluginManager::new();
for plugin in plugin_loader.loaded_plugins() {
    plugin_manager.register(plugin).await?;
}
plugin_manager.startup().await?;
```

---

## 2. Hook Registrations

### 2.1 Hooks Registered by RustCommerce

RustCommerce registers both **actions** (side effects) and **filters** (data transformations):

#### Actions Fired by RustCommerce

These hooks are fired by RustCommerce so other plugins can react:

| Hook Name | Fired When | Payload |
|-----------|-----------|---------|
| `rustcommerce_order_created` | New order placed and paid | `{ order_id, order_number, grand_total, customer_email }` |
| `rustcommerce_order_status_changed` | Order status updated | `{ order_id, order_number, old_status, new_status, changed_by }` |
| `rustcommerce_payment_completed` | Stripe payment confirmed | `{ order_id, payment_id, amount, currency }` |
| `rustcommerce_payment_failed` | Stripe payment failed | `{ checkout_session_id, error }` |
| `rustcommerce_refund_issued` | Refund processed | `{ order_id, refund_id, amount, reason }` |
| `rustcommerce_product_created` | New product saved | `{ product_id, name, sku, status }` |
| `rustcommerce_product_updated` | Product modified | `{ product_id, changed_fields }` |
| `rustcommerce_product_deleted` | Product archived/deleted | `{ product_id, sku }` |
| `rustcommerce_stock_low` | Stock at or below threshold | `{ product_id, variant_id, stock_quantity, threshold }` |
| `rustcommerce_stock_depleted` | Stock reached zero | `{ product_id, variant_id }` |
| `rustcommerce_customer_created` | New customer registered/created | `{ customer_id, email, source }` |
| `rustcommerce_review_submitted` | New review pending moderation | `{ review_id, product_id, rating, customer_id }` |
| `rustcommerce_cart_abandoned` | Cart inactive > 1 hour | `{ cart_id, user_id, email, item_count, subtotal }` |
| `rustcommerce_coupon_applied` | Coupon applied to cart | `{ cart_id, coupon_code, discount_amount }` |
| `rustcommerce_checkout_started` | Checkout initiated | `{ checkout_session_id, cart_id, item_count }` |
| `rustcommerce_checkout_completed` | Checkout fully complete | `{ order_id, order_number, checkout_session_id }` |

#### Actions Listened To by RustCommerce

RustCommerce listens to these hooks from RustPress core:

| Hook Name | Source | RustCommerce Action |
|-----------|--------|-------------------|
| `user_created` | RustPress Auth | Check if email matches guest customer, backfill `user_id` |
| `user_login` | RustPress Auth | Merge guest cart into user cart |
| `user_deleted` | RustPress Users | Anonymize associated customer record |
| `plugin_activated` | RustPress Core | Re-check dependencies if a new plugin affects commerce |

#### Filters Applied by RustCommerce

| Filter Name | Purpose | Input/Output Type |
|-------------|---------|-------------------|
| `rustcommerce_product_price` | Allow other plugins to modify display price | `Decimal` |
| `rustcommerce_cart_item_price` | Allow price modification at cart level | `Decimal` |
| `rustcommerce_shipping_cost` | Allow modification of calculated shipping | `Decimal` |
| `rustcommerce_tax_amount` | Allow modification of calculated tax | `Decimal` |
| `rustcommerce_order_total` | Allow final order total modification | `Decimal` |

### 2.2 Hook Registration Implementation

```rust
async fn register_hooks(&self, ctx: &AppContext) -> Result<()> {
    let hooks = ctx.get::<HookRegistry>()?;

    // ── Listen to RustPress core hooks ──

    // When a new user registers, check for existing guest customer with same email
    hooks.add_action(
        "user_created",
        Box::new(|data| Box::pin(async move {
            let user_data: UserCreatedData = serde_json::from_value(data.clone())?;
            crate::services::customer_service::on_user_created(
                &user_data.user_id,
                &user_data.email,
            ).await
        })),
        Priority::NORMAL,
        Some("rustcommerce".into()),
    );

    // When a user logs in, merge guest cart
    hooks.add_action(
        "user_login",
        Box::new(|data| Box::pin(async move {
            let login_data: UserLoginData = serde_json::from_value(data.clone())?;
            if let Some(session_id) = login_data.session_id {
                crate::services::cart_service::merge_guest_cart(
                    &login_data.user_id,
                    &session_id,
                ).await?;
            }
            Ok(())
        })),
        Priority::NORMAL,
        Some("rustcommerce".into()),
    );

    // When a user is deleted, anonymize their customer data
    hooks.add_action(
        "user_deleted",
        Box::new(|data| Box::pin(async move {
            let user_data: UserDeletedData = serde_json::from_value(data.clone())?;
            crate::services::customer_service::anonymize_by_user_id(
                &user_data.user_id,
            ).await
        })),
        Priority::NORMAL,
        Some("rustcommerce".into()),
    );

    // ── Register content filter for product shortcodes ──

    hooks.add_filter::<String>(
        "filter_the_content",
        Box::new(|content| Box::pin(async move {
            // Replace [product id="..."] shortcodes with product cards
            crate::services::shortcode_service::process_product_shortcodes(content).await
        })),
        Priority::NORMAL,
        Some("rustcommerce".into()),
    );

    tracing::info!("RustCommerce hooks registered");
    Ok(())
}
```

### 2.3 Firing Hooks from RustCommerce Services

```rust
// Example: Firing order_created hook from the order service

use rustpress_core::HookRegistry;

pub async fn create_order(
    pool: &PgPool,
    hooks: &HookRegistry,
    checkout: &CheckoutSession,
    payment: &Payment,
) -> Result<Order> {
    // ... create order in database ...

    let order = /* created order */;

    // Fire the action hook for other plugins to react
    hooks.do_action("rustcommerce_order_created", &serde_json::json!({
        "order_id": order.id,
        "order_number": order.order_number,
        "grand_total": order.grand_total.to_string(),
        "currency": order.currency,
        "customer_email": order.billing_address["email"],
        "item_count": order.items.len(),
    })).await?;

    Ok(order)
}
```

---

## 3. Route Registration

### 3.1 How RustCommerce Routes Are Added to the Core Router

In `rustpress-core-base/crates/rustpress-server/src/routes.rs`, the RustCommerce routes are nested:

```rust
// crates/rustpress-server/src/routes.rs

use rustcommerce::routes::commerce_routes;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // ... core routes ...
        .nest("/api/v1/rustcommerce", commerce_routes(state.clone()))
        // ... other plugin routes ...
        .with_state(state)
}
```

### 3.2 RustCommerce Route Tree

```rust
// rustpress-plugin-rustcommerce/src/routes.rs

use axum::{
    Router,
    routing::{get, post, put, delete},
    middleware,
};
use crate::handlers;
use crate::middleware as rc_middleware;

pub fn commerce_routes(state: AppState) -> Router<AppState> {
    Router::new()
        // ══════════════════════════════════════════════
        // PUBLIC ROUTES (no authentication required)
        // ══════════════════════════════════════════════
        .merge(public_routes())

        // ══════════════════════════════════════════════
        // SESSION ROUTES (guest or authenticated)
        // ══════════════════════════════════════════════
        .merge(session_routes())

        // ══════════════════════════════════════════════
        // CUSTOMER ROUTES (authentication required)
        // ══════════════════════════════════════════════
        .merge(customer_routes())

        // ══════════════════════════════════════════════
        // ADMIN ROUTES (authentication + permissions)
        // ══════════════════════════════════════════════
        .merge(admin_routes())

        // ══════════════════════════════════════════════
        // WEBHOOK ROUTES (custom verification)
        // ══════════════════════════════════════════════
        .merge(webhook_routes())
}

fn public_routes() -> Router<AppState> {
    Router::new()
        // Products
        .route("/products", get(handlers::product::list_products))
        .route("/products/:id", get(handlers::product::get_product))

        // Categories
        .route("/categories", get(handlers::category::list_categories))
        .route("/categories/:id", get(handlers::category::get_category))

        // Reviews (public read)
        .route("/products/:product_id/reviews",
            get(handlers::review::list_product_reviews))
        .route("/reviews/:id/helpful",
            post(handlers::review::mark_helpful))

        // Shipping (public rate check)
        .route("/shipping/methods",
            get(handlers::shipping::available_methods))

        // Tax (public estimate)
        .route("/tax/calculate",
            post(handlers::tax::calculate_tax))

        // Coupons (public validation)
        .route("/coupons/validate",
            post(handlers::coupon::validate_coupon))
}

fn session_routes() -> Router<AppState> {
    Router::new()
        // Cart
        .route("/cart", get(handlers::cart::get_cart))
        .route("/cart", delete(handlers::cart::clear_cart))
        .route("/cart/items", post(handlers::cart::add_item))
        .route("/cart/items/:item_id", put(handlers::cart::update_item))
        .route("/cart/items/:item_id", delete(handlers::cart::remove_item))
        .route("/cart/coupon", post(handlers::cart::apply_coupon))
        .route("/cart/coupon", delete(handlers::cart::remove_coupon))

        // Checkout
        .route("/checkout/init",
            post(handlers::checkout::init_checkout))
        .route("/checkout/shipping-address",
            post(handlers::checkout::set_shipping_address))
        .route("/checkout/shipping-method",
            post(handlers::checkout::set_shipping_method))
        .route("/checkout/payment-intent",
            post(handlers::checkout::create_payment_intent))
        .route("/checkout/complete",
            post(handlers::checkout::complete_checkout))

        // Apply session/auth identification middleware
        .layer(middleware::from_fn(rc_middleware::identify_session))
}

fn customer_routes() -> Router<AppState> {
    Router::new()
        // Orders (own orders)
        .route("/orders", get(handlers::order::list_my_orders))
        .route("/orders/:id", get(handlers::order::get_my_order))

        // Account
        .route("/account", get(handlers::customer::get_my_profile))
        .route("/account", put(handlers::customer::update_my_profile))

        // Addresses
        .route("/account/addresses",
            get(handlers::customer::list_addresses))
        .route("/account/addresses",
            post(handlers::customer::add_address))
        .route("/account/addresses/:id",
            put(handlers::customer::update_address))
        .route("/account/addresses/:id",
            delete(handlers::customer::delete_address))

        // Reviews (create)
        .route("/reviews", post(handlers::review::create_review))

        // Payments (own orders)
        .route("/payments/:payment_id",
            get(handlers::payment::get_payment_status))

        // Require authenticated user
        .layer(middleware::from_fn(rustpress_auth::middleware::require_auth))
}

fn admin_routes() -> Router<AppState> {
    Router::new()
        // ── Product Management ──
        .nest("/admin/products", admin_product_routes())
        .nest("/admin/categories", admin_category_routes())
        .nest("/admin/inventory", admin_inventory_routes())
        .nest("/admin/reviews", admin_review_routes())

        // ── Order Management ──
        .nest("/admin/orders", admin_order_routes())

        // ── Customer Management ──
        .nest("/admin/customers", admin_customer_routes())

        // ── Store Settings ──
        .nest("/admin/settings", admin_settings_routes())
        .nest("/admin/shipping", admin_shipping_routes())
        .nest("/admin/tax", admin_tax_routes())
        .nest("/admin/coupons", admin_coupon_routes())
        .nest("/admin/payments", admin_payment_routes())

        // ── Analytics ──
        .nest("/admin/analytics", admin_analytics_routes())
}

fn admin_product_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::admin::product::list_products))
        .route("/", post(handlers::admin::product::create_product))
        .route("/bulk", post(handlers::admin::product::bulk_operations))
        .route("/:id", get(handlers::admin::product::get_product))
        .route("/:id", put(handlers::admin::product::update_product))
        .route("/:id", delete(handlers::admin::product::delete_product))
        .layer(middleware::from_fn(
            |req, next| rustpress_auth::middleware::require_permission(req, next, "manage_products")
        ))
}

fn admin_order_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::admin::order::list_orders))
        .route("/:id", get(handlers::admin::order::get_order))
        .route("/:id/status", put(handlers::admin::order::update_status))
        .route("/:id/notes", post(handlers::admin::order::add_note))
        .route("/:id/refund", post(handlers::admin::order::process_refund))
        .layer(middleware::from_fn(
            |req, next| rustpress_auth::middleware::require_permission(req, next, "manage_orders")
        ))
}

fn admin_customer_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::admin::customer::list_customers))
        .route("/:id", get(handlers::admin::customer::get_customer))
        .route("/:id", put(handlers::admin::customer::update_customer))
        .route("/:id", delete(handlers::admin::customer::delete_customer))
        .layer(middleware::from_fn(
            |req, next| rustpress_auth::middleware::require_permission(req, next, "manage_customers")
        ))
}

fn admin_settings_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::admin::settings::get_settings))
        .route("/", put(handlers::admin::settings::update_settings))
        .layer(middleware::from_fn(
            |req, next| rustpress_auth::middleware::require_permission(req, next, "manage_store_settings")
        ))
}

fn admin_shipping_routes() -> Router<AppState> {
    Router::new()
        .route("/zones", get(handlers::admin::shipping::list_zones))
        .route("/zones", post(handlers::admin::shipping::create_zone))
        .route("/zones/:zone_id", put(handlers::admin::shipping::update_zone))
        .route("/zones/:zone_id", delete(handlers::admin::shipping::delete_zone))
        .route("/zones/:zone_id/methods",
            post(handlers::admin::shipping::create_method))
        .route("/zones/:zone_id/methods/:method_id",
            put(handlers::admin::shipping::update_method))
        .route("/zones/:zone_id/methods/:method_id",
            delete(handlers::admin::shipping::delete_method))
        .layer(middleware::from_fn(
            |req, next| rustpress_auth::middleware::require_permission(req, next, "manage_store_settings")
        ))
}

fn admin_tax_routes() -> Router<AppState> {
    Router::new()
        .route("/rates", get(handlers::admin::tax::list_rates))
        .route("/rates", post(handlers::admin::tax::create_rate))
        .route("/rates/:id", put(handlers::admin::tax::update_rate))
        .route("/rates/:id", delete(handlers::admin::tax::delete_rate))
        .layer(middleware::from_fn(
            |req, next| rustpress_auth::middleware::require_permission(req, next, "manage_store_settings")
        ))
}

fn admin_coupon_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::admin::coupon::list_coupons))
        .route("/", post(handlers::admin::coupon::create_coupon))
        .route("/:id", put(handlers::admin::coupon::update_coupon))
        .route("/:id", delete(handlers::admin::coupon::delete_coupon))
        .layer(middleware::from_fn(
            |req, next| rustpress_auth::middleware::require_permission(req, next, "manage_store_settings")
        ))
}

fn admin_category_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::admin::category::create_category))
        .route("/:id", put(handlers::admin::category::update_category))
        .route("/:id", delete(handlers::admin::category::delete_category))
        .layer(middleware::from_fn(
            |req, next| rustpress_auth::middleware::require_permission(req, next, "manage_products")
        ))
}

fn admin_inventory_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::admin::inventory::inventory_report))
        .route("/bulk", post(handlers::admin::inventory::bulk_update))
        .route("/:product_id", put(handlers::admin::inventory::update_stock))
        .layer(middleware::from_fn(
            |req, next| rustpress_auth::middleware::require_permission(req, next, "manage_products")
        ))
}

fn admin_review_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::admin::review::list_reviews))
        .route("/:id", put(handlers::admin::review::moderate_review))
        .route("/:id", delete(handlers::admin::review::delete_review))
        .layer(middleware::from_fn(
            |req, next| rustpress_auth::middleware::require_permission(req, next, "manage_products")
        ))
}

fn admin_payment_routes() -> Router<AppState> {
    Router::new()
        .route("/methods", get(handlers::admin::payment::list_methods))
        .layer(middleware::from_fn(
            |req, next| rustpress_auth::middleware::require_permission(req, next, "manage_store_settings")
        ))
}

fn admin_analytics_routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(handlers::admin::analytics::dashboard))
        .route("/revenue", get(handlers::admin::analytics::revenue))
        .route("/products", get(handlers::admin::analytics::products))
        .layer(middleware::from_fn(
            |req, next| rustpress_auth::middleware::require_permission(req, next, "view_store_reports")
        ))
}

fn webhook_routes() -> Router<AppState> {
    Router::new()
        .route("/webhooks/stripe",
            post(handlers::webhook::stripe_webhook))
        // No auth middleware - uses Stripe signature verification
}
```

### 3.3 Session Identification Middleware

Custom middleware for guest/user identification:

```rust
// src/middleware.rs

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

/// Extracts user identity from JWT or session ID.
/// Makes `SessionIdentity` available to handlers.
pub async fn identify_session(
    mut req: Request,
    next: Next,
) -> Result<Response, Error> {
    // Try JWT first
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(user) = extract_user_from_jwt(auth_header).await {
            req.extensions_mut().insert(SessionIdentity::User(user));
            return Ok(next.run(req).await);
        }
    }

    // Fall back to session ID
    if let Some(session_header) = req.headers().get("X-Session-ID") {
        let session_id = session_header.to_str()
            .map_err(|_| Error::Validation("Invalid X-Session-ID header".into()))?;

        // Validate UUID format
        let _uuid = uuid::Uuid::parse_str(session_id)
            .map_err(|_| Error::Validation("X-Session-ID must be a valid UUID".into()))?;

        req.extensions_mut().insert(SessionIdentity::Guest(session_id.to_string()));
        return Ok(next.run(req).await);
    }

    Err(Error::Authentication(
        "Either Authorization header or X-Session-ID header is required".into()
    ))
}

#[derive(Clone, Debug)]
pub enum SessionIdentity {
    User(AuthUser),
    Guest(String), // session_id
}
```

---

## 4. Migration Integration

### 4.1 Plugin Migration Discovery

RustCommerce migrations live in `rustpress-plugin-rustcommerce/migrations/`:

```
migrations/
├── 00001_ecommerce_core.sql
├── 00002_cart_and_orders.sql
├── 00003_customers.sql
├── 00004_payments.sql
├── 00005_shipping_and_tax.sql
├── 00006_coupons.sql
└── 00007_reviews.sql
```

### 4.2 plugin.toml Migration Configuration

```toml
[migrations]
directory = "migrations"
version_table = "rc_migrations"
```

### 4.3 Migration Tracking

RustCommerce uses its own migration tracking table (`rc_migrations`) separate from the core `_sqlx_migrations` table:

```sql
CREATE TABLE IF NOT EXISTS rc_migrations (
    id          SERIAL PRIMARY KEY,
    filename    VARCHAR(255) NOT NULL UNIQUE,
    applied_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    checksum    VARCHAR(64) NOT NULL
);
```

### 4.4 Migration Execution During Activation

```rust
async fn run_migrations(&self, pool: &sqlx::PgPool) -> Result<()> {
    // Ensure migration tracking table exists
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS rc_migrations (
            id SERIAL PRIMARY KEY,
            filename VARCHAR(255) NOT NULL UNIQUE,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            checksum VARCHAR(64) NOT NULL
        )"
    ).execute(pool).await?;

    // Read migration files from embedded directory
    let migrations_dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

    let mut migration_files: Vec<_> = migrations_dir.files()
        .filter(|f| f.path().extension() == Some("sql".as_ref()))
        .collect();
    migration_files.sort_by_key(|f| f.path().to_string_lossy().to_string());

    for file in migration_files {
        let filename = file.path().file_name().unwrap().to_string_lossy().to_string();
        let sql = file.contents_utf8().unwrap();
        let checksum = sha256_hex(sql);

        // Check if already applied
        let applied = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM rc_migrations WHERE filename = $1)",
            &filename
        ).fetch_one(pool).await?.unwrap_or(false);

        if !applied {
            tracing::info!("Running migration: {}", filename);

            // Execute in a transaction
            let mut tx = pool.begin().await?;
            sqlx::query(sql).execute(&mut *tx).await
                .map_err(|e| Error::Plugin(format!("Migration {} failed: {}", filename, e)))?;

            sqlx::query!(
                "INSERT INTO rc_migrations (filename, checksum) VALUES ($1, $2)",
                &filename, &checksum
            ).execute(&mut *tx).await?;

            tx.commit().await?;
            tracing::info!("Migration {} applied successfully", filename);
        }
    }

    Ok(())
}
```

### 4.5 Safe Uninstall (Data Preservation)

When the plugin is deactivated, tables are NOT dropped. Only an explicit uninstall command removes data:

```rust
// CLI command: rustpress-cli plugin:uninstall rustcommerce --confirm-data-deletion
pub async fn uninstall(pool: &PgPool, confirm: bool) -> Result<()> {
    if !confirm {
        return Err(Error::Validation(
            "Uninstalling will permanently delete all RustCommerce data. Pass --confirm-data-deletion to proceed.".into()
        ));
    }

    // Drop all rc_ tables in reverse dependency order
    let tables = vec![
        "rc_review_votes", "rc_reviews",
        "rc_coupon_usage", "rc_coupons",
        "rc_tax_rates", "rc_shipping_methods", "rc_shipping_zones",
        "rc_refunds", "rc_payments",
        "rc_customer_addresses", "rc_customers",
        "rc_stock_reservations", "rc_order_status_history", "rc_order_items", "rc_orders",
        "rc_cart_items", "rc_carts",
        "rc_product_categories", "rc_product_images", "rc_product_variants",
        "rc_categories", "rc_products",
        "rc_store_settings", "rc_migrations",
    ];

    for table in tables {
        sqlx::query(&format!("DROP TABLE IF EXISTS {} CASCADE", table))
            .execute(pool).await?;
    }

    Ok(())
}
```

---

## 5. Cache Strategy

### 5.1 Cache Layers

RustCommerce uses the RustPress cache system (`rustpress-cache` crate) which supports:

1. **In-Memory (moka)**: Fast L1 cache, per-instance, limited size
2. **Redis**: Shared L2 cache, cross-instance, persistent
3. **Hybrid**: L1 + L2 with write-through

### 5.2 Cache Key Namespace

All RustCommerce cache keys use the `rc:` prefix:

| Key Pattern | Data | TTL | Invalidation |
|-------------|------|-----|-------------|
| `rc:product:{id}` | Full product JSON | 5 min | On product update/delete |
| `rc:product:slug:{slug}` | Product ID (lookup) | 5 min | On product update/delete |
| `rc:products:list:{hash}` | Paginated product list | 2 min | On any product change |
| `rc:categories:tree` | Full category tree | 10 min | On category change |
| `rc:category:{id}` | Single category | 10 min | On category update/delete |
| `rc:cart:{user_id}` | Cart with items | No TTL | On cart modification |
| `rc:cart:session:{session_id}` | Guest cart | 1 hour | On cart modification |
| `rc:settings:{group}` | Settings by group | 30 min | On settings update |
| `rc:tax:rates:{country}:{state}` | Tax rates for location | 1 hour | On tax rate change |
| `rc:shipping:zones` | All shipping zones | 1 hour | On zone change |
| `rc:stripe:event:{event_id}` | Processed event marker | 24 hours | Never (TTL expiry) |

### 5.3 Cache Invalidation Patterns

#### Pattern 1: Direct Invalidation

```rust
// When a product is updated, invalidate its cache entries
pub async fn invalidate_product_cache(
    cache: &CacheManager,
    product_id: Uuid,
    product_slug: &str,
) {
    cache.delete(&format!("rc:product:{}", product_id)).await;
    cache.delete(&format!("rc:product:slug:{}", product_slug)).await;
    // Invalidate all product list caches (they may contain this product)
    cache.delete_pattern("rc:products:list:*").await;
}
```

#### Pattern 2: Tag-Based Invalidation

```rust
// Invalidate all caches tagged with "products"
cache.invalidate_tag("rc:products").await;
```

#### Pattern 3: TTL-Only (No Active Invalidation)

Tax rates and shipping zones change infrequently. Cache entries expire naturally via TTL.

### 5.4 Cache-Aside Pattern Implementation

```rust
pub async fn get_product(
    pool: &PgPool,
    cache: &CacheManager,
    product_id: Uuid,
) -> Result<Product> {
    let cache_key = format!("rc:product:{}", product_id);

    // Try cache first
    if let Some(cached) = cache.get(&cache_key).await {
        let product: Product = serde_json::from_str(&cached)?;
        return Ok(product);
    }

    // Cache miss - query database
    let product = sqlx::query_as!(Product,
        "SELECT * FROM rc_products WHERE id = $1",
        product_id
    ).fetch_optional(pool).await?
        .ok_or(Error::NotFound("Product not found".into()))?;

    // Store in cache
    cache.set(
        &cache_key,
        &serde_json::to_string(&product)?,
        Duration::from_secs(300), // 5 minutes
    ).await?;

    Ok(product)
}
```

### 5.5 Performance Targets

| Operation | Without Cache | With Cache | Target |
|-----------|--------------|------------|--------|
| Product listing (20 items) | ~15ms | ~2ms | < 100ms |
| Product detail | ~8ms | ~1ms | < 50ms |
| Category tree | ~10ms | ~1ms | < 50ms |
| Cart totals | ~20ms (always fresh) | N/A | < 50ms |

---

## 6. Event Bus Integration

### 6.1 Overview

RustPress provides an event bus (`rustpress-events` crate) for asynchronous, decoupled communication between plugins. RustCommerce publishes events for significant business occurrences.

### 6.2 Events Published

```rust
use rustpress_events::{Event, EventBus};

// Event definitions
pub mod events {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OrderCreatedEvent {
        pub order_id: Uuid,
        pub order_number: String,
        pub customer_id: Option<Uuid>,
        pub customer_email: String,
        pub grand_total: String,
        pub currency: String,
        pub item_count: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OrderStatusChangedEvent {
        pub order_id: Uuid,
        pub order_number: String,
        pub old_status: String,
        pub new_status: String,
        pub changed_by: Option<Uuid>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PaymentCompletedEvent {
        pub order_id: Uuid,
        pub payment_id: Uuid,
        pub amount: String,
        pub currency: String,
        pub payment_method: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LowStockEvent {
        pub product_id: Uuid,
        pub variant_id: Option<Uuid>,
        pub product_name: String,
        pub sku: Option<String>,
        pub current_stock: i32,
        pub threshold: i32,
    }
}
```

### 6.3 Publishing Events

```rust
pub async fn publish_order_created(
    event_bus: &EventBus,
    order: &Order,
) -> Result<()> {
    event_bus.publish(Event {
        event_type: "rustcommerce.order.created".into(),
        source: "rustcommerce".into(),
        data: serde_json::to_value(OrderCreatedEvent {
            order_id: order.id,
            order_number: order.order_number.clone(),
            customer_id: order.customer_id,
            customer_email: order.billing_address["email"].as_str().unwrap_or("").to_string(),
            grand_total: order.grand_total.to_string(),
            currency: order.currency.clone(),
            item_count: order.items.len(),
        })?,
        timestamp: Utc::now(),
    }).await?;

    Ok(())
}
```

### 6.4 Subscribing to Events from Other Plugins

```rust
// Other plugins can subscribe to RustCommerce events:

event_bus.subscribe("rustcommerce.order.created", |event| {
    Box::pin(async move {
        let order_data: OrderCreatedEvent = serde_json::from_value(event.data)?;
        // Send email notification, update analytics, trigger fulfillment, etc.
        Ok(())
    })
}).await;
```

---

## 7. Background Jobs

### 7.1 Registered Jobs

| Job Name | Schedule | Description |
|----------|----------|-------------|
| `rc_cart_cleanup` | Every 5 minutes | Mark abandoned carts, delete old carts, release expired stock reservations |
| `rc_abandoned_cart_notify` | Every hour | Send abandoned cart reminder emails to customers |
| `rc_low_stock_report` | Daily at 8:00 AM | Generate and email daily low stock report to admin |
| `rc_order_number_integrity` | Daily at 2:00 AM | Verify order number sequence integrity |
| `rc_customer_aggregate_sync` | Daily at 3:00 AM | Recalculate all customer aggregate fields for consistency |
| `rc_product_count_sync` | Daily at 4:00 AM | Recalculate category product counts |

### 7.2 Job Implementation

```rust
// src/jobs/cart_cleanup.rs

pub async fn run(ctx: &AppContext) -> Result<()> {
    let pool = ctx.get::<sqlx::PgPool>()?;
    let hooks = ctx.get::<HookRegistry>()?;

    // 1. Find carts abandoned > 1 hour (for notification)
    let newly_abandoned = sqlx::query_as!(Cart,
        "SELECT * FROM rc_carts
         WHERE status = 'active'
           AND updated_at < now() - interval '1 hour'"
    ).fetch_all(pool).await?;

    for cart in &newly_abandoned {
        hooks.do_action("rustcommerce_cart_abandoned", &json!({
            "cart_id": cart.id,
            "user_id": cart.user_id,
            "item_count": /* count items */,
        })).await?;
    }

    // 2. Mark abandoned
    let abandoned_count = sqlx::query!(
        "UPDATE rc_carts SET status = 'abandoned', updated_at = now()
         WHERE status = 'active' AND updated_at < now() - interval '1 hour'"
    ).execute(pool).await?.rows_affected();

    // 3. Delete old abandoned carts (> 30 days)
    let deleted_count = sqlx::query!(
        "DELETE FROM rc_carts
         WHERE status = 'abandoned' AND updated_at < now() - interval '30 days'"
    ).execute(pool).await?.rows_affected();

    // 4. Release expired stock reservations
    let expired = sqlx::query!(
        "UPDATE rc_stock_reservations SET status = 'expired'
         WHERE status = 'active' AND expires_at <= now()
         RETURNING product_id, variant_id, quantity"
    ).fetch_all(pool).await?;

    for res in &expired {
        // Restore stock
        if let Some(vid) = res.variant_id {
            sqlx::query!(
                "UPDATE rc_product_variants SET stock_quantity = stock_quantity + $1 WHERE id = $2",
                res.quantity, vid
            ).execute(pool).await?;
        } else {
            sqlx::query!(
                "UPDATE rc_products SET stock_quantity = stock_quantity + $1 WHERE id = $2",
                res.quantity, res.product_id
            ).execute(pool).await?;
        }
    }

    tracing::info!(
        "Cart cleanup: {} abandoned, {} deleted, {} reservations released",
        abandoned_count, deleted_count, expired.len()
    );

    Ok(())
}
```

---

## 8. Plugin Configuration

### 8.1 plugin.toml

The complete plugin manifest:

```toml
# ═══════════════════════════════════════
# RustCommerce Plugin Manifest
# ═══════════════════════════════════════

id = "rustcommerce"
name = "RustCommerce"
version = "1.0.0"
description = "Full-featured e-commerce plugin for RustPress CMS"
author = "RustPress"
license = "MIT"
tags = ["ecommerce", "store", "payments", "stripe", "shopping-cart"]
category = "ecommerce"
icon = "shopping-cart"

[requirements]
rustpress_version = ">=0.4.0"

# ── Dependencies ──
[dependencies]
required = ["rustpress-core >= 0.4.0", "rustpress-auth >= 0.4.0", "rustpress-database >= 0.4.0"]
optional = ["rustpress-cache >= 0.4.0", "rustpress-events >= 0.4.0"]
conflicts = []

# ── Database Migrations ──
[migrations]
directory = "migrations"
version_table = "rc_migrations"

# ── Permissions ──
[permissions]
manage_products = "Create, edit, and delete products, categories, and inventory"
manage_orders = "View and manage customer orders, process refunds"
manage_customers = "View and manage customer accounts"
manage_store_settings = "Configure store settings, payments, shipping, and tax"
manage_store_templates = "Upload and manage storefront templates"
view_store_reports = "View store analytics and reports"
manage_api_keys = "Create and manage store API keys"

# ── API Endpoints ──
[api]
namespace = "rustcommerce"

[[api.endpoints]]
method = "GET"
path = "/products"
handler = "list_products"
permission = "read"
rate_limit = 60

[[api.endpoints]]
method = "GET"
path = "/products/:id"
handler = "get_product"
permission = "read"
rate_limit = 60

[[api.endpoints]]
method = "POST"
path = "/admin/products"
handler = "create_product"
permission = "manage_products"
rate_limit = 120

# ... (all 75 endpoints declared similarly)

# ── Admin Menu ──
[[admin.menu]]
label = "Store"
icon = "shopping-cart"
position = 30

[[admin.menu.items]]
label = "Dashboard"
path = "/admin/store"
icon = "layout-dashboard"

[[admin.menu.items]]
label = "Products"
path = "/admin/store/products"
icon = "package"

[[admin.menu.items]]
label = "Orders"
path = "/admin/store/orders"
icon = "shopping-bag"

[[admin.menu.items]]
label = "Customers"
path = "/admin/store/customers"
icon = "users"

[[admin.menu.items]]
label = "Templates"
path = "/admin/store/templates"
icon = "palette"

[[admin.menu.items]]
label = "Settings"
path = "/admin/store/settings"
icon = "settings"

# ── Admin Pages ──
[[admin.pages]]
path = "/admin/store"
component = "AdminDashboard"
title = "Store Dashboard"

[[admin.pages]]
path = "/admin/store/products"
component = "AdminProducts"
title = "Products"

[[admin.pages]]
path = "/admin/store/orders"
component = "AdminOrders"
title = "Orders"

[[admin.pages]]
path = "/admin/store/customers"
component = "AdminCustomers"
title = "Customers"

[[admin.pages]]
path = "/admin/store/templates"
component = "AdminTemplates"
title = "Store Templates"

[[admin.pages]]
path = "/admin/store/settings"
component = "AdminSettings"
title = "Store Settings"

# ── Dashboard Widgets ──
[[admin.widgets]]
id = "rc-revenue-chart"
title = "Revenue"
component = "RevenueChart"
position = "main"
size = "large"

[[admin.widgets]]
id = "rc-recent-orders"
title = "Recent Orders"
component = "RecentOrders"
position = "main"
size = "medium"

[[admin.widgets]]
id = "rc-order-status"
title = "Orders by Status"
component = "OrderStatusPie"
position = "sidebar"
size = "small"

[[admin.widgets]]
id = "rc-top-products"
title = "Top Products"
component = "TopProducts"
position = "sidebar"
size = "small"

# ── Scheduled Jobs ──
[[cron]]
name = "rc_cart_cleanup"
schedule = "*/5 * * * *"
handler = "cleanup_expired_carts"
description = "Clean up abandoned carts and expired stock reservations"

[[cron]]
name = "rc_abandoned_cart_notify"
schedule = "0 * * * *"
handler = "notify_abandoned_carts"
description = "Send abandoned cart reminder emails"

[[cron]]
name = "rc_low_stock_report"
schedule = "0 8 * * *"
handler = "low_stock_daily_report"
description = "Daily low stock report"

[[cron]]
name = "rc_customer_aggregate_sync"
schedule = "0 3 * * *"
handler = "sync_customer_aggregates"
description = "Recalculate customer aggregate statistics"

[[cron]]
name = "rc_product_count_sync"
schedule = "0 4 * * *"
handler = "sync_category_product_counts"
description = "Recalculate category product counts"

# ── Settings Schema ──
[settings.schema.currency]
type = "select"
label = "Store Currency"
description = "Default currency for prices and orders"
default = "USD"
options = ["USD", "EUR", "GBP", "CAD", "AUD", "JPY"]
group = "general"
required = true

[settings.schema.stripe_publishable_key]
type = "string"
label = "Stripe Publishable Key"
description = "Your Stripe publishable API key"
group = "payments"

[settings.schema.stripe_secret_key]
type = "password"
label = "Stripe Secret Key"
description = "Your Stripe secret API key"
group = "payments"

[settings.schema.stripe_webhook_secret]
type = "password"
label = "Stripe Webhook Secret"
description = "Webhook signing secret from Stripe Dashboard"
group = "payments"

[settings.schema.guest_checkout]
type = "boolean"
label = "Guest Checkout"
description = "Allow customers to checkout without creating an account"
default = true
group = "general"

[settings.schema.tax_enabled]
type = "boolean"
label = "Enable Taxes"
description = "Enable tax calculation on orders"
default = true
group = "tax"

[settings.schema.shipping_enabled]
type = "boolean"
label = "Enable Shipping"
description = "Enable shipping calculation"
default = true
group = "shipping"

# ── Hooks ──
[hooks]
activate = "onActivate"
deactivate = "onDeactivate"
uninstall = "onUninstall"

[[hooks.actions]]
hook = "user_created"
callback = "onUserCreated"
priority = 10

[[hooks.actions]]
hook = "user_login"
callback = "onUserLogin"
priority = 10

[[hooks.actions]]
hook = "user_deleted"
callback = "onUserDeleted"
priority = 10

[[hooks.filters]]
hook = "filter_the_content"
callback = "processProductShortcodes"
priority = 10

# ── Feature Flags ──
[features]
wishlist = { enabled = true, rollout = 100 }
compare = { enabled = true, rollout = 100 }
reviews = { enabled = true, rollout = 100 }
coupons = { enabled = true, rollout = 100 }
digital_products = { enabled = false, rollout = 0 }
subscriptions = { enabled = false, rollout = 0 }
multi_currency = { enabled = false, rollout = 0 }
```

---

## 9. Dependency Management

### 9.1 Cargo.toml

```toml
[package]
name = "rustcommerce"
version = "1.0.0"
edition = "2021"
description = "Full-featured e-commerce plugin for RustPress CMS"
license = "MIT"
authors = ["RustPress <team@rustpress.io>"]
repository = "https://github.com/rustpress-net/rustpress-plugin-rustcommerce"

[lib]
name = "rustcommerce"
path = "src/lib.rs"

[dependencies]
# RustPress core dependencies
rustpress-core = { path = "../rustpress-core-base/crates/rustpress-core" }
rustpress-database = { path = "../rustpress-core-base/crates/rustpress-database" }
rustpress-auth = { path = "../rustpress-core-base/crates/rustpress-auth" }
rustpress-cache = { path = "../rustpress-core-base/crates/rustpress-cache" }
rustpress-events = { path = "../rustpress-core-base/crates/rustpress-events" }
rustpress-jobs = { path = "../rustpress-core-base/crates/rustpress-jobs" }

# Web framework
axum = { version = "0.7", features = ["json", "query", "multipart"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "compression-gzip"] }

# Async runtime
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "decimal"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Types
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
rust_decimal = { version = "1.34", features = ["serde-with-str", "db-postgres"] }

# Stripe payment processing
stripe-rust = { version = "0.28", features = ["runtime-tokio-hyper"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"

# Utilities
include_dir = "0.7"
sha2 = "0.10"
hex = "0.4"

[dev-dependencies]
# Testing
tokio-test = "0.4"
mockall = "0.12"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "testing"] }
wiremock = "0.6"           # For mocking Stripe API
tower = { version = "0.4", features = ["util"] }
axum-test = "0.14"         # For handler testing
```

---

## 10. Admin UI Integration

### 10.1 Plugin Registration in Admin UI

The admin UI discovers the plugin's pages and menu items from the `plugin.toml` manifest. The React components are registered in the admin-ui:

```typescript
// rustpress-core-admin-ui/src/pages/plugins/rustcommerce/index.tsx

import { lazy } from 'react';
import { RouteObject } from 'react-router-dom';

const Dashboard = lazy(() => import('./components/Dashboard'));
const ProductList = lazy(() => import('./components/ProductList'));
const ProductEditor = lazy(() => import('./components/ProductEditor'));
const OrderList = lazy(() => import('./components/OrderList'));
const OrderDetail = lazy(() => import('./components/OrderDetail'));
const CustomerList = lazy(() => import('./components/CustomerList'));
const CustomerDetail = lazy(() => import('./components/CustomerDetail'));
const Settings = lazy(() => import('./components/settings/index'));

export const rustcommerceRoutes: RouteObject[] = [
  { path: '/admin/store', element: <Dashboard /> },
  { path: '/admin/store/products', element: <ProductList /> },
  { path: '/admin/store/products/new', element: <ProductEditor /> },
  { path: '/admin/store/products/:id', element: <ProductEditor /> },
  { path: '/admin/store/orders', element: <OrderList /> },
  { path: '/admin/store/orders/:id', element: <OrderDetail /> },
  { path: '/admin/store/customers', element: <CustomerList /> },
  { path: '/admin/store/customers/:id', element: <CustomerDetail /> },
  { path: '/admin/store/settings', element: <Settings /> },
  { path: '/admin/store/settings/:tab', element: <Settings /> },
];
```

### 10.2 Admin API Client

```typescript
// rustpress-core-admin-ui/src/pages/plugins/rustcommerce/api/commerceApi.ts

import api from '@/api/client';

const BASE = '/v1/rustcommerce';

export const commerceApi = {
  // Products
  listProducts: (params?: Record<string, any>) =>
    api.get(`${BASE}/admin/products`, { params }),
  getProduct: (id: string) =>
    api.get(`${BASE}/admin/products/${id}`),
  createProduct: (data: any) =>
    api.post(`${BASE}/admin/products`, data),
  updateProduct: (id: string, data: any) =>
    api.put(`${BASE}/admin/products/${id}`, data),
  deleteProduct: (id: string) =>
    api.delete(`${BASE}/admin/products/${id}`),
  bulkProducts: (data: any) =>
    api.post(`${BASE}/admin/products/bulk`, data),

  // Categories
  listCategories: (params?: Record<string, any>) =>
    api.get(`${BASE}/categories`, { params }),
  createCategory: (data: any) =>
    api.post(`${BASE}/admin/categories`, data),
  updateCategory: (id: string, data: any) =>
    api.put(`${BASE}/admin/categories/${id}`, data),
  deleteCategory: (id: string) =>
    api.delete(`${BASE}/admin/categories/${id}`),

  // Orders
  listOrders: (params?: Record<string, any>) =>
    api.get(`${BASE}/admin/orders`, { params }),
  getOrder: (id: string) =>
    api.get(`${BASE}/admin/orders/${id}`),
  updateOrderStatus: (id: string, data: any) =>
    api.put(`${BASE}/admin/orders/${id}/status`, data),
  addOrderNote: (id: string, data: any) =>
    api.post(`${BASE}/admin/orders/${id}/notes`, data),
  refundOrder: (id: string, data: any) =>
    api.post(`${BASE}/admin/orders/${id}/refund`, data),

  // Customers
  listCustomers: (params?: Record<string, any>) =>
    api.get(`${BASE}/admin/customers`, { params }),
  getCustomer: (id: string) =>
    api.get(`${BASE}/admin/customers/${id}`),
  updateCustomer: (id: string, data: any) =>
    api.put(`${BASE}/admin/customers/${id}`, data),
  deleteCustomer: (id: string) =>
    api.delete(`${BASE}/admin/customers/${id}`),

  // Inventory
  getInventory: (params?: Record<string, any>) =>
    api.get(`${BASE}/admin/inventory`, { params }),
  updateStock: (productId: string, data: any) =>
    api.put(`${BASE}/admin/inventory/${productId}`, data),
  bulkStockUpdate: (data: any) =>
    api.post(`${BASE}/admin/inventory/bulk`, data),

  // Settings
  getSettings: () =>
    api.get(`${BASE}/admin/settings`),
  updateSettings: (data: any) =>
    api.put(`${BASE}/admin/settings`, data),

  // Shipping
  listShippingZones: () =>
    api.get(`${BASE}/admin/shipping/zones`),
  createShippingZone: (data: any) =>
    api.post(`${BASE}/admin/shipping/zones`, data),
  updateShippingZone: (id: string, data: any) =>
    api.put(`${BASE}/admin/shipping/zones/${id}`, data),
  deleteShippingZone: (id: string) =>
    api.delete(`${BASE}/admin/shipping/zones/${id}`),
  createShippingMethod: (zoneId: string, data: any) =>
    api.post(`${BASE}/admin/shipping/zones/${zoneId}/methods`, data),
  updateShippingMethod: (zoneId: string, methodId: string, data: any) =>
    api.put(`${BASE}/admin/shipping/zones/${zoneId}/methods/${methodId}`, data),
  deleteShippingMethod: (zoneId: string, methodId: string) =>
    api.delete(`${BASE}/admin/shipping/zones/${zoneId}/methods/${methodId}`),

  // Tax
  listTaxRates: (params?: Record<string, any>) =>
    api.get(`${BASE}/admin/tax/rates`, { params }),
  createTaxRate: (data: any) =>
    api.post(`${BASE}/admin/tax/rates`, data),
  updateTaxRate: (id: string, data: any) =>
    api.put(`${BASE}/admin/tax/rates/${id}`, data),
  deleteTaxRate: (id: string) =>
    api.delete(`${BASE}/admin/tax/rates/${id}`),

  // Coupons
  listCoupons: (params?: Record<string, any>) =>
    api.get(`${BASE}/admin/coupons`, { params }),
  createCoupon: (data: any) =>
    api.post(`${BASE}/admin/coupons`, data),
  updateCoupon: (id: string, data: any) =>
    api.put(`${BASE}/admin/coupons/${id}`, data),
  deleteCoupon: (id: string) =>
    api.delete(`${BASE}/admin/coupons/${id}`),

  // Reviews
  listReviews: (params?: Record<string, any>) =>
    api.get(`${BASE}/admin/reviews`, { params }),
  moderateReview: (id: string, data: any) =>
    api.put(`${BASE}/admin/reviews/${id}`, data),
  deleteReview: (id: string) =>
    api.delete(`${BASE}/admin/reviews/${id}`),

  // Analytics
  getDashboard: (params?: Record<string, any>) =>
    api.get(`${BASE}/admin/analytics/dashboard`, { params }),
  getRevenue: (params?: Record<string, any>) =>
    api.get(`${BASE}/admin/analytics/revenue`, { params }),
  getProductPerformance: (params?: Record<string, any>) =>
    api.get(`${BASE}/admin/analytics/products`, { params }),

  // Payment Methods
  listPaymentMethods: () =>
    api.get(`${BASE}/admin/payments/methods`),
};
```

### 10.3 Zustand Store

```typescript
// rustpress-core-admin-ui/src/pages/plugins/rustcommerce/stores/commerceStore.ts

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { commerceApi } from '../api/commerceApi';

interface CommerceState {
  // Products
  products: Product[];
  productsLoading: boolean;
  productsPagination: Pagination | null;

  // Orders
  orders: Order[];
  ordersLoading: boolean;

  // Dashboard
  dashboardData: DashboardData | null;
  dashboardLoading: boolean;

  // Actions
  fetchProducts: (params?: Record<string, any>) => Promise<void>;
  fetchOrders: (params?: Record<string, any>) => Promise<void>;
  fetchDashboard: (params?: Record<string, any>) => Promise<void>;
}

export const useCommerceStore = create<CommerceState>()(
  persist(
    (set, get) => ({
      products: [],
      productsLoading: false,
      productsPagination: null,
      orders: [],
      ordersLoading: false,
      dashboardData: null,
      dashboardLoading: false,

      fetchProducts: async (params) => {
        set({ productsLoading: true });
        try {
          const response = await commerceApi.listProducts(params);
          set({
            products: response.data.data,
            productsPagination: response.data.pagination,
          });
        } finally {
          set({ productsLoading: false });
        }
      },

      fetchOrders: async (params) => {
        set({ ordersLoading: true });
        try {
          const response = await commerceApi.listOrders(params);
          set({ orders: response.data.data });
        } finally {
          set({ ordersLoading: false });
        }
      },

      fetchDashboard: async (params) => {
        set({ dashboardLoading: true });
        try {
          const response = await commerceApi.getDashboard(params);
          set({ dashboardData: response.data.data });
        } finally {
          set({ dashboardLoading: false });
        }
      },
    }),
    {
      name: 'rustcommerce-store',
      partialize: (state) => ({
        // Only persist non-sensitive, lightweight data
      }),
    }
  )
);
```
