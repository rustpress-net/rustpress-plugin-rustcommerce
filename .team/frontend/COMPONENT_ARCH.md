# Component Architecture -- RustCommerce Admin UI

**Document Version**: 1.0
**Date**: 2026-02-24
**Owner**: Frontend Lead
**Status**: Approved

---

## 1. Directory Structure

All RustCommerce admin UI code lives under `src/pages/plugins/rustcommerce/` inside the `rustpress-core-admin-ui` repository.

```
src/pages/plugins/rustcommerce/
|-- index.tsx                          # Entry point: internal router, lazy-loaded sub-routes
|-- components/
|   |-- dashboard/
|   |   |-- Dashboard.tsx              # Store dashboard (main metrics view)
|   |   |-- RevenueChart.tsx           # Revenue over time (AreaChart)
|   |   |-- OrderStatusPie.tsx         # Orders by status (DonutChart)
|   |   |-- TopProducts.tsx            # Best-selling products widget
|   |   |-- RecentOrders.tsx           # Latest orders activity feed
|   |   |-- StoreMetrics.tsx           # KPI stat cards row
|   |   |-- LowStockAlerts.tsx         # Low inventory warning list
|   |   |-- index.ts                   # Barrel export
|   |
|   |-- products/
|   |   |-- ProductList.tsx            # Product listing with DataTable
|   |   |-- ProductEditor.tsx          # Product create/edit form (full-page)
|   |   |-- ProductForm.tsx            # Core form fields (title, desc, price, sku)
|   |   |-- VariantManager.tsx         # Add/edit/delete product variants
|   |   |-- VariantRow.tsx             # Single variant inline editor row
|   |   |-- ProductImageUpload.tsx     # Image gallery management (drag-to-reorder)
|   |   |-- ProductCategoryPicker.tsx  # Hierarchical category multi-select
|   |   |-- ProductTagInput.tsx        # Flat tag input with autocomplete
|   |   |-- ProductSEO.tsx            # Slug, meta title, meta description
|   |   |-- ProductInventory.tsx       # Stock quantity, low-stock threshold, backorder
|   |   |-- ProductPricing.tsx         # Price, compare-at price, cost price, tax class
|   |   |-- ProductStatusBar.tsx       # Draft/Published/Archived status selector
|   |   |-- ProductBulkActions.tsx     # Bulk delete, status change, category assign
|   |   |-- index.ts
|   |
|   |-- orders/
|   |   |-- OrderList.tsx              # Order listing with DataTable + filters
|   |   |-- OrderDetail.tsx            # Full order detail view
|   |   |-- OrderItems.tsx             # Line items table within order detail
|   |   |-- OrderTimeline.tsx          # Status change history timeline
|   |   |-- OrderStatusBadge.tsx       # Color-coded status badge
|   |   |-- OrderStatusUpdater.tsx     # Dropdown to advance order status
|   |   |-- OrderAddresses.tsx         # Billing/shipping address cards
|   |   |-- OrderPaymentInfo.tsx       # Payment method, transaction ID, Stripe link
|   |   |-- OrderNotes.tsx             # Admin notes + customer notes
|   |   |-- OrderRefundModal.tsx       # Refund initiation modal
|   |   |-- OrderBulkActions.tsx       # Bulk status update, export
|   |   |-- index.ts
|   |
|   |-- customers/
|   |   |-- CustomerList.tsx           # Customer listing with DataTable
|   |   |-- CustomerDetail.tsx         # Customer profile + order history
|   |   |-- CustomerInfo.tsx           # Name, email, phone, notes
|   |   |-- CustomerAddresses.tsx      # Saved billing/shipping addresses
|   |   |-- CustomerOrderHistory.tsx   # Orders placed by this customer
|   |   |-- CustomerStats.tsx          # Total spent, order count, AOV
|   |   |-- index.ts
|   |
|   |-- settings/
|   |   |-- SettingsLayout.tsx         # Settings page with tab navigation
|   |   |-- GeneralSettings.tsx        # Store name, currency, locale
|   |   |-- PaymentSettings.tsx        # Stripe keys, gateway toggle
|   |   |-- ShippingSettings.tsx       # Shipping zones + methods CRUD
|   |   |-- ShippingZoneEditor.tsx     # Zone countries/regions editor
|   |   |-- ShippingMethodRow.tsx      # Single shipping method config row
|   |   |-- TaxSettings.tsx            # Tax rates CRUD, tax zones
|   |   |-- TaxRateRow.tsx             # Single tax rate inline editor
|   |   |-- CheckoutSettings.tsx       # Guest checkout toggle, policies
|   |   |-- index.ts
|   |
|   |-- shared/
|   |   |-- MoneyDisplay.tsx           # Formatted currency display
|   |   |-- MoneyInput.tsx             # Currency-aware numeric input
|   |   |-- AddressCard.tsx            # Formatted address display card
|   |   |-- AddressForm.tsx            # Address entry form (reusable)
|   |   |-- PercentageInput.tsx        # Tax rate / discount percentage input
|   |   |-- SkuInput.tsx               # SKU input with uniqueness indicator
|   |   |-- WeightInput.tsx            # Weight input with unit selector
|   |   |-- DimensionsInput.tsx        # L x W x H input group
|   |   |-- StockBadge.tsx             # In Stock / Out of Stock / Low Stock badge
|   |   |-- DateRangeFilter.tsx        # Date range picker for list filters
|   |   |-- StatusFilter.tsx           # Multi-select status filter dropdown
|   |   |-- SearchBar.tsx              # Debounced search input
|   |   |-- index.ts
|
|-- stores/
|   |-- commerceStore.ts              # Main Zustand store (sliced)
|
|-- api/
|   |-- commerceApi.ts                # Axios-based API client module
|
|-- types/
|   |-- index.ts                      # All TypeScript interfaces and types
|
|-- hooks/
|   |-- useCommercePermissions.ts     # Permission check hooks
|   |-- useMoneyFormatter.ts          # Currency formatting hook
|   |-- useDebouncedSearch.ts         # Debounced search input hook
|   |-- useOrderStatusFlow.ts        # Valid status transitions hook
```

