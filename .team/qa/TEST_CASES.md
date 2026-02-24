# RustCommerce Test Case Catalog

**Document Version**: 1.0
**Date**: 2026-02-24
**Prepared By**: QA Lead
**Project**: RustCommerce (RCOM-001)
**Total Test Cases**: 52

---

## Legend

| Field | Description |
|-------|-------------|
| **ID** | Unique test case identifier: `TC-{AREA}-{NUM}` |
| **Priority** | P0 = Must pass for MVP, P1 = Important, P2 = Nice to have |
| **Category** | Functional area |

---

## 1. Product CRUD (10 cases)

### TC-PROD-001: Create simple product with all required fields

| Field | Value |
|-------|-------|
| **ID** | TC-PROD-001 |
| **Title** | Create a simple product with all required fields |
| **Category** | Product Management |
| **Priority** | P0 |
| **Preconditions** | Admin user is authenticated with `manage_products` permission |
| **Steps** | 1. Send `POST /api/v1/rustcommerce/admin/products` with body: `{ "name": "Wireless Headphones", "price": "49.99", "sku": "WH-001", "status": "published", "product_type": "simple", "stock_quantity": 100 }` |
| **Expected Result** | 201 Created. Response contains the created product with a UUID `id`, auto-generated `slug` ("wireless-headphones"), `created_at` timestamp, `stock_status` = "in_stock". Product is retrievable via `GET /products/{id}`. |

---

### TC-PROD-002: Read product by ID (public endpoint)

| Field | Value |
|-------|-------|
| **ID** | TC-PROD-002 |
| **Title** | Retrieve a published product by ID without authentication |
| **Category** | Product Management |
| **Priority** | P0 |
| **Preconditions** | A published product exists in the database |
| **Steps** | 1. Send `GET /api/v1/rustcommerce/products/{product_id}` without any auth header |
| **Expected Result** | 200 OK. Response contains the full product object including name, description, price, images, categories, stock_status. `cost_price` field is NOT included in the public response. |

---

### TC-PROD-003: Update product price and description

| Field | Value |
|-------|-------|
| **ID** | TC-PROD-003 |
| **Title** | Update an existing product's price and description |
| **Category** | Product Management |
| **Priority** | P0 |
| **Preconditions** | Admin user authenticated. Product with known ID exists. |
| **Steps** | 1. Send `PUT /api/v1/rustcommerce/admin/products/{id}` with body: `{ "price": "59.99", "description": "Updated description" }` |
| **Expected Result** | 200 OK. Product price is updated to "59.99", description is updated. `updated_at` timestamp is newer than `created_at`. Other fields remain unchanged. |

---

### TC-PROD-004: Delete (archive) a product

| Field | Value |
|-------|-------|
| **ID** | TC-PROD-004 |
| **Title** | Archive a product (soft delete) |
| **Category** | Product Management |
| **Priority** | P0 |
| **Preconditions** | Admin user authenticated. Published product exists. |
| **Steps** | 1. Send `DELETE /api/v1/rustcommerce/admin/products/{id}` 2. Send `GET /api/v1/rustcommerce/products/{id}` (public) |
| **Expected Result** | Step 1: 200 OK, product status changes to "archived". Step 2: 404 Not Found (archived products are not visible publicly). Admin GET still returns the product with status "archived". |

---

### TC-PROD-005: Create variable product with variants

| Field | Value |
|-------|-------|
| **ID** | TC-PROD-005 |
| **Title** | Create a variable product with size and color variants |
| **Category** | Product Management |
| **Priority** | P0 |
| **Preconditions** | Admin user authenticated |
| **Steps** | 1. Send `POST /api/v1/rustcommerce/admin/products` with `product_type: "variable"` and `variants` array containing: `[{ "name": "Small/Red", "sku": "WH-S-R", "price": "49.99", "stock_quantity": 25, "attributes": {"size": "S", "color": "Red"} }, { "name": "Large/Blue", "sku": "WH-L-B", "price": "54.99", "stock_quantity": 30, "attributes": {"size": "L", "color": "Blue"} }]` |
| **Expected Result** | 201 Created. Product has `product_type` = "variable". Two variants created with their own UUIDs, SKUs, and prices. Parent product `stock_quantity` = 55 (sum of variants). |

---

### TC-PROD-006: Add images to a product

| Field | Value |
|-------|-------|
| **ID** | TC-PROD-006 |
| **Title** | Upload and associate images with a product |
| **Category** | Product Management |
| **Priority** | P1 |
| **Preconditions** | Admin user authenticated. Product exists. |
| **Steps** | 1. Upload an image via the RustPress media system 2. Send `POST /api/v1/rustcommerce/admin/products/{id}/images` with `{ "image_url": "/media/product-photo.jpg", "alt_text": "Product front view", "position": 0 }` |
| **Expected Result** | 201 Created. Image is associated with the product. `GET /products/{id}` includes the image in the `images` array with correct `alt_text` and `position`. |

