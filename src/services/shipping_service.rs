//! Shipping service
//!
//! Shipping rate calculation and zone matching.

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Result;
use crate::models::shipping::*;

/// Service for shipping rate calculation.
pub struct ShippingService;

impl ShippingService {
    /// Get available shipping rates for an address and order.
    pub async fn get_rates(pool: &PgPool, req: &ShippingRateRequest) -> Result<Vec<ShippingRate>> {
        // Find matching zones
        let zones = sqlx::query_as::<_, ShippingZone>(
            r#"
            SELECT * FROM rc_shipping_zones
            WHERE is_active = TRUE
              AND ($1 = ANY(countries) OR countries = '{}')
            ORDER BY sort_order
            "#,
        )
        .bind(&req.country)
        .fetch_all(pool)
        .await?;

        let mut rates = Vec::new();

        for zone in &zones {
            let methods = sqlx::query_as::<_, ShippingMethod>(
                r#"
                SELECT * FROM rc_shipping_methods
                WHERE zone_id = $1
                  AND is_active = TRUE
                  AND ($2::numeric IS NULL OR min_order_amount IS NULL OR $2 >= min_order_amount)
                  AND ($2::numeric IS NULL OR max_order_amount IS NULL OR $2 <= max_order_amount)
                ORDER BY sort_order
                "#,
            )
            .bind(zone.id)
            .bind(req.order_subtotal)
            .fetch_all(pool)
            .await?;

            for method in methods {
                let price = Self::calculate_method_price(&method, &req);
                rates.push(ShippingRate {
                    method_id: method.id,
                    method_name: method.name.clone(),
                    price,
                    estimated_days_min: method.estimated_days_min,
                    estimated_days_max: method.estimated_days_max,
                    description: method.description.clone(),
                });
            }
        }

        Ok(rates)
    }

    /// Calculate shipping cost for a specific method.
    fn calculate_method_price(method: &ShippingMethod, req: &ShippingRateRequest) -> Decimal {
        match method.shipping_type {
            ShippingType::Free => Decimal::ZERO,
            ShippingType::FlatRate => method.price,
            ShippingType::PerItem => method.price * Decimal::from(req.item_count),
            ShippingType::WeightBased => {
                if let Some(weight) = req.total_weight {
                    method.price * weight
                } else {
                    method.price
                }
            }
            ShippingType::PriceBased => {
                // E.g., percentage of order subtotal
                let pct = method.price / Decimal::from(100);
                req.order_subtotal * pct
            }
        }
    }

    /// Simplified shipping calculation used during checkout.
    pub async fn calculate_for_order(
        pool: &PgPool,
        country: &str,
        state: Option<&str>,
        order_subtotal: Decimal,
        item_count: i32,
    ) -> Result<Decimal> {
        let req = ShippingRateRequest {
            country: country.to_string(),
            state: state.map(|s| s.to_string()),
            postal_code: None,
            order_subtotal,
            total_weight: None,
            item_count,
        };

        let rates = Self::get_rates(pool, &req).await?;

        // Return cheapest rate, or zero if no shipping zones configured
        Ok(rates
            .iter()
            .map(|r| r.price)
            .min()
            .unwrap_or(Decimal::ZERO))
    }

    /// Create a shipping zone.
    pub async fn create_zone(
        pool: &PgPool,
        req: &CreateShippingZoneRequest,
    ) -> Result<ShippingZone> {
        let zone = sqlx::query_as::<_, ShippingZone>(
            r#"
            INSERT INTO rc_shipping_zones (name, countries, states, postal_codes)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(&req.name)
        .bind(&req.countries)
        .bind(&req.states)
        .bind(&req.postal_codes)
        .fetch_one(pool)
        .await?;

        Ok(zone)
    }

    /// Create a shipping method within a zone.
    pub async fn create_method(
        pool: &PgPool,
        req: &CreateShippingMethodRequest,
    ) -> Result<ShippingMethod> {
        let method = sqlx::query_as::<_, ShippingMethod>(
            r#"
            INSERT INTO rc_shipping_methods (
                zone_id, name, description, shipping_type, price,
                min_order_amount, max_order_amount, min_weight, max_weight,
                estimated_days_min, estimated_days_max
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(req.zone_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.shipping_type)
        .bind(req.price)
        .bind(&req.min_order_amount)
        .bind(&req.max_order_amount)
        .bind(&req.min_weight)
        .bind(&req.max_weight)
        .bind(&req.estimated_days_min)
        .bind(&req.estimated_days_max)
        .fetch_one(pool)
        .await?;

        Ok(method)
    }

    /// List all shipping zones with their methods.
    pub async fn list_zones(pool: &PgPool) -> Result<Vec<ShippingZone>> {
        let zones = sqlx::query_as::<_, ShippingZone>(
            "SELECT * FROM rc_shipping_zones ORDER BY sort_order",
        )
        .fetch_all(pool)
        .await?;

        Ok(zones)
    }

    /// List methods for a zone.
    pub async fn list_methods(pool: &PgPool, zone_id: Uuid) -> Result<Vec<ShippingMethod>> {
        let methods = sqlx::query_as::<_, ShippingMethod>(
            "SELECT * FROM rc_shipping_methods WHERE zone_id = $1 ORDER BY sort_order",
        )
        .bind(zone_id)
        .fetch_all(pool)
        .await?;

        Ok(methods)
    }
}
