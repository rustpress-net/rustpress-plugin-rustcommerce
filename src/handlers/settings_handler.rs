//! Settings HTTP handlers
//!
//! CRUD for shipping zones, shipping methods, tax rates, and coupons.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::Result;
use crate::models::coupon::*;
use crate::models::shipping::*;
use crate::models::tax::*;
use crate::plugin::AppState;
use crate::services::{CouponService, ShippingService, TaxService};

// ── Shipping Zone Handlers ─────────────────────────────────────────────

/// GET /admin/shipping/zones — List all shipping zones.
pub async fn list_shipping_zones(
    State(state): State<AppState>,
) -> Result<Json<Vec<ShippingZone>>> {
    let zones = ShippingService::list_zones(&state.pool).await?;
    Ok(Json(zones))
}

/// POST /admin/shipping/zones — Create a shipping zone.
pub async fn create_shipping_zone(
    State(state): State<AppState>,
    Json(req): Json<CreateShippingZoneRequest>,
) -> Result<(StatusCode, Json<ShippingZone>)> {
    let zone = ShippingService::create_zone(&state.pool, &req).await?;
    Ok((StatusCode::CREATED, Json(zone)))
}

/// GET /admin/shipping/zones/:id/methods — List methods for a zone.
pub async fn list_shipping_methods(
    State(state): State<AppState>,
    Path(zone_id): Path<Uuid>,
) -> Result<Json<Vec<ShippingMethod>>> {
    let methods = ShippingService::list_methods(&state.pool, zone_id).await?;
    Ok(Json(methods))
}

/// POST /admin/shipping/methods — Create a shipping method.
pub async fn create_shipping_method(
    State(state): State<AppState>,
    Json(req): Json<CreateShippingMethodRequest>,
) -> Result<(StatusCode, Json<ShippingMethod>)> {
    let method = ShippingService::create_method(&state.pool, &req).await?;
    Ok((StatusCode::CREATED, Json(method)))
}

/// POST /shipping/rates — Calculate shipping rates for an address.
pub async fn calculate_shipping_rates(
    State(state): State<AppState>,
    Json(req): Json<ShippingRateRequest>,
) -> Result<Json<Vec<ShippingRate>>> {
    let rates = ShippingService::get_rates(&state.pool, &req).await?;
    Ok(Json(rates))
}

// ── Tax Rate Handlers ──────────────────────────────────────────────────

/// GET /admin/tax/rates — List all tax rates.
pub async fn list_tax_rates(
    State(state): State<AppState>,
) -> Result<Json<Vec<TaxRate>>> {
    let rates = TaxService::list_rates(&state.pool).await?;
    Ok(Json(rates))
}

/// POST /admin/tax/rates — Create a tax rate.
pub async fn create_tax_rate(
    State(state): State<AppState>,
    Json(req): Json<CreateTaxRateRequest>,
) -> Result<(StatusCode, Json<TaxRate>)> {
    let rate = TaxService::create_rate(&state.pool, &req).await?;
    Ok((StatusCode::CREATED, Json(rate)))
}

/// PUT /admin/tax/rates/:id — Update a tax rate.
pub async fn update_tax_rate(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTaxRateRequest>,
) -> Result<Json<TaxRate>> {
    let rate = TaxService::update_rate(&state.pool, id, &req).await?;
    Ok(Json(rate))
}

/// DELETE /admin/tax/rates/:id — Delete a tax rate.
pub async fn delete_tax_rate(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    TaxService::delete_rate(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Coupon Handlers ────────────────────────────────────────────────────

/// Query params for coupon listing.
#[derive(Debug, Deserialize)]
pub struct CouponListQuery {
    pub active_only: Option<bool>,
}

/// GET /admin/coupons — List coupons.
pub async fn list_coupons(
    State(state): State<AppState>,
    Query(query): Query<CouponListQuery>,
) -> Result<Json<Vec<Coupon>>> {
    let active_only = query.active_only.unwrap_or(false);
    let coupons = CouponService::list(&state.pool, active_only).await?;
    Ok(Json(coupons))
}

/// GET /admin/coupons/:id — Get a coupon by ID.
pub async fn get_coupon(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Coupon>> {
    let coupon = CouponService::get_by_id(&state.pool, id).await?;
    Ok(Json(coupon))
}

/// POST /admin/coupons — Create a coupon.
pub async fn create_coupon(
    State(state): State<AppState>,
    Json(req): Json<CreateCouponRequest>,
) -> Result<(StatusCode, Json<Coupon>)> {
    let coupon = CouponService::create(&state.pool, &req).await?;
    Ok((StatusCode::CREATED, Json(coupon)))
}

/// PUT /admin/coupons/:id — Update a coupon.
pub async fn update_coupon(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCouponRequest>,
) -> Result<Json<Coupon>> {
    let coupon = CouponService::update(&state.pool, id, &req).await?;
    Ok(Json(coupon))
}

/// DELETE /admin/coupons/:id — Delete a coupon.
pub async fn delete_coupon(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    CouponService::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
