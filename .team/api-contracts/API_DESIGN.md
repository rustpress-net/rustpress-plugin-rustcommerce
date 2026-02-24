# RustCommerce REST API Design Contract

**Version**: 1.0.0
**Base URL**: `/api/v1/rustcommerce/`
**Date**: 2026-02-24
**Status**: Approved

---

## Table of Contents

1. [Conventions](#1-conventions)
2. [Products](#2-products)
3. [Categories](#3-categories)
4. [Cart](#4-cart)
5. [Checkout](#5-checkout)
6. [Orders](#6-orders)
7. [Customers](#7-customers)
8. [Payments](#8-payments)
9. [Shipping](#9-shipping)
10. [Tax](#10-tax)
11. [Coupons](#11-coupons)
12. [Reviews](#12-reviews)
13. [Admin](#13-admin)
14. [Analytics](#14-analytics)
15. [Webhooks](#15-webhooks)
16. [Inventory](#16-inventory)

---

## 1. Conventions

### 1.1 Authentication

All requests use RustPress JWT authentication via the `Authorization` header:

```
Authorization: Bearer <jwt_token>
```

- **Public endpoints**: No authentication required (product listings, categories).
- **Customer endpoints**: Require a valid JWT for an authenticated user.
- **Admin endpoints**: Require a valid JWT with the appropriate `manage_*` permission.
- **Guest cart**: Uses a `X-Session-ID` header (UUID v4) for anonymous cart tracking.

### 1.2 Pagination (Cursor-Based)

All list endpoints use cursor-based pagination for consistent, performant paging:

**Request Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `cursor` | `string` | `null` | Opaque cursor from previous response. Omit for first page. |
| `limit` | `integer` | `20` | Number of items per page. Max `100`. |
| `sort` | `string` | `created_at` | Field to sort by (varies per endpoint). |
| `order` | `string` | `desc` | Sort direction: `asc` or `desc`. |

**Response Envelope:**

```json
{
  "data": [ ... ],
  "pagination": {
    "cursor": "eyJpZCI6IjAxOTVmYTk...",
    "has_more": true,
    "total_count": 342
  }
}
```

- `cursor`: Pass this value as the `cursor` query parameter to get the next page. `null` when there are no more results.
- `has_more`: `true` if there are additional pages.
- `total_count`: Total number of matching records (may be omitted on expensive queries; returned when `include_count=true`).

**Cursor encoding**: Base64-encoded JSON containing the last record's sort field value and ID, e.g. `base64({"created_at":"2026-02-24T10:00:00Z","id":"019..."})`.

### 1.3 Filtering

List endpoints support filtering via query parameters:

```
GET /api/v1/rustcommerce/products?status=published&category_id=<uuid>&min_price=10.00&max_price=100.00
```

Filters are combined with `AND` logic. Multiple values for the same field use comma separation:

```
GET /api/v1/rustcommerce/products?status=published,draft
```

### 1.4 Search

Text search uses the `q` query parameter:

```
GET /api/v1/rustcommerce/products?q=wireless+headphones
```

Search is performed against `name`, `description`, and `sku` fields using PostgreSQL full-text search (`to_tsvector` / `to_tsquery`).

### 1.5 Field Selection

Use `fields` query parameter to request only specific fields:

```
GET /api/v1/rustcommerce/products?fields=id,name,price,slug
```

### 1.6 Error Response Format

All errors follow the RustPress error convention:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input data",
    "status": 400,
    "details": [
      {
        "field": "price",
        "message": "Price must be greater than 0",
        "code": "INVALID_VALUE"
      }
    ],
    "request_id": "req_01HXYZ..."
  }
}
```

**Error Codes:**

| HTTP Status | Code | Description |
|-------------|------|-------------|
| 400 | `VALIDATION_ERROR` | Invalid request body or parameters |
| 400 | `INVALID_OPERATION` | Business rule violation |
| 401 | `AUTHENTICATION_REQUIRED` | Missing or invalid JWT |
| 403 | `PERMISSION_DENIED` | Valid JWT but insufficient permissions |
| 404 | `NOT_FOUND` | Resource does not exist |
| 409 | `CONFLICT` | Duplicate resource (e.g., duplicate SKU) |
| 422 | `UNPROCESSABLE_ENTITY` | Semantically invalid (e.g., out of stock) |
| 429 | `RATE_LIMIT_EXCEEDED` | Too many requests |
| 500 | `INTERNAL_ERROR` | Server error |
| 502 | `GATEWAY_ERROR` | External service error (e.g., Stripe) |
| 503 | `SERVICE_UNAVAILABLE` | System temporarily unavailable |

### 1.7 Rate Limits

| Endpoint Group | Authenticated | Anonymous |
|----------------|---------------|-----------|
| Product listings (GET) | 120/min | 60/min |
| Cart operations | 60/min | 30/min |
| Checkout/Payment | 10/min | 5/min |
| Admin operations | 120/min | N/A |
| Webhooks | Unlimited (verified) | N/A |
| Analytics | 30/min | N/A |

Rate limit headers are included in all responses:

```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 55
X-RateLimit-Reset: 1708776000
```

### 1.8 Common HTTP Headers

**Request:**

| Header | Required | Description |
|--------|----------|-------------|
| `Authorization` | Conditional | `Bearer <jwt>` for authenticated endpoints |
| `Content-Type` | POST/PUT | `application/json` |
| `X-Session-ID` | Conditional | UUID for guest cart tracking |
| `X-Idempotency-Key` | Recommended | UUID for POST requests (checkout, payment) to prevent duplicates |
| `X-Request-ID` | Optional | Client-generated request ID for tracing |

**Response:**

| Header | Description |
|--------|-------------|
| `X-Request-ID` | Server-generated or echoed request ID |
| `X-RateLimit-Limit` | Rate limit ceiling |
| `X-RateLimit-Remaining` | Remaining requests in window |
| `X-RateLimit-Reset` | Unix timestamp when window resets |

### 1.9 Money Representation

All monetary values are represented as strings with exactly two decimal places to avoid floating-point precision issues:

```json
{
  "price": "29.99",
  "tax_total": "2.40",
  "grand_total": "32.39"
}
```

Backend stores as `DECIMAL(10,2)`. API serializes as string. Clients parse as decimal.

---

## 2. Products

### 2.1 List Products (Public)

```
GET /api/v1/rustcommerce/products
```

**Description**: Retrieve a paginated list of published products. Public endpoint for storefront.

**Auth**: None required
**Rate Limit**: 60/min (anonymous), 120/min (authenticated)

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `cursor` | string | null | Pagination cursor |
| `limit` | integer | 20 | Items per page (max 100) |
| `sort` | string | `created_at` | Sort field: `created_at`, `price`, `name`, `popularity`, `rating` |
| `order` | string | `desc` | Sort direction: `asc`, `desc` |
| `q` | string | null | Full-text search query |
| `category_id` | uuid | null | Filter by category |
| `category_slug` | string | null | Filter by category slug |
| `status` | string | `published` | Product status (public only sees `published`) |
| `min_price` | string | null | Minimum price filter |
| `max_price` | string | null | Maximum price filter |
| `featured` | boolean | null | Only featured products |
| `in_stock` | boolean | null | Only in-stock products |
| `tag` | string | null | Filter by tag (comma-separated) |
| `product_type` | string | null | `simple`, `variable`, `grouped`, `digital` |
| `fields` | string | null | Comma-separated field names |

**Response** `200 OK`:

```json
{
  "data": [
    {
      "id": "0195fa9a-1234-7def-8abc-123456789012",
      "name": "Wireless Bluetooth Headphones",
      "slug": "wireless-bluetooth-headphones",
      "short_description": "Premium noise-cancelling wireless headphones",
      "price": "79.99",
      "compare_at_price": "129.99",
      "sku": "WBH-001",
      "status": "published",
      "product_type": "variable",
      "featured": true,
      "stock_status": "in_stock",
      "stock_quantity": 150,
      "images": [
        {
          "id": "0195fa9b-5678-7def-8abc-123456789012",
          "url": "/media/products/headphones-main.jpg",
          "alt_text": "Wireless Bluetooth Headphones - Black",
          "is_primary": true
        }
      ],
      "categories": [
        {
          "id": "0195fa9c-abcd-7def-8abc-123456789012",
          "name": "Electronics",
          "slug": "electronics"
        }
      ],
      "variants_count": 3,
      "average_rating": 4.5,
      "review_count": 23,
      "created_at": "2026-02-20T10:00:00Z",
      "updated_at": "2026-02-23T15:30:00Z"
    }
  ],
  "pagination": {
    "cursor": "eyJjcmVhdGVkX2F0IjoiMjAyNi0wMi0yMFQxMDowMDowMFoiLCJpZCI6IjAxOTVmYTlhLTEyMzQtN2RlZi04YWJjLTEyMzQ1Njc4OTAxMiJ9",
    "has_more": true,
    "total_count": 342
  }
}
```

### 2.2 Get Product (Public)

```
GET /api/v1/rustcommerce/products/:id
```

**Description**: Retrieve a single product by ID or slug. Includes full details, variants, images, and related products.

**Auth**: None required
**Rate Limit**: 60/min (anonymous), 120/min (authenticated)

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | uuid or string | Product UUID or slug |

**Response** `200 OK`:

```json
{
  "data": {
    "id": "0195fa9a-1234-7def-8abc-123456789012",
    "name": "Wireless Bluetooth Headphones",
    "slug": "wireless-bluetooth-headphones",
    "description": "<p>Premium noise-cancelling wireless headphones with 30-hour battery life...</p>",
    "short_description": "Premium noise-cancelling wireless headphones",
    "sku": "WBH-001",
    "price": "79.99",
    "compare_at_price": "129.99",
    "cost_price": null,
    "status": "published",
    "product_type": "variable",
    "featured": true,
    "stock_quantity": 150,
    "stock_status": "in_stock",
    "low_stock_threshold": 10,
    "weight": "0.35",
    "dimensions": {
      "length": "20.00",
      "width": "18.00",
      "height": "8.00"
    },
    "tax_class": "standard",
    "images": [
      {
        "id": "0195fa9b-5678-7def-8abc-123456789012",
        "url": "/media/products/headphones-main.jpg",
        "alt_text": "Wireless Bluetooth Headphones - Black",
        "position": 0,
        "is_primary": true
      },
      {
        "id": "0195fa9b-9abc-7def-8abc-123456789012",
        "url": "/media/products/headphones-side.jpg",
        "alt_text": "Wireless Bluetooth Headphones - Side View",
        "position": 1,
        "is_primary": false
      }
    ],
    "categories": [
      {
        "id": "0195fa9c-abcd-7def-8abc-123456789012",
        "name": "Electronics",
        "slug": "electronics"
      },
      {
        "id": "0195fa9c-cdef-7def-8abc-123456789012",
        "name": "Audio",
        "slug": "audio"
      }
    ],
    "variants": [
      {
        "id": "0195fa9d-1111-7def-8abc-123456789012",
        "sku": "WBH-001-BLK",
        "name": "Black",
        "price": "79.99",
        "compare_at_price": "129.99",
        "stock_quantity": 75,
        "stock_status": "in_stock",
        "attributes": {
          "color": "Black"
        },
        "image_url": "/media/products/headphones-black.jpg",
        "position": 0
      },
      {
        "id": "0195fa9d-2222-7def-8abc-123456789012",
        "sku": "WBH-001-WHT",
        "name": "White",
        "price": "79.99",
        "compare_at_price": "129.99",
        "stock_quantity": 50,
        "stock_status": "in_stock",
        "attributes": {
          "color": "White"
        },
        "image_url": "/media/products/headphones-white.jpg",
        "position": 1
      },
      {
        "id": "0195fa9d-3333-7def-8abc-123456789012",
        "sku": "WBH-001-RED",
        "name": "Red",
        "price": "84.99",
        "compare_at_price": "129.99",
        "stock_quantity": 25,
        "stock_status": "in_stock",
        "attributes": {
          "color": "Red"
        },
        "image_url": "/media/products/headphones-red.jpg",
        "position": 2
      }
    ],
    "average_rating": 4.5,
    "review_count": 23,
    "meta": {
      "seo_title": "Buy Wireless Bluetooth Headphones - Premium Audio",
      "seo_description": "Shop premium noise-cancelling wireless headphones...",
      "og_image": "/media/products/headphones-og.jpg"
    },
    "related_product_ids": [
      "0195fa9e-aaaa-7def-8abc-123456789012",
      "0195fa9e-bbbb-7def-8abc-123456789012"
    ],
    "created_at": "2026-02-20T10:00:00Z",
    "updated_at": "2026-02-23T15:30:00Z"
  }
}
```

**Error** `404 Not Found`:

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Product not found",
    "status": 404,
    "request_id": "req_01HXYZ123"
  }
}
```

### 2.3 Create Product (Admin)

```
POST /api/v1/rustcommerce/admin/products
```

**Description**: Create a new product. Supports simple and variable product types.

**Auth**: Required. Permission: `manage_products`
**Rate Limit**: 120/min

**Request Body:**

```json
{
  "name": "Wireless Bluetooth Headphones",
  "slug": "wireless-bluetooth-headphones",
  "description": "<p>Premium noise-cancelling wireless headphones...</p>",
  "short_description": "Premium noise-cancelling wireless headphones",
  "sku": "WBH-001",
  "price": "79.99",
  "compare_at_price": "129.99",
  "cost_price": "35.00",
  "status": "draft",
  "product_type": "variable",
  "featured": false,
  "stock_quantity": 150,
  "stock_status": "in_stock",
  "low_stock_threshold": 10,
  "weight": "0.35",
  "dimensions": {
    "length": "20.00",
    "width": "18.00",
    "height": "8.00"
  },
  "tax_class": "standard",
  "category_ids": [
    "0195fa9c-abcd-7def-8abc-123456789012",
    "0195fa9c-cdef-7def-8abc-123456789012"
  ],
  "images": [
    {
      "url": "/media/products/headphones-main.jpg",
      "alt_text": "Wireless Bluetooth Headphones - Black",
      "position": 0,
      "is_primary": true
    }
  ],
  "variants": [
    {
      "sku": "WBH-001-BLK",
      "name": "Black",
      "price": "79.99",
      "stock_quantity": 75,
      "attributes": { "color": "Black" },
      "image_url": "/media/products/headphones-black.jpg",
      "position": 0
    }
  ],
  "meta": {
    "seo_title": "Buy Wireless Bluetooth Headphones",
    "seo_description": "Shop premium noise-cancelling wireless headphones..."
  }
}
```

**Request Body JSON Schema:**

```json
{
  "type": "object",
  "required": ["name", "price"],
  "properties": {
    "name": { "type": "string", "minLength": 1, "maxLength": 255 },
    "slug": { "type": "string", "maxLength": 255, "pattern": "^[a-z0-9]+(?:-[a-z0-9]+)*$" },
    "description": { "type": "string" },
    "short_description": { "type": "string" },
    "sku": { "type": "string", "maxLength": 100 },
    "price": { "type": "string", "pattern": "^\\d+\\.\\d{2}$" },
    "compare_at_price": { "type": ["string", "null"], "pattern": "^\\d+\\.\\d{2}$" },
    "cost_price": { "type": ["string", "null"], "pattern": "^\\d+\\.\\d{2}$" },
    "status": { "type": "string", "enum": ["draft", "published", "archived"] },
    "product_type": { "type": "string", "enum": ["simple", "variable", "grouped", "digital"] },
    "featured": { "type": "boolean" },
    "stock_quantity": { "type": "integer", "minimum": 0 },
    "stock_status": { "type": "string", "enum": ["in_stock", "out_of_stock", "on_backorder"] },
    "low_stock_threshold": { "type": "integer", "minimum": 0 },
    "weight": { "type": ["string", "null"] },
    "dimensions": {
      "type": ["object", "null"],
      "properties": {
        "length": { "type": "string" },
        "width": { "type": "string" },
        "height": { "type": "string" }
      }
    },
    "tax_class": { "type": "string" },
    "category_ids": { "type": "array", "items": { "type": "string", "format": "uuid" } },
    "images": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["url"],
        "properties": {
          "url": { "type": "string" },
          "alt_text": { "type": "string", "maxLength": 255 },
          "position": { "type": "integer" },
          "is_primary": { "type": "boolean" }
        }
      }
    },
    "variants": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "price"],
        "properties": {
          "sku": { "type": "string", "maxLength": 100 },
          "name": { "type": "string", "maxLength": 255 },
          "price": { "type": "string", "pattern": "^\\d+\\.\\d{2}$" },
          "compare_at_price": { "type": ["string", "null"] },
          "stock_quantity": { "type": "integer", "minimum": 0 },
          "attributes": { "type": "object" },
          "image_url": { "type": ["string", "null"] },
          "position": { "type": "integer" }
        }
      }
    },
    "meta": { "type": "object" }
  }
}
```

**Response** `201 Created`:

Returns the full product object (same schema as GET product detail).

**Errors:**

- `400 VALIDATION_ERROR` - Missing required fields or invalid values
- `409 CONFLICT` - Duplicate SKU or slug

### 2.4 Update Product (Admin)

```
PUT /api/v1/rustcommerce/admin/products/:id
```

**Description**: Update an existing product. Supports partial updates - only include fields that need changing.

**Auth**: Required. Permission: `manage_products`
**Rate Limit**: 120/min

**Request Body:** Same schema as Create Product, but all fields are optional. Only provided fields are updated.

```json
{
  "price": "89.99",
  "status": "published",
  "stock_quantity": 200
}
```

**Response** `200 OK`: Returns the updated full product object.

**Errors:**

- `404 NOT_FOUND` - Product does not exist
- `409 CONFLICT` - Duplicate SKU or slug

### 2.5 Delete Product (Admin)

```
DELETE /api/v1/rustcommerce/admin/products/:id
```

**Description**: Soft-delete a product by setting status to `archived`. Products with existing orders are never hard-deleted.

**Auth**: Required. Permission: `manage_products`
**Rate Limit**: 120/min

**Response** `200 OK`:

```json
{
  "data": {
    "id": "0195fa9a-1234-7def-8abc-123456789012",
    "deleted": true,
    "message": "Product archived successfully"
  }
}
```

### 2.6 Bulk Product Operations (Admin)

```
POST /api/v1/rustcommerce/admin/products/bulk
```

**Description**: Perform bulk operations on multiple products.

**Auth**: Required. Permission: `manage_products`
**Rate Limit**: 30/min

**Request Body:**

```json
{
  "action": "update_status",
  "product_ids": [
    "0195fa9a-1111-7def-8abc-123456789012",
    "0195fa9a-2222-7def-8abc-123456789012"
  ],
  "data": {
    "status": "published"
  }
}
```

Supported actions: `update_status`, `update_category`, `update_price`, `delete`

**Response** `200 OK`:

```json
{
  "data": {
    "total": 2,
    "succeeded": 2,
    "failed": 0,
    "results": [
      { "id": "0195fa9a-1111-...", "success": true },
      { "id": "0195fa9a-2222-...", "success": true }
    ]
  }
}
```

---

## 3. Categories

### 3.1 List Categories (Public)

```
GET /api/v1/rustcommerce/categories
```

**Description**: Retrieve all product categories as a flat list or tree structure.

**Auth**: None required
**Rate Limit**: 60/min

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `tree` | boolean | `false` | Return as nested tree structure |
| `parent_id` | uuid | null | Filter by parent category |
| `include_empty` | boolean | `false` | Include categories with zero products |

**Response** `200 OK` (flat):

```json
{
  "data": [
    {
      "id": "0195fa9c-abcd-7def-8abc-123456789012",
      "name": "Electronics",
      "slug": "electronics",
      "description": "Electronic devices and accessories",
      "parent_id": null,
      "image_url": "/media/categories/electronics.jpg",
      "position": 0,
      "product_count": 45,
      "created_at": "2026-02-15T08:00:00Z"
    },
    {
      "id": "0195fa9c-cdef-7def-8abc-123456789012",
      "name": "Audio",
      "slug": "audio",
      "description": "Headphones, speakers, and audio equipment",
      "parent_id": "0195fa9c-abcd-7def-8abc-123456789012",
      "image_url": "/media/categories/audio.jpg",
      "position": 0,
      "product_count": 23,
      "created_at": "2026-02-15T08:05:00Z"
    }
  ]
}
```

**Response** `200 OK` (tree, `?tree=true`):

```json
{
  "data": [
    {
      "id": "0195fa9c-abcd-7def-8abc-123456789012",
      "name": "Electronics",
      "slug": "electronics",
      "description": "Electronic devices and accessories",
      "parent_id": null,
      "image_url": "/media/categories/electronics.jpg",
      "position": 0,
      "product_count": 45,
      "children": [
        {
          "id": "0195fa9c-cdef-7def-8abc-123456789012",
          "name": "Audio",
          "slug": "audio",
          "description": "Headphones, speakers, and audio equipment",
          "parent_id": "0195fa9c-abcd-7def-8abc-123456789012",
          "image_url": "/media/categories/audio.jpg",
          "position": 0,
          "product_count": 23,
          "children": []
        }
      ]
    }
  ]
}
```

### 3.2 Get Category (Public)

```
GET /api/v1/rustcommerce/categories/:id
```

**Description**: Retrieve a single category by ID or slug.

**Auth**: None required
**Rate Limit**: 60/min

**Response** `200 OK`:

```json
{
  "data": {
    "id": "0195fa9c-abcd-7def-8abc-123456789012",
    "name": "Electronics",
    "slug": "electronics",
    "description": "Electronic devices and accessories",
    "parent_id": null,
    "image_url": "/media/categories/electronics.jpg",
    "position": 0,
    "product_count": 45,
    "breadcrumb": [
      { "id": null, "name": "All Products", "slug": "" }
    ],
    "children": [
      {
        "id": "0195fa9c-cdef-7def-8abc-123456789012",
        "name": "Audio",
        "slug": "audio",
        "product_count": 23
      }
    ],
    "created_at": "2026-02-15T08:00:00Z",
    "updated_at": "2026-02-15T08:00:00Z"
  }
}
```

### 3.3 Create Category (Admin)

```
POST /api/v1/rustcommerce/admin/categories
```

**Description**: Create a new product category.

**Auth**: Required. Permission: `manage_products`
**Rate Limit**: 120/min

**Request Body:**

```json
{
  "name": "Audio",
  "slug": "audio",
  "description": "Headphones, speakers, and audio equipment",
  "parent_id": "0195fa9c-abcd-7def-8abc-123456789012",
  "image_url": "/media/categories/audio.jpg",
  "position": 0
}
```

**Request Body JSON Schema:**

```json
{
  "type": "object",
  "required": ["name"],
  "properties": {
    "name": { "type": "string", "minLength": 1, "maxLength": 255 },
    "slug": { "type": "string", "maxLength": 255, "pattern": "^[a-z0-9]+(?:-[a-z0-9]+)*$" },
    "description": { "type": "string" },
    "parent_id": { "type": ["string", "null"], "format": "uuid" },
    "image_url": { "type": ["string", "null"] },
    "position": { "type": "integer", "minimum": 0 }
  }
}
```

**Response** `201 Created`: Returns the created category.

### 3.4 Update Category (Admin)

```
PUT /api/v1/rustcommerce/admin/categories/:id
```

**Auth**: Required. Permission: `manage_products`
**Rate Limit**: 120/min

**Request Body:** Same as create, all fields optional.

**Response** `200 OK`: Returns the updated category.

### 3.5 Delete Category (Admin)

```
DELETE /api/v1/rustcommerce/admin/categories/:id
```

**Description**: Delete a category. Products in this category are uncategorized but not deleted. Child categories are re-parented to the deleted category's parent.

**Auth**: Required. Permission: `manage_products`
**Rate Limit**: 120/min

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `reassign_to` | uuid | null | Move products to this category instead of uncategorizing |

**Response** `200 OK`:

```json
{
  "data": {
    "id": "0195fa9c-cdef-7def-8abc-123456789012",
    "deleted": true,
    "products_reassigned": 23,
    "children_reparented": 2
  }
}
```

---

## 4. Cart

### 4.1 Get Cart

```
GET /api/v1/rustcommerce/cart
```

**Description**: Retrieve the current user's cart with calculated totals. For guests, requires `X-Session-ID` header.

**Auth**: Optional (authenticated user or guest with session)
**Rate Limit**: 60/min

**Headers:**

| Header | Required | Description |
|--------|----------|-------------|
| `Authorization` | No | JWT for logged-in user cart |
| `X-Session-ID` | No | Session UUID for guest cart |

At least one of `Authorization` or `X-Session-ID` must be provided.

**Response** `200 OK`:

```json
{
  "data": {
    "id": "0195fa9f-1234-7def-8abc-123456789012",
    "user_id": "0195fa9f-aaaa-7def-8abc-123456789012",
    "status": "active",
    "items": [
      {
        "id": "0195faa0-1111-7def-8abc-123456789012",
        "product_id": "0195fa9a-1234-7def-8abc-123456789012",
        "variant_id": "0195fa9d-1111-7def-8abc-123456789012",
        "product_name": "Wireless Bluetooth Headphones",
        "variant_name": "Black",
        "sku": "WBH-001-BLK",
        "image_url": "/media/products/headphones-black.jpg",
        "quantity": 2,
        "unit_price": "79.99",
        "line_total": "159.98",
        "stock_status": "in_stock",
        "stock_quantity": 75
      }
    ],
    "item_count": 2,
    "unique_item_count": 1,
    "subtotal": "159.98",
    "tax_estimate": "12.80",
    "shipping_estimate": "5.99",
    "discount_total": "0.00",
    "grand_total_estimate": "178.77",
    "coupon_code": null,
    "currency": "USD",
    "created_at": "2026-02-24T10:00:00Z",
    "updated_at": "2026-02-24T10:30:00Z",
    "expires_at": "2026-02-25T10:00:00Z"
  }
}
```

### 4.2 Add Item to Cart

```
POST /api/v1/rustcommerce/cart/items
```

**Description**: Add a product (or variant) to the cart. If the item already exists, increments the quantity.

**Auth**: Optional (authenticated or guest with `X-Session-ID`)
**Rate Limit**: 60/min

**Request Body:**

```json
{
  "product_id": "0195fa9a-1234-7def-8abc-123456789012",
  "variant_id": "0195fa9d-1111-7def-8abc-123456789012",
  "quantity": 1
}
```

**Request Body JSON Schema:**

```json
{
  "type": "object",
  "required": ["product_id", "quantity"],
  "properties": {
    "product_id": { "type": "string", "format": "uuid" },
    "variant_id": { "type": ["string", "null"], "format": "uuid" },
    "quantity": { "type": "integer", "minimum": 1, "maximum": 999 }
  }
}
```

**Response** `200 OK`: Returns the updated full cart object (same as GET cart).

**Errors:**

- `404 NOT_FOUND` - Product or variant does not exist
- `422 UNPROCESSABLE_ENTITY` - Insufficient stock

```json
{
  "error": {
    "code": "UNPROCESSABLE_ENTITY",
    "message": "Insufficient stock",
    "status": 422,
    "details": [
      {
        "field": "quantity",
        "message": "Only 5 units available for this variant",
        "code": "INSUFFICIENT_STOCK",
        "available_quantity": 5
      }
    ]
  }
}
```

### 4.3 Update Cart Item

```
PUT /api/v1/rustcommerce/cart/items/:item_id
```

**Description**: Update the quantity of an item in the cart.

**Auth**: Optional (authenticated or guest with `X-Session-ID`)
**Rate Limit**: 60/min

**Request Body:**

```json
{
  "quantity": 3
}
```

**Response** `200 OK`: Returns the updated full cart object.

**Errors:**

- `404 NOT_FOUND` - Cart item does not exist
- `422 UNPROCESSABLE_ENTITY` - Insufficient stock

### 4.4 Remove Cart Item

```
DELETE /api/v1/rustcommerce/cart/items/:item_id
```

**Description**: Remove an item from the cart.

**Auth**: Optional (authenticated or guest with `X-Session-ID`)
**Rate Limit**: 60/min

**Response** `200 OK`: Returns the updated full cart object.

### 4.5 Clear Cart

```
DELETE /api/v1/rustcommerce/cart
```

**Description**: Remove all items from the cart.

**Auth**: Optional (authenticated or guest with `X-Session-ID`)
**Rate Limit**: 60/min

**Response** `200 OK`:

```json
{
  "data": {
    "id": "0195fa9f-1234-7def-8abc-123456789012",
    "items": [],
    "item_count": 0,
    "subtotal": "0.00",
    "grand_total_estimate": "0.00"
  }
}
```

### 4.6 Apply Coupon to Cart

```
POST /api/v1/rustcommerce/cart/coupon
```

**Description**: Apply a coupon code to the cart.

**Auth**: Optional (authenticated or guest with `X-Session-ID`)
**Rate Limit**: 30/min

**Request Body:**

```json
{
  "coupon_code": "SAVE20"
}
```

**Response** `200 OK`: Returns the updated full cart object with discount applied.

**Errors:**

- `400 INVALID_OPERATION` - Coupon not found, expired, or not applicable

```json
{
  "error": {
    "code": "INVALID_OPERATION",
    "message": "Coupon code is not valid",
    "status": 400,
    "details": [
      {
        "field": "coupon_code",
        "message": "Coupon 'SAVE20' has expired",
        "code": "COUPON_EXPIRED"
      }
    ]
  }
}
```

### 4.7 Remove Coupon from Cart

```
DELETE /api/v1/rustcommerce/cart/coupon
```

**Description**: Remove the applied coupon from the cart.

**Auth**: Optional (authenticated or guest with `X-Session-ID`)
**Rate Limit**: 60/min

**Response** `200 OK`: Returns the updated full cart object without discount.

---

## 5. Checkout

### 5.1 Initialize Checkout

```
POST /api/v1/rustcommerce/checkout/init
```

**Description**: Initialize the checkout process. Validates the cart, reserves stock for 10 minutes, and returns a checkout session.

**Auth**: Optional (supports guest checkout if enabled)
**Rate Limit**: 10/min

**Headers:**

| Header | Recommended | Description |
|--------|-------------|-------------|
| `X-Idempotency-Key` | Yes | UUID to prevent duplicate checkouts |

**Request Body:**

```json
{
  "email": "customer@example.com"
}
```

Only required for guest checkout. For authenticated users, email is taken from the JWT.

**Response** `200 OK`:

```json
{
  "data": {
    "checkout_session_id": "0195fab0-1234-7def-8abc-123456789012",
    "cart_id": "0195fa9f-1234-7def-8abc-123456789012",
    "email": "customer@example.com",
    "items": [
      {
        "product_id": "0195fa9a-1234-7def-8abc-123456789012",
        "variant_id": "0195fa9d-1111-7def-8abc-123456789012",
        "product_name": "Wireless Bluetooth Headphones",
        "variant_name": "Black",
        "quantity": 2,
        "unit_price": "79.99",
        "line_total": "159.98"
      }
    ],
    "subtotal": "159.98",
    "stock_reserved": true,
    "stock_reservation_expires_at": "2026-02-24T10:40:00Z",
    "requires_shipping": true,
    "steps_remaining": ["shipping_address", "shipping_method", "payment"],
    "created_at": "2026-02-24T10:30:00Z"
  }
}
```

**Errors:**

- `422 UNPROCESSABLE_ENTITY` - Cart is empty or items are out of stock

### 5.2 Set Shipping Address

```
POST /api/v1/rustcommerce/checkout/shipping-address
```

**Description**: Set the shipping address for the checkout session. Also accepts a billing address or flags billing same as shipping.

**Auth**: Optional (guest or authenticated)
**Rate Limit**: 10/min

**Request Body:**

```json
{
  "checkout_session_id": "0195fab0-1234-7def-8abc-123456789012",
  "shipping_address": {
    "first_name": "Jane",
    "last_name": "Doe",
    "company": null,
    "address_line_1": "123 Main Street",
    "address_line_2": "Apt 4B",
    "city": "New York",
    "state": "NY",
    "postal_code": "10001",
    "country": "US",
    "phone": "+1-555-123-4567"
  },
  "billing_same_as_shipping": true,
  "billing_address": null
}
```

**Address JSON Schema:**

```json
{
  "type": "object",
  "required": ["first_name", "last_name", "address_line_1", "city", "postal_code", "country"],
  "properties": {
    "first_name": { "type": "string", "minLength": 1, "maxLength": 100 },
    "last_name": { "type": "string", "minLength": 1, "maxLength": 100 },
    "company": { "type": ["string", "null"], "maxLength": 255 },
    "address_line_1": { "type": "string", "minLength": 1, "maxLength": 255 },
    "address_line_2": { "type": ["string", "null"], "maxLength": 255 },
    "city": { "type": "string", "minLength": 1, "maxLength": 100 },
    "state": { "type": ["string", "null"], "maxLength": 100 },
    "postal_code": { "type": "string", "minLength": 1, "maxLength": 20 },
    "country": { "type": "string", "minLength": 2, "maxLength": 2, "description": "ISO 3166-1 alpha-2" },
    "phone": { "type": ["string", "null"], "maxLength": 50 }
  }
}
```

**Response** `200 OK`:

```json
{
  "data": {
    "checkout_session_id": "0195fab0-1234-7def-8abc-123456789012",
    "shipping_address_set": true,
    "billing_address_set": true,
    "tax_total": "12.80",
    "steps_remaining": ["shipping_method", "payment"]
  }
}
```

### 5.3 Select Shipping Method

```
POST /api/v1/rustcommerce/checkout/shipping-method
```

**Description**: Select a shipping method for the checkout.

**Auth**: Optional (guest or authenticated)
**Rate Limit**: 10/min

**Request Body:**

```json
{
  "checkout_session_id": "0195fab0-1234-7def-8abc-123456789012",
  "shipping_method_id": "0195fab1-aaaa-7def-8abc-123456789012"
}
```

**Response** `200 OK`:

```json
{
  "data": {
    "checkout_session_id": "0195fab0-1234-7def-8abc-123456789012",
    "shipping_method": {
      "id": "0195fab1-aaaa-7def-8abc-123456789012",
      "name": "Standard Shipping",
      "cost": "5.99",
      "estimated_days": "5-7 business days"
    },
    "subtotal": "159.98",
    "tax_total": "12.80",
    "shipping_total": "5.99",
    "discount_total": "0.00",
    "grand_total": "178.77",
    "steps_remaining": ["payment"]
  }
}
```

### 5.4 Create Payment Intent

```
POST /api/v1/rustcommerce/checkout/payment-intent
```

**Description**: Create a Stripe PaymentIntent for the checkout. Returns the client secret for Stripe.js on the frontend.

**Auth**: Optional (guest or authenticated)
**Rate Limit**: 5/min

**Headers:**

| Header | Required | Description |
|--------|----------|-------------|
| `X-Idempotency-Key` | Strongly recommended | UUID to prevent duplicate payment intents |

**Request Body:**

```json
{
  "checkout_session_id": "0195fab0-1234-7def-8abc-123456789012",
  "payment_method": "stripe"
}
```

**Response** `200 OK`:

```json
{
  "data": {
    "checkout_session_id": "0195fab0-1234-7def-8abc-123456789012",
    "payment_intent_id": "pi_3PxYz123456789",
    "client_secret": "pi_3PxYz123456789_secret_AbCdEf",
    "amount": "178.77",
    "currency": "usd",
    "status": "requires_payment_method"
  }
}
```

### 5.5 Complete Checkout

```
POST /api/v1/rustcommerce/checkout/complete
```

**Description**: Finalize the checkout after payment confirmation. Creates the order, decrements inventory, clears cart, and triggers notifications.

**Auth**: Optional (guest or authenticated)
**Rate Limit**: 5/min

**Request Body:**

```json
{
  "checkout_session_id": "0195fab0-1234-7def-8abc-123456789012",
  "payment_intent_id": "pi_3PxYz123456789",
  "customer_note": "Please leave at the front door"
}
```

**Response** `201 Created`:

```json
{
  "data": {
    "order_id": "0195fab2-1234-7def-8abc-123456789012",
    "order_number": "RC-00042",
    "status": "confirmed",
    "payment_status": "paid",
    "subtotal": "159.98",
    "tax_total": "12.80",
    "shipping_total": "5.99",
    "discount_total": "0.00",
    "grand_total": "178.77",
    "currency": "USD",
    "items": [
      {
        "product_name": "Wireless Bluetooth Headphones",
        "variant_name": "Black",
        "sku": "WBH-001-BLK",
        "quantity": 2,
        "unit_price": "79.99",
        "total": "159.98"
      }
    ],
    "shipping_address": {
      "first_name": "Jane",
      "last_name": "Doe",
      "address_line_1": "123 Main Street",
      "city": "New York",
      "state": "NY",
      "postal_code": "10001",
      "country": "US"
    },
    "shipping_method": "Standard Shipping",
    "created_at": "2026-02-24T10:35:00Z"
  }
}
```

**Errors:**

- `400 INVALID_OPERATION` - Payment not confirmed
- `422 UNPROCESSABLE_ENTITY` - Stock reservation expired

---

## 6. Orders

### 6.1 List My Orders (Customer)

```
GET /api/v1/rustcommerce/orders
```

**Description**: Retrieve the authenticated customer's order history.

**Auth**: Required (customer JWT)
**Rate Limit**: 60/min

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `cursor` | string | null | Pagination cursor |
| `limit` | integer | 20 | Items per page |
| `status` | string | null | Filter by status |
| `sort` | string | `created_at` | Sort field |
| `order` | string | `desc` | Sort direction |

**Response** `200 OK`:

```json
{
  "data": [
    {
      "id": "0195fab2-1234-7def-8abc-123456789012",
      "order_number": "RC-00042",
      "status": "shipped",
      "payment_status": "paid",
      "grand_total": "178.77",
      "currency": "USD",
      "item_count": 2,
      "items_summary": [
        {
          "product_name": "Wireless Bluetooth Headphones",
          "variant_name": "Black",
          "quantity": 2,
          "image_url": "/media/products/headphones-black.jpg"
        }
      ],
      "created_at": "2026-02-24T10:35:00Z",
      "updated_at": "2026-02-24T14:00:00Z"
    }
  ],
  "pagination": {
    "cursor": "eyJjcmVhdGVkX2F0Ij...",
    "has_more": false,
    "total_count": 5
  }
}
```

### 6.2 Get My Order (Customer)

```
GET /api/v1/rustcommerce/orders/:id
```

**Description**: Retrieve a single order belonging to the authenticated customer.

**Auth**: Required (customer JWT, must own the order)
**Rate Limit**: 60/min

**Response** `200 OK`:

```json
{
  "data": {
    "id": "0195fab2-1234-7def-8abc-123456789012",
    "order_number": "RC-00042",
    "status": "shipped",
    "payment_status": "paid",
    "subtotal": "159.98",
    "tax_total": "12.80",
    "shipping_total": "5.99",
    "discount_total": "0.00",
    "grand_total": "178.77",
    "currency": "USD",
    "items": [
      {
        "id": "0195fab3-1111-7def-8abc-123456789012",
        "product_id": "0195fa9a-1234-7def-8abc-123456789012",
        "variant_id": "0195fa9d-1111-7def-8abc-123456789012",
        "product_name": "Wireless Bluetooth Headphones",
        "variant_name": "Black",
        "sku": "WBH-001-BLK",
        "quantity": 2,
        "unit_price": "79.99",
        "subtotal": "159.98",
        "tax_amount": "12.80",
        "discount_amount": "0.00",
        "total": "172.78"
      }
    ],
    "billing_address": {
      "first_name": "Jane",
      "last_name": "Doe",
      "address_line_1": "123 Main Street",
      "address_line_2": "Apt 4B",
      "city": "New York",
      "state": "NY",
      "postal_code": "10001",
      "country": "US",
      "phone": "+1-555-123-4567"
    },
    "shipping_address": {
      "first_name": "Jane",
      "last_name": "Doe",
      "address_line_1": "123 Main Street",
      "address_line_2": "Apt 4B",
      "city": "New York",
      "state": "NY",
      "postal_code": "10001",
      "country": "US",
      "phone": "+1-555-123-4567"
    },
    "shipping_method": "Standard Shipping",
    "payment_method": "stripe",
    "stripe_payment_intent_id": "pi_3PxYz123456789",
    "coupon_code": null,
    "customer_note": "Please leave at the front door",
    "tracking_number": "1Z999AA10123456784",
    "tracking_url": "https://www.ups.com/track?tracknum=1Z999AA10123456784",
    "status_history": [
      { "status": "pending", "timestamp": "2026-02-24T10:35:00Z" },
      { "status": "confirmed", "timestamp": "2026-02-24T10:35:05Z" },
      { "status": "processing", "timestamp": "2026-02-24T12:00:00Z" },
      { "status": "shipped", "timestamp": "2026-02-24T14:00:00Z", "note": "Shipped via UPS Ground" }
    ],
    "created_at": "2026-02-24T10:35:00Z",
    "updated_at": "2026-02-24T14:00:00Z",
    "completed_at": null
  }
}
```

### 6.3 List All Orders (Admin)

```
GET /api/v1/rustcommerce/admin/orders
```

**Description**: Retrieve all orders with full admin filtering.

**Auth**: Required. Permission: `manage_orders`
**Rate Limit**: 120/min

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `cursor` | string | null | Pagination cursor |
| `limit` | integer | 20 | Items per page |
| `status` | string | null | Order status filter (comma-separated) |
| `payment_status` | string | null | Payment status filter |
| `customer_id` | uuid | null | Filter by customer |
| `date_from` | string | null | ISO 8601 date |
| `date_to` | string | null | ISO 8601 date |
| `min_total` | string | null | Minimum grand total |
| `max_total` | string | null | Maximum grand total |
| `q` | string | null | Search by order number, customer name, or email |
| `sort` | string | `created_at` | Sort field |
| `order` | string | `desc` | Sort direction |

**Response** `200 OK`: Same as customer order list but includes admin fields (`admin_note`, `ip_address`, `user_agent`, full customer info).

### 6.4 Get Order Detail (Admin)

```
GET /api/v1/rustcommerce/admin/orders/:id
```

**Auth**: Required. Permission: `manage_orders`
**Rate Limit**: 120/min

**Response** `200 OK`: Full order object with admin-only fields:

```json
{
  "data": {
    "...all customer-visible fields...",
    "admin_note": "Customer called about delivery time",
    "ip_address": "192.168.1.100",
    "user_agent": "Mozilla/5.0 ...",
    "customer": {
      "id": "0195fab4-aaaa-7def-8abc-123456789012",
      "email": "jane@example.com",
      "first_name": "Jane",
      "last_name": "Doe",
      "total_orders": 5,
      "total_spent": "892.45"
    },
    "payments": [
      {
        "id": "0195fab5-1111-7def-8abc-123456789012",
        "payment_method": "stripe",
        "status": "completed",
        "amount": "178.77",
        "currency": "USD",
        "transaction_id": "ch_3PxYz123456789",
        "created_at": "2026-02-24T10:35:05Z"
      }
    ],
    "refunds": []
  }
}
```

### 6.5 Update Order Status (Admin)

```
PUT /api/v1/rustcommerce/admin/orders/:id/status
```

**Description**: Update the status of an order. Enforces valid state transitions.

**Auth**: Required. Permission: `manage_orders`
**Rate Limit**: 120/min

**Request Body:**

```json
{
  "status": "shipped",
  "note": "Shipped via UPS Ground",
  "tracking_number": "1Z999AA10123456784",
  "tracking_url": "https://www.ups.com/track?tracknum=1Z999AA10123456784",
  "notify_customer": true
}
```

**Response** `200 OK`: Returns the updated order.

**Errors:**

- `400 INVALID_OPERATION` - Invalid status transition (see Business Logic doc for valid transitions)

```json
{
  "error": {
    "code": "INVALID_OPERATION",
    "message": "Cannot transition from 'delivered' to 'processing'",
    "status": 400,
    "details": [
      {
        "field": "status",
        "message": "Valid transitions from 'delivered' are: ['refunded']",
        "code": "INVALID_STATUS_TRANSITION"
      }
    ]
  }
}
```

### 6.6 Add Admin Note (Admin)

```
POST /api/v1/rustcommerce/admin/orders/:id/notes
```

**Auth**: Required. Permission: `manage_orders`
**Rate Limit**: 120/min

**Request Body:**

```json
{
  "note": "Customer called requesting expedited shipping",
  "is_customer_visible": false
}
```

**Response** `201 Created`:

```json
{
  "data": {
    "id": "0195fab6-1111-7def-8abc-123456789012",
    "order_id": "0195fab2-1234-7def-8abc-123456789012",
    "note": "Customer called requesting expedited shipping",
    "is_customer_visible": false,
    "author": "admin@example.com",
    "created_at": "2026-02-24T16:00:00Z"
  }
}
```

### 6.7 Refund Order (Admin)

```
POST /api/v1/rustcommerce/admin/orders/:id/refund
```

**Description**: Issue a full or partial refund via Stripe.

**Auth**: Required. Permission: `manage_orders`
**Rate Limit**: 10/min

**Request Body:**

```json
{
  "amount": "79.99",
  "reason": "Customer returned one item",
  "restock_items": [
    {
      "order_item_id": "0195fab3-1111-7def-8abc-123456789012",
      "quantity": 1
    }
  ],
  "notify_customer": true
}
```

Omit `amount` for a full refund.

**Response** `200 OK`:

```json
{
  "data": {
    "refund_id": "0195fab7-1111-7def-8abc-123456789012",
    "order_id": "0195fab2-1234-7def-8abc-123456789012",
    "amount": "79.99",
    "currency": "USD",
    "stripe_refund_id": "re_3PxYz987654321",
    "status": "completed",
    "reason": "Customer returned one item",
    "items_restocked": true,
    "order_payment_status": "partially_refunded",
    "created_at": "2026-02-24T16:30:00Z"
  }
}
```

---

## 7. Customers

### 7.1 Get My Profile (Customer)

```
GET /api/v1/rustcommerce/account
```

**Auth**: Required (customer JWT)
**Rate Limit**: 60/min

**Response** `200 OK`:

```json
{
  "data": {
    "id": "0195fab4-aaaa-7def-8abc-123456789012",
    "user_id": "0195fa9f-aaaa-7def-8abc-123456789012",
    "email": "jane@example.com",
    "first_name": "Jane",
    "last_name": "Doe",
    "phone": "+1-555-123-4567",
    "total_orders": 5,
    "total_spent": "892.45",
    "average_order_value": "178.49",
    "last_order_at": "2026-02-24T10:35:00Z",
    "addresses": [
      {
        "id": "0195fab8-1111-7def-8abc-123456789012",
        "address_type": "shipping",
        "is_default": true,
        "first_name": "Jane",
        "last_name": "Doe",
        "address_line_1": "123 Main Street",
        "address_line_2": "Apt 4B",
        "city": "New York",
        "state": "NY",
        "postal_code": "10001",
        "country": "US",
        "phone": "+1-555-123-4567"
      }
    ],
    "created_at": "2026-01-15T08:00:00Z",
    "updated_at": "2026-02-24T10:35:00Z"
  }
}
```

### 7.2 Update My Profile (Customer)

```
PUT /api/v1/rustcommerce/account
```

**Auth**: Required (customer JWT)
**Rate Limit**: 30/min

**Request Body:**

```json
{
  "first_name": "Jane",
  "last_name": "Doe",
  "phone": "+1-555-999-8888"
}
```

**Response** `200 OK`: Returns updated profile.

### 7.3 List My Addresses (Customer)

```
GET /api/v1/rustcommerce/account/addresses
```

**Auth**: Required (customer JWT)
**Rate Limit**: 60/min

**Response** `200 OK`:

```json
{
  "data": [
    {
      "id": "0195fab8-1111-7def-8abc-123456789012",
      "address_type": "shipping",
      "is_default": true,
      "first_name": "Jane",
      "last_name": "Doe",
      "company": null,
      "address_line_1": "123 Main Street",
      "address_line_2": "Apt 4B",
      "city": "New York",
      "state": "NY",
      "postal_code": "10001",
      "country": "US",
      "phone": "+1-555-123-4567"
    }
  ]
}
```

### 7.4 Add Address (Customer)

```
POST /api/v1/rustcommerce/account/addresses
```

**Auth**: Required (customer JWT)
**Rate Limit**: 30/min

**Request Body:** Uses the Address JSON Schema defined in Section 5.2, plus:

```json
{
  "address_type": "shipping",
  "is_default": true,
  "first_name": "Jane",
  "last_name": "Doe",
  "address_line_1": "456 Oak Avenue",
  "city": "Los Angeles",
  "state": "CA",
  "postal_code": "90001",
  "country": "US"
}
```

**Response** `201 Created`: Returns the created address.

### 7.5 Update Address (Customer)

```
PUT /api/v1/rustcommerce/account/addresses/:id
```

**Auth**: Required (customer JWT, must own the address)
**Rate Limit**: 30/min

**Request Body:** Same as add, all fields optional.

**Response** `200 OK`: Returns the updated address.

### 7.6 Delete Address (Customer)

```
DELETE /api/v1/rustcommerce/account/addresses/:id
```

**Auth**: Required (customer JWT, must own the address)
**Rate Limit**: 30/min

**Response** `200 OK`:

```json
{
  "data": {
    "id": "0195fab8-1111-7def-8abc-123456789012",
    "deleted": true
  }
}
```

### 7.7 List All Customers (Admin)

```
GET /api/v1/rustcommerce/admin/customers
```

**Auth**: Required. Permission: `manage_customers`
**Rate Limit**: 120/min

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `cursor` | string | null | Pagination cursor |
| `limit` | integer | 20 | Items per page |
| `q` | string | null | Search by name, email, phone |
| `sort` | string | `created_at` | Sort field: `created_at`, `total_spent`, `total_orders`, `last_order_at` |
| `order` | string | `desc` | Sort direction |
| `min_orders` | integer | null | Minimum order count |
| `min_spent` | string | null | Minimum total spent |

**Response** `200 OK`:

```json
{
  "data": [
    {
      "id": "0195fab4-aaaa-7def-8abc-123456789012",
      "email": "jane@example.com",
      "first_name": "Jane",
      "last_name": "Doe",
      "phone": "+1-555-123-4567",
      "total_orders": 5,
      "total_spent": "892.45",
      "average_order_value": "178.49",
      "last_order_at": "2026-02-24T10:35:00Z",
      "created_at": "2026-01-15T08:00:00Z"
    }
  ],
  "pagination": {
    "cursor": "...",
    "has_more": true,
    "total_count": 1250
  }
}
```

### 7.8 Get Customer Detail (Admin)

```
GET /api/v1/rustcommerce/admin/customers/:id
```

**Auth**: Required. Permission: `manage_customers`
**Rate Limit**: 120/min

**Response** `200 OK`: Full customer profile with addresses, recent orders, and notes.

```json
{
  "data": {
    "id": "0195fab4-aaaa-7def-8abc-123456789012",
    "user_id": "0195fa9f-aaaa-7def-8abc-123456789012",
    "email": "jane@example.com",
    "first_name": "Jane",
    "last_name": "Doe",
    "phone": "+1-555-123-4567",
    "total_orders": 5,
    "total_spent": "892.45",
    "average_order_value": "178.49",
    "last_order_at": "2026-02-24T10:35:00Z",
    "addresses": [ "...address objects..." ],
    "recent_orders": [
      {
        "id": "0195fab2-1234-...",
        "order_number": "RC-00042",
        "status": "shipped",
        "grand_total": "178.77",
        "created_at": "2026-02-24T10:35:00Z"
      }
    ],
    "notes": "VIP customer, always requests expedited shipping",
    "meta": {},
    "created_at": "2026-01-15T08:00:00Z",
    "updated_at": "2026-02-24T10:35:00Z"
  }
}
```

### 7.9 Update Customer (Admin)

```
PUT /api/v1/rustcommerce/admin/customers/:id
```

**Auth**: Required. Permission: `manage_customers`
**Rate Limit**: 120/min

**Request Body:**

```json
{
  "notes": "VIP customer, always requests expedited shipping",
  "meta": {
    "vip_tier": "gold"
  }
}
```

**Response** `200 OK`: Returns the updated customer.

### 7.10 Delete Customer (Admin)

```
DELETE /api/v1/rustcommerce/admin/customers/:id
```

**Description**: Anonymize customer data (GDPR compliance). Does not delete orders but removes PII.

**Auth**: Required. Permission: `manage_customers`
**Rate Limit**: 30/min

**Response** `200 OK`:

```json
{
  "data": {
    "id": "0195fab4-aaaa-7def-8abc-123456789012",
    "anonymized": true,
    "orders_preserved": 5,
    "message": "Customer data anonymized. Orders preserved with anonymized references."
  }
}
```

---

## 8. Payments

### 8.1 Create Payment Intent

See [Section 5.4 - Checkout: Create Payment Intent](#54-create-payment-intent).

### 8.2 Get Payment Status

```
GET /api/v1/rustcommerce/payments/:payment_id
```

**Description**: Check the status of a payment.

**Auth**: Required (customer JWT, must own the order; or admin with `manage_orders`)
**Rate Limit**: 30/min

**Response** `200 OK`:

```json
{
  "data": {
    "id": "0195fab5-1111-7def-8abc-123456789012",
    "order_id": "0195fab2-1234-7def-8abc-123456789012",
    "payment_method": "stripe",
    "status": "completed",
    "amount": "178.77",
    "currency": "USD",
    "transaction_id": "ch_3PxYz123456789",
    "refund_amount": "0.00",
    "created_at": "2026-02-24T10:35:05Z",
    "updated_at": "2026-02-24T10:35:05Z"
  }
}
```

### 8.3 List Payment Methods (Admin)

```
GET /api/v1/rustcommerce/admin/payments/methods
```

**Description**: List configured payment gateways and their status.

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 60/min

**Response** `200 OK`:

```json
{
  "data": [
    {
      "id": "stripe",
      "name": "Stripe",
      "description": "Accept credit and debit card payments via Stripe",
      "enabled": true,
      "test_mode": false,
      "supports": ["credit_card", "debit_card", "apple_pay", "google_pay"],
      "config": {
        "publishable_key_set": true,
        "secret_key_set": true,
        "webhook_secret_set": true
      }
    }
  ]
}
```

---

## 9. Shipping

### 9.1 Get Available Shipping Methods (Public)

```
GET /api/v1/rustcommerce/shipping/methods
```

**Description**: Calculate available shipping methods and costs for a given address and cart.

**Auth**: Optional
**Rate Limit**: 30/min

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `country` | string | Yes | ISO 3166-1 alpha-2 |
| `state` | string | No | State/province code |
| `postal_code` | string | No | Postal/ZIP code |
| `cart_id` | uuid | No | Cart ID for weight/price-based calculation |

**Response** `200 OK`:

```json
{
  "data": [
    {
      "id": "0195fab1-aaaa-7def-8abc-123456789012",
      "name": "Standard Shipping",
      "method_type": "flat_rate",
      "cost": "5.99",
      "estimated_days": "5-7 business days",
      "free_threshold": "100.00",
      "free_threshold_met": true,
      "adjusted_cost": "0.00"
    },
    {
      "id": "0195fab1-bbbb-7def-8abc-123456789012",
      "name": "Express Shipping",
      "method_type": "flat_rate",
      "cost": "14.99",
      "estimated_days": "2-3 business days",
      "free_threshold": null,
      "free_threshold_met": false,
      "adjusted_cost": "14.99"
    }
  ]
}
```

### 9.2 List Shipping Zones (Admin)

```
GET /api/v1/rustcommerce/admin/shipping/zones
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 120/min

**Response** `200 OK`:

```json
{
  "data": [
    {
      "id": "0195fabc-1111-7def-8abc-123456789012",
      "name": "Domestic (US)",
      "countries": ["US"],
      "regions": [],
      "postal_codes": [],
      "is_default": false,
      "position": 0,
      "methods": [
        {
          "id": "0195fab1-aaaa-7def-8abc-123456789012",
          "name": "Standard Shipping",
          "method_type": "flat_rate",
          "cost": "5.99",
          "free_threshold": "100.00",
          "enabled": true
        },
        {
          "id": "0195fab1-bbbb-7def-8abc-123456789012",
          "name": "Express Shipping",
          "method_type": "flat_rate",
          "cost": "14.99",
          "free_threshold": null,
          "enabled": true
        }
      ],
      "created_at": "2026-02-15T08:00:00Z"
    },
    {
      "id": "0195fabc-2222-7def-8abc-123456789012",
      "name": "Rest of World",
      "countries": [],
      "regions": [],
      "postal_codes": [],
      "is_default": true,
      "position": 99,
      "methods": [
        {
          "id": "0195fab1-cccc-7def-8abc-123456789012",
          "name": "International Shipping",
          "method_type": "weight_based",
          "cost": "0.00",
          "settings": {
            "base_cost": "15.00",
            "per_kg": "5.00"
          },
          "enabled": true
        }
      ],
      "created_at": "2026-02-15T08:00:00Z"
    }
  ]
}
```

### 9.3 Create Shipping Zone (Admin)

```
POST /api/v1/rustcommerce/admin/shipping/zones
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 60/min

**Request Body:**

```json
{
  "name": "European Union",
  "countries": ["DE", "FR", "IT", "ES", "NL", "BE", "AT"],
  "regions": [],
  "postal_codes": [],
  "is_default": false,
  "position": 1
}
```

**Response** `201 Created`: Returns the created zone.

### 9.4 Create Shipping Method (Admin)

```
POST /api/v1/rustcommerce/admin/shipping/zones/:zone_id/methods
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 60/min

**Request Body:**

```json
{
  "name": "Standard EU Shipping",
  "method_type": "flat_rate",
  "cost": "9.99",
  "free_threshold": "150.00",
  "enabled": true,
  "position": 0
}
```

**Method Types:**

| Type | Description | Extra Settings |
|------|-------------|----------------|
| `flat_rate` | Fixed cost | `cost` |
| `free_shipping` | Always free | - |
| `weight_based` | Cost per weight unit | `settings.base_cost`, `settings.per_kg`, `settings.min_weight`, `settings.max_weight` |
| `price_based` | Cost tiers based on order total | `settings.tiers: [{min: "0.00", max: "50.00", cost: "7.99"}, ...]` |

**Response** `201 Created`: Returns the created method.

### 9.5 Update Shipping Method (Admin)

```
PUT /api/v1/rustcommerce/admin/shipping/zones/:zone_id/methods/:method_id
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 60/min

### 9.6 Delete Shipping Zone (Admin)

```
DELETE /api/v1/rustcommerce/admin/shipping/zones/:zone_id
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 60/min

**Response** `200 OK`:

```json
{
  "data": {
    "id": "0195fabc-1111-...",
    "deleted": true,
    "methods_deleted": 2
  }
}
```

---

## 10. Tax

### 10.1 Calculate Tax (Internal / Public)

```
POST /api/v1/rustcommerce/tax/calculate
```

**Description**: Calculate tax for a given address and set of line items. Used internally by the checkout flow but also available for tax estimation on the cart page.

**Auth**: Optional
**Rate Limit**: 30/min

**Request Body:**

```json
{
  "address": {
    "country": "US",
    "state": "NY",
    "postal_code": "10001",
    "city": "New York"
  },
  "line_items": [
    {
      "product_id": "0195fa9a-1234-7def-8abc-123456789012",
      "quantity": 2,
      "unit_price": "79.99",
      "tax_class": "standard"
    }
  ],
  "shipping_cost": "5.99"
}
```

**Response** `200 OK`:

```json
{
  "data": {
    "tax_total": "14.50",
    "shipping_tax": "0.53",
    "line_item_taxes": [
      {
        "product_id": "0195fa9a-1234-7def-8abc-123456789012",
        "tax_amount": "13.97",
        "tax_rates_applied": [
          {
            "name": "NY State Tax",
            "rate": "0.0400",
            "amount": "6.40"
          },
          {
            "name": "NYC Local Tax",
            "rate": "0.0450",
            "amount": "7.20",
            "compound": true
          },
          {
            "name": "NYC MTA Surcharge",
            "rate": "0.00375",
            "amount": "0.37",
            "compound": false
          }
        ]
      }
    ],
    "rates_applied": [
      { "name": "NY State Tax", "rate": "0.0400" },
      { "name": "NYC Local Tax", "rate": "0.0450", "compound": true },
      { "name": "NYC MTA Surcharge", "rate": "0.00375" }
    ]
  }
}
```

### 10.2 List Tax Rates (Admin)

```
GET /api/v1/rustcommerce/admin/tax/rates
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 120/min

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `country` | string | Filter by country code |
| `state` | string | Filter by state |
| `tax_class` | string | Filter by tax class |
| `enabled` | boolean | Filter by enabled status |

**Response** `200 OK`:

```json
{
  "data": [
    {
      "id": "0195fac0-1111-7def-8abc-123456789012",
      "name": "NY State Tax",
      "rate": "0.0400",
      "country": "US",
      "state": "NY",
      "postal_code": null,
      "city": null,
      "tax_class": "standard",
      "compound": false,
      "shipping": true,
      "priority": 1,
      "enabled": true,
      "created_at": "2026-02-15T08:00:00Z"
    }
  ]
}
```

### 10.3 Create Tax Rate (Admin)

```
POST /api/v1/rustcommerce/admin/tax/rates
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 60/min

**Request Body:**

```json
{
  "name": "NY State Tax",
  "rate": "0.0400",
  "country": "US",
  "state": "NY",
  "postal_code": null,
  "city": null,
  "tax_class": "standard",
  "compound": false,
  "shipping": true,
  "priority": 1,
  "enabled": true
}
```

**Request Body JSON Schema:**

```json
{
  "type": "object",
  "required": ["name", "rate", "country"],
  "properties": {
    "name": { "type": "string", "minLength": 1, "maxLength": 255 },
    "rate": { "type": "string", "pattern": "^\\d+\\.\\d{4}$", "description": "Decimal rate, e.g. 0.0825 for 8.25%" },
    "country": { "type": "string", "minLength": 2, "maxLength": 2 },
    "state": { "type": ["string", "null"], "maxLength": 100 },
    "postal_code": { "type": ["string", "null"], "maxLength": 20 },
    "city": { "type": ["string", "null"], "maxLength": 100 },
    "tax_class": { "type": "string", "default": "standard" },
    "compound": { "type": "boolean", "default": false },
    "shipping": { "type": "boolean", "default": false },
    "priority": { "type": "integer", "minimum": 1 },
    "enabled": { "type": "boolean", "default": true }
  }
}
```

**Response** `201 Created`: Returns the created tax rate.

### 10.4 Update Tax Rate (Admin)

```
PUT /api/v1/rustcommerce/admin/tax/rates/:id
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 60/min

### 10.5 Delete Tax Rate (Admin)

```
DELETE /api/v1/rustcommerce/admin/tax/rates/:id
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 60/min

---

## 11. Coupons

### 11.1 Validate Coupon (Public)

```
POST /api/v1/rustcommerce/coupons/validate
```

**Description**: Check if a coupon code is valid for the current cart.

**Auth**: Optional
**Rate Limit**: 30/min

**Request Body:**

```json
{
  "code": "SAVE20",
  "cart_id": "0195fa9f-1234-7def-8abc-123456789012"
}
```

**Response** `200 OK`:

```json
{
  "data": {
    "valid": true,
    "code": "SAVE20",
    "discount_type": "percentage",
    "discount_value": "20.00",
    "description": "20% off your order",
    "estimated_discount": "31.99",
    "minimum_spend": "50.00",
    "expires_at": "2026-03-31T23:59:59Z"
  }
}
```

**Response** `200 OK` (invalid coupon):

```json
{
  "data": {
    "valid": false,
    "code": "EXPIRED10",
    "reason": "COUPON_EXPIRED",
    "message": "This coupon has expired"
  }
}
```

### 11.2 List Coupons (Admin)

```
GET /api/v1/rustcommerce/admin/coupons
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 120/min

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `cursor` | string | null | Pagination cursor |
| `limit` | integer | 20 | Items per page |
| `enabled` | boolean | null | Filter by active status |
| `discount_type` | string | null | Filter by type |
| `q` | string | null | Search by code or description |

**Response** `200 OK`:

```json
{
  "data": [
    {
      "id": "0195fac5-1111-7def-8abc-123456789012",
      "code": "SAVE20",
      "description": "20% off your order",
      "discount_type": "percentage",
      "discount_value": "20.00",
      "minimum_spend": "50.00",
      "maximum_spend": null,
      "usage_limit": 1000,
      "usage_count": 234,
      "usage_limit_per_user": 1,
      "product_ids": [],
      "category_ids": [],
      "excluded_product_ids": [],
      "starts_at": "2026-02-01T00:00:00Z",
      "expires_at": "2026-03-31T23:59:59Z",
      "enabled": true,
      "created_at": "2026-02-01T08:00:00Z",
      "updated_at": "2026-02-01T08:00:00Z"
    }
  ],
  "pagination": {
    "cursor": "...",
    "has_more": false,
    "total_count": 12
  }
}
```

### 11.3 Create Coupon (Admin)

```
POST /api/v1/rustcommerce/admin/coupons
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 60/min

**Request Body:**

```json
{
  "code": "SUMMER25",
  "description": "25% off for summer sale",
  "discount_type": "percentage",
  "discount_value": "25.00",
  "minimum_spend": "75.00",
  "maximum_spend": null,
  "usage_limit": 500,
  "usage_limit_per_user": 1,
  "product_ids": [],
  "category_ids": ["0195fa9c-abcd-7def-8abc-123456789012"],
  "excluded_product_ids": [],
  "starts_at": "2026-06-01T00:00:00Z",
  "expires_at": "2026-08-31T23:59:59Z",
  "enabled": true
}
```

**Request Body JSON Schema:**

```json
{
  "type": "object",
  "required": ["code", "discount_type", "discount_value"],
  "properties": {
    "code": { "type": "string", "minLength": 1, "maxLength": 100, "pattern": "^[A-Z0-9_-]+$" },
    "description": { "type": "string" },
    "discount_type": { "type": "string", "enum": ["percentage", "fixed_cart", "fixed_product", "free_shipping"] },
    "discount_value": { "type": "string", "pattern": "^\\d+\\.\\d{2}$" },
    "minimum_spend": { "type": ["string", "null"] },
    "maximum_spend": { "type": ["string", "null"] },
    "usage_limit": { "type": ["integer", "null"], "minimum": 1 },
    "usage_limit_per_user": { "type": ["integer", "null"], "minimum": 1 },
    "product_ids": { "type": "array", "items": { "type": "string", "format": "uuid" } },
    "category_ids": { "type": "array", "items": { "type": "string", "format": "uuid" } },
    "excluded_product_ids": { "type": "array", "items": { "type": "string", "format": "uuid" } },
    "starts_at": { "type": ["string", "null"], "format": "date-time" },
    "expires_at": { "type": ["string", "null"], "format": "date-time" },
    "enabled": { "type": "boolean" }
  }
}
```

**Response** `201 Created`: Returns the created coupon.

### 11.4 Update Coupon (Admin)

```
PUT /api/v1/rustcommerce/admin/coupons/:id
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 60/min

### 11.5 Delete Coupon (Admin)

```
DELETE /api/v1/rustcommerce/admin/coupons/:id
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 60/min

---

## 12. Reviews

### 12.1 List Product Reviews (Public)

```
GET /api/v1/rustcommerce/products/:product_id/reviews
```

**Description**: Retrieve approved reviews for a product.

**Auth**: None required
**Rate Limit**: 60/min

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `cursor` | string | null | Pagination cursor |
| `limit` | integer | 20 | Items per page |
| `rating` | integer | null | Filter by star rating (1-5) |
| `sort` | string | `created_at` | Sort: `created_at`, `rating`, `helpful_count` |
| `order` | string | `desc` | Sort direction |
| `verified_only` | boolean | false | Only show verified purchase reviews |

**Response** `200 OK`:

```json
{
  "data": {
    "summary": {
      "average_rating": 4.5,
      "total_reviews": 23,
      "rating_distribution": {
        "5": 12,
        "4": 7,
        "3": 2,
        "2": 1,
        "1": 1
      }
    },
    "reviews": [
      {
        "id": "0195fad0-1111-7def-8abc-123456789012",
        "rating": 5,
        "title": "Amazing sound quality!",
        "content": "These headphones have incredible noise cancellation and the battery lasts forever.",
        "verified_purchase": true,
        "helpful_count": 15,
        "author": {
          "first_name": "John",
          "last_initial": "D"
        },
        "created_at": "2026-02-20T14:30:00Z"
      }
    ]
  },
  "pagination": {
    "cursor": "...",
    "has_more": true,
    "total_count": 23
  }
}
```

### 12.2 Create Review (Customer)

```
POST /api/v1/rustcommerce/reviews
```

**Description**: Submit a review for a product the customer has purchased.

**Auth**: Required (customer JWT)
**Rate Limit**: 10/min

**Request Body:**

```json
{
  "product_id": "0195fa9a-1234-7def-8abc-123456789012",
  "rating": 5,
  "title": "Amazing sound quality!",
  "content": "These headphones have incredible noise cancellation and the battery lasts forever."
}
```

**Request Body JSON Schema:**

```json
{
  "type": "object",
  "required": ["product_id", "rating"],
  "properties": {
    "product_id": { "type": "string", "format": "uuid" },
    "rating": { "type": "integer", "minimum": 1, "maximum": 5 },
    "title": { "type": "string", "maxLength": 255 },
    "content": { "type": "string", "maxLength": 5000 }
  }
}
```

**Response** `201 Created`:

```json
{
  "data": {
    "id": "0195fad0-2222-7def-8abc-123456789012",
    "product_id": "0195fa9a-1234-7def-8abc-123456789012",
    "rating": 5,
    "title": "Amazing sound quality!",
    "content": "These headphones have incredible noise cancellation and the battery lasts forever.",
    "status": "pending",
    "verified_purchase": true,
    "message": "Thank you for your review! It will be visible after moderation."
  }
}
```

### 12.3 Mark Review Helpful (Customer)

```
POST /api/v1/rustcommerce/reviews/:id/helpful
```

**Auth**: Optional (prevents duplicate votes if authenticated)
**Rate Limit**: 30/min

**Response** `200 OK`:

```json
{
  "data": {
    "id": "0195fad0-1111-7def-8abc-123456789012",
    "helpful_count": 16
  }
}
```

### 12.4 List All Reviews (Admin)

```
GET /api/v1/rustcommerce/admin/reviews
```

**Auth**: Required. Permission: `manage_products`
**Rate Limit**: 120/min

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `status` | string | null | `pending`, `approved`, `rejected`, `spam` |
| `product_id` | uuid | null | Filter by product |
| `rating` | integer | null | Filter by rating |
| `cursor` | string | null | Pagination cursor |
| `limit` | integer | 20 | Items per page |

**Response** `200 OK`: Returns reviews with full customer information and moderation controls.

### 12.5 Moderate Review (Admin)

```
PUT /api/v1/rustcommerce/admin/reviews/:id
```

**Auth**: Required. Permission: `manage_products`
**Rate Limit**: 120/min

**Request Body:**

```json
{
  "status": "approved"
}
```

Valid status transitions: `pending` -> `approved` | `rejected` | `spam`

**Response** `200 OK`: Returns the updated review.

### 12.6 Delete Review (Admin)

```
DELETE /api/v1/rustcommerce/admin/reviews/:id
```

**Auth**: Required. Permission: `manage_products`
**Rate Limit**: 120/min

---

## 13. Admin

### 13.1 Get Store Settings (Admin)

```
GET /api/v1/rustcommerce/admin/settings
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 60/min

**Response** `200 OK`:

```json
{
  "data": {
    "general": {
      "store_name": "My RustPress Store",
      "store_email": "store@example.com",
      "store_phone": "+1-555-000-0000",
      "store_address": {
        "address_line_1": "100 Commerce Way",
        "city": "San Francisco",
        "state": "CA",
        "postal_code": "94105",
        "country": "US"
      },
      "currency": "USD",
      "currency_symbol": "$",
      "currency_position": "before",
      "thousand_separator": ",",
      "decimal_separator": ".",
      "guest_checkout_enabled": true,
      "order_number_prefix": "RC-",
      "order_number_start": 1
    },
    "payments": {
      "stripe": {
        "enabled": true,
        "test_mode": false,
        "publishable_key": "pk_live_***...***abc",
        "webhook_configured": true,
        "payment_methods": ["card", "apple_pay", "google_pay"]
      }
    },
    "shipping": {
      "enabled": true,
      "weight_unit": "kg",
      "dimension_unit": "cm",
      "zones_count": 3,
      "methods_count": 5
    },
    "tax": {
      "enabled": true,
      "prices_include_tax": false,
      "calculate_tax_on": "shipping_address",
      "display_prices_in_shop": "excluding_tax",
      "display_prices_during_checkout": "excluding_tax",
      "rates_count": 15
    },
    "inventory": {
      "manage_stock": true,
      "hold_stock_minutes": 10,
      "low_stock_threshold": 5,
      "out_of_stock_visibility": "hide",
      "backorders_allowed": false
    },
    "reviews": {
      "enabled": true,
      "require_verification": true,
      "auto_approve": false
    },
    "emails": {
      "new_order_admin_notification": true,
      "order_confirmation_customer": true,
      "order_shipped_notification": true,
      "order_delivered_notification": true,
      "low_stock_alert": true
    }
  }
}
```

### 13.2 Update Store Settings (Admin)

```
PUT /api/v1/rustcommerce/admin/settings
```

**Auth**: Required. Permission: `manage_store_settings`
**Rate Limit**: 30/min

**Request Body:** Partial update - only include sections/fields that need changing.

```json
{
  "general": {
    "currency": "EUR",
    "currency_symbol": "EUR",
    "guest_checkout_enabled": false
  },
  "inventory": {
    "hold_stock_minutes": 15,
    "low_stock_threshold": 10
  }
}
```

**Response** `200 OK`: Returns the full updated settings.

### 13.3 Admin Product List

```
GET /api/v1/rustcommerce/admin/products
```

**Description**: List all products (including drafts and archived) with admin-specific data.

**Auth**: Required. Permission: `manage_products`
**Rate Limit**: 120/min

**Additional Query Parameters (beyond public list):**

| Parameter | Type | Description |
|-----------|------|-------------|
| `status` | string | `draft`, `published`, `archived` (comma-separated) |
| `stock_status` | string | `in_stock`, `out_of_stock`, `on_backorder` |
| `low_stock` | boolean | Only products at or below low stock threshold |

**Response** `200 OK`: Same as public product list but includes `cost_price`, `stock_quantity`, `low_stock_threshold`, full status values, and total revenue per product.

---

## 14. Analytics

### 14.1 Store Dashboard Analytics (Admin)

```
GET /api/v1/rustcommerce/admin/analytics/dashboard
```

**Auth**: Required. Permission: `view_store_reports`
**Rate Limit**: 30/min

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `period` | string | `30d` | Time period: `7d`, `30d`, `90d`, `365d`, `custom` |
| `date_from` | string | null | Start date (ISO 8601). Required if `period=custom`. |
| `date_to` | string | null | End date (ISO 8601). Required if `period=custom`. |
| `compare` | boolean | false | Include comparison with previous period |

**Response** `200 OK`:

```json
{
  "data": {
    "period": {
      "from": "2026-01-25T00:00:00Z",
      "to": "2026-02-24T23:59:59Z"
    },
    "summary": {
      "total_revenue": "45230.50",
      "total_orders": 253,
      "average_order_value": "178.78",
      "total_customers": 198,
      "new_customers": 87,
      "returning_customers": 111,
      "conversion_rate": "3.2",
      "items_sold": 612
    },
    "comparison": {
      "total_revenue_change": "+12.5",
      "total_orders_change": "+8.3",
      "average_order_value_change": "+3.9",
      "new_customers_change": "+15.2"
    },
    "revenue_chart": [
      { "date": "2026-01-25", "revenue": "1450.00", "orders": 8 },
      { "date": "2026-01-26", "revenue": "1230.50", "orders": 7 }
    ],
    "top_products": [
      {
        "id": "0195fa9a-1234-...",
        "name": "Wireless Bluetooth Headphones",
        "units_sold": 89,
        "revenue": "7109.11"
      }
    ],
    "orders_by_status": {
      "pending": 5,
      "confirmed": 8,
      "processing": 12,
      "shipped": 45,
      "delivered": 175,
      "cancelled": 6,
      "refunded": 2
    },
    "recent_orders": [
      {
        "id": "0195fab2-1234-...",
        "order_number": "RC-00042",
        "customer_name": "Jane Doe",
        "grand_total": "178.77",
        "status": "shipped",
        "created_at": "2026-02-24T10:35:00Z"
      }
    ]
  }
}
```

### 14.2 Revenue Report (Admin)

```
GET /api/v1/rustcommerce/admin/analytics/revenue
```

**Auth**: Required. Permission: `view_store_reports`
**Rate Limit**: 30/min

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `period` | string | `30d` | Time period |
| `granularity` | string | `day` | `hour`, `day`, `week`, `month` |

**Response** `200 OK`:

```json
{
  "data": {
    "total_revenue": "45230.50",
    "total_tax": "3618.44",
    "total_shipping": "1519.47",
    "total_discounts": "890.00",
    "net_revenue": "39202.59",
    "total_refunds": "523.99",
    "data_points": [
      {
        "date": "2026-02-01",
        "gross_revenue": "1580.00",
        "net_revenue": "1350.00",
        "orders": 9,
        "refunds": "0.00"
      }
    ]
  }
}
```

### 14.3 Product Performance (Admin)

```
GET /api/v1/rustcommerce/admin/analytics/products
```

**Auth**: Required. Permission: `view_store_reports`
**Rate Limit**: 30/min

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `period` | string | `30d` | Time period |
| `sort` | string | `revenue` | `revenue`, `units_sold`, `views` |
| `limit` | integer | 20 | Number of products |

**Response** `200 OK`:

```json
{
  "data": [
    {
      "product_id": "0195fa9a-1234-...",
      "product_name": "Wireless Bluetooth Headphones",
      "sku": "WBH-001",
      "units_sold": 89,
      "revenue": "7109.11",
      "average_price": "79.88",
      "stock_remaining": 61,
      "refund_rate": "1.1"
    }
  ]
}
```

---

## 15. Webhooks

### 15.1 Stripe Webhook

```
POST /api/v1/rustcommerce/webhooks/stripe
```

**Description**: Receives Stripe webhook events. This endpoint is called by Stripe's servers, not by clients.

**Auth**: Stripe webhook signature verification (see AUTH_FLOW.md)
**Rate Limit**: Unlimited (verified by signature)

**Headers:**

| Header | Required | Description |
|--------|----------|-------------|
| `Stripe-Signature` | Yes | HMAC signature for verification |
| `Content-Type` | Yes | `application/json` |

**Handled Event Types:**

| Event | Action |
|-------|--------|
| `payment_intent.succeeded` | Mark payment as completed, create/confirm order |
| `payment_intent.payment_failed` | Mark payment as failed, release stock reservation |
| `payment_intent.canceled` | Cancel checkout, release stock |
| `charge.refunded` | Update order payment status, restock if applicable |
| `charge.dispute.created` | Flag order for review, notify admin |
| `charge.dispute.closed` | Update dispute status |

**Request Body** (from Stripe):

```json
{
  "id": "evt_1PxYz123456789",
  "object": "event",
  "type": "payment_intent.succeeded",
  "data": {
    "object": {
      "id": "pi_3PxYz123456789",
      "amount": 17877,
      "currency": "usd",
      "status": "succeeded",
      "metadata": {
        "order_id": "0195fab2-1234-7def-8abc-123456789012",
        "checkout_session_id": "0195fab0-1234-7def-8abc-123456789012"
      }
    }
  }
}
```

**Response** `200 OK`:

```json
{
  "received": true
}
```

Always returns 200 to acknowledge receipt. Failed processing triggers internal retry mechanisms.

### 15.2 RustCommerce Event Webhooks (Outgoing)

RustCommerce fires hooks on key events that other RustPress plugins can subscribe to. These can also be configured as outgoing webhooks to external URLs.

**Configurable Events:**

| Event Name | Description | Payload Key Fields |
|------------|-------------|-------------------|
| `rustcommerce_order_created` | New order placed | `order_id`, `order_number`, `grand_total`, `customer_email` |
| `rustcommerce_order_status_changed` | Order status updated | `order_id`, `old_status`, `new_status` |
| `rustcommerce_payment_completed` | Payment successful | `order_id`, `payment_id`, `amount` |
| `rustcommerce_payment_failed` | Payment failed | `order_id`, `payment_id`, `error` |
| `rustcommerce_refund_issued` | Refund processed | `order_id`, `refund_id`, `amount` |
| `rustcommerce_product_created` | New product | `product_id`, `name`, `sku` |
| `rustcommerce_product_updated` | Product changed | `product_id`, `changed_fields` |
| `rustcommerce_product_deleted` | Product archived | `product_id` |
| `rustcommerce_stock_low` | Stock below threshold | `product_id`, `variant_id`, `stock_quantity`, `threshold` |
| `rustcommerce_stock_depleted` | Stock reached zero | `product_id`, `variant_id` |
| `rustcommerce_customer_created` | New customer registered | `customer_id`, `email` |
| `rustcommerce_review_submitted` | New review pending moderation | `review_id`, `product_id`, `rating` |
| `rustcommerce_cart_abandoned` | Cart inactive for > 1 hour | `cart_id`, `user_id`, `total` |

---

## 16. Inventory

### 16.1 Inventory Report (Admin)

```
GET /api/v1/rustcommerce/admin/inventory
```

**Auth**: Required. Permission: `manage_products`
**Rate Limit**: 120/min

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `cursor` | string | null | Pagination cursor |
| `limit` | integer | 50 | Items per page |
| `stock_status` | string | null | `in_stock`, `low_stock`, `out_of_stock`, `on_backorder` |
| `sort` | string | `stock_quantity` | Sort field |
| `order` | string | `asc` | Sort direction |

**Response** `200 OK`:

```json
{
  "data": {
    "summary": {
      "total_products": 342,
      "in_stock": 290,
      "low_stock": 25,
      "out_of_stock": 22,
      "on_backorder": 5,
      "total_stock_value": "125430.00"
    },
    "items": [
      {
        "product_id": "0195fa9a-1234-...",
        "product_name": "Wireless Bluetooth Headphones",
        "sku": "WBH-001",
        "stock_quantity": 150,
        "stock_status": "in_stock",
        "low_stock_threshold": 10,
        "cost_price": "35.00",
        "stock_value": "5250.00",
        "variants": [
          {
            "variant_id": "0195fa9d-1111-...",
            "variant_name": "Black",
            "sku": "WBH-001-BLK",
            "stock_quantity": 75
          },
          {
            "variant_id": "0195fa9d-2222-...",
            "variant_name": "White",
            "sku": "WBH-001-WHT",
            "stock_quantity": 50
          },
          {
            "variant_id": "0195fa9d-3333-...",
            "variant_name": "Red",
            "sku": "WBH-001-RED",
            "stock_quantity": 25
          }
        ]
      }
    ]
  },
  "pagination": {
    "cursor": "...",
    "has_more": true,
    "total_count": 342
  }
}
```

### 16.2 Update Stock (Admin)

```
PUT /api/v1/rustcommerce/admin/inventory/:product_id
```

**Auth**: Required. Permission: `manage_products`
**Rate Limit**: 120/min

**Request Body:**

```json
{
  "stock_quantity": 200,
  "stock_status": "in_stock",
  "low_stock_threshold": 15,
  "variants": [
    {
      "variant_id": "0195fa9d-1111-7def-8abc-123456789012",
      "stock_quantity": 100
    },
    {
      "variant_id": "0195fa9d-2222-7def-8abc-123456789012",
      "stock_quantity": 60
    }
  ]
}
```

**Response** `200 OK`: Returns the updated inventory for the product.

### 16.3 Bulk Stock Update (Admin)

```
POST /api/v1/rustcommerce/admin/inventory/bulk
```

**Auth**: Required. Permission: `manage_products`
**Rate Limit**: 30/min

**Request Body:**

```json
{
  "adjustments": [
    {
      "product_id": "0195fa9a-1234-...",
      "variant_id": null,
      "adjustment": 50,
      "type": "add",
      "reason": "Restock from supplier"
    },
    {
      "product_id": "0195fa9a-5678-...",
      "variant_id": "0195fa9d-4444-...",
      "adjustment": 10,
      "type": "subtract",
      "reason": "Damaged inventory"
    }
  ]
}
```

**Response** `200 OK`:

```json
{
  "data": {
    "total": 2,
    "succeeded": 2,
    "failed": 0,
    "results": [
      {
        "product_id": "0195fa9a-1234-...",
        "success": true,
        "new_stock_quantity": 200
      },
      {
        "product_id": "0195fa9a-5678-...",
        "variant_id": "0195fa9d-4444-...",
        "success": true,
        "new_stock_quantity": 15
      }
    ]
  }
}
```

---

## Appendix A: Complete Endpoint Reference

| # | Method | Path | Auth | Permission | Rate Limit |
|---|--------|------|------|------------|------------|
| **Products** | | | | | |
| 1 | GET | `/products` | None | - | 60/min |
| 2 | GET | `/products/:id` | None | - | 60/min |
| 3 | POST | `/admin/products` | JWT | `manage_products` | 120/min |
| 4 | PUT | `/admin/products/:id` | JWT | `manage_products` | 120/min |
| 5 | DELETE | `/admin/products/:id` | JWT | `manage_products` | 120/min |
| 6 | POST | `/admin/products/bulk` | JWT | `manage_products` | 30/min |
| **Categories** | | | | | |
| 7 | GET | `/categories` | None | - | 60/min |
| 8 | GET | `/categories/:id` | None | - | 60/min |
| 9 | POST | `/admin/categories` | JWT | `manage_products` | 120/min |
| 10 | PUT | `/admin/categories/:id` | JWT | `manage_products` | 120/min |
| 11 | DELETE | `/admin/categories/:id` | JWT | `manage_products` | 120/min |
| **Cart** | | | | | |
| 12 | GET | `/cart` | Session/JWT | - | 60/min |
| 13 | POST | `/cart/items` | Session/JWT | - | 60/min |
| 14 | PUT | `/cart/items/:item_id` | Session/JWT | - | 60/min |
| 15 | DELETE | `/cart/items/:item_id` | Session/JWT | - | 60/min |
| 16 | DELETE | `/cart` | Session/JWT | - | 60/min |
| 17 | POST | `/cart/coupon` | Session/JWT | - | 30/min |
| 18 | DELETE | `/cart/coupon` | Session/JWT | - | 60/min |
| **Checkout** | | | | | |
| 19 | POST | `/checkout/init` | Session/JWT | - | 10/min |
| 20 | POST | `/checkout/shipping-address` | Session/JWT | - | 10/min |
| 21 | POST | `/checkout/shipping-method` | Session/JWT | - | 10/min |
| 22 | POST | `/checkout/payment-intent` | Session/JWT | - | 5/min |
| 23 | POST | `/checkout/complete` | Session/JWT | - | 5/min |
| **Orders** | | | | | |
| 24 | GET | `/orders` | JWT | - | 60/min |
| 25 | GET | `/orders/:id` | JWT | - | 60/min |
| 26 | GET | `/admin/orders` | JWT | `manage_orders` | 120/min |
| 27 | GET | `/admin/orders/:id` | JWT | `manage_orders` | 120/min |
| 28 | PUT | `/admin/orders/:id/status` | JWT | `manage_orders` | 120/min |
| 29 | POST | `/admin/orders/:id/notes` | JWT | `manage_orders` | 120/min |
| 30 | POST | `/admin/orders/:id/refund` | JWT | `manage_orders` | 10/min |
| **Customers** | | | | | |
| 31 | GET | `/account` | JWT | - | 60/min |
| 32 | PUT | `/account` | JWT | - | 30/min |
| 33 | GET | `/account/addresses` | JWT | - | 60/min |
| 34 | POST | `/account/addresses` | JWT | - | 30/min |
| 35 | PUT | `/account/addresses/:id` | JWT | - | 30/min |
| 36 | DELETE | `/account/addresses/:id` | JWT | - | 30/min |
| 37 | GET | `/admin/customers` | JWT | `manage_customers` | 120/min |
| 38 | GET | `/admin/customers/:id` | JWT | `manage_customers` | 120/min |
| 39 | PUT | `/admin/customers/:id` | JWT | `manage_customers` | 120/min |
| 40 | DELETE | `/admin/customers/:id` | JWT | `manage_customers` | 30/min |
| **Payments** | | | | | |
| 41 | GET | `/payments/:payment_id` | JWT | - | 30/min |
| 42 | GET | `/admin/payments/methods` | JWT | `manage_store_settings` | 60/min |
| **Shipping** | | | | | |
| 43 | GET | `/shipping/methods` | None | - | 30/min |
| 44 | GET | `/admin/shipping/zones` | JWT | `manage_store_settings` | 120/min |
| 45 | POST | `/admin/shipping/zones` | JWT | `manage_store_settings` | 60/min |
| 46 | PUT | `/admin/shipping/zones/:zone_id` | JWT | `manage_store_settings` | 60/min |
| 47 | DELETE | `/admin/shipping/zones/:zone_id` | JWT | `manage_store_settings` | 60/min |
| 48 | POST | `/admin/shipping/zones/:zone_id/methods` | JWT | `manage_store_settings` | 60/min |
| 49 | PUT | `/admin/shipping/zones/:zone_id/methods/:method_id` | JWT | `manage_store_settings` | 60/min |
| 50 | DELETE | `/admin/shipping/zones/:zone_id/methods/:method_id` | JWT | `manage_store_settings` | 60/min |
| **Tax** | | | | | |
| 51 | POST | `/tax/calculate` | None | - | 30/min |
| 52 | GET | `/admin/tax/rates` | JWT | `manage_store_settings` | 120/min |
| 53 | POST | `/admin/tax/rates` | JWT | `manage_store_settings` | 60/min |
| 54 | PUT | `/admin/tax/rates/:id` | JWT | `manage_store_settings` | 60/min |
| 55 | DELETE | `/admin/tax/rates/:id` | JWT | `manage_store_settings` | 60/min |
| **Coupons** | | | | | |
| 56 | POST | `/coupons/validate` | None | - | 30/min |
| 57 | GET | `/admin/coupons` | JWT | `manage_store_settings` | 120/min |
| 58 | POST | `/admin/coupons` | JWT | `manage_store_settings` | 60/min |
| 59 | PUT | `/admin/coupons/:id` | JWT | `manage_store_settings` | 60/min |
| 60 | DELETE | `/admin/coupons/:id` | JWT | `manage_store_settings` | 60/min |
| **Reviews** | | | | | |
| 61 | GET | `/products/:product_id/reviews` | None | - | 60/min |
| 62 | POST | `/reviews` | JWT | - | 10/min |
| 63 | POST | `/reviews/:id/helpful` | None | - | 30/min |
| 64 | GET | `/admin/reviews` | JWT | `manage_products` | 120/min |
| 65 | PUT | `/admin/reviews/:id` | JWT | `manage_products` | 120/min |
| 66 | DELETE | `/admin/reviews/:id` | JWT | `manage_products` | 120/min |
| **Admin Settings** | | | | | |
| 67 | GET | `/admin/settings` | JWT | `manage_store_settings` | 60/min |
| 68 | PUT | `/admin/settings` | JWT | `manage_store_settings` | 30/min |
| **Analytics** | | | | | |
| 69 | GET | `/admin/analytics/dashboard` | JWT | `view_store_reports` | 30/min |
| 70 | GET | `/admin/analytics/revenue` | JWT | `view_store_reports` | 30/min |
| 71 | GET | `/admin/analytics/products` | JWT | `view_store_reports` | 30/min |
| **Inventory** | | | | | |
| 72 | GET | `/admin/inventory` | JWT | `manage_products` | 120/min |
| 73 | PUT | `/admin/inventory/:product_id` | JWT | `manage_products` | 120/min |
| 74 | POST | `/admin/inventory/bulk` | JWT | `manage_products` | 30/min |
| **Webhooks** | | | | | |
| 75 | POST | `/webhooks/stripe` | Stripe Signature | - | Unlimited |

**Total: 75 endpoints**
