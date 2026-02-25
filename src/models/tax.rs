//! Tax models
//!
//! Tax rate definitions and calculation types.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// How tax is applied: percentage of subtotal or fixed amount.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "rc_tax_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TaxType {
    Percentage,
    Fixed,
}

impl std::fmt::Display for TaxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Percentage => write!(f, "percentage"),
            Self::Fixed => write!(f, "fixed"),
        }
    }
}

/// A tax rate applicable to a geographic region.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaxRate {
    pub id: Uuid,
    pub name: String,
    pub rate: Decimal,
    pub tax_type: TaxType,
    pub country: String,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub priority: i32,
    pub is_compound: bool,
    pub is_shipping_taxed: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a tax rate.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTaxRateRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub rate: Decimal,
    pub tax_type: Option<TaxType>,
    #[validate(length(equal = 2, message = "Country must be a 2-letter ISO code"))]
    pub country: String,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub priority: Option<i32>,
    #[serde(default)]
    pub is_compound: bool,
    #[serde(default)]
    pub is_shipping_taxed: bool,
}

/// Request to update a tax rate.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateTaxRateRequest {
    pub name: Option<String>,
    pub rate: Option<Decimal>,
    pub tax_type: Option<TaxType>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub priority: Option<i32>,
    pub is_compound: Option<bool>,
    pub is_shipping_taxed: Option<bool>,
    pub is_active: Option<bool>,
}

/// Result of a tax calculation for a given order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxCalculation {
    pub subtotal: Decimal,
    pub shipping_amount: Decimal,
    pub tax_lines: Vec<TaxLine>,
    pub total_tax: Decimal,
}

/// Individual tax line item showing which rate was applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLine {
    pub tax_rate_id: Uuid,
    pub name: String,
    pub rate: Decimal,
    pub tax_type: TaxType,
    pub taxable_amount: Decimal,
    pub tax_amount: Decimal,
}
