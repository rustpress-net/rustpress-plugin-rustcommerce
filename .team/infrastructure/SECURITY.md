# Security Architecture — RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Author**: Infrastructure Agent
**Status**: Draft

---

## 1. Authentication

### 1.1 JWT Token Flow for Storefront Customers

RustCommerce extends RustPress's existing JWT authentication system (`rustpress-auth` crate) to support storefront customer sessions:

```
┌─────────────┐                    ┌─────────────────┐                ┌──────────────┐
│  Storefront  │                    │  RustPress Auth  │                │  PostgreSQL   │
│  (Browser)   │                    │  + RustCommerce  │                │              │
└──────┬───────┘                    └────────┬─────────┘                └──────┬───────┘
       │                                      │                                │
       │  POST /api/v1/auth/login             │                                │
       │  { email, password }                 │                                │
       ├─────────────────────────────────────▶│                                │
       │                                      │  SELECT * FROM users           │
       │                                      │  + rc_customers WHERE ...      │
       │                                      ├───────────────────────────────▶│
       │                                      │◀───────────────────────────────┤
       │                                      │                                │
       │                                      │  Verify password (argon2id)    │
       │                                      │  Generate JWT access token     │
       │                                      │  Generate refresh token        │
       │                                      │                                │
       │  200 OK                              │                                │
       │  { access_token, refresh_token,      │                                │
       │    expires_in: 900 }                 │                                │
       │◀─────────────────────────────────────┤                                │
       │                                      │                                │
       │  GET /api/v1/rustcommerce/cart       │                                │
       │  Authorization: Bearer {token}       │                                │
       ├─────────────────────────────────────▶│                                │
       │                                      │  Validate JWT signature        │
       │                                      │  Extract user_id, roles        │
       │                                      │  Check token expiry            │
       │                                      │                                │
       │  200 OK { cart data }                │                                │
       │◀─────────────────────────────────────┤                                │
```

### 1.2 Token Structure

```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "email": "customer@example.com",
  "roles": ["customer"],
  "permissions": ["browse_products", "manage_own_cart", "checkout", "manage_own_orders"],
  "iss": "rustpress",
  "iat": 1740000000,
  "exp": 1740000900,
  "jti": "unique-token-id"
}
```

### 1.3 Token Lifecycle

| Token Type | Lifetime | Storage | Renewal |
|------------|----------|---------|---------|
| Access token | 15 minutes | Client memory (JavaScript variable) | Via refresh token |
| Refresh token | 7 days | HttpOnly, Secure, SameSite=Strict cookie | Re-authentication |
| Session token (guest cart) | 30 days | HttpOnly, Secure cookie | Automatic |

### 1.4 Guest Session Handling

Unauthenticated shoppers receive a guest session token (opaque, not JWT) to maintain their cart:

```
Set-Cookie: rc_session={random_uuid}; HttpOnly; Secure; SameSite=Lax; Path=/api/v1/rustcommerce/; Max-Age=2592000
```

When a guest completes checkout and creates an account, the guest cart is merged into the new customer's account.

### 1.5 Customer Registration

Customers can register via:
- Explicit registration (`POST /api/v1/auth/register` with `role: "customer"`)
- Implicit registration during checkout (if "create account" is selected)

Passwords are hashed using **argon2id** (RustPress default) with the following parameters:
- Memory: 64 MiB
- Iterations: 3
- Parallelism: 4

---

## 2. Authorization (RBAC)

### 2.1 Permission Model

RustCommerce defines plugin-specific permissions that integrate with RustPress's existing RBAC system:

```toml
# In plugin.toml
[permissions]
browse_products = "Browse product catalog"
manage_own_cart = "Manage own shopping cart"
checkout = "Complete checkout"
manage_own_orders = "View own order history"
submit_reviews = "Submit product reviews"
manage_products = "Create, edit, delete products"
manage_orders = "View and manage all orders"
manage_customers = "View and manage customer accounts"
manage_inventory = "Update stock levels"
manage_coupons = "Create and manage coupons"
manage_store_settings = "Configure store settings"
manage_reviews = "Moderate product reviews"
view_store_reports = "View store analytics"
process_refunds = "Issue payment refunds"
import_export_products = "Import/export product CSV"
```

### 2.2 Role-Permission Mapping

