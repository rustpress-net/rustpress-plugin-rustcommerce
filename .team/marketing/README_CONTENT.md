# Draft README.md Content — RustCommerce

**Document Version**: 1.0
**Date**: 2026-02-24
**Author**: Marketing Strategist
**Note**: This is the draft content for the repository's top-level `README.md`. Copy and adapt as needed.

---

*Begin README content below:*

---

<div align="center">

# RustCommerce

**Blazing-fast e-commerce, forged in Rust.**

A full-featured, high-performance e-commerce plugin for [RustPress CMS](https://github.com/rustpress-net/rustpress-core-base).

[![Crates.io](https://img.shields.io/crates/v/rustcommerce.svg)](https://crates.io/crates/rustcommerce)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/actions/workflows/ci.yml/badge.svg)](https://github.com/rustpress-net/rustpress-plugin-rustcommerce/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16-336791.svg)](https://www.postgresql.org/)

[Features](#features) | [Quick Start](#quick-start) | [Documentation](#documentation) | [API Reference](#api-reference) | [Contributing](#contributing)

</div>

---

## Overview

RustCommerce adds complete e-commerce functionality to any RustPress site. Products, categories, shopping cart, checkout, Stripe payments, order management, inventory tracking, and a full admin dashboard — all running as a native RustPress plugin with sub-100ms API responses.

<!-- Screenshot placeholder -->
<!-- ![RustCommerce Admin Dashboard](docs/images/dashboard-screenshot.png) -->

---

## Features

### Core Commerce
- **Product Management** — Full CRUD with variants (size, color), categories, tags, images, SKU tracking, and draft/published/archived status
- **Shopping Cart** — Server-side persistent cart (logged-in users) + client-side cart (guests) with real-time totals
- **Checkout Flow** — Multi-step checkout with guest checkout support: shipping address -> shipping method -> payment -> confirmation
- **Order Management** — Complete order lifecycle: Pending -> Processing -> Shipped -> Delivered / Cancelled / Refunded
- **Stripe Payments** — Payment Intent API with secure webhook verification. No credit card data stored locally.
- **Inventory Tracking** — Per-product and per-variant stock management, low-stock alerts, backorder configuration

### Admin Dashboard
- **Store Metrics** — Revenue, order count, average order value, best sellers, conversion data
- **Product Editor** — Rich product creation/editing with variant management, image upload, and SEO fields
- **Order Management** — Order list with filtering, detail views, status updates, and refund processing
- **Customer Management** — Customer list, order history, address management
- **Store Settings** — Currency, tax rules (flat rate + zone-based), shipping methods, store policies

### Technical
- **REST API** — Full CRUD API under `/api/v1/rustcommerce/` for headless storefront development
- **Hook System** — Fires RustPress hooks on key events (`order_created`, `payment_completed`, `product_updated`)
- **Database Migrations** — PostgreSQL schema managed through versioned migration files
- **Extensible Payments** — Gateway interface for adding PayPal, Square, or custom payment processors

### Post-MVP (Planned)
- Coupon and discount system
- Customer reviews and ratings
- Wishlist functionality
- Faceted search and filtering
- Email notifications (order confirmation, shipping updates)
- CSV product import/export
- Store analytics dashboard

---

## Quick Start

### Prerequisites

- [RustPress CMS](https://github.com/rustpress-net/rustpress-core-base) installed and running
- Rust 1.75 or later
- PostgreSQL 16
- Node.js 20+ (for admin UI development)
- Stripe account (for payment processing)

### Installation

**Option 1: Via RustPress Plugin Manager** (recommended)

```bash
rustpress plugin install rustcommerce
```

**Option 2: Manual Installation**

1. Clone the repository:

```bash
git clone https://github.com/rustpress-net/rustpress-plugin-rustcommerce.git
cd rustpress-plugin-rustcommerce
```

2. Build the plugin:

```bash
cargo build --release
```

3. Copy the compiled plugin to your RustPress plugins directory:

```bash
cp target/release/librustcommerce.so /path/to/rustpress/plugins/
# or on macOS:
cp target/release/librustcommerce.dylib /path/to/rustpress/plugins/
```

4. Activate the plugin in RustPress admin panel or via CLI:

```bash
rustpress plugin activate rustcommerce
```

### Configuration

1. Navigate to **Admin > RustCommerce > Settings** in the RustPress dashboard.

2. Configure your store basics:
   - Store name and currency
   - Tax rates and zones
   - Shipping methods and rates

3. Set up Stripe:
   - Enter your Stripe API keys (publishable and secret)
   - Configure the webhook endpoint URL: `https://yoursite.com/api/v1/rustcommerce/webhooks/stripe`
   - Add the webhook signing secret

4. Add your first product:
   - Navigate to **Admin > RustCommerce > Products > Add New**
   - Fill in product details, pricing, and inventory
   - Set status to "Published"

5. Your store is live. Customers can browse products via the storefront API or theme-rendered pages.

---

## Documentation

### Architecture

RustCommerce follows a clean layered architecture:

```
handlers/       API route handlers (Axum)
    |
services/       Business logic layer
    |
repositories/   Database access layer (sqlx)
    |
models/         Data structures and types
```

### Project Structure

```
rustpress-plugin-rustcommerce/
├── src/
│   ├── lib.rs                  # Plugin entry point
│   ├── plugin.rs               # RustCommercePlugin struct (Plugin trait impl)
│   ├── config.rs               # Plugin configuration
│   ├── error.rs                # Error types
│   ├── models/                 # Database models
│   │   ├── product.rs          # Product, ProductVariant
│   │   ├── category.rs         # ProductCategory
│   │   ├── cart.rs             # Cart, CartItem
│   │   ├── order.rs            # Order, OrderItem, OrderStatus
│   │   ├── customer.rs         # Customer, Address
│   │   ├── payment.rs          # Payment, PaymentMethod
│   │   ├── shipping.rs         # ShippingMethod, ShippingZone
│   │   ├── tax.rs              # TaxRate, TaxZone
│   │   ├── coupon.rs           # Coupon, DiscountType
│   │   └── review.rs           # ProductReview
│   ├── repositories/           # Database queries
│   ├── services/               # Business logic
│   ├── handlers/               # HTTP handlers
│   ├── routes.rs               # Route definitions
│   ├── hooks.rs                # Hook registrations
│   └── middleware.rs           # Plugin middleware
├── migrations/                 # PostgreSQL migration files
├── plugin.toml                 # Plugin manifest
├── Cargo.toml
└── README.md
```

### Admin UI

The admin interface is built with React 18 + TypeScript + Tailwind CSS and integrates into the RustPress admin dashboard. Source lives in the RustPress admin UI repository:

```
src/pages/plugins/rustcommerce/
├── components/
│   ├── Dashboard.tsx           # Store metrics
│   ├── ProductEditor.tsx       # Product create/edit
│   ├── ProductList.tsx         # Product listing
│   ├── OrderList.tsx           # Order management
│   ├── OrderDetail.tsx         # Order detail view
│   ├── CustomerList.tsx        # Customer management
│   └── settings/               # Settings pages
├── stores/
│   └── commerceStore.ts        # Zustand state management
├── api/
│   └── commerceApi.ts          # API client
└── types/
    └── index.ts                # TypeScript types
```

---

## API Reference

All API endpoints are served under `/api/v1/rustcommerce/`. Authentication uses RustPress JWT tokens.

### Products

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/rustcommerce/products` | List products (paginated, filterable) |
| `GET` | `/api/v1/rustcommerce/products/:id` | Get product detail |
| `POST` | `/api/v1/rustcommerce/products` | Create product (admin) |
| `PUT` | `/api/v1/rustcommerce/products/:id` | Update product (admin) |
| `DELETE` | `/api/v1/rustcommerce/products/:id` | Delete product (admin) |

### Categories

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/rustcommerce/categories` | List categories |
| `GET` | `/api/v1/rustcommerce/categories/:id` | Get category detail |
| `POST` | `/api/v1/rustcommerce/categories` | Create category (admin) |
| `PUT` | `/api/v1/rustcommerce/categories/:id` | Update category (admin) |
| `DELETE` | `/api/v1/rustcommerce/categories/:id` | Delete category (admin) |

### Cart

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/rustcommerce/cart` | Get current cart |
| `POST` | `/api/v1/rustcommerce/cart/items` | Add item to cart |
| `PUT` | `/api/v1/rustcommerce/cart/items/:id` | Update cart item quantity |
| `DELETE` | `/api/v1/rustcommerce/cart/items/:id` | Remove item from cart |
| `DELETE` | `/api/v1/rustcommerce/cart` | Clear cart |

### Checkout

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/rustcommerce/checkout` | Initiate checkout from cart |
| `POST` | `/api/v1/rustcommerce/checkout/shipping` | Set shipping address and method |
| `POST` | `/api/v1/rustcommerce/checkout/payment` | Create payment intent (Stripe) |
| `POST` | `/api/v1/rustcommerce/checkout/confirm` | Confirm and place order |

### Orders

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/rustcommerce/orders` | List orders (admin: all; customer: own) |
| `GET` | `/api/v1/rustcommerce/orders/:id` | Get order detail |
| `PUT` | `/api/v1/rustcommerce/orders/:id/status` | Update order status (admin) |
| `POST` | `/api/v1/rustcommerce/orders/:id/refund` | Process refund (admin) |

### Customers

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/rustcommerce/customers` | List customers (admin) |
| `GET` | `/api/v1/rustcommerce/customers/:id` | Get customer detail |
| `GET` | `/api/v1/rustcommerce/customers/:id/orders` | Get customer orders |

### Webhooks

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/rustcommerce/webhooks/stripe` | Stripe webhook endpoint |

### Example: Create a Product

```bash
curl -X POST https://yoursite.com/api/v1/rustcommerce/products \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Classic T-Shirt",
    "description": "A comfortable cotton t-shirt.",
    "price": 29.99,
    "sku": "TSHIRT-001",
    "stock": 100,
    "status": "published",
    "categories": ["apparel"],
    "variants": [
      { "name": "Small", "sku": "TSHIRT-001-S", "stock": 30 },
      { "name": "Medium", "sku": "TSHIRT-001-M", "stock": 40 },
      { "name": "Large", "sku": "TSHIRT-001-L", "stock": 30 }
    ]
  }'
```

### Example: Add to Cart

```bash
curl -X POST https://yoursite.com/api/v1/rustcommerce/cart/items \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "product_id": "550e8400-e29b-41d4-a716-446655440000",
    "variant_id": "660e8400-e29b-41d4-a716-446655440001",
    "quantity": 2
  }'
```

For the full API specification, see the [OpenAPI documentation](docs/api/openapi.yaml).

---

## Performance

RustCommerce is designed for speed:

| Metric | Target |
|--------|--------|
| Product listing API (cached) | < 100ms |
| Checkout flow (end-to-end) | < 3 seconds |
| Concurrent shoppers per instance | 100+ |
| Memory footprint | Minimal (no GC, no interpreter) |

<!-- Performance benchmark results will be published here after v1.0 release. -->

---

## Security

- **Memory Safety**: Rust's ownership model eliminates buffer overflows, use-after-free, and null pointer dereferences at compile time.
- **PCI-DSS Aware**: Zero local storage of credit card data. All payment processing delegated to Stripe.
- **CSRF Protection**: All checkout and mutation endpoints are CSRF-protected.
- **Rate Limiting**: Checkout and payment endpoints are rate-limited to prevent abuse.
- **Input Validation**: All user input is validated and sanitized. SQL injection and XSS are prevented by design (sqlx parameterized queries, proper escaping).
- **Webhook Verification**: Stripe webhooks are verified using the webhook signing secret.

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend Language | Rust |
| Web Framework | Axum |
| Async Runtime | Tokio |
| Database | PostgreSQL 16 (via sqlx) |
| Admin UI | React 18 + TypeScript + Tailwind CSS |
| State Management | Zustand 5.0 |
| Build (Rust) | Cargo |
| Build (Frontend) | Vite |
| Payments | Stripe |
| Authentication | RustPress JWT |

---

## Contributing

We welcome contributions. Please read our [Contributing Guide](CONTRIBUTING.md) before submitting a pull request.

### Development Setup

1. **Clone the repository**:

```bash
git clone https://github.com/rustpress-net/rustpress-plugin-rustcommerce.git
cd rustpress-plugin-rustcommerce
```

2. **Install Rust** (if not already installed):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

3. **Set up PostgreSQL** and create a development database:

```bash
createdb rustcommerce_dev
```

4. **Set environment variables**:

```bash
export DATABASE_URL="postgresql://localhost/rustcommerce_dev"
export STRIPE_SECRET_KEY="sk_test_..."
export STRIPE_WEBHOOK_SECRET="whsec_..."
```

5. **Run migrations**:

```bash
sqlx migrate run
```

6. **Build and test**:

```bash
cargo build
cargo test
```

7. **Run with RustPress** (from your RustPress installation):

```bash
rustpress serve --plugin-dev /path/to/rustpress-plugin-rustcommerce
```

### Code Style

- Rust: Follow `rustfmt` defaults. Run `cargo fmt` before committing.
- Lint: Run `cargo clippy` and resolve all warnings.
- Tests: Add tests for all new business logic in `services/` and `handlers/`.
- Commits: Use [Conventional Commits](https://www.conventionalcommits.org/) format.

### Pull Request Process

1. Fork the repository and create a feature branch.
2. Write tests for your changes.
3. Ensure `cargo test`, `cargo fmt --check`, and `cargo clippy` all pass.
4. Submit a pull request with a clear description of the changes.
5. Respond to review feedback.

---

## Roadmap

- [x] Project skeleton and plugin manifest
- [ ] Database schema and migrations (M1)
- [ ] Product CRUD and category system (M1)
- [ ] Cart and checkout flow (M2)
- [ ] Stripe payment integration (M2)
- [ ] Admin dashboard (M3)
- [ ] Storefront API and hooks (M4)
- [ ] Coupons, reviews, and email notifications (M4)
- [ ] Testing and documentation (M5)
- [ ] v1.0 Release

---

## License

RustCommerce is open source software licensed under the [MIT License](LICENSE).

---

## Acknowledgments

- Built on [RustPress CMS](https://github.com/rustpress-net/rustpress-core-base)
- Payment processing by [Stripe](https://stripe.com)
- Inspired by [WooCommerce](https://woocommerce.com), [Medusa](https://medusajs.com), and [Saleor](https://saleor.io)

---

<div align="center">

**[Documentation](docs/)** | **[API Reference](docs/api/)** | **[Changelog](CHANGELOG.md)** | **[License](LICENSE)**

Made with Rust. Built for speed.

</div>