---

## 2. Full Component Tree

```
<App>
  <BrowserRouter basename="/admin">
    <EnterpriseLayout>                      // Existing RustPress layout shell
      <Routes>
        <Route path="/store/*">
          <RustCommerceAdmin>                // index.tsx -- internal router
            |
            |-- /store                      -> <Dashboard />
            |-- /store/products             -> <ProductList />
            |-- /store/products/new         -> <ProductEditor mode="create" />
            |-- /store/products/:id         -> <ProductEditor mode="edit" />
            |-- /store/orders               -> <OrderList />
            |-- /store/orders/:id           -> <OrderDetail />
            |-- /store/customers            -> <CustomerList />
            |-- /store/customers/:id        -> <CustomerDetail />
            |-- /store/settings             -> <SettingsLayout />
            |   |-- /store/settings/general    -> <GeneralSettings />
            |   |-- /store/settings/payments   -> <PaymentSettings />
            |   |-- /store/settings/shipping   -> <ShippingSettings />
            |   |-- /store/settings/taxes      -> <TaxSettings />
            |   |-- /store/settings/checkout   -> <CheckoutSettings />
          </RustCommerceAdmin>
        </Route>
      </Routes>
    </EnterpriseLayout>
  </BrowserRouter>
</App>
```

---

## 3. Component Breakdown by Page

### 3.1 Dashboard (`/store`)

```
<Dashboard>
  <PageContainer title="Store Dashboard" description="Overview of your store performance">
    <Grid cols={4}>
      <StoreMetrics>                          // 4x StatCard: Revenue, Orders, Customers, AOV
        <StatCard icon={DollarSign} />         // Total Revenue (today/week/month)
        <StatCard icon={ShoppingCart} />        // Total Orders
        <StatCard icon={Users} />              // Total Customers
        <StatCard icon={TrendingUp} />          // Average Order Value
      </StoreMetrics>
    </Grid>

    <Grid cols={2}>
      <Card>
        <RevenueChart />                       // AreaChart: revenue over last 30 days
      </Card>
      <Card>
        <OrderStatusPie />                     // DonutChart: orders by status
      </Card>
    </Grid>

    <Grid cols={2}>
      <Card>
        <TopProducts />                        // DataTable: top 10 products by revenue
      </Card>
      <Card>
        <RecentOrders />                       // ActivityFeed: last 10 orders
      </Card>
    </Grid>

    <Card>
      <LowStockAlerts />                       // Alert list: products below threshold
    </Card>
  </PageContainer>
</Dashboard>
```

### 3.2 Product List (`/store/products`)

