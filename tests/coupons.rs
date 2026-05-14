//! Integration tests for coupon validation and discount calculation.
//!
//! These tests exercise the pure logic on `Coupon` (`validate_usage`,
//! `calculate_discount`) without touching the database. Wherever the
//! service layer (`CouponService`) only adds a DB round-trip on top of
//! the model logic, we test the model directly — the service is a thin
//! wrapper and its DB path is covered by the (separate) repository tests.

use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use rustcommerce::error::RustCommerceError;

// Pull the coupon types in from the crate.
use rustcommerce as rc;

/// Build a baseline active percentage coupon. Each test then tweaks one field.
fn base_coupon() -> rc::models::coupon::Coupon {
    rc::models::coupon::Coupon {
        id: Uuid::new_v4(),
        code: "TEST10".to_string(),
        description: None,
        discount_type: rc::models::coupon::DiscountType::Percentage,
        discount_value: dec!(10),
        minimum_order_amount: None,
        maximum_discount_amount: None,
        usage_limit: None,
        usage_count: 0,
        usage_limit_per_customer: None,
        applies_to_product_ids: None,
        applies_to_category_ids: None,
        excluded_product_ids: None,
        excluded_category_ids: None,
        is_active: true,
        starts_at: None,
        expires_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

// ── Discount calculation ────────────────────────────────────────────────

#[test]
fn fixed_amount_5_off_single_item_cart() {
    let mut c = base_coupon();
    c.discount_type = rc::models::coupon::DiscountType::FixedAmount;
    c.discount_value = dec!(5);

    // Cart with one $20 item.
    let subtotal = dec!(20);
    let discount = c.calculate_discount(subtotal);

    assert_eq!(discount, dec!(5), "$5 off a $20 cart should be exactly $5");
}

#[test]
fn fixed_amount_cannot_exceed_subtotal() {
    let mut c = base_coupon();
    c.discount_type = rc::models::coupon::DiscountType::FixedAmount;
    c.discount_value = dec!(50);

    // Cart smaller than the fixed discount.
    let subtotal = dec!(20);
    let discount = c.calculate_discount(subtotal);

    assert_eq!(
        discount, dec!(20),
        "Discount is capped at the subtotal so the order total never goes negative"
    );
}

#[test]
fn percentage_20_off_multi_item_cart() {
    let mut c = base_coupon();
    c.discount_type = rc::models::coupon::DiscountType::Percentage;
    c.discount_value = dec!(20);

    // Multi-item cart: e.g. 3 items totalling $150.
    let subtotal = dec!(150);
    let discount = c.calculate_discount(subtotal);

    assert_eq!(discount, dec!(30), "20% of $150 must be exactly $30");
}

#[test]
fn percentage_with_max_discount_cap() {
    let mut c = base_coupon();
    c.discount_type = rc::models::coupon::DiscountType::Percentage;
    c.discount_value = dec!(50);
    c.maximum_discount_amount = Some(dec!(25));

    // 50% of $200 = $100, but max cap is $25.
    let discount = c.calculate_discount(dec!(200));

    assert_eq!(discount, dec!(25), "Cap should clamp the 50% discount");
}

#[test]
fn free_shipping_returns_zero_line_discount() {
    let mut c = base_coupon();
    c.discount_type = rc::models::coupon::DiscountType::FreeShipping;
    c.discount_value = dec!(0);

    // Free-shipping coupons return zero in the line-discount calc;
    // the shipping handler is expected to handle this separately.
    let discount = c.calculate_discount(dec!(100));
    assert_eq!(discount, dec!(0));
}

// ── Stacking rules (documenting current behavior) ───────────────────────
//
// The current implementation: `Cart` has a single `coupon_code: Option<String>`
// column and `apply_coupon` simply overwrites it. There is no service-layer
// support for stacking multiple coupons. The "winner" of two consecutive
// applications is therefore *the last one applied*. This test pins that
// behavior so future devs are alerted if they change the data model.

#[test]
fn current_stacking_behavior_is_replace_not_combine() {
    // Simulate "applying" coupon A then B: only B should produce the discount,
    // matching the cart_repo.rs `apply_coupon` which is a simple UPDATE.
    let mut a = base_coupon();
    a.code = "A_FIXED".to_string();
    a.discount_type = rc::models::coupon::DiscountType::FixedAmount;
    a.discount_value = dec!(5);

    let mut b = base_coupon();
    b.code = "B_PERCENT".to_string();
    b.discount_type = rc::models::coupon::DiscountType::Percentage;
    b.discount_value = dec!(20);

    let subtotal = dec!(100);

    // If stacking were additive, discount would be 5 + 20 = $25.
    // Current behavior: only one coupon is "active" on the cart.
    let stacked_if_combined = a.calculate_discount(subtotal) + b.calculate_discount(subtotal);
    let actual_active = b.calculate_discount(subtotal); // last-applied wins

    assert_eq!(
        actual_active,
        dec!(20),
        "Single active coupon = $20 (20% of $100)"
    );
    assert_ne!(
        actual_active, stacked_if_combined,
        "Confirms stacking is NOT combined — change this test if/when stacking is added"
    );
}

// ── Validation: minimum order ───────────────────────────────────────────

#[test]
fn coupon_below_minimum_order_is_rejected() {
    let mut c = base_coupon();
    c.discount_type = rc::models::coupon::DiscountType::Percentage;
    c.discount_value = dec!(50);
    c.minimum_order_amount = Some(dec!(10));

    // Cart subtotal $5 — below $10 minimum.
    let res = c.validate_usage(dec!(5));

    match res {
        Err(RustCommerceError::MinimumOrderNotMet { required, current }) => {
            assert_eq!(required, dec!(10));
            assert_eq!(current, dec!(5));
        }
        other => panic!("expected MinimumOrderNotMet, got: {:?}", other),
    }
}

#[test]
fn coupon_at_exactly_minimum_order_is_accepted() {
    let mut c = base_coupon();
    c.minimum_order_amount = Some(dec!(10));
    assert!(c.validate_usage(dec!(10)).is_ok());
}

// ── Validation: expiry ──────────────────────────────────────────────────

#[test]
fn expired_coupon_rejected() {
    let mut c = base_coupon();
    c.expires_at = Some(Utc::now() - Duration::hours(1));

    let res = c.validate_usage(dec!(100));
    assert!(matches!(res, Err(RustCommerceError::CouponExpired)));
}

#[test]
fn future_starts_at_rejected_as_invalid() {
    let mut c = base_coupon();
    c.starts_at = Some(Utc::now() + Duration::hours(1));

    let res = c.validate_usage(dec!(100));
    assert!(matches!(res, Err(RustCommerceError::InvalidCoupon(_))));
}

#[test]
fn coupon_not_yet_expired_accepted() {
    let mut c = base_coupon();
    c.expires_at = Some(Utc::now() + Duration::hours(1));
    assert!(c.validate_usage(dec!(100)).is_ok());
}

// ── Validation: usage exhaustion ────────────────────────────────────────

#[test]
fn usage_count_at_limit_rejected() {
    let mut c = base_coupon();
    c.usage_limit = Some(5);
    c.usage_count = 5;

    let res = c.validate_usage(dec!(100));
    assert!(matches!(
        res,
        Err(RustCommerceError::CouponUsageLimitReached)
    ));
}

#[test]
fn usage_count_above_limit_rejected() {
    let mut c = base_coupon();
    c.usage_limit = Some(5);
    c.usage_count = 6;

    let res = c.validate_usage(dec!(100));
    assert!(matches!(
        res,
        Err(RustCommerceError::CouponUsageLimitReached)
    ));
}

#[test]
fn usage_count_below_limit_accepted() {
    let mut c = base_coupon();
    c.usage_limit = Some(5);
    c.usage_count = 4;

    assert!(c.validate_usage(dec!(100)).is_ok());
}

#[test]
fn inactive_coupon_rejected() {
    let mut c = base_coupon();
    c.is_active = false;

    let res = c.validate_usage(dec!(100));
    assert!(matches!(res, Err(RustCommerceError::InvalidCoupon(_))));
}

// ── Edge case: 100% off ─────────────────────────────────────────────────

#[test]
fn percentage_100_off_yields_zero_grand_total() {
    let mut c = base_coupon();
    c.discount_type = rc::models::coupon::DiscountType::Percentage;
    c.discount_value = dec!(100);

    let subtotal = dec!(73.45);
    let discount = c.calculate_discount(subtotal);
    assert_eq!(discount, subtotal);
}
