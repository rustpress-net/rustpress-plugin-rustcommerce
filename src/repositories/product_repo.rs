//! Product repository
//!
//! Database access layer for products, variants, and images.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{Result, RustCommerceError};
use crate::models::product::*;

/// Repository for product database operations.
pub struct ProductRepository;

impl ProductRepository {
    /// Fetch a single product by ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Product> {
        sqlx::query_as::<_, Product>("SELECT * FROM rc_products WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(RustCommerceError::ProductNotFound(id))
    }

    /// Fetch a single product by slug.
    pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Product> {
        sqlx::query_as::<_, Product>("SELECT * FROM rc_products WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| {
                RustCommerceError::InvalidInput(format!("Product with slug '{}' not found", slug))
            })
    }

    /// List products with filtering, sorting, and pagination.
    pub async fn list(pool: &PgPool, params: &ProductListParams) -> Result<(Vec<Product>, i64)> {
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(20).min(100);
        let offset = ((page - 1) * per_page) as i64;
        let sort_by = params
            .sort_by
            .as_deref()
            .unwrap_or("created_at");
        let sort_order = params
            .sort_order
            .as_deref()
            .unwrap_or("desc");

        // Validate sort_by to prevent SQL injection
        let sort_column = match sort_by {
            "name" => "name",
            "price" => "price",
            "created_at" => "created_at",
            "updated_at" => "updated_at",
            "stock_quantity" => "stock_quantity",
            _ => "created_at",
        };
        let order = if sort_order.eq_ignore_ascii_case("asc") {
            "ASC"
        } else {
            "DESC"
        };

        let query = format!(
            r#"
            SELECT * FROM rc_products
            WHERE ($1::rc_product_status IS NULL OR status = $1)
              AND ($2::text IS NULL OR name ILIKE '%' || $2 || '%' OR description ILIKE '%' || $2 || '%')
              AND ($3::numeric IS NULL OR price >= $3)
              AND ($4::numeric IS NULL OR price <= $4)
              AND ($5::boolean IS NULL OR is_featured = $5)
            ORDER BY {} {}
            LIMIT $6 OFFSET $7
            "#,
            sort_column, order
        );

        let products = sqlx::query_as::<_, Product>(&query)
            .bind(&params.status)
            .bind(&params.search)
            .bind(&params.min_price)
            .bind(&params.max_price)
            .bind(&params.is_featured)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        let count_row = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM rc_products
            WHERE ($1::rc_product_status IS NULL OR status = $1)
              AND ($2::text IS NULL OR name ILIKE '%' || $2 || '%' OR description ILIKE '%' || $2 || '%')
              AND ($3::numeric IS NULL OR price >= $3)
              AND ($4::numeric IS NULL OR price <= $4)
              AND ($5::boolean IS NULL OR is_featured = $5)
            "#,
        )
        .bind(&params.status)
        .bind(&params.search)
        .bind(&params.min_price)
        .bind(&params.max_price)
        .bind(&params.is_featured)
        .fetch_one(pool)
        .await?;

        Ok((products, count_row))
    }

    /// Create a new product.
    pub async fn create(pool: &PgPool, req: &CreateProductRequest) -> Result<Product> {
        let slug = req.slug.clone().unwrap_or_else(|| slug::slugify(&req.name));
        let status = req.status.clone().unwrap_or(ProductStatus::Draft);
        let low_stock = req.low_stock_threshold.unwrap_or(5);
        let track_inv = req.track_inventory.unwrap_or(true);

        let product = sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO rc_products (
                name, slug, description, short_description, sku,
                price, compare_at_price, cost_price, status,
                is_featured, is_digital, weight, width, height, depth,
                stock_quantity, low_stock_threshold, track_inventory, allow_backorders,
                meta_title, meta_description
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12, $13, $14, $15,
                $16, $17, $18, $19,
                $20, $21
            ) RETURNING *
            "#,
        )
        .bind(&req.name)
        .bind(&slug)
        .bind(&req.description)
        .bind(&req.short_description)
        .bind(&req.sku)
        .bind(&req.price)
        .bind(&req.compare_at_price)
        .bind(&req.cost_price)
        .bind(&status)
        .bind(req.is_featured)
        .bind(req.is_digital)
        .bind(&req.weight)
        .bind(&req.width)
        .bind(&req.height)
        .bind(&req.depth)
        .bind(req.stock_quantity)
        .bind(low_stock)
        .bind(track_inv)
        .bind(req.allow_backorders)
        .bind(&req.meta_title)
        .bind(&req.meta_description)
        .fetch_one(pool)
        .await?;

        // Assign categories if provided
        if let Some(ref cat_ids) = req.category_ids {
            for cat_id in cat_ids {
                sqlx::query(
                    "INSERT INTO rc_product_categories (product_id, category_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                )
                .bind(product.id)
                .bind(cat_id)
                .execute(pool)
                .await?;
            }
        }

        Ok(product)
    }

    /// Update an existing product.
    pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateProductRequest) -> Result<Product> {
        let existing = Self::find_by_id(pool, id).await?;

        let name = req.name.as_deref().unwrap_or(&existing.name);
        let slug_val = req
            .slug
            .clone()
            .unwrap_or_else(|| existing.slug.clone());
        let description = req.description.as_deref().or(existing.description.as_deref());
        let short_desc = req
            .short_description
            .as_deref()
            .or(existing.short_description.as_deref());
        let sku = req.sku.as_deref().or(existing.sku.as_deref());
        let price = req.price.unwrap_or(existing.price);
        let compare_at = req.compare_at_price.or(existing.compare_at_price);
        let cost = req.cost_price.or(existing.cost_price);
        let status = req.status.clone().unwrap_or(existing.status);
        let featured = req.is_featured.unwrap_or(existing.is_featured);
        let digital = req.is_digital.unwrap_or(existing.is_digital);
        let weight = req.weight.or(existing.weight);
        let width = req.width.or(existing.width);
        let height = req.height.or(existing.height);
        let depth = req.depth.or(existing.depth);
        let stock = req.stock_quantity.unwrap_or(existing.stock_quantity);
        let low_thresh = req.low_stock_threshold.unwrap_or(existing.low_stock_threshold);
        let track = req.track_inventory.unwrap_or(existing.track_inventory);
        let backorder = req.allow_backorders.unwrap_or(existing.allow_backorders);
        let meta_title = req.meta_title.as_deref().or(existing.meta_title.as_deref());
        let meta_desc = req
            .meta_description
            .as_deref()
            .or(existing.meta_description.as_deref());

        let product = sqlx::query_as::<_, Product>(
            r#"
            UPDATE rc_products SET
                name = $1, slug = $2, description = $3, short_description = $4, sku = $5,
                price = $6, compare_at_price = $7, cost_price = $8, status = $9,
                is_featured = $10, is_digital = $11, weight = $12, width = $13, height = $14, depth = $15,
                stock_quantity = $16, low_stock_threshold = $17, track_inventory = $18, allow_backorders = $19,
                meta_title = $20, meta_description = $21
            WHERE id = $22
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(&slug_val)
        .bind(description)
        .bind(short_desc)
        .bind(sku)
        .bind(price)
        .bind(compare_at)
        .bind(cost)
        .bind(&status)
        .bind(featured)
        .bind(digital)
        .bind(weight)
        .bind(width)
        .bind(height)
        .bind(depth)
        .bind(stock)
        .bind(low_thresh)
        .bind(track)
        .bind(backorder)
        .bind(meta_title)
        .bind(meta_desc)
        .bind(id)
        .fetch_one(pool)
        .await?;

        // Update categories if provided
        if let Some(ref cat_ids) = req.category_ids {
            sqlx::query("DELETE FROM rc_product_categories WHERE product_id = $1")
                .bind(id)
                .execute(pool)
                .await?;
            for cat_id in cat_ids {
                sqlx::query(
                    "INSERT INTO rc_product_categories (product_id, category_id) VALUES ($1, $2)",
                )
                .bind(id)
                .bind(cat_id)
                .execute(pool)
                .await?;
            }
        }

        Ok(product)
    }

    /// Delete a product by ID.
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM rc_products WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RustCommerceError::ProductNotFound(id));
        }
        Ok(())
    }

    /// Update stock quantity for a product.
    pub async fn update_stock(pool: &PgPool, id: Uuid, quantity_change: i32) -> Result<Product> {
        let product = sqlx::query_as::<_, Product>(
            r#"
            UPDATE rc_products
            SET stock_quantity = stock_quantity + $1
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(quantity_change)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(RustCommerceError::ProductNotFound(id))?;

        Ok(product)
    }

    /// List all variants for a product.
    pub async fn list_variants(pool: &PgPool, product_id: Uuid) -> Result<Vec<ProductVariant>> {
        let variants = sqlx::query_as::<_, ProductVariant>(
            "SELECT * FROM rc_product_variants WHERE product_id = $1 ORDER BY sort_order",
        )
        .bind(product_id)
        .fetch_all(pool)
        .await?;

        Ok(variants)
    }

    /// Create a product variant.
    pub async fn create_variant(
        pool: &PgPool,
        product_id: Uuid,
        req: &CreateVariantRequest,
    ) -> Result<ProductVariant> {
        let variant = sqlx::query_as::<_, ProductVariant>(
            r#"
            INSERT INTO rc_product_variants (
                product_id, name, sku, price, compare_at_price, cost_price,
                stock_quantity, weight,
                option1_name, option1_value, option2_name, option2_value, option3_name, option3_value
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(product_id)
        .bind(&req.name)
        .bind(&req.sku)
        .bind(&req.price)
        .bind(&req.compare_at_price)
        .bind(&req.cost_price)
        .bind(req.stock_quantity)
        .bind(&req.weight)
        .bind(&req.option1_name)
        .bind(&req.option1_value)
        .bind(&req.option2_name)
        .bind(&req.option2_value)
        .bind(&req.option3_name)
        .bind(&req.option3_value)
        .fetch_one(pool)
        .await?;

        Ok(variant)
    }

    /// List images for a product.
    pub async fn list_images(pool: &PgPool, product_id: Uuid) -> Result<Vec<ProductImage>> {
        let images = sqlx::query_as::<_, ProductImage>(
            "SELECT * FROM rc_product_images WHERE product_id = $1 ORDER BY sort_order",
        )
        .bind(product_id)
        .fetch_all(pool)
        .await?;

        Ok(images)
    }

    /// Find products with low stock.
    pub async fn find_low_stock(pool: &PgPool) -> Result<Vec<Product>> {
        let products = sqlx::query_as::<_, Product>(
            r#"
            SELECT * FROM rc_products
            WHERE track_inventory = TRUE
              AND stock_quantity <= low_stock_threshold
              AND status = 'active'
            ORDER BY stock_quantity ASC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(products)
    }
}