```
<ProductList>
  <PageContainer title="Products" description="Manage your product catalog">
    <Card>
      <div className="flex justify-between items-center mb-4">
        <div className="flex gap-3">
          <SearchBar placeholder="Search products..." />
          <StatusFilter options={['draft','published','archived']} />
          <Dropdown label="Category" options={categories} />
          <Dropdown label="Stock Status" options={['in_stock','out_of_stock','on_backorder']} />
        </div>
        <Button icon={Plus} onClick={navigateToNew}>Add Product</Button>
      </div>

      <ProductBulkActions selectedIds={selected}>
        <BulkActionsToolbar actions={[delete, changeStatus, assignCategory]} />
      </ProductBulkActions>

      <DataTable
        data={products}
        columns={productColumns}
        loading={loading}
        sortable
        selectable
        pagination
        onRowClick={navigateToEdit}
      />
      // Columns: Checkbox | Image | Name | SKU | Price | Stock | Status | Category | Date
    </Card>
  </PageContainer>
</ProductList>
```

### 3.3 Product Editor (`/store/products/new` and `/store/products/:id`)

```
<ProductEditor>
  <PageContainer
    title={isEdit ? "Edit Product" : "New Product"}
    breadcrumbs={[{label:"Store",href:"/store"},{label:"Products",href:"/store/products"},{label:name}]}
  >
    <Grid cols={3}>
      // Main content (col-span-2)
      <div className="col-span-2 space-y-6">
        <Card>
          <ProductForm>                          // Title, slug, description, short description
            <InputField label="Product Name" />
            <InputField label="Slug" />
            <RichTextEditor label="Description" />
            <TextareaField label="Short Description" />
          </ProductForm>
        </Card>

        <Card title="Pricing">
          <ProductPricing>
            <MoneyInput label="Price" />
            <MoneyInput label="Compare at Price" />
            <MoneyInput label="Cost Price" />
            <SelectField label="Tax Class" options={taxClasses} />
          </ProductPricing>
        </Card>

        <Card title="Inventory">
          <ProductInventory>
            <InputField label="SKU" />
            <NumberField label="Stock Quantity" />
            <NumberField label="Low Stock Threshold" />
            <SelectField label="Stock Status" options={stockStatuses} />
            <Switch label="Allow Backorders" />
          </ProductInventory>
        </Card>

        <Card title="Shipping">
          <WeightInput label="Weight" />
          <DimensionsInput label="Dimensions (L x W x H)" />
        </Card>

        <Card title="Variants">
          <VariantManager>
            <Button icon={Plus}>Add Variant</Button>
            {variants.map(v => (
              <VariantRow variant={v}>                // Inline editable row
                <InputField label="Name" />
                <InputField label="SKU" />
                <MoneyInput label="Price" />
                <NumberField label="Stock" />
                <Button icon={Trash2} variant="ghost" />
              </VariantRow>
            ))}
          </VariantManager>
        </Card>

        <Card title="SEO">
          <ProductSEO>
            <InputField label="Meta Title" />
            <TextareaField label="Meta Description" />
            // Google preview snippet
          </ProductSEO>
        </Card>
      </div>

      // Sidebar (col-span-1)
      <div className="space-y-6">
        <Card>
          <ProductStatusBar>
            <SelectField label="Status" options={['draft','published','archived']} />
            <Switch label="Featured" />
            <SelectField label="Product Type" options={['simple','variable','digital']} />
            <div className="flex gap-2 mt-4">
              <Button variant="primary">Save</Button>
              <Button variant="outline">Save Draft</Button>
            </div>
          </ProductStatusBar>
        </Card>

        <Card title="Images">
          <ProductImageUpload>
            <FileUpload accept="image/*" multiple />
            // Drag-to-reorder grid with primary image selection
          </ProductImageUpload>
        </Card>

        <Card title="Categories">
          <ProductCategoryPicker>
            // Hierarchical checkbox tree
          </ProductCategoryPicker>
        </Card>

        <Card title="Tags">
          <ProductTagInput>
            <TagInput suggestions={existingTags} />
          </ProductTagInput>
        </Card>
      </div>
    </Grid>
  </PageContainer>
</ProductEditor>
```

### 3.4 Order List (`/store/orders`)

```
<OrderList>
  <PageContainer title="Orders" description="View and manage customer orders">
    <Card>
      <div className="flex justify-between items-center mb-4">
        <div className="flex gap-3">
          <SearchBar placeholder="Search by order number, customer..." />
          <StatusFilter options={orderStatuses} />
          <DateRangeFilter />
          <Dropdown label="Payment Status" options={paymentStatuses} />
        </div>
        <ExportOptions formats={['csv','pdf']} />
      </div>

      <OrderBulkActions selectedIds={selected}>
        <BulkActionsToolbar actions={[updateStatus, exportSelected]} />
      </OrderBulkActions>

      <DataTable
        data={orders}
        columns={orderColumns}
        loading={loading}
        sortable
        selectable
        pagination
        onRowClick={navigateToDetail}
      />
      // Columns: Checkbox | Order # | Customer | Date | Status | Payment | Total
    </Card>
  </PageContainer>
</OrderList>
```

