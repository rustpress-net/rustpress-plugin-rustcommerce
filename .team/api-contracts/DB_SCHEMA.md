# RustCommerce Database Schema

**Version**: 1.0.0
**Database**: PostgreSQL 16
**ORM/Driver**: sqlx (compile-time checked queries)
**Date**: 2026-02-24
**Status**: Approved

---

## Table of Contents

1. [Conventions](#1-conventions)
2. [Table Overview](#2-table-overview)
3. [Detailed Table Definitions](#3-detailed-table-definitions)
4. [Indexes](#4-indexes)
5. [Relationship Diagram](#5-relationship-diagram)
6. [Migration Strategy](#6-migration-strategy)
7. [Seed Data Requirements](#7-seed-data-requirements)
8. [Common Query Patterns](#8-common-query-patterns)

---

## 1. Conventions

### Naming

- **Table prefix**: `rc_` (RustCommerce) -- prevents collision with RustPress core tables.
- **Table names**: Lowercase, snake_case, plural (e.g., `rc_products`, `rc_order_items`).
- **Column names**: Lowercase, snake_case (e.g., `created_at`, `stock_quantity`).
- **Join tables**: Named as `rc_{entity1}_{entity2}` (e.g., `rc_product_categories`).
- **Foreign keys**: Named as `{referenced_table_singular}_id` (e.g., `product_id`, `order_id`).
- **Indexes**: Named as `idx_rc_{table}_{column(s)}` (e.g., `idx_rc_products_slug`).
- **Constraints**: Named as `chk_rc_{table}_{description}` for check constraints.

### Data Types

| Concept | PostgreSQL Type | Notes |
|---------|----------------|-------|
| Primary keys | `UUID` | `DEFAULT gen_random_uuid()`, UUID v4 |
| Money/price | `DECIMAL(10,2)` | Never use `FLOAT` or `REAL` for money |
| Tax rates | `DECIMAL(6,4)` | e.g., `0.0825` for 8.25% |
| Timestamps | `TIMESTAMPTZ` | Always with timezone, `DEFAULT now()` |
| Status fields | `VARCHAR(30)` | String enums for readability and flexibility |
| Short text | `VARCHAR(N)` | With explicit max length |
| Long text | `TEXT` | Unbounded text |
| Structured data | `JSONB` | For flexible/nested data (addresses on orders, variant attributes, metadata) |
| Arrays | `TEXT[]`, `UUID[]` | PostgreSQL native arrays for simple lists |
| Boolean | `BOOLEAN` | `DEFAULT false` or `DEFAULT true` |
| Counts | `INTEGER` | `DEFAULT 0`, non-negative |
| Weight/dimensions | `DECIMAL(8,2)` | Stored in a configured unit (kg, cm by default) |

### Standard Columns

Every table includes:

| Column | Type | Description |
|--------|------|-------------|
| `id` | `UUID PRIMARY KEY DEFAULT gen_random_uuid()` | Unique identifier |
| `created_at` | `TIMESTAMPTZ NOT NULL DEFAULT now()` | Record creation time |

Most tables also include:

| Column | Type | Description |
|--------|------|-------------|
| `updated_at` | `TIMESTAMPTZ NOT NULL DEFAULT now()` | Last modification time |

The application sets `updated_at = now()` on every UPDATE. No database triggers are used to keep behavior explicit and testable.

---

## 2. Table Overview

| # | Table Name | Description | Migration |
|---|-----------|-------------|-----------|
| 1 | `rc_products` | Product catalog | 00001 |
| 2 | `rc_product_variants` | Size/color variants for variable products | 00001 |
| 3 | `rc_product_images` | Product image gallery | 00001 |
| 4 | `rc_categories` | Hierarchical product categories | 00001 |
| 5 | `rc_product_categories` | Many-to-many: products <-> categories | 00001 |
| 6 | `rc_carts` | Shopping cart headers | 00002 |
| 7 | `rc_cart_items` | Items within a cart | 00002 |
| 8 | `rc_orders` | Order headers | 00002 |
| 9 | `rc_order_items` | Line items within an order | 00002 |
| 10 | `rc_order_status_history` | Audit trail of order status changes | 00002 |
| 11 | `rc_customers` | Customer profiles (extends RustPress users) | 00003 |
| 12 | `rc_customer_addresses` | Shipping and billing addresses | 00003 |
| 13 | `rc_payments` | Payment transactions | 00004 |
| 14 | `rc_refunds` | Refund transactions | 00004 |
| 15 | `rc_shipping_zones` | Geographic shipping zones | 00005 |
| 16 | `rc_shipping_methods` | Shipping methods per zone | 00005 |
| 17 | `rc_tax_rates` | Tax rates by location | 00005 |
| 18 | `rc_coupons` | Discount coupons | 00006 |
| 19 | `rc_coupon_usage` | Tracks per-user coupon usage | 00006 |
| 20 | `rc_reviews` | Product reviews and ratings | 00007 |
| 21 | `rc_review_votes` | Helpful votes on reviews | 00007 |
| 22 | `rc_store_settings` | Plugin-specific key-value settings | 00001 |
| 23 | `rc_stock_reservations` | Temporary stock holds during checkout | 00002 |

**Total: 23 tables**

---

## 3. Detailed Table Definitions

### 3.1 rc_products

Core product catalog table.

```sql
CREATE TABLE rc_products (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                VARCHAR(255) NOT NULL,
    slug                VARCHAR(255) NOT NULL UNIQUE,
    description         TEXT,
    short_description   TEXT,
    sku                 VARCHAR(100) UNIQUE,
    price               DECIMAL(10,2) NOT NULL,
    compare_at_price    DECIMAL(10,2),
    cost_price          DECIMAL(10,2),
    status              VARCHAR(20) NOT NULL DEFAULT 'draft',
    product_type        VARCHAR(50) NOT NULL DEFAULT 'simple',
    featured            BOOLEAN NOT NULL DEFAULT false,
    stock_quantity       INTEGER NOT NULL DEFAULT 0,
    stock_status        VARCHAR(20) NOT NULL DEFAULT 'in_stock',
    low_stock_threshold INTEGER NOT NULL DEFAULT 5,
    weight              DECIMAL(8,2),
    dimensions_length   DECIMAL(8,2),
    dimensions_width    DECIMAL(8,2),
    dimensions_height   DECIMAL(8,2),
    tax_class           VARCHAR(50) NOT NULL DEFAULT 'standard',
    meta                JSONB NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_rc_products_status
        CHECK (status IN ('draft', 'published', 'archived')),
    CONSTRAINT chk_rc_products_type
        CHECK (product_type IN ('simple', 'variable', 'grouped', 'digital')),
    CONSTRAINT chk_rc_products_stock_status
        CHECK (stock_status IN ('in_stock', 'out_of_stock', 'on_backorder')),
    CONSTRAINT chk_rc_products_price_positive
        CHECK (price >= 0),
    CONSTRAINT chk_rc_products_stock_nonneg
        CHECK (stock_quantity >= 0)
);
```

**Column Details:**

| Column | Description |
|--------|-------------|
| `name` | Product display name. Required. |
| `slug` | URL-friendly identifier. Auto-generated from `name` if not provided. Unique. |
| `description` | Full HTML product description for the detail page. |
| `short_description` | Brief text used in product cards and listings. |
| `sku` | Stock-keeping unit. Unique across all products. |
| `price` | Current selling price. For variable products, this is the base/starting price. |
| `compare_at_price` | Original price (used for "was $X, now $Y" display). Null if no comparison. |
| `cost_price` | Wholesale/cost price. Admin-only. Used for profit calculations. |
| `status` | `draft` (not visible), `published` (visible), `archived` (soft-deleted). |
| `product_type` | `simple` (single product), `variable` (has variants), `grouped` (bundle), `digital` (downloadable). |
| `featured` | Flagged for featured product sections. |
| `stock_quantity` | Total stock across all variants (for simple products) or sum of variant stocks. |
| `stock_status` | Derived from `stock_quantity` and settings. `in_stock`, `out_of_stock`, `on_backorder`. |
| `low_stock_threshold` | Quantity at which a low-stock alert is triggered. |
| `weight` | Product weight in configured unit (default: kg). Used for shipping calculation. |
| `dimensions_*` | Length, width, height in configured unit (default: cm). |
| `tax_class` | Tax class for tax calculation. `standard`, `reduced`, `zero`, or custom classes. |
| `meta` | JSONB for extensible data: SEO fields, custom attributes, etc. |

### 3.2 rc_product_variants

Variants for variable products (e.g., size, color combinations).

```sql
CREATE TABLE rc_product_variants (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id        UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    sku               VARCHAR(100) UNIQUE,
    name              VARCHAR(255) NOT NULL,
    price             DECIMAL(10,2) NOT NULL,
    compare_at_price  DECIMAL(10,2),
    stock_quantity    INTEGER NOT NULL DEFAULT 0,
    stock_status      VARCHAR(20) NOT NULL DEFAULT 'in_stock',
    attributes        JSONB NOT NULL DEFAULT '{}',
    image_url         TEXT,
    position          INTEGER NOT NULL DEFAULT 0,
    enabled           BOOLEAN NOT NULL DEFAULT true,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_rc_variants_price_positive
        CHECK (price >= 0),
    CONSTRAINT chk_rc_variants_stock_nonneg
        CHECK (stock_quantity >= 0),
    CONSTRAINT chk_rc_variants_stock_status
        CHECK (stock_status IN ('in_stock', 'out_of_stock', 'on_backorder'))
);
```

**Column Details:**

| Column | Description |
|--------|-------------|
| `attributes` | JSONB object defining the variant's attributes, e.g., `{"color": "Red", "size": "XL"}`. |
| `position` | Display ordering within the product. |
| `enabled` | Soft-disable a variant without deleting. |

### 3.3 rc_product_images

Product image gallery. Supports multiple images with ordering.

```sql
CREATE TABLE rc_product_images (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id  UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    url         TEXT NOT NULL,
    alt_text    VARCHAR(255),
    position    INTEGER NOT NULL DEFAULT 0,
    is_primary  BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Business Rule**: Only one image per product should have `is_primary = true`. Enforced at application level.

### 3.4 rc_categories

Hierarchical product categories using adjacency list pattern.

```sql
CREATE TABLE rc_categories (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(255) NOT NULL,
    slug            VARCHAR(255) NOT NULL UNIQUE,
    description     TEXT,
    parent_id       UUID REFERENCES rc_categories(id) ON DELETE SET NULL,
    image_url       TEXT,
    position        INTEGER NOT NULL DEFAULT 0,
    product_count   INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Column Details:**

| Column | Description |
|--------|-------------|
| `parent_id` | Self-referencing FK for hierarchy. `NULL` = top-level category. |
| `product_count` | Denormalized count. Updated by application on product category changes. |
| `position` | Ordering among siblings at the same hierarchy level. |

### 3.5 rc_product_categories

Join table for many-to-many relationship between products and categories.

```sql
CREATE TABLE rc_product_categories (
    product_id  UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES rc_categories(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, category_id)
);
```

### 3.6 rc_carts

Shopping cart headers. One cart per user (or per guest session).

```sql
CREATE TABLE rc_carts (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID,
    session_id  VARCHAR(255),
    status      VARCHAR(20) NOT NULL DEFAULT 'active',
    coupon_code VARCHAR(100),
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ,

    CONSTRAINT chk_rc_carts_status
        CHECK (status IN ('active', 'abandoned', 'converted')),
    CONSTRAINT chk_rc_carts_identity
        CHECK (user_id IS NOT NULL OR session_id IS NOT NULL)
);
```

**Column Details:**

| Column | Description |
|--------|-------------|
| `user_id` | RustPress user UUID. NULL for guest carts. |
| `session_id` | Client-generated UUID for guest cart tracking (from `X-Session-ID` header). |
| `status` | `active` (in use), `abandoned` (inactive > threshold), `converted` (became an order). |
| `coupon_code` | Currently applied coupon. |
| `expires_at` | Auto-cleanup timestamp. Extended on each cart interaction. |

**Business Rule**: When a guest user logs in, merge the session cart into the user's cart.

### 3.7 rc_cart_items

Individual items within a cart.

```sql
CREATE TABLE rc_cart_items (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cart_id     UUID NOT NULL REFERENCES rc_carts(id) ON DELETE CASCADE,
    product_id  UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    variant_id  UUID REFERENCES rc_product_variants(id) ON DELETE CASCADE,
    quantity    INTEGER NOT NULL DEFAULT 1,
    unit_price  DECIMAL(10,2) NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_rc_cart_items_quantity
        CHECK (quantity >= 1),
    CONSTRAINT uq_rc_cart_items_product_variant
        UNIQUE (cart_id, product_id, variant_id)
);
```

**Column Details:**

| Column | Description |
|--------|-------------|
| `unit_price` | Price at time of adding to cart. Updated if product price changes. |
| `variant_id` | NULL for simple products; required for variable products. |

**Unique Constraint**: A cart cannot have duplicate entries for the same product+variant combination. Instead, quantity is incremented.

### 3.8 rc_orders

Order headers. Created when checkout completes.

```sql
CREATE TABLE rc_orders (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_number              VARCHAR(50) NOT NULL UNIQUE,
    user_id                   UUID,
    customer_id               UUID REFERENCES rc_customers(id),
    status                    VARCHAR(30) NOT NULL DEFAULT 'pending',
    subtotal                  DECIMAL(10,2) NOT NULL,
    tax_total                 DECIMAL(10,2) NOT NULL DEFAULT 0,
    shipping_total            DECIMAL(10,2) NOT NULL DEFAULT 0,
    discount_total            DECIMAL(10,2) NOT NULL DEFAULT 0,
    grand_total               DECIMAL(10,2) NOT NULL,
    currency                  VARCHAR(3) NOT NULL DEFAULT 'USD',
    billing_address           JSONB NOT NULL,
    shipping_address          JSONB NOT NULL,
    shipping_method           VARCHAR(100),
    shipping_method_id        UUID,
    payment_method            VARCHAR(100),
    payment_status            VARCHAR(30) NOT NULL DEFAULT 'unpaid',
    stripe_payment_intent_id  VARCHAR(255),
    coupon_code               VARCHAR(100),
    customer_note             TEXT,
    admin_note                TEXT,
    ip_address                VARCHAR(45),
    user_agent                TEXT,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at              TIMESTAMPTZ,
    cancelled_at              TIMESTAMPTZ,

    CONSTRAINT chk_rc_orders_status
        CHECK (status IN ('pending', 'confirmed', 'processing', 'shipped', 'delivered', 'cancelled', 'refunded')),
    CONSTRAINT chk_rc_orders_payment_status
        CHECK (payment_status IN ('unpaid', 'paid', 'partially_refunded', 'refunded', 'failed')),
    CONSTRAINT chk_rc_orders_grand_total
        CHECK (grand_total >= 0)
);
```

**Column Details:**

| Column | Description |
|--------|-------------|
| `order_number` | Human-readable number in format `RC-NNNNN`. Auto-incrementing sequence managed by application. |
| `user_id` | RustPress user UUID. NULL for guest orders. |
| `customer_id` | FK to `rc_customers`. Created automatically on first order. |
| `billing_address` / `shipping_address` | JSONB snapshots of addresses at time of order. Immune to future address edits. |
| `shipping_method` | Name of selected shipping method (snapshot). |
| `shipping_method_id` | FK reference for tracking (not enforced, as methods may be deleted). |
| `stripe_payment_intent_id` | Stripe PaymentIntent ID for payment tracking and refunds. |
| `completed_at` | Set when status transitions to `delivered`. |
| `cancelled_at` | Set when status transitions to `cancelled`. |

### 3.9 rc_order_items

Line items within an order. Snapshots of product data at time of purchase.

```sql
CREATE TABLE rc_order_items (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id        UUID NOT NULL REFERENCES rc_orders(id) ON DELETE CASCADE,
    product_id      UUID NOT NULL,
    variant_id      UUID,
    product_name    VARCHAR(255) NOT NULL,
    variant_name    VARCHAR(255),
    sku             VARCHAR(100),
    quantity        INTEGER NOT NULL,
    unit_price      DECIMAL(10,2) NOT NULL,
    subtotal        DECIMAL(10,2) NOT NULL,
    tax_amount      DECIMAL(10,2) NOT NULL DEFAULT 0,
    discount_amount DECIMAL(10,2) NOT NULL DEFAULT 0,
    total           DECIMAL(10,2) NOT NULL,
    meta            JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_rc_order_items_quantity
        CHECK (quantity >= 1),
    CONSTRAINT chk_rc_order_items_total
        CHECK (total >= 0)
);
```

**Column Details:**

| Column | Description |
|--------|-------------|
| `product_id` / `variant_id` | References for tracking. NOT FK constraints, because products may be deleted. |
| `product_name` / `variant_name` / `sku` | Snapshots at time of order. Never change after creation. |
| `subtotal` | `unit_price * quantity` |
| `total` | `subtotal + tax_amount - discount_amount` |
| `meta` | Additional line item data (e.g., custom engravings, gift wrap options). |

### 3.10 rc_order_status_history

Audit trail for order status changes.

```sql
CREATE TABLE rc_order_status_history (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id    UUID NOT NULL REFERENCES rc_orders(id) ON DELETE CASCADE,
    old_status  VARCHAR(30),
    new_status  VARCHAR(30) NOT NULL,
    note        TEXT,
    changed_by  UUID,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Column Details:**

| Column | Description |
|--------|-------------|
| `old_status` | Previous status. NULL for the initial `pending` status entry. |
| `changed_by` | UUID of the admin user who made the change. NULL for system-initiated changes. |

### 3.11 rc_customers

Customer profiles, extending RustPress's user system.

```sql
CREATE TABLE rc_customers (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID UNIQUE,
    email               VARCHAR(255) NOT NULL,
    first_name          VARCHAR(100),
    last_name           VARCHAR(100),
    phone               VARCHAR(50),
    total_orders        INTEGER NOT NULL DEFAULT 0,
    total_spent         DECIMAL(12,2) NOT NULL DEFAULT 0,
    average_order_value DECIMAL(10,2) NOT NULL DEFAULT 0,
    last_order_at       TIMESTAMPTZ,
    notes               TEXT,
    meta                JSONB NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Column Details:**

| Column | Description |
|--------|-------------|
| `user_id` | FK to RustPress `users.id`. NULL for guest checkout customers. Unique when set. |
| `total_orders` / `total_spent` / `average_order_value` | Denormalized aggregate fields. Updated by application after each order. |
| `notes` | Admin notes about the customer. |

### 3.12 rc_customer_addresses

Normalized address storage for customer saved addresses.

```sql
CREATE TABLE rc_customer_addresses (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id     UUID NOT NULL REFERENCES rc_customers(id) ON DELETE CASCADE,
    address_type    VARCHAR(20) NOT NULL DEFAULT 'shipping',
    is_default      BOOLEAN NOT NULL DEFAULT false,
    first_name      VARCHAR(100),
    last_name       VARCHAR(100),
    company         VARCHAR(255),
    address_line_1  VARCHAR(255) NOT NULL,
    address_line_2  VARCHAR(255),
    city            VARCHAR(100) NOT NULL,
    state           VARCHAR(100),
    postal_code     VARCHAR(20) NOT NULL,
    country         VARCHAR(2) NOT NULL,
    phone           VARCHAR(50),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_rc_addresses_type
        CHECK (address_type IN ('billing', 'shipping'))
);
```

**Business Rule**: Only one address per type per customer can be `is_default = true`. Enforced at application level.

### 3.13 rc_payments

Payment transaction records.

```sql
CREATE TABLE rc_payments (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id          UUID NOT NULL REFERENCES rc_orders(id),
    payment_method    VARCHAR(50) NOT NULL,
    status            VARCHAR(30) NOT NULL DEFAULT 'pending',
    amount            DECIMAL(10,2) NOT NULL,
    currency          VARCHAR(3) NOT NULL DEFAULT 'USD',
    transaction_id    VARCHAR(255),
    gateway_response  JSONB,
    error_message     TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_rc_payments_status
        CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'cancelled')),
    CONSTRAINT chk_rc_payments_method
        CHECK (payment_method IN ('stripe', 'paypal', 'manual', 'bank_transfer')),
    CONSTRAINT chk_rc_payments_amount
        CHECK (amount > 0)
);
```

**Column Details:**

| Column | Description |
|--------|-------------|
| `transaction_id` | External payment gateway's transaction ID (e.g., Stripe charge ID `ch_xxx`). |
| `gateway_response` | Raw JSON response from the payment gateway for debugging. |
| `error_message` | Human-readable error description for failed payments. |

### 3.14 rc_refunds

Refund transaction records.

```sql
CREATE TABLE rc_refunds (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id        UUID NOT NULL REFERENCES rc_orders(id),
    payment_id      UUID NOT NULL REFERENCES rc_payments(id),
    amount          DECIMAL(10,2) NOT NULL,
    currency        VARCHAR(3) NOT NULL DEFAULT 'USD',
    reason          TEXT,
    status          VARCHAR(30) NOT NULL DEFAULT 'pending',
    transaction_id  VARCHAR(255),
    gateway_response JSONB,
    refunded_by     UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_rc_refunds_status
        CHECK (status IN ('pending', 'completed', 'failed')),
    CONSTRAINT chk_rc_refunds_amount
        CHECK (amount > 0)
);
```

### 3.15 rc_shipping_zones

Geographic zones for shipping rate determination.

```sql
CREATE TABLE rc_shipping_zones (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL,
    countries   TEXT[] NOT NULL DEFAULT '{}',
    regions     TEXT[] DEFAULT '{}',
    postal_codes TEXT[] DEFAULT '{}',
    is_default  BOOLEAN NOT NULL DEFAULT false,
    position    INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Column Details:**

| Column | Description |
|--------|-------------|
| `countries` | Array of ISO 3166-1 alpha-2 codes (e.g., `{"US", "CA"}`). |
| `regions` | Array of state/province codes (e.g., `{"US-NY", "US-CA"}`). |
| `postal_codes` | Array of postal code patterns (e.g., `{"10*", "900*"}`). Supports wildcards. |
| `is_default` | Fallback zone for addresses not matching any other zone. Only one zone should be default. |

**Zone Matching Priority**: postal_codes (most specific) > regions > countries > default zone.

### 3.16 rc_shipping_methods

Shipping methods available within each zone.

```sql
CREATE TABLE rc_shipping_methods (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    zone_id         UUID REFERENCES rc_shipping_zones(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    method_type     VARCHAR(50) NOT NULL,
    cost            DECIMAL(10,2) NOT NULL DEFAULT 0,
    free_threshold  DECIMAL(10,2),
    min_weight      DECIMAL(8,2),
    max_weight      DECIMAL(8,2),
    settings        JSONB NOT NULL DEFAULT '{}',
    enabled         BOOLEAN NOT NULL DEFAULT true,
    position        INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_rc_shipping_type
        CHECK (method_type IN ('flat_rate', 'free_shipping', 'weight_based', 'price_based')),
    CONSTRAINT chk_rc_shipping_cost
        CHECK (cost >= 0)
);
```

**`settings` JSONB Examples:**

For `weight_based`:
```json
{
  "base_cost": "10.00",
  "per_kg": "2.50",
  "min_weight": "0.00",
  "max_weight": "30.00"
}
```

For `price_based`:
```json
{
  "tiers": [
    { "min": "0.00", "max": "49.99", "cost": "9.99" },
    { "min": "50.00", "max": "99.99", "cost": "5.99" },
    { "min": "100.00", "max": null, "cost": "0.00" }
  ]
}
```

### 3.17 rc_tax_rates

Tax rates by geographic location and product tax class.

```sql
CREATE TABLE rc_tax_rates (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL,
    rate        DECIMAL(6,4) NOT NULL,
    country     VARCHAR(2) NOT NULL,
    state       VARCHAR(100),
    postal_code VARCHAR(20),
    city        VARCHAR(100),
    tax_class   VARCHAR(50) NOT NULL DEFAULT 'standard',
    compound    BOOLEAN NOT NULL DEFAULT false,
    shipping    BOOLEAN NOT NULL DEFAULT false,
    priority    INTEGER NOT NULL DEFAULT 1,
    enabled     BOOLEAN NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_rc_tax_rate_positive
        CHECK (rate >= 0 AND rate < 1),
    CONSTRAINT chk_rc_tax_priority
        CHECK (priority >= 1)
);
```

**Column Details:**

| Column | Description |
|--------|-------------|
| `rate` | Decimal rate, e.g., `0.0825` for 8.25%. Must be between 0 and 1. |
| `country` | ISO 3166-1 alpha-2. Required. |
| `state` | State/province. NULL means "all states in this country". |
| `postal_code` | NULL means "all postal codes". Supports wildcard via application logic. |
| `city` | NULL means "all cities". |
| `tax_class` | `standard`, `reduced`, `zero`, or custom class names. |
| `compound` | If true, this tax is calculated on top of (subtotal + non-compound taxes). |
| `shipping` | If true, this tax rate also applies to shipping costs. |
| `priority` | Tax rates with the same priority are summed; different priorities are compounded. |

### 3.18 rc_coupons

Discount coupon definitions.

```sql
CREATE TABLE rc_coupons (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code                 VARCHAR(100) NOT NULL UNIQUE,
    description          TEXT,
    discount_type        VARCHAR(30) NOT NULL,
    discount_value       DECIMAL(10,2) NOT NULL,
    minimum_spend        DECIMAL(10,2),
    maximum_spend        DECIMAL(10,2),
    usage_limit          INTEGER,
    usage_count          INTEGER NOT NULL DEFAULT 0,
    usage_limit_per_user INTEGER,
    product_ids          UUID[] DEFAULT '{}',
    category_ids         UUID[] DEFAULT '{}',
    excluded_product_ids UUID[] DEFAULT '{}',
    starts_at            TIMESTAMPTZ,
    expires_at           TIMESTAMPTZ,
    enabled              BOOLEAN NOT NULL DEFAULT true,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_rc_coupons_type
        CHECK (discount_type IN ('percentage', 'fixed_cart', 'fixed_product', 'free_shipping')),
    CONSTRAINT chk_rc_coupons_value
        CHECK (discount_value >= 0),
    CONSTRAINT chk_rc_coupons_percentage
        CHECK (discount_type != 'percentage' OR discount_value <= 100)
);
```

**Column Details:**

| Column | Description |
|--------|-------------|
| `discount_type` | `percentage` (% off), `fixed_cart` ($ off total), `fixed_product` ($ off each qualifying item), `free_shipping`. |
| `discount_value` | Amount: percentage (0-100) or fixed dollar amount. |
| `minimum_spend` / `maximum_spend` | Cart subtotal thresholds for eligibility. |
| `usage_limit` | Global max uses. NULL = unlimited. |
| `usage_count` | Current total uses. Incremented atomically on order placement. |
| `usage_limit_per_user` | Max uses per customer. NULL = unlimited. |
| `product_ids` | If set, coupon only applies to these products. |
| `category_ids` | If set, coupon only applies to products in these categories. |
| `excluded_product_ids` | Products excluded from this coupon even if category matches. |

### 3.19 rc_coupon_usage

Tracks per-user coupon usage for `usage_limit_per_user` enforcement.

```sql
CREATE TABLE rc_coupon_usage (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    coupon_id   UUID NOT NULL REFERENCES rc_coupons(id) ON DELETE CASCADE,
    customer_id UUID REFERENCES rc_customers(id),
    order_id    UUID NOT NULL REFERENCES rc_orders(id),
    used_at     TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_rc_coupon_usage_order
        UNIQUE (coupon_id, order_id)
);
```

### 3.20 rc_reviews

Product reviews and ratings.

```sql
CREATE TABLE rc_reviews (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id        UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    customer_id       UUID REFERENCES rc_customers(id),
    rating            INTEGER NOT NULL,
    title             VARCHAR(255),
    content           TEXT,
    status            VARCHAR(20) NOT NULL DEFAULT 'pending',
    verified_purchase BOOLEAN NOT NULL DEFAULT false,
    helpful_count     INTEGER NOT NULL DEFAULT 0,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_rc_reviews_rating
        CHECK (rating >= 1 AND rating <= 5),
    CONSTRAINT chk_rc_reviews_status
        CHECK (status IN ('pending', 'approved', 'rejected', 'spam'))
);
```

**Business Rule**: `verified_purchase` is set to `true` automatically if the customer has a completed order containing the reviewed product.

### 3.21 rc_review_votes

Prevents duplicate "helpful" votes from the same user.

```sql
CREATE TABLE rc_review_votes (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    review_id   UUID NOT NULL REFERENCES rc_reviews(id) ON DELETE CASCADE,
    user_id     UUID,
    ip_address  VARCHAR(45),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_rc_review_votes_user
        UNIQUE (review_id, user_id),
    CONSTRAINT chk_rc_review_votes_identity
        CHECK (user_id IS NOT NULL OR ip_address IS NOT NULL)
);
```

### 3.22 rc_store_settings

Plugin-specific settings stored as key-value pairs.

```sql
CREATE TABLE rc_store_settings (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key         VARCHAR(255) NOT NULL UNIQUE,
    value       JSONB NOT NULL,
    group_name  VARCHAR(100) NOT NULL DEFAULT 'general',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Example Rows:**

| key | value | group_name |
|-----|-------|------------|
| `store_name` | `"My RustPress Store"` | `general` |
| `currency` | `"USD"` | `general` |
| `stripe_publishable_key` | `"pk_live_..."` | `payments` |
| `stripe_secret_key` | `"sk_live_..."` | `payments` |
| `stripe_webhook_secret` | `"whsec_..."` | `payments` |
| `hold_stock_minutes` | `10` | `inventory` |
| `low_stock_threshold` | `5` | `inventory` |
| `order_number_sequence` | `42` | `orders` |

### 3.23 rc_stock_reservations

Temporary stock holds during checkout to prevent overselling.

```sql
CREATE TABLE rc_stock_reservations (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    checkout_session_id UUID NOT NULL,
    product_id          UUID NOT NULL REFERENCES rc_products(id),
    variant_id          UUID REFERENCES rc_product_variants(id),
    quantity            INTEGER NOT NULL,
    reserved_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at          TIMESTAMPTZ NOT NULL,
    status              VARCHAR(20) NOT NULL DEFAULT 'active',

    CONSTRAINT chk_rc_reservations_status
        CHECK (status IN ('active', 'converted', 'expired', 'released')),
    CONSTRAINT chk_rc_reservations_quantity
        CHECK (quantity >= 1)
);
```

**Column Details:**

| Column | Description |
|--------|-------------|
| `checkout_session_id` | Links to the checkout session. Not an FK because checkout sessions are transient. |
| `expires_at` | Typically `now() + 10 minutes`. |
| `status` | `active` (held), `converted` (order created), `expired` (timed out), `released` (manually released). |

**Background Job**: A scheduled job runs every minute to release expired reservations and restore stock.

---

## 4. Indexes

### 4.1 Product Indexes

```sql
-- Primary lookups
CREATE INDEX idx_rc_products_slug ON rc_products(slug);
CREATE INDEX idx_rc_products_sku ON rc_products(sku);

-- Listing filters
CREATE INDEX idx_rc_products_status ON rc_products(status);
CREATE INDEX idx_rc_products_type ON rc_products(product_type);
CREATE INDEX idx_rc_products_featured ON rc_products(featured) WHERE featured = true;
CREATE INDEX idx_rc_products_stock_status ON rc_products(stock_status);
CREATE INDEX idx_rc_products_price ON rc_products(price);

-- Sorting
CREATE INDEX idx_rc_products_created_at ON rc_products(created_at DESC);

-- Full-text search
CREATE INDEX idx_rc_products_search ON rc_products
    USING GIN (to_tsvector('english', coalesce(name, '') || ' ' || coalesce(description, '') || ' ' || coalesce(sku, '')));

-- Variant lookups
CREATE INDEX idx_rc_variants_product ON rc_product_variants(product_id);
CREATE INDEX idx_rc_variants_sku ON rc_product_variants(sku);

-- Image lookups
CREATE INDEX idx_rc_images_product ON rc_product_images(product_id);

-- Category join
CREATE INDEX idx_rc_product_categories_product ON rc_product_categories(product_id);
CREATE INDEX idx_rc_product_categories_category ON rc_product_categories(category_id);
```

### 4.2 Category Indexes

```sql
CREATE INDEX idx_rc_categories_slug ON rc_categories(slug);
CREATE INDEX idx_rc_categories_parent ON rc_categories(parent_id);
```

### 4.3 Cart Indexes

```sql
CREATE INDEX idx_rc_carts_user ON rc_carts(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_rc_carts_session ON rc_carts(session_id) WHERE session_id IS NOT NULL;
CREATE INDEX idx_rc_carts_status ON rc_carts(status);
CREATE INDEX idx_rc_carts_expires ON rc_carts(expires_at) WHERE status = 'active';

CREATE INDEX idx_rc_cart_items_cart ON rc_cart_items(cart_id);
CREATE INDEX idx_rc_cart_items_product ON rc_cart_items(product_id);
```

### 4.4 Order Indexes

```sql
CREATE INDEX idx_rc_orders_number ON rc_orders(order_number);
CREATE INDEX idx_rc_orders_user ON rc_orders(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_rc_orders_customer ON rc_orders(customer_id);
CREATE INDEX idx_rc_orders_status ON rc_orders(status);
CREATE INDEX idx_rc_orders_payment_status ON rc_orders(payment_status);
CREATE INDEX idx_rc_orders_created_at ON rc_orders(created_at DESC);
CREATE INDEX idx_rc_orders_stripe_pi ON rc_orders(stripe_payment_intent_id) WHERE stripe_payment_intent_id IS NOT NULL;

CREATE INDEX idx_rc_order_items_order ON rc_order_items(order_id);
CREATE INDEX idx_rc_order_items_product ON rc_order_items(product_id);

CREATE INDEX idx_rc_order_history_order ON rc_order_status_history(order_id);
```

### 4.5 Customer Indexes

```sql
CREATE INDEX idx_rc_customers_email ON rc_customers(email);
CREATE INDEX idx_rc_customers_user ON rc_customers(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_rc_customers_last_order ON rc_customers(last_order_at DESC);

CREATE INDEX idx_rc_addresses_customer ON rc_customer_addresses(customer_id);
```

### 4.6 Payment Indexes

```sql
CREATE INDEX idx_rc_payments_order ON rc_payments(order_id);
CREATE INDEX idx_rc_payments_transaction ON rc_payments(transaction_id) WHERE transaction_id IS NOT NULL;
CREATE INDEX idx_rc_payments_status ON rc_payments(status);

CREATE INDEX idx_rc_refunds_order ON rc_refunds(order_id);
CREATE INDEX idx_rc_refunds_payment ON rc_refunds(payment_id);
```

### 4.7 Tax/Shipping Indexes

```sql
CREATE INDEX idx_rc_tax_rates_location ON rc_tax_rates(country, state, postal_code, city);
CREATE INDEX idx_rc_tax_rates_class ON rc_tax_rates(tax_class);
CREATE INDEX idx_rc_tax_rates_enabled ON rc_tax_rates(enabled) WHERE enabled = true;

CREATE INDEX idx_rc_shipping_zones_default ON rc_shipping_zones(is_default) WHERE is_default = true;
CREATE INDEX idx_rc_shipping_methods_zone ON rc_shipping_methods(zone_id);
CREATE INDEX idx_rc_shipping_methods_enabled ON rc_shipping_methods(enabled) WHERE enabled = true;
```

### 4.8 Coupon Indexes

```sql
CREATE INDEX idx_rc_coupons_code ON rc_coupons(code);
CREATE INDEX idx_rc_coupons_enabled ON rc_coupons(enabled) WHERE enabled = true;
CREATE INDEX idx_rc_coupons_dates ON rc_coupons(starts_at, expires_at);

CREATE INDEX idx_rc_coupon_usage_coupon ON rc_coupon_usage(coupon_id);
CREATE INDEX idx_rc_coupon_usage_customer ON rc_coupon_usage(customer_id);
```

### 4.9 Review Indexes

```sql
CREATE INDEX idx_rc_reviews_product ON rc_reviews(product_id);
CREATE INDEX idx_rc_reviews_customer ON rc_reviews(customer_id);
CREATE INDEX idx_rc_reviews_status ON rc_reviews(status);
CREATE INDEX idx_rc_reviews_rating ON rc_reviews(product_id, rating);

CREATE INDEX idx_rc_review_votes_review ON rc_review_votes(review_id);
```

### 4.10 Stock Reservation Indexes

```sql
CREATE INDEX idx_rc_reservations_session ON rc_stock_reservations(checkout_session_id);
CREATE INDEX idx_rc_reservations_product ON rc_stock_reservations(product_id);
CREATE INDEX idx_rc_reservations_expires ON rc_stock_reservations(expires_at) WHERE status = 'active';
```

### 4.11 Settings Indexes

```sql
CREATE INDEX idx_rc_settings_group ON rc_store_settings(group_name);
```

---

## 5. Relationship Diagram

```
                                    ┌─────────────────────┐
                                    │  RustPress Users     │
                                    │  (core users table)  │
                                    └──────────┬──────────┘
                                               │ 1:1 (user_id)
                                               ▼
┌──────────────────┐           ┌──────────────────────────┐
│  rc_categories   │◄──────────│     rc_customers          │
│                  │           │                            │
│  id              │           │  id                        │
│  name            │           │  user_id (unique, nullable)│
│  slug            │           │  email                     │
│  parent_id ──┐   │           │  total_orders              │
│              │   │           │  total_spent               │
│  ◄───────────┘   │           └──────┬───────────┬────────┘
└────────┬─────────┘                  │           │
         │                            │ 1:N       │ 1:N
         │ M:N                        ▼           ▼
         │                 ┌─────────────┐  ┌──────────────────┐
         │                 │ rc_customer_ │  │    rc_reviews     │
         ▼                 │ _addresses   │  │                  │
┌────────────────────┐     └─────────────┘  │  rating (1-5)    │
│rc_product_categories│                     │  status           │
│                    │                      │  verified_purchase │
│  product_id (PK)   │                     └────────┬──────────┘
│  category_id (PK)  │                              │ 1:N
└────────┬───────────┘                              ▼
         │                                  ┌──────────────────┐
         │ M:N                              │  rc_review_votes  │
         ▼                                  └──────────────────┘
┌────────────────────────┐
│     rc_products        │
│                        │
│  id                    │──────────────┐
│  name, slug, sku       │              │ 1:N
│  price                 │              ▼
│  status                │    ┌──────────────────────────┐
│  stock_quantity        │    │  rc_product_variants      │
│  product_type          │    │                          │
│                        │    │  sku, price              │
│                        │    │  stock_quantity           │
│                        │    │  attributes (JSONB)       │
│                        │    └──────────────────────────┘
│                        │──────────────┐
│                        │              │ 1:N
│                        │              ▼
│                        │    ┌──────────────────────────┐
│                        │    │  rc_product_images        │
│                        │    │                          │
│                        │    │  url, alt_text           │
│                        │    │  position, is_primary    │
│                        │    └──────────────────────────┘
└───────────┬────────────┘
            │
            │ Referenced by (not FK)
            ▼
┌────────────────────────┐         ┌──────────────────────────┐
│     rc_cart_items      │◄────────│       rc_carts            │
│                        │   1:N   │                          │
│  product_id            │         │  user_id / session_id     │
│  variant_id            │         │  status                   │
│  quantity              │         │  coupon_code              │
│  unit_price            │         │  expires_at               │
└────────────────────────┘         └──────────────────────────┘

                              Checkout converts cart to order
                                           │
                                           ▼
┌────────────────────────┐         ┌──────────────────────────┐
│    rc_order_items      │◄────────│       rc_orders           │
│                        │   1:N   │                          │
│  product_name (snap)   │         │  order_number (RC-NNNNN) │
│  variant_name (snap)   │         │  user_id / customer_id    │
│  quantity              │         │  status                   │
│  unit_price            │         │  subtotal, tax, shipping  │
│  tax_amount            │         │  discount_total           │
│  discount_amount       │         │  grand_total              │
│  total                 │         │  billing_address (JSONB)  │
└────────────────────────┘         │  shipping_address (JSONB) │
                                   │  stripe_payment_intent_id │
                                   └──────┬──────────┬────────┘
                                          │          │
                                   1:N    │          │ 1:N
                                          ▼          ▼
                               ┌──────────────┐ ┌─────────────────────────┐
                               │ rc_payments  │ │ rc_order_status_history  │
                               │              │ │                         │
                               │ amount       │ │ old_status → new_status │
                               │ status       │ │ note                    │
                               │ transaction_ │ │ changed_by              │
                               │ _id          │ └─────────────────────────┘
                               └──────┬───────┘
                                      │ 1:N
                                      ▼
                               ┌──────────────┐
                               │  rc_refunds  │
                               │              │
                               │  amount      │
                               │  reason      │
                               │  status      │
                               └──────────────┘

┌──────────────────────────┐  ┌──────────────────────────┐  ┌─────────────────────┐
│   rc_shipping_zones      │  │    rc_tax_rates           │  │    rc_coupons        │
│                          │  │                          │  │                     │
│  countries[]             │  │  country, state, city    │  │  code (unique)      │
│  regions[]               │  │  rate (DECIMAL 6,4)      │  │  discount_type      │
│  postal_codes[]          │  │  tax_class               │  │  discount_value     │
│  is_default              │  │  compound, shipping      │  │  usage_limit/count  │
│                          │  │  priority                │  └──────────┬──────────┘
│         │ 1:N            │  └──────────────────────────┘             │ 1:N
│         ▼                │                                          ▼
│  ┌───────────────────┐   │                                ┌──────────────────┐
│  │rc_shipping_methods│   │                                │ rc_coupon_usage   │
│  │                   │   │                                │                  │
│  │ method_type       │   │                                │ coupon_id        │
│  │ cost              │   │                                │ customer_id      │
│  │ free_threshold    │   │                                │ order_id         │
│  │ settings (JSONB)  │   │                                └──────────────────┘
│  └───────────────────┘   │
└──────────────────────────┘

┌──────────────────────────┐  ┌──────────────────────────┐
│  rc_stock_reservations   │  │   rc_store_settings      │
│                          │  │                          │
│  checkout_session_id     │  │  key (unique)            │
│  product_id              │  │  value (JSONB)           │
│  variant_id              │  │  group_name              │
│  quantity                │  └──────────────────────────┘
│  expires_at              │
│  status                  │
└──────────────────────────┘
```

---

## 6. Migration Strategy

### 6.1 Migration Files

Migrations are ordered and each file is idempotent. They live in `migrations/` within the plugin directory:

| # | File | Tables Created | Dependencies |
|---|------|---------------|-------------|
| 1 | `00001_ecommerce_core.sql` | `rc_products`, `rc_product_variants`, `rc_product_images`, `rc_categories`, `rc_product_categories`, `rc_store_settings` | None |
| 2 | `00002_cart_and_orders.sql` | `rc_carts`, `rc_cart_items`, `rc_orders`, `rc_order_items`, `rc_order_status_history`, `rc_stock_reservations` | Migration 1 (references `rc_products`, `rc_product_variants`) |
| 3 | `00003_customers.sql` | `rc_customers`, `rc_customer_addresses` | None (references RustPress `users` table by convention, not FK) |
| 4 | `00004_payments.sql` | `rc_payments`, `rc_refunds` | Migration 2 (references `rc_orders`) |
| 5 | `00005_shipping_and_tax.sql` | `rc_shipping_zones`, `rc_shipping_methods`, `rc_tax_rates` | None |
| 6 | `00006_coupons.sql` | `rc_coupons`, `rc_coupon_usage` | Migrations 2 and 3 (references `rc_orders`, `rc_customers`) |
| 7 | `00007_reviews.sql` | `rc_reviews`, `rc_review_votes` | Migrations 1 and 3 (references `rc_products`, `rc_customers`) |

### 6.2 Migration Execution Order

```
00001 ─── 00002 ─── 00004
  │         │
  │         └── 00006 (needs rc_orders + rc_customers)
  │
  ├── 00003 ─── 00006
  │         └── 00007
  │
  ├── 00005
  │
  └── 00007
```

**Safe execution order**: `00001` -> `00002` -> `00003` -> `00004` -> `00005` -> `00006` -> `00007`

### 6.3 Integration with RustPress

Each migration file is declared in `plugin.toml`:

```toml
[migrations]
directory = "migrations"
version_table = "rc_migrations"
```

RustPress's migration system tracks which migrations have been applied in the `rc_migrations` table:

```sql
CREATE TABLE IF NOT EXISTS rc_migrations (
    id          SERIAL PRIMARY KEY,
    filename    VARCHAR(255) NOT NULL UNIQUE,
    applied_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    checksum    VARCHAR(64) NOT NULL
);
```

### 6.4 Rollback Strategy

Each migration file includes a corresponding `-- DOWN` section (commented out, for reference). Rollbacks are manual operations during development:

```sql
-- DOWN (for reference, not auto-executed)
-- DROP TABLE IF EXISTS rc_product_categories CASCADE;
-- DROP TABLE IF EXISTS rc_product_images CASCADE;
-- DROP TABLE IF EXISTS rc_product_variants CASCADE;
-- DROP TABLE IF EXISTS rc_categories CASCADE;
-- DROP TABLE IF EXISTS rc_products CASCADE;
-- DROP TABLE IF EXISTS rc_store_settings CASCADE;
```

In production, rollbacks involve creating a new forward migration that undoes changes.

### 6.5 Schema Versioning

Plugin deactivation does NOT drop tables. Data persists across plugin activate/deactivate cycles. Only an explicit "uninstall" operation (with admin confirmation) drops tables.

---

## 7. Seed Data Requirements

### 7.1 Required Seed Data (on first activation)

The following data is inserted during the `activate()` lifecycle method:

**Store Settings:**

```sql
INSERT INTO rc_store_settings (key, value, group_name) VALUES
  ('store_name', '"My Store"', 'general'),
  ('currency', '"USD"', 'general'),
  ('currency_symbol', '"$"', 'general'),
  ('currency_position', '"before"', 'general'),
  ('thousand_separator', '","', 'general'),
  ('decimal_separator', '"."', 'general'),
  ('guest_checkout_enabled', 'true', 'general'),
  ('order_number_prefix', '"RC-"', 'orders'),
  ('order_number_sequence', '0', 'orders'),
  ('hold_stock_minutes', '10', 'inventory'),
  ('low_stock_threshold_default', '5', 'inventory'),
  ('manage_stock', 'true', 'inventory'),
  ('backorders_allowed', 'false', 'inventory'),
  ('out_of_stock_visibility', '"hide"', 'inventory'),
  ('tax_enabled', 'true', 'tax'),
  ('prices_include_tax', 'false', 'tax'),
  ('calculate_tax_on', '"shipping_address"', 'tax'),
  ('shipping_enabled', 'true', 'shipping'),
  ('weight_unit', '"kg"', 'shipping'),
  ('dimension_unit', '"cm"', 'shipping'),
  ('reviews_enabled', 'true', 'reviews'),
  ('review_require_verification', 'true', 'reviews'),
  ('review_auto_approve', 'false', 'reviews')
ON CONFLICT (key) DO NOTHING;
```

**Default Shipping Zone:**

```sql
INSERT INTO rc_shipping_zones (name, countries, is_default, position) VALUES
  ('Rest of World', '{}', true, 99)
ON CONFLICT DO NOTHING;
```

**Default Tax Class:**

A "standard" tax class is implicit (the default value of `rc_products.tax_class`). No separate table tracks tax classes; they are defined by their use in `rc_tax_rates.tax_class`.

### 7.2 Development Seed Data

For development and testing, a seed script creates sample data:

- 10 sample categories (3 top-level, 7 subcategories)
- 50 sample products (mix of simple and variable)
- 5 sample customers with addresses
- 10 sample orders in various statuses
- 3 sample coupons
- 2 shipping zones with methods
- 5 tax rates (US federal and state examples)
- 15 sample reviews

This seed data is loaded via a CLI command: `rustpress-cli rustcommerce:seed`

---

## 8. Common Query Patterns

### 8.1 Product Listing (Public Storefront)

```sql
-- Paginated product listing with cursor-based pagination
SELECT
    p.id, p.name, p.slug, p.short_description,
    p.price, p.compare_at_price, p.sku,
    p.status, p.product_type, p.featured,
    p.stock_status, p.stock_quantity,
    p.created_at, p.updated_at,
    (SELECT json_agg(json_build_object(
        'id', pi.id, 'url', pi.url, 'alt_text', pi.alt_text, 'is_primary', pi.is_primary
    ) ORDER BY pi.position)
    FROM rc_product_images pi WHERE pi.product_id = p.id) AS images,
    (SELECT json_agg(json_build_object(
        'id', c.id, 'name', c.name, 'slug', c.slug
    ))
    FROM rc_categories c
    JOIN rc_product_categories pc ON pc.category_id = c.id
    WHERE pc.product_id = p.id) AS categories,
    (SELECT COUNT(*) FROM rc_product_variants pv WHERE pv.product_id = p.id) AS variants_count,
    (SELECT AVG(r.rating)::NUMERIC(3,2) FROM rc_reviews r WHERE r.product_id = p.id AND r.status = 'approved') AS average_rating,
    (SELECT COUNT(*) FROM rc_reviews r WHERE r.product_id = p.id AND r.status = 'approved') AS review_count
FROM rc_products p
WHERE p.status = 'published'
    AND (p.created_at, p.id) < ($1, $2)  -- cursor condition
ORDER BY p.created_at DESC, p.id DESC
LIMIT $3;
```

### 8.2 Product Detail

```sql
-- Full product detail with variants and images
SELECT
    p.*,
    (SELECT json_agg(row_to_json(pv.*) ORDER BY pv.position)
     FROM rc_product_variants pv WHERE pv.product_id = p.id AND pv.enabled = true) AS variants,
    (SELECT json_agg(row_to_json(pi.*) ORDER BY pi.position)
     FROM rc_product_images pi WHERE pi.product_id = p.id) AS images,
    (SELECT json_agg(json_build_object('id', c.id, 'name', c.name, 'slug', c.slug))
     FROM rc_categories c
     JOIN rc_product_categories pc ON pc.category_id = c.id
     WHERE pc.product_id = p.id) AS categories
FROM rc_products p
WHERE p.id = $1 OR p.slug = $1;
```

### 8.3 Full-Text Product Search

```sql
SELECT p.id, p.name, p.slug, p.price, p.short_description,
    ts_rank_cd(
        to_tsvector('english', coalesce(p.name, '') || ' ' || coalesce(p.description, '') || ' ' || coalesce(p.sku, '')),
        plainto_tsquery('english', $1)
    ) AS rank
FROM rc_products p
WHERE p.status = 'published'
    AND to_tsvector('english', coalesce(p.name, '') || ' ' || coalesce(p.description, '') || ' ' || coalesce(p.sku, ''))
        @@ plainto_tsquery('english', $1)
ORDER BY rank DESC, p.created_at DESC
LIMIT $2 OFFSET $3;
```

### 8.4 Cart with Totals

```sql
-- Get cart with items and current product prices
SELECT
    c.id, c.user_id, c.session_id, c.status, c.coupon_code,
    c.created_at, c.updated_at, c.expires_at,
    json_agg(json_build_object(
        'id', ci.id,
        'product_id', ci.product_id,
        'variant_id', ci.variant_id,
        'quantity', ci.quantity,
        'unit_price', ci.unit_price,
        'product_name', p.name,
        'variant_name', pv.name,
        'sku', COALESCE(pv.sku, p.sku),
        'image_url', COALESCE(pv.image_url, (SELECT url FROM rc_product_images WHERE product_id = p.id AND is_primary = true LIMIT 1)),
        'stock_status', COALESCE(pv.stock_status, p.stock_status),
        'stock_quantity', COALESCE(pv.stock_quantity, p.stock_quantity),
        'line_total', (ci.quantity * ci.unit_price)
    ) ORDER BY ci.created_at) AS items,
    SUM(ci.quantity) AS item_count,
    COUNT(DISTINCT ci.id) AS unique_item_count,
    SUM(ci.quantity * ci.unit_price) AS subtotal
FROM rc_carts c
LEFT JOIN rc_cart_items ci ON ci.cart_id = c.id
LEFT JOIN rc_products p ON p.id = ci.product_id
LEFT JOIN rc_product_variants pv ON pv.id = ci.variant_id
WHERE (c.user_id = $1 OR c.session_id = $2)
    AND c.status = 'active'
GROUP BY c.id;
```

### 8.5 Tax Calculation Query

```sql
-- Find applicable tax rates for an address and tax class
SELECT id, name, rate, compound, shipping, priority
FROM rc_tax_rates
WHERE enabled = true
    AND country = $1
    AND (state IS NULL OR state = $2)
    AND (postal_code IS NULL OR $3 LIKE REPLACE(postal_code, '*', '%'))
    AND (city IS NULL OR city = $4)
    AND tax_class = $5
ORDER BY priority ASC, rate DESC;
```

### 8.6 Order with Items

```sql
-- Full order detail
SELECT
    o.*,
    json_agg(json_build_object(
        'id', oi.id,
        'product_id', oi.product_id,
        'variant_id', oi.variant_id,
        'product_name', oi.product_name,
        'variant_name', oi.variant_name,
        'sku', oi.sku,
        'quantity', oi.quantity,
        'unit_price', oi.unit_price,
        'subtotal', oi.subtotal,
        'tax_amount', oi.tax_amount,
        'discount_amount', oi.discount_amount,
        'total', oi.total
    ) ORDER BY oi.created_at) AS items,
    (SELECT json_agg(json_build_object(
        'status', h.new_status,
        'timestamp', h.created_at,
        'note', h.note
    ) ORDER BY h.created_at)
    FROM rc_order_status_history h WHERE h.order_id = o.id) AS status_history
FROM rc_orders o
LEFT JOIN rc_order_items oi ON oi.order_id = o.id
WHERE o.id = $1
GROUP BY o.id;
```

### 8.7 Shipping Zone Matching

```sql
-- Find the best matching shipping zone for an address
SELECT sz.*, json_agg(row_to_json(sm.*)) AS methods
FROM rc_shipping_zones sz
LEFT JOIN rc_shipping_methods sm ON sm.zone_id = sz.id AND sm.enabled = true
WHERE
    -- Match by postal code (most specific)
    ($3 IS NOT NULL AND EXISTS (
        SELECT 1 FROM unnest(sz.postal_codes) pc WHERE $3 LIKE REPLACE(pc, '*', '%')
    ))
    -- Match by region
    OR ($2 IS NOT NULL AND $1 || '-' || $2 = ANY(sz.regions))
    -- Match by country
    OR ($1 = ANY(sz.countries))
    -- Default zone fallback
    OR sz.is_default = true
GROUP BY sz.id
ORDER BY
    CASE WHEN EXISTS (SELECT 1 FROM unnest(sz.postal_codes) pc WHERE $3 LIKE REPLACE(pc, '*', '%')) THEN 0
         WHEN $1 || '-' || $2 = ANY(sz.regions) THEN 1
         WHEN $1 = ANY(sz.countries) THEN 2
         WHEN sz.is_default THEN 3
    END ASC
LIMIT 1;
```

### 8.8 Stock Reservation Check

```sql
-- Check available stock (actual stock minus active reservations)
SELECT
    p.stock_quantity - COALESCE(
        (SELECT SUM(sr.quantity)
         FROM rc_stock_reservations sr
         WHERE sr.product_id = p.id
           AND sr.variant_id IS NULL
           AND sr.status = 'active'
           AND sr.expires_at > now()),
        0
    ) AS available_quantity
FROM rc_products p
WHERE p.id = $1;

-- Same for variant
SELECT
    pv.stock_quantity - COALESCE(
        (SELECT SUM(sr.quantity)
         FROM rc_stock_reservations sr
         WHERE sr.product_id = pv.product_id
           AND sr.variant_id = pv.id
           AND sr.status = 'active'
           AND sr.expires_at > now()),
        0
    ) AS available_quantity
FROM rc_product_variants pv
WHERE pv.id = $1;
```

### 8.9 Analytics: Revenue by Day

```sql
SELECT
    DATE(o.created_at AT TIME ZONE 'UTC') AS date,
    SUM(o.grand_total) AS gross_revenue,
    SUM(o.grand_total - o.tax_total - o.shipping_total) AS net_revenue,
    COUNT(*) AS order_count,
    AVG(o.grand_total) AS average_order_value,
    COALESCE(SUM(r.amount), 0) AS total_refunds
FROM rc_orders o
LEFT JOIN rc_refunds r ON r.order_id = o.id AND r.status = 'completed'
WHERE o.status NOT IN ('cancelled')
    AND o.payment_status IN ('paid', 'partially_refunded')
    AND o.created_at >= $1
    AND o.created_at <= $2
GROUP BY DATE(o.created_at AT TIME ZONE 'UTC')
ORDER BY date ASC;
```

### 8.10 Coupon Validation

```sql
-- Check coupon validity
SELECT c.*,
    (c.usage_limit IS NULL OR c.usage_count < c.usage_limit) AS within_global_limit,
    (c.usage_limit_per_user IS NULL OR
        (SELECT COUNT(*) FROM rc_coupon_usage cu
         WHERE cu.coupon_id = c.id AND cu.customer_id = $2) < c.usage_limit_per_user
    ) AS within_user_limit,
    (c.starts_at IS NULL OR c.starts_at <= now()) AS has_started,
    (c.expires_at IS NULL OR c.expires_at > now()) AS not_expired
FROM rc_coupons c
WHERE c.code = $1
    AND c.enabled = true;
```

### 8.11 Expired Reservation Cleanup (Background Job)

```sql
-- Release expired stock reservations
UPDATE rc_stock_reservations
SET status = 'expired'
WHERE status = 'active'
    AND expires_at <= now()
RETURNING product_id, variant_id, quantity;

-- Then for each returned row, restore stock:
-- UPDATE rc_products SET stock_quantity = stock_quantity + $quantity WHERE id = $product_id;
-- UPDATE rc_product_variants SET stock_quantity = stock_quantity + $quantity WHERE id = $variant_id;
```
