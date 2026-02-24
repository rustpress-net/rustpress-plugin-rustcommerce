# Infrastructure Architecture — RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Author**: Infrastructure Agent
**Status**: Draft

---

## 1. Deployment Topology

RustCommerce runs as a plugin within the RustPress CMS process. It does not introduce separate services or containers. The deployment topology is:

```
                       ┌─────────────────────────────────────────────┐
                       │              Load Balancer / Reverse Proxy  │
                       │          (nginx / Caddy / cloud ALB)        │
                       └──────────────────┬──────────────────────────┘
                                          │
                    ┌─────────────────────▼─────────────────────────┐
                    │          RustPress Server (Axum)               │
                    │  ┌──────────────────────────────────────────┐  │
                    │  │  Core Middleware Stack                   │  │
                    │  │  (compression, tracing, CORS, rate       │  │
                    │  │   limiting, security headers, auth)      │  │
                    │  └──────────────────────────────────────────┘  │
                    │                                                │
                    │  ┌─────────────┐  ┌─────────────────────────┐  │
                    │  │ RustPress   │  │ RustCommerce Plugin     │  │
                    │  │ Core Routes │  │ /api/v1/rustcommerce/*  │  │
                    │  │ /api/v1/*   │  │                         │  │
                    │  │ /admin      │  │ - Product handlers      │  │
                    │  │ /health     │  │ - Cart handlers         │  │
                    │  └─────┬───────┘  │ - Checkout handlers     │  │
                    │        │          │ - Order handlers        │  │
                    │        │          │ - Webhook handlers      │  │
                    │        │          │ - Admin handlers        │  │
                    │        │          └────────┬────────────────┘  │
                    │        │                   │                   │
                    │  ┌─────▼───────────────────▼────────────────┐  │
                    │  │          Shared AppContext                │  │
                    │  │  - PgPool (sqlx)                         │  │
                    │  │  - CacheProvider (Redis / moka)          │  │
                    │  │  - StorageBackend (local / S3)           │  │
                    │  │  - HookRegistry                          │  │
                    │  │  - Config                                │  │
                    │  └─────┬──────────────┬─────────────────────┘  │
                    └────────┼──────────────┼────────────────────────┘
                             │              │
                    ┌────────▼──────┐  ┌────▼───────────┐
                    │ PostgreSQL 16 │  │ Redis 7        │
                    │ (primary DB)  │  │ (cache layer)  │
                    └───────────────┘  └────────────────┘
```

### Key Architectural Decisions

1. **In-process plugin**: RustCommerce runs inside the RustPress process as a dynamically registered plugin. No sidecar containers, no separate microservices.
2. **Shared infrastructure**: The plugin reuses the PostgreSQL connection pool, Redis cache, storage backend, and authentication system already provisioned by RustPress core.
3. **Plugin isolation via namespacing**: All database tables use the `rc_` prefix. All API routes are under `/api/v1/rustcommerce/`. All hooks use the `rustcommerce.` prefix. All cache keys use the `rc:` prefix.

---

## 2. Database Schema Isolation

### 2.1 Table Naming Convention

All RustCommerce tables live in the same PostgreSQL database as RustPress core, but are prefixed with `rc_` to prevent collisions:

| Table Name | Purpose |
|------------|---------|
| `rc_products` | Product catalog (title, slug, description, price, status) |
| `rc_product_variants` | Size/color/material variants per product |
| `rc_product_images` | Image references per product (FK to RustPress media) |
| `rc_categories` | Hierarchical product categories |
| `rc_product_categories` | Many-to-many: products to categories |
| `rc_tags` | Flat product tags |
| `rc_product_tags` | Many-to-many: products to tags |
| `rc_carts` | Shopping cart per session/user |
| `rc_cart_items` | Line items within a cart |
| `rc_orders` | Orders with status workflow |
| `rc_order_items` | Line items within an order |
| `rc_customers` | Customer accounts (may link to RustPress users) |
| `rc_addresses` | Shipping/billing addresses |
| `rc_payments` | Payment records (Stripe references) |
| `rc_shipping_methods` | Configured shipping methods |
| `rc_shipping_zones` | Geographic shipping zones |
| `rc_tax_rates` | Tax rate definitions |
| `rc_tax_zones` | Geographic tax zones |
| `rc_coupons` | Discount codes and rules |
| `rc_reviews` | Product reviews and ratings |
| `rc_inventory_log` | Stock change audit trail |
| `rc_store_settings` | Plugin-specific key-value settings |

### 2.2 Primary Key Strategy

All tables use UUID v4 primary keys, consistent with RustPress core:

