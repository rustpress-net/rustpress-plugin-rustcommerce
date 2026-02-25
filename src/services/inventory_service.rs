//! Inventory service
//!
//! Stock tracking, alerts, and inventory management.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Result;
use crate::hooks::HookRegistry;
use crate::models::product::Product;
use crate::repositories::ProductRepository;

/// Service for inventory management.
pub struct InventoryService;

impl InventoryService {
    /// Adjust stock for a product (positive to add, negative to deduct).
    pub async fn adjust_stock(
        pool: &PgPool,
        product_id: Uuid,
        quantity_change: i32,
        reason: &str,
    ) -> Result<Product> {
        let product = ProductRepository::update_stock(pool, product_id, quantity_change).await?;

        tracing::info!(
            product_id = %product_id,
            change = quantity_change,
            new_quantity = product.stock_quantity,
            reason = reason,
            "Stock adjusted"
        );

        // Check low stock threshold
        if product.track_inventory
            && product.stock_quantity <= product.low_stock_threshold
            && product.stock_quantity > 0
        {
            tracing::warn!(
                product_id = %product_id,
                stock = product.stock_quantity,
                threshold = product.low_stock_threshold,
                "Low stock alert"
            );

            HookRegistry::fire_action(
                "stock_low",
                &serde_json::json!({
                    "product_id": product.id,
                    "product_name": product.name,
                    "stock_quantity": product.stock_quantity,
                    "threshold": product.low_stock_threshold,
                }),
            )
            .await;
        }

        Ok(product)
    }

    /// Get all products with low stock levels.
    pub async fn get_low_stock_products(pool: &PgPool) -> Result<Vec<Product>> {
        ProductRepository::find_low_stock(pool).await
    }

    /// Set an absolute stock quantity for a product.
    pub async fn set_stock(
        pool: &PgPool,
        product_id: Uuid,
        quantity: i32,
    ) -> Result<Product> {
        let current = ProductRepository::find_by_id(pool, product_id).await?;
        let change = quantity - current.stock_quantity;
        Self::adjust_stock(pool, product_id, change, "manual_set").await
    }

    /// Reserve stock for a cart (deducts from available quantity).
    pub async fn reserve_stock(pool: &PgPool, product_id: Uuid, quantity: i32) -> Result<Product> {
        let product = ProductRepository::find_by_id(pool, product_id).await?;

        if product.track_inventory
            && !product.allow_backorders
            && product.stock_quantity < quantity
        {
            return Err(crate::error::RustCommerceError::InsufficientStock {
                product_id,
                requested: quantity,
                available: product.stock_quantity,
            });
        }

        Self::adjust_stock(pool, product_id, -quantity, "stock_reservation").await
    }

    /// Release previously reserved stock (e.g., when cart expires).
    pub async fn release_stock(pool: &PgPool, product_id: Uuid, quantity: i32) -> Result<Product> {
        Self::adjust_stock(pool, product_id, quantity, "stock_release").await
    }
}
