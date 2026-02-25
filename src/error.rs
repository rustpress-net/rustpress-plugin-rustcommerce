//! RustCommerce error types
//!
//! Centralized error handling using thiserror for derivation
//! and IntoResponse implementation for Axum compatibility.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// Primary error type for the RustCommerce plugin.
#[derive(Debug, thiserror::Error)]
pub enum RustCommerceError {
    // ── Database Errors ────────────────────────────────────────────────
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Database migration error: {0}")]
    Migration(String),

    // ── Not Found ──────────────────────────────────────────────────────
    #[error("Product not found: {0}")]
    ProductNotFound(uuid::Uuid),

    #[error("Category not found: {0}")]
    CategoryNotFound(uuid::Uuid),

    #[error("Cart not found: {0}")]
    CartNotFound(uuid::Uuid),

    #[error("Order not found: {0}")]
    OrderNotFound(uuid::Uuid),

    #[error("Customer not found: {0}")]
    CustomerNotFound(uuid::Uuid),

    #[error("Payment not found: {0}")]
    PaymentNotFound(uuid::Uuid),

    #[error("Coupon not found: {0}")]
    CouponNotFound(String),

    #[error("Review not found: {0}")]
    ReviewNotFound(uuid::Uuid),

    // ── Validation Errors ──────────────────────────────────────────────
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    // ── Business Logic Errors ──────────────────────────────────────────
    #[error("Insufficient stock for product {product_id}: requested {requested}, available {available}")]
    InsufficientStock {
        product_id: uuid::Uuid,
        requested: i32,
        available: i32,
    },

    #[error("Cart is empty")]
    EmptyCart,

    #[error("Invalid order status transition from {from} to {to}")]
    InvalidStatusTransition { from: String, to: String },

    #[error("Coupon is invalid: {0}")]
    InvalidCoupon(String),

    #[error("Coupon has expired")]
    CouponExpired,

    #[error("Coupon usage limit reached")]
    CouponUsageLimitReached,

    #[error("Minimum order amount not met: required {required}, current {current}")]
    MinimumOrderNotMet {
        required: rust_decimal::Decimal,
        current: rust_decimal::Decimal,
    },

    // ── Payment Errors ─────────────────────────────────────────────────
    #[error("Payment processing error: {0}")]
    PaymentProcessing(String),

    #[error("Payment gateway error: {0}")]
    PaymentGateway(String),

    #[error("Refund error: {0}")]
    RefundError(String),

    // ── Auth Errors ────────────────────────────────────────────────────
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    // ── External Service Errors ────────────────────────────────────────
    #[error("Stripe error: {0}")]
    Stripe(String),

    // ── General Errors ─────────────────────────────────────────────────
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl RustCommerceError {
    /// Returns the appropriate HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            // 400 Bad Request
            Self::Validation(_)
            | Self::InvalidInput(_)
            | Self::EmptyCart
            | Self::InvalidCoupon(_)
            | Self::CouponExpired
            | Self::CouponUsageLimitReached
            | Self::MinimumOrderNotMet { .. }
            | Self::InvalidStatusTransition { .. } => StatusCode::BAD_REQUEST,

            // 401 Unauthorized
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,

            // 403 Forbidden
            Self::Forbidden(_) => StatusCode::FORBIDDEN,

            // 404 Not Found
            Self::ProductNotFound(_)
            | Self::CategoryNotFound(_)
            | Self::CartNotFound(_)
            | Self::OrderNotFound(_)
            | Self::CustomerNotFound(_)
            | Self::PaymentNotFound(_)
            | Self::CouponNotFound(_)
            | Self::ReviewNotFound(_) => StatusCode::NOT_FOUND,

            // 409 Conflict
            Self::DuplicateEntry(_) => StatusCode::CONFLICT,

            // 422 Unprocessable Entity
            Self::InsufficientStock { .. } => StatusCode::UNPROCESSABLE_ENTITY,

            // 502 Bad Gateway
            Self::PaymentGateway(_) | Self::Stripe(_) => StatusCode::BAD_GATEWAY,

            // 500 Internal Server Error
            Self::Database(_)
            | Self::Migration(_)
            | Self::PaymentProcessing(_)
            | Self::RefundError(_)
            | Self::Configuration(_)
            | Self::Internal(_)
            | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Returns a machine-readable error code string.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Database(_) => "DATABASE_ERROR",
            Self::Migration(_) => "MIGRATION_ERROR",
            Self::ProductNotFound(_) => "PRODUCT_NOT_FOUND",
            Self::CategoryNotFound(_) => "CATEGORY_NOT_FOUND",
            Self::CartNotFound(_) => "CART_NOT_FOUND",
            Self::OrderNotFound(_) => "ORDER_NOT_FOUND",
            Self::CustomerNotFound(_) => "CUSTOMER_NOT_FOUND",
            Self::PaymentNotFound(_) => "PAYMENT_NOT_FOUND",
            Self::CouponNotFound(_) => "COUPON_NOT_FOUND",
            Self::ReviewNotFound(_) => "REVIEW_NOT_FOUND",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::DuplicateEntry(_) => "DUPLICATE_ENTRY",
            Self::InsufficientStock { .. } => "INSUFFICIENT_STOCK",
            Self::EmptyCart => "EMPTY_CART",
            Self::InvalidStatusTransition { .. } => "INVALID_STATUS_TRANSITION",
            Self::InvalidCoupon(_) => "INVALID_COUPON",
            Self::CouponExpired => "COUPON_EXPIRED",
            Self::CouponUsageLimitReached => "COUPON_USAGE_LIMIT",
            Self::MinimumOrderNotMet { .. } => "MINIMUM_ORDER_NOT_MET",
            Self::PaymentProcessing(_) => "PAYMENT_PROCESSING_ERROR",
            Self::PaymentGateway(_) => "PAYMENT_GATEWAY_ERROR",
            Self::RefundError(_) => "REFUND_ERROR",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::Stripe(_) => "STRIPE_ERROR",
            Self::Configuration(_) => "CONFIGURATION_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Anyhow(_) => "INTERNAL_ERROR",
        }
    }
}

impl IntoResponse for RustCommerceError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = json!({
            "error": {
                "code": self.error_code(),
                "message": self.to_string(),
                "status": status.as_u16(),
            }
        });

        tracing::error!(
            error_code = self.error_code(),
            status = status.as_u16(),
            "Request error: {}",
            self
        );

        (status, axum::Json(body)).into_response()
    }
}

/// Result type alias for RustCommerce operations.
pub type Result<T> = std::result::Result<T, RustCommerceError>;
