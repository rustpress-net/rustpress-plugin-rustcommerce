# State Management Design -- RustCommerce Admin UI

**Document Version**: 1.0
**Date**: 2026-02-24
**Owner**: Frontend Lead
**Status**: Approved

---

## 1. Overview

RustCommerce uses **Zustand 5.0** for state management, following the same patterns established in the RustPress admin UI (e.g., `pluginStore.ts`, `queueManagerStore.ts`). The store is organized into logical slices combined into a single `useCommerceStore` hook.

**File**: `src/pages/plugins/rustcommerce/stores/commerceStore.ts`

---

## 2. Store Architecture

```
useCommerceStore
|
|-- products slice       # Product catalog state
|-- orders slice         # Order management state
|-- customers slice      # Customer data state
|-- analytics slice      # Dashboard analytics state
|-- settings slice       # Store configuration state
|-- ui slice             # UI-specific transient state
```

---

## 3. Store Definition

```typescript
// src/pages/plugins/rustcommerce/stores/commerceStore.ts

import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { commerceApi } from '../api/commerceApi';
import type {
  Product,
  ProductVariant,
  ProductImage,
  Category,
  Order,
  OrderStatus,
  Customer,
  StoreSettings,
  ShippingZone,
  ShippingMethod,
  TaxRate,
  AnalyticsSummary,
  PaginatedResponse,
  ListParams,
} from '../types';

// =============================================================================
// SLICE INTERFACES
// =============================================================================

interface ProductsSlice {
  products: Product[];
  productsTotal: number;
  productsPage: number;
  productsPageSize: number;
  productsLoading: boolean;
  productsError: string | null;
  currentProduct: Product | null;
  currentProductLoading: boolean;
  productCategories: Category[];

  fetchProducts: (params?: ListParams) => Promise<void>;
  fetchProduct: (id: string) => Promise<void>;
  createProduct: (data: Partial<Product>) => Promise<Product>;
  updateProduct: (id: string, data: Partial<Product>) => Promise<Product>;
  deleteProduct: (id: string) => Promise<void>;
  deleteProducts: (ids: string[]) => Promise<void>;
  updateProductStatus: (ids: string[], status: string) => Promise<void>;
  fetchCategories: () => Promise<void>;
  setProductsPage: (page: number) => void;
  setProductsPageSize: (size: number) => void;
  clearCurrentProduct: () => void;

  // Variant operations
  addVariant: (productId: string, variant: Partial<ProductVariant>) => Promise<void>;
  updateVariant: (productId: string, variantId: string, data: Partial<ProductVariant>) => Promise<void>;
  deleteVariant: (productId: string, variantId: string) => Promise<void>;

  // Image operations
  uploadProductImages: (productId: string, files: File[]) => Promise<void>;
  deleteProductImage: (productId: string, imageId: string) => Promise<void>;
  setPrimaryImage: (productId: string, imageId: string) => Promise<void>;
  reorderProductImages: (productId: string, imageIds: string[]) => Promise<void>;
}

interface OrdersSlice {
  orders: Order[];
  ordersTotal: number;
  ordersPage: number;
  ordersPageSize: number;
  ordersLoading: boolean;
  ordersError: string | null;
  currentOrder: Order | null;
  currentOrderLoading: boolean;

  fetchOrders: (params?: ListParams) => Promise<void>;
  fetchOrder: (id: string) => Promise<void>;
  updateOrderStatus: (id: string, status: OrderStatus, note?: string) => Promise<void>;
  addOrderNote: (id: string, note: string) => Promise<void>;
  refundOrder: (id: string, amount: number, reason: string) => Promise<void>;
  setOrdersPage: (page: number) => void;
  setOrdersPageSize: (size: number) => void;
  clearCurrentOrder: () => void;
}

interface CustomersSlice {
  customers: Customer[];
  customersTotal: number;
  customersPage: number;
  customersPageSize: number;
  customersLoading: boolean;
  customersError: string | null;
  currentCustomer: Customer | null;
  currentCustomerLoading: boolean;

  fetchCustomers: (params?: ListParams) => Promise<void>;
  fetchCustomer: (id: string) => Promise<void>;
  setCustomersPage: (page: number) => void;
  setCustomersPageSize: (size: number) => void;
  clearCurrentCustomer: () => void;
}

interface AnalyticsSlice {
  analytics: AnalyticsSummary | null;
  analyticsLoading: boolean;
  analyticsPeriod: 'today' | '7d' | '30d' | '90d' | 'year';
  revenueChartData: Array<{ date: string; revenue: number; orders: number }>;
  orderStatusData: Array<{ status: OrderStatus; count: number }>;
  topProducts: Array<{ id: string; name: string; imageUrl?: string; unitsSold: number; revenue: number }>;
  recentOrders: Array<{ id: string; orderNumber: string; customerName: string; total: number; status: OrderStatus; createdAt: string }>;
  lowStockProducts: Array<{ id: string; name: string; sku: string; stockQuantity: number; lowStockThreshold: number }>;

  fetchAnalytics: (period?: string) => Promise<void>;
  setAnalyticsPeriod: (period: 'today' | '7d' | '30d' | '90d' | 'year') => void;
}

interface SettingsSlice {
  storeSettings: StoreSettings | null;
  shippingZones: ShippingZone[];
  shippingMethods: ShippingMethod[];
  taxRates: TaxRate[];
  settingsLoading: boolean;
  settingsSaving: boolean;
  settingsError: string | null;

  fetchSettings: () => Promise<void>;
  updateSettings: (data: Partial<StoreSettings>) => Promise<void>;
  fetchShippingZones: () => Promise<void>;
  createShippingZone: (zone: Partial<ShippingZone>) => Promise<void>;
  updateShippingZone: (id: string, data: Partial<ShippingZone>) => Promise<void>;
  deleteShippingZone: (id: string) => Promise<void>;
  createShippingMethod: (zoneId: string, method: Partial<ShippingMethod>) => Promise<void>;
  updateShippingMethod: (id: string, data: Partial<ShippingMethod>) => Promise<void>;
  deleteShippingMethod: (id: string) => Promise<void>;
  fetchTaxRates: () => Promise<void>;
  createTaxRate: (rate: Partial<TaxRate>) => Promise<void>;
  updateTaxRate: (id: string, data: Partial<TaxRate>) => Promise<void>;
  deleteTaxRate: (id: string) => Promise<void>;
}

interface UISlice {
  sidebarSection: string;
  productListFilters: {
    search: string;
    status: string[];
    category: string;
    stockStatus: string;
  };
  orderListFilters: {
    search: string;
    status: string[];
    paymentStatus: string[];
    dateRange: { start?: string; end?: string };
  };
  customerListFilters: {
    search: string;
  };
  selectedProductIds: string[];
  selectedOrderIds: string[];

  setSidebarSection: (section: string) => void;
  setProductListFilters: (filters: Partial<UISlice['productListFilters']>) => void;
  setOrderListFilters: (filters: Partial<UISlice['orderListFilters']>) => void;
  setCustomerListFilters: (filters: Partial<UISlice['customerListFilters']>) => void;
  setSelectedProductIds: (ids: string[]) => void;
  setSelectedOrderIds: (ids: string[]) => void;
  clearAllFilters: () => void;
}

// =============================================================================
// COMBINED STORE TYPE
// =============================================================================

type CommerceState =
  & ProductsSlice
  & OrdersSlice
  & CustomersSlice
  & AnalyticsSlice
  & SettingsSlice
  & UISlice;
```

