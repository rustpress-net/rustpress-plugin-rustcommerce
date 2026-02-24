# UI Mockups (ASCII Wireframes) -- RustCommerce Admin UI

**Document Version**: 1.0
**Date**: 2026-02-24
**Owner**: Frontend Lead
**Status**: Approved

---

## 1. Store Dashboard (`/store`)

```
+-----------------------------------------------------------------------------------+
| [<] Store > Dashboard                                          [Today v] [30d] [Y]|
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  +-- Revenue -------+  +-- Orders --------+  +-- Customers ----+  +-- AOV -------+|
|  |  $47,832.50      |  |  342             |  |  1,247          |  |  $139.86     ||
|  |  +12.4% vs prior |  |  +8.2% vs prior  |  |  +15.1% vs prior|  |  +3.7% vs p ||
|  |  [^ green arrow] |  |  [^ green arrow]  |  |  [^ green arrow]|  |  [^ green]  ||
|  +------------------+  +------------------+  +------------------+  +-------------+|
|                                                                                   |
|  +-- Revenue Over Time (30 Days) --------+  +-- Orders by Status ---------------+|
|  |                                        |  |                                    ||
|  |  $2.5k ._                              |  |        .--------.                  ||
|  |        | '._    .---.                  |  |       /  Pending  \                ||
|  |  $2.0k |    '--'     '-.               |  |      |    23%     |                ||
|  |        |                '._.           |  |       \          /                 ||
|  |  $1.5k |                    '-.        |  |   .----|--------|----.             ||
|  |        |                       '.      |  |  / Processing 31%    \            ||
|  |  $1.0k |                         |     |  |  |                    |            ||
|  |        |                         |     |  |  \ Shipped  28%      /            ||
|  |  $0.5k |                         |     |  |   '----.---------.--'             ||
|  |        |                         |     |  |        | Delivered 18% |           ||
|  |    $0  +---+---+---+---+---+---+-+     |  |                                    ||
|  |        Feb 1   Feb 8  Feb 15 Feb 22    |  |  [*] Pending: 79   [*] Shipped: 96 ||
|  |                                        |  |  [*] Processing:106 [*] Deliv: 61  ||
|  +----------------------------------------+  +------------------------------------+|
|                                                                                   |
|  +-- Top Products by Revenue -----------+  +-- Recent Orders -------------------+|
|  |  #   Product              Revenue    |  |                                    ||
|  |  --  -------------------  --------   |  |  RC-00342  Sarah M.    $234.50     ||
|  |  1   Premium Widget       $4,230.00  |  |  Processing           2 min ago    ||
|  |  2   Deluxe Gadget Pro    $3,891.50  |  |  ---                               ||
|  |  3   Standard Kit         $3,450.00  |  |  RC-00341  John D.    $89.99      ||
|  |  4   Ultra Component      $2,780.25  |  |  Pending              15 min ago   ||
|  |  5   Basic Starter Set    $2,340.00  |  |  ---                               ||
|  |  6   Pro Bundle           $2,100.00  |  |  RC-00340  Emily R.   $567.00     ||
|  |  7   Economy Pack         $1,950.75  |  |  Shipped              1 hour ago   ||
|  |  8   Mini Accessory       $1,670.00  |  |  ---                               ||
|  |  9   Connector Cable      $1,440.50  |  |  RC-00339  Mike T.    $45.00      ||
|  |  10  Replacement Part     $1,200.00  |  |  Delivered             3 hours ago  ||
|  +--------------------------------------+  +------------------------------------+|
|                                                                                   |
|  +-- Low Stock Alerts (5 products below threshold) -----------------------------+|
|  |  [!] Premium Widget (SKU: PRW-001) -- 3 remaining (threshold: 10)            ||
|  |  [!] Deluxe Gadget Pro (SKU: DGP-002) -- 5 remaining (threshold: 15)         ||
|  |  [!] Connector Cable (SKU: CC-009) -- 2 remaining (threshold: 5)             ||
|  |  [!] Replacement Part (SKU: RP-010) -- 0 remaining (threshold: 5) [OUT]      ||
|  |  [!] Mini Accessory (SKU: MA-008) -- 4 remaining (threshold: 10)             ||
|  +-------------------------------------------------------------------------------+|
+-----------------------------------------------------------------------------------+
```

