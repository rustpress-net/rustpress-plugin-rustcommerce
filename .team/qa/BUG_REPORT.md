# RustCommerce Bug Report Template

**Document Version**: 1.0
**Date**: 2026-02-24
**Project**: RustCommerce (RCOM-001)

---

## How to Use This Template

Copy the template below for each new bug report. Fill in all applicable fields. File bugs in the project issue tracker with the `bug` label and the appropriate severity label.

---

## Bug Report Template

### _[BUG-NNN]_: _[Short descriptive title]_

---

#### 1. Summary

| Field | Value |
|-------|-------|
| **Bug ID** | BUG-_[NNN]_ |
| **Title** | _[Short, descriptive title -- e.g., "Coupon discount not applied to grand total during checkout"]_ |
| **Severity** | _[S1 - Critical / S2 - Major / S3 - Minor / S4 - Trivial]_ |
| **Priority** | _[P0 / P1 / P2 / P3]_ |
| **Status** | _[New / Confirmed / In Progress / Fixed / Verified / Closed / Won't Fix]_ |
| **Reporter** | _[Name]_ |
| **Assignee** | _[Name or Unassigned]_ |
| **Date Reported** | _[YYYY-MM-DD]_ |
| **Date Resolved** | _[YYYY-MM-DD or N/A]_ |
| **Related Test Case** | _[TC-XXX-NNN or N/A]_ |
| **Component** | _[e.g., cart_service, checkout_handler, payment_service, admin_ui, product_editor]_ |
| **Labels** | _[e.g., bug, payment, checkout, security, performance]_ |

---

#### 2. Severity Definitions

| Level | Name | Description | Examples |
|-------|------|-------------|----------|
| S1 | Critical | System unusable, data loss, security vulnerability, payment errors | Payment charged but order not created; stock goes negative; SQL injection possible |
| S2 | Major | Feature broken, no workaround, blocks key user flow | Checkout fails for all users; admin cannot create products; orders stuck in wrong status |
| S3 | Minor | Feature degraded but workaround exists, non-critical functionality broken | Filter not working on product list (can still browse manually); coupon error message unclear |
| S4 | Trivial | Cosmetic issue, typo, minor UI misalignment | Button color wrong; spacing off by a few pixels; tooltip text has typo |

---

#### 3. Environment

| Field | Value |
|-------|-------|
| **Environment** | _[local / CI / staging / production]_ |
| **OS** | _[e.g., Ubuntu 22.04 / macOS 14.3 / Windows 11]_ |
| **Browser** | _[e.g., Chrome 122 / Firefox 123 / Safari 17 -- if frontend bug]_ |
| **Rust Version** | _[e.g., rustc 1.76.0]_ |
| **Node Version** | _[e.g., v20.11.0 -- if frontend bug]_ |
| **Database** | _[e.g., PostgreSQL 16.2]_ |
| **RustPress Version** | _[e.g., 0.5.0]_ |
| **RustCommerce Version** | _[e.g., 0.1.0-alpha or commit SHA]_ |
| **Branch / Commit** | _[Git branch and commit SHA]_ |

---

#### 4. Reproducibility

| Field | Value |
|-------|-------|
| **Reproducible** | _[Always / Sometimes / Rarely / Once]_ |
| **Frequency** | _[e.g., 100%, 50%, happened once out of 10 attempts]_ |

---

#### 5. Steps to Reproduce

_Provide clear, numbered steps that reliably reproduce the bug. Include exact API requests, payloads, and parameters._

1. _[Step 1 -- e.g., "Create a product with price $100.00 and stock_quantity = 5"]_
2. _[Step 2 -- e.g., "Add the product to cart with quantity 2"]_
3. _[Step 3 -- e.g., "Apply coupon code SAVE10 (10% discount)"]_
4. _[Step 4 -- e.g., "Initiate checkout and proceed to payment intent"]_
5. _[Step 5 -- e.g., "Observe the payment intent amount"]_

**Curl / API Example** (if applicable):

```bash
# Example API request that triggers the bug
curl -X POST http://localhost:8080/api/v1/rustcommerce/checkout/payment-intent \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -H "X-Session-ID: <session-id>" \
  -d '{}'
```

---

#### 6. Expected Result

_Describe what should happen._

> _[e.g., "Payment intent amount should be 9000 cents ($90.00 = $100.00 subtotal - $10.00 discount)."]_

---

#### 7. Actual Result

_Describe what actually happens._

> _[e.g., "Payment intent amount is 10000 cents ($100.00). The coupon discount is not subtracted from the payment amount."]_

---

#### 8. Evidence

**Screenshots / Video**:

_[Attach screenshots or screen recordings if applicable. For UI bugs, always include a screenshot.]_

- _[screenshot_1.png -- description]_
- _[screenshot_2.png -- description]_

**Logs**:

_[Include relevant log output. Redact any sensitive information (tokens, keys, PII).]_

```
[2026-02-24T10:15:23Z INFO  rustcommerce::services::checkout_service] Creating payment intent for checkout session abc-123
[2026-02-24T10:15:23Z DEBUG rustcommerce::services::checkout_service] Cart totals: subtotal=100.00, discount=10.00, tax=0.00, shipping=0.00, grand_total=90.00
[2026-02-24T10:15:23Z ERROR rustcommerce::services::payment_service] Stripe amount calculation: using subtotal (100.00) instead of grand_total
```

**Database State** (if applicable):

```sql
-- Relevant query to show incorrect data state
SELECT id, grand_total, discount_total FROM rc_orders WHERE id = 'abc-123';
-- Result: grand_total=100.00, discount_total=10.00 (grand_total should be 90.00)
```

**API Response**:

```json
{
  "error": {
    "code": "...",
    "message": "...",
    "status": 500
  }
}
```

---

#### 9. Impact Analysis

| Question | Answer |
|----------|--------|
| **Who is affected?** | _[e.g., "All customers applying coupons at checkout"]_ |
| **How many users impacted?** | _[e.g., "Any customer using a coupon -- estimated 20% of checkouts"]_ |
| **Data integrity risk?** | _[e.g., "Customers are being overcharged by the discount amount"]_ |
| **Security risk?** | _[e.g., "None" or "PII exposed in error response"]_ |
| **Workaround available?** | _[e.g., "Customers can remove the coupon and still checkout at full price"]_ |

---

#### 10. Additional Context

_Any other information that might help diagnose or fix the bug._

- _[e.g., "This only happens with percentage coupons, fixed_cart coupons work correctly"]_
- _[e.g., "Introduced in commit abc1234 which refactored the payment intent creation"]_
- _[e.g., "Related to TC-CHK-009 which was testing coupon discount at checkout"]_
- _[e.g., "Similar to BUG-042 which was fixed last week -- possible regression"]_

---

#### 11. Resolution (filled after fix)

| Field | Value |
|-------|-------|
| **Root Cause** | _[e.g., "payment_service.rs line 45 used `subtotal` instead of `grand_total` when creating the PaymentIntent amount"]_ |
| **Fix Description** | _[e.g., "Changed `create_payment_intent` to use `totals.grand_total` instead of `totals.subtotal`"]_ |
| **Fix Commit** | _[Git commit SHA]_ |
| **Fix PR** | _[PR URL]_ |
| **Regression Test Added** | _[YES/NO -- link to test if YES]_ |
| **Verified By** | _[QA name and date]_ |
| **Verification Method** | _[e.g., "Re-ran TC-CHK-009 -- PASS. Also manually tested with multiple coupon types."]_ |

---

## Quick Reference: Bug Lifecycle

```
New -> Confirmed -> In Progress -> Fixed -> Verified -> Closed
                                      |                   ^
                                      +-- Won't Fix ----->|
                                      +-- Duplicate ----->|
                                      +-- Cannot Reproduce -->|
```

**Status Definitions**:

| Status | Description |
|--------|-------------|
| New | Bug reported, not yet triaged |
| Confirmed | Bug reproduced and accepted |
| In Progress | Developer is working on a fix |
| Fixed | Fix committed and available for verification |
| Verified | QA has verified the fix resolves the issue |
| Closed | Bug fully resolved and closed |
| Won't Fix | Decision made not to fix (with justification) |

---

*Use this template consistently for all bug reports. Thorough bug reports reduce investigation time and accelerate fixes.*
