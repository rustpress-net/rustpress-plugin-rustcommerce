# Monitoring and Observability Plan -- RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Owner**: DevOps Lead
**Status**: Approved

---

## 1. Overview

This document defines the observability strategy for the RustCommerce plugin. It covers three pillars: **metrics** (Prometheus), **logs** (structured logging with `tracing`), and **alerts** (rule-based alerting for critical e-commerce events). The goal is to provide full visibility into store operations, payment health, and system performance.

### Observability Stack

| Pillar | Tool | Integration Point |
|--------|------|-------------------|
| Metrics | Prometheus | RustPress `/metrics` endpoint |
| Visualization | Grafana | Prometheus data source |
| Logging | `tracing` + `tracing-subscriber` | stdout (JSON format) / log aggregator |
| Alerting | Prometheus Alertmanager | Webhook / Slack / PagerDuty |
| Tracing (distributed) | OpenTelemetry (future) | Jaeger / Tempo |

---

## 2. Metrics

All metrics are exposed via the RustPress Prometheus endpoint at `/metrics`. RustCommerce registers its metrics under the `rustcommerce_` namespace.

### 2.1 Business Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `rustcommerce_orders_total` | Counter | `status` (pending, processing, shipped, delivered, cancelled, refunded) | Total orders by status |
| `rustcommerce_orders_per_minute` | Gauge | -- | Rolling 1-minute order rate |
| `rustcommerce_revenue_total_cents` | Counter | `currency` | Cumulative revenue in smallest currency unit |
| `rustcommerce_cart_items_total` | Gauge | -- | Current total items across all active carts |
| `rustcommerce_cart_conversion_rate` | Gauge | -- | Ratio of completed checkouts to carts with items (rolling 1h window) |
| `rustcommerce_cart_abandonment_rate` | Gauge | -- | Ratio of abandoned carts to total carts (rolling 24h window) |
| `rustcommerce_average_order_value_cents` | Gauge | `currency` | Average order value over rolling 1h window |
| `rustcommerce_products_active` | Gauge | -- | Count of published products |
| `rustcommerce_inventory_low_stock` | Gauge | -- | Count of products/variants below low-stock threshold |
| `rustcommerce_inventory_out_of_stock` | Gauge | -- | Count of products/variants with zero stock |
| `rustcommerce_customers_total` | Counter | -- | Total registered customers |

### 2.2 Payment Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `rustcommerce_payment_attempts_total` | Counter | `gateway` (stripe), `status` (success, failed, pending) | Payment processing attempts |
| `rustcommerce_payment_success_rate` | Gauge | `gateway` | Success rate over rolling 1h window |
| `rustcommerce_payment_amount_total_cents` | Counter | `gateway`, `currency` | Total payment amount processed |
| `rustcommerce_payment_duration_seconds` | Histogram | `gateway` | Time from payment initiation to gateway response |
| `rustcommerce_refunds_total` | Counter | `gateway` | Total refunds processed |
| `rustcommerce_refund_amount_total_cents` | Counter | `gateway`, `currency` | Total refund amount |
| `rustcommerce_webhook_events_total` | Counter | `event_type` (payment_intent.succeeded, charge.refunded, etc.) | Stripe webhook events received |
| `rustcommerce_webhook_processing_errors_total` | Counter | `event_type` | Webhook processing failures |

### 2.3 API Performance Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `rustcommerce_http_requests_total` | Counter | `method`, `endpoint`, `status_code` | Total HTTP requests |
| `rustcommerce_http_request_duration_seconds` | Histogram | `method`, `endpoint` | Request latency distribution |
| `rustcommerce_http_requests_in_flight` | Gauge | -- | Currently processing requests |
| `rustcommerce_db_query_duration_seconds` | Histogram | `query_name` | Database query latency |
| `rustcommerce_cache_hits_total` | Counter | `cache_type` (product, cart, session) | Cache hit count |
| `rustcommerce_cache_misses_total` | Counter | `cache_type` | Cache miss count |
| `rustcommerce_cache_hit_rate` | Gauge | `cache_type` | Cache hit ratio |

