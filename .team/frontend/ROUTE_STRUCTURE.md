# Route Structure -- RustCommerce Admin UI

**Document Version**: 1.0
**Date**: 2026-02-24
**Owner**: Frontend Lead
**Status**: Approved

---

## 1. Overview

RustCommerce registers its admin routes under the `/store` path prefix inside the RustPress admin UI. All routes are wrapped in `EnterpriseLayout` (the main admin shell) and lazy-loaded for performance.

---

## 2. All Admin Routes

| Path | Component | Description | Permissions Required |
|------|-----------|-------------|---------------------|
| `/store` | `Dashboard` | Store dashboard with metrics, charts, alerts | `view_store_reports` |
| `/store/products` | `ProductList` | Product catalog listing | `manage_products` |
| `/store/products/new` | `ProductEditor (create)` | Create new product | `manage_products` |
| `/store/products/:id` | `ProductEditor (edit)` | Edit existing product | `manage_products` |
| `/store/orders` | `OrderList` | Order management listing | `manage_orders` |
| `/store/orders/:id` | `OrderDetail` | Single order detail view | `manage_orders` |
| `/store/customers` | `CustomerList` | Customer listing | `manage_customers` |
| `/store/customers/:id` | `CustomerDetail` | Single customer detail | `manage_customers` |
| `/store/settings` | `SettingsLayout` | Store settings (redirects to general) | `manage_store_settings` |
| `/store/settings/general` | `GeneralSettings` | Currency, locale, store info | `manage_store_settings` |
| `/store/settings/payments` | `PaymentSettings` | Stripe gateway configuration | `manage_store_settings` |
| `/store/settings/shipping` | `ShippingSettings` | Shipping zones and methods | `manage_store_settings` |
| `/store/settings/taxes` | `TaxSettings` | Tax rates and zones | `manage_store_settings` |
| `/store/settings/checkout` | `CheckoutSettings` | Checkout flow configuration | `manage_store_settings` |

---

## 3. Route Registration in App.tsx

Add the following to `src/App.tsx` in the `rustpress-core-admin-ui` repository.

### 3.1 Lazy Import

```typescript
// At the top of App.tsx, with other lazy imports:
const RustCommerceAdmin = lazy(() => import('./pages/plugins/rustcommerce'));
```

### 3.2 Route Definition

Inside the `<Route element={<EnterpriseLayout />}>` block, add:

```tsx
{/* RustCommerce Plugin */}
<Route
  path="/store/*"
  element={
    <Suspense fallback={<LoadingSkeleton />}>
      <RustCommerceAdmin />
    </Suspense>
  }
/>
```

### 3.3 Full Context in App.tsx

```tsx
// src/App.tsx (relevant section only)
import { lazy, Suspense } from 'react';
import { Routes, Route } from 'react-router-dom';
import { EnterpriseLayout } from './layouts/EnterpriseLayout';
import { SkeletonTable } from './design-system';

// Lazy imports
const RustCommerceAdmin = lazy(() => import('./pages/plugins/rustcommerce'));
// ... other lazy imports ...

function LoadingSkeleton() {
  return (
    <div className="p-6">
      <SkeletonTable rows={8} columns={6} />
    </div>
  );
}

export default function App() {
  return (
    <Routes>
      {/* Full-screen routes (no admin layout) */}
      <Route path="/login" element={<LoginPage />} />

      {/* Admin routes (wrapped in EnterpriseLayout) */}
      <Route element={<EnterpriseLayout />}>
        {/* Core admin pages */}
        <Route path="/" element={<DashboardPage />} />
        <Route path="/posts/*" element={/* ... */} />
        <Route path="/media/*" element={/* ... */} />

        {/* Plugin management */}
        <Route path="/plugins" element={<PluginsPage />} />
        <Route path="/plugins/add" element={<PluginStorePage />} />
        <Route path="/plugins/:pluginSlug" element={<PluginDetailPage />} />

        {/* === RustCommerce Plugin === */}
        <Route
          path="/store/*"
          element={
            <Suspense fallback={<LoadingSkeleton />}>
              <RustCommerceAdmin />
            </Suspense>
          }
        />

        {/* Other plugin routes */}
        {/* ... */}
      </Route>
    </Routes>
  );
}
```

