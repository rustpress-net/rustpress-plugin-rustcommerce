//! Shipping models
//!
//! Shipping zones, methods, and rate types.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// How shipping cost is calculated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "rc_shipping_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ShippingType {
    FlatRate,
    Free,
    WeightBased,
    PriceBased,
    PerItem,
}

impl std::fmt::Display for ShippingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FlatRate => write!(f, "flat_rate"),
            Self::Free => write!(f, "free"),
            Self::WeightBased => write!(f, "weight_based"),
            Self::PriceBased => write!(f, "price_based"),
            Self::PerItem => write!(f, "per_item"),
        }
    }
}

/// A geographic zone for shipping rate determination.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShippingZone {
    pub id: Uuid,
    pub name: String,
    pub countries: Vec<String>,
    pub states: Vec<String>,
    pub postal_codes: Vec<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A shipping method available within a zone.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShippingMethod {
    pub id: Uuid,
    pub zone_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub shipping_type: ShippingType,
    pub price: Decimal,
    pub min_order_amount: Option<Decimal>,
    pub max_order_amount: Option<Decimal>,
    pub min_weight: Option<Decimal>,
    pub max_weight: Option<Decimal>,
    pub estimated_days_min: Option<i32>,
    pub estimated_days_max: Option<i32>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A calculated shipping rate returned to the customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingRate {
    pub method_id: Uuid,
    pub method_name: String,
    pub price: Decimal,
    pub estimated_days_min: Option<i32>,
    pub estimated_days_max: Option<i32>,
    pub description: Option<String>,
}

/// Request to create a shipping zone.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateShippingZoneRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[serde(default)]
    pub countries: Vec<String>,
    #[serde(default)]
    pub states: Vec<String>,
    #[serde(default)]
    pub postal_codes: Vec<String>,
}

/// Request to create a shipping method.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateShippingMethodRequest {
    pub zone_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    pub shipping_type: ShippingType,
    pub price: Decimal,
    pub min_order_amount: Option<Decimal>,
    pub max_order_amount: Option<Decimal>,
    pub min_weight: Option<Decimal>,
    pub max_weight: Option<Decimal>,
    pub estimated_days_min: Option<i32>,
    pub estimated_days_max: Option<i32>,
}

/// Parameters for calculating shipping rates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingRateRequest {
    pub country: String,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub order_subtotal: Decimal,
    pub total_weight: Option<Decimal>,
    pub item_count: i32,
}