### 2.4 Histogram Buckets

Latency histograms use the following bucket boundaries (in seconds):

```
API endpoints:     [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
Payment gateway:   [0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0]
Database queries:  [0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]
```

### 2.5 Metric Registration (Rust)

```rust
use prometheus::{register_counter_vec, register_histogram_vec, register_gauge};

lazy_static! {
    pub static ref ORDERS_TOTAL: CounterVec = register_counter_vec!(
        "rustcommerce_orders_total",
        "Total orders by status",
        &["status"]
    ).unwrap();

    pub static ref PAYMENT_DURATION: HistogramVec = register_histogram_vec!(
        "rustcommerce_payment_duration_seconds",
        "Payment processing duration",
        &["gateway"],
        vec![0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0]
    ).unwrap();

    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "rustcommerce_http_request_duration_seconds",
        "HTTP request duration",
        &["method", "endpoint"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    ).unwrap();

    pub static ref INVENTORY_LOW_STOCK: Gauge = register_gauge!(
        "rustcommerce_inventory_low_stock",
        "Products below low-stock threshold"
    ).unwrap();
}
```

---

## 3. Structured Logging

### 3.1 Logging Framework

RustCommerce uses the `tracing` crate for structured logging, following the RustPress core logging conventions. All log entries are emitted as structured JSON when running in production, and human-readable format in development.

### 3.2 Log Levels

| Level | Usage |
|-------|-------|
| `ERROR` | Failures requiring immediate attention (payment failures, data corruption, unhandled errors) |
| `WARN` | Degraded conditions (low stock, high latency, retry attempts) |
| `INFO` | Normal business events (order created, payment completed, product updated) |
| `DEBUG` | Detailed operational information (query timing, cache behavior, request routing) |
| `TRACE` | Fine-grained diagnostic data (full request/response bodies, SQL queries) |

### 3.3 E-Commerce Event Log Patterns

Each log event includes a structured `event_type` field for filtering and aggregation.

#### Order Events

```rust
// Order created
tracing::info!(
    event_type = "order.created",
    order_id = %order.id,
    customer_id = %order.customer_id,
    total_cents = order.total_cents,
    item_count = order.items.len(),
    currency = %order.currency,
    "Order created"
);

// Order status changed
tracing::info!(
    event_type = "order.status_changed",
    order_id = %order.id,
    previous_status = %previous,
    new_status = %new,
    changed_by = %actor,
    "Order status updated"
);

// Order cancelled
tracing::warn!(
    event_type = "order.cancelled",
    order_id = %order.id,
    reason = %reason,
    refund_initiated = refund,
    "Order cancelled"
);
```

#### Payment Events

```rust
// Payment initiated
tracing::info!(
    event_type = "payment.initiated",
    order_id = %order_id,
    amount_cents = amount,
    currency = %currency,
    gateway = "stripe",
    "Payment processing started"
);

// Payment succeeded
tracing::info!(
    event_type = "payment.succeeded",
    order_id = %order_id,
    payment_intent_id = %intent_id,
    amount_cents = amount,
    duration_ms = elapsed.as_millis(),
    "Payment completed successfully"
);

// Payment failed
tracing::error!(
    event_type = "payment.failed",
    order_id = %order_id,
    error_code = %stripe_error.code,
    error_message = %stripe_error.message,
    decline_code = ?stripe_error.decline_code,
    amount_cents = amount,
    "Payment failed"
);

// Webhook received
tracing::info!(
    event_type = "webhook.received",
    event_id = %event.id,
    event_type_stripe = %event.event_type,
    "Stripe webhook event received"
);

// Webhook verification failed
tracing::error!(
    event_type = "webhook.verification_failed",
    remote_addr = %addr,
    "Stripe webhook signature verification failed — possible spoofing attempt"
);
```

#### Inventory Events