---

## 4. Store Implementation Pattern

```typescript
export const useCommerceStore = create<CommerceState>()(
  persist(
    (set, get) => ({
      // =====================================================================
      // PRODUCTS SLICE
      // =====================================================================
      products: [],
      productsTotal: 0,
      productsPage: 1,
      productsPageSize: 20,
      productsLoading: false,
      productsError: null,
      currentProduct: null,
      currentProductLoading: false,
      productCategories: [],

      fetchProducts: async (params) => {
        set({ productsLoading: true, productsError: null });
        try {
          const { page, pageSize, productListFilters } = get();
          const response = await commerceApi.listProducts({
            page: params?.page ?? page,
            pageSize: params?.pageSize ?? pageSize,
            search: params?.search ?? productListFilters.search,
            status: params?.status ?? productListFilters.status,
            ...params,
          });
          set({
            products: response.data.items,
            productsTotal: response.data.total,
            productsLoading: false,
          });
        } catch (error: any) {
          set({
            productsError: error.response?.data?.message ?? 'Failed to load products',
            productsLoading: false,
          });
        }
      },

      fetchProduct: async (id) => {
        set({ currentProductLoading: true });
        try {
          const response = await commerceApi.getProduct(id);
          set({ currentProduct: response.data, currentProductLoading: false });
        } catch (error: any) {
          set({ currentProductLoading: false });
          throw error;
        }
      },

      createProduct: async (data) => {
        const response = await commerceApi.createProduct(data);
        const newProduct = response.data;
        set((state) => ({
          products: [newProduct, ...state.products],
          productsTotal: state.productsTotal + 1,
        }));
        return newProduct;
      },

      updateProduct: async (id, data) => {
        // Optimistic update
        const previous = get().currentProduct;
        if (previous) {
          set({ currentProduct: { ...previous, ...data } as Product });
        }
        try {
          const response = await commerceApi.updateProduct(id, data);
          const updated = response.data;
          set((state) => ({
            currentProduct: updated,
            products: state.products.map((p) => (p.id === id ? updated : p)),
          }));
          return updated;
        } catch (error) {
          // Rollback optimistic update
          set({ currentProduct: previous });
          throw error;
        }
      },

      deleteProduct: async (id) => {
        // Optimistic removal
        const previousProducts = get().products;
        set((state) => ({
          products: state.products.filter((p) => p.id !== id),
          productsTotal: state.productsTotal - 1,
        }));
        try {
          await commerceApi.deleteProduct(id);
        } catch (error) {
          // Rollback
          set({ products: previousProducts, productsTotal: previousProducts.length });
          throw error;
        }
      },

      deleteProducts: async (ids) => {
        const previousProducts = get().products;
        set((state) => ({
          products: state.products.filter((p) => !ids.includes(p.id)),
          productsTotal: state.productsTotal - ids.length,
          selectedProductIds: [],
        }));
        try {
          await Promise.all(ids.map((id) => commerceApi.deleteProduct(id)));
        } catch (error) {
          set({ products: previousProducts, productsTotal: previousProducts.length });
          throw error;
        }
      },

      updateProductStatus: async (ids, status) => {
        const previousProducts = get().products;
        // Optimistic update
        set((state) => ({
          products: state.products.map((p) =>
            ids.includes(p.id) ? { ...p, status } : p
          ),
        }));
        try {
          await Promise.all(ids.map((id) => commerceApi.updateProduct(id, { status })));
        } catch (error) {
          set({ products: previousProducts });
          throw error;
        }
      },

      fetchCategories: async () => {
        try {
          const response = await commerceApi.listCategories();
          set({ productCategories: response.data });
        } catch (error) {
          console.error('Failed to load categories:', error);
        }
      },

      setProductsPage: (page) => {
        set({ productsPage: page });
        get().fetchProducts({ page });
      },

      setProductsPageSize: (size) => {
        set({ productsPageSize: size, productsPage: 1 });
        get().fetchProducts({ page: 1, pageSize: size });
      },

      clearCurrentProduct: () => set({ currentProduct: null }),

      addVariant: async (productId, variant) => {
        const response = await commerceApi.addProductVariant(productId, variant);
        set((state) => {
          if (state.currentProduct?.id === productId) {
            return {
              currentProduct: {
                ...state.currentProduct,
                variants: [...(state.currentProduct.variants ?? []), response.data],
              },
            };
          }
          return {};
        });
      },

      updateVariant: async (productId, variantId, data) => {
        const response = await commerceApi.updateProductVariant(productId, variantId, data);
        set((state) => {
          if (state.currentProduct?.id === productId) {
            return {
              currentProduct: {
                ...state.currentProduct,
                variants: state.currentProduct.variants?.map((v) =>
                  v.id === variantId ? response.data : v
                ),
              },
            };
          }
          return {};
        });
      },

      deleteVariant: async (productId, variantId) => {
        await commerceApi.deleteProductVariant(productId, variantId);
        set((state) => {
          if (state.currentProduct?.id === productId) {
            return {
              currentProduct: {
                ...state.currentProduct,
                variants: state.currentProduct.variants?.filter((v) => v.id !== variantId),
              },
            };
          }
          return {};
        });
      },

      uploadProductImages: async (productId, files) => {
        const response = await commerceApi.uploadProductImages(productId, files);
        set((state) => {
          if (state.currentProduct?.id === productId) {
            return {
              currentProduct: {
                ...state.currentProduct,
                images: [...(state.currentProduct.images ?? []), ...response.data],
              },
            };
          }
          return {};
        });
      },

      deleteProductImage: async (productId, imageId) => {
        await commerceApi.deleteProductImage(productId, imageId);
        set((state) => {
          if (state.currentProduct?.id === productId) {
            return {
              currentProduct: {
                ...state.currentProduct,
                images: state.currentProduct.images?.filter((i) => i.id !== imageId),
              },
            };
          }
          return {};
        });
      },

      setPrimaryImage: async (productId, imageId) => {
        await commerceApi.setPrimaryProductImage(productId, imageId);
        set((state) => {
          if (state.currentProduct?.id === productId) {
            return {
              currentProduct: {
                ...state.currentProduct,
                images: state.currentProduct.images?.map((i) => ({
                  ...i,
                  isPrimary: i.id === imageId,
                })),
              },
            };
          }
          return {};
        });
      },

      reorderProductImages: async (productId, imageIds) => {
        await commerceApi.reorderProductImages(productId, imageIds);
        // Re-fetch to get updated positions
        get().fetchProduct(productId);
      },

      // =====================================================================
      // ORDERS SLICE
      // =====================================================================
      orders: [],
      ordersTotal: 0,
      ordersPage: 1,
      ordersPageSize: 20,
      ordersLoading: false,
      ordersError: null,
      currentOrder: null,
      currentOrderLoading: false,

      fetchOrders: async (params) => {
        set({ ordersLoading: true, ordersError: null });
        try {
          const { ordersPage, ordersPageSize, orderListFilters } = get();
          const response = await commerceApi.listOrders({
            page: params?.page ?? ordersPage,
            pageSize: params?.pageSize ?? ordersPageSize,
            search: params?.search ?? orderListFilters.search,
            status: params?.status ?? orderListFilters.status,
            ...params,
          });
          set({
            orders: response.data.items,
            ordersTotal: response.data.total,
            ordersLoading: false,
          });
        } catch (error: any) {
          set({
            ordersError: error.response?.data?.message ?? 'Failed to load orders',
            ordersLoading: false,
          });
        }
      },

      fetchOrder: async (id) => {
        set({ currentOrderLoading: true });
        try {
          const response = await commerceApi.getOrder(id);
          set({ currentOrder: response.data, currentOrderLoading: false });
        } catch (error) {
          set({ currentOrderLoading: false });
          throw error;
        }
      },

      updateOrderStatus: async (id, status, note) => {
        const previousOrder = get().currentOrder;
        // Optimistic update
        if (previousOrder?.id === id) {
          set({ currentOrder: { ...previousOrder, status } as Order });
        }
        try {
          const response = await commerceApi.updateOrderStatus(id, status, note);
          set((state) => ({
            currentOrder: response.data,
            orders: state.orders.map((o) =>
              o.id === id ? { ...o, status } : o
            ),
          }));
        } catch (error) {
          set({ currentOrder: previousOrder });
          throw error;
        }
      },

      addOrderNote: async (id, note) => {
        await commerceApi.addOrderNote(id, note);
        get().fetchOrder(id);
      },

      refundOrder: async (id, amount, reason) => {
        await commerceApi.refundOrder(id, amount, reason);
        get().fetchOrder(id);
      },

      setOrdersPage: (page) => {
        set({ ordersPage: page });
        get().fetchOrders({ page });
      },

      setOrdersPageSize: (size) => {
        set({ ordersPageSize: size, ordersPage: 1 });
        get().fetchOrders({ page: 1, pageSize: size });
      },

      clearCurrentOrder: () => set({ currentOrder: null }),

      // =====================================================================
      // CUSTOMERS SLICE
      // =====================================================================
      customers: [],
      customersTotal: 0,
      customersPage: 1,
      customersPageSize: 20,
      customersLoading: false,
      customersError: null,
      currentCustomer: null,
      currentCustomerLoading: false,

      fetchCustomers: async (params) => {
        set({ customersLoading: true, customersError: null });
        try {
          const { customersPage, customersPageSize, customerListFilters } = get();
          const response = await commerceApi.listCustomers({
            page: params?.page ?? customersPage,
            pageSize: params?.pageSize ?? customersPageSize,
            search: params?.search ?? customerListFilters.search,
            ...params,
          });
          set({
            customers: response.data.items,
            customersTotal: response.data.total,
            customersLoading: false,
          });
        } catch (error: any) {
          set({
            customersError: error.response?.data?.message ?? 'Failed to load customers',
            customersLoading: false,
          });
        }
      },

      fetchCustomer: async (id) => {
        set({ currentCustomerLoading: true });
        try {
          const response = await commerceApi.getCustomer(id);
          set({ currentCustomer: response.data, currentCustomerLoading: false });
        } catch (error) {
          set({ currentCustomerLoading: false });
          throw error;
        }
      },

      setCustomersPage: (page) => {
        set({ customersPage: page });
        get().fetchCustomers({ page });
      },

      setCustomersPageSize: (size) => {
        set({ customersPageSize: size, customersPage: 1 });
        get().fetchCustomers({ page: 1, pageSize: size });
      },

      clearCurrentCustomer: () => set({ currentCustomer: null }),

      // =====================================================================
      // ANALYTICS SLICE
      // =====================================================================
      analytics: null,
      analyticsLoading: false,
      analyticsPeriod: '30d',
      revenueChartData: [],
      orderStatusData: [],
      topProducts: [],
      recentOrders: [],
      lowStockProducts: [],

      fetchAnalytics: async (period) => {
        const activePeriod = period ?? get().analyticsPeriod;
        set({ analyticsLoading: true });
        try {
          const response = await commerceApi.getAnalytics({ period: activePeriod });
          set({
            analytics: response.data.summary,
            revenueChartData: response.data.revenueChart,
            orderStatusData: response.data.ordersByStatus,
            topProducts: response.data.topProducts,
            recentOrders: response.data.recentOrders,
            lowStockProducts: response.data.lowStockProducts,
            analyticsLoading: false,
          });
        } catch (error) {
          set({ analyticsLoading: false });
        }
      },

      setAnalyticsPeriod: (period) => {
        set({ analyticsPeriod: period });
        get().fetchAnalytics(period);
      },

      // =====================================================================
      // SETTINGS SLICE
      // =====================================================================
      storeSettings: null,
      shippingZones: [],
      shippingMethods: [],
      taxRates: [],
      settingsLoading: false,
      settingsSaving: false,
      settingsError: null,

      fetchSettings: async () => {
        set({ settingsLoading: true });
        try {
          const response = await commerceApi.getSettings();
          set({
            storeSettings: response.data.general,
            shippingZones: response.data.shippingZones,
            shippingMethods: response.data.shippingMethods,
            taxRates: response.data.taxRates,
            settingsLoading: false,
          });
        } catch (error: any) {
          set({
            settingsError: error.response?.data?.message ?? 'Failed to load settings',
            settingsLoading: false,
          });
        }
      },

      updateSettings: async (data) => {
        set({ settingsSaving: true });
        try {
          await commerceApi.updateSettings(data);
          set((state) => ({
            storeSettings: { ...state.storeSettings, ...data } as StoreSettings,
            settingsSaving: false,
          }));
        } catch (error) {
          set({ settingsSaving: false });
          throw error;
        }
      },

      fetchShippingZones: async () => {
        const response = await commerceApi.listShippingZones();
        set({ shippingZones: response.data });
      },

      createShippingZone: async (zone) => {
        const response = await commerceApi.createShippingZone(zone);
        set((state) => ({
          shippingZones: [...state.shippingZones, response.data],
        }));
      },

      updateShippingZone: async (id, data) => {
        const response = await commerceApi.updateShippingZone(id, data);
        set((state) => ({
          shippingZones: state.shippingZones.map((z) =>
            z.id === id ? response.data : z
          ),
        }));
      },

      deleteShippingZone: async (id) => {
        await commerceApi.deleteShippingZone(id);
        set((state) => ({
          shippingZones: state.shippingZones.filter((z) => z.id !== id),
        }));
      },

      createShippingMethod: async (zoneId, method) => {
        const response = await commerceApi.createShippingMethod(zoneId, method);
        set((state) => ({
          shippingMethods: [...state.shippingMethods, response.data],
        }));
      },

      updateShippingMethod: async (id, data) => {
        const response = await commerceApi.updateShippingMethod(id, data);
        set((state) => ({
          shippingMethods: state.shippingMethods.map((m) =>
            m.id === id ? response.data : m
          ),
        }));
      },

      deleteShippingMethod: async (id) => {
        await commerceApi.deleteShippingMethod(id);
        set((state) => ({
          shippingMethods: state.shippingMethods.filter((m) => m.id !== id),
        }));
      },

      fetchTaxRates: async () => {
        const response = await commerceApi.listTaxRates();
        set({ taxRates: response.data });
      },

      createTaxRate: async (rate) => {
        const response = await commerceApi.createTaxRate(rate);
        set((state) => ({
          taxRates: [...state.taxRates, response.data],
        }));
      },

      updateTaxRate: async (id, data) => {
        const response = await commerceApi.updateTaxRate(id, data);
        set((state) => ({
          taxRates: state.taxRates.map((r) =>
            r.id === id ? response.data : r
          ),
        }));
      },

      deleteTaxRate: async (id) => {
        await commerceApi.deleteTaxRate(id);
        set((state) => ({
          taxRates: state.taxRates.filter((r) => r.id !== id),
        }));
      },

      // =====================================================================
      // UI SLICE
      // =====================================================================
      sidebarSection: 'dashboard',
      productListFilters: {
        search: '',
        status: [],
        category: '',
        stockStatus: '',
      },
      orderListFilters: {
        search: '',
        status: [],
        paymentStatus: [],
        dateRange: {},
      },
      customerListFilters: {
        search: '',
      },
      selectedProductIds: [],
      selectedOrderIds: [],

      setSidebarSection: (section) => set({ sidebarSection: section }),

      setProductListFilters: (filters) =>
        set((state) => ({
          productListFilters: { ...state.productListFilters, ...filters },
          productsPage: 1, // Reset to page 1 on filter change
        })),

      setOrderListFilters: (filters) =>
        set((state) => ({
          orderListFilters: { ...state.orderListFilters, ...filters },
          ordersPage: 1,
        })),

      setCustomerListFilters: (filters) =>
        set((state) => ({
          customerListFilters: { ...state.customerListFilters, ...filters },
          customersPage: 1,
        })),

      setSelectedProductIds: (ids) => set({ selectedProductIds: ids }),
      setSelectedOrderIds: (ids) => set({ selectedOrderIds: ids }),

      clearAllFilters: () =>
        set({
          productListFilters: { search: '', status: [], category: '', stockStatus: '' },
          orderListFilters: { search: '', status: [], paymentStatus: [], dateRange: {} },
          customerListFilters: { search: '' },
        }),
    }),

    // Persistence configuration (see Section 5)
    {
      name: 'rustcommerce-store',
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        // Only persist UI preferences and pagination settings
        analyticsPeriod: state.analyticsPeriod,
        productsPageSize: state.productsPageSize,
        ordersPageSize: state.ordersPageSize,
        customersPageSize: state.customersPageSize,
        productListFilters: state.productListFilters,
        orderListFilters: state.orderListFilters,
        customerListFilters: state.customerListFilters,
        sidebarSection: state.sidebarSection,
      }),
    }
  )
);
```

