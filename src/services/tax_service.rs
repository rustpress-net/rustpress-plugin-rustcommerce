//! Tax service
//!
//! Tax rate lookup and calculation for orders.

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::error::{Result, RustCommerceError};
use crate::models::tax::*;

/// Service for tax calculation.
pub struct TaxService;

impl TaxService {
    /// Calculate taxes for an order based on shipping address.
    pub async fn calculate(
        pool: &PgPool,
        subtotal: Decimal,
        shipping_amount: Decimal,
        country: &str,
        state: Option<&str>,
        _postal_code: Option<&str>,
    ) -> Result<TaxCalculation> {
        let rates = Self::find_applicable_rates(pool, country, state).await?;

        let mut tax_lines = Vec::new();
        let mut total_tax = Decimal::ZERO;
        let mut compound_base = subtotal;

        // Sort by priority (lower = applied first)
        let mut sorted_rates = rates;
        sorted_rates.sort_by_key(|r| r.priority);

        for rate in &sorted_rates {
            let taxable_amount = if rate.is_compound {
                compound_base + total_tax
            } else {
                compound_base
            };

            // Optionally include shipping in taxable amount
            let taxable_with_shipping = if rate.is_shipping_taxed {
                taxable_amount + shipping_amount
            } else {
                taxable_amount
            };

            let tax_amount = match rate.tax_type {
                TaxType::Percentage => {
                    let pct = rate.rate / Decimal::from(100);
                    taxable_with_shipping * pct
                }
                TaxType::Fixed => rate.rate,
            };

            // Round to 2 decimal places
            let tax_amount = tax_amount.round_dp(2);

            tax_lines.push(TaxLine {
                tax_rate_id: rate.id,
                name: rate.name.clone(),
                rate: rate.rate,
                tax_type: rate.tax_type.clone(),
                taxable_amount: taxable_with_shipping,
                tax_amount,
            });

            total_tax += tax_amount;

            if rate.is_compound {
                compound_base = subtotal + total_tax;
            }
        }

        Ok(TaxCalculation {
            subtotal,
            shipping_amount,
            tax_lines,
            total_tax,
        })
    }

    /// Simplified tax calculation for checkout that returns just the total.
    pub async fn calculate_tax_for_address(
        pool: &PgPool,
        subtotal: Decimal,
        country: &str,
        state: Option<&str>,
    ) -> Result<Decimal> {
        let result = Self::calculate(
            pool,
            subtotal,
            Decimal::ZERO,
            country,
            state,
            None,
        )
        .await?;

        Ok(result.total_tax)
    }

    /// Find applicable tax rates for a given address.
    async fn find_applicable_rates(
        pool: &PgPool,
        country: &str,
        state: Option<&str>,
    ) -> Result<Vec<TaxRate>> {
        let rates = sqlx::query_as::<_, TaxRate>(
            r#"
            SELECT * FROM rc_tax_rates
            WHERE is_active = TRUE
              AND country = $1
              AND (state IS NULL OR state = $2)
            ORDER BY priority ASC
            "#,
        )
        .bind(country)
        .bind(state)
        .fetch_all(pool)
        .await?;

        Ok(rates)
    }

    /// Create a tax rate.
    pub async fn create_rate(pool: &PgPool, req: &CreateTaxRateRequest) -> Result<TaxRate> {
        req.validate()
            .map_err(|e| RustCommerceError::Validation(e.to_string()))?;

        let tax_type = req.tax_type.clone().unwrap_or(TaxType::Percentage);
        let priority = req.priority.unwrap_or(0);

        let rate = sqlx::query_as::<_, TaxRate>(
            r#"
            INSERT INTO rc_tax_rates (
                name, rate, tax_type, country, state, postal_code,
                priority, is_compound, is_shipping_taxed
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(&req.name)
        .bind(req.rate)
        .bind(&tax_type)
        .bind(&req.country)
        .bind(&req.state)
        .bind(&req.postal_code)
        .bind(priority)
        .bind(req.is_compound)
        .bind(req.is_shipping_taxed)
        .fetch_one(pool)
        .await?;

        Ok(rate)
    }

    /// Update a tax rate.
    pub async fn update_rate(pool: &PgPool, id: Uuid, req: &UpdateTaxRateRequest) -> Result<TaxRate> {
        let existing = sqlx::query_as::<_, TaxRate>(
            "SELECT * FROM rc_tax_rates WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RustCommerceError::InvalidInput(format!("Tax rate {} not found", id)))?;

        let name = req.name.as_deref().unwrap_or(&existing.name);
        let rate_val = req.rate.unwrap_or(existing.rate);
        let tax_type = req.tax_type.clone().unwrap_or(existing.tax_type);
        let country = req.country.as_deref().unwrap_or(&existing.country);
        let state = req.state.as_deref().or(existing.state.as_deref());
        let postal = req.postal_code.as_deref().or(existing.postal_code.as_deref());
        let priority = req.priority.unwrap_or(existing.priority);
        let compound = req.is_compound.unwrap_or(existing.is_compound);
        let ship_taxed = req.is_shipping_taxed.unwrap_or(existing.is_shipping_taxed);
        let active = req.is_active.unwrap_or(existing.is_active);

        let rate = sqlx::query_as::<_, TaxRate>(
            r#"
            UPDATE rc_tax_rates SET
                name = $1, rate = $2, tax_type = $3, country = $4, state = $5,
                postal_code = $6, priority = $7, is_compound = $8,
                is_shipping_taxed = $9, is_active = $10
            WHERE id = $11
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(rate_val)
        .bind(&tax_type)
        .bind(country)
        .bind(state)
        .bind(postal)
        .bind(priority)
        .bind(compound)
        .bind(ship_taxed)
        .bind(active)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(rate)
    }

    /// List all tax rates.
    pub async fn list_rates(pool: &PgPool) -> Result<Vec<TaxRate>> {
        let rates = sqlx::query_as::<_, TaxRate>(
            "SELECT * FROM rc_tax_rates ORDER BY country, state, priority",
        )
        .fetch_all(pool)
        .await?;

        Ok(rates)
    }

    /// Delete a tax rate.
    pub async fn delete_rate(pool: &PgPool, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM rc_tax_rates WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(RustCommerceError::InvalidInput(format!(
                "Tax rate {} not found",
                id
            )));
        }
        Ok(())
    }
}
