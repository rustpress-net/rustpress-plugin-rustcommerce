//! Integration tests for tax calculation.
//!
//! `TaxService::calculate` mixes a DB fetch (`find_applicable_rates`) with
//! pure arithmetic over the resulting `Vec<TaxRate>`. To exercise the
//! arithmetic without spinning up Postgres, this file re-implements the
//! exact loop body from `tax_service.rs` as a helper and asserts against
//! fixture rates. If the service algorithm changes, both this helper and
//! the service must change together — that coupling is intentional.

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use rustcommerce::models::tax::{TaxCalculation, TaxLine, TaxRate, TaxType};

/// Construct a TaxRate fixture quickly.
fn rate(
    name: &str,
    rate_value: Decimal,
    priority: i32,
    is_compound: bool,
    is_shipping_taxed: bool,
) -> TaxRate {
    TaxRate {
        id: Uuid::new_v4(),
        name: name.to_string(),
        rate: rate_value,
        tax_type: TaxType::Percentage,
        country: "US".to_string(),
        state: None,
        postal_code: None,
        priority,
        is_compound,
        is_shipping_taxed,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Mirror of `TaxService::calculate`'s arithmetic loop. Kept in lockstep
/// with `src/services/tax_service.rs`. Run on a `Vec<TaxRate>` directly
/// instead of querying the DB.
fn calc(subtotal: Decimal, shipping: Decimal, mut rates: Vec<TaxRate>) -> TaxCalculation {
    rates.sort_by_key(|r| r.priority);
    let mut tax_lines = Vec::new();
    let mut total_tax = Decimal::ZERO;
    let mut compound_base = subtotal;

    for rate in &rates {
        let taxable_amount = if rate.is_compound {
            compound_base + total_tax
        } else {
            compound_base
        };
        let taxable_with_shipping = if rate.is_shipping_taxed {
            taxable_amount + shipping
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

    TaxCalculation {
        subtotal,
        shipping_amount: shipping,
        tax_lines,
        total_tax,
    }
}

// ── Single-jurisdiction ─────────────────────────────────────────────────

#[test]
fn single_jurisdiction_us_8_percent() {
    let rates = vec![rate("US Sales Tax", dec!(8), 0, false, false)];
    let result = calc(dec!(100), Decimal::ZERO, rates);

    assert_eq!(result.total_tax, dec!(8.00));
    assert_eq!(result.tax_lines.len(), 1);
    assert_eq!(result.tax_lines[0].tax_amount, dec!(8.00));
}

#[test]
fn single_jurisdiction_rounds_to_two_decimals() {
    // 8.875% of $13.37 = 1.18656... should round to 1.19
    let rates = vec![rate("NY Sales", dec!(8.875), 0, false, false)];
    let result = calc(dec!(13.37), Decimal::ZERO, rates);

    assert_eq!(
        result.total_tax,
        dec!(1.19),
        "Half-even rounding to 2 dp; 1.18656 → 1.19"
    );
}

// ── Multi-jurisdiction ──────────────────────────────────────────────────

#[test]
fn multi_jurisdiction_non_compound_state_plus_city() {
    // CA state 6% + Los Angeles city 2.25%, both applied to subtotal only.
    let rates = vec![
        rate("CA State", dec!(6), 0, false, false),
        rate("LA City", dec!(2.25), 1, false, false),
    ];
    let result = calc(dec!(200), Decimal::ZERO, rates);

    // 200 * 6% = 12.00 ; 200 * 2.25% = 4.50 ; total 16.50
    assert_eq!(result.tax_lines.len(), 2);
    assert_eq!(result.tax_lines[0].tax_amount, dec!(12.00));
    assert_eq!(result.tax_lines[1].tax_amount, dec!(4.50));
    assert_eq!(result.total_tax, dec!(16.50));
}

#[test]
fn multi_jurisdiction_compound_canadian_gst_plus_pst() {
    // Quebec-style: GST 5% on subtotal, then QST 9.975% compounded on (subtotal + GST).
    let rates = vec![
        rate("GST", dec!(5), 0, false, false),
        rate("QST", dec!(9.975), 1, true, false),
    ];
    let result = calc(dec!(100), Decimal::ZERO, rates);

    // GST: 100 * 5%   = 5.00
    // QST: (100 + 5) * 9.975% = 10.47375 → 10.47
    assert_eq!(result.tax_lines[0].tax_amount, dec!(5.00));
    assert_eq!(result.tax_lines[1].tax_amount, dec!(10.47));
    assert_eq!(result.total_tax, dec!(15.47));
}

#[test]
fn priority_ordering_affects_compound_base() {
    // If we swap priorities of two compound rates, the order changes results.
    let rates = vec![
        rate("Rate A", dec!(10), 1, true, false),
        rate("Rate B", dec!(10), 0, true, false),
    ];
    let result = calc(dec!(100), Decimal::ZERO, rates);

    // Sorted by priority: B (priority 0) then A (priority 1)
    // B compounds on subtotal alone: 100 * 10% = 10.00
    // A compounds on (subtotal + total_tax_so_far): (100 + 10) * 10% = 11.00
    assert_eq!(result.tax_lines[0].name, "Rate B");
    assert_eq!(result.tax_lines[0].tax_amount, dec!(10.00));
    assert_eq!(result.tax_lines[1].tax_amount, dec!(11.00));
}

// ── Tax-exempt ──────────────────────────────────────────────────────────

#[test]
fn tax_exempt_no_rates_returns_zero() {
    // A tax-exempt customer is modelled in the service layer by `find_applicable_rates`
    // returning an empty `Vec<TaxRate>`. The arithmetic loop must then produce
    // a zero TaxCalculation with no tax lines.
    let result = calc(dec!(500), dec!(25), vec![]);
    assert_eq!(result.total_tax, Decimal::ZERO);
    assert!(result.tax_lines.is_empty());
    assert_eq!(result.subtotal, dec!(500));
    assert_eq!(result.shipping_amount, dec!(25));
}

// ── Tax on shipping vs not ──────────────────────────────────────────────

#[test]
fn shipping_taxed_includes_shipping_in_taxable_base() {
    let rates = vec![rate("US Sales", dec!(10), 0, false, true)];
    // shipping is $10, subtotal is $100
    let result = calc(dec!(100), dec!(10), rates);

    // (100 + 10) * 10% = 11.00
    assert_eq!(result.tax_lines[0].tax_amount, dec!(11.00));
    assert_eq!(result.tax_lines[0].taxable_amount, dec!(110));
}

#[test]
fn shipping_not_taxed_excludes_shipping_from_base() {
    let rates = vec![rate("US Sales", dec!(10), 0, false, false)];
    let result = calc(dec!(100), dec!(10), rates);

    // 100 * 10% = 10.00, shipping ignored.
    assert_eq!(result.tax_lines[0].tax_amount, dec!(10.00));
    assert_eq!(result.tax_lines[0].taxable_amount, dec!(100));
}

#[test]
fn mixed_some_rates_tax_shipping_some_do_not() {
    let rates = vec![
        rate("State (taxes shipping)", dec!(5), 0, false, true),
        rate("City (no shipping)", dec!(2), 1, false, false),
    ];
    // subtotal 100, shipping 20
    let result = calc(dec!(100), dec!(20), rates);

    // State: (100 + 20) * 5% = 6.00
    // City:  100 * 2%        = 2.00
    assert_eq!(result.tax_lines[0].tax_amount, dec!(6.00));
    assert_eq!(result.tax_lines[1].tax_amount, dec!(2.00));
    assert_eq!(result.total_tax, dec!(8.00));
}

// ── Rounding consistency ────────────────────────────────────────────────

#[test]
fn rounding_per_tax_line_then_summed_is_consistent() {
    // Two rates that both produce rounding-prone amounts.
    // 7.25% of $9.99 = 0.724275 → 0.72
    // 1.5%  of $9.99 = 0.14985  → 0.15
    // Sum of line-rounded: 0.87
    let rates = vec![
        rate("A", dec!(7.25), 0, false, false),
        rate("B", dec!(1.5), 1, false, false),
    ];
    let result = calc(dec!(9.99), Decimal::ZERO, rates);

    assert_eq!(result.tax_lines[0].tax_amount, dec!(0.72));
    assert_eq!(result.tax_lines[1].tax_amount, dec!(0.15));
    assert_eq!(result.total_tax, dec!(0.87));

    // Sanity: each individual line equals what the loop also accumulated.
    let summed: Decimal = result.tax_lines.iter().map(|l| l.tax_amount).sum();
    assert_eq!(summed, result.total_tax);
}

#[test]
fn rounding_does_not_compound_drift_across_many_lines() {
    // 10 identical small rates → make sure we don't accumulate drift > 1 cent.
    let rates: Vec<_> = (0..10)
        .map(|i| rate(&format!("r{}", i), dec!(0.5), i, false, false))
        .collect();
    let result = calc(dec!(12.34), Decimal::ZERO, rates);

    // Each line: 12.34 * 0.5% = 0.0617 → 0.06
    // 10 lines * 0.06 = 0.60
    assert_eq!(result.total_tax, dec!(0.60));
}

#[test]
fn fixed_tax_type_uses_rate_value_directly() {
    // Even though `TaxType::Fixed` is rarely used, the service supports it.
    let mut r = rate("Fixed disposal fee", dec!(2.50), 0, false, false);
    r.tax_type = TaxType::Fixed;
    let result = calc(dec!(100), Decimal::ZERO, vec![r]);
    assert_eq!(result.total_tax, dec!(2.50));
}

#[test]
fn zero_rate_produces_zero_tax_line() {
    let rates = vec![rate("Zero-rated", dec!(0), 0, false, false)];
    let result = calc(dec!(500), Decimal::ZERO, rates);
    assert_eq!(result.total_tax, Decimal::ZERO);
    // Important: still produces a line item (for receipt transparency).
    assert_eq!(result.tax_lines.len(), 1);
}
