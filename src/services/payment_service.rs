//! Payment service
//!
//! Business logic for payment processing including Stripe integration.

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::StripeConfig;
use crate::error::{Result, RustCommerceError};
use crate::hooks::HookRegistry;
use crate::models::payment::*;
use crate::repositories::PaymentRepository;

/// Service for payment business logic.
pub struct PaymentService;

impl PaymentService {
    /// Create a payment record.
    pub async fn create_payment(pool: &PgPool, req: &CreatePaymentRequest) -> Result<Payment> {
        let payment = PaymentRepository::create(pool, req).await?;

        tracing::info!(
            payment_id = %payment.id,
            order_id = %payment.order_id,
            amount = %payment.amount,
            method = %payment.payment_method,
            "Payment created"
        );

        Ok(payment)
    }

    /// Create a Stripe payment intent for an order.
    pub async fn create_stripe_intent(
        pool: &PgPool,
        stripe_config: &StripeConfig,
        order_id: Uuid,
        amount: Decimal,
        currency: &str,
    ) -> Result<PaymentIntentResponse> {
        if !stripe_config.enabled {
            return Err(RustCommerceError::Configuration(
                "Stripe is not enabled".to_string(),
            ));
        }

        // Create payment record
        let payment = PaymentRepository::create(
            pool,
            &CreatePaymentRequest {
                order_id,
                payment_method: PaymentMethod::Stripe,
                amount,
                currency: Some(currency.to_string()),
                metadata: None,
            },
        )
        .await?;

        // Create Stripe PaymentIntent via stripe-rust
        let client = stripe::Client::new(&stripe_config.secret_key);

        // Convert Decimal to smallest currency unit (cents)
        let amount_cents = (amount * Decimal::from(100))
            .to_string()
            .parse::<i64>()
            .map_err(|_| {
                RustCommerceError::PaymentProcessing("Failed to convert amount".to_string())
            })?;

        let mut create_params = stripe::CreatePaymentIntent::new(amount_cents, stripe::Currency::USD);
        create_params.metadata = Some(
            [
                ("order_id".to_string(), order_id.to_string()),
                ("payment_id".to_string(), payment.id.to_string()),
            ]
            .into_iter()
            .collect(),
        );

        let intent = stripe::PaymentIntent::create(&client, create_params)
            .await
            .map_err(|e| RustCommerceError::Stripe(e.to_string()))?;

        // Update payment with Stripe transaction ID
        PaymentRepository::update_status(
            pool,
            payment.id,
            &PaymentStatus::Processing,
            Some(&intent.id.to_string()),
            None,
        )
        .await?;

        Ok(PaymentIntentResponse {
            payment_id: payment.id,
            client_secret: intent.client_secret,
            status: PaymentStatus::Processing,
            amount,
            currency: currency.to_string(),
        })
    }

    /// Handle Stripe webhook confirmation of payment.
    pub async fn confirm_stripe_payment(
        pool: &PgPool,
        gateway_transaction_id: &str,
    ) -> Result<Payment> {
        let payment = PaymentRepository::find_by_gateway_id(pool, gateway_transaction_id)
            .await?
            .ok_or_else(|| {
                RustCommerceError::PaymentNotFound(Uuid::nil())
            })?;

        let updated = PaymentRepository::update_status(
            pool,
            payment.id,
            &PaymentStatus::Completed,
            None,
            None,
        )
        .await?;

        tracing::info!(
            payment_id = %updated.id,
            order_id = %updated.order_id,
            "Payment confirmed via Stripe webhook"
        );

        HookRegistry::fire_action(
            "payment_completed",
            &serde_json::json!({
                "payment_id": updated.id,
                "order_id": updated.order_id,
                "amount": updated.amount.to_string(),
            }),
        )
        .await;

        Ok(updated)
    }

    /// Handle Stripe webhook for failed payment.
    pub async fn fail_stripe_payment(
        pool: &PgPool,
        gateway_transaction_id: &str,
        error_response: Option<&serde_json::Value>,
    ) -> Result<Payment> {
        let payment = PaymentRepository::find_by_gateway_id(pool, gateway_transaction_id)
            .await?
            .ok_or_else(|| {
                RustCommerceError::PaymentNotFound(Uuid::nil())
            })?;

        let updated = PaymentRepository::update_status(
            pool,
            payment.id,
            &PaymentStatus::Failed,
            None,
            error_response,
        )
        .await?;

        tracing::warn!(
            payment_id = %updated.id,
            order_id = %updated.order_id,
            "Payment failed via Stripe webhook"
        );

        HookRegistry::fire_action(
            "payment_failed",
            &serde_json::json!({
                "payment_id": updated.id,
                "order_id": updated.order_id,
            }),
        )
        .await;

        Ok(updated)
    }

    /// Process a refund.
    pub async fn process_refund(
        pool: &PgPool,
        req: &RefundRequest,
    ) -> Result<Payment> {
        let payment = PaymentRepository::find_by_id(pool, req.payment_id).await?;

        if payment.status != PaymentStatus::Completed {
            return Err(RustCommerceError::RefundError(
                "Can only refund completed payments".to_string(),
            ));
        }

        let refund_amount = req.amount.unwrap_or(payment.amount);
        let already_refunded = payment.refund_amount.unwrap_or(Decimal::ZERO);

        if refund_amount + already_refunded > payment.amount {
            return Err(RustCommerceError::RefundError(
                "Refund amount exceeds payment amount".to_string(),
            ));
        }

        let updated = PaymentRepository::record_refund(
            pool,
            req.payment_id,
            refund_amount,
            req.reason.as_deref(),
        )
        .await?;

        tracing::info!(
            payment_id = %updated.id,
            refund_amount = %refund_amount,
            "Refund processed"
        );

        Ok(updated)
    }

    /// Get payments for an order.
    pub async fn get_order_payments(pool: &PgPool, order_id: Uuid) -> Result<Vec<Payment>> {
        PaymentRepository::find_by_order(pool, order_id).await
    }
}
