# RustCommerce - Enterprise E-Commerce for RustPress

> A full-featured, high-performance e-commerce plugin for RustPress CMS built entirely in Rust and React.

![RustCommerce Dashboard](/.team/screenshots/02_store_dashboard.png)

## Overview

RustCommerce transforms any RustPress CMS site into a fully functional online store. Built with Rust for blazing-fast performance and React for a modern admin experience, it provides everything you need to sell products online.

## Features

### Product Management
- Full CRUD with title, description, pricing, SKU, stock tracking
- Product variants (size, color, material) with independent pricing and inventory
- Hierarchical category system with unlimited nesting
- Product image gallery with drag-and-drop ordering
- Product status workflow (Draft -> Published -> Archived)
- Bulk operations (publish, archive, delete)

![Product Editor](/.team/screenshots/04_create_product.png)
![Product List](/.team/screenshots/09_product_list_filters.png)

### Shopping Cart & Checkout
- Server-side persistent cart for logged-in users
- Client-side cart for guest shoppers
- Multi-step checkout: Address -> Shipping -> Payment -> Confirmation
- Guest checkout support
- Real-time cart totals with tax and shipping preview
- Coupon/discount code application

### Order Management
- Complete order lifecycle (Pending -> Processing -> Shipped -> Delivered)
- Order detail view with items, addresses, timeline, and notes
- One-click status updates with automatic email notifications
- Refund processing via Stripe
- Order search and status filtering

![Order Management](/.team/screenshots/13_order_list.png)
![Order Detail](/.team/screenshots/14_order_detail.png)

### Payment Processing
- Stripe integration with PaymentIntent API
- Secure webhook handling with signature verification
- Test mode for development
- PCI-DSS compliant (no card data stored locally)
- Extensible payment gateway interface

### Store Dashboard
- Real-time revenue, orders, customers, and AOV metrics
- Revenue trend chart with period selection (7d/30d/90d)
- Order status distribution pie chart
- Top selling products ranking
- Recent orders feed

![Dashboard](/.team/screenshots/03_dashboard_metrics.png)

### Customer Management
- Customer profiles with contact info and addresses
- Order history per customer
- Total spend tracking
- Customer search and export

![Customers](/.team/screenshots/16_customer_list.png)

### Store Settings
- **General**: Store name, currency, weight/dimension units
- **Payments**: Stripe API keys, test mode, payment methods
- **Shipping**: Zones with multiple methods (flat rate, free shipping, weight-based)
- **Tax**: Rates by country/state, percentage or fixed, compound support
- **Email**: Notification templates for order events

![Settings](/.team/screenshots/18_general_settings.png)

### Advanced Features
- Coupon/discount system (percentage, fixed amount, free shipping)
- Product reviews with star ratings and moderation queue
- Product search with faceted filtering
- Response caching for < 100ms API responses
- Rate limiting on checkout endpoints
- Hook system for plugin extensibility

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Backend | Rust 2021, Axum 0.7, Tokio |
| Database | PostgreSQL 16, sqlx (compile-time checked) |
| Frontend | React 18, TypeScript, Tailwind CSS |
| State | Zustand 5.0 with persist |
| Charts | Recharts |
| Payments | Stripe (async-stripe) |
| Icons | Lucide React |
| Build | Cargo + Vite |

## Architecture

```
+-------------------------------------------------------+
|                   Admin UI (React)                     |
|  Dashboard | Products | Orders | Customers | Settings |
+-------------------------------------------------------+
|                  REST API (Axum)                       |
|    /api/v1/rustcommerce/* | /api/v1/admin/*            |
+-------------------------------------------------------+
|                Handlers (HTTP Layer)                    |
+-------------------------------------------------------+
|              Services (Business Logic)                  |
|  Product | Cart | Checkout | Order | Payment | ...     |
+-------------------------------------------------------+
|            Repositories (Data Access)                   |
|              sqlx queries to PostgreSQL                 |
+-------------------------------------------------------+
|              Models (Domain Types)                      |
|  UUID IDs | Decimal money | DateTime timestamps        |
+-------------------------------------------------------+
|                 PostgreSQL 16                           |
|  14 tables (rc_* prefix) | 7 migrations                |
+-------------------------------------------------------+
```

## Quick Start

### Prerequisites
- RustPress v0.4.0+
- Docker & Docker Compose (for PostgreSQL + Redis)
- Rust 1.70+ and Cargo
- Node.js 20+ and npm

### Installation

1. **Start infrastructure**:
```bash
docker-compose up -d  # PostgreSQL 16 + Redis 7
```

2. **Build the plugin**:
```bash
cd rustpress-plugin-rustcommerce
cargo build --release
```

3. **Build the admin UI**:
```bash
cd rustpress-core-admin-ui
npm install && npm run build
```

4. **Activate the plugin** in the RustPress admin panel under Plugins -> RustCommerce -> Activate

5. **Configure your store** at Settings -> General (currency, store name)

6. **Set up payments** at Settings -> Payments (add Stripe API keys)

## API Endpoints

### Public API
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/rustcommerce/products` | List products |
| GET | `/api/v1/rustcommerce/products/:id` | Get product |
| GET | `/api/v1/rustcommerce/categories` | List categories |
| POST | `/api/v1/rustcommerce/cart` | Create cart |
| POST | `/api/v1/rustcommerce/cart/:id/items` | Add to cart |
| POST | `/api/v1/rustcommerce/checkout` | Process checkout |

### Admin API
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/admin/rustcommerce/products` | Create product |
| PUT | `/api/v1/admin/rustcommerce/products/:id` | Update product |
| GET | `/api/v1/admin/rustcommerce/orders` | List orders |
| PUT | `/api/v1/admin/rustcommerce/orders/:id/status` | Update status |
| GET | `/api/v1/admin/rustcommerce/dashboard` | Dashboard stats |

*50+ endpoints total -- see `plugin.toml` for the complete list.*

## Database Schema

14 tables with `rc_` prefix:
- `rc_products`, `rc_product_variants`, `rc_product_images`
- `rc_categories`, `rc_product_categories`
- `rc_carts`, `rc_cart_items`
- `rc_orders`, `rc_order_items`
- `rc_customers`, `rc_customer_addresses`
- `rc_payments`
- `rc_shipping_zones`, `rc_shipping_methods`, `rc_tax_rates`
- `rc_coupons`, `rc_reviews`

## Hook System

RustCommerce fires hooks that other plugins can subscribe to:

| Hook | Trigger |
|------|---------|
| `product_created` | New product created |
| `product_updated` | Product modified |
| `product_deleted` | Product removed |
| `order_created` | New order placed |
| `order_status_changed` | Order status updated |
| `payment_completed` | Payment successful |
| `payment_failed` | Payment failed |
| `stock_low` | Stock below threshold |
| `customer_registered` | New customer |
| `coupon_applied` | Coupon used |

## Project Stats

| Metric | Value |
|--------|-------|
| Rust source files | 47 |
| Lines of Rust | 7,166 |
| SQL migration files | 7 |
| React/TypeScript files | 18 |
| Database tables | 14 |
| API endpoints | 50+ |
| Admin pages | 14 |
| Design artifacts | 56 |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and contribution guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

*Built with Rust for RustPress CMS by the RustPress Team*
