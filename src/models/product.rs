//! Product models
//!
//! Represents products, variants, images, and related enums.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Product status lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "rc_product_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ProductStatus {
    Draft,
    Active,
    Archived,
    OutOfStock,
}

impl std::fmt::Display for ProductStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Active => write!(f, "active"),
            Self::Archived => write!(f, "archived"),
            Self::OutOfStock => write!(f, "out_of_stock"),
        }
    }
}

/// A product in the catalogue.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub sku: Option<String>,
    pub price: Decimal,
    pub compare_at_price: Option<Decimal>,
    pub cost_price: Option<Decimal>,
    pub status: ProductStatus,
    pub is_featured: bool,
    pub is_digital: bool,
    pub weight: Option<Decimal>,
    pub width: Option<Decimal>,
    pub height: Option<Decimal>,
    pub depth: Option<Decimal>,
    pub stock_quantity: i32,
    pub low_stock_threshold: i32,
    pub track_inventory: bool,
    pub allow_backorders: bool,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request payload for creating a new product.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be 1-255 characters"))]
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub sku: Option<String>,
    pub price: Decimal,
    pub compare_at_price: Option<Decimal>,
    pub cost_price: Option<Decimal>,
    pub status: Option<ProductStatus>,
    #[serde(default)]
    pub is_featured: bool,
    #[serde(default)]
    pub is_digital: bool,
    pub weight: Option<Decimal>,
    pub width: Option<Decimal>,
    pub height: Option<Decimal>,
    pub depth: Option<Decimal>,
    #[serde(default)]
    pub stock_quantity: i32,
    pub low_stock_threshold: Option<i32>,
    pub track_inventory: Option<bool>,
    #[serde(default)]
    pub allow_backorders: bool,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub category_ids: Option<Vec<Uuid>>,
}

/// Request payload for updating an existing product.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateProductRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be 1-255 characters"))]
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub short_description: Option<String>,
    pub sku: Option<String>,
    pub price: Option<Decimal>,
    pub compare_at_price: Option<Decimal>,
    pub cost_price: Option<Decimal>,
    pub status: Option<ProductStatus>,
    pub is_featured: Option<bool>,
    pub is_digital: Option<bool>,
    pub weight: Option<Decimal>,
    pub width: Option<Decimal>,
    pub height: Option<Decimal>,
    pub depth: Option<Decimal>,
    pub stock_quantity: Option<i32>,
    pub low_stock_threshold: Option<i32>,
    pub track_inventory: Option<bool>,
    pub allow_backorders: Option<bool>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub category_ids: Option<Vec<Uuid>>,
}

/// A specific variant of a product (e.g., size/color combination).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductVariant {
    pub id: Uuid,
    pub product_id: Uuid,
    pub name: String,
    pub sku: Option<String>,
    pub price: Decimal,
    pub compare_at_price: Option<Decimal>,
    pub cost_price: Option<Decimal>,
    pub stock_quantity: i32,
    pub weight: Option<Decimal>,
    pub option1_name: Option<String>,
    pub option1_value: Option<String>,
    pub option2_name: Option<String>,
    pub option2_value: Option<String>,
    pub option3_name: Option<String>,
    pub option3_value: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request payload for creating a product variant.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateVariantRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub sku: Option<String>,
    pub price: Decimal,
    pub compare_at_price: Option<Decimal>,
    pub cost_price: Option<Decimal>,
    #[serde(default)]
    pub stock_quantity: i32,
    pub weight: Option<Decimal>,
    pub option1_name: Option<String>,
    pub option1_value: Option<String>,
    pub option2_name: Option<String>,
    pub option2_value: Option<String>,
    pub option3_name: Option<String>,
    pub option3_value: Option<String>,
}

/// An image associated with a product or variant.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductImage {
    pub id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub url: String,
    pub alt_text: Option<String>,
    pub sort_order: i32,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

/// Query parameters for listing products.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub status: Option<ProductStatus>,
    pub category_id: Option<Uuid>,
    pub is_featured: Option<bool>,
    pub search: Option<String>,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

impl Default for ProductListParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
            status: None,
            category_id: None,
            is_featured: None,
            search: None,
            min_price: None,
            max_price: None,
            sort_by: Some("created_at".to_string()),
            sort_order: Some("desc".to_string()),
        }
    }
}

/// Paginated list response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: i64, page: u32, per_page: u32) -> Self {
        let total_pages = if per_page > 0 {
            ((total as u32) + per_page - 1) / per_page
        } else {
            0
        };
        Self {
            items,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}