---

## 2. Product List (`/store/products`)

```
+-----------------------------------------------------------------------------------+
| [<] Store > Products                                                              |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  +-- Products ------------------------------------------------------------------+|
|  |                                                                               ||
|  |  [Search products...________]  [Status: All v]  [Category: All v]  [Stock: v] ||
|  |                                                            [+ Add Product]    ||
|  |                                                                               ||
|  |  [x] 3 selected:  [Delete]  [Change Status v]  [Assign Category v]  [Clear]  ||
|  |                                                                               ||
|  |  +--------------------------------------------------------------------------+||
|  |  | [ ] | IMG | Product Name        | SKU      | Price   | Stock | Status    |||
|  |  |-----|-----|---------------------|----------|---------|-------|-----------|||
|  |  | [x] | [#] | Premium Widget      | PRW-001  | $49.99  |    3  | Published |||
|  |  | [x] | [#] | Deluxe Gadget Pro   | DGP-002  | $129.99 |    5  | Published |||
|  |  | [ ] | [#] | Standard Kit        | STK-003  | $79.99  |   42  | Published |||
|  |  | [ ] | [#] | Ultra Component     | UC-004   | $199.99 |   18  | Published |||
|  |  | [x] | [#] | Basic Starter Set   | BSS-005  | $29.99  |   67  | Draft     |||
|  |  | [ ] | [#] | Pro Bundle          | PB-006   | $349.99 |   12  | Published |||
|  |  | [ ] | [#] | Economy Pack        | EP-007   | $19.99  |  150  | Published |||
|  |  | [ ] | [#] | Mini Accessory      | MA-008   | $9.99   |    4  | Archived  |||
|  |  | [ ] | [#] | Connector Cable     | CC-009   | $14.99  |    2  | Published |||
|  |  | [ ] | [#] | Replacement Part    | RP-010   | $7.99   |    0  | Published |||
|  |  +--------------------------------------------------------------------------+||
|  |                                                                               ||
|  |  Showing 1-10 of 48 products                    [< Prev]  1  2  3  4  [Next >]||
|  +-------------------------------------------------------------------------------+|
+-----------------------------------------------------------------------------------+
```

---

## 3. Product Editor (`/store/products/new`)

