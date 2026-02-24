# Networking and API Design — RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Author**: Infrastructure Agent
**Status**: Draft

---

## 1. API Gateway Considerations

### 1.1 Request Flow

All RustCommerce API traffic flows through the RustPress Axum middleware stack. There is no separate API gateway service. The middleware stack provides:

```
Client Request
    │
    ▼
┌──────────────────────────────────┐
│  Reverse Proxy (nginx/Caddy)     │  TLS termination, basic rate limiting
└──────────────┬───────────────────┘
               │
┌──────────────▼───────────────────┐
│  RustPress Middleware Stack       │
│                                  │
│  1. Compression (gzip/brotli)    │
│  2. Request ID (X-Request-Id)    │
│  3. Tracing (span creation)      │
│  4. Bot Detection                │
│  5. Security Headers             │
│  6. CORS                         │
│  7. Rate Limiting                │
│  8. Tenant Resolution            │
│  9. Request Validation           │
│ 10. Authentication (JWT)         │
└──────────────┬───────────────────┘
               │
┌──────────────▼───────────────────┐
│  RustCommerce Route Handlers     │
│  /api/v1/rustcommerce/*          │
└──────────────────────────────────┘
```

### 1.2 Rate Limiting

RustPress provides built-in rate limiting at the middleware level. RustCommerce configures endpoint-specific limits in its `plugin.toml`:

| Endpoint Category | Rate Limit | Window | Rationale |
|-------------------|-----------|--------|-----------|
| **Product browsing** (GET /products, /categories) | 120 req/min | Per IP | High-traffic public endpoints; generous limit |
| **Cart operations** (POST/PUT/DELETE /cart) | 60 req/min | Per session | Normal shopping behavior |
| **Checkout** (POST /checkout) | 10 req/min | Per session | Prevents checkout abuse |
| **Payment** (POST /payments) | 5 req/min | Per session | Strict; payment processing is expensive |
| **Webhooks** (POST /webhooks/stripe) | 100 req/min | Per IP | Stripe sends bursts; must accommodate |
| **Admin CRUD** (all /admin/*) | 60 req/min | Per user | Standard admin usage |
| **Search** (GET /products/search) | 30 req/min | Per IP | Search queries are heavier on DB |

Plugin-level rate limit declaration in `plugin.toml`:

```toml
[[api.endpoints]]
method = "POST"
path = "/checkout"
handler = "create_checkout"
permission = "checkout"
rate_limit = 10

[[api.endpoints]]
method = "GET"
path = "/products"
handler = "list_products"
permission = "read"
rate_limit = 120
```

### 1.3 Request Throttling

For computationally expensive operations, RustCommerce implements application-level throttling using Tokio semaphores:

```rust
// Limit concurrent checkout processing to prevent database connection exhaustion
static CHECKOUT_SEMAPHORE: Lazy<Semaphore> = Lazy::new(|| Semaphore::new(20));

pub async fn process_checkout(...) -> Result<Json<Order>, Error> {
    let _permit = CHECKOUT_SEMAPHORE.acquire().await
        .map_err(|_| Error::Internal("Checkout capacity exceeded".into()))?;
    // ... process checkout
}
```

| Operation | Max Concurrent | Rationale |
|-----------|---------------|-----------|
| Checkout processing | 20 | Involves payment API calls, stock reservation, order creation |
| Image upload | 10 | CPU-intensive (resizing, format conversion) |
| CSV import | 2 | Bulk DB writes; must not starve other queries |
| Report generation | 3 | Heavy aggregate queries |

---

## 2. Stripe Webhook Endpoint Security

### 2.1 Webhook Endpoint

```
POST /api/v1/rustcommerce/webhooks/stripe
```

This endpoint receives asynchronous payment event notifications from Stripe (e.g., `payment_intent.succeeded`, `charge.refunded`).

### 2.2 Signature Verification

Every incoming webhook request is verified using Stripe's signature mechanism:

```rust
use stripe::Webhook;

pub async fn handle_stripe_webhook(
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, Error> {
    let signature = headers
        .get("Stripe-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(Error::Authentication("Missing Stripe-Signature header".into()))?;

    let webhook_secret = config.stripe_webhook_secret();

    // Verify signature (includes timestamp check to prevent replay attacks)
    let event = Webhook::construct_event(
        &String::from_utf8_lossy(&body),
        signature,
        &webhook_secret,
    ).map_err(|_| Error::Authentication("Invalid webhook signature".into()))?;

    // Process the verified event
    match event.type_ {
        EventType::PaymentIntentSucceeded => { /* ... */ },
        EventType::ChargeRefunded => { /* ... */ },
        _ => { /* log and acknowledge */ },
    }

    Ok(StatusCode::OK)
}
```

### 2.3 IP Whitelisting (Optional)

For defense-in-depth, the webhook endpoint can optionally restrict to Stripe's published IP ranges. This is configured at the reverse proxy level (nginx/Caddy) rather than in application code:

```nginx
# nginx configuration for Stripe webhook IP restriction
location /api/v1/rustcommerce/webhooks/stripe {
    # Stripe webhook IPs (https://docs.stripe.com/ips)
    allow 3.18.12.63;
    allow 3.130.192.0/24;
    allow 13.235.14.0/24;
    allow 13.235.122.0/24;
    allow 18.211.135.69;
    allow 35.154.171.200;
    allow 52.15.183.38;
    allow 54.88.130.119;
    allow 54.88.130.237;
    allow 54.187.174.169;
    allow 54.187.205.235;
    allow 54.187.216.72;
    deny all;

    proxy_pass http://rustpress_backend;
}
```

**Recommendation**: Use signature verification as the primary security mechanism. IP whitelisting is supplementary and should not be relied upon exclusively, as Stripe's IP ranges can change.

### 2.4 Webhook Idempotency

Stripe may deliver the same event multiple times. RustCommerce handles this by:

1. Storing the Stripe event ID (`evt_...`) in `rc_payments.stripe_event_id`.
2. Checking for duplicate event IDs before processing.
3. Using database transactions to ensure atomicity.

```rust
// Idempotency check
if payment_repo.event_exists(&event.id).await? {
    tracing::info!("Duplicate webhook event {}, skipping", event.id);
    return Ok(StatusCode::OK);
}
```

### 2.5 Webhook Event Types Handled

| Stripe Event | RustCommerce Action |
|--------------|---------------------|
| `payment_intent.succeeded` | Mark order as paid, reduce inventory, send confirmation email |
| `payment_intent.payment_failed` | Mark order as payment failed, release inventory hold |
| `charge.refunded` | Update order status to refunded, restore inventory |
| `charge.dispute.created` | Flag order for admin review, fire `rustcommerce.dispute_created` hook |
| `customer.subscription.updated` | (P2) Handle subscription changes |

---

## 3. WebSocket Requirements

### 3.1 Real-Time Admin Notifications

RustCommerce uses WebSocket connections for real-time notifications in the admin dashboard:

```
wss://store.example.com/api/v1/rustcommerce/ws/admin
```

### 3.2 WebSocket Events

| Event | Payload | Purpose |
|-------|---------|---------|
| `order.new` | `{ order_id, total, customer_name }` | Alert admin of new order |
| `order.status_changed` | `{ order_id, old_status, new_status }` | Order status updates |
| `inventory.low_stock` | `{ product_id, variant_id, stock_level }` | Low stock alert |
| `payment.received` | `{ order_id, amount, method }` | Payment confirmation |
| `payment.failed` | `{ order_id, error }` | Payment failure alert |
| `review.new` | `{ product_id, rating, needs_moderation }` | New review submitted |

### 3.3 WebSocket Authentication

WebSocket connections require authentication via JWT token passed as a query parameter during the handshake:

```
wss://store.example.com/api/v1/rustcommerce/ws/admin?token={jwt_token}
```

The server validates the JWT and checks for the `manage_orders` permission before upgrading the connection. Connections from users without admin permissions are rejected with a 403 status.

### 3.4 WebSocket Architecture

```
┌──────────────────────────────────────┐
│  Admin Browser (React)               │
│  WebSocket Client                    │
└──────────────┬───────────────────────┘
               │ wss://
