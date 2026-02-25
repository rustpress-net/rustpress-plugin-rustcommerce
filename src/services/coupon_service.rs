//! Coupon service
//!
//! Business logic for coupon validation, application, and management.

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::error::{Result, RustCommerceError};
use crate::hooks::HookRegistry;
use crate::models::coupon::*;
use crate::repositories::CouponRepository;

/// Service for coupon business logic.
pub struct CouponService;

impl CouponService {
    /// Validate and return discount amount for a coupon code.
    pub async fn validate_and_calculate(
        pool: &PgPool,
        code: &str,
        subtotal: Decimal,
    ) -> Result<(Coupon, Decimal)> {
        let coupon = CouponRepository::find_by_code(pool, code).await?;

        // Validate coupon is usable
        coupon.validate_usage(subtotal)?;

        // Calculate discount
        let discount = coupon.calculate_discount(subtotal);

        Ok((coupon, discount))
    }

    /// Create a new coupon.
    pub async fn create(pool: &PgPool, req: &CreateCouponRequest) -> Result<Coupon> {
        req.validate()
            .map_err(|e| RustCommerceError::Validation(e.to_string()))?;

        // Validate discount value based on type
        match req.discount_type {
            DiscountType::Percentage => {
                if req.discount_value > Decimal::from(100) {
                    return Err(RustCommerceError::Validation(
                        "Percentage discount cannot exceed 100%".to_string(),
                    ));
                }
            }
            DiscountType::FixedAmount => {
                if req.discount_value <= Decimal::ZERO {
                    return Err(RustCommerceError::Validation(
                        "Fixed discount must be positive".to_string(),
                    ));
                }
            }
            DiscountType::FreeShipping => {
                // Free shipping doesn't need a value, but we store it anyway
            }
        }

        let coupon = CouponRepository::create(pool, req).await?;

        tracing::info!(
            coupon_id = %coupon.id,
            code = %coupon.code,
            "Coupon created"
        );

        Ok(coupon)
    }

    /// Update a coupon.
    pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateCouponRequest) -> Result<Coupon> {
        CouponRepository::update(pool, id, req).await
    }

    /// List all coupons.
    pub async fn list(pool: &PgPool, active_only: bool) -> Result<Vec<Coupon>> {
        CouponRepository::list(pool, active_only).await
    }

    /// Get a coupon by ID.
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Coupon> {
        CouponRepository::find_by_id(pool, id).await
    }

    /// Apply a coupon to an order/cart (increments usage).
    pub async fn apply(pool: &PgPool, code: &str, subtotal: Decimal) -> Result<Decimal> {
        let (coupon, discount) = Self::validate_and_calculate(pool, code, subtotal).await?;

        CouponRepository::increment_usage(pool, coupon.id).await?;

        HookRegistry::fire_action(
            "coupon_applied",
            &serde_json::json!({
                "coupon_id": coupon.id,
                "code": coupon.code,
                "discount": discount.to_string(),
            }),
        )
        .await;

        tracing::info!(
            coupon_code = %code,
            discount = %discount,
            "Coupon applied"
        );

        Ok(discount)
    }

    /// Delete a coupon.
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        CouponRepository::delete(pool, id).await
    }
}
