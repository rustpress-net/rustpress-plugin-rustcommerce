# RustCommerce Test Results

**Document Version**: 1.0
**Date**: _[YYYY-MM-DD]_
**Prepared By**: _[QA Engineer Name]_
**Project**: RustCommerce (RCOM-001)

---

## 1. Test Run Metadata

| Field | Value |
|-------|-------|
| **Test Run ID** | _[TR-YYYY-MM-DD-NNN]_ |
| **Date** | _[YYYY-MM-DD HH:MM UTC]_ |
| **Environment** | _[local / CI / staging / production-like]_ |
| **Branch** | _[Git branch name]_ |
| **Commit** | _[Git commit SHA]_ |
| **Build Version** | _[Cargo.toml version / tag]_ |
| **Database** | PostgreSQL _[version]_ |
| **Rust Version** | _[rustc --version output]_ |
| **Node Version** | _[node --version output]_ |
| **OS** | _[Operating system and version]_ |
| **Executed By** | _[Name or CI pipeline URL]_ |
| **Test Suite** | _[Unit / Integration / E2E / Full / Security / Performance]_ |

---

## 2. Summary

| Metric | Value |
|--------|:-----:|
| **Total Test Cases** | _[N]_ |
| **Passed** | _[N]_ |
| **Failed** | _[N]_ |
| **Skipped** | _[N]_ |
| **Blocked** | _[N]_ |
| **Pass Rate** | _[N%]_ |
| **Execution Duration** | _[HH:MM:SS]_ |

### Coverage

| Area | Line Coverage | Branch Coverage | Target | Met? |
|------|:------------:|:--------------:|:------:|:----:|
| Backend Unit (services/) | _[N%]_ | _[N%]_ | 80% | _[YES/NO]_ |
| Backend Unit (handlers/) | _[N%]_ | _[N%]_ | 70% | _[YES/NO]_ |
| Backend Integration | _[N%]_ | _[N%]_ | 60% | _[YES/NO]_ |
| Frontend Unit (stores/) | _[N%]_ | _[N%]_ | 80% | _[YES/NO]_ |
| Frontend Unit (components/) | _[N%]_ | _[N%]_ | 70% | _[YES/NO]_ |
| **Overall Backend** | _[N%]_ | _[N%]_ | **80%** | _[YES/NO]_ |
| **Overall Frontend** | _[N%]_ | _[N%]_ | **75%** | _[YES/NO]_ |

---

## 3. Results by Category

### 3.1 Product CRUD

| ID | Title | Status | Duration | Notes |
|----|-------|:------:|:--------:|-------|
| TC-PROD-001 | Create simple product | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-PROD-002 | Read product by ID (public) | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-PROD-003 | Update product price and description | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-PROD-004 | Delete (archive) product | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-PROD-005 | Create variable product with variants | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-PROD-006 | Add images to product | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-PROD-007 | Assign product to categories | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-PROD-008 | Search products by text | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-PROD-009 | Filter by price range and status | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-PROD-010 | Cursor-based pagination | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |

### 3.2 Cart Operations

| ID | Title | Status | Duration | Notes |
|----|-------|:------:|:--------:|-------|
| TC-CART-001 | Add item to cart | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CART-002 | Update cart item quantity | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CART-003 | Remove item from cart | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CART-004 | Guest cart with session ID | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CART-005 | Merge guest cart on login | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CART-006 | Apply coupon to cart | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CART-007 | Cart expiration cleanup | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CART-008 | Stock validation on add to cart | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |

### 3.3 Checkout Flow

| ID | Title | Status | Duration | Notes |
|----|-------|:------:|:--------:|-------|
| TC-CHK-001 | Happy path checkout (authenticated) | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CHK-002 | Guest checkout without login | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CHK-003 | Address validation rejects invalid | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CHK-004 | Shipping method selection | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CHK-005 | Payment success via webhook | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CHK-006 | Payment failure handling | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CHK-007 | Stock reservation during checkout | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CHK-008 | Concurrent checkout prevention | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CHK-009 | Coupon discount at checkout | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-CHK-010 | Order creation field completeness | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |

### 3.4 Order Management

| ID | Title | Status | Duration | Notes |
|----|-------|:------:|:--------:|-------|
| TC-ORD-001 | Valid status transition | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-ORD-002 | Invalid status transition rejected | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-ORD-003 | Process full refund | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-ORD-004 | Admin ships order with tracking | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-ORD-005 | Customer views order history | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-ORD-006 | Search orders by number | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-ORD-007 | Filter orders by status/date | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-ORD-008 | Order export to CSV | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |

### 3.5 Payment Integration

| ID | Title | Status | Duration | Notes |
|----|-------|:------:|:--------:|-------|
| TC-PAY-001 | PaymentIntent correct amount | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-PAY-002 | Webhook valid signature | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-PAY-003 | Webhook forged signature rejected | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-PAY-004 | Webhook replay rejected | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-PAY-005 | Refund via Stripe API | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-PAY-006 | Test mode uses test keys | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |

### 3.6 Authentication / Authorization

| ID | Title | Status | Duration | Notes |
|----|-------|:------:|:--------:|-------|
| TC-AUTH-001 | Guest access to public endpoints | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-AUTH-002 | Customer access own orders only | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-AUTH-003 | Admin permission scoping | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-AUTH-004 | Unauthenticated access rejected | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-AUTH-005 | Expired JWT rejected | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |

### 3.7 Business Logic

| ID | Title | Status | Duration | Notes |
|----|-------|:------:|:--------:|-------|
| TC-BIZ-001 | Tax calculation with compounding | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-BIZ-002 | Shipping free threshold | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-BIZ-003 | Stock reservation and expiration | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-BIZ-004 | Expired coupon rejected | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |
| TC-BIZ-005 | Order number uniqueness and format | _[PASS/FAIL/SKIP]_ | _[ms]_ | _[notes]_ |

---

## 4. Failed Test Details

_For each failed test, provide details below. Copy this block for each failure._

### _[TC-XXX-NNN]_: _[Test Title]_

| Field | Value |
|-------|-------|
| **Status** | FAIL |
| **Severity** | _[S1/S2/S3/S4]_ |
| **Bug Report ID** | _[BUG-NNN or link]_ |
| **Failure Description** | _[Brief description of what went wrong]_ |
| **Expected** | _[What was expected]_ |
| **Actual** | _[What actually happened]_ |
| **Stack Trace / Logs** | _[Relevant log output]_ |
| **Screenshot** | _[Path or link if applicable]_ |
| **Root Cause (if known)** | _[Analysis]_ |
| **Assigned To** | _[Developer name]_ |

---

## 5. Skipped / Blocked Test Details

| ID | Reason | Blocker |
|----|--------|---------|
| _[TC-XXX-NNN]_ | _[Reason for skip/block]_ | _[Blocking issue or dependency]_ |

---

## 6. Performance Test Results (if applicable)

| Scenario | Target | p50 | p95 | p99 | Status |
|----------|--------|:---:|:---:|:---:|:------:|
| Product listing (cached) | < 100ms | _[ms]_ | _[ms]_ | _[ms]_ | _[PASS/FAIL]_ |
| Product detail | < 150ms | _[ms]_ | _[ms]_ | _[ms]_ | _[PASS/FAIL]_ |
| Add to cart | < 200ms | _[ms]_ | _[ms]_ | _[ms]_ | _[PASS/FAIL]_ |
| Full checkout | < 3s | _[ms]_ | _[ms]_ | _[ms]_ | _[PASS/FAIL]_ |
| 100 concurrent shoppers | No errors | _[rps]_ | _[ms]_ | _[ms]_ | _[PASS/FAIL]_ |

---

## 7. Security Test Results (if applicable)

| Check | Tool | Status | Findings |
|-------|------|:------:|----------|
| Dependency vulnerabilities | `cargo-audit` | _[PASS/FAIL]_ | _[N findings]_ |
| Dependency vulnerabilities | `npm audit` | _[PASS/FAIL]_ | _[N findings]_ |
| OWASP ZAP scan | OWASP ZAP | _[PASS/FAIL]_ | _[N high, N medium, N low]_ |
| No card data in DB/logs | Manual + grep | _[PASS/FAIL]_ | _[details]_ |
| Webhook forgery protection | Custom tests | _[PASS/FAIL]_ | _[details]_ |
| Auth bypass | Custom tests | _[PASS/FAIL]_ | _[details]_ |
| Rate limiting | Custom tests | _[PASS/FAIL]_ | _[details]_ |

---

## 8. Sign-off

| Role | Name | Date | Verdict |
|------|------|------|:-------:|
| QA Lead | _[Name]_ | _[Date]_ | _[PASS / FAIL / CONDITIONAL]_ |
| Dev Lead | _[Name]_ | _[Date]_ | _[PASS / FAIL / CONDITIONAL]_ |

**Notes**: _[Any conditions, known issues, or caveats for the sign-off]_

---

*This template should be filled in for each test execution cycle. Archive completed test results with the test run ID.*