---

## 4. Internal Route Definitions (index.tsx)

The `RustCommerceAdmin` component (`src/pages/plugins/rustcommerce/index.tsx`) handles its own sub-routing:

```typescript
// src/pages/plugins/rustcommerce/index.tsx
import React, { Suspense, lazy } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { SkeletonTable } from '@/design-system';

// Lazy load all page components for code splitting
const Dashboard = lazy(() => import('./components/dashboard/Dashboard'));
const ProductList = lazy(() => import('./components/products/ProductList'));
const ProductEditor = lazy(() => import('./components/products/ProductEditor'));
const OrderList = lazy(() => import('./components/orders/OrderList'));
const OrderDetail = lazy(() => import('./components/orders/OrderDetail'));
const CustomerList = lazy(() => import('./components/customers/CustomerList'));
const CustomerDetail = lazy(() => import('./components/customers/CustomerDetail'));
const SettingsLayout = lazy(() => import('./components/settings/SettingsLayout'));

function PageFallback() {
  return (
    <div className="p-6 animate-fade-in">
      <SkeletonTable rows={8} columns={6} />
    </div>
  );
}

export default function RustCommerceAdmin() {
  return (
    <Suspense fallback={<PageFallback />}>
      <Routes>
        {/* Dashboard */}
        <Route index element={<Dashboard />} />

        {/* Products */}
        <Route path="products" element={<ProductList />} />
        <Route path="products/new" element={<ProductEditor mode="create" />} />
        <Route path="products/:id" element={<ProductEditor mode="edit" />} />

        {/* Orders */}
        <Route path="orders" element={<OrderList />} />
        <Route path="orders/:id" element={<OrderDetail />} />

        {/* Customers */}
        <Route path="customers" element={<CustomerList />} />
        <Route path="customers/:id" element={<CustomerDetail />} />

        {/* Settings (with nested tabs) */}
        <Route path="settings" element={<SettingsLayout />}>
          <Route index element={<Navigate to="general" replace />} />
          <Route path="general" element={<GeneralSettings />} />
          <Route path="payments" element={<PaymentSettings />} />
          <Route path="shipping" element={<ShippingSettings />} />
          <Route path="taxes" element={<TaxSettings />} />
          <Route path="checkout" element={<CheckoutSettings />} />
        </Route>

        {/* Catch-all: redirect to store dashboard */}
        <Route path="*" element={<Navigate to="/store" replace />} />
      </Routes>
    </Suspense>
  );
}
```

---

## 5. Lazy Loading Configuration

### 5.1 Code Splitting Strategy

Each major page is a separate chunk. Vite handles the splitting automatically via `React.lazy()` + dynamic `import()`:

```
Chunk                           Routes
-----                           ------
rustcommerce-index.js           Entry point + router
rustcommerce-dashboard.js       /store
rustcommerce-product-list.js    /store/products
rustcommerce-product-editor.js  /store/products/new, /store/products/:id
rustcommerce-order-list.js      /store/orders
rustcommerce-order-detail.js    /store/orders/:id
rustcommerce-customer-list.js   /store/customers
rustcommerce-customer-detail.js /store/customers/:id
rustcommerce-settings.js        /store/settings/*
```

### 5.2 Vite Chunk Naming (optional optimization)

If needed, add manual chunk naming in `vite.config.ts`:

```typescript
// vite.config.ts
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'rustcommerce-core': [
            './src/pages/plugins/rustcommerce/stores/commerceStore.ts',
            './src/pages/plugins/rustcommerce/api/commerceApi.ts',
            './src/pages/plugins/rustcommerce/types/index.ts',
          ],
        },
      },
    },
  },
});
```

### 5.3 Prefetching Strategy

Prefetch the most likely next navigation targets when the user is on a page:

```typescript
// When user is on Dashboard, prefetch ProductList and OrderList
useEffect(() => {
  const prefetchProducts = () => import('./components/products/ProductList');
  const prefetchOrders = () => import('./components/orders/OrderList');

  // Prefetch after initial render is complete
  const timer = setTimeout(() => {
    prefetchProducts();
    prefetchOrders();
  }, 2000);

  return () => clearTimeout(timer);
}, []);
```

