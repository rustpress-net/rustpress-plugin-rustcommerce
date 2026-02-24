# Frontend API Integration -- RustCommerce Admin UI

**Document Version**: 1.0
**Date**: 2026-02-24
**Owner**: Frontend Lead
**Status**: Approved

---

## 1. Overview

The RustCommerce admin UI communicates with the backend through a dedicated `commerceApi` module that extends the existing RustPress Axios client (`@/api/client`). All requests are authenticated via JWT (handled by the client's interceptors) and target the `/api/v1/rustcommerce/` endpoint namespace.

**File**: `src/pages/plugins/rustcommerce/api/commerceApi.ts`

---

## 2. API Client Module Structure

```typescript
// src/pages/plugins/rustcommerce/api/commerceApi.ts

import { apiClient } from '@/api/client';
import type { AxiosResponse } from 'axios';
import type {
  Product,
  ProductVariant,
  ProductImage,
  Category,
  Order,
  OrderStatus,
  Customer,
  CustomerAddress,
  StoreSettings,
  ShippingZone,
  ShippingMethod,
  TaxRate,
  AnalyticsResponse,
  PaginatedResponse,
  ListParams,
  InventoryReport,
  Coupon,
} from '../types';

const BASE = '/v1/rustcommerce';
const ADMIN = `${BASE}/admin`;

// =============================================================================
// PRODUCTS API
// =============================================================================

export const productsApi = {
  list: (params?: ListParams): Promise<AxiosResponse<PaginatedResponse<Product>>> =>
    apiClient.get(`${ADMIN}/products`, { params }),

  get: (id: string): Promise<AxiosResponse<Product>> =>
    apiClient.get(`${ADMIN}/products/${id}`),

  create: (data: Partial<Product>): Promise<AxiosResponse<Product>> =>
    apiClient.post(`${ADMIN}/products`, data),

  update: (id: string, data: Partial<Product>): Promise<AxiosResponse<Product>> =>
    apiClient.put(`${ADMIN}/products/${id}`, data),

  delete: (id: string): Promise<AxiosResponse<void>> =>
    apiClient.delete(`${ADMIN}/products/${id}`),

  // Variants
  addVariant: (productId: string, data: Partial<ProductVariant>): Promise<AxiosResponse<ProductVariant>> =>
    apiClient.post(`${ADMIN}/products/${productId}/variants`, data),

  updateVariant: (productId: string, variantId: string, data: Partial<ProductVariant>): Promise<AxiosResponse<ProductVariant>> =>
    apiClient.put(`${ADMIN}/products/${productId}/variants/${variantId}`, data),

  deleteVariant: (productId: string, variantId: string): Promise<AxiosResponse<void>> =>
    apiClient.delete(`${ADMIN}/products/${productId}/variants/${variantId}`),

  // Images
  uploadImages: (productId: string, files: File[]): Promise<AxiosResponse<ProductImage[]>> => {
    const formData = new FormData();
    files.forEach((file) => formData.append('images', file));
    return apiClient.post(`${ADMIN}/products/${productId}/images`, formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
  },

  deleteImage: (productId: string, imageId: string): Promise<AxiosResponse<void>> =>
    apiClient.delete(`${ADMIN}/products/${productId}/images/${imageId}`),

  setPrimaryImage: (productId: string, imageId: string): Promise<AxiosResponse<void>> =>
    apiClient.put(`${ADMIN}/products/${productId}/images/${imageId}/primary`),

  reorderImages: (productId: string, imageIds: string[]): Promise<AxiosResponse<void>> =>
    apiClient.put(`${ADMIN}/products/${productId}/images/reorder`, { imageIds }),
};

// =============================================================================
// CATEGORIES API
// =============================================================================

export const categoriesApi = {
  list: (): Promise<AxiosResponse<Category[]>> =>
    apiClient.get(`${ADMIN}/categories`),

  get: (id: string): Promise<AxiosResponse<Category>> =>
    apiClient.get(`${ADMIN}/categories/${id}`),

  create: (data: Partial<Category>): Promise<AxiosResponse<Category>> =>
    apiClient.post(`${ADMIN}/categories`, data),

  update: (id: string, data: Partial<Category>): Promise<AxiosResponse<Category>> =>
    apiClient.put(`${ADMIN}/categories/${id}`, data),

  delete: (id: string): Promise<AxiosResponse<void>> =>
    apiClient.delete(`${ADMIN}/categories/${id}`),
};

// =============================================================================
// ORDERS API
// =============================================================================

export const ordersApi = {
  list: (params?: ListParams): Promise<AxiosResponse<PaginatedResponse<Order>>> =>
    apiClient.get(`${ADMIN}/orders`, { params }),

  get: (id: string): Promise<AxiosResponse<Order>> =>
    apiClient.get(`${ADMIN}/orders/${id}`),

  updateStatus: (id: string, status: OrderStatus, note?: string): Promise<AxiosResponse<Order>> =>
    apiClient.put(`${ADMIN}/orders/${id}`, { status, note }),

  addNote: (id: string, note: string): Promise<AxiosResponse<void>> =>
    apiClient.post(`${ADMIN}/orders/${id}/notes`, { note }),

  refund: (id: string, amount: number, reason: string): Promise<AxiosResponse<Order>> =>
    apiClient.post(`${ADMIN}/orders/${id}/refund`, { amount, reason }),

  exportOrders: (params?: ListParams & { format: 'csv' | 'pdf' }): Promise<AxiosResponse<Blob>> =>
    apiClient.get(`${ADMIN}/orders/export`, {
      params,
      responseType: 'blob',
    }),
};

// =============================================================================
// CUSTOMERS API
// =============================================================================

export const customersApi = {
  list: (params?: ListParams): Promise<AxiosResponse<PaginatedResponse<Customer>>> =>
    apiClient.get(`${ADMIN}/customers`, { params }),

  get: (id: string): Promise<AxiosResponse<Customer>> =>
    apiClient.get(`${ADMIN}/customers/${id}`),

  update: (id: string, data: Partial<Customer>): Promise<AxiosResponse<Customer>> =>
    apiClient.put(`${ADMIN}/customers/${id}`, data),

  delete: (id: string): Promise<AxiosResponse<void>> =>
    apiClient.delete(`${ADMIN}/customers/${id}`),

  getOrders: (customerId: string, params?: ListParams): Promise<AxiosResponse<PaginatedResponse<Order>>> =>
    apiClient.get(`${ADMIN}/customers/${customerId}/orders`, { params }),

  getAddresses: (customerId: string): Promise<AxiosResponse<CustomerAddress[]>> =>
    apiClient.get(`${ADMIN}/customers/${customerId}/addresses`),

  exportCustomers: (params?: { format: 'csv' }): Promise<AxiosResponse<Blob>> =>
    apiClient.get(`${ADMIN}/customers/export`, {
      params,
      responseType: 'blob',
    }),
};

// =============================================================================
// ANALYTICS API
// =============================================================================

export const analyticsApi = {
  getSummary: (params?: { period: string }): Promise<AxiosResponse<AnalyticsResponse>> =>
    apiClient.get(`${ADMIN}/analytics`, { params }),

  getRevenueChart: (params?: { period: string; granularity?: 'day' | 'week' | 'month' }): Promise<AxiosResponse<Array<{ date: string; revenue: number; orders: number }>>> =>
    apiClient.get(`${ADMIN}/analytics/revenue`, { params }),

  getTopProducts: (params?: { period: string; limit?: number }): Promise<AxiosResponse<Array<{ id: string; name: string; imageUrl?: string; unitsSold: number; revenue: number }>>> =>
    apiClient.get(`${ADMIN}/analytics/top-products`, { params }),
};

// =============================================================================
// INVENTORY API
// =============================================================================

export const inventoryApi = {
  getReport: (params?: ListParams): Promise<AxiosResponse<PaginatedResponse<InventoryReport>>> =>
    apiClient.get(`${ADMIN}/inventory`, { params }),

  updateStock: (productId: string, quantity: number, reason?: string): Promise<AxiosResponse<void>> =>
    apiClient.put(`${ADMIN}/inventory/${productId}`, { quantity, reason }),

  getLowStock: (threshold?: number): Promise<AxiosResponse<Array<{ id: string; name: string; sku: string; stockQuantity: number; lowStockThreshold: number }>>> =>
    apiClient.get(`${ADMIN}/inventory/low-stock`, { params: { threshold } }),
};

// =============================================================================
// SETTINGS API
// =============================================================================

export const settingsApi = {
  get: (): Promise<AxiosResponse<{
    general: StoreSettings;
    shippingZones: ShippingZone[];
    shippingMethods: ShippingMethod[];
    taxRates: TaxRate[];
  }>> =>
    apiClient.get(`${ADMIN}/settings`),

  update: (data: Partial<StoreSettings>): Promise<AxiosResponse<StoreSettings>> =>
    apiClient.put(`${ADMIN}/settings`, data),

  // Shipping Zones
  listShippingZones: (): Promise<AxiosResponse<ShippingZone[]>> =>
    apiClient.get(`${ADMIN}/settings/shipping/zones`),

  createShippingZone: (data: Partial<ShippingZone>): Promise<AxiosResponse<ShippingZone>> =>
    apiClient.post(`${ADMIN}/settings/shipping/zones`, data),

  updateShippingZone: (id: string, data: Partial<ShippingZone>): Promise<AxiosResponse<ShippingZone>> =>
    apiClient.put(`${ADMIN}/settings/shipping/zones/${id}`, data),

  deleteShippingZone: (id: string): Promise<AxiosResponse<void>> =>
    apiClient.delete(`${ADMIN}/settings/shipping/zones/${id}`),

  // Shipping Methods
  createShippingMethod: (zoneId: string, data: Partial<ShippingMethod>): Promise<AxiosResponse<ShippingMethod>> =>
    apiClient.post(`${ADMIN}/settings/shipping/zones/${zoneId}/methods`, data),

  updateShippingMethod: (id: string, data: Partial<ShippingMethod>): Promise<AxiosResponse<ShippingMethod>> =>
    apiClient.put(`${ADMIN}/settings/shipping/methods/${id}`, data),

  deleteShippingMethod: (id: string): Promise<AxiosResponse<void>> =>
    apiClient.delete(`${ADMIN}/settings/shipping/methods/${id}`),

  // Tax Rates
  listTaxRates: (): Promise<AxiosResponse<TaxRate[]>> =>
    apiClient.get(`${ADMIN}/settings/tax/rates`),

  createTaxRate: (data: Partial<TaxRate>): Promise<AxiosResponse<TaxRate>> =>
    apiClient.post(`${ADMIN}/settings/tax/rates`, data),

  updateTaxRate: (id: string, data: Partial<TaxRate>): Promise<AxiosResponse<TaxRate>> =>
    apiClient.put(`${ADMIN}/settings/tax/rates/${id}`, data),

  deleteTaxRate: (id: string): Promise<AxiosResponse<void>> =>
    apiClient.delete(`${ADMIN}/settings/tax/rates/${id}`),

  // Test payment gateway connection
  testPaymentGateway: (): Promise<AxiosResponse<{ success: boolean; message: string }>> =>
    apiClient.post(`${ADMIN}/settings/payments/test`),
};

// =============================================================================
// COUPONS API (P1 -- Post-MVP)
// =============================================================================

export const couponsApi = {
  list: (params?: ListParams): Promise<AxiosResponse<PaginatedResponse<Coupon>>> =>
    apiClient.get(`${ADMIN}/coupons`, { params }),

  get: (id: string): Promise<AxiosResponse<Coupon>> =>
    apiClient.get(`${ADMIN}/coupons/${id}`),

  create: (data: Partial<Coupon>): Promise<AxiosResponse<Coupon>> =>
    apiClient.post(`${ADMIN}/coupons`, data),

  update: (id: string, data: Partial<Coupon>): Promise<AxiosResponse<Coupon>> =>
    apiClient.put(`${ADMIN}/coupons/${id}`, data),

  delete: (id: string): Promise<AxiosResponse<void>> =>
    apiClient.delete(`${ADMIN}/coupons/${id}`),
};

// =============================================================================
// UNIFIED commerceApi FACADE
// =============================================================================

export const commerceApi = {
  // Products
  listProducts: productsApi.list,
  getProduct: productsApi.get,
  createProduct: productsApi.create,
  updateProduct: productsApi.update,
  deleteProduct: productsApi.delete,
  addProductVariant: productsApi.addVariant,
  updateProductVariant: productsApi.updateVariant,
  deleteProductVariant: productsApi.deleteVariant,
  uploadProductImages: productsApi.uploadImages,
  deleteProductImage: productsApi.deleteImage,
  setPrimaryProductImage: productsApi.setPrimaryImage,
  reorderProductImages: productsApi.reorderImages,

  // Categories
  listCategories: categoriesApi.list,
  getCategory: categoriesApi.get,
  createCategory: categoriesApi.create,
  updateCategory: categoriesApi.update,
  deleteCategory: categoriesApi.delete,

  // Orders
  listOrders: ordersApi.list,
  getOrder: ordersApi.get,
  updateOrderStatus: ordersApi.updateStatus,
  addOrderNote: ordersApi.addNote,
  refundOrder: ordersApi.refund,
  exportOrders: ordersApi.exportOrders,

  // Customers
  listCustomers: customersApi.list,
  getCustomer: customersApi.get,
  updateCustomer: customersApi.update,
  deleteCustomer: customersApi.delete,
  getCustomerOrders: customersApi.getOrders,
  getCustomerAddresses: customersApi.getAddresses,
  exportCustomers: customersApi.exportCustomers,

  // Analytics
  getAnalytics: analyticsApi.getSummary,
  getRevenueChart: analyticsApi.getRevenueChart,
  getTopProducts: analyticsApi.getTopProducts,

  // Inventory
  getInventoryReport: inventoryApi.getReport,
  updateStock: inventoryApi.updateStock,
  getLowStock: inventoryApi.getLowStock,

  // Settings
  getSettings: settingsApi.get,
  updateSettings: settingsApi.update,
  listShippingZones: settingsApi.listShippingZones,
  createShippingZone: settingsApi.createShippingZone,
  updateShippingZone: settingsApi.updateShippingZone,
  deleteShippingZone: settingsApi.deleteShippingZone,
  createShippingMethod: settingsApi.createShippingMethod,
  updateShippingMethod: settingsApi.updateShippingMethod,
  deleteShippingMethod: settingsApi.deleteShippingMethod,
  listTaxRates: settingsApi.listTaxRates,
  createTaxRate: settingsApi.createTaxRate,
  updateTaxRate: settingsApi.updateTaxRate,
  deleteTaxRate: settingsApi.deleteTaxRate,
  testPaymentGateway: settingsApi.testPaymentGateway,

  // Coupons (P1)
  listCoupons: couponsApi.list,
  getCoupon: couponsApi.get,
  createCoupon: couponsApi.create,
  updateCoupon: couponsApi.update,
  deleteCoupon: couponsApi.delete,
};
```

---

## 3. Request/Response TypeScript Interfaces

```typescript
// src/pages/plugins/rustcommerce/types/index.ts

// =============================================================================
// COMMON TYPES
// =============================================================================

export interface ListParams {
  page?: number;
  pageSize?: number;
  search?: string;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
  status?: string | string[];
  [key: string]: any;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

export interface ApiError {
  message: string;
  code: string;
  details?: Record<string, string[]>;   // Field-level validation errors
  statusCode: number;
}

// =============================================================================
// PRODUCT TYPES
// =============================================================================

export type ProductStatus = 'draft' | 'published' | 'archived';
export type ProductType = 'simple' | 'variable' | 'grouped' | 'digital';
export type StockStatus = 'in_stock' | 'out_of_stock' | 'on_backorder';

export interface Product {
  id: string;
  name: string;
  slug: string;
  description: string;
  shortDescription: string;
  sku: string;
  price: number;
  compareAtPrice?: number;
  costPrice?: number;
  status: ProductStatus;
  productType: ProductType;
  featured: boolean;
  stockQuantity: number;
  stockStatus: StockStatus;
  lowStockThreshold: number;
  weight?: number;
  dimensionsLength?: number;
  dimensionsWidth?: number;
  dimensionsHeight?: number;
  taxClass: string;
  meta: Record<string, any>;
  createdAt: string;
  updatedAt: string;

  // Relations (populated on detail fetch)
  variants?: ProductVariant[];
  images?: ProductImage[];
  categories?: Category[];
  tags?: string[];
}

export interface ProductVariant {
  id: string;
  productId: string;
  sku: string;
  name: string;
  price: number;
  compareAtPrice?: number;
  stockQuantity: number;
  attributes: Record<string, string>;   // e.g., { color: "Red", size: "XL" }
  imageUrl?: string;
  position: number;
  createdAt: string;
  updatedAt: string;
}

export interface ProductImage {
  id: string;
  productId: string;
  url: string;
  altText: string;
  position: number;
  isPrimary: boolean;
  createdAt: string;
}

// =============================================================================
// CATEGORY TYPES
// =============================================================================

export interface Category {
  id: string;
  name: string;
  slug: string;
  description?: string;
  parentId?: string;
  imageUrl?: string;
  position: number;
  productCount: number;
  createdAt: string;
  updatedAt: string;
  children?: Category[];               // Populated for tree display
}

// =============================================================================
// ORDER TYPES
// =============================================================================

export type OrderStatus =
  | 'pending'
  | 'confirmed'
  | 'processing'
  | 'shipped'
  | 'delivered'
  | 'cancelled'
  | 'refunded';

export type PaymentStatus =
  | 'unpaid'
  | 'paid'
  | 'partially_refunded'
  | 'refunded';

export interface Order {
  id: string;
  orderNumber: string;               // Human-readable: RC-00001
  userId?: string;
  status: OrderStatus;
  subtotal: number;
  taxTotal: number;
  shippingTotal: number;
  discountTotal: number;
  grandTotal: number;
  currency: string;
  billingAddress: Address;
  shippingAddress: Address;
  shippingMethod?: string;
  paymentMethod?: string;
  paymentStatus: PaymentStatus;
  stripePaymentIntentId?: string;
  couponCode?: string;
  customerNote?: string;
  adminNote?: string;
  ipAddress?: string;
  createdAt: string;
  updatedAt: string;
  completedAt?: string;
  cancelledAt?: string;

  // Relations (populated on detail fetch)
  items?: OrderItem[];
  customer?: Customer;
  payment?: Payment;
  statusHistory?: OrderEvent[];
}

export interface OrderItem {
  id: string;
  orderId: string;
  productId: string;
  variantId?: string;
  productName: string;                // Snapshot at order time
  variantName?: string;
  sku?: string;
  quantity: number;
  unitPrice: number;
  subtotal: number;
  taxAmount: number;
  discountAmount: number;
  total: number;
  meta: Record<string, any>;
  createdAt: string;
}

export interface OrderEvent {
  id: string;
  type: 'status_change' | 'payment' | 'note' | 'refund' | 'shipment';
  title: string;
  description?: string;
  timestamp: string;
  actor?: string;
}

// =============================================================================
// CUSTOMER TYPES
// =============================================================================

export interface Customer {
  id: string;
  userId?: string;
  email: string;
  firstName: string;
  lastName: string;
  phone?: string;
  totalOrders: number;
  totalSpent: number;
  averageOrderValue: number;
  lastOrderAt?: string;
  notes?: string;
  meta: Record<string, any>;
  createdAt: string;
  updatedAt: string;

  // Relations
  addresses?: CustomerAddress[];
}

export interface CustomerAddress {
  id: string;
  customerId: string;
  addressType: 'billing' | 'shipping';
  isDefault: boolean;
  firstName: string;
  lastName: string;
  company?: string;
  addressLine1: string;
  addressLine2?: string;
  city: string;
  state?: string;
  postalCode: string;
  country: string;                    // ISO 3166-1 alpha-2
  phone?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Address {
  firstName: string;
  lastName: string;
  company?: string;
  addressLine1: string;
  addressLine2?: string;
  city: string;
  state?: string;
  postalCode: string;
  country: string;
  phone?: string;
}

// =============================================================================
// PAYMENT TYPES
// =============================================================================

export interface Payment {
  id: string;
  orderId: string;
  paymentMethod: string;
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'cancelled' | 'refunded';
  amount: number;
  currency: string;
  transactionId?: string;
  gatewayResponse?: Record<string, any>;
  refundAmount: number;
  refundReason?: string;
  createdAt: string;
  updatedAt: string;
}

// =============================================================================
// SHIPPING TYPES
// =============================================================================

export interface ShippingZone {
  id: string;
  name: string;
  countries: string[];
  regions: string[];
  postalCodes: string[];
  isDefault: boolean;
  position: number;
  createdAt: string;
  methods?: ShippingMethod[];
}

export type ShippingMethodType = 'flat_rate' | 'free_shipping' | 'weight_based' | 'price_based';

export interface ShippingMethod {
  id: string;
  zoneId: string;
  name: string;
  methodType: ShippingMethodType;
  cost: number;
  freeThreshold?: number;
  minWeight?: number;
  maxWeight?: number;
  settings: Record<string, any>;
  enabled: boolean;
  position: number;
  createdAt: string;
}

// =============================================================================
// TAX TYPES
// =============================================================================

export interface TaxRate {
  id: string;
  name: string;
  rate: number;                       // Decimal: 0.0825 = 8.25%
  country: string;
  state?: string;
  postalCode?: string;
  city?: string;
  taxClass: string;
  compound: boolean;
  shipping: boolean;
  priority: number;
  enabled: boolean;
  createdAt: string;
}

// =============================================================================
// SETTINGS TYPES
// =============================================================================

export interface StoreSettings {
  storeName: string;
  storeUrl: string;
  currency: string;                   // ISO 4217: USD, EUR, GBP
  currencySymbol: string;
  currencyPosition: 'before' | 'after';
  thousandSeparator: string;
  decimalSeparator: string;
  decimalPlaces: number;
  weightUnit: 'kg' | 'lb' | 'oz' | 'g';
  dimensionUnit: 'cm' | 'in' | 'mm' | 'm';
  taxEnabled: boolean;
  taxIncludedInPrice: boolean;
  shippingEnabled: boolean;
  guestCheckout: boolean;
  reviewsEnabled: boolean;
  wishlistEnabled: boolean;
}

export interface PaymentGatewaySettings {
  stripeEnabled: boolean;
  stripePublishableKey: string;
  stripeSecretKey: string;            // Write-only: backend never returns the full key
  stripeWebhookSecret: string;        // Write-only
  testMode: boolean;
}

export interface CheckoutConfig {
  guestCheckout: boolean;
  createAccountDuringCheckout: boolean;
  requirePhone: boolean;
  requireCompany: boolean;
  termsAndConditionsUrl?: string;
  privacyPolicyUrl?: string;
  stockReservationMinutes: number;    // Default: 10
}

// =============================================================================
// ANALYTICS TYPES
// =============================================================================

export interface AnalyticsSummary {
  totalRevenue: number;
  revenueChange: number;              // Percentage change vs prior period
  totalOrders: number;
  ordersChange: number;
  totalCustomers: number;
  customersChange: number;
  averageOrderValue: number;
  aovChange: number;
}

export interface AnalyticsResponse {
  summary: AnalyticsSummary;
  revenueChart: Array<{
    date: string;
    revenue: number;
    orders: number;
  }>;
  ordersByStatus: Array<{
    status: OrderStatus;
    count: number;
  }>;
  topProducts: Array<{
    id: string;
    name: string;
    imageUrl?: string;
    unitsSold: number;
    revenue: number;
  }>;
  recentOrders: Array<{
    id: string;
    orderNumber: string;
    customerName: string;
    total: number;
    status: OrderStatus;
    createdAt: string;
  }>;
  lowStockProducts: Array<{
    id: string;
    name: string;
    sku: string;
    stockQuantity: number;
    lowStockThreshold: number;
  }>;
}

// =============================================================================
// INVENTORY TYPES
// =============================================================================

export interface InventoryReport {
  productId: string;
  productName: string;
  sku: string;
  stockQuantity: number;
  stockStatus: StockStatus;
  lowStockThreshold: number;
  reservedQuantity: number;           // Held during checkout
  availableQuantity: number;          // stockQuantity - reservedQuantity
}

// =============================================================================
// COUPON TYPES (P1)
// =============================================================================

export type DiscountType = 'percentage' | 'fixed_cart' | 'fixed_product' | 'free_shipping';

export interface Coupon {
  id: string;
  code: string;
  description?: string;
  discountType: DiscountType;
  discountValue: number;
  minimumSpend?: number;
  maximumSpend?: number;
  usageLimit?: number;
  usageCount: number;
  usageLimitPerUser?: number;
  productIds: string[];
  categoryIds: string[];
  excludedProductIds: string[];
  startsAt?: string;
  expiresAt?: string;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}
```

---

## 4. Error Handling Patterns

### 4.1 API Error Structure

The backend returns errors in this format:

```json
{
  "message": "Product not found",
  "code": "PRODUCT_NOT_FOUND",
  "statusCode": 404
}
```

For validation errors:

```json
{
  "message": "Validation failed",
  "code": "VALIDATION_ERROR",
  "statusCode": 422,
  "details": {
    "name": ["Product name is required"],
    "price": ["Price must be greater than 0"],
    "sku": ["SKU already exists"]
  }
}
```

### 4.2 Error Handling in the Store

```typescript
// In the Zustand store:
fetchProducts: async (params) => {
  set({ productsLoading: true, productsError: null });
  try {
    const response = await commerceApi.listProducts(params);
    set({
      products: response.data.items,
      productsTotal: response.data.total,
      productsLoading: false,
    });
  } catch (error: any) {
    const message = error.response?.data?.message ?? 'Failed to load products';
    set({ productsError: message, productsLoading: false });
  }
},

// For mutations that the component handles directly:
createProduct: async (data) => {
  const response = await commerceApi.createProduct(data);
  // ... update state ...
  return response.data;
  // Error is thrown to the component for toast/form handling
},
```

### 4.3 Error Handling in Components

```typescript
// Component-level error handling:
import { useToast } from '@/design-system';

function ProductEditor() {
  const toast = useToast();
  const createProduct = useCommerceStore((s) => s.createProduct);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string[]>>({});

  const handleSave = async (formData: Partial<Product>) => {
    setFieldErrors({});
    try {
      const product = await createProduct(formData);
      toast({ type: 'success', message: 'Product created successfully' });
      navigate(`/store/products/${product.id}`);
    } catch (error: any) {
      if (error.response?.status === 422) {
        // Validation error: show field-level errors
        setFieldErrors(error.response.data.details ?? {});
        toast({ type: 'error', message: 'Please fix the validation errors' });
      } else {
        // Generic error
        toast({ type: 'error', message: error.response?.data?.message ?? 'Failed to save product' });
      }
    }
  };
}
```

### 4.4 Error Codes Reference

| Code | HTTP Status | Description |
|------|------------|-------------|
| `VALIDATION_ERROR` | 422 | Request body failed validation |
| `PRODUCT_NOT_FOUND` | 404 | Product ID does not exist |
| `ORDER_NOT_FOUND` | 404 | Order ID does not exist |
| `CUSTOMER_NOT_FOUND` | 404 | Customer ID does not exist |
| `SKU_ALREADY_EXISTS` | 409 | Product SKU must be unique |
| `SLUG_ALREADY_EXISTS` | 409 | Product slug must be unique |
| `INVALID_STATUS_TRANSITION` | 400 | Order status cannot transition to requested state |
| `INSUFFICIENT_STOCK` | 400 | Not enough stock for the operation |
| `PAYMENT_FAILED` | 402 | Stripe payment failed |
| `REFUND_FAILED` | 400 | Stripe refund failed |
| `STRIPE_NOT_CONFIGURED` | 500 | Stripe keys not set in settings |
| `UNAUTHORIZED` | 401 | JWT token invalid or expired |
| `FORBIDDEN` | 403 | User lacks required permission |
| `RATE_LIMITED` | 429 | Too many requests |

---

## 5. Loading and Error States Management

### 5.1 Loading States per Slice

Each slice maintains its own loading flags:

```typescript
// Products
productsLoading: boolean;        // List loading
currentProductLoading: boolean;  // Detail loading

// Orders
ordersLoading: boolean;
currentOrderLoading: boolean;

// Customers
customersLoading: boolean;
currentCustomerLoading: boolean;

// Analytics
analyticsLoading: boolean;

// Settings
settingsLoading: boolean;
settingsSaving: boolean;         // Separate flag for save operations
```

### 5.2 Loading State in Components

```typescript
function ProductList() {
  const loading = useCommerceStore((s) => s.productsLoading);
  const error = useCommerceStore((s) => s.productsError);
  const products = useCommerceStore((s) => s.products);

  if (loading) {
    return <SkeletonTable rows={10} columns={8} />;
  }

  if (error) {
    return (
      <Alert variant="danger" title="Error loading products">
        {error}
        <Button variant="outline" onClick={fetchProducts}>Retry</Button>
      </Alert>
    );
  }

  if (products.length === 0) {
    return (
      <EmptyState
        icon={Package}
        title="No products yet"
        description="Create your first product to get started."
        action={{ label: 'Add Product', onClick: () => navigate('/store/products/new') }}
      />
    );
  }

  return <DataTable data={products} columns={columns} />;
}
```

### 5.3 Inline Saving States

For mutation operations, use local component state:

```typescript
function GeneralSettings() {
  const settings = useCommerceStore((s) => s.storeSettings);
  const updateSettings = useCommerceStore((s) => s.updateSettings);
  const [saving, setSaving] = useState(false);

  const handleSave = async (data: Partial<StoreSettings>) => {
    setSaving(true);
    try {
      await updateSettings(data);
      toast({ type: 'success', message: 'Settings saved' });
    } catch (error) {
      toast({ type: 'error', message: 'Failed to save settings' });
    } finally {
      setSaving(false);
    }
  };

  return (
    <Card>
      {/* form fields */}
      <Button loading={saving} onClick={() => handleSave(formData)}>
        Save Settings
      </Button>
    </Card>
  );
}
```

---

## 6. API Endpoint Summary

| Method | Endpoint | Description |
|--------|----------|-------------|
| **Products** | | |
| GET | `/v1/rustcommerce/admin/products` | List products (paginated, filterable) |
| GET | `/v1/rustcommerce/admin/products/:id` | Get product with variants, images, categories |
| POST | `/v1/rustcommerce/admin/products` | Create product |
| PUT | `/v1/rustcommerce/admin/products/:id` | Update product |
| DELETE | `/v1/rustcommerce/admin/products/:id` | Delete product |
| POST | `/v1/rustcommerce/admin/products/:id/variants` | Add variant |
| PUT | `/v1/rustcommerce/admin/products/:id/variants/:vid` | Update variant |
| DELETE | `/v1/rustcommerce/admin/products/:id/variants/:vid` | Delete variant |
| POST | `/v1/rustcommerce/admin/products/:id/images` | Upload images (multipart) |
| DELETE | `/v1/rustcommerce/admin/products/:id/images/:iid` | Delete image |
| PUT | `/v1/rustcommerce/admin/products/:id/images/:iid/primary` | Set primary image |
| PUT | `/v1/rustcommerce/admin/products/:id/images/reorder` | Reorder images |
| **Categories** | | |
| GET | `/v1/rustcommerce/admin/categories` | List categories (tree) |
| POST | `/v1/rustcommerce/admin/categories` | Create category |
| PUT | `/v1/rustcommerce/admin/categories/:id` | Update category |
| DELETE | `/v1/rustcommerce/admin/categories/:id` | Delete category |
| **Orders** | | |
| GET | `/v1/rustcommerce/admin/orders` | List orders (paginated, filterable) |
| GET | `/v1/rustcommerce/admin/orders/:id` | Get order with items, customer, payment |
| PUT | `/v1/rustcommerce/admin/orders/:id` | Update order status |
| POST | `/v1/rustcommerce/admin/orders/:id/notes` | Add admin note |
| POST | `/v1/rustcommerce/admin/orders/:id/refund` | Initiate refund |
| GET | `/v1/rustcommerce/admin/orders/export` | Export orders (CSV/PDF blob) |
| **Customers** | | |
| GET | `/v1/rustcommerce/admin/customers` | List customers (paginated) |
| GET | `/v1/rustcommerce/admin/customers/:id` | Get customer with addresses |
| PUT | `/v1/rustcommerce/admin/customers/:id` | Update customer |
| DELETE | `/v1/rustcommerce/admin/customers/:id` | Delete customer |
| GET | `/v1/rustcommerce/admin/customers/:id/orders` | Get customer orders |
| GET | `/v1/rustcommerce/admin/customers/export` | Export customers (CSV blob) |
| **Analytics** | | |
| GET | `/v1/rustcommerce/admin/analytics` | Dashboard summary with all widgets |
| GET | `/v1/rustcommerce/admin/analytics/revenue` | Revenue time series |
| GET | `/v1/rustcommerce/admin/analytics/top-products` | Best sellers |
| **Inventory** | | |
| GET | `/v1/rustcommerce/admin/inventory` | Inventory report |
| PUT | `/v1/rustcommerce/admin/inventory/:id` | Update stock quantity |
| GET | `/v1/rustcommerce/admin/inventory/low-stock` | Low stock alerts |
| **Settings** | | |
| GET | `/v1/rustcommerce/admin/settings` | Get all settings |
| PUT | `/v1/rustcommerce/admin/settings` | Update general settings |
| GET/POST/PUT/DELETE | `/v1/rustcommerce/admin/settings/shipping/zones` | Shipping zone CRUD |
| POST/PUT/DELETE | `/v1/rustcommerce/admin/settings/shipping/methods` | Shipping method CRUD |
| GET/POST/PUT/DELETE | `/v1/rustcommerce/admin/settings/tax/rates` | Tax rate CRUD |
| POST | `/v1/rustcommerce/admin/settings/payments/test` | Test Stripe connection |
| **Coupons (P1)** | | |
| GET | `/v1/rustcommerce/admin/coupons` | List coupons |
| POST | `/v1/rustcommerce/admin/coupons` | Create coupon |
| PUT | `/v1/rustcommerce/admin/coupons/:id` | Update coupon |
| DELETE | `/v1/rustcommerce/admin/coupons/:id` | Delete coupon |