┌──────────────▼───────────────────────┐
│  RustPress Axum Server               │
│  WebSocket Upgrade Handler           │
│  ┌────────────────────────────────┐  │
│  │ rustcommerce::ws::AdminHub     │  │
│  │                                │  │
│  │ - Connection registry          │  │
│  │ - Broadcast channel (tokio)    │  │
│  │ - Per-connection send task     │  │
│  └────────────────────────────────┘  │
└──────────────────────────────────────┘
```

RustCommerce publishes events to a `tokio::sync::broadcast` channel. Each connected admin WebSocket has a receiver that forwards messages to the client. When an order is created or a payment is processed, the service layer publishes to the broadcast channel:

```rust
// In order_service.rs after creating an order
admin_broadcast.send(AdminEvent::NewOrder {
    order_id: order.id,
    total: order.total,
    customer_name: order.customer_name.clone(),
})?;
```

### 3.5 Multi-Instance WebSocket Considerations

In a horizontally-scaled deployment with multiple RustPress instances, the in-process broadcast channel only reaches clients connected to the same instance. To deliver notifications across all instances:

- **With Redis**: Use Redis Pub/Sub as a cross-instance message bus. Each RustPress instance subscribes to a `rc:admin:events` channel and forwards received messages to its local WebSocket clients.
- **Without Redis**: WebSocket notifications are instance-local only. This is acceptable for single-server deployments.

---

## 4. CORS Configuration

### 4.1 RustPress Core CORS

CORS is handled by the RustPress middleware stack. The default configuration allows requests from the admin UI origin. RustCommerce inherits this configuration.

### 4.2 Storefront CORS Requirements

Storefronts (themes, headless frontends, mobile apps) may be served from different origins than the RustPress server. The CORS configuration must be extended to support these:

```toml
# In rustpress.toml or environment variables
[cors]
allowed_origins = [
    "https://store.example.com",       # Main storefront
    "https://admin.example.com",       # Admin UI
    "https://mobile.example.com",      # Mobile web app
]
# Or use a wildcard for development:
# allowed_origins = ["*"]
```

### 4.3 CORS Headers by Endpoint Category

| Endpoint Category | Allowed Origins | Allowed Methods | Credentials |
|-------------------|----------------|-----------------|-------------|
| Public storefront API (`/api/v1/rustcommerce/products`, `/categories`) | Configured origins | GET, OPTIONS | No |
| Cart/checkout API (`/api/v1/rustcommerce/cart`, `/checkout`) | Configured origins | GET, POST, PUT, DELETE, OPTIONS | Yes (cookies/JWT) |
| Admin API (`/api/v1/rustcommerce/admin/*`) | Admin UI origin only | GET, POST, PUT, DELETE, PATCH, OPTIONS | Yes |
| Webhooks (`/api/v1/rustcommerce/webhooks/*`) | N/A (server-to-server) | POST | No |

### 4.4 Preflight Caching

CORS preflight (OPTIONS) responses include `Access-Control-Max-Age: 86400` (24 hours) to minimize redundant preflight requests from browsers.

### 4.5 Security Headers

In addition to CORS, RustPress's security middleware applies these headers to all RustCommerce responses:

```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 0
Referrer-Policy: strict-origin-when-cross-origin
Content-Security-Policy: default-src 'self'; img-src 'self' https://cdn.example.com; ...
Strict-Transport-Security: max-age=31536000; includeSubDomains
```

---

## 5. API Route Map

### 5.1 Complete Route Namespace

All routes are under `/api/v1/rustcommerce/`:

```
# Public Storefront API (authentication optional)
GET    /api/v1/rustcommerce/products                  # List products (paginated, filtered)
GET    /api/v1/rustcommerce/products/:id               # Get product detail
GET    /api/v1/rustcommerce/products/:id/reviews        # Get product reviews
GET    /api/v1/rustcommerce/products/search             # Search products
GET    /api/v1/rustcommerce/categories                  # List categories (tree)
GET    /api/v1/rustcommerce/categories/:id              # Get category with products

# Cart API (session-based or authenticated)
GET    /api/v1/rustcommerce/cart                        # Get current cart
POST   /api/v1/rustcommerce/cart/items                  # Add item to cart
PUT    /api/v1/rustcommerce/cart/items/:id              # Update cart item quantity
DELETE /api/v1/rustcommerce/cart/items/:id              # Remove item from cart
DELETE /api/v1/rustcommerce/cart                        # Clear cart

# Checkout API (session-based or authenticated)
POST   /api/v1/rustcommerce/checkout                    # Create checkout session
PUT    /api/v1/rustcommerce/checkout/shipping-address    # Set shipping address
PUT    /api/v1/rustcommerce/checkout/shipping-method     # Select shipping method
POST   /api/v1/rustcommerce/checkout/complete            # Complete checkout (create order + payment)
GET    /api/v1/rustcommerce/checkout/shipping-rates      # Get available shipping rates

# Customer API (authenticated)
GET    /api/v1/rustcommerce/customer/orders              # Customer order history
GET    /api/v1/rustcommerce/customer/orders/:id          # Customer order detail
GET    /api/v1/rustcommerce/customer/addresses            # List saved addresses
POST   /api/v1/rustcommerce/customer/addresses            # Add address
POST   /api/v1/rustcommerce/customer/reviews              # Submit product review

# Webhook API (Stripe server-to-server)
POST   /api/v1/rustcommerce/webhooks/stripe              # Stripe event webhook

# Admin API (authenticated + admin permissions)
GET    /api/v1/rustcommerce/admin/dashboard              # Dashboard metrics
GET    /api/v1/rustcommerce/admin/products               # List all products (including drafts)
POST   /api/v1/rustcommerce/admin/products               # Create product
PUT    /api/v1/rustcommerce/admin/products/:id           # Update product
DELETE /api/v1/rustcommerce/admin/products/:id           # Delete product
GET    /api/v1/rustcommerce/admin/orders                 # List all orders
GET    /api/v1/rustcommerce/admin/orders/:id             # Order detail
PUT    /api/v1/rustcommerce/admin/orders/:id/status       # Update order status
POST   /api/v1/rustcommerce/admin/orders/:id/refund       # Issue refund
GET    /api/v1/rustcommerce/admin/customers               # List customers
GET    /api/v1/rustcommerce/admin/customers/:id           # Customer detail
GET    /api/v1/rustcommerce/admin/inventory               # Inventory overview
PUT    /api/v1/rustcommerce/admin/inventory/:variant_id   # Update stock
GET    /api/v1/rustcommerce/admin/settings                # Get store settings
PUT    /api/v1/rustcommerce/admin/settings                # Update store settings
POST   /api/v1/rustcommerce/admin/products/import          # CSV import
GET    /api/v1/rustcommerce/admin/products/export          # CSV export
GET    /api/v1/rustcommerce/admin/coupons                  # List coupons
POST   /api/v1/rustcommerce/admin/coupons                  # Create coupon
PUT    /api/v1/rustcommerce/admin/coupons/:id              # Update coupon
DELETE /api/v1/rustcommerce/admin/coupons/:id              # Delete coupon
GET    /api/v1/rustcommerce/admin/reviews                  # Review moderation queue
PUT    /api/v1/rustcommerce/admin/reviews/:id              # Approve/reject review

# WebSocket (admin only)
GET    /api/v1/rustcommerce/ws/admin                     # Admin real-time notifications
```

### 5.2 Response Format

All JSON responses follow a consistent envelope:

```json
{
  "data": { ... },
  "meta": {
    "page": 1,
    "per_page": 20,
    "total": 150,
    "total_pages": 8
  }
}
```

Error responses:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid product data",
    "details": [
      { "field": "price", "message": "Price must be greater than zero" }
    ]
  }
}
```

---

## 6. Network Security Summary

| Layer | Mechanism | Responsibility |
|-------|-----------|---------------|
| Transport | TLS 1.2+ (terminated at reverse proxy) | Encryption in transit |
| Reverse proxy | IP-based rate limiting, geo-blocking | First line of defense |
| Application middleware | Token-based rate limiting, CORS, security headers | Application-level controls |
| Route-level | Permission checks, input validation, CSRF | Endpoint-specific security |
| Webhook | Stripe signature verification, idempotency | Webhook integrity |
| WebSocket | JWT authentication on upgrade, permission check | Real-time channel security |