```rust
// Low stock warning
tracing::warn!(
    event_type = "inventory.low_stock",
    product_id = %product_id,
    variant_id = ?variant_id,
    sku = %sku,
    current_stock = quantity,
    threshold = threshold,
    "Product stock below threshold"
);

// Out of stock
tracing::error!(
    event_type = "inventory.out_of_stock",
    product_id = %product_id,
    variant_id = ?variant_id,
    sku = %sku,
    "Product is out of stock"
);

// Stock reserved for checkout
tracing::info!(
    event_type = "inventory.reserved",
    product_id = %product_id,
    quantity = qty,
    checkout_session_id = %session_id,
    expires_at = %expiry,
    "Stock reserved for checkout"
);
```

#### Cart Events

```rust
// Cart abandoned
tracing::info!(
    event_type = "cart.abandoned",
    cart_id = %cart_id,
    item_count = items,
    total_cents = total,
    age_hours = age.as_secs() / 3600,
    "Cart marked as abandoned"
);

// Checkout started
tracing::info!(
    event_type = "checkout.started",
    cart_id = %cart_id,
    customer_id = ?customer_id,
    item_count = items,
    total_cents = total,
    "Checkout flow initiated"
);

// Checkout completed
tracing::info!(
    event_type = "checkout.completed",
    cart_id = %cart_id,
    order_id = %order_id,
    duration_seconds = elapsed.as_secs(),
    "Checkout completed successfully"
);
```

### 3.4 Log Format

**Development** (human-readable):
```
2026-02-24T10:30:15.123Z  INFO rustcommerce::services::order: Order created
    event_type=order.created order_id=550e8400-e29b-41d4-a716-446655440000
    customer_id=6ba7b810-9dad-11d1-80b4-00c04fd430c8 total_cents=5999
    item_count=3 currency=USD
```

**Production** (JSON):
```json
{
  "timestamp": "2026-02-24T10:30:15.123Z",
  "level": "INFO",
  "target": "rustcommerce::services::order",
  "message": "Order created",
  "fields": {
    "event_type": "order.created",
    "order_id": "550e8400-e29b-41d4-a716-446655440000",
    "customer_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "total_cents": 5999,
    "item_count": 3,
    "currency": "USD"
  }
}
```

### 3.5 Log Configuration

```toml
# In the RUST_LOG environment variable:

# Development — verbose plugin logging
RUST_LOG=debug,rustcommerce=trace,sqlx=warn,hyper=warn

# Production — business events only, minimal noise
RUST_LOG=info,rustcommerce=info,sqlx=warn,hyper=warn

# Debugging payments in production (temporary)
RUST_LOG=info,rustcommerce::services::payment=debug,rustcommerce::handlers::webhook=debug
```

---

## 4. Alerting Rules

### 4.1 Critical Alerts (P0 -- Immediate Response)

These alerts fire to PagerDuty / on-call channel and require immediate human response.

#### Payment Failure Spike

```yaml
- alert: RustCommercePaymentFailureSpike
  expr: |
    (
      rate(rustcommerce_payment_attempts_total{status="failed"}[5m])
      /
      rate(rustcommerce_payment_attempts_total[5m])
    ) > 0.2
  for: 3m
  labels:
    severity: critical
    team: commerce
  annotations:
    summary: "Payment failure rate above 20% for 3 minutes"
    description: >
      Payment failure rate is {{ $value | humanizePercentage }}.
      This may indicate a Stripe outage, misconfigured API keys,
      or a fraud attack. Check Stripe dashboard and recent deployments.
    runbook: "https://wiki.internal/runbooks/payment-failure-spike"
```

#### Payment Gateway Unreachable

```yaml
- alert: RustCommercePaymentGatewayDown
  expr: |
    rate(rustcommerce_payment_attempts_total{status="success"}[5m]) == 0
    AND
    rate(rustcommerce_orders_total{status="pending"}[5m]) > 0
  for: 5m
  labels:
    severity: critical
    team: commerce
  annotations:
    summary: "No successful payments in 5 minutes while orders are being created"
    description: >
      Orders are being created but no payments are succeeding.
      The payment gateway may be unreachable.
```

