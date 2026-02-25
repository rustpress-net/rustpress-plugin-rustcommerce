//! Order HTTP handlers
//!
//! Endpoints for order management and status updates.

use axum::extract::{Path, Query, State};
use axum::Json;
use uuid::Uuid;

use crate::error::Result;
use crate::models::order::*;
use crate::models::product::PaginatedResponse;
use crate::plugin::AppState;
use crate::services::OrderService;

/// GET /orders — List orders with filtering.
pub async fn list_orders(
    State(state): State<AppState>,
    Query(params): Query<OrderListParams>,
) -> Result<Json<PaginatedResponse<Order>>> {
    let result = OrderService::list(&state.pool, &params).await?;
    Ok(Json(result))
}

/// GET /orders/:id — Get an order with items.
pub async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<OrderWithItems>> {
    let order = OrderService::get_with_items(&state.pool, id).await?;
    Ok(Json(order))
}

/// GET /orders/number/:number — Get an order by order number.
pub async fn get_order_by_number(
    State(state): State<AppState>,
    Path(number): Path<String>,
) -> Result<Json<OrderWithItems>> {
    let order = OrderService::get_by_order_number(&state.pool, &number).await?;
    Ok(Json(order))
}

/// PUT /orders/:id/status — Update order status.
pub async fn update_order_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateOrderStatusRequest>,
) -> Result<Json<Order>> {
    let order = OrderService::update_status(&state.pool, id, &req).await?;
    Ok(Json(order))
}