```
+-----------------------------------------------------------------------------------+
| [<] Store > Products > New Product                                                |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  +-- MAIN CONTENT (col-span-2) --+  +-- SIDEBAR (col-span-1) --+               |
|  |                                |  |                           |               |
|  |  +-- Product Details -------+  |  |  +-- Publish -----------+|               |
|  |  |                          |  |  |  |                       ||               |
|  |  |  Product Name            |  |  |  |  Status: [Draft v]    ||               |
|  |  |  [_____________________] |  |  |  |                       ||               |
|  |  |                          |  |  |  |  Featured: [ ] No     ||               |
|  |  |  Slug                    |  |  |  |                       ||               |
|  |  |  [premium-widget_______] |  |  |  |  Type: [Simple v]     ||               |
|  |  |                          |  |  |  |                       ||               |
|  |  |  Description             |  |  |  |  [Save Draft]         ||               |
|  |  |  +--------------------+  |  |  |  |  [Publish]            ||               |
|  |  |  | B I U | H1 H2 | UL |  |  |  |  +-----------------------+|               |
|  |  |  |--------------------+  |  |  |                           |               |
|  |  |  | Rich text editor   |  |  |  |  +-- Product Images ----+|               |
|  |  |  | content area       |  |  |  |  |                       ||               |
|  |  |  | ...                 |  |  |  |  |  +---+ +---+ +---+   ||               |
|  |  |  +--------------------+  |  |  |  |  |[*]| |   | |   |   ||               |
|  |  |                          |  |  |  |  | img| | img| | img|  ||               |
|  |  |  Short Description      |  |  |  |  +---+ +---+ +---+   ||               |
|  |  |  [_____________________] |  |  |  |                       ||               |
|  |  |  [_____________________] |  |  |  |  [+ Upload Images]    ||               |
|  |  +--------------------------+  |  |  |  * = primary image    ||               |
|  |                                |  |  +-----------------------+|               |
|  |  +-- Pricing ---------------+  |  |                           |               |
|  |  |                          |  |  |  +-- Categories ---------+|               |
|  |  |  Price          Compare  |  |  |  |                       ||               |
|  |  |  [$_49.99____]  [$____]  |  |  |  |  [x] Electronics     ||               |
|  |  |                          |  |  |  |    [x] Widgets        ||               |
|  |  |  Cost Price    Tax Class |  |  |  |    [ ] Gadgets        ||               |
|  |  |  [$_25.00____] [Std. v]  |  |  |  |  [ ] Accessories     ||               |
|  |  +--------------------------+  |  |  |  [ ] Bundles          ||               |
|  |                                |  |  +-----------------------+|               |
|  |  +-- Inventory -------------+  |  |                           |               |
|  |  |                          |  |  |  +-- Tags ---------------+|               |
|  |  |  SKU           Stock     |  |  |  |                       ||               |
|  |  |  [PRW-001____] [_100__]  |  |  |  |  [new] [premium] [x] ||               |
|  |  |                          |  |  |  |  [Add tag...________] ||               |
|  |  |  Low Stock Threshold     |  |  |  +-----------------------+|               |
|  |  |  [_10__________]         |  |  |                           |               |
|  |  |                          |  |  +---------------------------+               |
|  |  |  Stock Status: [In Stock v]                                |               |
|  |  |  Allow Backorders: [ ] No  |                               |               |
|  |  +--------------------------+  |                               |               |
|  |                                |                               |               |
|  |  +-- Shipping ---------------+ |                               |               |
|  |  |                          |  |                               |               |
|  |  |  Weight (kg)             |  |                               |               |
|  |  |  [_0.5___________]      |  |                               |               |
|  |  |                          |  |                               |               |
|  |  |  Dimensions (cm)         |  |                               |               |
|  |  |  L [_10_] W [_5_] H [_3]|  |                               |               |
|  |  +--------------------------+  |                               |               |
|  |                                |                               |               |
|  |  +-- Variants ---------------+ |                               |               |
|  |  |                          |  |                               |               |
|  |  |  [+ Add Variant]         |  |                               |               |
|  |  |                          |  |                               |               |
|  |  |  Name       SKU     Price   Stock  Actions                  |               |
|  |  |  ------     -----   -----   -----  -------                  |               |
|  |  |  Red/S      PRW-R-S $49.99    25   [Edit] [x]              |               |
|  |  |  Red/M      PRW-R-M $49.99    30   [Edit] [x]              |               |
|  |  |  Red/L      PRW-R-L $54.99    20   [Edit] [x]              |               |
|  |  |  Blue/S     PRW-B-S $49.99    15   [Edit] [x]              |               |
|  |  |  Blue/M     PRW-B-M $49.99    22   [Edit] [x]              |               |
|  |  +--------------------------+  |                               |               |
|  |                                |                               |               |
|  |  +-- SEO -------------------+  |                               |               |
|  |  |                          |  |                               |               |
|  |  |  Meta Title              |  |                               |               |
|  |  |  [Premium Widget - My..] |  |                               |               |
|  |  |                          |  |                               |               |
|  |  |  Meta Description        |  |                               |               |
|  |  |  [High-quality premium.] |  |                               |               |
|  |  |                          |  |                               |               |
|  |  |  Preview:                |  |                               |               |
|  |  |  +--------------------+  |  |                               |               |
|  |  |  | Premium Widget     |  |  |                               |               |
|  |  |  | mystore.com/store/.|  |  |                               |               |
|  |  |  | High-quality prem..|  |  |                               |               |
|  |  |  +--------------------+  |  |                               |               |
|  |  +--------------------------+  |                               |               |
|  +--------------------------------+                               |               |
+-----------------------------------------------------------------------------------+
```

