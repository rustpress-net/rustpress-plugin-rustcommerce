//! Product HTTP handlers
//!
//! CRUD endpoints for product management.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::error::Result;
use crate::models::product::*;
use crate::plugin::AppState;
use crate::services::ProductService;

/// GET /products — List products with filtering.
pub async fn list_products(
    State(state): State<AppState>,
    Query(params): Query<ProductListParams>,
) -> Result<Json<PaginatedResponse<Product>>> {
    let result = ProductService::list(&state.pool, &params).await?;
    Ok(Json(result))
}

/// GET /products/:id — Get a single product by ID.
pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Product>> {
    let product = ProductService::get_by_id(&state.pool, id).await?;
    Ok(Json(product))
}

/// GET /products/slug/:slug — Get a product by slug.
pub async fn get_product_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Product>> {
    let product = ProductService::get_by_slug(&state.pool, &slug).await?;
    Ok(Json(product))
}

/// POST /products — Create a new product.
pub async fn create_product(
    State(state): State<AppState>,
    Json(req): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<Product>)> {
    let product = ProductService::create(&state.pool, &req).await?;
    Ok((StatusCode::CREATED, Json(product)))
}

/// PUT /products/:id — Update a product.
pub async fn update_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProductRequest>,
) -> Result<Json<Product>> {
    let product = ProductService::update(&state.pool, id, &req).await?;
    Ok(Json(product))
}

/// DELETE /products/:id — Delete a product.
pub async fn delete_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    ProductService::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /products/:id/variants — List variants for a product.
pub async fn list_variants(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Vec<ProductVariant>>> {
    let variants = ProductService::list_variants(&state.pool, product_id).await?;
    Ok(Json(variants))
}

/// POST /products/:id/variants — Create a variant for a product.
pub async fn create_variant(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
    Json(req): Json<CreateVariantRequest>,
) -> Result<(StatusCode, Json<ProductVariant>)> {
    let variant = ProductService::create_variant(&state.pool, product_id, &req).await?;
    Ok((StatusCode::CREATED, Json(variant)))
}

/// GET /products/:id/images — List images for a product.
pub async fn list_product_images(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Vec<ProductImage>>> {
    let images = ProductService::list_images(&state.pool, product_id).await?;
    Ok(Json(images))
}
