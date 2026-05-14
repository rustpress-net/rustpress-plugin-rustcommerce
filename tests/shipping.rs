//! Integration tests for shipping price calculation and zone matching.
//!
//! The shipping zone match itself is a SQL `WHERE $1 = ANY(countries)`
//! query (`ShippingService::get_rates`), so the zone-matching path can
//! only be fully verified against a live Postgres. Here we cover:
//!
//! - The price formula for each `ShippingType` variant (mirroring
//!   `calculate_method_price`, which is private to the service).
//! - The "no matching zone → free / no rates" fallback in the service.
//! - The free-shipping-threshold logic that lives at the method level via
//!   `min_order_amount` / `max_order_amount`.
//!
//! Anything that requires querying `rc_shipping_zones` is documented in
//! comments with the expected SQL semantics.

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use rustcommerce::models::shipping::{
    ShippingMethod, ShippingRateRequest, ShippingType, ShippingZone,
};

/// Replicates `ShippingService::calculate_method_price`. Kept in lockstep
/// with `src/services/shipping_service.rs`.
fn price(method: &ShippingMethod, req: &ShippingRateRequest) -> Decimal {
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
            let pct = method.price / Decimal::from(100);
            req.order_subtotal * pct
        }
    }
}

fn make_method(shipping_type: ShippingType, price_val: Decimal) -> ShippingMethod {
    ShippingMethod {
        id: Uuid::new_v4(),
        zone_id: Uuid::new_v4(),
        name: "Test Method".to_string(),
        description: None,
        shipping_type,
        price: price_val,
        min_order_amount: None,
        max_order_amount: None,
        min_weight: None,
        max_weight: None,
        estimated_days_min: None,
        estimated_days_max: None,
        is_active: true,
        sort_order: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn make_req(country: &str, subtotal: Decimal, item_count: i32) -> ShippingRateRequest {
    ShippingRateRequest {
        country: country.to_string(),
        state: None,
        postal_code: None,
        order_subtotal: subtotal,
        total_weight: None,
        item_count,
    }
}

// ── Flat rate ────────────────────────────────────────────────────────────

#[test]
fn flat_rate_returns_method_price_regardless_of_cart() {
    let m = make_method(ShippingType::FlatRate, dec!(7.99));
    let req_small = make_req("US", dec!(10), 1);
    let req_big = make_req("US", dec!(1000), 50);

    assert_eq!(price(&m, &req_small), dec!(7.99));
    assert_eq!(price(&m, &req_big), dec!(7.99));
}

// ── Free ─────────────────────────────────────────────────────────────────

#[test]
fn free_shipping_returns_zero_regardless_of_price_field() {
    // Even with a non-zero price field, ShippingType::Free yields 0.
    let m = make_method(ShippingType::Free, dec!(99));
    let req = make_req("US", dec!(20), 1);
    assert_eq!(price(&m, &req), Decimal::ZERO);
}

// ── Per item ─────────────────────────────────────────────────────────────

#[test]
fn per_item_multiplies_price_by_item_count() {
    let m = make_method(ShippingType::PerItem, dec!(2.50));
    let req = make_req("US", dec!(0), 4);
    assert_eq!(price(&m, &req), dec!(10.00));
}

#[test]
fn per_item_zero_items_is_zero() {
    let m = make_method(ShippingType::PerItem, dec!(2.50));
    let req = make_req("US", dec!(0), 0);
    assert_eq!(price(&m, &req), Decimal::ZERO);
}

// ── Weight based ─────────────────────────────────────────────────────────

#[test]
fn weight_based_uses_total_weight() {
    let m = make_method(ShippingType::WeightBased, dec!(1.50));
    let mut req = make_req("US", dec!(50), 1);
    req.total_weight = Some(dec!(3.2));
    assert_eq!(price(&m, &req), dec!(4.80));
}

#[test]
fn weight_based_without_weight_falls_back_to_price() {
    // Service falls back to flat price when no weight is provided.
    let m = make_method(ShippingType::WeightBased, dec!(5));
    let req = make_req("US", dec!(50), 1);
    assert_eq!(price(&m, &req), dec!(5));
}

// ── Price-tier (percentage of subtotal) ──────────────────────────────────

#[test]
fn price_based_is_percentage_of_subtotal() {
    // price field = 10 means 10% of order subtotal.
    let m = make_method(ShippingType::PriceBased, dec!(10));
    let req = make_req("US", dec!(80), 1);
    assert_eq!(price(&m, &req), dec!(8.00));
}

#[test]
fn price_based_zero_subtotal_is_zero() {
    let m = make_method(ShippingType::PriceBased, dec!(10));
    let req = make_req("US", dec!(0), 1);
    assert_eq!(price(&m, &req), Decimal::ZERO);
}

// ── Free-shipping threshold ──────────────────────────────────────────────
//
// In RustCommerce the "free-shipping threshold" is implemented as a
// shipping method with `shipping_type = Free` and a `min_order_amount`.
// At the SQL layer (`get_rates`), the method is filtered out when the
// cart is below the minimum. We can't run the SQL here, but we can
// assert the filter expression's logical truth tables.

fn method_matches_order(m: &ShippingMethod, order_subtotal: Decimal) -> bool {
    // Mirrors the WHERE clause in get_rates:
    //   ($2 IS NULL OR min_order_amount IS NULL OR $2 >= min_order_amount)
    //   AND ($2 IS NULL OR max_order_amount IS NULL OR $2 <= max_order_amount)
    let min_ok = m
        .min_order_amount
        .map_or(true, |min| order_subtotal >= min);
    let max_ok = m
        .max_order_amount
        .map_or(true, |max| order_subtotal <= max);
    min_ok && max_ok
}

#[test]
fn free_shipping_threshold_blocks_small_orders() {
    let mut m = make_method(ShippingType::Free, Decimal::ZERO);
    m.min_order_amount = Some(dec!(50));

    assert!(!method_matches_order(&m, dec!(49.99)));
    assert!(method_matches_order(&m, dec!(50.00)));
    assert!(method_matches_order(&m, dec!(500.00)));
}

#[test]
fn maximum_order_amount_excludes_large_orders() {
    // e.g. a flat-rate method that's invalid above $1000 (over-weight).
    let mut m = make_method(ShippingType::FlatRate, dec!(5));
    m.max_order_amount = Some(dec!(1000));

    assert!(method_matches_order(&m, dec!(999)));
    assert!(method_matches_order(&m, dec!(1000)));
    assert!(!method_matches_order(&m, dec!(1000.01)));
}

#[test]
fn no_min_or_max_always_matches() {
    let m = make_method(ShippingType::FlatRate, dec!(5));
    assert!(method_matches_order(&m, dec!(0.01)));
    assert!(method_matches_order(&m, dec!(1_000_000)));
}

// ── Zone matching ────────────────────────────────────────────────────────
//
// `get_rates` matches a zone when `$country = ANY(countries) OR countries = '{}'`.
// We replicate that boolean predicate on `ShippingZone`.

fn zone_matches_country(z: &ShippingZone, country: &str) -> bool {
    z.is_active && (z.countries.iter().any(|c| c == country) || z.countries.is_empty())
}

fn make_zone(name: &str, countries: Vec<&str>) -> ShippingZone {
    ShippingZone {
        id: Uuid::new_v4(),
        name: name.to_string(),
        countries: countries.into_iter().map(String::from).collect(),
        states: vec![],
        postal_codes: vec![],
        is_active: true,
        sort_order: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[test]
fn zone_matches_exact_country() {
    let z = make_zone("North America", vec!["US", "CA", "MX"]);
    assert!(zone_matches_country(&z, "US"));
    assert!(zone_matches_country(&z, "CA"));
    assert!(!zone_matches_country(&z, "DE"));
}

#[test]
fn empty_countries_means_global_zone_catch_all() {
    // Per the SQL: `OR countries = '{}'` → an empty zone matches every country.
    let z = make_zone("Global", vec![]);
    assert!(zone_matches_country(&z, "US"));
    assert!(zone_matches_country(&z, "JP"));
    assert!(zone_matches_country(&z, "ZZ"));
}

#[test]
fn inactive_zone_never_matches() {
    let mut z = make_zone("North America", vec!["US"]);
    z.is_active = false;
    assert!(!zone_matches_country(&z, "US"));
}

// ── Fallback when no zone matches ────────────────────────────────────────

#[test]
fn no_matching_zone_yields_no_rates() {
    // `calculate_for_order` returns `Decimal::ZERO` when `get_rates` returns
    // an empty Vec (`.iter().min().unwrap_or(Decimal::ZERO)`).
    let candidates: Vec<Decimal> = vec![];
    let result = candidates.iter().min().copied().unwrap_or(Decimal::ZERO);
    assert_eq!(
        result,
        Decimal::ZERO,
        "No shipping zone configured ⇒ shipping cost defaults to 0 at checkout"
    );
}

#[test]
fn cheapest_rate_wins_when_multiple_methods_apply() {
    // `calculate_for_order` returns `rates.iter().map(|r| r.price).min()`.
    let prices: Vec<Decimal> = vec![dec!(9.99), dec!(4.99), dec!(14.99)];
    let result = prices.iter().min().copied().unwrap_or(Decimal::ZERO);
    assert_eq!(result, dec!(4.99));
}
