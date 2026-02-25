//! Review models
//!
//! Product reviews and ratings with moderation workflow.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Review moderation status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "rc_review_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
    Spam,
}

impl std::fmt::Display for ReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Approved => write!(f, "approved"),
            Self::Rejected => write!(f, "rejected"),
            Self::Spam => write!(f, "spam"),
        }
    }
}

/// A product review submitted by a customer.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Review {
    pub id: Uuid,
    pub product_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub reviewer_name: String,
    pub reviewer_email: String,
    pub rating: i32,
    pub title: Option<String>,
    pub content: Option<String>,
    pub status: ReviewStatus,
    pub is_verified_purchase: bool,
    pub helpful_count: i32,
    pub admin_response: Option<String>,
    pub admin_responded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to submit a new review.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateReviewRequest {
    pub product_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    #[validate(length(min = 1, max = 255))]
    pub reviewer_name: String,
    #[validate(email(message = "Valid email required"))]
    pub reviewer_email: String,
    #[validate(range(min = 1, max = 5, message = "Rating must be between 1 and 5"))]
    pub rating: i32,
    #[validate(length(max = 255))]
    pub title: Option<String>,
    pub content: Option<String>,
}

/// Request to moderate a review (admin action).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerateReviewRequest {
    pub status: ReviewStatus,
    pub admin_response: Option<String>,
}

/// Query parameters for listing reviews.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub product_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub status: Option<ReviewStatus>,
    pub min_rating: Option<i32>,
    pub max_rating: Option<i32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Aggregate rating statistics for a product.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRatingStats {
    pub product_id: Uuid,
    pub average_rating: f64,
    pub total_reviews: i64,
    pub rating_distribution: RatingDistribution,
}

/// How many reviews at each star level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingDistribution {
    pub one_star: i64,
    pub two_star: i64,
    pub three_star: i64,
    pub four_star: i64,
    pub five_star: i64,
}