---

### TC-PROD-007: Assign product to categories

| Field | Value |
|-------|-------|
| **ID** | TC-PROD-007 |
| **Title** | Assign a product to multiple categories |
| **Category** | Product Management |
| **Priority** | P0 |
| **Preconditions** | Admin user authenticated. Product and two categories exist. |
| **Steps** | 1. Send `PUT /api/v1/rustcommerce/admin/products/{id}` with `{ "category_ids": ["{cat_id_1}", "{cat_id_2}"] }` 2. Send `GET /api/v1/rustcommerce/products?category_id={cat_id_1}` |
| **Expected Result** | Step 1: 200 OK. Step 2: The product appears in the filtered results for category 1. Product detail also returns both categories in its `categories` array. |

---

### TC-PROD-008: Search products by text query

| Field | Value |
|-------|-------|
| **ID** | TC-PROD-008 |
| **Title** | Full-text search for products by name and description |
| **Category** | Product Management |
| **Priority** | P1 |
| **Preconditions** | Multiple published products exist, including one named "Wireless Bluetooth Headphones" |
| **Steps** | 1. Send `GET /api/v1/rustcommerce/products?q=bluetooth+headphones` |
| **Expected Result** | 200 OK. Results include the "Wireless Bluetooth Headphones" product. Results are ranked by relevance. Only published products appear. |

---

### TC-PROD-009: Filter products by price range and status

| Field | Value |
|-------|-------|
| **ID** | TC-PROD-009 |
| **Title** | Filter product listing by price range and status |
| **Category** | Product Management |
| **Priority** | P1 |
| **Preconditions** | Products exist with various prices and statuses |
| **Steps** | 1. Send `GET /api/v1/rustcommerce/products?min_price=20.00&max_price=50.00&status=published` |
| **Expected Result** | 200 OK. All returned products have price >= 20.00 and <= 50.00, and status = "published". No draft or archived products appear. |

---

### TC-PROD-010: Paginate product listings with cursor

| Field | Value |
|-------|-------|
| **ID** | TC-PROD-010 |
| **Title** | Cursor-based pagination through product listings |
| **Category** | Product Management |
| **Priority** | P0 |
| **Preconditions** | At least 25 published products exist |
| **Steps** | 1. Send `GET /api/v1/rustcommerce/products?limit=10` 2. Extract `cursor` from response `pagination` object 3. Send `GET /api/v1/rustcommerce/products?limit=10&cursor={cursor}` 4. Repeat until `has_more` = false |
| **Expected Result** | Each page returns up to 10 products. No product appears in more than one page. All pages combined equal the total product count. `has_more` is false on the last page. `cursor` is null when there are no more pages. |

---

## 2. Cart Operations (8 cases)

### TC-CART-001: Add item to cart

| Field | Value |
|-------|-------|
| **ID** | TC-CART-001 |
| **Title** | Add a product to the shopping cart |
| **Category** | Cart Operations |
| **Priority** | P0 |
| **Preconditions** | Published product with stock > 0 exists |
| **Steps** | 1. Send `POST /api/v1/rustcommerce/cart/items` with `X-Session-ID` header and body: `{ "product_id": "{id}", "quantity": 2 }` |
| **Expected Result** | 200 OK. Cart response contains the item with correct `product_id`, `quantity` = 2, `unit_price` matching the product price, `line_total` = unit_price * 2. Cart `subtotal` reflects the item total. |

---

### TC-CART-002: Update cart item quantity

| Field | Value |
|-------|-------|
| **ID** | TC-CART-002 |
| **Title** | Update the quantity of an existing cart item |
| **Category** | Cart Operations |
| **Priority** | P0 |
| **Preconditions** | Cart exists with at least one item |
| **Steps** | 1. Send `PUT /api/v1/rustcommerce/cart/items/{item_id}` with body: `{ "quantity": 5 }` |
| **Expected Result** | 200 OK. Item quantity updated to 5. Cart subtotal recalculated. |

---

### TC-CART-003: Remove item from cart

| Field | Value |
|-------|-------|
| **ID** | TC-CART-003 |
| **Title** | Remove a specific item from the cart |
| **Category** | Cart Operations |
| **Priority** | P0 |
| **Preconditions** | Cart exists with at least two items |
| **Steps** | 1. Send `DELETE /api/v1/rustcommerce/cart/items/{item_id}` 2. Send `GET /api/v1/rustcommerce/cart` |
| **Expected Result** | Step 1: 200 OK. Step 2: Cart no longer contains the deleted item. Other items remain. Subtotal recalculated. |

