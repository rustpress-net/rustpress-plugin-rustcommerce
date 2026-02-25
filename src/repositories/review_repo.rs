//! Review repository
//!
//! Database access for product reviews and ratings.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{Result, RustCommerceError};
use crate::models::review::*;

/// Repository for review database operations.
pub struct ReviewRepository;

impl ReviewRepository {
    /// Find a review by ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Review> {
        sqlx::query_as::<_, Review>("SELECT * FROM rc_reviews WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(RustCommerceError::ReviewNotFound(id))
    }

    /// List reviews with filtering and pagination.
    pub async fn list(pool: &PgPool, params: &ReviewListParams) -> Result<(Vec<Review>, i64)> {
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(20).min(100);
        let offset = ((page - 1) * per_page) as i64;

        let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
        let sort_order = params.sort_order.as_deref().unwrap_or("desc");

        let sort_column = match sort_by {
            "rating" => "rating",
            "helpful_count" => "helpful_count",
            "created_at" => "created_at",
            _ => "created_at",
        };
        let order = if sort_order.eq_ignore_ascii_case("asc") {
            "ASC"
        } else {
            "DESC"
        };

        let query = format!(
            r#"
            SELECT * FROM rc_reviews
            WHERE ($1::uuid IS NULL OR product_id = $1)
              AND ($2::uuid IS NULL OR customer_id = $2)
              AND ($3::rc_review_status IS NULL OR status = $3)
              AND ($4::integer IS NULL OR rating >= $4)
              AND ($5::integer IS NULL OR rating <= $5)
            ORDER BY {} {}
            LIMIT $6 OFFSET $7
            "#,
            sort_column, order
        );

        let reviews = sqlx::query_as::<_, Review>(&query)
            .bind(&params.product_id)
            .bind(&params.customer_id)
            .bind(&params.status)
            .bind(&params.min_rating)
            .bind(&params.max_rating)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM rc_reviews
            WHERE ($1::uuid IS NULL OR product_id = $1)
              AND ($2::uuid IS NULL OR customer_id = $2)
              AND ($3::rc_review_status IS NULL OR status = $3)
              AND ($4::integer IS NULL OR rating >= $4)
              AND ($5::integer IS NULL OR rating <= $5)
            "#,
        )
        .bind(&params.product_id)
        .bind(&params.customer_id)
        .bind(&params.status)
        .bind(&params.min_rating)
        .bind(&params.max_rating)
        .fetch_one(pool)
        .await?;

        Ok((reviews, total))
    }

    /// Create a new review.
    pub async fn create(pool: &PgPool, req: &CreateReviewRequest) -> Result<Review> {
        let review = sqlx::query_as::<_, Review>(
            r#"
            INSERT INTO rc_reviews (
                product_id, customer_id, order_id, reviewer_name, reviewer_email,
                rating, title, content, status, is_verified_purchase
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending', $9)
            RETURNING *
            "#,
        )
        .bind(req.product_id)
        .bind(&req.customer_id)
        .bind(&req.order_id)
        .bind(&req.reviewer_name)
        .bind(&req.reviewer_email)
        .bind(req.rating)
        .bind(&req.title)
        .bind(&req.content)
        .bind(req.order_id.is_some()) // is_verified_purchase if order_id provided
        .fetch_one(pool)
        .await?;

        Ok(review)
    }

    /// Moderate a review (approve, reject, mark as spam).
    pub async fn moderate(
        pool: &PgPool,
        id: Uuid,
        req: &ModerateReviewRequest,
    ) -> Result<Review> {
        let review = if req.admin_response.is_some() {
            sqlx::query_as::<_, Review>(
                r#"
                UPDATE rc_reviews SET status = $1, admin_response = $2, admin_responded_at = NOW()
                WHERE id = $3 RETURNING *
                "#,
            )
            .bind(&req.status)
            .bind(&req.admin_response)
            .bind(id)
            .fetch_optional(pool)
            .await?
        } else {
            sqlx::query_as::<_, Review>(
                "UPDATE rc_reviews SET status = $1 WHERE id = $2 RETURNING *",
            )
            .bind(&req.status)
            .bind(id)
            .fetch_optional(pool)
            .await?
        };

        review.ok_or(RustCommerceError::ReviewNotFound(id))
    }

    /// Get rating statistics for a product.
    pub async fn get_product_stats(pool: &PgPool, product_id: Uuid) -> Result<ProductRatingStats> {
        let avg_row = sqlx::query_as::<_, (Option<f64>, i64)>(
            r#"
            SELECT AVG(rating::float8), COUNT(*)
            FROM rc_reviews
            WHERE product_id = $1 AND status = 'approved'
            "#,
        )
        .bind(product_id)
        .fetch_one(pool)
        .await?;

        let distribution = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE rating = 1),
                COUNT(*) FILTER (WHERE rating = 2),
                COUNT(*) FILTER (WHERE rating = 3),
                COUNT(*) FILTER (WHERE rating = 4),
                COUNT(*) FILTER (WHERE rating = 5)
            FROM rc_reviews
            WHERE product_id = $1 AND status = 'approved'
            "#,
        )
        .bind(product_id)
        .fetch_one(pool)
        .await?;

        Ok(ProductRatingStats {
            product_id,
            average_rating: avg_row.0.unwrap_or(0.0),
            total_reviews: avg_row.1,
            rating_distribution: RatingDistribution {
                one_star: distribution.0,
                two_star: distribution.1,
                three_star: distribution.2,
                four_star: distribution.3,
                five_star: distribution.4,
            },
        })
    }

    /// Increment the helpful count for a review.
    pub async fn mark_helpful(pool: &PgPool, id: Uuid) -> Result<Review> {
        let review = sqlx::query_as::<_, Review>(
            "UPDATE rc_reviews SET helpful_count = helpful_count + 1 WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(RustCommerceError::ReviewNotFound(id))?;

        Ok(review)
    }

    /// Delete a review.
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM rc_reviews WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RustCommerceError::ReviewNotFound(id));
        }
        Ok(())
    }
}
