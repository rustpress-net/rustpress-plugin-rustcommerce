# Scalability Plan — RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Author**: Infrastructure Agent
**Status**: Draft

---

## 1. Horizontal Scaling via RustPress Multi-Instance

### 1.1 Scaling Architecture

RustPress supports horizontal scaling by running multiple instances behind a load balancer. RustCommerce is designed to be stateless at the application layer, making it fully compatible with multi-instance deployments.

```
                    ┌────────────────────┐
                    │   Load Balancer     │
                    │   (ALB / nginx)     │
                    └──┬──────┬──────┬───┘
                       │      │      │
              ┌────────▼┐  ┌──▼─────┐ ┌▼────────┐
              │ RustPress│  │RustPress│ │RustPress│
              │ Instance │  │Instance │ │Instance │
              │    #1    │  │   #2   │ │   #3   │
              └────┬─────┘  └──┬─────┘ └──┬─────┘
                   │           │          │
          ┌────────▼───────────▼──────────▼────────┐
          │              Shared State               │
          │  ┌──────────┐   ┌──────────┐            │
          │  │PostgreSQL │   │  Redis   │            │
          │  │  (primary │   │ (cache + │            │
          │  │  + replica)│  │  pubsub) │            │
          │  └──────────┘   └──────────┘            │
          └────────────────────────────────────────┘
```

### 1.2 Statelessness Requirements

For RustCommerce to scale horizontally, these state concerns must be externalized:

| State Type | Single Instance | Multi-Instance |
|------------|----------------|---------------|
| Shopping cart | In-memory or PostgreSQL | PostgreSQL (shared) |
| Session data | In-memory | Redis or PostgreSQL |
| Product cache | moka (in-memory) | Redis (shared) |
| File uploads | Local filesystem | S3 or shared NFS |
| WebSocket connections | In-process broadcast | Redis Pub/Sub relay |
| Background jobs | In-process queue | PostgreSQL-backed job queue (rustpress-jobs) |

### 1.3 Load Balancer Configuration

| Setting | Value | Rationale |
|---------|-------|-----------|
| Algorithm | Least connections | Even distribution across instances |
| Health check | `GET /health` every 10s | RustPress health endpoint |
| Session affinity | None (sticky sessions not required) | All state is externalized |
| Connection draining | 30 seconds | Allow in-flight requests to complete during deploys |
| WebSocket support | Enabled | Required for admin real-time notifications |

### 1.4 Scaling Triggers

| Metric | Scale Up Threshold | Scale Down Threshold | Cooldown |
|--------|-------------------|---------------------|----------|
| CPU utilization | > 70% for 3 min | < 30% for 10 min | 5 min |
| Request latency (p95) | > 200ms for 5 min | < 50ms for 10 min | 5 min |
| Active connections | > 1000 per instance | < 200 per instance | 5 min |
| Memory utilization | > 80% | < 40% | 5 min |

### 1.5 Instance Sizing Recommendations

| Scale | Instances | Instance Size | Total Capacity |
|-------|-----------|--------------|----------------|
| Small (< 50 orders/day) | 1 | 2 vCPU, 4 GB RAM | ~200 req/s |
| Medium (< 500 orders/day) | 2 | 2 vCPU, 4 GB RAM | ~400 req/s |
| Large (< 5000 orders/day) | 3-5 | 4 vCPU, 8 GB RAM | ~2000 req/s |
| Peak handling (flash sales) | Auto-scale to 10 | 4 vCPU, 8 GB RAM | ~4000 req/s |

> Rust/Axum is exceptionally efficient. A single 2-vCPU instance can handle 200+ requests/second for typical e-commerce workloads. These estimates are conservative.

---

## 2. Database Connection Pooling

### 2.1 sqlx Pool Configuration

RustCommerce shares the RustPress PostgreSQL connection pool configured via `sqlx`:

```rust
// Pool configuration in RustPress AppState
let pool = PgPoolOptions::new()
    .max_connections(max_pool_size)
    .min_connections(min_pool_size)
    .acquire_timeout(Duration::from_secs(5))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .test_before_acquire(true)
    .connect(&database_url)
    .await?;
```

### 2.2 Pool Size Recommendations

| Scale | RustPress Instances | Pool Size per Instance | Total Connections | PostgreSQL max_connections |
|-------|--------------------|-----------------------|-------------------|--------------------------|
| Small | 1 | 10 | 10 | 50 |
| Medium | 2 | 20 | 40 | 100 |
| Large | 5 | 25 | 125 | 200 |

