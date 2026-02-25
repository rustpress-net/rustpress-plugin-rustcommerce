//! Review service
//!
//! Business logic for product reviews, ratings, and moderation.

use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::error::{Result, RustCommerceError};
use crate::models::product::PaginatedResponse;
use crate::models::review::*;
use crate::repositories::ReviewRepository;

/// Service for review business logic.
pub struct ReviewService;

impl ReviewService {
    /// Submit a new review.
    pub async fn submit(pool: &PgPool, req: &CreateReviewRequest) -> Result<Review> {
        req.validate()
            .map_err(|e| RustCommerceError::Validation(e.to_string()))?;

        // Ensure the product exists
        crate::repositories::ProductRepository::find_by_id(pool, req.product_id).await?;

        let review = ReviewRepository::create(pool, req).await?;

        tracing::info!(
            review_id = %review.id,
            product_id = %review.product_id,
            rating = review.rating,
            "Review submitted"
        );

        Ok(review)
    }

    /// List reviews with filtering and pagination.
    pub async fn list(
        pool: &PgPool,
        params: &ReviewListParams,
    ) -> Result<PaginatedResponse<Review>> {
        let (reviews, total) = ReviewRepository::list(pool, params).await?;
        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);
        Ok(PaginatedResponse::new(reviews, total, page, per_page))
    }

    /// Get a review by ID.
    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Review> {
        ReviewRepository::find_by_id(pool, id).await
    }

    /// Moderate a review (admin action).
    pub async fn moderate(
        pool: &PgPool,
        id: Uuid,
        req: &ModerateReviewRequest,
    ) -> Result<Review> {
        let review = ReviewRepository::moderate(pool, id, req).await?;

        tracing::info!(
            review_id = %id,
            new_status = %review.status,
            "Review moderated"
        );

        Ok(review)
    }

    /// Get rating statistics for a product.
    pub async fn get_product_stats(pool: &PgPool, product_id: Uuid) -> Result<ProductRatingStats> {
        ReviewRepository::get_product_stats(pool, product_id).await
    }

    /// Mark a review as helpful.
    pub async fn mark_helpful(pool: &PgPool, id: Uuid) -> Result<Review> {
        ReviewRepository::mark_helpful(pool, id).await
    }

    /// Delete a review.
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        ReviewRepository::delete(pool, id).await
    }
}