### 3.5 Order Detail (`/store/orders/:id`)

```
<OrderDetail>
  <PageContainer
    title={`Order ${order.orderNumber}`}
    breadcrumbs={[{label:"Store"},{label:"Orders",href:"/store/orders"},{label:order.orderNumber}]}
  >
    <div className="flex justify-between items-center mb-6">
      <div className="flex gap-3 items-center">
        <OrderStatusBadge status={order.status} />
        <Badge variant={paymentVariant}>{order.paymentStatus}</Badge>
      </div>
      <div className="flex gap-2">
        <OrderStatusUpdater currentStatus={order.status} onUpdate={updateStatus} />
        <Button variant="outline" icon={RotateCcw} onClick={openRefundModal}>Refund</Button>
        <Dropdown label="More" items={[printInvoice, resendEmail]} />
      </div>
    </div>

    <Grid cols={3}>
      <div className="col-span-2 space-y-6">
        <Card title="Items">
          <OrderItems items={order.items} />
          // Table: Product | SKU | Qty | Unit Price | Tax | Total
          <Divider />
          // Subtotal, Tax, Shipping, Discount, Grand Total summary
        </Card>

        <Card title="Order Timeline">
          <OrderTimeline events={order.statusHistory} />
          // Timeline component with status changes, notes, payment events
        </Card>

        <Card title="Notes">
          <OrderNotes
            adminNote={order.adminNote}
            customerNote={order.customerNote}
            onSaveNote={saveNote}
          />
        </Card>
      </div>

      <div className="space-y-6">
        <Card title="Customer">
          <CustomerInfo customer={order.customer} />
        </Card>

        <Card title="Shipping Address">
          <AddressCard address={order.shippingAddress} />
        </Card>

        <Card title="Billing Address">
          <AddressCard address={order.billingAddress} />
        </Card>

        <Card title="Payment">
          <OrderPaymentInfo payment={order.payment} />
        </Card>
      </div>
    </Grid>

    <OrderRefundModal
      open={refundModalOpen}
      order={order}
      onRefund={processRefund}
      onClose={closeRefundModal}
    />
  </PageContainer>
</OrderDetail>
```

### 3.6 Customer List (`/store/customers`)

```
<CustomerList>
  <PageContainer title="Customers" description="View your customer base">
    <Card>
      <div className="flex justify-between items-center mb-4">
        <SearchBar placeholder="Search by name, email..." />
        <ExportOptions formats={['csv']} />
      </div>

      <DataTable
        data={customers}
        columns={customerColumns}
        loading={loading}
        sortable
        pagination
        onRowClick={navigateToDetail}
      />
      // Columns: Name | Email | Orders | Total Spent | AOV | Last Order | Joined
    </Card>
  </PageContainer>
</CustomerList>
```

### 3.7 Customer Detail (`/store/customers/:id`)

```
<CustomerDetail>
  <PageContainer
    title={`${customer.firstName} ${customer.lastName}`}
    breadcrumbs={[{label:"Store"},{label:"Customers",href:"/store/customers"},{label:name}]}
  >
    <Grid cols={3}>
      <div className="col-span-2 space-y-6">
        <Card title="Order History">
          <CustomerOrderHistory customerId={id} />
        </Card>
      </div>

      <div className="space-y-6">
        <Card>
          <CustomerStats stats={customer} />
          // StatCards: total orders, total spent, AOV
        </Card>

        <Card title="Contact Information">
          <CustomerInfo customer={customer} />
        </Card>

        <Card title="Addresses">
          <CustomerAddresses addresses={customer.addresses} />
        </Card>
      </div>
    </Grid>
  </PageContainer>
</CustomerDetail>
```

### 3.8 Settings (`/store/settings`)

```
<SettingsLayout>
  <PageContainer title="Store Settings" description="Configure your online store">
    <Tabs defaultTab="general">
      <TabList>
        <Tab id="general" icon={Settings}>General</Tab>
        <Tab id="payments" icon={CreditCard}>Payments</Tab>
        <Tab id="shipping" icon={Truck}>Shipping</Tab>
        <Tab id="taxes" icon={Receipt}>Taxes</Tab>
        <Tab id="checkout" icon={ShoppingBag}>Checkout</Tab>
      </TabList>

      <TabPanel id="general"><GeneralSettings /></TabPanel>
      <TabPanel id="payments"><PaymentSettings /></TabPanel>
      <TabPanel id="shipping"><ShippingSettings /></TabPanel>
      <TabPanel id="taxes"><TaxSettings /></TabPanel>
      <TabPanel id="checkout"><CheckoutSettings /></TabPanel>
    </Tabs>
  </PageContainer>
</SettingsLayout>
```