---

## 5. Persistence Strategy

### 5.1 What Gets Persisted (localStorage)

These values represent user preferences and should survive page reloads:

| Key | Why Persist |
|-----|-------------|
| `analyticsPeriod` | User's preferred dashboard time range |
| `productsPageSize` | User's preferred items-per-page |
| `ordersPageSize` | User's preferred items-per-page |
| `customersPageSize` | User's preferred items-per-page |
| `productListFilters` | Active filters should survive navigation |
| `orderListFilters` | Active filters should survive navigation |
| `customerListFilters` | Active filters should survive navigation |
| `sidebarSection` | Remember last active section |

### 5.2 What Gets Fetched Fresh (NOT persisted)

These are volatile data that must always come from the server:

| Key | Why Fetch Fresh |
|-----|----------------|
| `products`, `orders`, `customers` | Data changes frequently; stale lists are misleading |
| `currentProduct`, `currentOrder`, `currentCustomer` | Must reflect latest state |
| `analytics`, `revenueChartData`, etc. | Real-time metrics must be current |
| `storeSettings`, `shippingZones`, `taxRates` | Config changes affect the whole store |
| All `*Loading`, `*Error` states | Transient; should reset on page load |
| `selectedProductIds`, `selectedOrderIds` | Selection is ephemeral |