---

## 4. Order List (`/store/orders`)

```
+-----------------------------------------------------------------------------------+
| [<] Store > Orders                                                                |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  +-- Orders -------------------------------------------------------------------+|
|  |                                                                              ||
|  |  [Search order #, customer...__]  [Status: All v]  [Payment: All v]          ||
|  |  [Date Range: Last 30 days v]                             [Export CSV v]      ||
|  |                                                                              ||
|  |  +-------------------------------------------------------------------------+||
|  |  | [ ] | Order #    | Customer       | Date       | Status     | Payment   |||
|  |  |     |            |                |            |            | Total     |||
|  |  |-----|------------|----------------|------------|------------|-----------|||
|  |  | [ ] | RC-00342   | Sarah Mitchell | Feb 24, 26 | Processing | $234.50   |||
|  |  |     |            |                | 2:34 PM    | [*blue*]   | Paid      |||
|  |  |-----|------------|----------------|------------|------------|-----------|||
|  |  | [ ] | RC-00341   | John Doe       | Feb 24, 26 | Pending    | $89.99    |||
|  |  |     |            |                | 2:19 PM    | [*yellow*] | Unpaid    |||
|  |  |-----|------------|----------------|------------|------------|-----------|||
|  |  | [ ] | RC-00340   | Emily Roberts  | Feb 24, 26 | Shipped    | $567.00   |||
|  |  |     |            |                | 1:45 PM    | [*purple*] | Paid      |||
|  |  |-----|------------|----------------|------------|------------|-----------|||
|  |  | [ ] | RC-00339   | Mike Torres    | Feb 24, 26 | Delivered  | $45.00    |||
|  |  |     |            |                | 11:02 AM   | [*green*]  | Paid      |||
|  |  |-----|------------|----------------|------------|------------|-----------|||
|  |  | [ ] | RC-00338   | Lisa Wang      | Feb 23, 26 | Cancelled  | $320.00   |||
|  |  |     |            |                | 4:56 PM    | [*red*]    | Refunded  |||
|  |  |-----|------------|----------------|------------|------------|-----------|||
|  |  | [ ] | RC-00337   | David Kim      | Feb 23, 26 | Processing | $178.50   |||
|  |  |     |            |                | 3:22 PM    | [*blue*]   | Paid      |||
|  |  |-----|------------|----------------|------------|------------|-----------|||
|  |  | [ ] | RC-00336   | Anna Schmidt   | Feb 23, 26 | Shipped    | $92.00    |||
|  |  |     |            |                | 1:10 PM    | [*purple*] | Paid      |||
|  |  +-------------------------------------------------------------------------+||
|  |                                                                              ||
|  |  Showing 1-20 of 342 orders                     [< Prev]  1  2  3  [Next >] ||
|  +------------------------------------------------------------------------------+|
+-----------------------------------------------------------------------------------+
```

---

## 5. Order Detail (`/store/orders/RC-00342`)

