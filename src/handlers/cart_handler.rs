//! Cart HTTP handlers
//!
//! Endpoints for shopping cart management.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::error::Result;
use crate::models::cart::*;
use crate::plugin::AppState;
use crate::services::CartService;

/// POST /cart — Create or get a cart.
pub async fn create_cart(
    State(state): State<AppState>,
    Json(req): Json<CreateCartRequest>,
) -> Result<(StatusCode, Json<Cart>)> {
    let cart = CartService::get_or_create(&state.pool, &req).await?;
    Ok((StatusCode::CREATED, Json(cart)))
}

/// GET /cart/:id — Get a cart with all items.
pub async fn get_cart(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CartWithItems>> {
    let cart = CartService::get_with_items(&state.pool, id).await?;
    Ok(Json(cart))
}

/// POST /cart/:id/items — Add an item to a cart.
pub async fn add_cart_item(
    State(state): State<AppState>,
    Path(cart_id): Path<Uuid>,
    Json(req): Json<AddToCartRequest>,
) -> Result<(StatusCode, Json<CartItem>)> {
    let item = CartService::add_item(&state.pool, cart_id, &req).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// PUT /cart/:cart_id/items/:item_id — Update item quantity.
pub async fn update_cart_item(
    State(state): State<AppState>,
    Path((cart_id, item_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateCartItemRequest>,
) -> Result<Json<CartItem>> {
    let item = CartService::update_item(&state.pool, cart_id, item_id, &req).await?;
    Ok(Json(item))
}

/// DELETE /cart/:cart_id/items/:item_id — Remove an item from a cart.
pub async fn remove_cart_item(
    State(state): State<AppState>,
    Path((cart_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
    CartService::remove_item(&state.pool, cart_id, item_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /cart/:id/items — Clear all items from a cart.
pub async fn clear_cart(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    CartService::clear(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /cart/:id/coupon — Apply a coupon to a cart.
pub async fn apply_coupon(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApplyCouponRequest>,
) -> Result<Json<Cart>> {
    let cart = CartService::apply_coupon(&state.pool, id, &req.code).await?;
    Ok(Json(cart))
}

/// DELETE /cart/:id/coupon — Remove coupon from a cart.
pub async fn remove_coupon(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Cart>> {
    let cart = CartService::remove_coupon(&state.pool, id).await?;
    Ok(Json(cart))
}