---

### TC-CART-004: Guest cart with session ID

| Field | Value |
|-------|-------|
| **ID** | TC-CART-004 |
| **Title** | Guest user creates and retrieves cart using session ID |
| **Category** | Cart Operations |
| **Priority** | P0 |
| **Preconditions** | No authentication. Valid `X-Session-ID` UUID generated. |
| **Steps** | 1. Send `POST /api/v1/rustcommerce/cart/items` with `X-Session-ID: {uuid}` and a product 2. Send `GET /api/v1/rustcommerce/cart` with same `X-Session-ID` 3. Send `GET /api/v1/rustcommerce/cart` with a DIFFERENT `X-Session-ID` |
| **Expected Result** | Step 2: Returns the cart with the added item. Step 3: Returns an empty cart (different session = different cart). |

---

### TC-CART-005: Merge guest cart on login

| Field | Value |
|-------|-------|
| **ID** | TC-CART-005 |
| **Title** | Guest cart merges with authenticated user cart on login |
| **Category** | Cart Operations |
| **Priority** | P1 |
| **Preconditions** | Guest has a cart with 2 items (session ID). Authenticated user has a saved cart with 1 different item. |
| **Steps** | 1. User logs in (obtains JWT) 2. Send `POST /api/v1/rustcommerce/cart/merge` with JWT and `X-Session-ID` header |
| **Expected Result** | 200 OK. Merged cart contains all 3 items. If both carts had the same product, quantities are summed. Guest cart is deleted. |

---

### TC-CART-006: Apply coupon to cart

| Field | Value |
|-------|-------|
| **ID** | TC-CART-006 |
| **Title** | Apply a valid percentage coupon to the cart |
| **Category** | Cart Operations |
| **Priority** | P1 |
| **Preconditions** | Cart has items with subtotal $100.00. A 10% coupon code "SAVE10" exists and is active. |
| **Steps** | 1. Send `POST /api/v1/rustcommerce/cart/coupon` with body: `{ "code": "SAVE10" }` 2. Send `GET /api/v1/rustcommerce/cart` |
| **Expected Result** | Step 1: 200 OK with coupon validation response showing estimated discount. Step 2: Cart shows `discount_total` = "10.00", coupon code applied, `grand_total` reflects the discount. |

---

### TC-CART-007: Cart expiration cleanup

| Field | Value |
|-------|-------|
| **ID** | TC-CART-007 |
| **Title** | Abandoned cart is cleaned up after expiration period |
| **Category** | Cart Operations |
| **Priority** | P2 |
| **Preconditions** | A guest cart exists that has not been accessed for more than the configured expiration period (e.g., 30 days) |
| **Steps** | 1. Create a cart with items 2. Simulate passage of time beyond expiration (or set `updated_at` to 31 days ago in test DB) 3. Run the cart cleanup background job 4. Attempt to retrieve the cart |
| **Expected Result** | The expired cart is deleted. Retrieval returns an empty cart. |

---

### TC-CART-008: Stock validation on add to cart

| Field | Value |
|-------|-------|
| **ID** | TC-CART-008 |
| **Title** | Adding more items than available stock is rejected |
| **Category** | Cart Operations |
| **Priority** | P0 |
| **Preconditions** | Product exists with `stock_quantity` = 3, `backorders_allowed` = false |
| **Steps** | 1. Send `POST /api/v1/rustcommerce/cart/items` with `{ "product_id": "{id}", "quantity": 5 }` |
| **Expected Result** | 422 Unprocessable Entity. Error code `UNPROCESSABLE_ENTITY` with message indicating insufficient stock. Cart is not modified. |

---

## 3. Checkout Flow (10 cases)

### TC-CHK-001: Happy path checkout (authenticated customer)

| Field | Value |
|-------|-------|
| **ID** | TC-CHK-001 |
| **Title** | Complete checkout flow from cart to order confirmation |
| **Category** | Checkout |
| **Priority** | P0 |
| **Preconditions** | Authenticated customer. Cart has 2 items with sufficient stock. Shipping and tax rates configured. Stripe test mode active. |
| **Steps** | 1. `POST /checkout/init` -- initialize checkout session 2. `POST /checkout/shipping-address` with valid address 3. `POST /checkout/shipping-method` with a valid method ID 4. `POST /checkout/payment-intent` -- get Stripe client_secret 5. Simulate Stripe payment success (webhook `payment_intent.succeeded`) 6. `POST /checkout/complete` |
| **Expected Result** | Each step returns 200 OK. After completion: order created with status "confirmed", stock decremented, payment record created, cart cleared. Order response includes `order_number`, items, totals, shipping address. |

---