---

## 6. Navigation Menu Integration

### 6.1 Sidebar Configuration

RustCommerce appears in the admin sidebar under the "Plugins" section. The navigation is configured in two places:

**A. Plugin Store registration** (`src/store/pluginStore.ts`):

```typescript
{
  id: 'rustcommerce',
  name: 'RustCommerce',
  slug: 'rustcommerce',
  description: 'Full-featured e-commerce plugin for RustPress',
  version: '1.0.0',
  author: 'RustPress',
  active: true,
  icon: 'ShoppingCart',          // Lucide icon name
  category: 'Commerce',
  isRustPlugin: true,
  settings: {},
  menuHref: '/store',
  menuLabel: 'Store',
  showInMenu: true,
  installedAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
}
```

**B. Sidebar sub-menu items** (in `EnterpriseLayout.tsx`):

RustCommerce registers a sidebar group with sub-items. The sidebar should show:

```
Store                          [ShoppingCart icon]
  |-- Dashboard                [LayoutDashboard]
  |-- Products                 [Package]
  |-- Orders                   [ClipboardList]
  |-- Customers                [Users]
  |-- Settings                 [Settings]
```

### 6.2 Sidebar Item Configuration

```typescript
// Navigation items for the RustCommerce sidebar group
const storeNavItems = [
  {
    id: 'store-dashboard',
    label: 'Dashboard',
    href: '/store',
    icon: 'LayoutDashboard',
  },
  {
    id: 'store-products',
    label: 'Products',
    href: '/store/products',
    icon: 'Package',
    badge: productCount,               // Optional: total product count
  },
  {
    id: 'store-orders',
    label: 'Orders',
    href: '/store/orders',
    icon: 'ClipboardList',
    badge: pendingOrderCount,          // Optional: pending order count
  },
  {
    id: 'store-customers',
    label: 'Customers',
    href: '/store/customers',
    icon: 'Users',
  },
  {
    id: 'store-settings',
    label: 'Settings',
    href: '/store/settings',
    icon: 'Settings',
  },
];
```

### 6.3 MegaMenu Integration

The Plugins MegaMenu dropdown (in the top navigation) shows active plugins. RustCommerce appears there automatically because `showInMenu: true` is set in the plugin registration.

The MegaMenu entry for RustCommerce:

```
RustCommerce
  "Full-featured e-commerce for RustPress"
  [ShoppingCart icon]
  -> Links to /store
```

### 6.4 Implementation in EnterpriseLayout

The sidebar items are added by checking active plugins with `showInMenu: true`. To add sub-navigation for the Store, modify `EnterpriseLayout.tsx`:

```tsx
// In the sidebar section of EnterpriseLayout.tsx:

// Check if RustCommerce is active
const commercePlugin = plugins.find((p) => p.id === 'rustcommerce' && p.active);

{commercePlugin && (
  <SidebarGroup label="Store" icon={<ShoppingCart className="w-4 h-4" />}>
    <SidebarItem
      href="/store"
      icon={<LayoutDashboard className="w-4 h-4" />}
      label="Dashboard"
      active={pathname === '/store'}
    />
    <SidebarItem
      href="/store/products"
      icon={<Package className="w-4 h-4" />}
      label="Products"
      active={pathname.startsWith('/store/products')}
    />
    <SidebarItem
      href="/store/orders"
      icon={<ClipboardList className="w-4 h-4" />}
      label="Orders"
      active={pathname.startsWith('/store/orders')}
      badge={pendingOrdersCount > 0 ? pendingOrdersCount : undefined}
    />
    <SidebarItem
      href="/store/customers"
      icon={<Users className="w-4 h-4" />}
      label="Customers"
      active={pathname.startsWith('/store/customers')}
    />
    <SidebarItem
      href="/store/settings"
      icon={<Settings className="w-4 h-4" />}
      label="Settings"
      active={pathname.startsWith('/store/settings')}
    />
  </SidebarGroup>
)}
```

---

## 7. Breadcrumb Configuration

Breadcrumbs are rendered by `PageContainer` using the `breadcrumbs` prop. Each page provides its own breadcrumb trail.

### 7.1 Breadcrumb Definitions by Route

