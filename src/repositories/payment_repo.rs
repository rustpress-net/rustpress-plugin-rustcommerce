//! Payment repository
//!
//! Database access for payment records.

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{Result, RustCommerceError};
use crate::models::payment::*;

/// Repository for payment database operations.
pub struct PaymentRepository;

impl PaymentRepository {
    /// Find a payment by ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Payment> {
        sqlx::query_as::<_, Payment>("SELECT * FROM rc_payments WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(RustCommerceError::PaymentNotFound(id))
    }

    /// Find payments for an order.
    pub async fn find_by_order(pool: &PgPool, order_id: Uuid) -> Result<Vec<Payment>> {
        let payments = sqlx::query_as::<_, Payment>(
            "SELECT * FROM rc_payments WHERE order_id = $1 ORDER BY created_at DESC",
        )
        .bind(order_id)
        .fetch_all(pool)
        .await?;

        Ok(payments)
    }

    /// Find a payment by gateway transaction ID (e.g., Stripe PaymentIntent ID).
    pub async fn find_by_gateway_id(pool: &PgPool, gateway_id: &str) -> Result<Option<Payment>> {
        let payment = sqlx::query_as::<_, Payment>(
            "SELECT * FROM rc_payments WHERE gateway_transaction_id = $1",
        )
        .bind(gateway_id)
        .fetch_optional(pool)
        .await?;

        Ok(payment)
    }

    /// Create a new payment record.
    pub async fn create(pool: &PgPool, req: &CreatePaymentRequest) -> Result<Payment> {
        let currency = req.currency.as_deref().unwrap_or("USD");

        let payment = sqlx::query_as::<_, Payment>(
            r#"
            INSERT INTO rc_payments (order_id, payment_method, status, amount, currency, metadata)
            VALUES ($1, $2, 'pending', $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(req.order_id)
        .bind(&req.payment_method)
        .bind(req.amount)
        .bind(currency)
        .bind(&req.metadata)
        .fetch_one(pool)
        .await?;

        Ok(payment)
    }

    /// Update payment status and optionally set gateway details.
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: &PaymentStatus,
        gateway_transaction_id: Option<&str>,
        gateway_response: Option<&serde_json::Value>,
    ) -> Result<Payment> {
        let now = chrono::Utc::now();

        let payment = match status {
            PaymentStatus::Completed => {
                sqlx::query_as::<_, Payment>(
                    r#"
                    UPDATE rc_payments SET
                        status = $1, gateway_transaction_id = COALESCE($2, gateway_transaction_id),
                        gateway_response = COALESCE($3, gateway_response), paid_at = $4
                    WHERE id = $5 RETURNING *
                    "#,
                )
                .bind(status)
                .bind(gateway_transaction_id)
                .bind(gateway_response)
                .bind(now)
                .bind(id)
                .fetch_one(pool)
                .await?
            }
            PaymentStatus::Failed => {
                sqlx::query_as::<_, Payment>(
                    r#"
                    UPDATE rc_payments SET
                        status = $1, gateway_transaction_id = COALESCE($2, gateway_transaction_id),
                        gateway_response = COALESCE($3, gateway_response), failed_at = $4
                    WHERE id = $5 RETURNING *
                    "#,
                )
                .bind(status)
                .bind(gateway_transaction_id)
                .bind(gateway_response)
                .bind(now)
                .bind(id)
                .fetch_one(pool)
                .await?
            }
            _ => {
                sqlx::query_as::<_, Payment>(
                    r#"
                    UPDATE rc_payments SET
                        status = $1, gateway_transaction_id = COALESCE($2, gateway_transaction_id),
                        gateway_response = COALESCE($3, gateway_response)
                    WHERE id = $4 RETURNING *
                    "#,
                )
                .bind(status)
                .bind(gateway_transaction_id)
                .bind(gateway_response)
                .bind(id)
                .fetch_one(pool)
                .await?
            }
        };

        Ok(payment)
    }

    /// Record a refund on a payment.
    pub async fn record_refund(
        pool: &PgPool,
        id: Uuid,
        refund_amount: Decimal,
        reason: Option<&str>,
    ) -> Result<Payment> {
        let existing = Self::find_by_id(pool, id).await?;
        let new_refund_total = existing.refund_amount.unwrap_or(Decimal::ZERO) + refund_amount;

        let status = if new_refund_total >= existing.amount {
            PaymentStatus::Refunded
        } else {
            PaymentStatus::PartiallyRefunded
        };

        let payment = sqlx::query_as::<_, Payment>(
            r#"
            UPDATE rc_payments SET
                status = $1, refund_amount = $2, refund_reason = COALESCE($3, refund_reason),
                refunded_at = NOW()
            WHERE id = $4 RETURNING *
            "#,
        )
        .bind(&status)
        .bind(new_refund_total)
        .bind(reason)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(payment)
    }
}
