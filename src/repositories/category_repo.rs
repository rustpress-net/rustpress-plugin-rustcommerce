//! Category repository
//!
//! Database access layer for product categories.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{Result, RustCommerceError};
use crate::models::category::*;

/// Repository for category database operations.
pub struct CategoryRepository;

impl CategoryRepository {
    /// Fetch a category by ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Category> {
        sqlx::query_as::<_, Category>("SELECT * FROM rc_categories WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(RustCommerceError::CategoryNotFound(id))
    }

    /// Fetch a category by slug.
    pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Category> {
        sqlx::query_as::<_, Category>("SELECT * FROM rc_categories WHERE slug = $1")
            .bind(slug)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| {
                RustCommerceError::InvalidInput(format!(
                    "Category with slug '{}' not found",
                    slug
                ))
            })
    }

    /// List all categories, ordered by sort_order.
    pub async fn list_all(pool: &PgPool) -> Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>(
            "SELECT * FROM rc_categories ORDER BY sort_order, name",
        )
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }

    /// List only active categories.
    pub async fn list_active(pool: &PgPool) -> Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>(
            "SELECT * FROM rc_categories WHERE is_active = TRUE ORDER BY sort_order, name",
        )
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }

    /// List children of a given category.
    pub async fn list_children(pool: &PgPool, parent_id: Uuid) -> Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>(
            "SELECT * FROM rc_categories WHERE parent_id = $1 ORDER BY sort_order, name",
        )
        .bind(parent_id)
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }

    /// Create a new category.
    pub async fn create(pool: &PgPool, req: &CreateCategoryRequest) -> Result<Category> {
        let slug = req.slug.clone().unwrap_or_else(|| slug::slugify(&req.name));
        let sort_order = req.sort_order.unwrap_or(0);

        let category = sqlx::query_as::<_, Category>(
            r#"
            INSERT INTO rc_categories (name, slug, description, parent_id, image_url, sort_order, meta_title, meta_description)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(&req.name)
        .bind(&slug)
        .bind(&req.description)
        .bind(&req.parent_id)
        .bind(&req.image_url)
        .bind(sort_order)
        .bind(&req.meta_title)
        .bind(&req.meta_description)
        .fetch_one(pool)
        .await?;

        Ok(category)
    }

    /// Update a category.
    pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateCategoryRequest) -> Result<Category> {
        let existing = Self::find_by_id(pool, id).await?;

        let name = req.name.as_deref().unwrap_or(&existing.name);
        let slug_val = req.slug.clone().unwrap_or(existing.slug);
        let description = req.description.as_deref().or(existing.description.as_deref());
        let parent_id = req.parent_id.or(existing.parent_id);
        let image_url = req.image_url.as_deref().or(existing.image_url.as_deref());
        let sort_order = req.sort_order.unwrap_or(existing.sort_order);
        let is_active = req.is_active.unwrap_or(existing.is_active);
        let meta_title = req.meta_title.as_deref().or(existing.meta_title.as_deref());
        let meta_desc = req
            .meta_description
            .as_deref()
            .or(existing.meta_description.as_deref());

        let category = sqlx::query_as::<_, Category>(
            r#"
            UPDATE rc_categories SET
                name = $1, slug = $2, description = $3, parent_id = $4,
                image_url = $5, sort_order = $6, is_active = $7,
                meta_title = $8, meta_description = $9
            WHERE id = $10
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(&slug_val)
        .bind(description)
        .bind(parent_id)
        .bind(image_url)
        .bind(sort_order)
        .bind(is_active)
        .bind(meta_title)
        .bind(meta_desc)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(category)
    }

    /// Delete a category by ID.
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM rc_categories WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RustCommerceError::CategoryNotFound(id));
        }
        Ok(())
    }

    /// Count products in a category.
    pub async fn count_products(pool: &PgPool, category_id: Uuid) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM rc_product_categories WHERE category_id = $1",
        )
        .bind(category_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }
}