```
+-----------------------------------------------------------------------------------+
| [<] Store > Orders > RC-00342                                                     |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  RC-00342  [Processing]  [Paid]         [Mark as Shipped v]  [Refund]  [More v]  |
|                                                                                   |
|  +-- MAIN CONTENT (col-span-2) -----+  +-- SIDEBAR (col-span-1) ----+           |
|  |                                    |  |                            |           |
|  |  +-- Order Items ---------------+  |  |  +-- Customer -----------+|           |
|  |  |                              |  |  |  |                       ||           |
|  |  |  Product         Qty  Price  |  |  |  |  Sarah Mitchell       ||           |
|  |  |  ---------------  ---  ----  |  |  |  |  sarah@example.com    ||           |
|  |  |  Premium Widget    2  $99.98 |  |  |  |  +1 (555) 123-4567   ||           |
|  |  |  SKU: PRW-001               |  |  |  |                       ||           |
|  |  |  Variant: Red / Medium      |  |  |  |  5 orders | $1,234    ||           |
|  |  |                              |  |  |  |  [View Profile ->]    ||           |
|  |  |  Standard Kit      1  $79.99 |  |  |  +-----------------------+|           |
|  |  |  SKU: STK-003               |  |  |                            |           |
|  |  |                              |  |  |  +-- Shipping Address ---+|           |
|  |  |  Connector Cable   3  $44.97 |  |  |  |                       ||           |
|  |  |  SKU: CC-009                 |  |  |  |  Sarah Mitchell       ||           |
|  |  |                              |  |  |  |  123 Main Street      ||           |
|  |  |  ----------------------------+  |  |  |  Apt 4B               ||           |
|  |  |                              |  |  |  |  New York, NY 10001   ||           |
|  |  |  Subtotal:          $224.94  |  |  |  |  United States        ||           |
|  |  |  Tax (8.25%):        $18.56  |  |  |  +-----------------------+|           |
|  |  |  Shipping (Std):      $9.99  |  |  |                            |           |
|  |  |  Discount:           -$18.99 |  |  |  +-- Billing Address ---+|           |
|  |  |  ----------------------------+  |  |  |                       ||           |
|  |  |  GRAND TOTAL:       $234.50  |  |  |  |  Sarah Mitchell       ||           |
|  |  +------------------------------+  |  |  |  123 Main Street      ||           |
|  |                                    |  |  |  Apt 4B               ||           |
|  |  +-- Order Timeline ------------+  |  |  |  New York, NY 10001   ||           |
|  |  |                              |  |  |  |  United States        ||           |
|  |  |  [*] Feb 24, 2:34 PM        |  |  |  +-----------------------+|           |
|  |  |  |  Status changed to       |  |  |                            |           |
|  |  |  |  Processing              |  |  |  +-- Payment ------------+|           |
|  |  |  |  by Admin                |  |  |  |                       ||           |
|  |  |  |                          |  |  |  |  Method: Stripe       ||           |
|  |  |  [*] Feb 24, 2:34 PM        |  |  |  |  Status: Paid        ||           |
|  |  |  |  Payment received        |  |  |  |  Amount: $234.50     ||           |
|  |  |  |  Stripe: pi_3Ox...       |  |  |  |  Transaction:        ||           |
|  |  |  |                          |  |  |  |  ch_3OxABC123...     ||           |
|  |  |  [*] Feb 24, 2:30 PM        |  |  |  |                       ||           |
|  |  |  |  Order created           |  |  |  |  Coupon: SAVE10       ||           |
|  |  |  |  Status: Pending         |  |  |  |  Discount: -$18.99   ||           |
|  |  |  |                          |  |  |  +-----------------------+|           |
|  |  +------------------------------+  |  |                            |           |
|  |                                    |  +----------------------------+           |
|  |  +-- Admin Notes ---------------+  |                               |           |
|  |  |                              |  |                               |           |
|  |  |  Customer note:              |  |                               |           |
|  |  |  "Please gift wrap this"     |  |                               |           |
|  |  |                              |  |                               |           |
|  |  |  Admin note:                 |  |                               |           |
|  |  |  [________________________]  |  |                               |           |
|  |  |  [________________________]  |  |                               |           |
|  |  |  [Save Note]                 |  |                               |           |
|  |  +------------------------------+  |                               |           |
|  +------------------------------------+                               |           |
+-----------------------------------------------------------------------------------+
```

---

## 6. Customer List (`/store/customers`)