### 5.3 Storage Key

```
localStorage key: "rustcommerce-store"
```

Namespaced to avoid collision with the main RustPress admin store and other plugin stores.

---

## 6. Optimistic Updates

Optimistic updates are applied for operations where immediate UI feedback improves the user experience. The pattern is:

1. Save the previous state
2. Apply the change to the local store immediately
3. Send the API request
4. On success: update with server response (which may differ, e.g., `updated_at` timestamp)
5. On failure: rollback to previous state and throw the error for the UI to show a toast

### 6.1 Operations With Optimistic Updates

| Operation | Optimistic Behavior |
|-----------|-------------------|
| `updateProduct` | Update `currentProduct` immediately, rollback on error |
| `deleteProduct` | Remove from `products` list immediately, rollback on error |
| `deleteProducts` | Remove all selected from list, clear selection, rollback on error |
| `updateProductStatus` | Update status field in list, rollback on error |
| `updateOrderStatus` | Update `currentOrder.status` immediately, rollback on error |

### 6.2 Operations WITHOUT Optimistic Updates

| Operation | Reason |
|-----------|--------|
| `createProduct` | Need server-generated ID, slug, timestamps |
| `refundOrder` | Critical financial operation -- must confirm server success |
| `addVariant` / `updateVariant` | Need server validation of SKU uniqueness |
| `uploadProductImages` | Async file upload -- cannot predict server response |
| Settings mutations | Config changes are infrequent and need server validation |