### TC-CHK-002: Guest checkout without login

| Field | Value |
|-------|-------|
| **ID** | TC-CHK-002 |
| **Title** | Guest user completes checkout without creating an account |
| **Category** | Checkout |
| **Priority** | P0 |
| **Preconditions** | Guest checkout is enabled in store settings. Guest has a cart via `X-Session-ID`. |
| **Steps** | 1. `POST /checkout/init` with `X-Session-ID` and `{ "email": "guest@example.com" }` 2. Provide shipping address 3. Select shipping method 4. Create payment intent 5. Simulate Stripe payment success 6. Complete checkout |
| **Expected Result** | Order is created with `customer_id` = null (or a guest customer record). Email is stored on the order. Order confirmation can be sent to the provided email. |

---

### TC-CHK-003: Address validation rejects invalid address

| Field | Value |
|-------|-------|
| **ID** | TC-CHK-003 |
| **Title** | Checkout rejects an incomplete or invalid shipping address |
| **Category** | Checkout |
| **Priority** | P0 |
| **Preconditions** | Checkout session initialized |
| **Steps** | 1. `POST /checkout/shipping-address` with `{ "address_line1": "", "city": "", "state": "XX", "postal_code": "abc", "country": "ZZ" }` |
| **Expected Result** | 400 Validation Error. Error details list each invalid field: address_line1 required, city required, invalid state code, invalid postal code format, unknown country code. |

---

### TC-CHK-004: Shipping method selection with cost calculation

| Field | Value |
|-------|-------|
| **ID** | TC-CHK-004 |
| **Title** | Select a shipping method and verify cost is calculated correctly |
| **Category** | Checkout |
| **Priority** | P0 |
| **Preconditions** | Checkout session with valid shipping address. Flat-rate shipping method ($5.99) configured for the address zone. |
| **Steps** | 1. `GET /shipping/methods?address_id={addr_id}` to list available methods 2. `POST /checkout/shipping-method` with the flat-rate method ID |
| **Expected Result** | Step 1: Returns available shipping methods for the zone with calculated costs. Step 2: 200 OK. Cart totals updated: `shipping_total` = "5.99", `grand_total` includes shipping cost. |

---

### TC-CHK-005: Payment success via Stripe webhook

| Field | Value |
|-------|-------|
| **ID** | TC-CHK-005 |
| **Title** | Stripe payment_intent.succeeded webhook creates confirmed order |
| **Category** | Checkout |
| **Priority** | P0 |
| **Preconditions** | Checkout session created, payment intent created, stock reserved |
| **Steps** | 1. Construct a valid `payment_intent.succeeded` webhook event payload with correct checkout_session_id in metadata 2. Sign the payload with the Stripe webhook signing secret 3. Send `POST /api/v1/rustcommerce/webhooks/stripe` with `Stripe-Signature` header |
| **Expected Result** | 200 OK. Order status transitions to "confirmed". Payment record created in `rc_payments` with Stripe charge ID, amount, last4, card brand. Stock reservations converted. Actual stock decremented. `order_confirmed` hook fired. |

---

### TC-CHK-006: Payment failure handling

| Field | Value |
|-------|-------|
| **ID** | TC-CHK-006 |
| **Title** | Stripe payment failure does not create an order |
| **Category** | Checkout |
| **Priority** | P0 |
| **Preconditions** | Checkout session and payment intent exist |
| **Steps** | 1. Simulate a `payment_intent.payment_failed` webhook event 2. Check order status 3. Check stock reservations |
| **Expected Result** | Webhook returns 200 OK (acknowledged). No order is created or order stays in "pending" status. Stock reservations remain active (they will expire naturally). Payment record is created with status "failed". |

---

### TC-CHK-007: Stock reservation during checkout

| Field | Value |
|-------|-------|
| **ID** | TC-CHK-007 |
| **Title** | Stock is reserved when checkout is initiated and prevents overselling |
| **Category** | Checkout |
| **Priority** | P0 |
| **Preconditions** | Product with `stock_quantity` = 2. Two customers have the product in their carts. |
| **Steps** | 1. Customer A initiates checkout (reserves 1 unit) 2. Customer B initiates checkout (reserves 1 unit) 3. Customer C tries to initiate checkout for the same product (quantity 1) |
| **Expected Result** | Steps 1-2: Checkout init succeeds. Available stock = 0 (2 actual - 2 reserved). Step 3: 422 Unprocessable Entity with "Insufficient stock" error. |

---

### TC-CHK-008: Concurrent checkout does not oversell

