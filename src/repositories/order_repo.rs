//! Order repository
//!
//! Database access for orders and order items.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{Result, RustCommerceError};
use crate::models::order::*;

/// Repository for order database operations.
pub struct OrderRepository;

impl OrderRepository {
    /// Fetch an order by ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Order> {
        sqlx::query_as::<_, Order>("SELECT * FROM rc_orders WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(RustCommerceError::OrderNotFound(id))
    }

    /// Fetch an order by order number.
    pub async fn find_by_order_number(pool: &PgPool, order_number: &str) -> Result<Order> {
        sqlx::query_as::<_, Order>("SELECT * FROM rc_orders WHERE order_number = $1")
            .bind(order_number)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| {
                RustCommerceError::InvalidInput(format!("Order '{}' not found", order_number))
            })
    }

    /// List orders with filtering and pagination.
    pub async fn list(pool: &PgPool, params: &OrderListParams) -> Result<(Vec<Order>, i64)> {
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(20).min(100);
        let offset = ((page - 1) * per_page) as i64;

        let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
        let sort_order = params.sort_order.as_deref().unwrap_or("desc");

        let sort_column = match sort_by {
            "order_number" => "order_number",
            "grand_total" => "grand_total",
            "created_at" => "created_at",
            "updated_at" => "updated_at",
            _ => "created_at",
        };
        let order = if sort_order.eq_ignore_ascii_case("asc") {
            "ASC"
        } else {
            "DESC"
        };

        let query = format!(
            r#"
            SELECT * FROM rc_orders
            WHERE ($1::rc_order_status IS NULL OR status = $1)
              AND ($2::uuid IS NULL OR customer_id = $2)
              AND ($3::text IS NULL OR customer_email = $3)
              AND ($4::timestamptz IS NULL OR created_at >= $4)
              AND ($5::timestamptz IS NULL OR created_at <= $5)
            ORDER BY {} {}
            LIMIT $6 OFFSET $7
            "#,
            sort_column, order
        );

        let orders = sqlx::query_as::<_, Order>(&query)
            .bind(&params.status)
            .bind(&params.customer_id)
            .bind(&params.customer_email)
            .bind(&params.date_from)
            .bind(&params.date_to)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM rc_orders
            WHERE ($1::rc_order_status IS NULL OR status = $1)
              AND ($2::uuid IS NULL OR customer_id = $2)
              AND ($3::text IS NULL OR customer_email = $3)
              AND ($4::timestamptz IS NULL OR created_at >= $4)
              AND ($5::timestamptz IS NULL OR created_at <= $5)
            "#,
        )
        .bind(&params.status)
        .bind(&params.customer_id)
        .bind(&params.customer_email)
        .bind(&params.date_from)
        .bind(&params.date_to)
        .fetch_one(pool)
        .await?;

        Ok((orders, total))
    }

    /// Create a new order from a checkout request and cart data.
    pub async fn create(pool: &PgPool, req: &CheckoutRequest, order_number: &str, subtotal: rust_decimal::Decimal, tax_total: rust_decimal::Decimal, shipping_total: rust_decimal::Decimal, discount_total: rust_decimal::Decimal, grand_total: rust_decimal::Decimal, coupon_code: Option<&str>) -> Result<Order> {
        let billing = req.billing_address.as_ref().unwrap_or(&req.shipping_address);

        let order = sqlx::query_as::<_, Order>(
            r#"
            INSERT INTO rc_orders (
                order_number, customer_id, customer_email, customer_name, status, currency,
                subtotal, tax_total, shipping_total, discount_total, grand_total,
                shipping_address_line1, shipping_address_line2, shipping_city,
                shipping_state, shipping_postal_code, shipping_country,
                billing_address_line1, billing_address_line2, billing_city,
                billing_state, billing_postal_code, billing_country,
                shipping_method, coupon_code, notes
            ) VALUES (
                $1, $2, $3, $4, 'pending', 'USD',
                $5, $6, $7, $8, $9,
                $10, $11, $12, $13, $14, $15,
                $16, $17, $18, $19, $20, $21,
                $22, $23, $24
            ) RETURNING *
            "#,
        )
        .bind(order_number)
        .bind(&req.customer_id)
        .bind(&req.customer_email)
        .bind(&req.customer_name)
        .bind(subtotal)
        .bind(tax_total)
        .bind(shipping_total)
        .bind(discount_total)
        .bind(grand_total)
        .bind(&req.shipping_address.address_line1)
        .bind(&req.shipping_address.address_line2)
        .bind(&req.shipping_address.city)
        .bind(&req.shipping_address.state)
        .bind(&req.shipping_address.postal_code)
        .bind(&req.shipping_address.country)
        .bind(&billing.address_line1)
        .bind(&billing.address_line2)
        .bind(&billing.city)
        .bind(&billing.state)
        .bind(&billing.postal_code)
        .bind(&billing.country)
        .bind(&req.shipping_method)
        .bind(coupon_code)
        .bind(&req.notes)
        .fetch_one(pool)
        .await?;

        Ok(order)
    }

    /// Create an order item.
    pub async fn create_item(
        pool: &PgPool,
        order_id: Uuid,
        product_id: Uuid,
        variant_id: Option<Uuid>,
        product_name: &str,
        variant_name: Option<&str>,
        sku: Option<&str>,
        quantity: i32,
        unit_price: rust_decimal::Decimal,
        total_price: rust_decimal::Decimal,
        tax_amount: rust_decimal::Decimal,
        discount_amount: rust_decimal::Decimal,
    ) -> Result<OrderItem> {
        let item = sqlx::query_as::<_, OrderItem>(
            r#"
            INSERT INTO rc_order_items (
                order_id, product_id, variant_id, product_name, variant_name, sku,
                quantity, unit_price, total_price, tax_amount, discount_amount
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(order_id)
        .bind(product_id)
        .bind(variant_id)
        .bind(product_name)
        .bind(variant_name)
        .bind(sku)
        .bind(quantity)
        .bind(unit_price)
        .bind(total_price)
        .bind(tax_amount)
        .bind(discount_amount)
        .fetch_one(pool)
        .await?;

        Ok(item)
    }

    /// List items belonging to an order.
    pub async fn list_items(pool: &PgPool, order_id: Uuid) -> Result<Vec<OrderItem>> {
        let items = sqlx::query_as::<_, OrderItem>(
            "SELECT * FROM rc_order_items WHERE order_id = $1 ORDER BY created_at",
        )
        .bind(order_id)
        .fetch_all(pool)
        .await?;

        Ok(items)
    }

    /// Update an order's status.
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: &OrderStatus,
        tracking_number: Option<&str>,
        internal_notes: Option<&str>,
    ) -> Result<Order> {
        // Set timestamp fields based on status
        let now = chrono::Utc::now();
        let order = match status {
            OrderStatus::Shipped => {
                sqlx::query_as::<_, Order>(
                    r#"
                    UPDATE rc_orders SET status = $1, tracking_number = COALESCE($2, tracking_number),
                    internal_notes = COALESCE($3, internal_notes), shipped_at = $4 WHERE id = $5 RETURNING *
                    "#,
                )
                .bind(status)
                .bind(tracking_number)
                .bind(internal_notes)
                .bind(now)
                .bind(id)
                .fetch_one(pool)
                .await?
            }
            OrderStatus::Delivered => {
                sqlx::query_as::<_, Order>(
                    r#"
                    UPDATE rc_orders SET status = $1, internal_notes = COALESCE($2, internal_notes),
                    delivered_at = $3 WHERE id = $4 RETURNING *
                    "#,
                )
                .bind(status)
                .bind(internal_notes)
                .bind(now)
                .bind(id)
                .fetch_one(pool)
                .await?
            }
            OrderStatus::Cancelled => {
                sqlx::query_as::<_, Order>(
                    r#"
                    UPDATE rc_orders SET status = $1, internal_notes = COALESCE($2, internal_notes),
                    cancelled_at = $3 WHERE id = $4 RETURNING *
                    "#,
                )
                .bind(status)
                .bind(internal_notes)
                .bind(now)
                .bind(id)
                .fetch_one(pool)
                .await?
            }
            _ => {
                sqlx::query_as::<_, Order>(
                    r#"
                    UPDATE rc_orders SET status = $1, tracking_number = COALESCE($2, tracking_number),
                    internal_notes = COALESCE($3, internal_notes) WHERE id = $4 RETURNING *
                    "#,
                )
                .bind(status)
                .bind(tracking_number)
                .bind(internal_notes)
                .bind(id)
                .fetch_one(pool)
                .await?
            }
        };

        Ok(order)
    }

    /// Count orders by status (for dashboard).
    pub async fn count_by_status(pool: &PgPool) -> Result<Vec<(String, i64)>> {
        let rows = sqlx::query_as::<_, (String, i64)>(
            "SELECT status::text, COUNT(*) FROM rc_orders GROUP BY status",
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    /// Get total revenue.
    pub async fn total_revenue(pool: &PgPool) -> Result<rust_decimal::Decimal> {
        let total = sqlx::query_scalar::<_, Option<rust_decimal::Decimal>>(
            "SELECT SUM(grand_total) FROM rc_orders WHERE status NOT IN ('cancelled', 'refunded', 'failed')",
        )
        .fetch_one(pool)
        .await?;

        Ok(total.unwrap_or(rust_decimal::Decimal::ZERO))
    }
}