#### Inventory Zero (High-Demand Product)

```yaml
- alert: RustCommerceHighDemandOutOfStock
  expr: |
    rustcommerce_inventory_out_of_stock > 0
    AND
    rate(rustcommerce_http_requests_total{endpoint=~"/api/v1/rustcommerce/products/.*"}[10m]) > 1
  for: 1m
  labels:
    severity: critical
    team: commerce
  annotations:
    summary: "High-demand product is out of stock"
    description: >
      {{ $value }} products are out of stock while receiving active traffic.
      Revenue is being lost. Restock urgently or mark as backorder.
```

### 4.2 Warning Alerts (P1 -- Investigate Within 1 Hour)

#### Low Stock Threshold

```yaml
- alert: RustCommerceLowStockWarning
  expr: rustcommerce_inventory_low_stock > 5
  for: 10m
  labels:
    severity: warning
    team: commerce
  annotations:
    summary: "{{ $value }} products below low-stock threshold"
    description: >
      Multiple products are running low on inventory.
      Review stock levels in the admin dashboard.
```

#### Cart Abandonment Rate High

```yaml
- alert: RustCommerceHighCartAbandonment
  expr: rustcommerce_cart_abandonment_rate > 0.75
  for: 30m
  labels:
    severity: warning
    team: commerce
  annotations:
    summary: "Cart abandonment rate above 75%"
    description: >
      Cart abandonment rate is {{ $value | humanizePercentage }}.
      This may indicate UX issues in the checkout flow, unexpected costs,
      or technical errors.
```

#### API Latency Degradation

```yaml
- alert: RustCommerceAPILatencyHigh
  expr: |
    histogram_quantile(0.95,
      rate(rustcommerce_http_request_duration_seconds_bucket[5m])
    ) > 0.5
  for: 10m
  labels:
    severity: warning
    team: commerce
  annotations:
    summary: "RustCommerce API p95 latency above 500ms"
    description: >
      The 95th percentile API latency is {{ $value }}s.
      Performance target is < 100ms for cached endpoints.
      Check database query performance and cache hit rates.
```

#### Webhook Processing Errors

```yaml
- alert: RustCommerceWebhookErrors
  expr: rate(rustcommerce_webhook_processing_errors_total[10m]) > 0.1
  for: 5m
  labels:
    severity: warning
    team: commerce
  annotations:
    summary: "Stripe webhook processing errors detected"
    description: >
      Webhook errors at {{ $value }}/s. This may cause missed payment
      confirmations or refund processing delays.
```

### 4.3 Informational Alerts (P2 -- Review During Business Hours)

#### Order Volume Anomaly

```yaml
- alert: RustCommerceOrderVolumeAnomaly
  expr: |
    abs(
      rate(rustcommerce_orders_total[1h])
      -
      avg_over_time(rate(rustcommerce_orders_total[1h])[7d:1h])
    ) > 2 * stddev_over_time(rate(rustcommerce_orders_total[1h])[7d:1h])
  for: 30m
  labels:
    severity: info
    team: commerce
  annotations:
    summary: "Order volume is significantly different from the 7-day average"
    description: >
      Order rate has deviated more than 2 standard deviations from the
      weekly average. Could be a marketing campaign, seasonal effect,
      or an issue worth investigating.
```

#### Cache Hit Rate Drop

```yaml
- alert: RustCommerceCacheHitRateLow
  expr: rustcommerce_cache_hit_rate{cache_type="product"} < 0.8
  for: 15m
  labels:
    severity: info
    team: commerce
  annotations:
    summary: "Product cache hit rate below 80%"
    description: >
      Product cache hit rate is {{ $value | humanizePercentage }}.
      This may indicate cache invalidation issues, increased unique
      product traffic, or cache capacity limits.
```

---

## 5. Grafana Dashboard Design

### 5.1 Store Overview Dashboard