| Permission | Customer | Shop Manager | Administrator |
|------------|----------|-------------|---------------|
| `browse_products` | Yes | Yes | Yes |
| `manage_own_cart` | Yes | Yes | Yes |
| `checkout` | Yes | Yes | Yes |
| `manage_own_orders` | Yes | Yes | Yes |
| `submit_reviews` | Yes | Yes | Yes |
| `manage_products` | No | Yes | Yes |
| `manage_orders` | No | Yes | Yes |
| `manage_customers` | No | Yes | Yes |
| `manage_inventory` | No | Yes | Yes |
| `manage_coupons` | No | Yes | Yes |
| `manage_store_settings` | No | No | Yes |
| `manage_reviews` | No | Yes | Yes |
| `view_store_reports` | No | Yes | Yes |
| `process_refunds` | No | No | Yes |
| `import_export_products` | No | Yes | Yes |

### 2.3 Permission Enforcement

Permissions are checked at the handler level using RustPress auth extractors:

```rust
pub async fn create_product(
    State(state): State<AppState>,
    auth: AuthenticatedUser,          // Extracts and validates JWT
    Json(payload): Json<CreateProductRequest>,
) -> Result<Json<Product>, Error> {
    // Check permission
    auth.require_permission("manage_products")?;

    // Proceed with product creation
    let product = product_service.create(&payload).await?;
    Ok(Json(product))
}
```

### 2.4 Row-Level Access Control

For customer-facing endpoints, RustCommerce enforces that customers can only access their own data:

```rust
pub async fn get_customer_order(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(order_id): Path<Uuid>,
) -> Result<Json<Order>, Error> {
    let order = order_repo.find_by_id(order_id).await?
        .ok_or(Error::NotFound("Order not found".into()))?;

    // Customers can only see their own orders
    if !auth.has_permission("manage_orders") {
        if order.customer_id != auth.customer_id() {
            return Err(Error::Authorization("Access denied".into()));
        }
    }

    Ok(Json(order))
}
```

---

## 3. Payment Security

### 3.1 PCI-DSS Compliance Strategy

RustCommerce achieves **PCI-DSS SAQ A** compliance by never handling raw card data:

```
┌──────────────┐     ┌────────────────────┐     ┌──────────────┐
│  Storefront   │     │  Stripe.js         │     │  Stripe API  │
│  (Browser)    │     │  (client-side SDK) │     │              │
└──────┬────────┘     └────────┬───────────┘     └──────┬───────┘
       │                       │                         │
       │  1. Load Stripe.js    │                         │
       │  ─────────────────── ▶│                         │
       │                       │                         │
       │  2. Customer enters   │                         │
       │     card details in   │                         │
       │     Stripe Element    │                         │
       │     (iframe)          │                         │
       │                       │                         │
       │                       │  3. Card data sent      │
       │                       │     directly to Stripe  │
       │                       │  ─────────────────────▶ │
       │                       │                         │
       │                       │  4. PaymentMethod ID    │
       │                       │  ◀───────────────────── │
       │                       │                         │
       │  5. pm_xxx ID sent    │                         │
       │     to our server     │                         │
       │  ◀─────────────────── │                         │
       │                       │                         │
       │  6. Server creates    │                         │
       │     PaymentIntent     │                         │
       │     with pm_xxx       ─────────────────────────▶│
       │                       │                         │
       │                       │     7. Confirmation     │
       │  ◀──────────────────────────────────────────────│
```

**Critical rule**: The RustCommerce server NEVER receives, processes, logs, or stores raw credit card numbers, CVVs, or expiration dates. Only Stripe-generated tokens (`pm_`, `pi_`, `ch_`) pass through the server.

### 3.2 Stripe PaymentIntent Flow

```rust
// Server-side: Create PaymentIntent
pub async fn create_payment_intent(
    order: &Order,
    stripe_client: &stripe::Client,
) -> Result<PaymentIntentResponse, Error> {
    let params = CreatePaymentIntent {
        amount: order.total_cents(),            // Amount in cents
        currency: order.currency.clone(),
        payment_method: Some(order.payment_method_id.clone()),
        confirmation_method: Some(PaymentIntentConfirmationMethod::Manual),
        confirm: Some(true),
        metadata: Some(HashMap::from([
            ("order_id".into(), order.id.to_string()),
            ("customer_id".into(), order.customer_id.to_string()),
        ])),
        ..Default::default()
    };

    let intent = PaymentIntent::create(&stripe_client, params).await
        .map_err(|e| Error::Plugin(format!("Stripe error: {}", e)))?;

    Ok(PaymentIntentResponse {
        client_secret: intent.client_secret,
        status: intent.status.to_string(),
    })
}
```

### 3.3 Data Stored in RustCommerce Database