| Field | Value |
|-------|-------|
| **ID** | TC-CHK-008 |
| **Title** | Two concurrent checkouts for the last item -- only one succeeds |
| **Category** | Checkout |
| **Priority** | P0 |
| **Preconditions** | Product with `stock_quantity` = 1. Two customers each have 1 of this product in their cart. |
| **Steps** | 1. Simultaneously send two `POST /checkout/init` requests for the same product 2. Both attempt to complete payment |
| **Expected Result** | Exactly one checkout succeeds. The other receives a stock validation error. Database stock never goes negative. The `stock_quantity >= 0` constraint is enforced. |

---

### TC-CHK-009: Coupon discount applied at checkout

| Field | Value |
|-------|-------|
| **ID** | TC-CHK-009 |
| **Title** | Coupon discount is correctly applied to the checkout totals |
| **Category** | Checkout |
| **Priority** | P1 |
| **Preconditions** | Cart subtotal = $100.00. Valid 15% coupon applied. Shipping = $5.99. Tax rate = 8.5%. |
| **Steps** | 1. Apply coupon to cart 2. Initiate checkout 3. Set shipping address (triggers tax calculation) 4. Select shipping method 5. Create payment intent |
| **Expected Result** | Discount = $15.00 (15% of $100). Taxable amount = $85.00. Tax = $7.23 (rounded). Grand total = $85.00 + $7.23 + $5.99 = $98.22. Payment intent amount = 9822 cents. |

---

### TC-CHK-010: Order creation populates all required fields

| Field | Value |
|-------|-------|
| **ID** | TC-CHK-010 |
| **Title** | Completed checkout creates an order with all required data |
| **Category** | Checkout |
| **Priority** | P0 |
| **Preconditions** | Successful checkout completion |
| **Steps** | 1. Complete the full checkout flow 2. Retrieve the created order via `GET /orders/{id}` |
| **Expected Result** | Order has: unique `order_number` (format RC-YYYYMMDD-XXXXX), `status` = "confirmed", `payment_status` = "paid", correct `subtotal`, `discount_total`, `tax_total`, `shipping_total`, `grand_total`, `shipping_address` (JSONB snapshot), `billing_address`, `items` array with product snapshots, `customer_id`, `created_at`. |

---

## 4. Order Management (8 cases)

### TC-ORD-001: Valid order status transition (confirmed to processing)

| Field | Value |
|-------|-------|
| **ID** | TC-ORD-001 |
| **Title** | Admin transitions order from confirmed to processing |
| **Category** | Order Management |
| **Priority** | P0 |
| **Preconditions** | Admin authenticated. Order exists with status "confirmed". |
| **Steps** | 1. Send `PUT /api/v1/rustcommerce/admin/orders/{id}/status` with `{ "status": "processing" }` |
| **Expected Result** | 200 OK. Order status = "processing". Status change recorded in `rc_order_status_history` with admin user ID and timestamp. `order_processing` hook fired. |

---

### TC-ORD-002: Invalid order status transition rejected

| Field | Value |
|-------|-------|
| **ID** | TC-ORD-002 |
| **Title** | Attempting an invalid status transition returns an error |
| **Category** | Order Management |
| **Priority** | P0 |
| **Preconditions** | Order exists with status "pending" |
| **Steps** | 1. Send `PUT /api/v1/rustcommerce/admin/orders/{id}/status` with `{ "status": "shipped" }` |
| **Expected Result** | 400 Invalid Operation. Error message: "Cannot transition from 'pending' to 'shipped'. Valid transitions: ['confirmed', 'cancelled']". Order status unchanged. |

---

### TC-ORD-003: Process a full refund

| Field | Value |
|-------|-------|
| **ID** | TC-ORD-003 |
| **Title** | Admin processes a full refund for a delivered order |
| **Category** | Order Management |
| **Priority** | P0 |
| **Preconditions** | Admin authenticated. Order exists with status "delivered", payment_status "paid". |
| **Steps** | 1. Send `POST /api/v1/rustcommerce/admin/orders/{id}/refund` with `{ "amount": "{full_amount}", "reason": "Customer request", "restock": true }` |
| **Expected Result** | 200 OK. Stripe refund API called. Order status transitions to "refunded". `payment_status` = "refunded". Refund record created in `rc_refunds`. Stock restored (restock = true). `refund_issued` hook fired. |

---

### TC-ORD-004: Admin updates order with tracking number

| Field | Value |
|-------|-------|
| **ID** | TC-ORD-004 |
| **Title** | Admin ships an order with tracking information |
| **Category** | Order Management |
| **Priority** | P0 |
| **Preconditions** | Order with status "processing" |
| **Steps** | 1. Send `PUT /api/v1/rustcommerce/admin/orders/{id}/status` with `{ "status": "shipped", "tracking_number": "1Z999AA10123456784", "tracking_url": "https://tracking.example.com/1Z999AA10123456784" }` |
| **Expected Result** | 200 OK. Order status = "shipped". Tracking number and URL saved on the order. `order_shipped` hook fired. |

