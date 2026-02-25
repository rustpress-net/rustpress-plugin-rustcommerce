//! Coupon repository
//!
//! Database access for discount coupons.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{Result, RustCommerceError};
use crate::models::coupon::*;

/// Repository for coupon database operations.
pub struct CouponRepository;

impl CouponRepository {
    /// Find a coupon by ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Coupon> {
        sqlx::query_as::<_, Coupon>("SELECT * FROM rc_coupons WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(RustCommerceError::CouponNotFound(id.to_string()))
    }

    /// Find a coupon by code (case-insensitive).
    pub async fn find_by_code(pool: &PgPool, code: &str) -> Result<Coupon> {
        sqlx::query_as::<_, Coupon>("SELECT * FROM rc_coupons WHERE UPPER(code) = UPPER($1)")
            .bind(code)
            .fetch_optional(pool)
            .await?
            .ok_or(RustCommerceError::CouponNotFound(code.to_string()))
    }

    /// List all coupons with optional active-only filter.
    pub async fn list(pool: &PgPool, active_only: bool) -> Result<Vec<Coupon>> {
        let coupons = if active_only {
            sqlx::query_as::<_, Coupon>(
                "SELECT * FROM rc_coupons WHERE is_active = TRUE ORDER BY created_at DESC",
            )
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, Coupon>("SELECT * FROM rc_coupons ORDER BY created_at DESC")
                .fetch_all(pool)
                .await?
        };

        Ok(coupons)
    }

    /// Create a new coupon.
    pub async fn create(pool: &PgPool, req: &CreateCouponRequest) -> Result<Coupon> {
        // Check for duplicate code
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM rc_coupons WHERE UPPER(code) = UPPER($1)",
        )
        .bind(&req.code)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            return Err(RustCommerceError::DuplicateEntry(format!(
                "Coupon with code '{}' already exists",
                req.code
            )));
        }

        let coupon = sqlx::query_as::<_, Coupon>(
            r#"
            INSERT INTO rc_coupons (
                code, description, discount_type, discount_value,
                minimum_order_amount, maximum_discount_amount,
                usage_limit, usage_limit_per_customer,
                applies_to_product_ids, applies_to_category_ids,
                excluded_product_ids, excluded_category_ids,
                starts_at, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(&req.code)
        .bind(&req.description)
        .bind(&req.discount_type)
        .bind(&req.discount_value)
        .bind(&req.minimum_order_amount)
        .bind(&req.maximum_discount_amount)
        .bind(&req.usage_limit)
        .bind(&req.usage_limit_per_customer)
        .bind(&req.applies_to_product_ids)
        .bind(&req.applies_to_category_ids)
        .bind(&req.excluded_product_ids)
        .bind(&req.excluded_category_ids)
        .bind(&req.starts_at)
        .bind(&req.expires_at)
        .fetch_one(pool)
        .await?;

        Ok(coupon)
    }

    /// Update a coupon.
    pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateCouponRequest) -> Result<Coupon> {
        let existing = Self::find_by_id(pool, id).await?;

        let discount_type = req.discount_type.clone().unwrap_or(existing.discount_type);
        let discount_value = req.discount_value.unwrap_or(existing.discount_value);
        let min_order = req.minimum_order_amount.or(existing.minimum_order_amount);
        let max_discount = req
            .maximum_discount_amount
            .or(existing.maximum_discount_amount);
        let usage_limit = req.usage_limit.or(existing.usage_limit);
        let per_customer = req
            .usage_limit_per_customer
            .or(existing.usage_limit_per_customer);
        let is_active = req.is_active.unwrap_or(existing.is_active);
        let starts_at = req.starts_at.or(existing.starts_at);
        let expires_at = req.expires_at.or(existing.expires_at);

        let coupon = sqlx::query_as::<_, Coupon>(
            r#"
            UPDATE rc_coupons SET
                description = COALESCE($1, description),
                discount_type = $2, discount_value = $3,
                minimum_order_amount = $4, maximum_discount_amount = $5,
                usage_limit = $6, usage_limit_per_customer = $7,
                is_active = $8, starts_at = $9, expires_at = $10
            WHERE id = $11
            RETURNING *
            "#,
        )
        .bind(&req.description)
        .bind(&discount_type)
        .bind(discount_value)
        .bind(min_order)
        .bind(max_discount)
        .bind(usage_limit)
        .bind(per_customer)
        .bind(is_active)
        .bind(starts_at)
        .bind(expires_at)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(coupon)
    }

    /// Increment the usage count for a coupon.
    pub async fn increment_usage(pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE rc_coupons SET usage_count = usage_count + 1 WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Delete a coupon.
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM rc_coupons WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RustCommerceError::CouponNotFound(id.to_string()));
        }
        Ok(())
    }
}