```
+-----------------------------------------------------------------------------------+
| [<] Store > Customers                                                             |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  +-- Customers ----------------------------------------------------------------+|
|  |                                                                              ||
|  |  [Search by name, email..._____]                         [Export CSV v]      ||
|  |                                                                              ||
|  |  +-------------------------------------------------------------------------+||
|  |  | Name               | Email               | Orders | Spent    | Last Order|||
|  |  |--------------------|---------------------|--------|----------|-----------|||
|  |  | Sarah Mitchell     | sarah@example.com   |      5 | $1,234.50| Feb 24    |||
|  |  | John Doe           | john@example.com    |      3 |   $267.97| Feb 24    |||
|  |  | Emily Roberts      | emily@example.com   |     12 | $4,560.00| Feb 24    |||
|  |  | Mike Torres        | mike@example.com    |      1 |    $45.00| Feb 24    |||
|  |  | Lisa Wang          | lisa@example.com    |      8 | $2,180.50| Feb 23    |||
|  |  | David Kim          | david@example.com   |      4 |   $712.00| Feb 23    |||
|  |  | Anna Schmidt       | anna@example.com    |      6 | $1,890.25| Feb 23    |||
|  |  | Robert Chen        | robert@example.com  |      2 |   $156.98| Feb 22    |||
|  |  | Jennifer Park      | jen@example.com     |     15 | $6,320.75| Feb 22    |||
|  |  | Carlos Ruiz        | carlos@example.com  |      1 |    $29.99| Feb 21    |||
|  |  +-------------------------------------------------------------------------+||
|  |                                                                              ||
|  |  Showing 1-10 of 1,247 customers              [< Prev]  1  2  3  [Next >]   ||
|  +------------------------------------------------------------------------------+|
+-----------------------------------------------------------------------------------+
```

---

## 7. Customer Detail (`/store/customers/:id`)

```
+-----------------------------------------------------------------------------------+
| [<] Store > Customers > Sarah Mitchell                                            |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  +-- MAIN CONTENT (col-span-2) -----+  +-- SIDEBAR (col-span-1) ----+           |
|  |                                    |  |                            |           |
|  |  +-- Order History -------------+  |  |  +-- Customer Stats ----+|           |
|  |  |                              |  |  |  |                       ||           |
|  |  |  Order #   | Date    | Total |  |  |  |  Total Orders:  5     ||           |
|  |  |  ----------|---------|-------|  |  |  |  Total Spent: $1,234  ||           |
|  |  |  RC-00342  | Feb 24  | $234  |  |  |  |  Avg. Order:  $246   ||           |
|  |  |  RC-00298  | Feb 15  | $180  |  |  |  |  Last Order: Feb 24  ||           |
|  |  |  RC-00245  | Feb 03  | $420  |  |  |  +-----------------------+|           |
|  |  |  RC-00201  | Jan 22  | $312  |  |  |                            |           |
|  |  |  RC-00150  | Jan 10  |  $88  |  |  |  +-- Contact Info ------+|           |
|  |  |                              |  |  |  |                       ||           |
|  |  +------------------------------+  |  |  |  sarah@example.com    ||           |
|  |                                    |  |  |  +1 (555) 123-4567   ||           |
|  +------------------------------------+  |  |  Member since Jan '26 ||           |
|                                          |  +-----------------------+|           |
|                                          |                            |           |
|                                          |  +-- Addresses ----------+|           |
|                                          |  |                       ||           |
|                                          |  |  Shipping (default):  ||           |
|                                          |  |  123 Main Street      ||           |
|                                          |  |  Apt 4B               ||           |
|                                          |  |  New York, NY 10001   ||           |
|                                          |  |                       ||           |
|                                          |  |  Billing:             ||           |
|                                          |  |  456 Oak Avenue       ||           |
|                                          |  |  Suite 200            ||           |
|                                          |  |  New York, NY 10002   ||           |
|                                          |  +-----------------------+|           |
|                                          +----------------------------+           |
+-----------------------------------------------------------------------------------+
```

