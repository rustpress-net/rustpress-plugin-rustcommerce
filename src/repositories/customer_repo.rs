//! Customer repository
//!
//! Database access for customers and their addresses.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{Result, RustCommerceError};
use crate::models::customer::*;

/// Repository for customer database operations.
pub struct CustomerRepository;

impl CustomerRepository {
    /// Fetch a customer by ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Customer> {
        sqlx::query_as::<_, Customer>("SELECT * FROM rc_customers WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(RustCommerceError::CustomerNotFound(id))
    }

    /// Fetch a customer by email.
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<Customer>> {
        let customer = sqlx::query_as::<_, Customer>(
            "SELECT * FROM rc_customers WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(customer)
    }

    /// List customers with filtering and pagination.
    pub async fn list(pool: &PgPool, params: &CustomerListParams) -> Result<(Vec<Customer>, i64)> {
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(20).min(100);
        let offset = ((page - 1) * per_page) as i64;

        let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
        let sort_order = params.sort_order.as_deref().unwrap_or("desc");

        let sort_column = match sort_by {
            "email" => "email",
            "first_name" => "first_name",
            "last_name" => "last_name",
            "total_orders" => "total_orders",
            "total_spent" => "total_spent",
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
            SELECT * FROM rc_customers
            WHERE ($1::boolean IS NULL OR is_active = $1)
              AND ($2::text IS NULL OR first_name ILIKE '%' || $2 || '%' OR last_name ILIKE '%' || $2 || '%' OR email ILIKE '%' || $2 || '%')
            ORDER BY {} {}
            LIMIT $3 OFFSET $4
            "#,
            sort_column, order
        );

        let customers = sqlx::query_as::<_, Customer>(&query)
            .bind(&params.is_active)
            .bind(&params.search)
            .bind(per_page as i64)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM rc_customers
            WHERE ($1::boolean IS NULL OR is_active = $1)
              AND ($2::text IS NULL OR first_name ILIKE '%' || $2 || '%' OR last_name ILIKE '%' || $2 || '%' OR email ILIKE '%' || $2 || '%')
            "#,
        )
        .bind(&params.is_active)
        .bind(&params.search)
        .fetch_one(pool)
        .await?;

        Ok((customers, total))
    }

    /// Create a new customer.
    pub async fn create(pool: &PgPool, req: &CreateCustomerRequest) -> Result<Customer> {
        // Check for duplicate email
        if let Some(_existing) = Self::find_by_email(pool, &req.email).await? {
            return Err(RustCommerceError::DuplicateEntry(format!(
                "Customer with email '{}' already exists",
                req.email
            )));
        }

        let customer = sqlx::query_as::<_, Customer>(
            r#"
            INSERT INTO rc_customers (user_id, email, first_name, last_name, phone, company, accepts_marketing, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(&req.user_id)
        .bind(&req.email)
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.phone)
        .bind(&req.company)
        .bind(req.accepts_marketing)
        .bind(&req.notes)
        .fetch_one(pool)
        .await?;

        Ok(customer)
    }

    /// Update a customer.
    pub async fn update(pool: &PgPool, id: Uuid, req: &UpdateCustomerRequest) -> Result<Customer> {
        let existing = Self::find_by_id(pool, id).await?;

        let email = req.email.as_deref().unwrap_or(&existing.email);
        let first_name = req.first_name.as_deref().unwrap_or(&existing.first_name);
        let last_name = req.last_name.as_deref().unwrap_or(&existing.last_name);
        let phone = req.phone.as_deref().or(existing.phone.as_deref());
        let company = req.company.as_deref().or(existing.company.as_deref());
        let is_active = req.is_active.unwrap_or(existing.is_active);
        let accepts_marketing = req.accepts_marketing.unwrap_or(existing.accepts_marketing);
        let notes = req.notes.as_deref().or(existing.notes.as_deref());

        let customer = sqlx::query_as::<_, Customer>(
            r#"
            UPDATE rc_customers SET
                email = $1, first_name = $2, last_name = $3, phone = $4,
                company = $5, is_active = $6, accepts_marketing = $7, notes = $8
            WHERE id = $9
            RETURNING *
            "#,
        )
        .bind(email)
        .bind(first_name)
        .bind(last_name)
        .bind(phone)
        .bind(company)
        .bind(is_active)
        .bind(accepts_marketing)
        .bind(notes)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(customer)
    }

    /// Delete a customer.
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM rc_customers WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RustCommerceError::CustomerNotFound(id));
        }
        Ok(())
    }

    /// Increment order totals for a customer after a completed order.
    pub async fn record_order(
        pool: &PgPool,
        customer_id: Uuid,
        order_total: rust_decimal::Decimal,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE rc_customers SET
                total_orders = total_orders + 1,
                total_spent = total_spent + $1,
                last_order_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(order_total)
        .bind(customer_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// List addresses for a customer.
    pub async fn list_addresses(pool: &PgPool, customer_id: Uuid) -> Result<Vec<CustomerAddress>> {
        let addresses = sqlx::query_as::<_, CustomerAddress>(
            "SELECT * FROM rc_customer_addresses WHERE customer_id = $1 ORDER BY is_default_shipping DESC, created_at",
        )
        .bind(customer_id)
        .fetch_all(pool)
        .await?;

        Ok(addresses)
    }

    /// Create a customer address.
    pub async fn create_address(
        pool: &PgPool,
        customer_id: Uuid,
        req: &CreateAddressRequest,
    ) -> Result<CustomerAddress> {
        // If this is set as default, unset previous defaults
        if req.is_default_shipping {
            sqlx::query("UPDATE rc_customer_addresses SET is_default_shipping = FALSE WHERE customer_id = $1")
                .bind(customer_id)
                .execute(pool)
                .await?;
        }
        if req.is_default_billing {
            sqlx::query("UPDATE rc_customer_addresses SET is_default_billing = FALSE WHERE customer_id = $1")
                .bind(customer_id)
                .execute(pool)
                .await?;
        }

        let address = sqlx::query_as::<_, CustomerAddress>(
            r#"
            INSERT INTO rc_customer_addresses (
                customer_id, label, first_name, last_name, company,
                address_line1, address_line2, city, state, postal_code, country, phone,
                is_default_shipping, is_default_billing
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(customer_id)
        .bind(&req.label)
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.company)
        .bind(&req.address_line1)
        .bind(&req.address_line2)
        .bind(&req.city)
        .bind(&req.state)
        .bind(&req.postal_code)
        .bind(&req.country)
        .bind(&req.phone)
        .bind(req.is_default_shipping)
        .bind(req.is_default_billing)
        .fetch_one(pool)
        .await?;

        Ok(address)
    }

    /// Delete a customer address.
    pub async fn delete_address(pool: &PgPool, address_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM rc_customer_addresses WHERE id = $1")
            .bind(address_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RustCommerceError::InvalidInput(format!(
                "Address {} not found",
                address_id
            )));
        }
        Ok(())
    }
}