### 6.3 Error Handling in Optimistic Updates

```typescript
// Pattern used throughout the store:
try {
  const response = await commerceApi.someOperation(data);
  set({ /* update with server response */ });
} catch (error) {
  set({ /* rollback to previousState */ });
  throw error; // Let the component catch and show toast
}

// In the component:
const handleAction = async () => {
  try {
    await store.someOperation(data);
    toast({ type: 'success', message: 'Action completed' });
  } catch (error) {
    toast({ type: 'error', message: error.message ?? 'Action failed' });
  }
};
```

---

## 7. Cache Invalidation Patterns

### 7.1 When to Re-fetch Lists

| Trigger | Action |
|---------|--------|
| Navigate to list page | Always re-fetch (component `useEffect` on mount) |
| Create new entity | Prepend to list locally + increment total |
| Delete entity | Remove from list locally + decrement total |
| Bulk status update | Update items locally, no re-fetch needed |
| Filter/sort change | Re-fetch with new params, reset to page 1 |
| Page change | Re-fetch with new page param |

### 7.2 When to Re-fetch Detail

| Trigger | Action |
|---------|--------|
| Navigate to detail page | Always fetch fresh by ID |
| After refund | Re-fetch order (status/payment may change) |
| After note added | Re-fetch order |
| Leave detail page | Clear `currentProduct` / `currentOrder` / `currentCustomer` |