---

## 4. Design System Components Used vs Custom Components

### 4.1 Reused from `@/design-system`

| Component | Import Path | Used In |
|-----------|------------|---------|
| `PageContainer` | `@/design-system` | Every page |
| `Card`, `CardHeader`, `CardBody` | `@/design-system` | Every page |
| `DataTable` | `@/design-system` | ProductList, OrderList, CustomerList, TopProducts |
| `Button`, `IconButton` | `@/design-system` | Every page |
| `Input`, `SearchInput` | `@/design-system` | Search bars, form fields |
| `FormField`, `InputField`, `TextareaField`, `SelectField`, `NumberField` | `@/design-system` | ProductEditor, Settings |
| `Badge`, `StatusBadge` | `@/design-system` | OrderStatusBadge, StockBadge |
| `Modal`, `ConfirmDialog` | `@/design-system` | OrderRefundModal, delete confirmations |
| `Dropdown` | `@/design-system` | Status filters, action menus |
| `Tabs`, `TabList`, `Tab`, `TabPanel` | `@/design-system` | SettingsLayout |
| `Breadcrumbs` | `@/design-system` | Every page via PageContainer |
| `Grid`, `Stack`, `Flex`, `Divider` | `@/design-system` | Layout in all pages |
| `StatCard`, `MetricCard` | `@/design-system` | Dashboard StoreMetrics |
| `AreaChart` | `@/design-system` | RevenueChart |
| `DonutChart` | `@/design-system` | OrderStatusPie |
| `ActivityFeed` | `@/design-system` | RecentOrders |
| `Timeline` | `@/design-system` | OrderTimeline |
| `FileUpload` | `@/design-system` | ProductImageUpload |
| `TagInput` | `@/design-system` | ProductTagInput |
| `Switch`, `LabeledSwitch` | `@/design-system` | ProductEditor, Settings toggles |
| `RichTextEditor` | `@/design-system` | ProductEditor description |
| `Alert` | `@/design-system` | LowStockAlerts, form validation |
| `EmptyState` | `@/design-system` | Empty product/order lists |
| `Skeleton`, `SkeletonTable` | `@/design-system` | Loading states |
| `Pagination` | `@/design-system` | List pages |
| `BulkActionsToolbar` | `@/design-system` | ProductBulkActions, OrderBulkActions |
| `ExportOptions` | `@/design-system` | OrderList, CustomerList |
| `DateRangePicker` | `@/design-system` | DateRangeFilter |
| `Drawer` | `@/design-system` | Mobile filter panels |
| `LoadingSpinner` | `@/design-system` | Async operation indicators |
| `ConfirmDialog` | `@/design-system` | Destructive action confirmations |
| `Chip`, `ChipGroup` | `@/design-system` | Active filter display |
| `Tooltip` | `@/design-system` | Action button hints |
| `ProgressBar` | `@/design-system` | CSV import progress |

### 4.2 Custom Components (RustCommerce-specific)

| Component | Reason |
|-----------|--------|
| `MoneyDisplay` | Currency-aware formatting with store currency |
| `MoneyInput` | Decimal input with currency symbol prefix |
| `PercentageInput` | Tax rate / discount entry with % suffix |
| `SkuInput` | SKU uniqueness validation against backend |
| `WeightInput` | Weight with unit selector (kg, lb, oz, g) |
| `DimensionsInput` | Three-field L x W x H group |
| `StockBadge` | Commerce-specific in_stock / out_of_stock / low_stock / on_backorder states |
| `AddressCard` | Formatted multi-line address display |
| `AddressForm` | Country-aware address form with state/province dropdown |
| `VariantManager` | Product variant CRUD with attribute matrix |
| `VariantRow` | Inline variant editing row |
| `ProductCategoryPicker` | Hierarchical checkbox tree for rc_categories |
| `OrderStatusBadge` | Color-coded badge mapping 7 order statuses |
| `OrderStatusUpdater` | State-machine-aware status transition dropdown |
| `OrderRefundModal` | Stripe refund initiation with amount/reason |
| `OrderPaymentInfo` | Stripe payment details display |
| `ShippingZoneEditor` | Country/region multi-select for zones |
| `ShippingMethodRow` | Shipping method config (type-specific fields) |
| `TaxRateRow` | Tax rate inline editor with location fields |
| `SearchBar` | Debounced search with clear button (wraps SearchInput) |
| `StatusFilter` | Multi-select status dropdown (wraps Dropdown) |
| `DateRangeFilter` | Preset + custom date range filter (wraps DateRangePicker) |

