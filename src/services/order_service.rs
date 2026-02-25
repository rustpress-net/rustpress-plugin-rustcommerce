//! Order service
//!
//! Business logic for order management, status transitions, and history.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Result;
use crate::hooks::HookRegistry;
use crate::models::order::*;
use crate::models::product::PaginatedResponse;
use crate::repositories::OrderRepository;

/// Service for order business logic.
pub struct OrderService;

impl OrderService {
    /// List orders with filtering and pagination.
    pub async fn list(
        pool: &PgPool,
        params: &OrderListParams,
    ) -> Result<PaginatedResponse<Order>> {
        let (orders, total) = OrderRepository::list(pool, params).await?;
        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);
        Ok(PaginatedResponse::new(orders, total, page, per_page))
    }

    /// Get an order by ID with its items.
    pub async fn get_with_items(pool: &PgPool, id: Uuid) -> Result<OrderWithItems> {
        let order = OrderRepository::find_by_id(pool, id).await?;
        let items = OrderRepository::list_items(pool, id).await?;
        Ok(OrderWithItems { order, items })
    }

    /// Get an order by order number with items.
    pub async fn get_by_order_number(
        pool: &PgPool,
        order_number: &str,
    ) -> Result<OrderWithItems> {
        let order = OrderRepository::find_by_order_number(pool, order_number).await?;
        let items = OrderRepository::list_items(pool, order.id).await?;
        Ok(OrderWithItems { order, items })
    }

    /// Update order status with validation of the state transition.
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateOrderStatusRequest,
    ) -> Result<Order> {
        let existing = OrderRepository::find_by_id(pool, id).await?;

        // Validate transition
        let new_status = existing.status.transition_to(req.status.clone())?;

        let order = OrderRepository::update_status(
            pool,
            id,
            &new_status,
            req.tracking_number.as_deref(),
            req.internal_notes.as_deref(),
        )
        .await?;

        tracing::info!(
            order_id = %id,
            from = %existing.status,
            to = %new_status,
            "Order status updated"
        );

        HookRegistry::fire_action(
            "order_status_changed",
            &serde_json::json!({
                "order_id": id,
                "from": existing.status.to_string(),
                "to": new_status.to_string(),
            }),
        )
        .await;

        Ok(order)
    }

    /// Get order counts by status for dashboard display.
    pub async fn get_status_counts(pool: &PgPool) -> Result<Vec<(String, i64)>> {
        OrderRepository::count_by_status(pool).await
    }

    /// Get total revenue across all completed orders.
    pub async fn get_total_revenue(pool: &PgPool) -> Result<rust_decimal::Decimal> {
        OrderRepository::total_revenue(pool).await
    }
}
