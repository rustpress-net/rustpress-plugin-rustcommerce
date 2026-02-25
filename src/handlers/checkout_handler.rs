//! Checkout HTTP handlers
//!
//! Endpoints for the checkout workflow.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::error::Result;
use crate::models::order::*;
use crate::plugin::AppState;
use crate::services::CheckoutService;

/// POST /checkout — Process checkout: validate cart, create order, initiate payment.
pub async fn process_checkout(
    State(state): State<AppState>,
    Json(req): Json<CheckoutRequest>,
) -> Result<(StatusCode, Json<OrderWithItems>)> {
    let order = CheckoutService::process(&state.pool, &state.config, &req).await?;
    Ok((StatusCode::CREATED, Json(order)))
}