---

## 5. Component Props Interfaces (TypeScript)

### 5.1 Page-Level Components

```typescript
// Dashboard
interface DashboardProps {
  // No props -- reads from commerceStore
}

// Product List
interface ProductListProps {
  // No props -- reads from commerceStore
}

// Product Editor
interface ProductEditorProps {
  mode: 'create' | 'edit';
  productId?: string;  // Required when mode='edit'
}

// Order List
interface OrderListProps {
  // No props -- reads from commerceStore
}

// Order Detail
interface OrderDetailProps {
  orderId: string;  // From route param
}

// Customer List
interface CustomerListProps {
  // No props -- reads from commerceStore
}

// Customer Detail
interface CustomerDetailProps {
  customerId: string;  // From route param
}

// Settings Layout
interface SettingsLayoutProps {
  defaultTab?: 'general' | 'payments' | 'shipping' | 'taxes' | 'checkout';
}
```

### 5.2 Dashboard Widget Components

```typescript
interface StoreMetricsProps {
  period: 'today' | '7d' | '30d' | '90d' | 'year';
  metrics: {
    revenue: number;
    revenueChange: number;       // Percentage change vs prior period
    orders: number;
    ordersChange: number;
    customers: number;
    customersChange: number;
    averageOrderValue: number;
    aovChange: number;
  };
  loading: boolean;
}

interface RevenueChartProps {
  data: Array<{
    date: string;              // ISO date
    revenue: number;
    orders: number;
  }>;
  period: 'today' | '7d' | '30d' | '90d' | 'year';
  loading: boolean;
}

interface OrderStatusPieProps {
  data: Array<{
    status: OrderStatus;
    count: number;
    color: string;
  }>;
  loading: boolean;
}

interface TopProductsProps {
  products: Array<{
    id: string;
    name: string;
    imageUrl?: string;
    unitsSold: number;
    revenue: number;
  }>;
  loading: boolean;
}

interface RecentOrdersProps {
  orders: Array<{
    id: string;
    orderNumber: string;
    customerName: string;
    total: number;
    status: OrderStatus;
    createdAt: string;
  }>;
  loading: boolean;
}

interface LowStockAlertsProps {
  products: Array<{
    id: string;
    name: string;
    sku: string;
    stockQuantity: number;
    lowStockThreshold: number;
  }>;
  loading: boolean;
}
```

### 5.3 Product Components

```typescript
interface ProductFormProps {
  product: Partial<Product>;
  onChange: (field: keyof Product, value: any) => void;
  errors: Record<string, string>;
}

interface VariantManagerProps {
  productId?: string;
  variants: ProductVariant[];
  onAdd: () => void;
  onUpdate: (variantId: string, data: Partial<ProductVariant>) => void;
  onDelete: (variantId: string) => void;
  onReorder: (fromIndex: number, toIndex: number) => void;
}

interface VariantRowProps {
  variant: ProductVariant;
  onUpdate: (data: Partial<ProductVariant>) => void;
  onDelete: () => void;
}

interface ProductImageUploadProps {
  images: ProductImage[];
  onUpload: (files: File[]) => Promise<void>;
  onDelete: (imageId: string) => void;
  onSetPrimary: (imageId: string) => void;
  onReorder: (fromIndex: number, toIndex: number) => void;
  maxImages?: number;        // Default: 10
}

interface ProductCategoryPickerProps {
  categories: Category[];          // Full category tree
  selectedIds: string[];
  onChange: (selectedIds: string[]) => void;
}

interface ProductTagInputProps {
  tags: string[];
  suggestions: string[];
  onChange: (tags: string[]) => void;
}

interface ProductSEOProps {
  slug: string;
  metaTitle: string;
  metaDescription: string;
  onChange: (field: string, value: string) => void;
  productName: string;            // For auto-generation
}

interface ProductInventoryProps {
  sku: string;
  stockQuantity: number;
  stockStatus: StockStatus;
  lowStockThreshold: number;
  allowBackorders: boolean;
  onChange: (field: string, value: any) => void;
  errors: Record<string, string>;
}

interface ProductPricingProps {
  price: number;
  compareAtPrice?: number;
  costPrice?: number;
  taxClass: string;
  taxClasses: Array<{ value: string; label: string }>;
  onChange: (field: string, value: any) => void;
  currency: string;               // From store settings
}

interface ProductStatusBarProps {
  status: ProductStatus;
  featured: boolean;
  productType: ProductType;
  onStatusChange: (status: ProductStatus) => void;
  onFeaturedChange: (featured: boolean) => void;
  onTypeChange: (type: ProductType) => void;
  onSave: () => Promise<void>;
  onSaveDraft: () => Promise<void>;
  saving: boolean;
  hasChanges: boolean;
}

interface ProductBulkActionsProps {
  selectedIds: string[];
  onDelete: (ids: string[]) => Promise<void>;
  onStatusChange: (ids: string[], status: ProductStatus) => Promise<void>;
  onCategoryAssign: (ids: string[], categoryIds: string[]) => Promise<void>;
  onClearSelection: () => void;
}
```

