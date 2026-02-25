//! Route definitions for RustCommerce
//!
//! Builds the complete Axum router with all route groups:
//! public storefront, authenticated customer, admin, and webhooks.

use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::handlers;
use crate::plugin::AppState;

/// Build the complete RustCommerce router.
///
/// Route groups:
/// - `/api/v1/rustcommerce/` — Public storefront endpoints
/// - `/api/v1/admin/rustcommerce/` — Admin endpoints (require auth)
/// - `/api/v1/webhooks/rustcommerce/` — Webhook endpoints
pub fn build_router(state: AppState) -> Router {
    let public_routes = public_router();
    let admin_routes = admin_router();
    let webhook_routes = webhook_router();

    Router::new()
        .nest("/api/v1/rustcommerce", public_routes)
        .nest("/api/v1/admin/rustcommerce", admin_routes)
        .nest("/api/v1/webhooks/rustcommerce", webhook_routes)
        .with_state(state)
}

/// Public storefront routes (no auth required for browsing).
fn public_router() -> Router<AppState> {
    Router::new()
        // Products
        .route("/products", get(handlers::product_handler::list_products))
        .route("/products/{id}", get(handlers::product_handler::get_product))
        .route(
            "/products/slug/{slug}",
            get(handlers::product_handler::get_product_by_slug),
        )
        .route(
            "/products/{id}/variants",
            get(handlers::product_handler::list_variants),
        )
        .route(
            "/products/{id}/images",
            get(handlers::product_handler::list_product_images),
        )
        // Product reviews (public read)
        .route(
            "/products/{id}/reviews",
            get(handlers::admin_handler::list_reviews_admin), // reuse with product_id filter
        )
        .route(
            "/products/{id}/reviews/stats",
            get(get_product_review_stats),
        )
        .route("/reviews", post(submit_review))
        .route("/reviews/{id}/helpful", post(mark_review_helpful))
        // Cart
        .route("/cart", post(handlers::cart_handler::create_cart))
        .route("/cart/{id}", get(handlers::cart_handler::get_cart))
        .route(
            "/cart/{id}/items",
            post(handlers::cart_handler::add_cart_item)
                .delete(handlers::cart_handler::clear_cart),
        )
        .route(
            "/cart/{cart_id}/items/{item_id}",
            put(handlers::cart_handler::update_cart_item)
                .delete(handlers::cart_handler::remove_cart_item),
        )
        .route(
            "/cart/{id}/coupon",
            post(handlers::cart_handler::apply_coupon)
                .delete(handlers::cart_handler::remove_coupon),
        )
        // Checkout
        .route(
            "/checkout",
            post(handlers::checkout_handler::process_checkout),
        )
        // Shipping rates
        .route(
            "/shipping/rates",
            post(handlers::settings_handler::calculate_shipping_rates),
        )
        // Orders (customer-facing, would normally require auth)
        .route("/orders/{id}", get(handlers::order_handler::get_order))
        .route(
            "/orders/number/{number}",
            get(handlers::order_handler::get_order_by_number),
        )
        // Customers
        .route(
            "/customers",
            post(handlers::customer_handler::create_customer),
        )
        .route(
            "/customers/{id}",
            get(handlers::customer_handler::get_customer)
                .put(handlers::customer_handler::update_customer),
        )
        .route(
            "/customers/{id}/addresses",
            get(handlers::customer_handler::list_addresses)
                .post(handlers::customer_handler::create_address),
        )
        .route(
            "/customers/{customer_id}/addresses/{address_id}",
            delete(handlers::customer_handler::delete_address),
        )
}

/// Admin routes (require admin authentication).
fn admin_router() -> Router<AppState> {
    Router::new()
        // Dashboard
        .route("/dashboard", get(handlers::admin_handler::get_dashboard))
        // Product management
        .route(
            "/products",
            post(handlers::product_handler::create_product),
        )
        .route(
            "/products/{id}",
            put(handlers::product_handler::update_product)
                .delete(handlers::product_handler::delete_product),
        )
        .route(
            "/products/{id}/variants",
            post(handlers::product_handler::create_variant),
        )
        // Order management
        .route("/orders", get(handlers::order_handler::list_orders))
        .route(
            "/orders/{id}/status",
            put(handlers::order_handler::update_order_status),
        )
        // Customer management
        .route(
            "/customers",
            get(handlers::customer_handler::list_customers),
        )
        .route(
            "/customers/{id}",
            delete(handlers::customer_handler::delete_customer),
        )
        // Reviews
        .route(
            "/reviews",
            get(handlers::admin_handler::list_reviews_admin),
        )
        .route(
            "/reviews/{id}/moderate",
            put(handlers::admin_handler::moderate_review),
        )
        // Coupons
        .route(
            "/coupons",
            get(handlers::settings_handler::list_coupons)
                .post(handlers::settings_handler::create_coupon),
        )
        .route(
            "/coupons/{id}",
            get(handlers::settings_handler::get_coupon)
                .put(handlers::settings_handler::update_coupon)
                .delete(handlers::settings_handler::delete_coupon),
        )
        // Shipping settings
        .route(
            "/shipping/zones",
            get(handlers::settings_handler::list_shipping_zones)
                .post(handlers::settings_handler::create_shipping_zone),
        )
        .route(
            "/shipping/zones/{id}/methods",
            get(handlers::settings_handler::list_shipping_methods),
        )
        .route(
            "/shipping/methods",
            post(handlers::settings_handler::create_shipping_method),
        )
        // Tax settings
        .route(
            "/tax/rates",
            get(handlers::settings_handler::list_tax_rates)
                .post(handlers::settings_handler::create_tax_rate),
        )
        .route(
            "/tax/rates/{id}",
            put(handlers::settings_handler::update_tax_rate)
                .delete(handlers::settings_handler::delete_tax_rate),
        )
}

/// Webhook routes (no auth, signature-verified).
fn webhook_router() -> Router<AppState> {
    Router::new().route(
        "/stripe",
        post(handlers::webhook_handler::handle_stripe_webhook),
    )
}

// ── Inline helper handlers ─────────────────────────────────────────────

/// GET /products/:id/reviews/stats
async fn get_product_review_stats(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(product_id): axum::extract::Path<uuid::Uuid>,
) -> crate::error::Result<axum::Json<crate::models::review::ProductRatingStats>> {
    let stats = crate::services::ReviewService::get_product_stats(&state.pool, product_id).await?;
    Ok(axum::Json(stats))
}

/// POST /reviews
async fn submit_review(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::Json(req): axum::Json<crate::models::review::CreateReviewRequest>,
) -> crate::error::Result<(axum::http::StatusCode, axum::Json<crate::models::review::Review>)> {
    let review = crate::services::ReviewService::submit(&state.pool, &req).await?;
    Ok((axum::http::StatusCode::CREATED, axum::Json(review)))
}

/// POST /reviews/:id/helpful
async fn mark_review_helpful(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> crate::error::Result<axum::Json<crate::models::review::Review>> {
    let review = crate::services::ReviewService::mark_helpful(&state.pool, id).await?;
    Ok(axum::Json(review))
}