---

### TC-ORD-005: Customer views order history

| Field | Value |
|-------|-------|
| **ID** | TC-ORD-005 |
| **Title** | Authenticated customer retrieves their order history |
| **Category** | Order Management |
| **Priority** | P0 |
| **Preconditions** | Customer authenticated. Customer has 3 orders. |
| **Steps** | 1. Send `GET /api/v1/rustcommerce/orders` with customer JWT |
| **Expected Result** | 200 OK. Returns only the 3 orders belonging to this customer. Does not include orders from other customers. Orders sorted by `created_at` descending. Each order includes summary fields (order_number, status, grand_total, item_count, created_at). |

---

### TC-ORD-006: Search orders by order number

| Field | Value |
|-------|-------|
| **ID** | TC-ORD-006 |
| **Title** | Admin searches for an order by order number |
| **Category** | Order Management |
| **Priority** | P1 |
| **Preconditions** | Admin authenticated. Multiple orders exist. |
| **Steps** | 1. Send `GET /api/v1/rustcommerce/admin/orders?q=RC-20260224-00042` |
| **Expected Result** | 200 OK. Returns exactly one order matching the order number. |

---

### TC-ORD-007: Filter orders by status and date range

| Field | Value |
|-------|-------|
| **ID** | TC-ORD-007 |
| **Title** | Admin filters orders by status and date range |
| **Category** | Order Management |
| **Priority** | P1 |
| **Preconditions** | Admin authenticated. Orders exist across various statuses and dates. |
| **Steps** | 1. Send `GET /api/v1/rustcommerce/admin/orders?status=processing&date_from=2026-02-01&date_to=2026-02-28` |
| **Expected Result** | 200 OK. All returned orders have status "processing" and `created_at` within the date range. |

---

### TC-ORD-008: Order export to CSV

| Field | Value |
|-------|-------|
| **ID** | TC-ORD-008 |
| **Title** | Admin exports orders to CSV file |
| **Category** | Order Management |
| **Priority** | P2 |
| **Preconditions** | Admin authenticated with orders in the system |
| **Steps** | 1. Send `GET /api/v1/rustcommerce/admin/orders/export?format=csv&status=confirmed,processing` |
| **Expected Result** | 200 OK. Content-Type: `text/csv`. Response body is a valid CSV with headers: order_number, customer_email, status, subtotal, tax, shipping, grand_total, created_at. Only confirmed and processing orders are included. |

---

## 5. Payment Integration (6 cases)

### TC-PAY-001: Create PaymentIntent with correct amount

| Field | Value |
|-------|-------|
| **ID** | TC-PAY-001 |
| **Title** | PaymentIntent is created with the correct amount in cents |
| **Category** | Payment |
| **Priority** | P0 |
| **Preconditions** | Checkout session with `grand_total` = "98.22" |
| **Steps** | 1. Send `POST /api/v1/rustcommerce/checkout/payment-intent` 2. Verify the Stripe API call (via wiremock) |
| **Expected Result** | Stripe API called with `amount: 9822` (cents), `currency: "usd"`. Response includes `client_secret`. Checkout session updated with `payment_intent_id`. |

---

### TC-PAY-002: Webhook signature verification succeeds for valid signature

| Field | Value |
|-------|-------|
| **ID** | TC-PAY-002 |
| **Title** | Valid Stripe webhook with correct signature is processed |
| **Category** | Payment |
| **Priority** | P0 |
| **Preconditions** | Webhook signing secret configured. Valid event payload constructed. |
| **Steps** | 1. Construct webhook payload 2. Generate valid `Stripe-Signature` header using the signing secret 3. Send `POST /api/v1/rustcommerce/webhooks/stripe` |
| **Expected Result** | 200 OK. Event is processed. Order status updated accordingly. |

---

### TC-PAY-003: Webhook with forged signature is rejected

| Field | Value |
|-------|-------|
| **ID** | TC-PAY-003 |
| **Title** | Webhook with invalid/forged signature is rejected |
| **Category** | Payment |
| **Priority** | P0 |
| **Preconditions** | Webhook endpoint is accessible |
| **Steps** | 1. Send `POST /api/v1/rustcommerce/webhooks/stripe` with a valid-looking payload but an incorrect `Stripe-Signature` header |
| **Expected Result** | 401 Unauthorized. No order or payment records are modified. Verification failure is logged with source IP. |

---

### TC-PAY-004: Webhook replay attack is rejected

