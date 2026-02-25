//! Cart repository
//!
//! Database access for shopping carts and cart items.

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{Result, RustCommerceError};
use crate::models::cart::*;

/// Repository for cart database operations.
pub struct CartRepository;

impl CartRepository {
    /// Fetch a cart by ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Cart> {
        sqlx::query_as::<_, Cart>("SELECT * FROM rc_carts WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(RustCommerceError::CartNotFound(id))
    }

    /// Find a cart by customer ID.
    pub async fn find_by_customer(pool: &PgPool, customer_id: Uuid) -> Result<Option<Cart>> {
        let cart = sqlx::query_as::<_, Cart>(
            "SELECT * FROM rc_carts WHERE customer_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(customer_id)
        .fetch_optional(pool)
        .await?;

        Ok(cart)
    }

    /// Find a cart by session ID.
    pub async fn find_by_session(pool: &PgPool, session_id: &str) -> Result<Option<Cart>> {
        let cart = sqlx::query_as::<_, Cart>(
            "SELECT * FROM rc_carts WHERE session_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await?;

        Ok(cart)
    }

    /// Create a new cart.
    pub async fn create(pool: &PgPool, req: &CreateCartRequest) -> Result<Cart> {
        let currency = req.currency.as_deref().unwrap_or("USD");

        let cart = sqlx::query_as::<_, Cart>(
            r#"
            INSERT INTO rc_carts (customer_id, session_id, currency)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(&req.customer_id)
        .bind(&req.session_id)
        .bind(currency)
        .fetch_one(pool)
        .await?;

        Ok(cart)
    }

    /// Add an item to a cart.
    pub async fn add_item(
        pool: &PgPool,
        cart_id: Uuid,
        product_id: Uuid,
        variant_id: Option<Uuid>,
        quantity: i32,
        unit_price: Decimal,
    ) -> Result<CartItem> {
        let total_price = unit_price * Decimal::from(quantity);

        let item = sqlx::query_as::<_, CartItem>(
            r#"
            INSERT INTO rc_cart_items (cart_id, product_id, variant_id, quantity, unit_price, total_price)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (cart_id, product_id, variant_id) DO UPDATE SET
                quantity = rc_cart_items.quantity + $4,
                total_price = rc_cart_items.unit_price * (rc_cart_items.quantity + $4)
            RETURNING *
            "#,
        )
        .bind(cart_id)
        .bind(product_id)
        .bind(variant_id)
        .bind(quantity)
        .bind(unit_price)
        .bind(total_price)
        .fetch_one(pool)
        .await?;

        Ok(item)
    }

    /// Update the quantity of a cart item.
    pub async fn update_item_quantity(
        pool: &PgPool,
        item_id: Uuid,
        quantity: i32,
    ) -> Result<CartItem> {
        let item = sqlx::query_as::<_, CartItem>(
            r#"
            UPDATE rc_cart_items
            SET quantity = $1, total_price = unit_price * $1
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(quantity)
        .bind(item_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            RustCommerceError::InvalidInput(format!("Cart item {} not found", item_id))
        })?;

        Ok(item)
    }

    /// Remove a specific item from a cart.
    pub async fn remove_item(pool: &PgPool, item_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM rc_cart_items WHERE id = $1")
            .bind(item_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RustCommerceError::InvalidInput(format!(
                "Cart item {} not found",
                item_id
            )));
        }
        Ok(())
    }

    /// List all items in a cart.
    pub async fn list_items(pool: &PgPool, cart_id: Uuid) -> Result<Vec<CartItem>> {
        let items = sqlx::query_as::<_, CartItem>(
            "SELECT * FROM rc_cart_items WHERE cart_id = $1 ORDER BY created_at",
        )
        .bind(cart_id)
        .fetch_all(pool)
        .await?;

        Ok(items)
    }

    /// Clear all items from a cart.
    pub async fn clear_items(pool: &PgPool, cart_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM rc_cart_items WHERE cart_id = $1")
            .bind(cart_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Update cart totals.
    pub async fn update_totals(
        pool: &PgPool,
        cart_id: Uuid,
        subtotal: Decimal,
        tax_total: Decimal,
        shipping_total: Decimal,
        discount_total: Decimal,
    ) -> Result<Cart> {
        let grand_total =
            (subtotal + tax_total + shipping_total - discount_total).max(Decimal::ZERO);

        let cart = sqlx::query_as::<_, Cart>(
            r#"
            UPDATE rc_carts SET
                subtotal = $1, tax_total = $2, shipping_total = $3,
                discount_total = $4, grand_total = $5
            WHERE id = $6
            RETURNING *
            "#,
        )
        .bind(subtotal)
        .bind(tax_total)
        .bind(shipping_total)
        .bind(discount_total)
        .bind(grand_total)
        .bind(cart_id)
        .fetch_one(pool)
        .await?;

        Ok(cart)
    }

    /// Apply a coupon code to a cart.
    pub async fn apply_coupon(pool: &PgPool, cart_id: Uuid, coupon_code: &str) -> Result<Cart> {
        let cart = sqlx::query_as::<_, Cart>(
            "UPDATE rc_carts SET coupon_code = $1 WHERE id = $2 RETURNING *",
        )
        .bind(coupon_code)
        .bind(cart_id)
        .fetch_one(pool)
        .await?;

        Ok(cart)
    }

    /// Remove coupon from a cart.
    pub async fn remove_coupon(pool: &PgPool, cart_id: Uuid) -> Result<Cart> {
        let cart = sqlx::query_as::<_, Cart>(
            "UPDATE rc_carts SET coupon_code = NULL, discount_total = 0 WHERE id = $1 RETURNING *",
        )
        .bind(cart_id)
        .fetch_one(pool)
        .await?;

        Ok(cart)
    }

    /// Delete a cart and all its items (cascade).
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM rc_carts WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RustCommerceError::CartNotFound(id));
        }
        Ok(())
    }

    /// Delete expired carts.
    pub async fn delete_expired(pool: &PgPool) -> Result<u64> {
        let result = sqlx::query("DELETE FROM rc_carts WHERE expires_at IS NOT NULL AND expires_at < NOW()")
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }
}