---

## 8. Settings Page (`/store/settings`)

```
+-----------------------------------------------------------------------------------+
| [<] Store > Settings                                                              |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  [General]  [Payments]  [Shipping]  [Taxes]  [Checkout]                          |
|  =========                                                                        |
|                                                                                   |
|  +-- General Settings (active tab) -------------------------------------------+  |
|  |                                                                             |  |
|  |  Store Information                                                          |  |
|  |  -----------------                                                          |  |
|  |                                                                             |  |
|  |  Store Name                                                                 |  |
|  |  [My Awesome Store_______________________________]                          |  |
|  |                                                                             |  |
|  |  Store URL                                                                  |  |
|  |  [https://mystore.com___________________________]                           |  |
|  |                                                                             |  |
|  |  Currency                                                                   |  |
|  |  ---------                                                                  |  |
|  |                                                                             |  |
|  |  Currency         Symbol    Position                                        |  |
|  |  [USD - US Dollar v]  [$]   [Before price v]                                |  |
|  |                                                                             |  |
|  |  Thousand Separator    Decimal Separator    Decimal Places                  |  |
|  |  [, (comma) v]        [. (period) v]        [2 v]                           |  |
|  |                                                                             |  |
|  |  Units                                                                      |  |
|  |  -----                                                                      |  |
|  |                                                                             |  |
|  |  Weight Unit           Dimension Unit                                       |  |
|  |  [kg v]                [cm v]                                               |  |
|  |                                                                             |  |
|  |                                                    [Save Settings]           |  |
|  +-----------------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------------+
```

### 8.1 Payment Settings Tab

```
|  [General]  [Payments]  [Shipping]  [Taxes]  [Checkout]                          |
|              =========                                                            |
|                                                                                   |
|  +-- Payment Settings --------------------------------------------------------+  |
|  |                                                                             |  |
|  |  Stripe                                            [x] Enabled              |  |
|  |  ------                                                                     |  |
|  |                                                                             |  |
|  |  Mode:  ( ) Live   (*) Test                                                 |  |
|  |                                                                             |  |
|  |  Publishable Key                                                            |  |
|  |  [pk_test_51ABC________________________________]                            |  |
|  |                                                                             |  |
|  |  Secret Key                                                                 |  |
|  |  [sk_test_************************************]                             |  |
|  |                                                                             |  |
|  |  Webhook Secret                                                             |  |
|  |  [whsec_************************************]                               |  |
|  |                                                                             |  |
|  |  [Test Connection]  Connection status: Connected                            |  |
|  |                                                                             |  |
|  |                                                    [Save Settings]           |  |
|  +-----------------------------------------------------------------------------+  |
```

### 8.2 Shipping Settings Tab

```
|  [General]  [Payments]  [Shipping]  [Taxes]  [Checkout]                          |
|                          =========                                                |
|                                                                                   |
|  +-- Shipping Settings -------------------------------------------------------+  |
|  |                                                                             |  |
|  |  Shipping Zones                                  [+ Add Zone]               |  |
|  |                                                                             |  |
|  |  +-- Domestic (US) -------------------------------------------------------+||
|  |  |  Countries: United States                                [Edit] [x]    |||
|  |  |                                                                        |||
|  |  |  Methods:                                                              |||
|  |  |  +-------------------------------------------------------------------+|||
|  |  |  | [x] | Flat Rate Shipping   | flat_rate     | $9.99   | [Edit] [x] ||||
|  |  |  | [x] | Free Shipping        | free_shipping | $0.00   | [Edit] [x] ||||
|  |  |  |     |  (Orders over $50)   |               |         |            ||||
|  |  |  | [ ] | Express Shipping     | flat_rate     | $19.99  | [Edit] [x] ||||
|  |  |  +-------------------------------------------------------------------+|||
|  |  |  [+ Add Method]                                                        |||
|  |  +------------------------------------------------------------------------+||
|  |                                                                             |  |
|  |  +-- International -------------------------------------------------------+||
|  |  |  Countries: Canada, United Kingdom, Germany, France    [Edit] [x]      |||
|  |  |                                                                        |||
|  |  |  Methods:                                                              |||
|  |  |  +-------------------------------------------------------------------+|||
|  |  |  | [x] | International Standard | flat_rate   | $24.99  | [Edit] [x] ||||
|  |  |  | [x] | International Express  | flat_rate   | $49.99  | [Edit] [x] ||||
|  |  |  +-------------------------------------------------------------------+|||
|  |  |  [+ Add Method]                                                        |||
|  |  +------------------------------------------------------------------------+||
|  |                                                                             |  |
|  |                                                    [Save Settings]           |  |
|  +-----------------------------------------------------------------------------+  |
```

