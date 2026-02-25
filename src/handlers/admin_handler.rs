//! Admin HTTP handlers
//!
//! Dashboard analytics and administrative endpoints.

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::models::product::*;
use crate::models::review::*;
use crate::plugin::AppState;
use crate::services::{InventoryService, OrderService, ReviewService};

/// Dashboard summary response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_revenue: String,
    pub order_counts: Vec<OrderStatusCount>,
    pub low_stock_products: Vec<LowStockItem>,
    pub recent_reviews: Vec<Review>,
}

/// Order count by status for dashboard.
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderStatusCount {
    pub status: String,
    pub count: i64,
}

/// Low stock product summary.
#[derive(Debug, Serialize, Deserialize)]
pub struct LowStockItem {
    pub id: uuid::Uuid,
    pub name: String,
    pub sku: Option<String>,
    pub stock_quantity: i32,
    pub low_stock_threshold: i32,
}

/// GET /admin/dashboard — Get dashboard analytics.
pub async fn get_dashboard(
    State(state): State<AppState>,
) -> Result<Json<DashboardStats>> {
    let revenue = OrderService::get_total_revenue(&state.pool).await?;
    let status_counts = OrderService::get_status_counts(&state.pool).await?;
    let low_stock = InventoryService::get_low_stock_products(&state.pool).await?;

    let order_counts = status_counts
        .into_iter()
        .map(|(status, count)| OrderStatusCount { status, count })
        .collect();

    let low_stock_items = low_stock
        .into_iter()
        .map(|p| LowStockItem {
            id: p.id,
            name: p.name,
            sku: p.sku,
            stock_quantity: p.stock_quantity,
            low_stock_threshold: p.low_stock_threshold,
        })
        .collect();

    // Get recent pending reviews
    let review_params = ReviewListParams {
        page: Some(1),
        per_page: Some(5),
        product_id: None,
        customer_id: None,
        status: Some(ReviewStatus::Pending),
        min_rating: None,
        max_rating: None,
        sort_by: Some("created_at".to_string()),
        sort_order: Some("desc".to_string()),
    };
    let recent_reviews_result = ReviewService::list(&state.pool, &review_params).await?;

    Ok(Json(DashboardStats {
        total_revenue: revenue.to_string(),
        order_counts,
        low_stock_products: low_stock_items,
        recent_reviews: recent_reviews_result.items,
    }))
}

/// Query params for admin review listing.
#[derive(Debug, Deserialize)]
pub struct AdminReviewQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub status: Option<ReviewStatus>,
}

/// GET /admin/reviews — List reviews for moderation.
pub async fn list_reviews_admin(
    State(state): State<AppState>,
    Query(query): Query<AdminReviewQuery>,
) -> Result<Json<PaginatedResponse<Review>>> {
    let params = ReviewListParams {
        page: query.page,
        per_page: query.per_page,
        product_id: None,
        customer_id: None,
        status: query.status,
        min_rating: None,
        max_rating: None,
        sort_by: Some("created_at".to_string()),
        sort_order: Some("desc".to_string()),
    };

    let result = ReviewService::list(&state.pool, &params).await?;
    Ok(Json(result))
}

/// PUT /admin/reviews/:id/moderate — Moderate a review.
pub async fn moderate_review(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(req): Json<crate::models::review::ModerateReviewRequest>,
) -> Result<Json<Review>> {
    let review = ReviewService::moderate(&state.pool, id, &req).await?;
    Ok(Json(review))
}