### 5.4 Order Components

```typescript
interface OrderItemsProps {
  items: OrderItem[];
  currency: string;
  subtotal: number;
  taxTotal: number;
  shippingTotal: number;
  discountTotal: number;
  grandTotal: number;
}

interface OrderTimelineProps {
  events: Array<{
    id: string;
    type: 'status_change' | 'payment' | 'note' | 'refund' | 'shipment';
    title: string;
    description?: string;
    timestamp: string;
    actor?: string;
  }>;
}

interface OrderStatusBadgeProps {
  status: OrderStatus;
  size?: 'sm' | 'md' | 'lg';
}

interface OrderStatusUpdaterProps {
  currentStatus: OrderStatus;
  onUpdate: (newStatus: OrderStatus) => Promise<void>;
  loading: boolean;
}

interface OrderAddressesProps {
  billingAddress: Address;
  shippingAddress: Address;
}

interface OrderPaymentInfoProps {
  paymentMethod: string;
  paymentStatus: PaymentStatus;
  transactionId?: string;
  stripePaymentIntentId?: string;
  amount: number;
  currency: string;
  refundAmount?: number;
}

interface OrderNotesProps {
  adminNote?: string;
  customerNote?: string;
  onSaveAdminNote: (note: string) => Promise<void>;
}

interface OrderRefundModalProps {
  open: boolean;
  order: Order;
  onRefund: (amount: number, reason: string) => Promise<void>;
  onClose: () => void;
  processing: boolean;
}

interface OrderBulkActionsProps {
  selectedIds: string[];
  onStatusUpdate: (ids: string[], status: OrderStatus) => Promise<void>;
  onExport: (ids: string[], format: 'csv' | 'pdf') => Promise<void>;
  onClearSelection: () => void;
}
```

### 5.5 Customer Components

```typescript
interface CustomerInfoProps {
  customer: Customer;
  editable?: boolean;
  onUpdate?: (data: Partial<Customer>) => Promise<void>;
}

interface CustomerAddressesProps {
  addresses: CustomerAddress[];
  onAdd?: (address: CustomerAddress) => Promise<void>;
  onUpdate?: (addressId: string, data: Partial<CustomerAddress>) => Promise<void>;
  onDelete?: (addressId: string) => Promise<void>;
  onSetDefault?: (addressId: string) => Promise<void>;
}

interface CustomerOrderHistoryProps {
  customerId: string;
  orders: Order[];
  loading: boolean;
  onViewOrder: (orderId: string) => void;
}

interface CustomerStatsProps {
  totalOrders: number;
  totalSpent: number;
  averageOrderValue: number;
  lastOrderAt?: string;
  currency: string;
}
```

### 5.6 Settings Components

```typescript
interface GeneralSettingsProps {
  settings: StoreSettings;
  onSave: (data: Partial<StoreSettings>) => Promise<void>;
  saving: boolean;
}

interface PaymentSettingsProps {
  settings: PaymentGatewaySettings;
  onSave: (data: Partial<PaymentGatewaySettings>) => Promise<void>;
  onTestConnection: () => Promise<{ success: boolean; message: string }>;
  saving: boolean;
}

interface ShippingSettingsProps {
  zones: ShippingZone[];
  methods: ShippingMethod[];
  onCreateZone: (zone: Partial<ShippingZone>) => Promise<void>;
  onUpdateZone: (zoneId: string, data: Partial<ShippingZone>) => Promise<void>;
  onDeleteZone: (zoneId: string) => Promise<void>;
  onCreateMethod: (zoneId: string, method: Partial<ShippingMethod>) => Promise<void>;
  onUpdateMethod: (methodId: string, data: Partial<ShippingMethod>) => Promise<void>;
  onDeleteMethod: (methodId: string) => Promise<void>;
  saving: boolean;
}

interface TaxSettingsProps {
  rates: TaxRate[];
  onCreateRate: (rate: Partial<TaxRate>) => Promise<void>;
  onUpdateRate: (rateId: string, data: Partial<TaxRate>) => Promise<void>;
  onDeleteRate: (rateId: string) => Promise<void>;
  saving: boolean;
}

interface CheckoutSettingsProps {
  settings: CheckoutConfig;
  onSave: (data: Partial<CheckoutConfig>) => Promise<void>;
  saving: boolean;
}
```

