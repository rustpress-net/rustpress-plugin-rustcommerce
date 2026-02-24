# RustCommerce Authentication and Authorization

**Version**: 1.0.0
**Date**: 2026-02-24
**Status**: Approved

---

## Table of Contents

1. [Overview](#1-overview)
2. [RustPress JWT Integration](#2-rustpress-jwt-integration)
3. [User Types and Session Flows](#3-user-types-and-session-flows)
4. [Permission Model](#4-permission-model)
5. [Endpoint Authorization Matrix](#5-endpoint-authorization-matrix)
6. [Guest vs Authenticated Flows](#6-guest-vs-authenticated-flows)
7. [Admin Permission Matrix](#7-admin-permission-matrix)
8. [Stripe Webhook Verification](#8-stripe-webhook-verification)
9. [CSRF Protection](#9-csrf-protection)
10. [Rate Limiting Strategy](#10-rate-limiting-strategy)
11. [Security Considerations](#11-security-considerations)

---

## 1. Overview

RustCommerce does not implement its own authentication system. It fully integrates with the RustPress core authentication and authorization infrastructure, which provides:

- **JWT-based authentication** (access + refresh tokens)
- **Role-based access control (RBAC)** with capabilities
- **API key authentication** for programmatic access
- **Session management** with device tracking

RustCommerce extends this system by:

1. Defining e-commerce-specific **permissions** (capabilities)
2. Implementing a **guest session** mechanism for anonymous shopping
3. Adding **Stripe webhook signature verification** for payment callbacks
4. Applying **CSRF protection** to checkout-sensitive operations

---

## 2. RustPress JWT Integration

### 2.1 Token Format

RustPress issues JWTs with the following claims:

```json
{
  "sub": "0195fa9f-aaaa-7def-8abc-123456789012",
  "email": "user@example.com",
  "username": "johndoe",
  "role": "subscriber",
  "capabilities": ["read", "manage_own_profile"],
  "iat": 1708776000,
  "exp": 1708776900,
  "iss": "rustpress",
  "jti": "tok_01HXYZ..."
}
```

### 2.2 How RustCommerce Reads the JWT

RustCommerce uses the shared Axum middleware from `rustpress-auth` to extract and validate the JWT:

```rust
use rustpress_auth::middleware::OptionalAuth;
use rustpress_auth::middleware::RequireAuth;
use rustpress_auth::middleware::RequirePermission;

// Public endpoint - JWT optional, extracts user if present
async fn list_products(
    OptionalAuth(user): OptionalAuth,
    // ...
) -> Result<Json<...>, Error> { ... }

// Customer endpoint - JWT required
async fn list_my_orders(
    RequireAuth(user): RequireAuth,
    // ...
) -> Result<Json<...>, Error> { ... }

// Admin endpoint - JWT required + specific permission
async fn create_product(
    RequirePermission(user, "manage_products"): RequirePermission,
    // ...
) -> Result<Json<...>, Error> { ... }
```

### 2.3 Middleware Stack

The RustCommerce route tree applies authentication middleware at different levels:

```
/api/v1/rustcommerce/
    ├── Public routes (no auth middleware)
    │   ├── GET /products
    │   ├── GET /products/:id
    │   ├── GET /categories
    │   ├── GET /categories/:id
    │   ├── GET /products/:product_id/reviews
    │   ├── GET /shipping/methods
    │   ├── POST /tax/calculate
    │   └── POST /coupons/validate
    │
    ├── Session routes (OptionalAuth + X-Session-ID)
    │   ├── GET /cart
    │   ├── POST /cart/items
    │   ├── PUT /cart/items/:id
    │   ├── DELETE /cart/items/:id
    │   ├── DELETE /cart
    │   ├── POST /cart/coupon
    │   ├── DELETE /cart/coupon
    │   ├── POST /checkout/init
    │   ├── POST /checkout/shipping-address
    │   ├── POST /checkout/shipping-method
    │   ├── POST /checkout/payment-intent
    │   └── POST /checkout/complete
    │
    ├── Customer routes (RequireAuth)
    │   ├── GET /orders
    │   ├── GET /orders/:id
    │   ├── GET /account
    │   ├── PUT /account
    │   ├── GET /account/addresses
    │   ├── POST /account/addresses
    │   ├── PUT /account/addresses/:id
    │   ├── DELETE /account/addresses/:id
    │   ├── POST /reviews
    │   ├── POST /reviews/:id/helpful
    │   └── GET /payments/:id
    │
    ├── Admin routes (RequirePermission)
    │   ├── /admin/products/* -> manage_products
    │   ├── /admin/categories/* -> manage_products
    │   ├── /admin/orders/* -> manage_orders
    │   ├── /admin/customers/* -> manage_customers
    │   ├── /admin/settings/* -> manage_store_settings
    │   ├── /admin/shipping/* -> manage_store_settings
    │   ├── /admin/tax/* -> manage_store_settings
    │   ├── /admin/coupons/* -> manage_store_settings
    │   ├── /admin/inventory/* -> manage_products
    │   ├── /admin/reviews/* -> manage_products
    │   ├── /admin/analytics/* -> view_store_reports
    │   └── /admin/payments/methods -> manage_store_settings
    │
    └── Webhook routes (custom verification)
        └── POST /webhooks/stripe -> Stripe signature verification
```

### 2.4 Token Flow

```
                  ┌───────────────┐
                  │   Client App  │
                  │  (Browser/    │
                  │   Mobile)     │
                  └──────┬────────┘
                         │
           ┌─────────────┴──────────────┐
           │                            │
     Login Flow                    API Call
           │                            │
           ▼                            ▼
  POST /api/v1/auth/login     GET /api/v1/rustcommerce/orders
  { email, password }         Authorization: Bearer <access_token>
           │                            │
           ▼                            ▼
  ┌──────────────┐            ┌──────────────────┐
  │ RustPress    │            │ RustPress Auth   │
  │ Auth Service │            │ Middleware        │
  │              │            │                  │
  │ Returns:     │            │ 1. Extract JWT   │
  │ access_token │            │ 2. Verify sig    │
  │ refresh_token│            │ 3. Check expiry  │
  │ expires_in   │            │ 4. Extract claims│
  └──────────────┘            │ 5. Inject User   │
                              └────────┬─────────┘
                                       │
                                       ▼
                              ┌──────────────────┐
                              │ RustCommerce     │
                              │ Handler          │
                              │                  │
                              │ Receives:        │
                              │ - AuthUser struct│
                              │ - user.id        │
                              │ - user.role      │
                              │ - user.caps      │
                              └──────────────────┘
```

### 2.5 Token Refresh

Access tokens expire after a configurable period (default: 15 minutes). Clients use the refresh token to obtain a new access token:

```
POST /api/v1/auth/refresh
{
  "refresh_token": "rt_01HXYZ..."
}
```

RustCommerce does not handle refresh logic -- it is entirely managed by RustPress core.

---

## 3. User Types and Session Flows

### 3.1 User Types

| User Type | Authentication | Capabilities |
|-----------|----------------|-------------|
| **Anonymous Guest** | `X-Session-ID` header | Browse products, manage cart, checkout (if guest checkout enabled) |
| **Registered Customer** | JWT Bearer token | All guest capabilities + order history, saved addresses, reviews, wishlist |
| **Store Admin** | JWT Bearer token + admin role | All customer capabilities + product/order/customer/settings management |
| **Super Admin** | JWT Bearer token + administrator role | Full access including dangerous operations (delete customers, manage API keys) |

### 3.2 Guest Session Flow

Guest users are identified by a client-generated UUID sent in the `X-Session-ID` header:

```
┌──────────────┐                    ┌──────────────────┐
│   Browser    │                    │  RustCommerce    │
└──────┬───────┘                    └────────┬─────────┘
       │                                     │
       │  1. Generate UUID on first visit    │
       │     (stored in localStorage)        │
       │                                     │
       │  GET /api/v1/rustcommerce/cart      │
       │  X-Session-ID: <generated-uuid>     │
       │  ─────────────────────────────────► │
       │                                     │
       │                   2. Create rc_carts│
       │                      record with    │
       │                      session_id     │
       │                                     │
       │  200 OK { cart }                    │
       │  ◄───────────────────────────────── │
       │                                     │
       │  POST /api/v1/rustcommerce/cart/items
       │  X-Session-ID: <same-uuid>          │
       │  { product_id, quantity }           │
       │  ─────────────────────────────────► │
       │                                     │
       │  200 OK { updated cart }            │
       │  ◄───────────────────────────────── │
```

**Session ID Rules:**
- Must be a valid UUID v4 format.
- Client generates and persists it (localStorage or cookie).
- Session carts expire after 7 days of inactivity.
- The `X-Session-ID` header is only used when no `Authorization` header is present.

### 3.3 Guest-to-User Cart Merge

When a guest user with an existing cart logs in or registers:

```
┌──────────────┐                    ┌──────────────────┐
│   Browser    │                    │  RustCommerce    │
└──────┬───────┘                    └────────┬─────────┘
       │                                     │
       │  POST /api/v1/auth/login            │
       │  { email, password }                │
       │  ─────────────────────────────────► │ (handled by RustPress)
       │                                     │
       │  200 OK { access_token }            │
       │  ◄───────────────────────────────── │
       │                                     │
       │  GET /api/v1/rustcommerce/cart      │
       │  Authorization: Bearer <token>      │
       │  X-Session-ID: <guest-session-id>   │
       │  ─────────────────────────────────► │
       │                                     │
       │  RustCommerce detects both:         │
       │  1. user_id from JWT                │
       │  2. session_id from header          │
       │                                     │
       │  Cart merge logic:                  │
       │  - If user has existing cart:       │
       │    merge guest items into user cart  │
       │    (quantities are added if same    │
       │     product/variant exists)          │
       │  - If user has no cart:             │
       │    reassign guest cart to user      │
       │    (set user_id, clear session_id)  │
       │  - Delete guest cart                │
       │                                     │
       │  200 OK { merged cart }             │
       │  ◄───────────────────────────────── │
```

**Merge Rules:**
1. If the same product+variant exists in both carts, use the higher quantity.
2. Items unique to the guest cart are added to the user cart.
3. Items unique to the user cart are preserved.
4. The guest cart is marked as `converted` and then deleted.
5. The user cart's `coupon_code` is preserved (user cart takes precedence).

### 3.4 Customer Registration During Checkout

If guest checkout is enabled, a guest can complete checkout without registering. During checkout, they provide an email. After order completion:

1. If a `rc_customers` record with that email exists, the order is linked to that customer.
2. If not, a new `rc_customers` record is created with `user_id = NULL`.
3. If the guest later registers with the same email, the customer record's `user_id` is backfilled.

---

## 4. Permission Model

### 4.1 RustCommerce Permissions (Capabilities)

These permissions are registered with RustPress's RBAC system during plugin activation:

| Permission | Slug | Description |
|-----------|------|-------------|
| Manage Products | `manage_products` | Create, edit, delete products, categories, images, variants. Manage inventory and reviews. |
| Manage Orders | `manage_orders` | View all orders, update status, process refunds, add notes. |
| Manage Customers | `manage_customers` | View all customers, edit profiles, anonymize data. |
| Manage Store Settings | `manage_store_settings` | Configure currency, payments, shipping, tax, coupons, email settings. |
| Manage Store Templates | `manage_store_templates` | Upload and manage storefront templates. |
| View Store Reports | `view_store_reports` | Access analytics dashboard, revenue reports, product performance. |
| Manage API Keys | `manage_api_keys` | Create and manage store API keys for external integrations. |

### 4.2 Default Role Assignments

| RustPress Role | RustCommerce Permissions |
|----------------|------------------------|
| `administrator` | All 7 permissions |
| `editor` | `manage_products`, `manage_orders`, `view_store_reports` |
| `author` | `manage_products` (own products only -- future consideration) |
| `subscriber` | None (customer-level, uses own data endpoints) |

These role-permission mappings are seeded during plugin activation and can be customized by the site admin through the RustPress role management UI.

### 4.3 Permission Registration

During `activate()`, RustCommerce registers its permissions:

```rust
async fn activate(&self, ctx: &AppContext) -> Result<()> {
    let auth_service = ctx.get::<AuthService>()?;

    // Register custom capabilities
    auth_service.register_capability("manage_products", "Manage store products, categories, and inventory").await?;
    auth_service.register_capability("manage_orders", "Manage customer orders and process refunds").await?;
    auth_service.register_capability("manage_customers", "Manage customer accounts and data").await?;
    auth_service.register_capability("manage_store_settings", "Configure store settings, payments, shipping, and tax").await?;
    auth_service.register_capability("manage_store_templates", "Manage storefront templates").await?;
    auth_service.register_capability("view_store_reports", "View store analytics and reports").await?;
    auth_service.register_capability("manage_api_keys", "Manage store API keys").await?;

    // Assign to administrator role
    auth_service.grant_capabilities_to_role("administrator", &[
        "manage_products", "manage_orders", "manage_customers",
        "manage_store_settings", "manage_store_templates",
        "view_store_reports", "manage_api_keys"
    ]).await?;

    // Assign to editor role
    auth_service.grant_capabilities_to_role("editor", &[
        "manage_products", "manage_orders", "view_store_reports"
    ]).await?;

    Ok(())
}
```

---

## 5. Endpoint Authorization Matrix

### 5.1 Public Endpoints (No Auth)

| # | Endpoint | Method | Description |
|---|----------|--------|-------------|
| 1 | `/products` | GET | List published products |
| 2 | `/products/:id` | GET | Get product detail |
| 3 | `/categories` | GET | List categories |
| 4 | `/categories/:id` | GET | Get category detail |
| 5 | `/products/:id/reviews` | GET | List approved reviews |
| 6 | `/shipping/methods` | GET | Get available shipping methods |
| 7 | `/tax/calculate` | POST | Calculate tax estimate |
| 8 | `/coupons/validate` | POST | Validate a coupon code |
| 9 | `/reviews/:id/helpful` | POST | Vote a review helpful |

### 5.2 Session Endpoints (Guest or Authenticated)

These endpoints work with either `X-Session-ID` (guest) or `Authorization` (authenticated):

| # | Endpoint | Method | Description |
|---|----------|--------|-------------|
| 10 | `/cart` | GET | Get current cart |
| 11 | `/cart/items` | POST | Add item to cart |
| 12 | `/cart/items/:id` | PUT | Update cart item quantity |
| 13 | `/cart/items/:id` | DELETE | Remove item from cart |
| 14 | `/cart` | DELETE | Clear cart |
| 15 | `/cart/coupon` | POST | Apply coupon |
| 16 | `/cart/coupon` | DELETE | Remove coupon |
| 17 | `/checkout/init` | POST | Start checkout |
| 18 | `/checkout/shipping-address` | POST | Set shipping address |
| 19 | `/checkout/shipping-method` | POST | Select shipping method |
| 20 | `/checkout/payment-intent` | POST | Create Stripe PaymentIntent |
| 21 | `/checkout/complete` | POST | Finalize order |

**Note**: If `guest_checkout_enabled` is `false` in store settings, endpoints 17-21 require authentication (JWT).

### 5.3 Customer Endpoints (Auth Required)

| # | Endpoint | Method | Required Auth |
|---|----------|--------|---------------|
| 22 | `/orders` | GET | Valid JWT (returns own orders) |
| 23 | `/orders/:id` | GET | Valid JWT + owns order |
| 24 | `/account` | GET | Valid JWT |
| 25 | `/account` | PUT | Valid JWT |
| 26 | `/account/addresses` | GET | Valid JWT |
| 27 | `/account/addresses` | POST | Valid JWT |
| 28 | `/account/addresses/:id` | PUT | Valid JWT + owns address |
| 29 | `/account/addresses/:id` | DELETE | Valid JWT + owns address |
| 30 | `/reviews` | POST | Valid JWT |
| 31 | `/payments/:id` | GET | Valid JWT + owns order |

**Ownership Check**: Customer endpoints verify that the requested resource belongs to the authenticated user. A customer cannot access another customer's orders, addresses, or payment details.

### 5.4 Admin Endpoints (Permission Required)

| # | Endpoint Group | Permission Required |
|---|---------------|-------------------|
| 32-36 | `/admin/products/*` | `manage_products` |
| 37-38 | `/admin/categories/*` | `manage_products` |
| 39-44 | `/admin/orders/*` | `manage_orders` |
| 45-48 | `/admin/customers/*` | `manage_customers` |
| 49-50 | `/admin/settings` | `manage_store_settings` |
| 51-56 | `/admin/shipping/*` | `manage_store_settings` |
| 57-59 | `/admin/tax/*` | `manage_store_settings` |
| 60-63 | `/admin/coupons/*` | `manage_store_settings` |
| 64-66 | `/admin/inventory/*` | `manage_products` |
| 67-69 | `/admin/reviews/*` | `manage_products` |
| 70-72 | `/admin/analytics/*` | `view_store_reports` |
| 73 | `/admin/payments/methods` | `manage_store_settings` |

---

## 6. Guest vs Authenticated Flows

### 6.1 Browsing (No Difference)

Both guests and authenticated users can:
- Browse products and categories
- View reviews
- Check shipping methods
- Estimate taxes

### 6.2 Cart (Different Persistence)

| Aspect | Guest | Authenticated |
|--------|-------|---------------|
| Cart Storage | `rc_carts` with `session_id` | `rc_carts` with `user_id` |
| Identification | `X-Session-ID` header | JWT `sub` claim |
| Persistence | 7-day TTL | Indefinite (until checkout or manual clear) |
| Cross-device | No (tied to session) | Yes (tied to user account) |
| Coupon application | Yes | Yes |

### 6.3 Checkout (Conditional)

```
                     Guest Checkout Enabled?
                            │
                    ┌───────┴───────┐
                    │ YES           │ NO
                    ▼               ▼
            ┌───────────┐   ┌──────────────┐
            │ Guest can │   │ 401 Error:   │
            │ checkout  │   │ "Please log  │
            │ with email│   │ in to        │
            │ only      │   │ checkout"    │
            └───────────┘   └──────────────┘
                 │
                 ▼
         Provide email at
         checkout/init
                 │
                 ▼
         Complete checkout
                 │
         ┌───────┴──────┐
         │              │
         ▼              ▼
    Email matches    No match:
    existing         Create new
    rc_customer →    rc_customer
    Link order       (user_id=NULL)
```

### 6.4 Order History

| Aspect | Guest | Authenticated |
|--------|-------|---------------|
| View past orders | No API access (email confirmation only) | `GET /orders` |
| Track specific order | Via order confirmation link (with order token) | `GET /orders/:id` |
| Reorder | No | Future feature |

### 6.5 Reviews

| Aspect | Guest | Authenticated |
|--------|-------|---------------|
| Read reviews | Yes | Yes |
| Write reviews | No | Yes |
| Verified badge | N/A | Yes (if purchased product) |
| Vote helpful | Yes (IP-limited) | Yes (account-limited) |

---

## 7. Admin Permission Matrix

### 7.1 Detailed Permission Matrix

| Action | `manage_products` | `manage_orders` | `manage_customers` | `manage_store_settings` | `view_store_reports` | `manage_api_keys` |
|--------|:-----------------:|:---------------:|:------------------:|:-----------------------:|:--------------------:|:-----------------:|
| **Products** | | | | | | |
| Create product | X | | | | | |
| Edit product | X | | | | | |
| Delete product | X | | | | | |
| Manage categories | X | | | | | |
| Manage images | X | | | | | |
| Manage variants | X | | | | | |
| View inventory | X | | | | | |
| Update stock | X | | | | | |
| Moderate reviews | X | | | | | |
| Bulk operations | X | | | | | |
| **Orders** | | | | | | |
| View all orders | | X | | | | |
| View order detail | | X | | | | |
| Update order status | | X | | | | |
| Process refunds | | X | | | | |
| Add order notes | | X | | | | |
| Cancel orders | | X | | | | |
| **Customers** | | | | | | |
| View customer list | | | X | | | |
| View customer detail | | | X | | | |
| Edit customer info | | | X | | | |
| View customer orders | | X | X | | | |
| Anonymize customer | | | X | | | |
| **Settings** | | | | | | |
| View store settings | | | | X | | |
| Update settings | | | | X | | |
| Configure Stripe | | | | X | | |
| Manage shipping zones | | | | X | | |
| Manage tax rates | | | | X | | |
| Manage coupons | | | | X | | |
| **Analytics** | | | | | | |
| View dashboard | | | | | X | |
| View revenue report | | | | | X | |
| View product stats | | | | | X | |
| Export reports | | | | | X | |
| **API Keys** | | | | | | |
| Create API keys | | | | | | X |
| Revoke API keys | | | | | | X |
| View API usage | | | | | | X |

### 7.2 Permission Inheritance

Administrators automatically inherit all permissions. The permission system does not have explicit inheritance for other roles; each role must be explicitly granted the permissions it needs.

### 7.3 Permission Checking Implementation

```rust
/// Axum extractor that requires a specific permission
pub struct RequirePermission(pub AuthUser, pub &'static str);

#[async_trait]
impl<S> FromRequestParts<S> for RequirePermission
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract and validate JWT (via RustPress auth middleware)
        let user = extract_auth_user(parts, state).await?;

        // 2. Check if user has the required capability
        let required_permission = parts.extensions.get::<RequiredPermission>()
            .ok_or(Error::Internal("Missing permission requirement".into()))?;

        if !user.capabilities.contains(&required_permission.0.to_string()) {
            return Err(Error::Authorization(format!(
                "Permission '{}' required", required_permission.0
            )));
        }

        Ok(RequirePermission(user, required_permission.0))
    }
}
```

---

## 8. Stripe Webhook Verification

### 8.1 Overview

Stripe sends webhook events to `POST /api/v1/rustcommerce/webhooks/stripe`. These requests must be verified using Stripe's signature verification to prevent spoofing.

### 8.2 Verification Flow

```
┌──────────────┐                    ┌──────────────────┐
│   Stripe     │                    │  RustCommerce    │
│   Servers    │                    │  Webhook Handler │
└──────┬───────┘                    └────────┬─────────┘
       │                                     │
       │  POST /webhooks/stripe              │
       │  Stripe-Signature: t=1708776000,    │
       │    v1=5257a869e7ecebeda32a...       │
       │  Content-Type: application/json     │
       │  Body: { "id": "evt_...", ... }     │
       │  ─────────────────────────────────► │
       │                                     │
       │                                     │  1. Read raw request body
       │                                     │     (NOT parsed JSON yet)
       │                                     │
       │                                     │  2. Extract Stripe-Signature header
       │                                     │     Parse: t={timestamp}, v1={signature}
       │                                     │
       │                                     │  3. Compute expected signature:
       │                                     │     payload = "{timestamp}.{raw_body}"
       │                                     │     expected = HMAC-SHA256(
       │                                     │       key=webhook_secret, msg=payload
       │                                     │     )
       │                                     │
       │                                     │  4. Compare signatures:
       │                                     │     If v1 != expected → 400 Bad Request
       │                                     │
       │                                     │  5. Check timestamp tolerance:
       │                                     │     If |now - t| > 300s → 400 (replay)
       │                                     │
       │                                     │  6. Parse JSON and process event
       │                                     │
       │  200 OK { "received": true }        │
       │  ◄───────────────────────────────── │
```

### 8.3 Implementation

```rust
use stripe::Webhook;

pub async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,   // Raw body, not parsed JSON
) -> Result<Json<WebhookResponse>, Error> {
    // 1. Get the webhook signing secret from store settings
    let webhook_secret = state.settings
        .get("stripe_webhook_secret")
        .ok_or(Error::Internal("Stripe webhook secret not configured".into()))?;

    // 2. Get the Stripe-Signature header
    let sig_header = headers.get("Stripe-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(Error::Authentication("Missing Stripe-Signature header".into()))?;

    // 3. Verify the signature using the stripe-rust crate
    let event = Webhook::construct_event(
        &String::from_utf8_lossy(&body),
        sig_header,
        &webhook_secret,
    ).map_err(|e| Error::Authentication(format!("Invalid webhook signature: {}", e)))?;

    // 4. Process the event
    match event.type_ {
        EventType::PaymentIntentSucceeded => {
            handle_payment_succeeded(&state, &event).await?;
        }
        EventType::PaymentIntentPaymentFailed => {
            handle_payment_failed(&state, &event).await?;
        }
        EventType::ChargeRefunded => {
            handle_charge_refunded(&state, &event).await?;
        }
        EventType::ChargeDisputeCreated => {
            handle_dispute_created(&state, &event).await?;
        }
        _ => {
            tracing::info!("Unhandled Stripe event type: {}", event.type_);
        }
    }

    Ok(Json(WebhookResponse { received: true }))
}
```

### 8.4 Webhook Secret Management

- The Stripe webhook signing secret (`whsec_...`) is stored in `rc_store_settings` under the key `stripe_webhook_secret`.
- It is never exposed in API responses (the settings endpoint returns `"webhook_secret_set": true/false`, never the actual value).
- It is configured during store setup via the admin settings page.
- For test mode, a separate test webhook secret can be configured.

### 8.5 Webhook Idempotency

Stripe may send the same event multiple times. RustCommerce handles this by:

1. Storing the `event.id` (e.g., `evt_1PxYz123456789`) in a processed events cache (Redis or in-memory with TTL).
2. Before processing, checking if the event has already been handled.
3. If duplicate, returning `200 OK` without re-processing.

```rust
async fn is_event_processed(state: &AppState, event_id: &str) -> bool {
    state.cache.get(&format!("stripe_event:{}", event_id)).await.is_some()
}

async fn mark_event_processed(state: &AppState, event_id: &str) {
    // TTL of 24 hours to prevent reprocessing
    state.cache.set(
        &format!("stripe_event:{}", event_id),
        "processed",
        Duration::from_secs(86400)
    ).await;
}
```

### 8.6 Webhook Retry Handling

If RustCommerce returns a non-2xx response, Stripe retries with exponential backoff (up to 3 days). To prevent data inconsistency:

- The webhook handler always returns `200 OK` after successfully receiving the event, even if downstream processing fails.
- Failed processing is queued as a background job for retry.
- A dead-letter mechanism logs permanently failed events for admin review.

---

## 9. CSRF Protection

### 9.1 Where CSRF Protection Applies

CSRF protection is critical for state-changing operations initiated from web browsers:

| Endpoint | CSRF Required | Reason |
|----------|:------------:|--------|
| `POST /cart/items` | Yes | Prevents adding items via cross-site request |
| `PUT /cart/items/:id` | Yes | Prevents quantity manipulation |
| `DELETE /cart/items/:id` | Yes | Prevents item removal |
| `POST /checkout/init` | Yes | Prevents unauthorized checkout initiation |
| `POST /checkout/shipping-address` | Yes | Protects address submission |
| `POST /checkout/shipping-method` | Yes | Protects shipping selection |
| `POST /checkout/payment-intent` | Yes | Critical - prevents unauthorized payment creation |
| `POST /checkout/complete` | Yes | Critical - prevents unauthorized order creation |
| `POST /reviews` | Yes | Prevents fake review submission |
| All admin write endpoints | Yes | Protects admin actions |

### 9.2 Implementation Strategy: Double Submit Cookie

RustCommerce uses the Double Submit Cookie pattern, which is already implemented in RustPress core via the `rustpress-auth` crate:

```
┌──────────────┐                    ┌──────────────────┐
│   Browser    │                    │  RustPress       │
└──────┬───────┘                    └────────┬─────────┘
       │                                     │
       │  GET /api/v1/auth/csrf-token        │
       │  ─────────────────────────────────► │
       │                                     │
       │  200 OK                             │
       │  Set-Cookie: csrf_token=<token>;    │
       │    HttpOnly; SameSite=Strict;       │
       │    Secure; Path=/                   │
       │  Body: { "csrf_token": "<token>" }  │
       │  ◄───────────────────────────────── │
       │                                     │
       │  POST /api/v1/rustcommerce/cart/items
       │  Cookie: csrf_token=<token>         │
       │  X-CSRF-Token: <token>              │
       │  Body: { product_id, quantity }     │
       │  ─────────────────────────────────► │
       │                                     │
       │     Middleware checks:              │
       │     cookie.csrf_token == header.    │
       │     X-CSRF-Token                    │
       │                                     │
       │  200 OK { cart }                    │
       │  ◄───────────────────────────────── │
```

### 9.3 CSRF Exemptions

| Endpoint | Exempted | Reason |
|----------|:--------:|--------|
| `GET *` | Yes | Safe methods don't need CSRF |
| `POST /webhooks/stripe` | Yes | Not browser-initiated; uses Stripe signature verification instead |
| All endpoints with only `Authorization: Bearer` (no cookies) | Yes | Token-based auth is inherently CSRF-safe |

### 9.4 SameSite Cookie Configuration

```
Set-Cookie: csrf_token=<value>; HttpOnly; Secure; SameSite=Strict; Path=/
```

- `HttpOnly`: Prevents JavaScript access (mitigates XSS-based CSRF token theft)
- `Secure`: Only sent over HTTPS
- `SameSite=Strict`: Never sent with cross-site requests

---

## 10. Rate Limiting Strategy

### 10.1 Rate Limit Tiers

Rate limits are applied per IP address for anonymous users and per user ID for authenticated users:

| Tier | Target | Window | Limit |
|------|--------|--------|-------|
| **Public Read** | Product/category listings | 1 minute | 60 (anon), 120 (auth) |
| **Cart Operations** | Add/update/remove cart items | 1 minute | 30 (anon), 60 (auth) |
| **Checkout** | Init, address, shipping, complete | 1 minute | 5 (anon), 10 (auth) |
| **Payment** | Create PaymentIntent | 1 minute | 3 (anon), 5 (auth) |
| **Reviews** | Submit review | 1 minute | 3 (auth only) |
| **Admin Read** | List/detail admin endpoints | 1 minute | 120 |
| **Admin Write** | Create/update/delete admin endpoints | 1 minute | 60 |
| **Coupon Validation** | Validate coupon | 1 minute | 10 (anon), 30 (auth) |
| **Webhooks** | Stripe callbacks | N/A | Unlimited (verified) |

### 10.2 Rate Limit Enforcement

RustCommerce uses RustPress's rate limiting middleware:

```rust
use rustpress_server::middleware::rate_limit;

pub fn checkout_routes(state: AppState) -> Router {
    Router::new()
        .route("/init", post(checkout::init))
        .route("/shipping-address", post(checkout::shipping_address))
        .route("/shipping-method", post(checkout::shipping_method))
        .route("/payment-intent", post(checkout::payment_intent))
        .route("/complete", post(checkout::complete))
        .layer(rate_limit::per_minute(10))  // 10 requests/min for authenticated
        .with_state(state)
}
```

### 10.3 Rate Limit Response

When rate limit is exceeded:

```
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1708776060
Retry-After: 45

{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests. Please try again in 45 seconds.",
    "status": 429,
    "retry_after": 45
  }
}
```

---

## 11. Security Considerations

### 11.1 PCI-DSS Compliance

RustCommerce is designed for **PCI-DSS SAQ A** compliance (the least restrictive level):

- **Never stores** raw credit card numbers, CVV, or full card data.
- Payment is handled entirely by **Stripe Elements** (client-side) and **Stripe PaymentIntent** (server-side).
- The server only handles Stripe tokens and PaymentIntent IDs.
- No card data passes through or is logged by RustCommerce servers.

### 11.2 Input Validation

All user input is validated before processing:

| Input | Validation |
|-------|-----------|
| UUIDs | Regex: `^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$` |
| Prices | Decimal(10,2), non-negative |
| Quantities | Integer, 1-999 |
| Slugs | Regex: `^[a-z0-9]+(?:-[a-z0-9]+)*$`, max 255 chars |
| Emails | RFC 5322 compliant |
| Country codes | ISO 3166-1 alpha-2 (validated against list) |
| HTML content | Sanitized with allowlist (product descriptions) |
| Phone numbers | Regex: `^\+?[\d\s\-\(\)]+$`, max 50 chars |

### 11.3 SQL Injection Prevention

- All database queries use **parameterized queries** via sqlx.
- No string concatenation for SQL.
- Compile-time query checking via `sqlx::query!()` macro where possible.

### 11.4 XSS Prevention

- All JSON responses use `Content-Type: application/json`.
- Product descriptions (HTML) are sanitized on input using an allowlist (permitted tags: `p`, `br`, `strong`, `em`, `ul`, `ol`, `li`, `a`, `h2`, `h3`, `h4`, `blockquote`).
- Admin-entered content is trusted but still sanitized.

### 11.5 Sensitive Data Handling

| Data | Storage | API Exposure |
|------|---------|-------------|
| Stripe Secret Key | `rc_store_settings` (encrypted at rest) | Never returned. Settings API shows `"secret_key_set": true/false` |
| Stripe Webhook Secret | `rc_store_settings` (encrypted at rest) | Never returned |
| Customer Passwords | Not stored (RustPress core handles auth) | N/A |
| Payment Card Data | Not stored (Stripe handles) | N/A |
| Customer PII | `rc_customers`, `rc_customer_addresses` | Only to the customer themselves or admin |
| IP Addresses | `rc_orders.ip_address` | Admin-only |
| Order Addresses | `rc_orders` (JSONB snapshots) | Customer (own orders) and admin |

### 11.6 GDPR Considerations

- **Customer Deletion**: The `DELETE /admin/customers/:id` endpoint anonymizes PII rather than deleting records, preserving order history integrity.
- **Data Export**: Customers can request their data via the account API (future: full GDPR export endpoint).
- **Consent**: Guest checkout captures email consent as part of the checkout flow.

### 11.7 Audit Logging

All admin actions are logged via the RustPress audit log system:

```rust
// Example: Product creation audit log
audit_log::record(AuditEntry {
    action: "product.created",
    entity_type: "product",
    entity_id: product.id,
    user_id: admin_user.id,
    details: json!({
        "product_name": product.name,
        "sku": product.sku
    }),
    ip_address: request_ip,
});
```

Audited actions include:
- Product create/update/delete
- Order status changes
- Refund processing
- Customer data changes
- Settings modifications
- Coupon create/update/delete
- Stock adjustments