**Formula**: `max_connections = (instances * pool_size_per_instance) + overhead(20)`

The overhead accounts for:
- PostgreSQL superuser connections (monitoring, maintenance)
- Migration runner connections
- Background job workers

### 2.3 Connection Pool Monitoring

Key metrics to monitor:

| Metric | Warning Threshold | Critical Threshold | Action |
|--------|------------------|-------------------|--------|
| Pool utilization | > 70% | > 90% | Increase pool size or add instance |
| Connection wait time | > 100ms | > 1s | Increase pool size |
| Idle connections | > 80% of pool | — | Decrease min_connections |
| Connection errors | > 0/min | > 5/min | Check PostgreSQL health |

### 2.4 PgBouncer (Optional, Large Scale)

For large-scale deployments with many RustPress instances, introduce PgBouncer as a connection pooler between application instances and PostgreSQL:

```
RustPress Instances (5x, 25 conn each = 125)
    │
    ▼
PgBouncer (transaction pooling, max 50 server connections)
    │
    ▼
PostgreSQL (max_connections = 100, with headroom for admin)
```

PgBouncer configuration:

```ini
[pgbouncer]
pool_mode = transaction          ; Release connection after each transaction
max_client_conn = 200            ; Accept up to 200 from app instances
default_pool_size = 40           ; Keep 40 server connections warm
reserve_pool_size = 10           ; Extra connections for burst
reserve_pool_timeout = 3         ; Wait 3s before using reserve
server_idle_timeout = 600        ; Close idle server connections after 10min
```

**When to introduce PgBouncer**: When total application connections exceed 50% of PostgreSQL `max_connections`, or when connection creation latency exceeds 50ms.

---

## 3. Cache Warming and Invalidation Strategy

### 3.1 Cache Warming

On application startup or after a cache flush, RustCommerce proactively warms critical cache entries:

```rust
pub async fn warm_cache(cache: &CacheProvider, db: &PgPool) -> Result<()> {
    // 1. Warm store settings (accessed on every request)
    let settings = store_settings_repo::load_all(db).await?;
    cache.set("rc:store:settings", &settings, Duration::from_secs(1800)).await?;

    // 2. Warm category tree (used for navigation on every page)
    let categories = category_repo::load_tree(db).await?;
    cache.set("rc:categories:tree", &categories, Duration::from_secs(900)).await?;

    // 3. Warm top 100 most-viewed products (covers most traffic)
    let top_products = product_repo::list_top_by_views(db, 100).await?;
    for product in &top_products {
        cache.set(
            &format!("rc:product:{}", product.id),
            product,
            Duration::from_secs(300),
        ).await?;
    }

    // 4. Warm shipping methods and tax rates
    let shipping = shipping_repo::list_active(db).await?;
    cache.set("rc:shipping:methods:all", &shipping, Duration::from_secs(3600)).await?;

    let tax_rates = tax_repo::list_active(db).await?;
    cache.set("rc:tax:rates:all", &tax_rates, Duration::from_secs(3600)).await?;

    tracing::info!("Cache warming complete: {} products, {} categories",
        top_products.len(), categories.len());
    Ok(())
}
```

### 3.2 Cache Invalidation Patterns

| Trigger | Invalidation Scope | Strategy |
|---------|-------------------|----------|
| Product created | Product list caches | Delete pattern `rc:products:list:*` |
| Product updated | Product detail + list caches | Delete `rc:product:{id}`, `rc:product:slug:{slug}`, pattern `rc:products:list:*` |
| Product deleted | Product detail + list caches + reviews | Delete `rc:product:{id}`, pattern `rc:products:list:*`, `rc:product:{id}:reviews:*` |
| Category updated | Category tree + product list caches | Delete `rc:categories:tree`, pattern `rc:products:list:*` |
| Inventory changed | Inventory cache for variant | Delete `rc:inventory:{variant_id}` |
| Order placed | Inventory caches for all items | Delete `rc:inventory:{variant_id}` for each item |
| Settings changed | Settings cache | Delete `rc:store:settings` |
| Coupon created/updated | No cache impact | Coupons validated against DB in real-time |
| Review submitted | Review aggregate cache | Delete `rc:product:{id}:reviews:avg` |

### 3.3 Stampede Prevention