### 8.3 Tax Settings Tab

```
|  [General]  [Payments]  [Shipping]  [Taxes]  [Checkout]                          |
|                                      =======                                      |
|                                                                                   |
|  +-- Tax Settings -------------------------------------------------------------+  |
|  |                                                                             |  |
|  |  Tax Calculation                                                            |  |
|  |  ---------------                                                            |  |
|  |  Enable Taxes:       [x] Yes                                                |  |
|  |  Prices Include Tax: [ ] No                                                 |  |
|  |                                                                             |  |
|  |  Tax Rates                                           [+ Add Rate]           |  |
|  |                                                                             |  |
|  |  +------------------------------------------------------------------------+||
|  |  | Name            | Rate   | Country | State | Class    | Enabled         |||
|  |  |-----------------|--------|---------|-------|----------|----------|-------|||
|  |  | US Federal      | 0.00%  | US      | --    | Standard | [x]      | [x]  |||
|  |  | New York State  | 4.00%  | US      | NY    | Standard | [x]      | [x]  |||
|  |  | NYC Local       | 4.50%  | US      | NY    | Standard | [x]      | [x]  |||
|  |  | California      | 7.25%  | US      | CA    | Standard | [x]      | [x]  |||
|  |  | Texas           | 6.25%  | US      | TX    | Standard | [x]      | [x]  |||
|  |  | UK VAT          | 20.00% | GB      | --    | Standard | [x]      | [x]  |||
|  |  | DE MwSt         | 19.00% | DE      | --    | Standard | [x]      | [x]  |||
|  |  +------------------------------------------------------------------------+||
|  |                                                                             |  |
|  |                                                    [Save Settings]           |  |
|  +-----------------------------------------------------------------------------+  |
```

---

## 9. Color Legend for Status Badges

```
Order Status Colors:
  Pending     -> yellow/amber badge
  Confirmed   -> blue badge
  Processing  -> blue badge
  Shipped     -> purple/indigo badge
  Delivered   -> green badge
  Cancelled   -> red badge
  Refunded    -> gray badge

Payment Status Colors:
  Unpaid             -> yellow badge
  Paid               -> green badge
  Partially Refunded -> orange badge
  Refunded           -> gray badge

Product Status Colors:
  Draft     -> gray badge
  Published -> green badge
  Archived  -> yellow/amber badge

Stock Status Colors:
  In Stock     -> green badge
  Low Stock    -> yellow badge (with count)
  Out of Stock -> red badge
  On Backorder -> orange badge
```

---

## 10. Responsive Behavior Notes

All layouts follow these responsive rules:

- **Desktop (1280px+)**: Full 3-column grid on editor/detail pages, full DataTable columns visible
- **Tablet (768px-1279px)**: 2-column grid, sidebar stacks below main content, some table columns hidden
- **Mobile (< 768px)**: Single column, sidebar collapses to drawer, DataTable switches to card view, filters collapse to a filter drawer

The responsive behavior is handled by Tailwind responsive classes (e.g., `grid-cols-1 md:grid-cols-2 lg:grid-cols-3`) and the design system's `ResponsiveContainer` component.
