# RustCommerce Business Logic

**Version**: 1.0.0
**Date**: 2026-02-24
**Status**: Approved

---

## Table of Contents

1. [Cart Total Calculation](#1-cart-total-calculation)
2. [Tax Calculation Algorithm](#2-tax-calculation-algorithm)
3. [Shipping Cost Calculation](#3-shipping-cost-calculation)
4. [Inventory Management Rules](#4-inventory-management-rules)
5. [Order Status State Machine](#5-order-status-state-machine)
6. [Coupon Validation Rules](#6-coupon-validation-rules)
7. [Payment Flow (Stripe PaymentIntent)](#7-payment-flow-stripe-paymentintent)
8. [Order Number Generation](#8-order-number-generation)
9. [Price Calculation Precision](#9-price-calculation-precision)
10. [Product Slug Generation](#10-product-slug-generation)
11. [Customer Aggregate Updates](#11-customer-aggregate-updates)
12. [Cart Expiration and Cleanup](#12-cart-expiration-and-cleanup)

---

## 1. Cart Total Calculation

### 1.1 Formula

```
subtotal         = SUM(line_item.unit_price * line_item.quantity)
discount_total   = calculate_discount(subtotal, coupon)
taxable_amount   = subtotal - discount_total   (if discount applies before tax)
tax_total        = calculate_tax(taxable_amount, shipping_cost, address)
shipping_total   = calculate_shipping(cart, address, shipping_method)
grand_total      = subtotal - discount_total + tax_total + shipping_total
```

### 1.2 Calculation Order

The order of operations matters. RustCommerce follows this sequence:

```
Step 1: Calculate line item subtotals
    For each cart item:
        line_total = unit_price * quantity

Step 2: Sum to get cart subtotal
    subtotal = SUM(all line_totals)

Step 3: Apply coupon discount
    discount_total = apply_coupon(subtotal, cart_items, coupon)
    Note: discount cannot exceed subtotal (grand_total >= 0)

Step 4: Calculate tax
    taxable_subtotal = subtotal - discount_total
    tax_total = calculate_tax(taxable_subtotal, address)
    If tax applies to shipping:
        shipping_tax = calculate_shipping_tax(shipping_cost, address)
        tax_total += shipping_tax

Step 5: Calculate shipping
    shipping_total = calculate_shipping(cart, address, method)
    If coupon is free_shipping type:
        shipping_total = 0.00

Step 6: Grand total
    grand_total = subtotal - discount_total + tax_total + shipping_total
    Assert: grand_total >= 0
```

### 1.3 Implementation (Pseudocode)

```rust
pub struct CartTotals {
    pub subtotal: Decimal,
    pub discount_total: Decimal,
    pub tax_total: Decimal,
    pub shipping_total: Decimal,
    pub shipping_tax: Decimal,
    pub grand_total: Decimal,
    pub line_items: Vec<LineItemTotal>,
    pub tax_breakdown: Vec<TaxLineItem>,
}

pub fn calculate_cart_totals(
    cart: &Cart,
    coupon: Option<&Coupon>,
    shipping_method: Option<&ShippingMethod>,
    shipping_address: Option<&Address>,
    tax_rates: &[TaxRate],
) -> Result<CartTotals> {
    // Step 1-2: Line items and subtotal
    let mut subtotal = Decimal::ZERO;
    let mut line_items = Vec::new();
    for item in &cart.items {
        let line_total = item.unit_price * Decimal::from(item.quantity);
        subtotal += line_total;
        line_items.push(LineItemTotal {
            cart_item_id: item.id,
            unit_price: item.unit_price,
            quantity: item.quantity,
            line_total,
            tax_amount: Decimal::ZERO,       // calculated later
            discount_amount: Decimal::ZERO,  // calculated later
        });
    }

    // Step 3: Discount
    let discount_total = if let Some(coupon) = coupon {
        calculate_discount(subtotal, &line_items, coupon)?
    } else {
        Decimal::ZERO
    };
    let discount_total = discount_total.min(subtotal); // Never negative grand total

    // Step 4: Tax
    let taxable_amount = subtotal - discount_total;
    let (tax_total, tax_breakdown) = if let Some(address) = shipping_address {
        calculate_tax(taxable_amount, &line_items, address, tax_rates)?
    } else {
        (Decimal::ZERO, vec![])
    };

    // Step 5: Shipping
    let mut shipping_total = if let Some(method) = shipping_method {
        calculate_shipping_cost(method, &cart, subtotal)?
    } else {
        Decimal::ZERO
    };

    // Free shipping coupon overrides
    if coupon.map(|c| c.discount_type == "free_shipping").unwrap_or(false) {
        shipping_total = Decimal::ZERO;
    }

    // Shipping tax
    let shipping_tax = if let Some(address) = shipping_address {
        calculate_shipping_tax(shipping_total, address, tax_rates)?
    } else {
        Decimal::ZERO
    };

    // Step 6: Grand total
    let grand_total = subtotal - discount_total + tax_total + shipping_tax + shipping_total;
    assert!(grand_total >= Decimal::ZERO);

    Ok(CartTotals {
        subtotal: subtotal.round_dp(2),
        discount_total: discount_total.round_dp(2),
        tax_total: (tax_total + shipping_tax).round_dp(2),
        shipping_total: shipping_total.round_dp(2),
        shipping_tax: shipping_tax.round_dp(2),
        grand_total: grand_total.round_dp(2),
        line_items,
        tax_breakdown,
    })
}
```

### 1.4 Edge Cases

| Scenario | Behavior |
|----------|----------|
| Empty cart | All totals = 0.00 |
| Discount > subtotal | discount_total capped at subtotal, grand_total = tax + shipping |
| No shipping address | Tax and shipping estimates = 0.00 |
| Product price changed since added to cart | Cart GET recalculates using current product price and updates `unit_price` |
| Product deleted/archived | Item marked as unavailable in cart response, excluded from totals |

---

## 2. Tax Calculation Algorithm

### 2.1 Overview

RustCommerce implements zone-based tax calculation with support for:
- Multiple tax rates per location (e.g., state + city + special district)
- Tax priorities for compounding
- Different tax classes per product
- Optional tax on shipping

### 2.2 Tax Rate Matching

Tax rates are matched from most specific to least specific:

```
Priority: City + PostalCode + State + Country > PostalCode + State + Country
         > State + Country > Country > No match (0% tax)
```

**Matching Algorithm:**

```rust
pub fn find_applicable_rates(
    address: &Address,
    tax_class: &str,
    all_rates: &[TaxRate],
) -> Vec<TaxRate> {
    let mut matched_rates: Vec<TaxRate> = all_rates.iter()
        .filter(|r| r.enabled)
        .filter(|r| r.tax_class == tax_class)
        .filter(|r| r.country == address.country)
        .filter(|r| {
            // State filter: NULL matches all states
            r.state.is_none() || r.state.as_deref() == Some(&address.state)
        })
        .filter(|r| {
            // Postal code filter: NULL matches all, supports wildcards
            r.postal_code.is_none() ||
            matches_postal_pattern(r.postal_code.as_deref().unwrap(), &address.postal_code)
        })
        .filter(|r| {
            // City filter: NULL matches all
            r.city.is_none() ||
            r.city.as_deref().map(|c| c.to_lowercase()) == Some(address.city.to_lowercase())
        })
        .cloned()
        .collect();

    // Sort by specificity (most specific first) then by priority
    matched_rates.sort_by(|a, b| {
        let specificity_a = specificity_score(a);
        let specificity_b = specificity_score(b);
        specificity_b.cmp(&specificity_a).then(a.priority.cmp(&b.priority))
    });

    matched_rates
}

fn specificity_score(rate: &TaxRate) -> u8 {
    let mut score = 0;
    if rate.city.is_some() { score += 4; }
    if rate.postal_code.is_some() { score += 3; }
    if rate.state.is_some() { score += 2; }
    // Country is always present, so +1 implicitly
    score
}

fn matches_postal_pattern(pattern: &str, postal_code: &str) -> bool {
    if pattern.contains('*') {
        let prefix = pattern.trim_end_matches('*');
        postal_code.starts_with(prefix)
    } else {
        pattern == postal_code
    }
}
```

### 2.3 Tax Calculation with Priorities and Compounding

Tax rates have a `priority` field. Rates at the same priority level are **summed**. Rates at different priority levels are **compounded** (applied sequentially).

```
Example: New York City
  Priority 1: NY State Tax (4.00%)      -- non-compound
  Priority 1: NYC Local Tax (4.50%)     -- non-compound
  Priority 2: MTA Surcharge (0.375%)    -- compound

For a $100 item:
  Step 1 (Priority 1): sum rates = 4.00% + 4.50% = 8.50%
    Tax at priority 1 = $100.00 * 0.085 = $8.50

  Step 2 (Priority 2): compound on (subtotal + priority 1 tax)
    Compound base = $100.00 + $8.50 = $108.50
    Tax at priority 2 = $108.50 * 0.00375 = $0.41 (rounded)

  Total tax = $8.50 + $0.41 = $8.91
```

**Implementation:**

```rust
pub fn calculate_tax(
    taxable_amount: Decimal,
    line_items: &[LineItemTotal],
    address: &Address,
    all_rates: &[TaxRate],
) -> Result<(Decimal, Vec<TaxLineItem>)> {
    let mut total_tax = Decimal::ZERO;
    let mut tax_breakdown = Vec::new();

    // Group items by tax class
    // For simplicity, assume all items use the same tax class in this example
    let tax_class = "standard";

    let rates = find_applicable_rates(address, tax_class, all_rates);
    if rates.is_empty() {
        return Ok((Decimal::ZERO, vec![]));
    }

    // Group rates by priority
    let mut priority_groups: BTreeMap<i32, Vec<&TaxRate>> = BTreeMap::new();
    for rate in &rates {
        priority_groups.entry(rate.priority).or_default().push(rate);
    }

    let mut running_base = taxable_amount;

    for (priority, group_rates) in &priority_groups {
        let is_compound = group_rates.iter().any(|r| r.compound);

        // Sum all rates at this priority level
        let combined_rate: Decimal = group_rates.iter()
            .map(|r| r.rate)
            .sum();

        let base = if is_compound {
            running_base  // Includes tax from previous priorities
        } else {
            taxable_amount  // Original taxable amount
        };

        let priority_tax = (base * combined_rate).round_dp(2);
        total_tax += priority_tax;
        running_base = taxable_amount + total_tax;

        // Record breakdown
        for rate in group_rates {
            let individual_tax = (base * rate.rate).round_dp(2);
            tax_breakdown.push(TaxLineItem {
                name: rate.name.clone(),
                rate: rate.rate,
                amount: individual_tax,
                compound: rate.compound,
            });
        }
    }

    Ok((total_tax, tax_breakdown))
}
```

### 2.4 Tax on Shipping

If a tax rate has `shipping = true`, it also applies to the shipping cost:

```rust
pub fn calculate_shipping_tax(
    shipping_cost: Decimal,
    address: &Address,
    all_rates: &[TaxRate],
) -> Result<Decimal> {
    if shipping_cost == Decimal::ZERO {
        return Ok(Decimal::ZERO);
    }

    let rates = find_applicable_rates(address, "standard", all_rates);
    let shipping_rates: Vec<&TaxRate> = rates.iter()
        .filter(|r| r.shipping)
        .collect();

    let shipping_tax_rate: Decimal = shipping_rates.iter()
        .map(|r| r.rate)
        .sum();

    Ok((shipping_cost * shipping_tax_rate).round_dp(2))
}
```

### 2.5 Tax Configuration Options

| Setting | Values | Effect |
|---------|--------|--------|
| `tax_enabled` | `true/false` | Master switch. If false, all tax = 0. |
| `prices_include_tax` | `true/false` | If true, displayed prices include tax; tax is extracted rather than added. |
| `calculate_tax_on` | `shipping_address` / `billing_address` / `store_address` | Which address to use for tax calculation. |

**Prices Include Tax (tax-inclusive pricing):**

When `prices_include_tax = true`:
```
display_price = $100.00 (includes tax)
tax_rate = 8.50%
actual_price = $100.00 / (1 + 0.085) = $92.17
tax_amount = $100.00 - $92.17 = $7.83
```

---

## 3. Shipping Cost Calculation

### 3.1 Zone Matching

Before calculating shipping cost, determine the applicable shipping zone:

```rust
pub fn find_shipping_zone(
    address: &Address,
    zones: &[ShippingZone],
) -> Option<&ShippingZone> {
    // Priority: postal code match > region match > country match > default
    zones.iter()
        .filter(|z| !z.is_default) // Check non-default zones first
        .find(|z| {
            // Check postal code patterns
            if !z.postal_codes.is_empty() {
                return z.postal_codes.iter().any(|pc|
                    matches_postal_pattern(pc, &address.postal_code)
                );
            }
            // Check regions (format: "US-NY")
            if !z.regions.is_empty() {
                let region_code = format!("{}-{}", address.country, address.state);
                return z.regions.contains(&region_code);
            }
            // Check countries
            z.countries.contains(&address.country)
        })
        .or_else(|| {
            // Fall back to default zone
            zones.iter().find(|z| z.is_default)
        })
}
```

### 3.2 Shipping Method Types

#### 3.2.1 Flat Rate

Fixed cost regardless of cart contents.

```rust
fn calculate_flat_rate(method: &ShippingMethod, subtotal: Decimal) -> Decimal {
    // Check free shipping threshold
    if let Some(threshold) = method.free_threshold {
        if subtotal >= threshold {
            return Decimal::ZERO;
        }
    }
    method.cost
}
```

**Example:**
- Flat rate: $5.99
- Free threshold: $100.00
- Cart subtotal: $150.00 -> Shipping = $0.00
- Cart subtotal: $50.00 -> Shipping = $5.99

#### 3.2.2 Free Shipping

Always $0.00. Typically used as a promotional shipping method in a zone.

```rust
fn calculate_free_shipping(_method: &ShippingMethod) -> Decimal {
    Decimal::ZERO
}
```

#### 3.2.3 Weight-Based

Cost varies based on total cart weight.

```rust
fn calculate_weight_based(
    method: &ShippingMethod,
    cart: &Cart,
    products: &HashMap<Uuid, Product>,
) -> Result<Decimal> {
    let settings = &method.settings; // JSONB parsed
    let base_cost: Decimal = settings.get("base_cost")
        .and_then(|v| v.as_str())
        .and_then(|s| Decimal::from_str(s).ok())
        .unwrap_or(Decimal::ZERO);

    let per_kg: Decimal = settings.get("per_kg")
        .and_then(|v| v.as_str())
        .and_then(|s| Decimal::from_str(s).ok())
        .unwrap_or(Decimal::ZERO);

    // Calculate total cart weight
    let mut total_weight = Decimal::ZERO;
    for item in &cart.items {
        let product = products.get(&item.product_id)
            .ok_or(Error::NotFound("Product not found".into()))?;
        let item_weight = product.weight.unwrap_or(Decimal::ZERO);
        total_weight += item_weight * Decimal::from(item.quantity);
    }

    // Check weight limits
    if let Some(min) = method.min_weight {
        if total_weight < min {
            return Err(Error::Validation("Cart below minimum weight for this shipping method".into()));
        }
    }
    if let Some(max) = method.max_weight {
        if total_weight > max {
            return Err(Error::Validation("Cart exceeds maximum weight for this shipping method".into()));
        }
    }

    let cost = base_cost + (per_kg * total_weight);

    // Check free threshold
    // (weight-based can also have a free threshold based on subtotal)
    if let Some(threshold) = method.free_threshold {
        let subtotal = cart.items.iter()
            .map(|i| i.unit_price * Decimal::from(i.quantity))
            .sum::<Decimal>();
        if subtotal >= threshold {
            return Ok(Decimal::ZERO);
        }
    }

    Ok(cost.round_dp(2))
}
```

**Example:**
- Base cost: $10.00, Per kg: $2.50
- Cart: 2 items, 1.5kg each = 3.0kg total
- Shipping = $10.00 + ($2.50 * 3.0) = $17.50

#### 3.2.4 Price-Based (Tiered)

Cost determined by order subtotal tiers.

```rust
fn calculate_price_based(
    method: &ShippingMethod,
    subtotal: Decimal,
) -> Result<Decimal> {
    let settings = &method.settings;
    let tiers: Vec<PriceTier> = serde_json::from_value(
        settings.get("tiers").cloned().unwrap_or_default()
    )?;

    // Find the matching tier
    for tier in &tiers {
        let min = tier.min;
        let max = tier.max.unwrap_or(Decimal::MAX);
        if subtotal >= min && subtotal <= max {
            return Ok(tier.cost);
        }
    }

    // No matching tier - use the last tier as fallback
    tiers.last()
        .map(|t| t.cost)
        .ok_or(Error::Internal("No shipping tiers configured".into()))
}
```

**Example tiers:**
| Subtotal Range | Shipping Cost |
|----------------|---------------|
| $0.00 - $49.99 | $9.99 |
| $50.00 - $99.99 | $5.99 |
| $100.00+ | $0.00 (free) |

### 3.3 Shipping Calculation Master Function

```rust
pub fn calculate_shipping_cost(
    method: &ShippingMethod,
    cart: &Cart,
    subtotal: Decimal,
) -> Result<Decimal> {
    match method.method_type.as_str() {
        "flat_rate" => Ok(calculate_flat_rate(method, subtotal)),
        "free_shipping" => Ok(calculate_free_shipping(method)),
        "weight_based" => calculate_weight_based(method, cart, &product_map),
        "price_based" => calculate_price_based(method, subtotal),
        _ => Err(Error::Internal(format!("Unknown shipping method type: {}", method.method_type))),
    }
}
```

---

## 4. Inventory Management Rules

### 4.1 Stock Tracking

Stock is tracked at two levels:

| Product Type | Stock Tracked On |
|-------------|-----------------|
| Simple | `rc_products.stock_quantity` |
| Variable | `rc_product_variants.stock_quantity` (each variant independently) |
| Grouped | Each component product independently |
| Digital | No stock tracking (unlimited) |

For variable products, `rc_products.stock_quantity` is a denormalized sum of all variant quantities.

### 4.2 Stock Status Derivation

```rust
pub fn derive_stock_status(
    stock_quantity: i32,
    low_stock_threshold: i32,
    backorders_allowed: bool,
) -> StockStatus {
    if stock_quantity > low_stock_threshold {
        StockStatus::InStock
    } else if stock_quantity > 0 {
        StockStatus::InStock  // Low stock (triggers alert, but still in stock)
    } else if backorders_allowed {
        StockStatus::OnBackorder
    } else {
        StockStatus::OutOfStock
    }
}
```

### 4.3 Stock Reservation During Checkout

When a customer initiates checkout, stock is temporarily reserved to prevent overselling:

```
Timeline:
  T+0:00  Customer clicks "Checkout" -> POST /checkout/init
          Stock reserved for all cart items
          Reservation expires at T+10:00

  T+0:30  Customer enters shipping address (reservation still active)

  T+5:00  Customer completes payment -> POST /checkout/complete
          Reservation converted to permanent decrement
          Order created

  -- OR --

  T+10:00 Customer abandoned checkout
          Reservation expires
          Background job releases reserved stock
```

**Reservation Logic:**

```rust
pub async fn reserve_stock(
    pool: &PgPool,
    checkout_session_id: Uuid,
    cart_items: &[CartItem],
    hold_minutes: i32,
) -> Result<()> {
    let expires_at = Utc::now() + Duration::minutes(hold_minutes as i64);

    for item in cart_items {
        // Check available stock (actual - active reservations)
        let available = get_available_stock(pool, item.product_id, item.variant_id).await?;

        if available < item.quantity as i32 {
            return Err(Error::Validation(format!(
                "Insufficient stock for product. Available: {}, Requested: {}",
                available, item.quantity
            )));
        }

        // Create reservation
        sqlx::query!(
            "INSERT INTO rc_stock_reservations
             (checkout_session_id, product_id, variant_id, quantity, expires_at)
             VALUES ($1, $2, $3, $4, $5)",
            checkout_session_id, item.product_id, item.variant_id,
            item.quantity as i32, expires_at
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn get_available_stock(
    pool: &PgPool,
    product_id: Uuid,
    variant_id: Option<Uuid>,
) -> Result<i32> {
    let actual_stock = if let Some(vid) = variant_id {
        sqlx::query_scalar!(
            "SELECT stock_quantity FROM rc_product_variants WHERE id = $1", vid
        ).fetch_one(pool).await?
    } else {
        sqlx::query_scalar!(
            "SELECT stock_quantity FROM rc_products WHERE id = $1", product_id
        ).fetch_one(pool).await?
    };

    let reserved: i32 = sqlx::query_scalar!(
        "SELECT COALESCE(SUM(quantity), 0) FROM rc_stock_reservations
         WHERE product_id = $1
           AND ($2::uuid IS NULL OR variant_id = $2)
           AND status = 'active'
           AND expires_at > now()",
        product_id, variant_id
    ).fetch_one(pool).await?;

    Ok(actual_stock - reserved)
}
```

### 4.4 Stock Decrement on Order Completion

When an order is confirmed (payment successful):

```rust
pub async fn decrement_stock(pool: &PgPool, order_items: &[OrderItem]) -> Result<()> {
    for item in order_items {
        if let Some(variant_id) = item.variant_id {
            // Decrement variant stock
            sqlx::query!(
                "UPDATE rc_product_variants
                 SET stock_quantity = stock_quantity - $1,
                     stock_status = CASE
                         WHEN stock_quantity - $1 <= 0 THEN 'out_of_stock'
                         ELSE stock_status
                     END,
                     updated_at = now()
                 WHERE id = $2 AND stock_quantity >= $1",
                item.quantity, variant_id
            ).execute(pool).await?;

            // Update parent product aggregate stock
            sqlx::query!(
                "UPDATE rc_products
                 SET stock_quantity = (
                     SELECT COALESCE(SUM(stock_quantity), 0)
                     FROM rc_product_variants WHERE product_id = $1
                 ),
                 updated_at = now()
                 WHERE id = $1",
                item.product_id
            ).execute(pool).await?;
        } else {
            // Decrement product stock directly
            sqlx::query!(
                "UPDATE rc_products
                 SET stock_quantity = stock_quantity - $1,
                     stock_status = CASE
                         WHEN stock_quantity - $1 <= 0 THEN 'out_of_stock'
                         ELSE stock_status
                     END,
                     updated_at = now()
                 WHERE id = $2 AND stock_quantity >= $1",
                item.quantity, item.product_id
            ).execute(pool).await?;
        }

        // Check for low stock alert
        check_low_stock_alert(pool, item.product_id, item.variant_id).await?;
    }

    Ok(())
}
```

### 4.5 Stock Restoration on Cancellation/Refund

When an order is cancelled or items are refunded with restock:

```rust
pub async fn restore_stock(
    pool: &PgPool,
    items: &[(Uuid, Option<Uuid>, i32)], // (product_id, variant_id, quantity)
) -> Result<()> {
    for (product_id, variant_id, quantity) in items {
        if let Some(vid) = variant_id {
            sqlx::query!(
                "UPDATE rc_product_variants
                 SET stock_quantity = stock_quantity + $1,
                     stock_status = 'in_stock',
                     updated_at = now()
                 WHERE id = $2",
                quantity, vid
            ).execute(pool).await?;

            // Update parent product aggregate
            sqlx::query!(
                "UPDATE rc_products
                 SET stock_quantity = (
                     SELECT COALESCE(SUM(stock_quantity), 0)
                     FROM rc_product_variants WHERE product_id = $1
                 ),
                 stock_status = 'in_stock',
                 updated_at = now()
                 WHERE id = $1",
                product_id
            ).execute(pool).await?;
        } else {
            sqlx::query!(
                "UPDATE rc_products
                 SET stock_quantity = stock_quantity + $1,
                     stock_status = 'in_stock',
                     updated_at = now()
                 WHERE id = $2",
                quantity, product_id
            ).execute(pool).await?;
        }
    }

    Ok(())
}
```

### 4.6 Low Stock Alerts

```rust
async fn check_low_stock_alert(
    pool: &PgPool,
    product_id: Uuid,
    variant_id: Option<Uuid>,
) -> Result<()> {
    let (stock_quantity, threshold) = if let Some(vid) = variant_id {
        let variant = get_variant(pool, vid).await?;
        let product = get_product(pool, product_id).await?;
        (variant.stock_quantity, product.low_stock_threshold)
    } else {
        let product = get_product(pool, product_id).await?;
        (product.stock_quantity, product.low_stock_threshold)
    };

    if stock_quantity <= threshold && stock_quantity > 0 {
        // Fire low stock hook
        hooks.do_action("rustcommerce_stock_low", &json!({
            "product_id": product_id,
            "variant_id": variant_id,
            "stock_quantity": stock_quantity,
            "threshold": threshold,
        })).await?;
    }

    if stock_quantity == 0 {
        // Fire stock depleted hook
        hooks.do_action("rustcommerce_stock_depleted", &json!({
            "product_id": product_id,
            "variant_id": variant_id,
        })).await?;
    }

    Ok(())
}
```

### 4.7 Backorder Rules

When `backorders_allowed = true` for a product:

| Stock | Add to Cart | Checkout | Display |
|-------|:-----------:|:--------:|---------|
| > 0 | Allowed | Allowed | "In Stock" |
| = 0 | Allowed | Allowed | "Available on Backorder" |
| < 0 (oversold) | Allowed | Allowed | "Available on Backorder" |

When `backorders_allowed = false`:

| Stock | Add to Cart | Checkout | Display |
|-------|:-----------:|:--------:|---------|
| > 0 | Allowed (up to stock) | Allowed | "In Stock" |
| = 0 | Blocked | Blocked | "Out of Stock" |

---

## 5. Order Status State Machine

### 5.1 Status Definitions

| Status | Description | Triggered By |
|--------|-------------|-------------|
| `pending` | Order created, awaiting payment | Checkout initiation |
| `confirmed` | Payment received, order accepted | Stripe webhook (payment_intent.succeeded) |
| `processing` | Order being prepared/packed | Admin action |
| `shipped` | Order shipped, tracking available | Admin action (with tracking number) |
| `delivered` | Order delivered to customer | Admin action or delivery confirmation |
| `cancelled` | Order cancelled before shipment | Admin or customer action |
| `refunded` | Full refund issued | Admin action (via refund endpoint) |

### 5.2 Valid State Transitions

```
                    ┌─────────────────────────────┐
                    │                             │
                    ▼                             │
    ┌─────────┐  payment  ┌───────────┐  admin  ┌──────────────┐
    │ pending │──────────►│ confirmed │────────►│  processing  │
    └────┬────┘  success  └─────┬─────┘         └──────┬───────┘
         │                      │                      │
         │ cancel               │ cancel               │ ship
         │                      │                      │
         ▼                      ▼                      ▼
    ┌───────────┐         ┌───────────┐         ┌───────────┐
    │ cancelled │         │ cancelled │         │  shipped   │
    └───────────┘         └───────────┘         └─────┬─────┘
                                                      │
                                                      │ deliver
                                                      │
                                                      ▼
                                                ┌───────────┐
                                                │ delivered  │
                                                └─────┬─────┘
                                                      │
                                                      │ refund
                                                      ▼
                                                ┌───────────┐
                                                │  refunded  │
                                                └───────────┘
```

**Transition Matrix:**

| From \ To | pending | confirmed | processing | shipped | delivered | cancelled | refunded |
|-----------|:-------:|:---------:|:----------:|:-------:|:---------:|:---------:|:--------:|
| **pending** | - | YES | - | - | - | YES | - |
| **confirmed** | - | - | YES | - | - | YES | YES |
| **processing** | - | - | - | YES | - | YES | YES |
| **shipped** | - | - | - | - | YES | - | YES |
| **delivered** | - | - | - | - | - | - | YES |
| **cancelled** | - | - | - | - | - | - | - |
| **refunded** | - | - | - | - | - | - | - |

### 5.3 Transition Enforcement

```rust
pub fn validate_status_transition(
    current: &OrderStatus,
    target: &OrderStatus,
) -> Result<()> {
    let valid_transitions: HashMap<&str, Vec<&str>> = HashMap::from([
        ("pending",    vec!["confirmed", "cancelled"]),
        ("confirmed",  vec!["processing", "cancelled", "refunded"]),
        ("processing", vec!["shipped", "cancelled", "refunded"]),
        ("shipped",    vec!["delivered", "refunded"]),
        ("delivered",  vec!["refunded"]),
        ("cancelled",  vec![]),
        ("refunded",   vec![]),
    ]);

    let allowed = valid_transitions.get(current.as_str())
        .ok_or(Error::Internal(format!("Unknown order status: {}", current)))?;

    if !allowed.contains(&target.as_str()) {
        return Err(Error::Validation(format!(
            "Cannot transition from '{}' to '{}'. Valid transitions: {:?}",
            current, target, allowed
        )));
    }

    Ok(())
}
```

### 5.4 Side Effects on Transition

| Transition | Side Effects |
|-----------|-------------|
| `pending` -> `confirmed` | Log payment, fire `order_confirmed` hook, send order confirmation email |
| `confirmed` -> `processing` | Fire `order_processing` hook |
| `processing` -> `shipped` | Record tracking number, fire `order_shipped` hook, send shipping notification email |
| `shipped` -> `delivered` | Set `completed_at`, fire `order_delivered` hook, update customer aggregates, send delivery confirmation email |
| `* -> cancelled` | Set `cancelled_at`, restore stock (if not already shipped), fire `order_cancelled` hook, send cancellation email |
| `* -> refunded` | Process Stripe refund, restore stock (if requested), update payment_status, fire `refund_issued` hook, send refund email |

### 5.5 Payment Status Transitions

Separate from order status, the payment status tracks:

| Payment Status | Description |
|----------------|-------------|
| `unpaid` | No payment received yet |
| `paid` | Full payment received |
| `partially_refunded` | Some amount refunded |
| `refunded` | Full amount refunded |
| `failed` | Payment attempt failed |

```
unpaid → paid → partially_refunded → refunded
unpaid → failed
```

---

## 6. Coupon Validation Rules

### 6.1 Validation Checks (in Order)

When a coupon code is submitted, validate in this order:

```rust
pub fn validate_coupon(
    coupon: &Coupon,
    cart: &Cart,
    customer_id: Option<Uuid>,
    customer_usage_count: i32,
) -> Result<CouponValidation> {
    // 1. Coupon exists? (handled by lookup)

    // 2. Coupon enabled?
    if !coupon.enabled {
        return Err(coupon_error("COUPON_DISABLED", "This coupon is not active"));
    }

    // 3. Started?
    if let Some(starts_at) = coupon.starts_at {
        if Utc::now() < starts_at {
            return Err(coupon_error("COUPON_NOT_STARTED", "This coupon is not yet active"));
        }
    }

    // 4. Expired?
    if let Some(expires_at) = coupon.expires_at {
        if Utc::now() > expires_at {
            return Err(coupon_error("COUPON_EXPIRED", "This coupon has expired"));
        }
    }

    // 5. Global usage limit reached?
    if let Some(limit) = coupon.usage_limit {
        if coupon.usage_count >= limit {
            return Err(coupon_error("COUPON_USAGE_LIMIT", "This coupon has reached its usage limit"));
        }
    }

    // 6. Per-user usage limit reached?
    if let Some(per_user_limit) = coupon.usage_limit_per_user {
        if customer_usage_count >= per_user_limit as i32 {
            return Err(coupon_error("COUPON_USER_LIMIT", "You have already used this coupon the maximum number of times"));
        }
    }

    // 7. Minimum spend met?
    let cart_subtotal = calculate_cart_subtotal(cart);
    if let Some(min_spend) = coupon.minimum_spend {
        if cart_subtotal < min_spend {
            return Err(coupon_error("MINIMUM_SPEND_NOT_MET",
                &format!("Minimum spend of ${:.2} required", min_spend)));
        }
    }

    // 8. Maximum spend check
    if let Some(max_spend) = coupon.maximum_spend {
        if cart_subtotal > max_spend {
            return Err(coupon_error("MAXIMUM_SPEND_EXCEEDED",
                &format!("This coupon is valid for orders up to ${:.2}", max_spend)));
        }
    }

    // 9. Product restrictions?
    if !coupon.product_ids.is_empty() {
        let cart_product_ids: HashSet<Uuid> = cart.items.iter()
            .map(|i| i.product_id).collect();
        let qualifying_products: HashSet<&Uuid> = coupon.product_ids.iter()
            .filter(|id| cart_product_ids.contains(id))
            .collect();
        if qualifying_products.is_empty() {
            return Err(coupon_error("COUPON_PRODUCT_MISMATCH",
                "This coupon does not apply to any items in your cart"));
        }
    }

    // 10. Category restrictions?
    if !coupon.category_ids.is_empty() {
        // Check if any cart item belongs to a qualifying category
        let has_qualifying = check_cart_categories(cart, &coupon.category_ids);
        if !has_qualifying {
            return Err(coupon_error("COUPON_CATEGORY_MISMATCH",
                "This coupon does not apply to the categories in your cart"));
        }
    }

    // 11. Excluded products?
    // (handled during discount calculation, not validation)

    Ok(CouponValidation {
        valid: true,
        estimated_discount: estimate_discount(cart_subtotal, coupon),
    })
}
```

### 6.2 Discount Calculation by Type

```rust
pub fn calculate_discount(
    subtotal: Decimal,
    line_items: &[LineItemTotal],
    coupon: &Coupon,
) -> Result<Decimal> {
    match coupon.discount_type.as_str() {
        "percentage" => {
            // Apply percentage to qualifying items
            let qualifying_subtotal = get_qualifying_subtotal(line_items, coupon);
            let discount = (qualifying_subtotal * coupon.discount_value / Decimal::from(100))
                .round_dp(2);
            Ok(discount.min(subtotal))  // Never exceed subtotal
        }

        "fixed_cart" => {
            // Fixed amount off the entire cart
            Ok(coupon.discount_value.min(subtotal))
        }

        "fixed_product" => {
            // Fixed amount off each qualifying item
            let qualifying_items = get_qualifying_items(line_items, coupon);
            let discount: Decimal = qualifying_items.iter()
                .map(|item| {
                    let per_item_discount = coupon.discount_value.min(item.unit_price);
                    per_item_discount * Decimal::from(item.quantity)
                })
                .sum();
            Ok(discount.min(subtotal))
        }

        "free_shipping" => {
            // Discount is zero on subtotal; shipping is zeroed separately
            Ok(Decimal::ZERO)
        }

        _ => Err(Error::Internal(format!("Unknown discount type: {}", coupon.discount_type))),
    }
}

fn get_qualifying_subtotal(line_items: &[LineItemTotal], coupon: &Coupon) -> Decimal {
    line_items.iter()
        .filter(|item| is_item_qualifying(item, coupon))
        .map(|item| item.line_total)
        .sum()
}

fn is_item_qualifying(item: &LineItemTotal, coupon: &Coupon) -> bool {
    // Excluded products take precedence
    if coupon.excluded_product_ids.contains(&item.product_id) {
        return false;
    }

    // If product_ids is set, item must be in the list
    if !coupon.product_ids.is_empty() {
        return coupon.product_ids.contains(&item.product_id);
    }

    // If category_ids is set, item must be in a qualifying category
    // (requires category lookup - simplified here)
    if !coupon.category_ids.is_empty() {
        return item_in_categories(&item.product_id, &coupon.category_ids);
    }

    // No restrictions - all items qualify
    true
}
```

### 6.3 Coupon Usage Recording

After successful order completion:

```rust
pub async fn record_coupon_usage(
    pool: &PgPool,
    coupon_id: Uuid,
    customer_id: Option<Uuid>,
    order_id: Uuid,
) -> Result<()> {
    // Insert usage record
    sqlx::query!(
        "INSERT INTO rc_coupon_usage (coupon_id, customer_id, order_id)
         VALUES ($1, $2, $3)",
        coupon_id, customer_id, order_id
    ).execute(pool).await?;

    // Increment global usage count (atomic)
    sqlx::query!(
        "UPDATE rc_coupons SET usage_count = usage_count + 1, updated_at = now()
         WHERE id = $1",
        coupon_id
    ).execute(pool).await?;

    Ok(())
}
```

---

## 7. Payment Flow (Stripe PaymentIntent)

### 7.1 Lifecycle Overview

```
┌──────────┐    ┌──────────────┐    ┌───────────────┐    ┌──────────────┐
│  Client  │    │ RustCommerce │    │    Stripe     │    │   Webhook    │
└────┬─────┘    └──────┬───────┘    └──────┬────────┘    └──────┬───────┘
     │                 │                   │                    │
     │  1. POST /checkout/payment-intent   │                    │
     │ ────────────────►                   │                    │
     │                 │                   │                    │
     │                 │  2. Create        │                    │
     │                 │  PaymentIntent    │                    │
     │                 │ ─────────────────►│                    │
     │                 │                   │                    │
     │                 │  3. PI created    │                    │
     │                 │  (client_secret)  │                    │
     │                 │ ◄─────────────────│                    │
     │                 │                   │                    │
     │  4. Return client_secret            │                    │
     │ ◄────────────────                   │                    │
     │                 │                   │                    │
     │  5. stripe.confirmPayment()         │                    │
     │     (Stripe.js on frontend)         │                    │
     │ ────────────────────────────────────►                    │
     │                 │                   │                    │
     │                 │                   │  6. payment_intent │
     │                 │                   │  .succeeded        │
     │                 │                   │ ──────────────────►│
     │                 │                   │                    │
     │                 │  7. Webhook handler:                   │
     │                 │  - Verify signature                    │
     │                 │  - Find order by PI ID                 │
     │                 │  - Create payment record               │
     │                 │  - Update order status → confirmed     │
     │                 │  - Convert stock reservations          │
     │                 │  - Decrement actual stock              │
     │                 │  - Update customer aggregates          │
     │                 │  - Fire order_created hook             │
     │                 │  - Send confirmation email             │
     │                 │ ◄──────────────────────────────────────│
     │                 │                   │                    │
     │  8. POST /checkout/complete         │                    │
     │ ────────────────►                   │                    │
     │                 │                   │                    │
     │  9. 201 Created { order }           │                    │
     │ ◄────────────────                   │                    │
```

### 7.2 PaymentIntent Creation

```rust
pub async fn create_payment_intent(
    stripe_client: &stripe::Client,
    checkout: &CheckoutSession,
    totals: &CartTotals,
    customer_email: &str,
) -> Result<stripe::PaymentIntent> {
    // Convert to cents (Stripe uses smallest currency unit)
    let amount_cents = (totals.grand_total * Decimal::from(100))
        .to_i64()
        .ok_or(Error::Internal("Invalid amount".into()))?;

    let mut params = stripe::CreatePaymentIntent::new(amount_cents, stripe::Currency::USD);
    params.description = Some("RustCommerce Order");
    params.receipt_email = Some(customer_email);
    params.metadata = Some(HashMap::from([
        ("checkout_session_id".to_string(), checkout.id.to_string()),
        ("order_id".to_string(), "pending".to_string()), // Set after order creation
    ]));
    params.automatic_payment_methods = Some(
        stripe::CreatePaymentIntentAutomaticPaymentMethods {
            enabled: true,
            allow_redirects: None,
        }
    );

    let pi = stripe::PaymentIntent::create(&stripe_client, params).await
        .map_err(|e| Error::Plugin(format!("Stripe error: {}", e)))?;

    Ok(pi)
}
```

### 7.3 Webhook Event Processing

```rust
async fn handle_payment_succeeded(
    state: &AppState,
    event: &stripe::Event,
) -> Result<()> {
    let payment_intent = extract_payment_intent(event)?;
    let checkout_session_id = payment_intent.metadata.get("checkout_session_id")
        .ok_or(Error::Internal("Missing checkout_session_id in PI metadata".into()))?;

    // Idempotency check
    if is_event_processed(state, &event.id).await {
        tracing::info!("Duplicate event {}, skipping", event.id);
        return Ok(());
    }

    // Begin transaction
    let mut tx = state.db.pool.begin().await?;

    // 1. Create order from checkout session
    let order = create_order_from_checkout(&mut tx, checkout_session_id).await?;

    // 2. Create payment record
    let payment = create_payment_record(&mut tx, &order, &payment_intent).await?;

    // 3. Update order status to confirmed
    update_order_status(&mut tx, order.id, "confirmed", None).await?;

    // 4. Convert stock reservations
    convert_stock_reservations(&mut tx, checkout_session_id, order.id).await?;

    // 5. Decrement actual stock
    decrement_stock(&mut tx, &order.items).await?;

    // 6. Mark cart as converted
    mark_cart_converted(&mut tx, &checkout_session_id).await?;

    // 7. Update customer aggregates
    update_customer_aggregates(&mut tx, order.customer_id, &order).await?;

    // 8. Record coupon usage (if applicable)
    if let Some(ref coupon_code) = order.coupon_code {
        record_coupon_usage(&mut tx, coupon_code, order.customer_id, order.id).await?;
    }

    // Commit transaction
    tx.commit().await?;

    // 9. Fire hooks (outside transaction)
    state.hooks.do_action("rustcommerce_order_created", &json!({
        "order_id": order.id,
        "order_number": order.order_number,
        "grand_total": order.grand_total.to_string(),
        "customer_email": order.billing_address["email"],
    })).await?;

    state.hooks.do_action("rustcommerce_payment_completed", &json!({
        "order_id": order.id,
        "payment_id": payment.id,
        "amount": payment.amount.to_string(),
    })).await?;

    // 10. Send confirmation email
    send_order_confirmation_email(state, &order).await?;

    // Mark event as processed
    mark_event_processed(state, &event.id).await;

    Ok(())
}
```

### 7.4 Refund Processing

```rust
pub async fn process_refund(
    stripe_client: &stripe::Client,
    pool: &PgPool,
    order: &Order,
    amount: Option<Decimal>,  // None = full refund
    reason: &str,
    restock_items: Option<Vec<RestockItem>>,
) -> Result<Refund> {
    let payment = get_order_payment(pool, order.id).await?;

    // Calculate refund amount
    let refund_amount = amount.unwrap_or(order.grand_total);
    let refund_cents = (refund_amount * Decimal::from(100)).to_i64().unwrap();

    // Process refund via Stripe
    let mut params = stripe::CreateRefund::new();
    params.payment_intent = Some(stripe::PaymentIntentId::from_str(&order.stripe_payment_intent_id.as_ref().unwrap())?);
    params.amount = Some(refund_cents);
    params.reason = Some(stripe::RefundReason::RequestedByCustomer);

    let stripe_refund = stripe::Refund::create(&stripe_client, params).await
        .map_err(|e| Error::Plugin(format!("Stripe refund error: {}", e)))?;

    // Begin transaction
    let mut tx = pool.begin().await?;

    // Record refund
    let refund = sqlx::query_as!(Refund,
        "INSERT INTO rc_refunds (order_id, payment_id, amount, currency, reason, status, transaction_id, refunded_by)
         VALUES ($1, $2, $3, $4, $5, 'completed', $6, $7)
         RETURNING *",
        order.id, payment.id, refund_amount, order.currency, reason,
        stripe_refund.id.to_string(), admin_user_id
    ).fetch_one(&mut *tx).await?;

    // Update order payment status
    let new_payment_status = if refund_amount >= order.grand_total {
        "refunded"
    } else {
        "partially_refunded"
    };
    sqlx::query!(
        "UPDATE rc_orders SET payment_status = $1, updated_at = now() WHERE id = $2",
        new_payment_status, order.id
    ).execute(&mut *tx).await?;

    // Restock items if requested
    if let Some(items) = restock_items {
        for item in &items {
            restore_stock_item(&mut tx, item).await?;
        }
    }

    // If full refund, update order status
    if refund_amount >= order.grand_total {
        update_order_status(&mut tx, order.id, "refunded", Some(reason)).await?;
    }

    tx.commit().await?;

    Ok(refund)
}
```

### 7.5 Failed Payment Handling

```rust
async fn handle_payment_failed(
    state: &AppState,
    event: &stripe::Event,
) -> Result<()> {
    let payment_intent = extract_payment_intent(event)?;
    let checkout_session_id = payment_intent.metadata.get("checkout_session_id");

    if let Some(session_id) = checkout_session_id {
        // Release stock reservations
        release_stock_reservations(&state.db.pool, session_id).await?;

        // Fire hook
        state.hooks.do_action("rustcommerce_payment_failed", &json!({
            "checkout_session_id": session_id,
            "error": payment_intent.last_payment_error
                .map(|e| e.message.unwrap_or_default())
                .unwrap_or_default(),
        })).await?;
    }

    Ok(())
}
```

---

## 8. Order Number Generation

### 8.1 Format

Order numbers follow the format: `{prefix}{zero-padded sequence}`

Default: `RC-00001`, `RC-00002`, ..., `RC-99999`, `RC-100000`

### 8.2 Implementation

```rust
pub async fn generate_order_number(pool: &PgPool) -> Result<String> {
    // Atomic increment using PostgreSQL advisory lock + settings table
    let next_number: i64 = sqlx::query_scalar!(
        "UPDATE rc_store_settings
         SET value = to_jsonb((value::text::bigint + 1)),
             updated_at = now()
         WHERE key = 'order_number_sequence'
         RETURNING value::text::bigint"
    ).fetch_one(pool).await?;

    let prefix = get_setting(pool, "order_number_prefix").await?
        .unwrap_or_else(|| "RC-".to_string());

    Ok(format!("{}{:05}", prefix, next_number))
}
```

The sequence is stored in `rc_store_settings` and atomically incremented using a single UPDATE...RETURNING query, preventing race conditions.

---

## 9. Price Calculation Precision

### 9.1 Rules

1. All monetary values are stored as `DECIMAL(10,2)` in the database.
2. The Rust type for money is `rust_decimal::Decimal` (from the `rust_decimal` crate).
3. Intermediate calculations use full precision; rounding to 2 decimal places happens only at the final step.
4. Rounding uses **half-up** (banker's rounding): `round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)`.
5. Tax amounts are calculated per line item and then summed (not calculated on the total).
6. API serializes money as JSON strings: `"79.99"`, never as floating-point numbers.

### 9.2 Example

```
Cart:
  Item 1: $29.99 x 3 = $89.97
  Item 2: $14.50 x 2 = $29.00

Subtotal: $118.97

Coupon: 15% off
  Discount: $118.97 * 0.15 = $17.8455 -> rounded to $17.85

Taxable: $118.97 - $17.85 = $101.12

Tax (8.875%):
  Item 1 tax: $89.97 * 0.08875 = $7.98... -> $7.98
  Item 2 tax: $29.00 * 0.08875 = $2.57... -> $2.57
  (Proportional to discount: applied pro-rata in practice)
  Tax total: $8.97 (calculated on $101.12)

Shipping: $5.99 (flat rate)
Shipping tax: $5.99 * 0.08875 = $0.53

Grand Total: $118.97 - $17.85 + $8.97 + $0.53 + $5.99 = $116.61
```

---

## 10. Product Slug Generation

### 10.1 Algorithm

```rust
pub fn generate_slug(name: &str) -> String {
    let slug = name
        .to_lowercase()
        .trim()
        // Replace non-alphanumeric characters with hyphens
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        // Collapse multiple hyphens
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-");

    // Truncate to 255 characters
    slug[..slug.len().min(255)].to_string()
}

pub async fn ensure_unique_slug(pool: &PgPool, slug: &str, exclude_id: Option<Uuid>) -> Result<String> {
    let mut candidate = slug.to_string();
    let mut suffix = 1;

    loop {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM rc_products WHERE slug = $1 AND ($2::uuid IS NULL OR id != $2))",
            &candidate, exclude_id
        ).fetch_one(pool).await?;

        if !exists.unwrap_or(false) {
            return Ok(candidate);
        }

        candidate = format!("{}-{}", slug, suffix);
        suffix += 1;

        if suffix > 100 {
            return Err(Error::Internal("Could not generate unique slug".into()));
        }
    }
}
```

**Example:** "Wireless Bluetooth Headphones" -> `wireless-bluetooth-headphones`
If that exists: `wireless-bluetooth-headphones-1`, `wireless-bluetooth-headphones-2`, etc.

---

## 11. Customer Aggregate Updates

### 11.1 When to Update

Customer aggregates (`total_orders`, `total_spent`, `average_order_value`, `last_order_at`) are updated:

1. After order confirmation (payment successful)
2. After order cancellation (subtract from totals)
3. After full refund (subtract from totals)

### 11.2 Implementation

```rust
pub async fn update_customer_aggregates(
    pool: &PgPool,
    customer_id: Uuid,
) -> Result<()> {
    sqlx::query!(
        "UPDATE rc_customers SET
            total_orders = (
                SELECT COUNT(*) FROM rc_orders
                WHERE customer_id = $1
                  AND status NOT IN ('cancelled', 'refunded')
                  AND payment_status = 'paid'
            ),
            total_spent = (
                SELECT COALESCE(SUM(grand_total), 0) FROM rc_orders
                WHERE customer_id = $1
                  AND status NOT IN ('cancelled', 'refunded')
                  AND payment_status IN ('paid', 'partially_refunded')
            ),
            average_order_value = (
                SELECT COALESCE(AVG(grand_total), 0) FROM rc_orders
                WHERE customer_id = $1
                  AND status NOT IN ('cancelled', 'refunded')
                  AND payment_status IN ('paid', 'partially_refunded')
            ),
            last_order_at = (
                SELECT MAX(created_at) FROM rc_orders
                WHERE customer_id = $1
                  AND status NOT IN ('cancelled', 'refunded')
            ),
            updated_at = now()
        WHERE id = $1",
        customer_id
    ).execute(pool).await?;

    Ok(())
}
```

---

## 12. Cart Expiration and Cleanup

### 12.1 Cart Lifecycle

| Event | Action |
|-------|--------|
| Cart created | `expires_at = now() + 7 days` |
| Cart item added/updated | `expires_at = now() + 7 days` (refreshed) |
| Cart idle > 1 hour | Fire `rustcommerce_cart_abandoned` hook |
| Cart idle > 7 days | Mark as `abandoned`, eligible for cleanup |
| Checkout completed | Mark as `converted` |

### 12.2 Background Cleanup Job

Runs every 5 minutes via RustPress job system:

```rust
pub async fn cleanup_expired_carts(pool: &PgPool) -> Result<CleanupReport> {
    // 1. Mark abandoned carts (idle > 1 hour, still active)
    let abandoned_count = sqlx::query_scalar!(
        "UPDATE rc_carts SET status = 'abandoned', updated_at = now()
         WHERE status = 'active'
           AND updated_at < now() - interval '1 hour'
         RETURNING COUNT(*)"
    ).fetch_one(pool).await?;

    // 2. Delete expired carts (abandoned > 30 days)
    let deleted_count = sqlx::query_scalar!(
        "DELETE FROM rc_carts
         WHERE status = 'abandoned'
           AND updated_at < now() - interval '30 days'
         RETURNING COUNT(*)"
    ).fetch_one(pool).await?;

    // 3. Release expired stock reservations
    let released = sqlx::query!(
        "UPDATE rc_stock_reservations
         SET status = 'expired'
         WHERE status = 'active' AND expires_at <= now()
         RETURNING product_id, variant_id, quantity"
    ).fetch_all(pool).await?;

    // Restore stock for each expired reservation
    for reservation in &released {
        restore_stock(pool, &[(
            reservation.product_id,
            reservation.variant_id,
            reservation.quantity,
        )]).await?;
    }

    Ok(CleanupReport {
        carts_abandoned: abandoned_count,
        carts_deleted: deleted_count,
        reservations_released: released.len(),
    })
}
```

### 12.3 Registration as Cron Job

In `plugin.toml`:

```toml
[[cron]]
name = "rc_cart_cleanup"
schedule = "*/5 * * * *"    # Every 5 minutes
handler = "cleanup_expired_carts"
description = "Clean up abandoned carts and expired stock reservations"

[[cron]]
name = "rc_abandoned_cart_notify"
schedule = "0 * * * *"      # Every hour
handler = "notify_abandoned_carts"
description = "Send abandoned cart reminder emails"
```