When a popular cache key expires and multiple requests simultaneously attempt to regenerate it, a cache stampede occurs. RustCommerce prevents this using a lock-based approach:

```rust
pub async fn get_or_set_with_lock<T: Serialize + DeserializeOwned>(
    cache: &CacheProvider,
    key: &str,
    ttl: Duration,
    generate: impl Future<Output = Result<T>>,
) -> Result<T> {
    // Try cache first
    if let Some(cached) = cache.get::<T>(key).await? {
        return Ok(cached);
    }

    // Acquire a short-lived lock to prevent stampede
    let lock_key = format!("{}:lock", key);
    if cache.set_nx(&lock_key, "1", Duration::from_secs(5)).await? {
        // We got the lock — generate the value
        let value = generate.await?;
        cache.set(key, &value, ttl).await?;
        cache.delete(&lock_key).await?;
        Ok(value)
    } else {
        // Another instance is generating — wait briefly and retry from cache
        tokio::time::sleep(Duration::from_millis(100)).await;
        cache.get::<T>(key).await?.ok_or(Error::Internal("Cache miss after lock wait".into()))
    }
}
```

### 3.4 Multi-Instance Cache Coherence

In a multi-instance deployment with Redis, cache invalidation is automatically shared because all instances read from and write to the same Redis instance. For additional coordination:

- **Redis Pub/Sub channel** `rc:cache:invalidate`: When an instance invalidates a cache key, it publishes the key name to this channel. Other instances can subscribe and invalidate any in-memory (moka) caches they may hold as a secondary layer.

---

## 4. Product Search Performance at Scale

### 4.1 Search Architecture by Scale

| Scale | Products | Search Strategy | Response Time Target |
|-------|----------|----------------|---------------------|
| Small (< 100) | < 100 | PostgreSQL `ILIKE` with GIN index | < 50ms |
| Medium (< 10K) | < 10K | PostgreSQL full-text search (`tsvector`) | < 100ms |
| Large (< 100K) | < 100K | PostgreSQL full-text search + materialized views | < 100ms |
| Enterprise (100K+) | 100K+ | External search engine (Meilisearch / Elasticsearch) | < 50ms |

### 4.2 PostgreSQL Full-Text Search (Default)

For small-to-large scale, PostgreSQL's built-in full-text search is sufficient:

```sql
-- Search index (created in migration)
ALTER TABLE rc_products ADD COLUMN search_vector tsvector
    GENERATED ALWAYS AS (
        setweight(to_tsvector('english', coalesce(title, '')), 'A') ||
        setweight(to_tsvector('english', coalesce(description, '')), 'B') ||
        setweight(to_tsvector('english', coalesce(sku, '')), 'A') ||
        setweight(to_tsvector('english', coalesce(tags_text, '')), 'C')
    ) STORED;

CREATE INDEX idx_rc_products_search ON rc_products USING gin(search_vector);

-- Search query
SELECT id, title, slug, price, ts_rank(search_vector, query) AS rank
FROM rc_products,
     plainto_tsquery('english', $1) AS query
WHERE search_vector @@ query
  AND status = 'published'
ORDER BY rank DESC
LIMIT $2 OFFSET $3;
```

### 4.3 Faceted Filtering

Product filtering (by category, price range, attributes) is handled by PostgreSQL queries with composite indexes:

```sql
-- Faceted filter query
SELECT p.id, p.title, p.price
FROM rc_products p
JOIN rc_product_categories pc ON p.id = pc.product_id
WHERE p.status = 'published'
  AND pc.category_id = $1               -- Category filter
  AND p.price BETWEEN $2 AND $3         -- Price range filter
  AND p.id IN (                         -- Attribute filter (e.g., color = 'red')
      SELECT product_id FROM rc_product_variant_attributes
      WHERE attribute_key = $4 AND attribute_value = $5
  )
ORDER BY p.created_at DESC
LIMIT $6 OFFSET $7;

-- Supporting composite index
CREATE INDEX idx_rc_products_status_price ON rc_products(status, price)
    WHERE status = 'published';
```

### 4.4 Search Result Caching