| Field | Value |
|-------|-------|
| **ID** | TC-PAY-004 |
| **Title** | Replayed webhook event (old timestamp) is rejected |
| **Category** | Payment |
| **Priority** | P0 |
| **Preconditions** | A previously processed webhook event |
| **Steps** | 1. Resend the exact same webhook event (same signature, same payload) with a timestamp older than 5 minutes |
| **Expected Result** | The webhook is rejected due to timestamp tolerance check. If the event was previously processed, the idempotency check also prevents re-processing. |

---

### TC-PAY-005: Refund via Stripe API

| Field | Value |
|-------|-------|
| **ID** | TC-PAY-005 |
| **Title** | Refund is correctly processed through Stripe |
| **Category** | Payment |
| **Priority** | P0 |
| **Preconditions** | Order with successful payment. Stripe charge ID stored. |
| **Steps** | 1. Admin initiates refund for $25.00 2. Verify Stripe refund API call (via wiremock) 3. Check database records |
| **Expected Result** | Stripe Refund API called with correct charge_id and amount (2500 cents). Refund record created in `rc_refunds` with Stripe refund ID. Order `payment_status` updated to "partially_refunded" (or "refunded" if full amount). |

---

### TC-PAY-006: Payment in test mode uses test keys

| Field | Value |
|-------|-------|
| **ID** | TC-PAY-006 |
| **Title** | Payment processing uses Stripe test keys in non-production environments |
| **Category** | Payment |
| **Priority** | P0 |
| **Preconditions** | Environment configured with Stripe test keys (sk_test_*) |
| **Steps** | 1. Create a payment intent 2. Inspect the Stripe API call |
| **Expected Result** | API call uses the test secret key. Payment intent ID starts with "pi_" (not a live transaction). No real charges are created. |

---

## 6. Authentication / Authorization (5 cases)

### TC-AUTH-001: Guest access to public endpoints

| Field | Value |
|-------|-------|
| **ID** | TC-AUTH-001 |
| **Title** | Unauthenticated users can access public product and category endpoints |
| **Category** | Auth/Authz |
| **Priority** | P0 |
| **Preconditions** | Public products and categories exist |
| **Steps** | 1. Send `GET /api/v1/rustcommerce/products` without any auth header 2. Send `GET /api/v1/rustcommerce/categories` without any auth header |
| **Expected Result** | Both return 200 OK with data. No authentication error. |

---

### TC-AUTH-002: Customer access to own orders only

| Field | Value |
|-------|-------|
| **ID** | TC-AUTH-002 |
| **Title** | Authenticated customer can only access their own orders |
| **Category** | Auth/Authz |
| **Priority** | P0 |
| **Preconditions** | Customer A and Customer B each have orders. Customer A's JWT is available. |
| **Steps** | 1. Send `GET /api/v1/rustcommerce/orders` with Customer A's JWT 2. Send `GET /api/v1/rustcommerce/orders/{customer_b_order_id}` with Customer A's JWT |
| **Expected Result** | Step 1: Returns only Customer A's orders. Step 2: 404 Not Found (or 403 Forbidden) -- Customer A cannot access Customer B's order. |

---

### TC-AUTH-003: Admin access with manage_products permission

| Field | Value |
|-------|-------|
| **ID** | TC-AUTH-003 |
| **Title** | Admin with manage_products can create products but not manage orders |
| **Category** | Auth/Authz |
| **Priority** | P0 |
| **Preconditions** | Admin user with ONLY `manage_products` capability (no `manage_orders`) |
| **Steps** | 1. Send `POST /api/v1/rustcommerce/admin/products` with a valid product -- should succeed 2. Send `PUT /api/v1/rustcommerce/admin/orders/{id}/status` -- should be denied |
| **Expected Result** | Step 1: 201 Created. Step 2: 403 Permission Denied with code `PERMISSION_DENIED`. |

---

### TC-AUTH-004: Unauthenticated access to protected endpoint

| Field | Value |
|-------|-------|
| **ID** | TC-AUTH-004 |
| **Title** | Protected endpoints reject requests without valid JWT |
| **Category** | Auth/Authz |
| **Priority** | P0 |
| **Preconditions** | None |
| **Steps** | 1. Send `GET /api/v1/rustcommerce/orders` without Authorization header 2. Send `GET /api/v1/rustcommerce/orders` with `Authorization: Bearer invalid_token_here` |
| **Expected Result** | Step 1: 401 Authentication Required. Step 2: 401 Authentication Required (invalid JWT). Both return error code `AUTHENTICATION_REQUIRED`. |

---

### TC-AUTH-005: Expired JWT is rejected

| Field | Value |
|-------|-------|
| **ID** | TC-AUTH-005 |
| **Title** | Expired JWT token is rejected on all protected endpoints |
| **Category** | Auth/Authz |
| **Priority** | P0 |
| **Preconditions** | A JWT token that has expired (exp claim in the past) |
| **Steps** | 1. Send `GET /api/v1/rustcommerce/orders` with the expired JWT in the Authorization header |
| **Expected Result** | 401 Authentication Required. Error indicates token expiration. Client should use the refresh token flow. |