| Field | Stored | Example | Purpose |
|-------|--------|---------|---------|
| Stripe PaymentIntent ID | Yes | `pi_3MtwBw...` | Reference for refunds, lookups |
| Stripe Charge ID | Yes | `ch_3MtwBw...` | Reference for refund processing |
| Payment status | Yes | `succeeded` | Order status tracking |
| Amount | Yes | `4999` (cents) | Order records |
| Currency | Yes | `usd` | Order records |
| Last 4 digits of card | Yes | `4242` | Display to customer (provided by Stripe) |
| Card brand | Yes | `visa` | Display to customer (provided by Stripe) |
| Full card number | **NEVER** | --- | PCI-DSS prohibited |
| CVV | **NEVER** | --- | PCI-DSS prohibited |
| Full expiration date | **NEVER** | --- | PCI-DSS prohibited |

### 3.4 Stripe API Key Security

| Key Type | Environment Variable | Access |
|----------|---------------------|--------|
| Publishable key (`pk_`) | `STRIPE_PUBLISHABLE_KEY` | Sent to client (safe) |
| Secret key (`sk_`) | `STRIPE_SECRET_KEY` | Server-only; never logged, never in responses |
| Webhook secret (`whsec_`) | `STRIPE_WEBHOOK_SECRET` | Server-only; used for webhook verification |

All Stripe keys are loaded from environment variables, never hardcoded, and never included in configuration files that might be committed to version control.

---

## 4. Data Encryption

### 4.1 Encryption at Rest

| Data Store | Encryption Method | Details |
|------------|------------------|---------|
| PostgreSQL | Transparent Data Encryption (TDE) | Managed by the database provider (AWS RDS, Cloud SQL, or disk-level encryption for self-hosted) |
| Redis | At-rest encryption | Managed by provider (ElastiCache) or disk encryption for self-hosted |
| File storage (S3) | AES-256 server-side encryption | `x-amz-server-side-encryption: AES256` |
| File storage (local) | Disk-level encryption | Full-disk encryption (LUKS, BitLocker, FileVault) |
| Backups | Encrypted at rest | Same encryption as primary storage |

### 4.2 Encryption in Transit

| Connection | Protocol | Minimum Version |
|------------|----------|-----------------|
| Client to server | TLS | 1.2 (1.3 preferred) |
| Server to PostgreSQL | TLS | 1.2 (via `sslmode=require` in connection string) |
| Server to Redis | TLS | 1.2 (via `rediss://` URL scheme) |
| Server to Stripe API | TLS | 1.2 (enforced by Stripe) |
| Server to S3 | TLS | 1.2 (enforced by AWS) |

### 4.3 Sensitive Data Handling

| Data Category | Storage | Encryption | Access |
|---------------|---------|------------|--------|
| Customer passwords | PostgreSQL | argon2id hash (not reversible) | Auth service only |
| Customer email | PostgreSQL | Plaintext (needed for lookups) | Authenticated access only |
| Customer addresses | PostgreSQL | Plaintext (needed for shipping) | Owner + admin only |
| Payment tokens | PostgreSQL | Plaintext Stripe IDs (not sensitive) | Server-side only |
| JWT signing secret | Environment variable | N/A (in memory) | Server process only |
| Stripe API keys | Environment variable | N/A (in memory) | Server process only |

---

## 5. Input Validation and Sanitization

### 5.1 Validation Strategy

All input is validated at two layers:

1. **Deserialization layer** (serde): Type-level validation; malformed JSON is rejected with 400.
2. **Business logic layer** (custom validators): Field-level validation rules.

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,

    #[validate(length(max = 50000))]
    pub description: Option<String>,

    #[validate(range(min = 0))]
    pub price: i64,           // Price in cents; non-negative

    #[validate(length(min = 1, max = 100))]
    pub sku: Option<String>,

    #[validate(range(min = 0))]
    pub stock_quantity: Option<i32>,

    #[validate(length(max = 500))]
    pub slug: Option<String>,
}

