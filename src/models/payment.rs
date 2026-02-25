//! Payment models
//!
//! Payment records tracking charges, refunds, and gateway responses.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Payment processing status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "rc_payment_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
    Refunded,
    PartiallyRefunded,
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Processing => write!(f, "processing"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Refunded => write!(f, "refunded"),
            Self::PartiallyRefunded => write!(f, "partially_refunded"),
        }
    }
}

/// Supported payment methods.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "rc_payment_method", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Stripe,
    Paypal,
    BankTransfer,
    CashOnDelivery,
    Manual,
}

impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stripe => write!(f, "stripe"),
            Self::Paypal => write!(f, "paypal"),
            Self::BankTransfer => write!(f, "bank_transfer"),
            Self::CashOnDelivery => write!(f, "cash_on_delivery"),
            Self::Manual => write!(f, "manual"),
        }
    }
}

/// A payment record associated with an order.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub order_id: Uuid,
    pub payment_method: PaymentMethod,
    pub status: PaymentStatus,
    pub amount: Decimal,
    pub currency: String,
    pub gateway_transaction_id: Option<String>,
    pub gateway_response: Option<serde_json::Value>,
    pub refund_amount: Option<Decimal>,
    pub refund_reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub paid_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub refunded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a payment intent (for Stripe).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    pub order_id: Uuid,
    pub payment_method: PaymentMethod,
    pub amount: Decimal,
    pub currency: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Request to process a refund.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRequest {
    pub payment_id: Uuid,
    pub amount: Option<Decimal>,
    pub reason: Option<String>,
}

/// Response from a payment intent creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntentResponse {
    pub payment_id: Uuid,
    pub client_secret: Option<String>,
    pub status: PaymentStatus,
    pub amount: Decimal,
    pub currency: String,
}
