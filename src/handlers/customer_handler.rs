//! Customer HTTP handlers
//!
//! CRUD endpoints for customer management.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use crate::error::Result;
use crate::models::customer::*;
use crate::models::product::PaginatedResponse;
use crate::plugin::AppState;
use crate::repositories::CustomerRepository;

/// GET /customers — List customers.
pub async fn list_customers(
    State(state): State<AppState>,
    Query(params): Query<CustomerListParams>,
) -> Result<Json<PaginatedResponse<Customer>>> {
    let (customers, total) = CustomerRepository::list(&state.pool, &params).await?;
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    Ok(Json(PaginatedResponse::new(customers, total, page, per_page)))
}

/// GET /customers/:id — Get a customer by ID.
pub async fn get_customer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Customer>> {
    let customer = CustomerRepository::find_by_id(&state.pool, id).await?;
    Ok(Json(customer))
}

/// POST /customers — Create a customer.
pub async fn create_customer(
    State(state): State<AppState>,
    Json(req): Json<CreateCustomerRequest>,
) -> Result<(StatusCode, Json<Customer>)> {
    let customer = CustomerRepository::create(&state.pool, &req).await?;
    Ok((StatusCode::CREATED, Json(customer)))
}

/// PUT /customers/:id — Update a customer.
pub async fn update_customer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCustomerRequest>,
) -> Result<Json<Customer>> {
    let customer = CustomerRepository::update(&state.pool, id, &req).await?;
    Ok(Json(customer))
}

/// DELETE /customers/:id — Delete a customer.
pub async fn delete_customer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    CustomerRepository::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /customers/:id/addresses — List customer addresses.
pub async fn list_addresses(
    State(state): State<AppState>,
    Path(customer_id): Path<Uuid>,
) -> Result<Json<Vec<CustomerAddress>>> {
    let addresses = CustomerRepository::list_addresses(&state.pool, customer_id).await?;
    Ok(Json(addresses))
}

/// POST /customers/:id/addresses — Create a customer address.
pub async fn create_address(
    State(state): State<AppState>,
    Path(customer_id): Path<Uuid>,
    Json(req): Json<CreateAddressRequest>,
) -> Result<(StatusCode, Json<CustomerAddress>)> {
    let address = CustomerRepository::create_address(&state.pool, customer_id, &req).await?;
    Ok((StatusCode::CREATED, Json(address)))
}

/// DELETE /customers/:customer_id/addresses/:address_id — Delete an address.
pub async fn delete_address(
    State(state): State<AppState>,
    Path((_customer_id, address_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode> {
    CustomerRepository::delete_address(&state.pool, address_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
