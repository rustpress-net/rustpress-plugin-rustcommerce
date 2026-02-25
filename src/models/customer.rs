//! Customer models
//!
//! Customer profiles and addresses for shipping/billing.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// A registered customer.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub is_active: bool,
    pub accepts_marketing: bool,
    pub total_orders: i32,
    pub total_spent: Decimal,
    pub notes: Option<String>,
    pub last_order_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Customer {
    /// Full name for display purposes.
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

/// A saved address for a customer.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomerAddress {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub label: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub company: Option<String>,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
    pub phone: Option<String>,
    pub is_default_shipping: bool,
    pub is_default_billing: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request payload for creating a customer.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCustomerRequest {
    pub user_id: Option<Uuid>,
    #[validate(email(message = "Valid email required"))]
    pub email: String,
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    pub phone: Option<String>,
    pub company: Option<String>,
    #[serde(default)]
    pub accepts_marketing: bool,
    pub notes: Option<String>,
}

/// Request payload for updating a customer.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCustomerRequest {
    #[validate(email(message = "Valid email required"))]
    pub email: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub is_active: Option<bool>,
    pub accepts_marketing: Option<bool>,
    pub notes: Option<String>,
}

/// Request payload for creating a customer address.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateAddressRequest {
    pub label: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    pub company: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub address_line1: String,
    pub address_line2: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub city: String,
    pub state: Option<String>,
    #[validate(length(min = 1, max = 20))]
    pub postal_code: String,
    #[validate(length(equal = 2, message = "Country must be a 2-letter ISO code"))]
    pub country: String,
    pub phone: Option<String>,
    #[serde(default)]
    pub is_default_shipping: bool,
    #[serde(default)]
    pub is_default_billing: bool,
}

/// Query parameters for listing customers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub is_active: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}