```sql
CREATE TABLE rc_products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ...
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 2.3 Foreign Key References to Core

RustCommerce tables reference RustPress core tables where needed:

- `rc_customers.user_id` references `users.id` (optional; guest customers have NULL)
- `rc_product_images.media_id` references `media.id` (leveraging RustPress media system)
- `rc_orders.created_by` references `users.id` (admin who manually created an order)

### 2.4 Migration Management

Migrations are stored in the plugin's `migrations/` directory and tracked via a dedicated version table (`rc_migrations`), as declared in `plugin.toml`:

```toml
[migrations]
directory = "migrations"
version_table = "rc_migrations"
```

Migration files follow the convention `NNNNN_description.sql`:

| File | Content |
|------|---------|
| `00001_ecommerce_core.sql` | Products, categories, variants, tags, images |
| `00002_cart_and_orders.sql` | Cart, cart items, orders, order items |
| `00003_customers.sql` | Customers, addresses |
| `00004_payments.sql` | Payments, transactions |
| `00005_shipping_and_tax.sql` | Shipping methods/zones, tax rates/zones |
| `00006_coupons.sql` | Coupons, discount rules |
| `00007_reviews.sql` | Product reviews |

### 2.5 Index Strategy

Performance-critical indexes:

```sql
-- Product listing performance
CREATE INDEX idx_rc_products_status ON rc_products(status) WHERE status = 'published';
CREATE INDEX idx_rc_products_slug ON rc_products(slug);
CREATE INDEX idx_rc_products_created ON rc_products(created_at DESC);
CREATE INDEX idx_rc_products_price ON rc_products(price);

-- Category browsing
CREATE INDEX idx_rc_categories_parent ON rc_categories(parent_id);
CREATE INDEX idx_rc_categories_slug ON rc_categories(slug);
CREATE INDEX idx_rc_product_categories_product ON rc_product_categories(product_id);
CREATE INDEX idx_rc_product_categories_category ON rc_product_categories(category_id);

-- Order lookups
CREATE INDEX idx_rc_orders_customer ON rc_orders(customer_id);
CREATE INDEX idx_rc_orders_status ON rc_orders(status);
CREATE INDEX idx_rc_orders_created ON rc_orders(created_at DESC);
CREATE INDEX idx_rc_order_items_order ON rc_order_items(order_id);