| Panel | Visualization | Metric Source |
|-------|--------------|---------------|
| Revenue Today | Stat (big number) | `rustcommerce_revenue_total_cents` |
| Orders Today | Stat (big number) | `rustcommerce_orders_total` |
| Average Order Value | Stat (big number) | `rustcommerce_average_order_value_cents` |
| Orders per Minute | Time series | `rate(rustcommerce_orders_total[1m])` |
| Revenue Over Time | Time series | `rate(rustcommerce_revenue_total_cents[1h])` |
| Order Status Distribution | Pie chart | `rustcommerce_orders_total` by status label |
| Payment Success Rate | Gauge | `rustcommerce_payment_success_rate` |
| Cart Conversion Funnel | Bar gauge | Active carts -> Checkouts started -> Orders completed |

### 5.2 Payment Health Dashboard

| Panel | Visualization | Metric Source |
|-------|--------------|---------------|
| Payment Success Rate | Gauge (green/yellow/red) | `rustcommerce_payment_success_rate` |
| Payment Latency p50/p95/p99 | Time series | `rustcommerce_payment_duration_seconds` quantiles |
| Payment Attempts by Status | Stacked time series | `rustcommerce_payment_attempts_total` by status |
| Webhook Events | Time series | `rustcommerce_webhook_events_total` by event_type |
| Webhook Errors | Time series | `rustcommerce_webhook_processing_errors_total` |
| Refunds | Stat + time series | `rustcommerce_refunds_total` |

### 5.3 API Performance Dashboard

| Panel | Visualization | Metric Source |
|-------|--------------|---------------|
| Request Rate | Time series | `rate(rustcommerce_http_requests_total[5m])` |
| Latency p50/p95/p99 | Time series | `rustcommerce_http_request_duration_seconds` quantiles |
| Error Rate (5xx) | Time series | `rate(rustcommerce_http_requests_total{status_code=~"5.."}[5m])` |
| Top Endpoints by Latency | Table | Top 10 slowest endpoints |
| Database Query Latency | Time series | `rustcommerce_db_query_duration_seconds` quantiles |
| Cache Hit Rate | Time series | `rustcommerce_cache_hit_rate` by cache_type |
| Requests in Flight | Time series | `rustcommerce_http_requests_in_flight` |

### 5.4 Inventory Dashboard

| Panel | Visualization | Metric Source |
|-------|--------------|---------------|
| Active Products | Stat | `rustcommerce_products_active` |
| Low Stock Products | Stat (yellow) | `rustcommerce_inventory_low_stock` |
| Out of Stock Products | Stat (red) | `rustcommerce_inventory_out_of_stock` |
| Inventory Alerts Over Time | Time series | low_stock and out_of_stock gauges |

---

## 6. Health Checks

### 6.1 Plugin Health Endpoint

`GET /api/v1/rustcommerce/health`

```json
{
  "status": "healthy",
  "version": "1.0.0",
  "checks": {
    "database": { "status": "up", "latency_ms": 2 },
    "redis": { "status": "up", "latency_ms": 1 },
    "stripe": { "status": "up", "last_webhook": "2026-02-24T10:30:00Z" }
  },
  "uptime_seconds": 86400
}
```

### 6.2 Readiness vs. Liveness

| Probe | Endpoint | What it Checks |
|-------|----------|----------------|
| Liveness | `/api/v1/rustcommerce/health/live` | Process is running, not deadlocked |
| Readiness | `/api/v1/rustcommerce/health/ready` | Database connected, migrations applied, Stripe reachable |

---

## 7. Future Enhancements

- **Distributed tracing**: Integrate OpenTelemetry with Jaeger/Tempo for end-to-end request tracing across RustPress core and plugins.
- **Real User Monitoring (RUM)**: Track checkout flow timing from the browser.
- **SLO dashboards**: Define and track Service Level Objectives (e.g., 99.9% payment success rate, p95 latency < 200ms).
- **Anomaly detection**: Use Prometheus recording rules or an ML-based system for automated anomaly detection on order patterns.
- **Business intelligence export**: Periodic metric snapshots to a data warehouse for long-term trend analysis.