// In handler
pub async fn create_product(
    Json(payload): Json<CreateProductRequest>,
) -> Result<Json<Product>, Error> {
    payload.validate()
        .map_err(|e| Error::Validation(e.into()))?;
    // ...
}
```

### 5.2 Validation Rules by Entity

| Entity | Field | Rules |
|--------|-------|-------|
| Product | title | Required, 1-255 chars, trimmed |
| Product | price | Required, >= 0 (cents), integer |
| Product | sku | Optional, 1-100 chars, alphanumeric + dashes, unique |
| Product | slug | Optional (auto-generated from title), URL-safe, unique |
| Product | description | Optional, max 50,000 chars, HTML sanitized |
| Product | weight | Optional, >= 0, decimal |
| Cart Item | quantity | Required, 1-999, integer |
| Order | shipping_address | Required for physical products, validated address fields |
| Address | postal_code | Pattern-matched per country |
| Address | country_code | ISO 3166-1 alpha-2, validated against allowed list |
| Coupon | code | Required, 3-50 chars, alphanumeric + dashes, uppercase normalized |
| Coupon | discount_value | Required, > 0; if percentage, <= 100 |
| Review | rating | Required, integer 1-5 |
| Review | body | Optional, max 5,000 chars, HTML stripped |

---

## 6. SQL Injection Prevention

### 6.1 Compile-Time Query Checking

RustCommerce uses `sqlx` with compile-time checked queries wherever possible:

```rust
// Compile-time checked: query structure is validated at build time
let product = sqlx::query_as!(
    Product,
    r#"
    SELECT id, title, slug, description, price, status as "status: ProductStatus",
           created_at, updated_at
    FROM rc_products
    WHERE id = $1 AND status = 'published'
    "#,
    product_id
)
.fetch_optional(&pool)
.await?;
```

### 6.2 Parameterized Queries

All user-supplied values are passed as bind parameters (`$1`, `$2`, ...), never interpolated into query strings:

```rust
// CORRECT: Parameterized query
let products = sqlx::query_as!(
    Product,
    "SELECT * FROM rc_products WHERE category_id = $1 AND price <= $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
    category_id,
    max_price,
    limit,
    offset
)
.fetch_all(&pool)
.await?;

// NEVER: String interpolation (this would be a SQL injection vulnerability)
// let query = format!("SELECT * FROM rc_products WHERE title = '{}'", user_input);
```

### 6.3 Dynamic Query Building

For search and filtering with dynamic WHERE clauses, use a query builder that maintains parameterization:

```rust
pub struct ProductQueryBuilder {
    conditions: Vec<String>,
    params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send>>,
    param_count: usize,
}

impl ProductQueryBuilder {
    pub fn with_category(&mut self, category_id: Uuid) -> &mut Self {
        self.param_count += 1;
        self.conditions.push(format!(
            "id IN (SELECT product_id FROM rc_product_categories WHERE category_id = ${})",
            self.param_count
        ));
        self.params.push(Box::new(category_id));
        self
    }

    pub fn with_price_range(&mut self, min: Option<i64>, max: Option<i64>) -> &mut Self {
        if let Some(min) = min {
            self.param_count += 1;
            self.conditions.push(format!("price >= ${}", self.param_count));
            self.params.push(Box::new(min));
        }
        if let Some(max) = max {
            self.param_count += 1;
            self.conditions.push(format!("price <= ${}", self.param_count));
            self.params.push(Box::new(max));
        }
        self
    }
}
```

---

## 7. XSS Prevention

### 7.1 Product Description Sanitization

Product descriptions may contain rich HTML (from the admin editor). This HTML is sanitized on save using an allowlist approach:

```rust
use ammonia::Builder;

pub fn sanitize_product_html(input: &str) -> String {
    Builder::new()
        .tags(hashset!["p", "br", "strong", "em", "u", "s", "h2", "h3", "h4",
                       "ul", "ol", "li", "a", "img", "blockquote", "pre", "code",
                       "table", "thead", "tbody", "tr", "th", "td"])
        .link_rel(Some("noopener noreferrer"))
        .url_schemes(hashset!["http", "https"])
        .add_generic_attributes(&["class"])
        .add_tag_attributes("a", &["href", "title"])
        .add_tag_attributes("img", &["src", "alt", "width", "height"])
        .clean(input)
        .to_string()
}
```

### 7.2 API Response Encoding

All API responses are JSON-encoded by default (via serde), which escapes special characters. The `Content-Type: application/json` header prevents browsers from interpreting responses as HTML.

### 7.3 Admin UI Protection

The React admin UI uses JSX, which auto-escapes interpolated values. The only exception is `dangerouslySetInnerHTML` for rendering product descriptions, which must only render sanitized HTML from the server:

```tsx
// Product description is already sanitized server-side
<div
  className="product-description prose"
  dangerouslySetInnerHTML={{ __html: product.description }}
