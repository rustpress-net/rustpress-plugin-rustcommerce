//! Product service
//!
//! Business logic for product management including slug generation,
//! validation, and hook firing.

use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::error::{Result, RustCommerceError};
use crate::hooks::HookRegistry;
use crate::models::product::*;
use crate::repositories::ProductRepository;

/// Service for product business logic.
pub struct ProductService;

impl ProductService {
    /// List products with filtering and pagination.
    pub async fn list(
        pool: &PgPool,
        params: &ProductListParams,
    ) -> Result<PaginatedResponse<Product>> {
        let (products, total) = ProductRepository::list(pool, params).await?;
        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);
        Ok(PaginatedResponse::new(products, total, page, per_page))
    }

    /// Get a single product by ID.
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Product> {
        ProductRepository::find_by_id(pool, id).await
    }

    /// Get a single product by slug.
    pub async fn get_by_slug(pool: &PgPool, slug: &str) -> Result<Product> {
        ProductRepository::find_by_slug(pool, slug).await
    }

    /// Create a new product with validation.
    pub async fn create(pool: &PgPool, req: &CreateProductRequest) -> Result<Product> {
        req.validate()
            .map_err(|e| RustCommerceError::Validation(e.to_string()))?;

        if req.price < rust_decimal::Decimal::ZERO {
            return Err(RustCommerceError::Validation(
                "Price must be non-negative".to_string(),
            ));
        }

        let product = ProductRepository::create(pool, req).await?;

        tracing::info!(
            product_id = %product.id,
            product_name = %product.name,
            "Product created"
        );

        HookRegistry::fire_action("product_created", &serde_json::to_value(&product)?).await;

        Ok(product)
    }

    /// Update an existing product.
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateProductRequest,
    ) -> Result<Product> {
        req.validate()
            .map_err(|e| RustCommerceError::Validation(e.to_string()))?;

        if let Some(price) = req.price {
            if price < rust_decimal::Decimal::ZERO {
                return Err(RustCommerceError::Validation(
                    "Price must be non-negative".to_string(),
                ));
            }
        }

        let product = ProductRepository::update(pool, id, req).await?;

        tracing::info!(product_id = %id, "Product updated");
        HookRegistry::fire_action("product_updated", &serde_json::to_value(&product)?).await;

        Ok(product)
    }

    /// Delete a product.
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        ProductRepository::delete(pool, id).await?;

        tracing::info!(product_id = %id, "Product deleted");
        HookRegistry::fire_action(
            "product_deleted",
            &serde_json::json!({ "product_id": id }),
        )
        .await;

        Ok(())
    }

    /// List variants for a product.
    pub async fn list_variants(pool: &PgPool, product_id: Uuid) -> Result<Vec<ProductVariant>> {
        // Verify product exists
        ProductRepository::find_by_id(pool, product_id).await?;
        ProductRepository::list_variants(pool, product_id).await
    }

    /// Create a variant for a product.
    pub async fn create_variant(
        pool: &PgPool,
        product_id: Uuid,
        req: &CreateVariantRequest,
    ) -> Result<ProductVariant> {
        req.validate()
            .map_err(|e| RustCommerceError::Validation(e.to_string()))?;

        // Verify product exists
        ProductRepository::find_by_id(pool, product_id).await?;

        let variant = ProductRepository::create_variant(pool, product_id, req).await?;

        tracing::info!(
            product_id = %product_id,
            variant_id = %variant.id,
            "Product variant created"
        );

        Ok(variant)
    }

    /// List images for a product.
    pub async fn list_images(pool: &PgPool, product_id: Uuid) -> Result<Vec<ProductImage>> {
        ProductRepository::find_by_id(pool, product_id).await?;
        ProductRepository::list_images(pool, product_id).await
    }
}
