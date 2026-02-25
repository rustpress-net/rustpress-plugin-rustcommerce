//! Webhook HTTP handlers
//!
//! Stripe webhook handler for payment event processing.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::body::Bytes;

use crate::error::{Result, RustCommerceError};
use crate::models::order::OrderStatus;
use crate::plugin::AppState;
use crate::services::{OrderService, PaymentService};

/// POST /webhooks/stripe — Handle Stripe webhook events.
///
/// Validates the webhook signature, then processes the event type.
pub async fn handle_stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode> {
    // Extract Stripe signature
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            RustCommerceError::Unauthorized("Missing Stripe signature header".to_string())
        })?;

    // Verify webhook signature
    let webhook_secret = &state.config.stripe.webhook_secret;
    if webhook_secret.is_empty() {
        return Err(RustCommerceError::Configuration(
            "Stripe webhook secret not configured".to_string(),
        ));
    }

    let payload = std::str::from_utf8(&body).map_err(|_| {
        RustCommerceError::InvalidInput("Invalid webhook payload encoding".to_string())
    })?;

    // Verify the Stripe signature
    let event = stripe::Webhook::construct_event(payload, signature, webhook_secret)
        .map_err(|e| {
            tracing::warn!(error = %e, "Invalid Stripe webhook signature");
            RustCommerceError::Unauthorized("Invalid webhook signature".to_string())
        })?;

    tracing::info!(
        event_type = %event.type_,
        event_id = %event.id,
        "Received Stripe webhook"
    );

    // Process event based on type
    match event.type_ {
        stripe::EventType::PaymentIntentSucceeded => {
            if let stripe::EventObject::PaymentIntent(intent) = event.data.object {
                let intent_id = intent.id.to_string();

                // Mark payment as completed
                let payment = PaymentService::confirm_stripe_payment(&state.pool, &intent_id).await?;

                // Update order status to confirmed
                let _ = OrderService::update_status(
                    &state.pool,
                    payment.order_id,
                    &crate::models::order::UpdateOrderStatusRequest {
                        status: OrderStatus::Confirmed,
                        tracking_number: None,
                        internal_notes: Some("Payment confirmed via Stripe webhook".to_string()),
                    },
                )
                .await;

                tracing::info!(
                    payment_intent_id = %intent_id,
                    "Payment intent succeeded"
                );
            }
        }

        stripe::EventType::PaymentIntentPaymentFailed => {
            if let stripe::EventObject::PaymentIntent(intent) = event.data.object {
                let intent_id = intent.id.to_string();

                let error_response = intent
                    .last_payment_error
                    .as_ref()
                    .map(|e| serde_json::json!({ "message": e.message }));

                let payment = PaymentService::fail_stripe_payment(
                    &state.pool,
                    &intent_id,
                    error_response.as_ref(),
                )
                .await?;

                // Update order status to failed
                let _ = OrderService::update_status(
                    &state.pool,
                    payment.order_id,
                    &crate::models::order::UpdateOrderStatusRequest {
                        status: OrderStatus::Failed,
                        tracking_number: None,
                        internal_notes: Some("Payment failed via Stripe webhook".to_string()),
                    },
                )
                .await;

                tracing::warn!(
                    payment_intent_id = %intent_id,
                    "Payment intent failed"
                );
            }
        }

        stripe::EventType::ChargeRefunded => {
            tracing::info!("Charge refunded event received - processing handled by refund flow");
        }

        _ => {
            tracing::debug!(event_type = %event.type_, "Unhandled Stripe event type");
        }
    }

    Ok(StatusCode::OK)
}