| Route | Breadcrumbs |
|-------|------------|
| `/store` | Store > Dashboard |
| `/store/products` | Store > Products |
| `/store/products/new` | Store > Products > New Product |
| `/store/products/:id` | Store > Products > {Product Name} |
| `/store/orders` | Store > Orders |
| `/store/orders/:id` | Store > Orders > {Order Number} |
| `/store/customers` | Store > Customers |
| `/store/customers/:id` | Store > Customers > {Customer Name} |
| `/store/settings` | Store > Settings |
| `/store/settings/general` | Store > Settings > General |
| `/store/settings/payments` | Store > Settings > Payments |
| `/store/settings/shipping` | Store > Settings > Shipping |
| `/store/settings/taxes` | Store > Settings > Taxes |
| `/store/settings/checkout` | Store > Settings > Checkout |

### 7.2 Breadcrumb Implementation Pattern

```tsx
import { PageContainer } from '@/design-system';
import type { BreadcrumbItem } from '@/design-system';

function ProductEditor({ mode }: { mode: 'create' | 'edit' }) {
  const product = useCommerceStore((s) => s.currentProduct);

  const breadcrumbs: BreadcrumbItem[] = [
    { label: 'Store', href: '/store' },
    { label: 'Products', href: '/store/products' },
    {
      label: mode === 'create'
        ? 'New Product'
        : product?.name ?? 'Loading...',
    },
  ];

  return (
    <PageContainer
      title={mode === 'create' ? 'New Product' : `Edit: ${product?.name ?? ''}`}
      breadcrumbs={breadcrumbs}
    >
      {/* Page content */}
    </PageContainer>
  );
}
```

### 7.3 Reusable Breadcrumb Helper

```typescript
// src/pages/plugins/rustcommerce/hooks/useBreadcrumbs.ts
import type { BreadcrumbItem } from '@/design-system';

const STORE_ROOT: BreadcrumbItem = { label: 'Store', href: '/store' };

export function useStoreBreadcrumbs(...items: BreadcrumbItem[]): BreadcrumbItem[] {
  return [STORE_ROOT, ...items];
}

// Usage:
const breadcrumbs = useStoreBreadcrumbs(
  { label: 'Products', href: '/store/products' },
  { label: product.name }
);
```

---

## 8. Route Guards (Permission Checks)

Each RustCommerce route requires specific permissions. Permission checking is handled at the component level using the existing RustPress auth system.

```typescript
// src/pages/plugins/rustcommerce/hooks/useCommercePermissions.ts
import { useAuthStore } from '@/store/authStore';

export function useCommercePermissions() {
  const user = useAuthStore((s) => s.user);
  const permissions = user?.permissions ?? [];

  return {
    canManageProducts: permissions.includes('manage_products'),
    canManageOrders: permissions.includes('manage_orders'),
    canManageCustomers: permissions.includes('manage_customers'),
    canManageSettings: permissions.includes('manage_store_settings'),
    canViewReports: permissions.includes('view_store_reports'),
    canManageTemplates: permissions.includes('manage_store_templates'),
    isStoreAdmin: permissions.includes('manage_store_settings'),
  };
}

// Usage in a component:
function ProductList() {
  const { canManageProducts } = useCommercePermissions();

  if (!canManageProducts) {
    return <Alert variant="warning">You do not have permission to manage products.</Alert>;
  }

  return (/* product list UI */);
}
```

---

## 9. URL Structure Summary

```
/admin/store                            Dashboard
/admin/store/products                   Product list
/admin/store/products/new               Create product
/admin/store/products/abc-123           Edit product
/admin/store/orders                     Order list
/admin/store/orders/def-456             Order detail
/admin/store/customers                  Customer list
/admin/store/customers/ghi-789          Customer detail
/admin/store/settings                   Settings (redirects to /general)
/admin/store/settings/general           General settings
/admin/store/settings/payments          Payment settings
/admin/store/settings/shipping          Shipping settings
/admin/store/settings/taxes             Tax settings
/admin/store/settings/checkout          Checkout settings
```

Note: The `/admin` prefix comes from the `BrowserRouter basename="/admin"` in `main.tsx`. Route definitions in React Router do NOT include the `/admin` prefix -- it is applied automatically by the router.