### 5.7 Shared Components

```typescript
interface MoneyDisplayProps {
  amount: number;
  currency?: string;         // Default: from store settings
  showSign?: boolean;        // Show +/- for changes
  className?: string;
}

interface MoneyInputProps {
  value: number;
  onChange: (value: number) => void;
  currency?: string;
  label?: string;
  error?: string;
  disabled?: boolean;
  min?: number;
  max?: number;
  placeholder?: string;
}

interface AddressCardProps {
  address: Address;
  title?: string;
  editable?: boolean;
  onEdit?: () => void;
}

interface AddressFormProps {
  address: Partial<Address>;
  onChange: (address: Partial<Address>) => void;
  errors: Record<string, string>;
  countries: Array<{ code: string; name: string }>;
}

interface StockBadgeProps {
  stockStatus: StockStatus;
  stockQuantity?: number;
  lowStockThreshold?: number;
}

interface SearchBarProps {
  placeholder?: string;
  value: string;
  onChange: (value: string) => void;
  debounceMs?: number;      // Default: 300
}

interface StatusFilterProps {
  options: Array<{ value: string; label: string; color?: string }>;
  selected: string[];
  onChange: (selected: string[]) => void;
  label?: string;
}

interface DateRangeFilterProps {
  startDate?: string;
  endDate?: string;
  onChange: (start: string, end: string) => void;
  presets?: Array<{ label: string; days: number }>;
}
```

---

## 6. Page Layout Patterns

### 6.1 List Pages (Products, Orders, Customers)

All list pages follow this consistent structure:

```
PageContainer (title, description, breadcrumbs)
  Card
    Toolbar Row (search + filters + primary action button)
    BulkActionsToolbar (appears when items selected)
    DataTable (sortable, selectable, paginated)
```

### 6.2 Detail Pages (Order Detail, Customer Detail)

Detail pages use a 3-column grid:

```
PageContainer (title, breadcrumbs)
  Header Row (status badges + action buttons)
  Grid cols=3
    Main Content (col-span-2)
      Card sections stacked vertically
    Sidebar (col-span-1)
      Card sections stacked vertically
```

### 6.3 Editor Pages (Product Editor)

Editor pages use a 3-column grid with form in main area and meta in sidebar:

```
PageContainer (title, breadcrumbs)
  Grid cols=3
    Form Content (col-span-2)
      Card per form section (stacked)
    Sidebar (col-span-1)
      Status/Actions Card
      Media Card
      Taxonomy Cards
```

### 6.4 Settings Page

Settings use a tabbed layout within a single PageContainer:

```
PageContainer (title, description)
  Tabs
    TabList (horizontal tabs)
    TabPanel per settings section
      Card with FormField groups
      Save button at bottom of each panel
```

---

## 7. Entry Point Component (`index.tsx`)

```typescript
// src/pages/plugins/rustcommerce/index.tsx
import React, { Suspense, lazy } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { SkeletonTable } from '@/design-system';

const Dashboard = lazy(() => import('./components/dashboard/Dashboard'));
const ProductList = lazy(() => import('./components/products/ProductList'));
const ProductEditor = lazy(() => import('./components/products/ProductEditor'));
const OrderList = lazy(() => import('./components/orders/OrderList'));
const OrderDetail = lazy(() => import('./components/orders/OrderDetail'));
const CustomerList = lazy(() => import('./components/customers/CustomerList'));
const CustomerDetail = lazy(() => import('./components/customers/CustomerDetail'));
const SettingsLayout = lazy(() => import('./components/settings/SettingsLayout'));

function LoadingFallback() {
  return <SkeletonTable rows={8} columns={6} />;
}

export default function RustCommerceAdmin() {
  return (
    <Suspense fallback={<LoadingFallback />}>
      <Routes>
        <Route index element={<Dashboard />} />
        <Route path="products" element={<ProductList />} />
        <Route path="products/new" element={<ProductEditor mode="create" />} />
        <Route path="products/:id" element={<ProductEditor mode="edit" />} />
        <Route path="orders" element={<OrderList />} />
        <Route path="orders/:id" element={<OrderDetail />} />
        <Route path="customers" element={<CustomerList />} />
        <Route path="customers/:id" element={<CustomerDetail />} />
        <Route path="settings/*" element={<SettingsLayout />} />
        <Route path="*" element={<Navigate to="/store" replace />} />
      </Routes>
    </Suspense>
  );
}
```