Search results are cached with a hash of the query parameters as the key:

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn search_cache_key(params: &SearchParams) -> String {
    let mut hasher = DefaultHasher::new();
    params.hash(&mut hasher);
    format!("rc:search:{:x}", hasher.finish())
}
```

Cache TTL for search results: 2 minutes (balances freshness vs performance).

### 4.5 Materialized Views for Aggregates (Large Scale)

At large scale, use materialized views for expensive aggregate queries like category product counts and price range facets:

```sql
CREATE MATERIALIZED VIEW rc_category_product_counts AS
SELECT
    c.id AS category_id,
    c.name AS category_name,
    COUNT(pc.product_id) AS product_count,
    MIN(p.price) AS min_price,
    MAX(p.price) AS max_price
FROM rc_categories c
LEFT JOIN rc_product_categories pc ON c.id = pc.category_id
LEFT JOIN rc_products p ON pc.product_id = p.id AND p.status = 'published'
GROUP BY c.id, c.name;

CREATE UNIQUE INDEX idx_rc_cat_counts_id ON rc_category_product_counts(category_id);
```

Refresh strategy: Background job refreshes every 5 minutes via `rustpress-jobs`:

```rust
// Registered as a cron job in plugin.toml
// [[cron]]
// name = "refresh_category_counts"
// schedule = "*/5 * * * *"
// handler = "refresh_materialized_views"

pub async fn refresh_materialized_views(pool: &PgPool) -> Result<()> {
    sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY rc_category_product_counts")
        .execute(pool)
        .await?;
    Ok(())
}
```

---

## 5. Order Processing Throughput Optimization

### 5.1 Checkout Critical Path

The checkout flow is the most latency-sensitive operation. The critical path:

```
1. Validate cart contents (stock check)         ~5ms
2. Calculate totals (subtotal, tax, shipping)    ~10ms
3. Reserve inventory (UPDATE with row locks)     ~15ms
4. Create order + order items (INSERT)           ~10ms
5. Create Stripe PaymentIntent (API call)        ~300-800ms
6. Return payment confirmation to client         ~5ms
                                          Total: ~350-850ms
```

### 5.2 Optimization Strategies

**Strategy 1: Minimize database round-trips**

Batch cart validation and totals calculation into a single query:

```sql
-- Single query: validate stock, get prices, calculate totals
SELECT
    ci.id AS cart_item_id,
    pv.id AS variant_id,
    pv.price,
    pv.stock_quantity,
    ci.quantity,
    (pv.price * ci.quantity) AS line_total,
    CASE WHEN pv.stock_quantity >= ci.quantity THEN true ELSE false END AS in_stock
FROM rc_cart_items ci
JOIN rc_product_variants pv ON ci.variant_id = pv.id
WHERE ci.cart_id = $1;
```

**Strategy 2: Inventory reservation with advisory locks**

Use PostgreSQL advisory locks instead of `SELECT ... FOR UPDATE` to reduce contention:

```rust
pub async fn reserve_inventory(
    pool: &PgPool,
    items: &[(Uuid, i32)], // (variant_id, quantity)
) -> Result<()> {
    let mut tx = pool.begin().await?;

    for (variant_id, quantity) in items {
        // Advisory lock on variant ID to prevent double-reservation
        sqlx::query("SELECT pg_advisory_xact_lock($1)")
            .bind(variant_id.as_u128() as i64)
            .execute(&mut *tx)
            .await?;

        let updated = sqlx::query!(
            r#"
            UPDATE rc_product_variants
            SET stock_quantity = stock_quantity - $1,
                reserved_quantity = reserved_quantity + $1,
                updated_at = NOW()
            WHERE id = $2 AND stock_quantity >= $1
            "#,
            quantity,
            variant_id
        )
        .execute(&mut *tx)
        .await?;

        if updated.rows_affected() == 0 {
            return Err(Error::Validation("Insufficient stock".into()));
        }
    }

    tx.commit().await?;
    Ok(())
}
```

**Strategy 3: Async post-checkout processing**

After the payment succeeds, move non-critical operations to background jobs:

```
Synchronous (checkout response):
  ✓ Payment confirmation
  ✓ Order status update

Asynchronous (background jobs):
  → Send confirmation email
  → Update analytics/metrics
  → Fire rustcommerce.order_created hook
  → Generate invoice PDF
  → Notify admin via WebSocket