### 7.3 Dashboard Analytics

| Trigger | Action |
|---------|--------|
| Navigate to dashboard | Fetch analytics for current period |
| Period selector change | Re-fetch with new period |
| Auto-refresh | No auto-refresh (user can manually refresh) |

### 7.4 Settings

| Trigger | Action |
|---------|--------|
| Navigate to settings | Fetch all settings once |
| Save settings | Update local state with response, no re-fetch |
| Create/delete shipping zone or tax rate | Update local array, no full re-fetch |

---

## 8. Selector Hooks

For performance, components should select only the state they need:

```typescript
// Good: select specific fields
const products = useCommerceStore((s) => s.products);
const loading = useCommerceStore((s) => s.productsLoading);
const fetchProducts = useCommerceStore((s) => s.fetchProducts);

// Avoid: selecting the entire store (causes re-renders on any change)
// const store = useCommerceStore();  // BAD

// Derived selectors for computed values
const totalRevenue = useCommerceStore((s) => s.analytics?.totalRevenue ?? 0);
const activeProducts = useCommerceStore((s) =>
  s.products.filter((p) => p.status === 'published')
);
```

---

## 9. Store Initialization Flow

```
1. User navigates to /store (Dashboard)
   -> fetchAnalytics('30d')

2. User navigates to /store/products
   -> fetchProducts({ page: 1, pageSize: persisted })
   -> fetchCategories() (for filter dropdown)

3. User clicks "Edit" on a product
   -> fetchProduct(id)
   -> fetchCategories() (for category picker, if not already loaded)

4. User navigates to /store/orders
   -> fetchOrders({ page: 1, pageSize: persisted })

5. User clicks an order
   -> fetchOrder(id)

6. User navigates to /store/customers
   -> fetchCustomers({ page: 1, pageSize: persisted })

7. User navigates to /store/settings
   -> fetchSettings() (loads general, shipping zones, methods, tax rates)
```

---

## 10. Relationship to Core Stores

The commerce store is independent of the core RustPress stores (`pluginStore`, `navigationStore`, etc.) but can read from them when needed:

```typescript
// Access plugin activation state from pluginStore if needed
import { usePluginStore } from '@/store/pluginStore';

const isCommerceActive = usePluginStore((s) =>
  s.plugins.find((p) => p.id === 'rustcommerce')?.active ?? false
);
```

The commerce store does NOT write to core stores. It is a self-contained state island for the RustCommerce plugin UI.
