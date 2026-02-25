//! Order models
//!
//! Orders, order items, status workflow, and related DTOs.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::error::{Result, RustCommerceError};

/// Order status with defined state machine transitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "rc_order_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
    Failed,
}

impl OrderStatus {
    /// Valid transitions from the current status.
    pub fn valid_transitions(&self) -> Vec<OrderStatus> {
        match self {
            Self::Pending => vec![Self::Confirmed, Self::Cancelled, Self::Failed],
            Self::Confirmed => vec![Self::Processing, Self::Cancelled],
            Self::Processing => vec![Self::Shipped, Self::Cancelled],
            Self::Shipped => vec![Self::Delivered, Self::Cancelled],
            Self::Delivered => vec![Self::Refunded],
            Self::Cancelled => vec![],
            Self::Refunded => vec![],
            Self::Failed => vec![Self::Pending],
        }
    }

    /// Check whether transitioning to `target` is allowed.
    pub fn can_transition_to(&self, target: &OrderStatus) -> bool {
        self.valid_transitions().contains(target)
    }

    /// Attempt transition, returning error if invalid.
    pub fn transition_to(&self, target: OrderStatus) -> Result<OrderStatus> {
        if self.can_transition_to(&target) {
            Ok(target)
        } else {
            Err(RustCommerceError::InvalidStatusTransition {
                from: self.to_string(),
                to: target.to_string(),
            })
        }
    }
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Confirmed => write!(f, "confirmed"),
            Self::Processing => write!(f, "processing"),
            Self::Shipped => write!(f, "shipped"),
            Self::Delivered => write!(f, "delivered"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Refunded => write!(f, "refunded"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// An order placed by a customer.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub order_number: String,
    pub customer_id: Option<Uuid>,
    pub customer_email: String,
    pub customer_name: String,
    pub status: OrderStatus,
    pub currency: String,
    pub subtotal: Decimal,
    pub tax_total: Decimal,
    pub shipping_total: Decimal,
    pub discount_total: Decimal,
    pub grand_total: Decimal,
    pub shipping_address_line1: Option<String>,
    pub shipping_address_line2: Option<String>,
    pub shipping_city: Option<String>,
    pub shipping_state: Option<String>,
    pub shipping_postal_code: Option<String>,
    pub shipping_country: Option<String>,
    pub billing_address_line1: Option<String>,
    pub billing_address_line2: Option<String>,
    pub billing_city: Option<String>,
    pub billing_state: Option<String>,
    pub billing_postal_code: Option<String>,
    pub billing_country: Option<String>,
    pub shipping_method: Option<String>,
    pub tracking_number: Option<String>,
    pub coupon_code: Option<String>,
    pub notes: Option<String>,
    pub internal_notes: Option<String>,
    pub shipped_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single line item within an order.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub product_name: String,
    pub variant_name: Option<String>,
    pub sku: Option<String>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub total_price: Decimal,
    pub tax_amount: Decimal,
    pub discount_amount: Decimal,
    pub created_at: DateTime<Utc>,
}

/// An order with its items loaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderWithItems {
    #[serde(flatten)]
    pub order: Order,
    pub items: Vec<OrderItem>,
}

/// Request payload for updating order status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: OrderStatus,
    pub tracking_number: Option<String>,
    pub internal_notes: Option<String>,
}

/// Query parameters for listing orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub status: Option<OrderStatus>,
    pub customer_id: Option<Uuid>,
    pub customer_email: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Address information used at checkout.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CheckoutAddress {
    #[validate(length(min = 1, max = 255))]
    pub address_line1: String,
    pub address_line2: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub city: String,
    pub state: Option<String>,
    #[validate(length(min = 1, max = 20))]
    pub postal_code: String,
    #[validate(length(equal = 2))]
    pub country: String,
}

/// Checkout request to convert a cart into an order.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CheckoutRequest {
    pub cart_id: Uuid,
    #[validate(email(message = "Valid email required"))]
    pub customer_email: String,
    #[validate(length(min = 1, max = 255))]
    pub customer_name: String,
    pub customer_id: Option<Uuid>,
    #[validate(nested)]
    pub shipping_address: CheckoutAddress,
    #[validate(nested)]
    pub billing_address: Option<CheckoutAddress>,
    pub shipping_method: Option<String>,
    pub notes: Option<String>,
    pub payment_method: Option<String>,
}

/// Generate a unique order number.
pub fn generate_order_number() -> String {
    let now = Utc::now();
    let random: u32 = rand_component();
    format!("RC-{}{:04}", now.format("%Y%m%d%H%M"), random)
}

/// Simple random component for order numbers (no extra crate needed).
fn rand_component() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;

    let mut hasher = DefaultHasher::new();
    SystemTime::now().hash(&mut hasher);
    (hasher.finish() % 10000) as u32
}