```

```rust
// After successful payment
job_queue.enqueue(Job::new("send_order_confirmation_email", order.id)).await?;
job_queue.enqueue(Job::new("notify_admin_new_order", order.id)).await?;
job_queue.enqueue(Job::new("update_store_metrics", order.id)).await?;
```

**Strategy 4: Stripe PaymentIntent optimization**

The Stripe API call is the biggest latency contributor. Optimizations:
- Reuse HTTP connections to Stripe (connection pooling in the `stripe-rust` client).
- Create the PaymentIntent with `confirm: true` to combine creation and confirmation in a single API call.
- Use Stripe's idempotency keys to safely retry failed requests without double-charging.

### 5.3 Inventory Reservation Lifecycle

```
Cart Add           → No reservation (stock check only)
Checkout Start     → 10-minute hold (reserved_quantity += qty)
Payment Success    → Permanent deduction (stock_quantity stays reduced, reserved_quantity -= qty)
Payment Failure    → Release hold (stock_quantity += qty, reserved_quantity -= qty)
Reservation Expiry → Background job releases after 10 minutes
```

Background job for expired reservations:

```sql
-- Runs every minute via cron job
UPDATE rc_product_variants pv
SET stock_quantity = stock_quantity + r.quantity,
    reserved_quantity = reserved_quantity - r.quantity
FROM rc_inventory_reservations r
WHERE r.variant_id = pv.id
  AND r.status = 'reserved'
  AND r.expires_at < NOW();

-- Also update the reservation status
UPDATE rc_inventory_reservations
SET status = 'expired'
WHERE status = 'reserved'
  AND expires_at < NOW();
```

### 5.4 Order Processing Throughput Targets

| Scale | Target Throughput | Limiting Factor | Mitigation |
|-------|------------------|----------------|------------|
| Small | 1 order/sec | Stripe API latency | Acceptable |
| Medium | 10 orders/sec | Stripe API concurrency | Parallel Stripe calls, connection pooling |
| Large | 50 orders/sec | Database write throughput | Write-optimized PostgreSQL config, partitioned order tables |
| Flash sale | 100+ orders/sec | All of the above | Pre-authorized payments, queue-based checkout |

### 5.5 Database Performance Tuning (Large Scale)

PostgreSQL configuration recommendations for high-throughput order processing:

```ini
# postgresql.conf tuning for e-commerce workload
shared_buffers = 4GB                  # 25% of available RAM
effective_cache_size = 12GB           # 75% of available RAM
work_mem = 64MB                       # For sort/hash operations
maintenance_work_mem = 512MB          # For VACUUM, CREATE INDEX
wal_buffers = 64MB                    # WAL write buffer
checkpoint_completion_target = 0.9    # Spread checkpoint I/O
random_page_cost = 1.1                # SSD-optimized
effective_io_concurrency = 200        # SSD-optimized
max_parallel_workers_per_gather = 2   # Parallel query execution
```

### 5.6 Table Partitioning (Large Scale)

For stores with millions of orders, partition the orders table by date:

```sql
CREATE TABLE rc_orders (
    id UUID NOT NULL,
    order_number SERIAL,
    customer_id UUID,
    status VARCHAR(20),
    total INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- ... other columns
) PARTITION BY RANGE (created_at);

-- Monthly partitions
CREATE TABLE rc_orders_2026_01 PARTITION OF rc_orders
    FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');
CREATE TABLE rc_orders_2026_02 PARTITION OF rc_orders
    FOR VALUES FROM ('2026-02-01') TO ('2026-03-01');
-- etc. (automated via background job)
```

Benefits:
- Queries filtering by date range only scan relevant partitions.
- Old partitions can be archived or moved to cheaper storage.
- VACUUM and index maintenance is per-partition, reducing lock contention.

---

## 6. Scaling Roadmap

| Phase | Scale | Actions |
|-------|-------|---------|
| **Phase 1** (Launch) | Small | Single instance, moka cache, local storage, basic PostgreSQL config |
| **Phase 2** (Growth) | Medium | Add Redis, S3 storage, Cloudflare CDN, tune PostgreSQL, add monitoring |
| **Phase 3** (Scale) | Large | Multi-instance + load balancer, RDS Multi-AZ, ElastiCache cluster, PgBouncer, materialized views |
| **Phase 4** (Enterprise) | 100K+ | Table partitioning, read replicas, external search engine (Meilisearch), queue-based checkout, Stripe enterprise pricing |

Each phase is an incremental upgrade. The application code remains the same; only infrastructure configuration and deployment topology change. This is possible because RustCommerce accesses all infrastructure through RustPress's abstraction layers (`CacheProvider`, `StorageBackend`, `PgPool`).