-- Cart performance
CREATE INDEX idx_rc_carts_session ON rc_carts(session_id);
CREATE INDEX idx_rc_carts_user ON rc_carts(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_rc_cart_items_cart ON rc_cart_items(cart_id);

-- Search support
CREATE INDEX idx_rc_products_search ON rc_products USING gin(
    to_tsvector('english', title || ' ' || COALESCE(description, ''))
);

-- Inventory
CREATE INDEX idx_rc_product_variants_product ON rc_product_variants(product_id);
CREATE INDEX idx_rc_product_variants_sku ON rc_product_variants(sku);
```

---

## 3. Cache Layer Design

RustCommerce leverages RustPress's existing cache infrastructure (`rustpress-cache` crate), which provides Redis 7 as the primary cache with `moka` (in-memory) as a fallback.

### 3.1 Cache Key Namespace

All cache keys use the `rc:` prefix to avoid collisions with core cache entries:

```
rc:product:{id}                    -- Single product by ID
rc:product:slug:{slug}             -- Product by slug
rc:products:list:{hash}            -- Product listing (hash of query params)
rc:products:count:{hash}           -- Product count for a query
rc:category:{id}                   -- Single category
rc:categories:tree                 -- Full category hierarchy
rc:cart:{session_id}               -- Cart contents
rc:cart:{user_id}                  -- Cart for logged-in user
rc:inventory:{variant_id}          -- Stock level for a variant
rc:shipping:methods:{zone_hash}    -- Shipping methods for a zone
rc:tax:rate:{zone_hash}            -- Tax rate for a zone
rc:store:settings                  -- Store-wide settings
rc:product:{id}:reviews:avg        -- Average review rating
```

### 3.2 Cache TTL Strategy

| Cache Category | TTL | Rationale |
|----------------|-----|-----------|
| Product catalog (listings, details) | 5 minutes | Products change infrequently; cache warmth is high |
| Category tree | 15 minutes | Categories rarely change |
| Store settings | 30 minutes | Settings almost never change |
| Cart contents | No TTL (session-bound) | Must be fresh; invalidated on mutation |
| Inventory/stock levels | 30 seconds | Must stay near-real-time for accuracy |
| Tax/shipping rates | 1 hour | Configuration data, changes are rare |
| Review aggregates | 10 minutes | Acceptable staleness for star ratings |

### 3.3 Cache Invalidation

RustCommerce uses a write-through invalidation strategy:

1. **On product update**: Delete `rc:product:{id}`, `rc:product:slug:{slug}`, and all matching `rc:products:list:*` keys (pattern-based invalidation via Redis SCAN).
2. **On category update**: Delete `rc:category:{id}` and `rc:categories:tree`.
3. **On cart mutation**: Delete `rc:cart:{session_or_user_id}`.
4. **On inventory change**: Delete `rc:inventory:{variant_id}`.
5. **On settings change**: Delete `rc:store:settings`.

Hook integration for cross-plugin invalidation:

```rust
// When a product is updated, fire a hook so other plugins can react
hook_registry.do_action("rustcommerce.product_updated", &product_data).await?;

// Internally, invalidate cache
cache.delete_pattern("rc:products:list:*").await?;
cache.delete(&format!("rc:product:{}", product_id)).await?;
```

### 3.4 Cache Fallback (No Redis)

When Redis is unavailable, RustPress falls back to `moka` in-memory cache. RustCommerce works identically in both modes because it accesses the cache through the `CacheProvider` trait abstraction in `AppContext`. The only behavioral difference is that in-memory cache is not shared across multiple RustPress instances in a horizontally-scaled deployment.

---

## 4. File Storage for Product Images

### 4.1 Storage Backend Integration

Product images are stored via RustPress's `rustpress-storage` crate, which provides a `StorageBackend` trait with implementations for:

| Backend | Config Key | Use Case |
|---------|-----------|----------|
| **Local filesystem** | `storage.backend = "local"` | Development, single-server production |
| **Amazon S3** | `storage.backend = "s3"` | Production, multi-server |
| **Azure Blob Storage** | `storage.backend = "azure"` | Azure-hosted deployments |
| **Google Cloud Storage** | `storage.backend = "gcs"` | GCP-hosted deployments |

RustCommerce uses the `rustpress-media` crate to upload and manage images, which handles:
- Automatic thumbnail generation (multiple sizes for product grids, detail pages, cart)
- WebP conversion for modern browsers
- `srcset` generation for responsive images
- EXIF stripping for privacy

### 4.2 Product Image Storage Path

Images are organized under the RustPress media directory with a plugin-specific prefix:

```
uploads/
  rustcommerce/
    products/
      {product_id}/
        original.jpg          -- Original upload
        thumbnail_150x150.webp
        medium_600x600.webp
        large_1200x1200.webp
    categories/
      {category_id}/
        banner.jpg
```

The database stores references (not file paths) to media entries:

```sql
CREATE TABLE rc_product_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    media_id UUID NOT NULL REFERENCES media(id),
    position INTEGER NOT NULL DEFAULT 0,     -- Display ordering
    is_primary BOOLEAN NOT NULL DEFAULT false,
    alt_text VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### 4.3 Image Processing Pipeline

```
Upload → Validate (type, size) → Store Original → Generate Variants
                                                       │
                                                       ├── 150x150  (thumbnail, cart, grid)
                                                       ├── 600x600  (product listing card)
                                                       └── 1200x1200 (product detail page)
```

Configuration in store settings:

| Setting | Default | Description |
|---------|---------|-------------|
| `max_image_size_mb` | 10 | Maximum upload size |
| `allowed_image_types` | jpg, png, webp, gif | Accepted MIME types |
| `thumbnail_sizes` | 150, 600, 1200 | Generated variant widths |
| `enable_webp_conversion` | true | Auto-convert to WebP |

---

## 5. CDN Considerations

### 5.1 Integration with RustPress CDN Crate

RustPress provides `rustpress-cdn` with support for:
- **Cloudflare** (via the `rustcloudflare` plugin)
- **BunnyCDN**
- Custom CDN via configuration

When a CDN is configured, product image URLs are automatically rewritten:

```
Without CDN: https://store.example.com/uploads/rustcommerce/products/{id}/medium_600x600.webp
With CDN:    https://cdn.example.com/uploads/rustcommerce/products/{id}/medium_600x600.webp
```

### 5.2 Cache Headers for Product Images

RustCommerce sets appropriate cache headers on image responses:

| Asset Type | Cache-Control | Rationale |
|------------|---------------|-----------|
| Product images | `public, max-age=31536000, immutable` | Images are content-addressed; new upload = new URL |
| Category banners | `public, max-age=86400` | May change occasionally |
| Thumbnails | `public, max-age=31536000, immutable` | Generated deterministically from original |

### 5.3 CDN Purge on Image Update

When a product image is replaced or deleted:

1. Delete the old file from the storage backend.
2. If CDN is configured, issue a purge request for the old URL pattern.
3. The `rustpress-cdn` crate provides a `purge_url(url)` method that RustCommerce calls.

### 5.4 Image Optimization Recommendations

| Scale | Recommendation |
|-------|---------------|
| Small store (< 100 products) | Local storage with RustPress-served images; no CDN needed |
| Medium store (< 10K products) | S3 storage backend + Cloudflare CDN with Polish (image optimization) |
| Large store (< 100K products) | S3 storage + Cloudflare CDN + R2 (zero egress) or dedicated image CDN (imgix, Cloudinary) |

---

## 6. Infrastructure Dependencies Summary

| Component | Provider | Required | Shared with Core |
|-----------|----------|----------|------------------|
| PostgreSQL 16 | Docker / managed (RDS, Cloud SQL) | Yes | Yes |
| Redis 7 | Docker / managed (ElastiCache, Memorystore) | No (moka fallback) | Yes |
| File storage | Local / S3 / Azure / GCS | Yes | Yes |
| CDN | Cloudflare / BunnyCDN / none | No | Yes |
| Stripe API | stripe.com | Yes (for payments) | No |
| SMTP | Any (for email notifications) | No (optional) | Yes |

RustCommerce adds zero new infrastructure services. It operates entirely within the existing RustPress deployment footprint, accessing shared resources through the `AppContext` abstraction layer.
