//! Cart service
//!
//! Business logic for shopping cart operations including
//! item management, totals calculation, and coupon application.

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::error::{Result, RustCommerceError};
use crate::models::cart::*;
use crate::repositories::{CartRepository, CouponRepository, ProductRepository};

/// Service for cart business logic.
pub struct CartService;

impl CartService {
    /// Get or create a cart for the given parameters.
    pub async fn get_or_create(pool: &PgPool, req: &CreateCartRequest) -> Result<Cart> {
        // Try to find existing cart
        if let Some(customer_id) = req.customer_id {
            if let Some(cart) = CartRepository::find_by_customer(pool, customer_id).await? {
                return Ok(cart);
            }
        }

        if let Some(ref session_id) = req.session_id {
            if let Some(cart) = CartRepository::find_by_session(pool, session_id).await? {
                return Ok(cart);
            }
        }

        // Create new cart
        CartRepository::create(pool, req).await
    }

    /// Get a cart by ID with all items.
    pub async fn get_with_items(pool: &PgPool, cart_id: Uuid) -> Result<CartWithItems> {
        let cart = CartRepository::find_by_id(pool, cart_id).await?;
        let items = CartRepository::list_items(pool, cart_id).await?;

        // Enrich items with product details
        let mut item_details = Vec::new();
        for item in items {
            let product = ProductRepository::find_by_id(pool, item.product_id).await.ok();
            item_details.push(CartItemDetail {
                product_name: product.as_ref().map(|p| p.name.clone()),
                product_slug: product.as_ref().map(|p| p.slug.clone()),
                product_image_url: None, // Would need to fetch primary image
                variant_name: None,
                item,
            });
        }

        Ok(CartWithItems::from_cart_and_items(cart, item_details))
    }

    /// Add an item to a cart.
    pub async fn add_item(pool: &PgPool, cart_id: Uuid, req: &AddToCartRequest) -> Result<CartItem> {
        req.validate()
            .map_err(|e| RustCommerceError::Validation(e.to_string()))?;

        // Verify cart exists
        CartRepository::find_by_id(pool, cart_id).await?;

        // Verify product exists and get price
        let product = ProductRepository::find_by_id(pool, req.product_id).await?;

        // Check stock availability
        if product.track_inventory && !product.allow_backorders {
            if product.stock_quantity < req.quantity {
                return Err(RustCommerceError::InsufficientStock {
                    product_id: req.product_id,
                    requested: req.quantity,
                    available: product.stock_quantity,
                });
            }
        }

        let unit_price = product.price;
        let item =
            CartRepository::add_item(pool, cart_id, req.product_id, req.variant_id, req.quantity, unit_price)
                .await?;

        // Recalculate totals
        Self::recalculate_totals(pool, cart_id).await?;

        tracing::info!(
            cart_id = %cart_id,
            product_id = %req.product_id,
            quantity = req.quantity,
            "Item added to cart"
        );

        Ok(item)
    }

    /// Update item quantity in a cart.
    pub async fn update_item(
        pool: &PgPool,
        cart_id: Uuid,
        item_id: Uuid,
        req: &UpdateCartItemRequest,
    ) -> Result<CartItem> {
        req.validate()
            .map_err(|e| RustCommerceError::Validation(e.to_string()))?;

        // Verify cart exists
        CartRepository::find_by_id(pool, cart_id).await?;

        let item = CartRepository::update_item_quantity(pool, item_id, req.quantity).await?;

        // Recalculate totals
        Self::recalculate_totals(pool, cart_id).await?;

        Ok(item)
    }

    /// Remove an item from a cart.
    pub async fn remove_item(pool: &PgPool, cart_id: Uuid, item_id: Uuid) -> Result<()> {
        CartRepository::find_by_id(pool, cart_id).await?;
        CartRepository::remove_item(pool, item_id).await?;
        Self::recalculate_totals(pool, cart_id).await?;
        Ok(())
    }

    /// Clear all items from a cart.
    pub async fn clear(pool: &PgPool, cart_id: Uuid) -> Result<()> {
        CartRepository::find_by_id(pool, cart_id).await?;
        CartRepository::clear_items(pool, cart_id).await?;
        CartRepository::update_totals(
            pool,
            cart_id,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::ZERO,
        )
        .await?;
        Ok(())
    }

    /// Apply a coupon code to a cart.
    pub async fn apply_coupon(pool: &PgPool, cart_id: Uuid, code: &str) -> Result<Cart> {
        let cart = CartRepository::find_by_id(pool, cart_id).await?;

        // Find and validate coupon
        let coupon = CouponRepository::find_by_code(pool, code).await?;
        coupon.validate_usage(cart.subtotal)?;

        // Apply coupon
        CartRepository::apply_coupon(pool, cart_id, &coupon.code).await?;

        // Recalculate totals with discount
        Self::recalculate_totals(pool, cart_id).await
    }

    /// Remove coupon from a cart.
    pub async fn remove_coupon(pool: &PgPool, cart_id: Uuid) -> Result<Cart> {
        CartRepository::remove_coupon(pool, cart_id).await?;
        Self::recalculate_totals(pool, cart_id).await
    }

    /// Recalculate cart subtotal, discount, and grand total.
    async fn recalculate_totals(pool: &PgPool, cart_id: Uuid) -> Result<Cart> {
        let items = CartRepository::list_items(pool, cart_id).await?;
        let subtotal: Decimal = items.iter().map(|i| i.total_price).sum();

        let cart = CartRepository::find_by_id(pool, cart_id).await?;

        // Calculate discount if coupon is applied
        let discount = if let Some(ref code) = cart.coupon_code {
            if let Ok(coupon) = CouponRepository::find_by_code(pool, code).await {
                coupon.calculate_discount(subtotal)
            } else {
                Decimal::ZERO
            }
        } else {
            Decimal::ZERO
        };

        // Tax and shipping will be calculated at checkout;
        // keep existing values or zero for now
        CartRepository::update_totals(
            pool,
            cart_id,
            subtotal,
            cart.tax_total,
            cart.shipping_total,
            discount,
        )
        .await
    }

    /// Delete expired carts (for scheduled cleanup).
    pub async fn cleanup_expired(pool: &PgPool) -> Result<u64> {
        let count = CartRepository::delete_expired(pool).await?;
        if count > 0 {
            tracing::info!(count = count, "Cleaned up expired carts");
        }
        Ok(count)
    }
}
