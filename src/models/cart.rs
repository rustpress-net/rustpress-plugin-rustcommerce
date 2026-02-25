//! Cart models
//!
//! Shopping cart with items and totals calculation.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// A shopping cart belonging to a customer or anonymous session.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Cart {
    pub id: Uuid,
    pub customer_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub currency: String,
    pub subtotal: Decimal,
    pub tax_total: Decimal,
    pub shipping_total: Decimal,
    pub discount_total: Decimal,
    pub grand_total: Decimal,
    pub coupon_code: Option<String>,
    pub notes: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An individual item within a cart.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CartItem {
    pub id: Uuid,
    pub cart_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub total_price: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A cart with all its items loaded (used in API responses).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartWithItems {
    #[serde(flatten)]
    pub cart: Cart,
    pub items: Vec<CartItemDetail>,
    pub item_count: i32,
}

/// A cart item enriched with product name/image for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItemDetail {
    #[serde(flatten)]
    pub item: CartItem,
    pub product_name: Option<String>,
    pub product_slug: Option<String>,
    pub product_image_url: Option<String>,
    pub variant_name: Option<String>,
}

/// Request to add an item to the cart.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AddToCartRequest {
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    #[validate(range(min = 1, message = "Quantity must be at least 1"))]
    pub quantity: i32,
}

/// Request to update the quantity of a cart item.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCartItemRequest {
    #[validate(range(min = 1, message = "Quantity must be at least 1"))]
    pub quantity: i32,
}

/// Request to create or get a cart (for anonymous sessions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCartRequest {
    pub session_id: Option<String>,
    pub customer_id: Option<Uuid>,
    pub currency: Option<String>,
}

/// Request to apply a coupon to the cart.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ApplyCouponRequest {
    #[validate(length(min = 1, max = 100, message = "Coupon code must be 1-100 characters"))]
    pub code: String,
}

impl Cart {
    /// Recalculate the grand total from component totals.
    pub fn recalculate_grand_total(&mut self) {
        self.grand_total = self.subtotal + self.tax_total + self.shipping_total - self.discount_total;
        if self.grand_total < Decimal::ZERO {
            self.grand_total = Decimal::ZERO;
        }
    }
}

impl CartWithItems {
    /// Construct a CartWithItems from a cart and its loaded items.
    pub fn from_cart_and_items(cart: Cart, items: Vec<CartItemDetail>) -> Self {
        let item_count = items.iter().map(|i| i.item.quantity).sum();
        Self {
            cart,
            items,
            item_count,
        }
    }
}
