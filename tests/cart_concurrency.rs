//! Concurrency tests for shopping cart mutations.
//!
//! RustCommerce's cart layer relies on Postgres-level serialization
//! semantics:
//!
//!   - `add_item` uses `INSERT ... ON CONFLICT (cart_id, product_id,
//!     variant_id) DO UPDATE SET quantity = quantity + $4` — atomic on
//!     the row, so two parallel adds for the same (cart, product, variant)
//!     must sum (no lost update).
//!   - `update_item_quantity` is an UPDATE … RETURNING.
//!   - `recalculate_totals` is a read-modify-write of the cart row.
//!
//! Without a live Postgres pool we can't observe `ON CONFLICT`, but we
//! CAN simulate the contracts with `Arc<Mutex>` plus `tokio::join!` to:
//!
//!   1. Pin the documented "two concurrent adds → quantities sum" contract.
//!   2. Detect lost-update race conditions in any future pure-Rust replacement.
//!   3. Verify `Cart::recalculate_grand_total` math is consistent.

use std::sync::Arc;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tokio::sync::Mutex;

use rustcommerce::models::cart::{Cart, CartItem};

use chrono::Utc;
use uuid::Uuid;

fn empty_cart() -> Cart {
    Cart {
        id: Uuid::new_v4(),
        customer_id: None,
        session_id: Some("test-session".into()),
        currency: "USD".to_string(),
        subtotal: Decimal::ZERO,
        tax_total: Decimal::ZERO,
        shipping_total: Decimal::ZERO,
        discount_total: Decimal::ZERO,
        grand_total: Decimal::ZERO,
        coupon_code: None,
        notes: None,
        expires_at: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn new_item(cart_id: Uuid, product_id: Uuid, qty: i32, unit_price: Decimal) -> CartItem {
    CartItem {
        id: Uuid::new_v4(),
        cart_id,
        product_id,
        variant_id: None,
        quantity: qty,
        unit_price,
        total_price: unit_price * Decimal::from(qty),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// A test-double cart store that models `ON CONFLICT … DO UPDATE` semantics.
/// One Mutex around the whole row simulates the Postgres row-level lock that
/// makes `INSERT ... ON CONFLICT` atomic.
struct InMemoryCart {
    cart: Cart,
    items: Vec<CartItem>,
}

impl InMemoryCart {
    fn new() -> Self {
        Self {
            cart: empty_cart(),
            items: Vec::new(),
        }
    }

    /// Mirror of `CartRepository::add_item` upsert. Critical: if an item
    /// for the same product already exists, its quantity is summed — never
    /// replaced.
    fn add_item(&mut self, product_id: Uuid, qty: i32, unit_price: Decimal) -> CartItem {
        let cart_id = self.cart.id;
        if let Some(existing) = self
            .items
            .iter_mut()
            .find(|i| i.product_id == product_id && i.variant_id.is_none())
        {
            existing.quantity += qty;
            existing.total_price = existing.unit_price * Decimal::from(existing.quantity);
            existing.updated_at = Utc::now();
            existing.clone()
        } else {
            let item = new_item(cart_id, product_id, qty, unit_price);
            self.items.push(item.clone());
            item
        }
    }

    fn update_quantity(&mut self, item_id: Uuid, qty: i32) -> Option<CartItem> {
        let item = self.items.iter_mut().find(|i| i.id == item_id)?;
        item.quantity = qty;
        item.total_price = item.unit_price * Decimal::from(qty);
        item.updated_at = Utc::now();
        Some(item.clone())
    }

    fn recalculate(&mut self) {
        let subtotal: Decimal = self.items.iter().map(|i| i.total_price).sum();
        self.cart.subtotal = subtotal;
        self.cart.recalculate_grand_total();
    }
}

// ── Two concurrent adds → quantities sum ────────────────────────────────

#[tokio::test]
async fn two_concurrent_adds_to_same_product_sum_quantities() {
    let store = Arc::new(Mutex::new(InMemoryCart::new()));
    let product_id = Uuid::new_v4();
    let unit_price = dec!(9.99);

    let s1 = Arc::clone(&store);
    let s2 = Arc::clone(&store);

    let t1 = tokio::spawn(async move {
        let mut g = s1.lock().await;
        g.add_item(product_id, 2, unit_price);
    });
    let t2 = tokio::spawn(async move {
        let mut g = s2.lock().await;
        g.add_item(product_id, 3, unit_price);
    });

    let _ = tokio::join!(t1, t2);

    let mut s = store.lock().await;
    s.recalculate();

    assert_eq!(s.items.len(), 1, "must dedupe on (cart, product, variant)");
    assert_eq!(s.items[0].quantity, 5, "qty must sum (2 + 3 = 5)");
    assert_eq!(
        s.items[0].total_price,
        unit_price * Decimal::from(5),
        "line total must reflect summed qty × unit_price"
    );
    assert_eq!(
        s.cart.subtotal,
        unit_price * Decimal::from(5),
        "cart subtotal must equal line total"
    );
}

#[tokio::test]
async fn many_concurrent_adds_no_lost_updates() {
    let store = Arc::new(Mutex::new(InMemoryCart::new()));
    let product_id = Uuid::new_v4();
    let unit_price = dec!(1);

    let mut handles = Vec::new();
    for _ in 0..20 {
        let s = Arc::clone(&store);
        let h = tokio::spawn(async move {
            let mut g = s.lock().await;
            g.add_item(product_id, 1, unit_price);
        });
        handles.push(h);
    }
    for h in handles {
        h.await.unwrap();
    }

    let s = store.lock().await;
    assert_eq!(
        s.items[0].quantity, 20,
        "20 concurrent +1 adds must yield qty=20 (no lost updates)"
    );
}

// ── Concurrent add + quantity-change ────────────────────────────────────

#[tokio::test]
async fn concurrent_add_and_qty_update_final_state_deterministic() {
    // Seed the cart with an item of qty 1.
    let store = Arc::new(Mutex::new(InMemoryCart::new()));
    let product_id = Uuid::new_v4();
    let seeded_item_id;
    {
        let mut s = store.lock().await;
        let item = s.add_item(product_id, 1, dec!(10));
        seeded_item_id = item.id;
    }

    // Task A: add 4 more units of the same product → ON CONFLICT bumps qty to 5
    // Task B: set quantity directly to 7
    let s_a = Arc::clone(&store);
    let s_b = Arc::clone(&store);

    let t_a = tokio::spawn(async move {
        let mut g = s_a.lock().await;
        g.add_item(product_id, 4, dec!(10));
    });
    let t_b = tokio::spawn(async move {
        let mut g = s_b.lock().await;
        g.update_quantity(seeded_item_id, 7);
    });

    let _ = tokio::join!(t_a, t_b);

    let s = store.lock().await;
    let q = s.items[0].quantity;

    // Whichever task ran second wins for `update`, but `add` is additive.
    // The valid terminal states therefore are:
    //   - A then B: qty = 7 (B overwrote 1+4)
    //   - B then A: qty = 11 (A added 4 onto B's set-to-7)
    // Both are documented as valid — we just need to be sure we never see a
    // lost-update value like `4` (= add-only) or `1` (no effect).
    assert!(
        q == 7 || q == 11,
        "final qty must be one of the two documented serializations, got {}",
        q
    );
}

// ── Cart.recalculate_grand_total invariants ─────────────────────────────

#[test]
fn grand_total_equals_sum_components_when_positive() {
    let mut c = empty_cart();
    c.subtotal = dec!(100);
    c.tax_total = dec!(8);
    c.shipping_total = dec!(5);
    c.discount_total = dec!(10);

    c.recalculate_grand_total();
    assert_eq!(c.grand_total, dec!(103));
}

#[test]
fn grand_total_floored_at_zero_when_discount_exceeds_subtotal() {
    let mut c = empty_cart();
    c.subtotal = dec!(10);
    c.tax_total = Decimal::ZERO;
    c.shipping_total = Decimal::ZERO;
    c.discount_total = dec!(100);

    c.recalculate_grand_total();
    assert_eq!(c.grand_total, Decimal::ZERO);
}

#[tokio::test]
async fn recalculate_after_concurrent_mutations_is_consistent() {
    let store = Arc::new(Mutex::new(InMemoryCart::new()));
    let p1 = Uuid::new_v4();
    let p2 = Uuid::new_v4();

    let s1 = Arc::clone(&store);
    let s2 = Arc::clone(&store);
    let t1 = tokio::spawn(async move {
        let mut g = s1.lock().await;
        g.add_item(p1, 2, dec!(5));
    });
    let t2 = tokio::spawn(async move {
        let mut g = s2.lock().await;
        g.add_item(p2, 3, dec!(7));
    });
    let _ = tokio::join!(t1, t2);

    let mut s = store.lock().await;
    s.recalculate();

    // 2*5 + 3*7 = 10 + 21 = 31
    assert_eq!(s.cart.subtotal, dec!(31));
    assert_eq!(s.cart.grand_total, dec!(31));
}