/>
```

### 7.4 Review Content

Customer review text is stored and rendered as plain text, never interpreted as HTML. User-submitted HTML tags are stripped on input:

```rust
pub fn sanitize_review_text(input: &str) -> String {
    ammonia::clean(input) // Strips all HTML tags by default
}
```

---

## 8. Rate Limiting per Endpoint Category

### 8.1 Rate Limit Tiers

| Tier | Endpoints | Limit | Window | Key |
|------|-----------|-------|--------|-----|
| **Public Read** | GET /products, /categories | 120/min | Per IP | IP address |
| **Cart Write** | POST/PUT/DELETE /cart/* | 60/min | Per session | Session ID |
| **Search** | GET /products/search | 30/min | Per IP | IP address |
| **Checkout** | POST /checkout/* | 10/min | Per session | Session ID |
| **Payment** | POST /payments | 5/min | Per session | Session ID |
| **Auth** | POST /auth/login, /auth/register | 10/min | Per IP | IP address |
| **Admin Read** | GET /admin/* | 120/min | Per user | User ID |
| **Admin Write** | POST/PUT/DELETE /admin/* | 60/min | Per user | User ID |
| **Webhook** | POST /webhooks/* | 100/min | Per IP | IP address |
| **Review Submit** | POST /reviews | 5/min | Per user | User ID |

### 8.2 Rate Limit Response

When a rate limit is exceeded, the server returns:

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 30
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1740001800

{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests. Please try again in 30 seconds."
  }
}
```

### 8.3 Brute Force Protection

For authentication-related endpoints, progressive rate limiting is applied:

| Failed Attempts (per IP) | Delay |
|--------------------------|-------|
| 1-5 | No delay |
| 6-10 | 1 second response delay |
| 11-20 | 5 second response delay |
| 21+ | 30 second lockout, then CAPTCHA required |

---

## 9. Additional Security Measures

### 9.1 CSRF Protection

All state-changing requests from the admin UI and storefront forms include a CSRF token, managed by RustPress's `rustpress-auth` crate:

- CSRF token is set as a cookie (`__Host-csrf`) and must be echoed back as a header (`X-CSRF-Token`).
- API requests using `Authorization: Bearer` header are exempt from CSRF (token-based auth is not vulnerable to CSRF).
- Cookie-based sessions (guest cart) require CSRF validation on all non-GET requests.

### 9.2 Content Security Policy

RustPress sets a CSP header that restricts resource loading:

```
Content-Security-Policy:
  default-src 'self';
  script-src 'self' https://js.stripe.com;
  frame-src 'self' https://js.stripe.com;
  img-src 'self' data: https://cdn.example.com;
  style-src 'self' 'unsafe-inline';
  connect-src 'self' https://api.stripe.com;
```

The CSP explicitly allows Stripe.js (required for payment element iframe).

### 9.3 Dependency Security

- All Rust dependencies are audited via `cargo audit` in CI.
- The `stripe-rust` crate is pinned to a specific version and reviewed before upgrade.
- npm dependencies (admin UI) are audited via `npm audit` in CI.
- Dependabot (or equivalent) is configured for automated security update PRs.

### 9.4 Logging and Audit Trail

Security-relevant events are logged to the RustPress audit log (`audit_logs` table):

| Event | Logged Data |
|-------|-------------|
| Admin login | User ID, IP, timestamp, user agent |
| Failed login attempt | Email attempted, IP, timestamp |
| Product created/updated/deleted | Admin user ID, product ID, changes |
| Order status changed | Admin user ID, order ID, old/new status |
| Refund issued | Admin user ID, order ID, amount |
| Settings changed | Admin user ID, setting key, old/new value |
| Webhook received | Event type, Stripe event ID, processing result |
| Permission denied | User ID, requested permission, endpoint |

### 9.5 Security Checklist

| Category | Measure | Status |
|----------|---------|--------|
| Authentication | JWT with short-lived access tokens | Planned |
| Authentication | Refresh token rotation | Planned |
| Authentication | Password hashing with argon2id | Inherited from RustPress |
| Authorization | RBAC with fine-grained permissions | Planned |
| Authorization | Row-level access control for customer data | Planned |
| Payment | Stripe-only card handling (PCI SAQ A) | Planned |
| Payment | Webhook signature verification | Planned |
| Payment | Idempotent webhook processing | Planned |
| Data | TLS 1.2+ for all connections | Planned |
| Data | Encrypted at-rest storage | Planned |
| Input | serde type validation + custom validators | Planned |
| Input | sqlx compile-time query checking | Planned |
| Input | HTML sanitization (ammonia) | Planned |
| Input | CSRF protection on cookie-authenticated endpoints | Inherited from RustPress |
| Rate limiting | Per-endpoint rate limits | Planned |
| Rate limiting | Brute force protection on auth endpoints | Planned |
| Monitoring | Audit logging for admin actions | Planned |
| Dependencies | cargo audit + npm audit in CI | Planned |
| Headers | Security headers (HSTS, CSP, X-Frame-Options) | Inherited from RustPress |
