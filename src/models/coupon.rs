//! Coupon models
//!
//! Discount coupons with validation logic.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::error::{Result, RustCommerceError};

/// How the discount is applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "rc_discount_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DiscountType {
    Percentage,
    FixedAmount,
    FreeShipping,
}

impl std::fmt::Display for DiscountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Percentage => write!(f, "percentage"),
            Self::FixedAmount => write!(f, "fixed_amount"),
            Self::FreeShipping => write!(f, "free_shipping"),
        }
    }
}

/// A discount coupon.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Coupon {
    pub id: Uuid,
    pub code: String,
    pub description: Option<String>,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
    pub minimum_order_amount: Option<Decimal>,
    pub maximum_discount_amount: Option<Decimal>,
    pub usage_limit: Option<i32>,
    pub usage_count: i32,
    pub usage_limit_per_customer: Option<i32>,
    pub applies_to_product_ids: Option<Vec<Uuid>>,
    pub applies_to_category_ids: Option<Vec<Uuid>>,
    pub excluded_product_ids: Option<Vec<Uuid>>,
    pub excluded_category_ids: Option<Vec<Uuid>>,
    pub is_active: bool,
    pub starts_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Coupon {
    /// Validate that this coupon can be used right now for the given order amount.
    pub fn validate_usage(&self, order_subtotal: Decimal) -> Result<()> {
        if !self.is_active {
            return Err(RustCommerceError::InvalidCoupon(
                "Coupon is not active".to_string(),
            ));
        }

        let now = Utc::now();

        if let Some(starts_at) = self.starts_at {
            if now < starts_at {
                return Err(RustCommerceError::InvalidCoupon(
                    "Coupon is not yet valid".to_string(),
                ));
            }
        }

        if let Some(expires_at) = self.expires_at {
            if now > expires_at {
                return Err(RustCommerceError::CouponExpired);
            }
        }

        if let Some(limit) = self.usage_limit {
            if self.usage_count >= limit {
                return Err(RustCommerceError::CouponUsageLimitReached);
            }
        }

        if let Some(min_amount) = self.minimum_order_amount {
            if order_subtotal < min_amount {
                return Err(RustCommerceError::MinimumOrderNotMet {
                    required: min_amount,
                    current: order_subtotal,
                });
            }
        }

        Ok(())
    }

    /// Calculate the discount amount for a given subtotal.
    pub fn calculate_discount(&self, subtotal: Decimal) -> Decimal {
        let discount = match self.discount_type {
            DiscountType::Percentage => {
                let pct = self.discount_value / Decimal::from(100);
                subtotal * pct
            }
            DiscountType::FixedAmount => self.discount_value,
            DiscountType::FreeShipping => Decimal::ZERO, // handled separately
        };

        // Cap at maximum discount if set
        let discount = if let Some(max) = self.maximum_discount_amount {
            discount.min(max)
        } else {
            discount
        };

        // Never discount more than the subtotal
        discount.min(subtotal)
    }
}

/// Request to create a coupon.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCouponRequest {
    #[validate(length(min = 1, max = 100, message = "Code must be 1-100 characters"))]
    pub code: String,
    pub description: Option<String>,
    pub discount_type: DiscountType,
    pub discount_value: Decimal,
    pub minimum_order_amount: Option<Decimal>,
    pub maximum_discount_amount: Option<Decimal>,
    pub usage_limit: Option<i32>,
    pub usage_limit_per_customer: Option<i32>,
    pub applies_to_product_ids: Option<Vec<Uuid>>,
    pub applies_to_category_ids: Option<Vec<Uuid>>,
    pub excluded_product_ids: Option<Vec<Uuid>>,
    pub excluded_category_ids: Option<Vec<Uuid>>,
    pub starts_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Request to update a coupon.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCouponRequest {
    pub description: Option<String>,
    pub discount_type: Option<DiscountType>,
    pub discount_value: Option<Decimal>,
    pub minimum_order_amount: Option<Decimal>,
    pub maximum_discount_amount: Option<Decimal>,
    pub usage_limit: Option<i32>,
    pub usage_limit_per_customer: Option<i32>,
    pub is_active: Option<bool>,
    pub starts_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}