---

## 7. Business Logic (5 cases)

### TC-BIZ-001: Tax calculation with compounding rates

| Field | Value |
|-------|-------|
| **ID** | TC-BIZ-001 |
| **Title** | Tax is calculated correctly with compound and non-compound rates |
| **Category** | Business Logic |
| **Priority** | P0 |
| **Preconditions** | Tax rates configured: NY State 4.00% (priority 1, non-compound), NYC Local 4.50% (priority 1, non-compound), MTA Surcharge 0.375% (priority 2, compound). Product priced at $100.00. Shipping address is in NYC. |
| **Steps** | 1. Calculate tax for the $100.00 product with the NYC address |
| **Expected Result** | Priority 1 sum: 4.00% + 4.50% = 8.50%. Tax at P1 = $8.50. Compound base for P2 = $100.00 + $8.50 = $108.50. Tax at P2 = $108.50 * 0.00375 = $0.41. Total tax = $8.91. Tax breakdown shows all 3 line items. |

---

### TC-BIZ-002: Shipping cost with free threshold

| Field | Value |
|-------|-------|
| **ID** | TC-BIZ-002 |
| **Title** | Flat-rate shipping is waived when cart meets free shipping threshold |
| **Category** | Business Logic |
| **Priority** | P0 |
| **Preconditions** | Flat-rate shipping method: $5.99, free_threshold: $100.00 |
| **Steps** | 1. Calculate shipping for cart subtotal = $150.00 2. Calculate shipping for cart subtotal = $50.00 |
| **Expected Result** | Step 1: Shipping = $0.00 (subtotal exceeds threshold). Step 2: Shipping = $5.99 (subtotal below threshold). |

---

### TC-BIZ-003: Inventory stock reservation and expiration

| Field | Value |
|-------|-------|
| **ID** | TC-BIZ-003 |
| **Title** | Stock reservation expires after 10 minutes and releases inventory |
| **Category** | Business Logic |
| **Priority** | P0 |
| **Preconditions** | Product with `stock_quantity` = 5. Reservation hold time = 10 minutes. |
| **Steps** | 1. Initiate checkout (reserves 2 units) 2. Verify available stock = 3 3. Simulate 10 minutes passing (advance reservation `expires_at`) 4. Run the stock reservation cleanup job 5. Verify available stock = 5 |
| **Expected Result** | Step 2: Available stock = 3 (5 actual - 2 reserved). Step 5: Available stock = 5 (reservation expired and cleaned up). |

---

### TC-BIZ-004: Coupon validation rejects expired coupon

| Field | Value |
|-------|-------|
| **ID** | TC-BIZ-004 |
| **Title** | Applying an expired coupon returns a clear error |
| **Category** | Business Logic |
| **Priority** | P1 |
| **Preconditions** | Coupon exists with `expires_at` = yesterday |
| **Steps** | 1. Send `POST /api/v1/rustcommerce/cart/coupon` with the expired coupon code |
| **Expected Result** | 400 or 422 error. Error code = "COUPON_EXPIRED". Message = "This coupon has expired". Coupon is not applied to the cart. |

---

### TC-BIZ-005: Order number generation is unique and correctly formatted

| Field | Value |
|-------|-------|
| **ID** | TC-BIZ-005 |
| **Title** | Each order receives a unique, correctly formatted order number |
| **Category** | Business Logic |
| **Priority** | P0 |
| **Preconditions** | None |
| **Steps** | 1. Create 10 orders in rapid succession 2. Collect all order numbers |
| **Expected Result** | All 10 order numbers are unique. Each matches the format `RC-YYYYMMDD-XXXXX` where YYYYMMDD is the current date and XXXXX is a zero-padded sequential number. Numbers are monotonically increasing. |

---

## Summary

| Category | Count | P0 | P1 | P2 |
|----------|:-----:|:--:|:--:|:--:|
| Product CRUD | 10 | 6 | 3 | 1 |
| Cart Operations | 8 | 5 | 2 | 1 |
| Checkout Flow | 10 | 8 | 2 | 0 |
| Order Management | 8 | 5 | 2 | 1 |
| Payment Integration | 6 | 6 | 0 | 0 |
| Auth/Authz | 5 | 5 | 0 | 0 |
| Business Logic | 5 | 4 | 1 | 0 |
| **Total** | **52** | **39** | **10** | **3** |

---

*All P0 test cases must pass before MVP release. P1 test cases must pass before Post-MVP (P1 feature) release.*
