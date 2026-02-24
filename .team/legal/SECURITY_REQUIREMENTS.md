# Security Requirements — Legal Perspective

**Document Version**: 1.0
**Date**: 2026-02-24
**Prepared By**: Legal/Compliance Attorney
**Project**: RustCommerce (RCOM-001)

---

## 1. Executive Summary

This document defines the security requirements for RustCommerce from a legal and regulatory compliance perspective. These requirements are derived from PCI-DSS obligations, GDPR's "security of processing" mandate (Article 32), general e-commerce security best practices, and the need to protect RustCommerce and its store operators from legal liability arising from security failures.

Each requirement is tagged with its regulatory origin and assigned a priority. Requirements marked as **MANDATORY** must be implemented before the MVP release. Requirements marked as **RECOMMENDED** should be implemented as soon as practical.

---

## 2. Payment Data Security

### SEC-PAY-01: No Raw Credit Card Storage [MANDATORY]

**Regulatory Basis**: PCI-DSS Requirement 3; Project Charter Constraint C8

**Requirement**: RustCommerce MUST NOT collect, process, transmit, store, or log raw credit card data (Primary Account Number, CVV/CVC, PIN, full magnetic stripe data) at any point in its architecture.

**Implementation Specifications**:

1. **No card input fields** on any server-rendered or API-served page. All card input must occur within Stripe Elements (an iframe served from Stripe's domain).
2. **No card data in API requests**. The checkout API must accept only Stripe Payment Intent IDs or Stripe Payment Method IDs, never raw card numbers.
3. **No card data in logs**. Payment-related log entries must exclude request/response bodies. Implement structured logging with an explicit allowlist of fields, not a denylist.
4. **No card data in error reports**. Error handling in payment flows must sanitize all error context before logging or reporting.
5. **No card data in database**. The `payments` table may store only:
   - Stripe payment intent ID
   - Stripe charge ID
   - Payment status
   - Amount and currency
   - Last 4 digits of card (provided by Stripe)
   - Card brand (Visa, Mastercard, etc.)
   - Payment timestamp
   - Failure reason code (if applicable)

**Verification**:
- [ ] Code review: grep codebase for field names like `card_number`, `pan`, `cvv`, `cvc`, `card_exp`, `security_code`
- [ ] Integration test: submit a test payment and verify no card data appears in database, logs, or error outputs
- [ ] CI check: automated scan for patterns matching credit card number format (Luhn-valid 13-19 digit sequences)

---

### SEC-PAY-02: Stripe Payment Intents API [MANDATORY]

**Regulatory Basis**: PCI-DSS; PSD2/SCA

**Requirement**: RustCommerce MUST use the Stripe Payment Intents API (not the legacy Charges API) for all payment processing.

**Rationale**: Payment Intents API supports Strong Customer Authentication (SCA) required by PSD2 in the EU, handles 3D Secure authentication flows, and provides better fraud prevention.

**Implementation Specifications**:

1. Create Payment Intent on the backend with order amount, currency, and metadata
2. Return the `client_secret` to the frontend
3. Frontend uses `stripe.confirmPayment()` with the client secret
4. Backend confirms payment status via webhook, NOT by trusting client-side confirmation
5. Support 3D Secure redirect flow for SCA-required payments

---

### SEC-PAY-03: Webhook Signature Verification [MANDATORY]

**Regulatory Basis**: PCI-DSS Requirement 6.5 (secure coding); fraud prevention

**Requirement**: All incoming Stripe webhook events MUST be verified using Stripe's webhook signature before processing.

**Implementation Specifications**:

1. Store the Stripe webhook signing secret securely (environment variable or secrets manager, never in source code or database)
2. On every incoming webhook request:
   a. Extract the `Stripe-Signature` header
   b. Verify the signature against the raw request body using the signing secret
   c. Verify the timestamp tolerance (reject events older than 5 minutes to prevent replay attacks)
3. Reject all requests that fail signature verification with HTTP 401
4. Log all verification failures with source IP for security monitoring
5. Never process webhook events that fail verification, regardless of payload content

**Verification**:
- [ ] Unit test: verify that forged webhook events are rejected
- [ ] Unit test: verify that expired webhook events (replay) are rejected
- [ ] Integration test: send valid Stripe test webhook and confirm processing
- [ ] Code review: verify no code path bypasses signature verification

---

### SEC-PAY-04: Rate Limiting on Payment Endpoints [MANDATORY]

**Regulatory Basis**: PCI-DSS Requirement 6.5 (application security); fraud prevention

**Requirement**: Payment and checkout API endpoints MUST implement rate limiting to prevent automated abuse, card testing attacks, and denial-of-service.

**Implementation Specifications**:

| Endpoint Category | Rate Limit | Window | Key |
|------------------|-----------|--------|-----|
| Checkout creation | 10 requests | per minute | per IP + session |
| Payment intent creation | 5 requests | per minute | per IP + session |
| Payment confirmation | 5 requests | per minute | per IP + session |
| Webhook endpoint | 100 requests | per minute | per IP |
| Cart operations | 30 requests | per minute | per IP + session |
| Login/Register | 10 requests | per minute | per IP |
| Password reset | 3 requests | per hour | per email |

**Additional Requirements**:
1. Return HTTP 429 (Too Many Requests) with `Retry-After` header when limit is exceeded
2. Log rate limit violations for security monitoring
3. Implement progressive penalties for repeat offenders (increasing lockout duration)
4. Rate limits must be configurable by store operators
5. Consider using a token bucket or sliding window algorithm

---

## 3. Encryption Requirements

### SEC-ENC-01: Encryption in Transit [MANDATORY]

**Regulatory Basis**: PCI-DSS Requirement 4; GDPR Article 32(1)(a)

**Requirement**: All data transmitted between clients and the RustCommerce server, and between RustCommerce and third-party services, MUST be encrypted using TLS 1.2 or higher.

**Implementation Specifications**:

1. **Server Configuration**: The RustPress server must be configured with TLS 1.2+ (TLS 1.3 preferred)
2. **HTTP Strict Transport Security (HSTS)**: Set `Strict-Transport-Security` header with `max-age=31536000; includeSubDomains`
3. **No Mixed Content**: All resources (scripts, stylesheets, images, API calls) must be served over HTTPS
4. **Stripe API Calls**: All outbound requests to `api.stripe.com` must use HTTPS (enforced by Stripe SDK)
5. **Database Connections**: PostgreSQL connections should use SSL when the database is on a separate host
6. **Cipher Suite**: Disable weak cipher suites (RC4, DES, 3DES, NULL ciphers, export ciphers)

**Verification**:
- [ ] SSL/TLS scan (e.g., Qualys SSL Labs) must achieve A or A+ rating
- [ ] Verify no HTTP endpoints accept unencrypted requests (redirect or refuse)
- [ ] Verify HSTS header is present on all responses

---

### SEC-ENC-02: Encryption at Rest [RECOMMENDED — MANDATORY for PII]

**Regulatory Basis**: GDPR Article 32(1)(a); PCI-DSS Requirement 3 (for payment data)

**Requirement**: Customer personally identifiable information (PII) stored in the database SHOULD be encrypted at rest. At minimum, the following fields must be protected:

**Mandatory Encryption (application-level or database-level)**:
- Customer email addresses
- Customer phone numbers
- Full street addresses (shipping and billing)
- Customer IP addresses stored for fraud prevention

**Acceptable Approaches (in order of preference)**:
1. **Application-level encryption**: Encrypt PII fields before database insertion using AES-256-GCM; store encrypted values in the database. This protects against database breach and unauthorized DBA access.
2. **Database-level transparent data encryption (TDE)**: PostgreSQL TDE or full-disk encryption on the database server. This protects against physical media theft but not against application-level breaches.
3. **Column-level encryption**: PostgreSQL `pgcrypto` extension for specific columns.

**Key Management**:
- Encryption keys MUST NOT be stored in the database alongside encrypted data
- Keys should be stored in environment variables, a secrets manager (e.g., HashiCorp Vault, AWS KMS), or a hardware security module (HSM)
- Implement key rotation capability (ability to re-encrypt data with a new key)
- Document key management procedures

**Trade-offs and Notes**:
- Application-level encryption prevents database-level searching on encrypted fields. Consider storing a hash for lookup fields (e.g., email hash for login) alongside the encrypted value.
- Clearly document the encryption approach chosen so store operators understand their data protection posture.

---

### SEC-ENC-03: Password Hashing [MANDATORY]

**Regulatory Basis**: GDPR Article 32; OWASP Authentication Best Practices

**Requirement**: All customer and admin passwords MUST be hashed using a secure, modern algorithm. Passwords must NEVER be stored in plain text or with reversible encryption.

**Implementation Specifications**:

1. **Algorithm**: Use Argon2id (preferred) or bcrypt (acceptable minimum)
2. **Argon2id Parameters** (if using Argon2id):
   - Memory: 64 MB minimum
   - Iterations: 3 minimum
   - Parallelism: 1 (or match available cores)
3. **bcrypt Parameters** (if using bcrypt):
   - Cost factor: 12 minimum
4. Each password hash must include a unique, cryptographically random salt
5. Never log password values, even in hashed form
6. Implement password strength validation (minimum 8 characters, discourage known-breached passwords)

---

## 4. Audit Logging Requirements

### SEC-LOG-01: Financial Transaction Audit Log [MANDATORY]

**Regulatory Basis**: PCI-DSS Requirement 10; tax/accounting regulations; dispute resolution

**Requirement**: All financial transactions and payment-related events MUST be logged in an immutable audit trail.

**Events to Log**:

| Event | Data to Record | Retention |
|-------|---------------|-----------|
| Payment Intent Created | order_id, amount, currency, customer_id, timestamp | 7 years |
| Payment Succeeded | payment_intent_id, charge_id, amount, timestamp | 7 years |
| Payment Failed | payment_intent_id, failure_code, failure_message, timestamp | 7 years |
| Refund Initiated | order_id, refund_amount, reason, admin_user_id, timestamp | 7 years |
| Refund Completed | refund_id, amount, timestamp | 7 years |
| Order Created | order_id, customer_id, total_amount, item_count, timestamp | 7 years |
| Order Status Changed | order_id, old_status, new_status, changed_by, timestamp | 7 years |
| Coupon Applied | order_id, coupon_code, discount_amount, timestamp | 7 years |
| Price Override (admin) | order_id, product_id, original_price, new_price, admin_user_id, timestamp | 7 years |

**Log Properties**:
1. **Immutability**: Audit log records must be append-only; no updates or deletions permitted
2. **Tamper Evidence**: Consider using sequential IDs with hash chains, or write to a separate audit database/table with restricted write access
3. **Timestamps**: Use UTC timestamps with timezone offset stored. Use database `TIMESTAMPTZ` type.
4. **Actor Attribution**: Every logged event must record WHO performed the action (customer_id, admin_user_id, or "system" for automated events)
5. **No PII in Logs**: Audit logs should reference customer_id, not customer name/email (to allow log retention beyond PII deletion)

---

### SEC-LOG-02: Security Event Logging [MANDATORY]

**Regulatory Basis**: PCI-DSS Requirement 10; GDPR Article 33 (breach detection)

**Requirement**: Security-relevant events MUST be logged for monitoring and incident response.

**Events to Log**:

| Event | Data to Record |
|-------|---------------|
| Failed login attempt | username/email (hashed), IP address, timestamp, user-agent |
| Account locked (too many failures) | account_id, IP address, timestamp |
| Password changed | customer_id, timestamp, IP address |
| Admin login | admin_user_id, IP address, timestamp |
| Admin permission change | target_user_id, old_permissions, new_permissions, admin_user_id, timestamp |
| Customer data export requested | customer_id, requested_by, timestamp |
| Customer data deletion requested | customer_id, requested_by, timestamp |
| Webhook signature verification failed | source_IP, timestamp, event_type_attempted |
| Rate limit exceeded | IP address, endpoint, timestamp |
| Suspicious order (fraud signals) | order_id, signals detected, timestamp |
| API authentication failure | endpoint, IP address, timestamp |
| Configuration changed (payment settings) | setting_key, changed_by, timestamp |

**Log Properties**:
1. Logs must be structured (JSON format preferred) for automated analysis
2. Logs must not contain passwords, tokens, or API keys
3. Log retention: minimum 1 year for security events, 90 days for high-volume events (rate limit violations)
4. Store operators should be advised to forward logs to a SIEM or log aggregation service

---

### SEC-LOG-03: Data Access Logging [RECOMMENDED]

**Regulatory Basis**: GDPR Article 30 (records of processing); principle of accountability

**Requirement**: Access to customer personal data by admin users SHOULD be logged.

**Events to Log**:

| Event | Data to Record |
|-------|---------------|
| Admin viewed customer detail | admin_user_id, customer_id, timestamp |
| Admin exported customer data | admin_user_id, customer_id, export_format, timestamp |
| Admin modified customer data | admin_user_id, customer_id, fields_changed, timestamp |
| Bulk customer data export | admin_user_id, record_count, timestamp |
| Customer list accessed with filters | admin_user_id, filter_criteria, result_count, timestamp |

---

## 5. Authentication and Authorization

### SEC-AUTH-01: Authentication Security [MANDATORY]

**Regulatory Basis**: PCI-DSS Requirements 7, 8; OWASP Authentication

**Requirement**: All authentication mechanisms must follow security best practices.

**Implementation Specifications**:

1. **JWT Token Security**:
   - Sign tokens with RS256 or EdDSA (asymmetric) or HS256 with a strong secret (minimum 256 bits)
   - Set reasonable expiration (access tokens: 15-60 minutes; refresh tokens: 7-30 days)
   - Include `iss`, `exp`, `iat`, `sub` claims at minimum
   - Store refresh tokens securely (HttpOnly, Secure, SameSite=Strict cookies)
   - Implement token revocation capability (for logout, password change, account compromise)

2. **Session Management**:
   - Generate cryptographically random session identifiers (minimum 128 bits of entropy)
   - Regenerate session ID after authentication (prevent session fixation)
   - Invalidate session on logout, password change, and after inactivity timeout
   - Set session cookies with `HttpOnly`, `Secure`, `SameSite=Strict` (or `Lax` for cross-site checkout flows)

3. **Account Security**:
   - Implement account lockout after 5-10 consecutive failed login attempts (with exponential backoff)
   - Provide password reset via secure, time-limited token (1 hour maximum)
   - Do not reveal whether an email is registered (use generic error messages: "If an account exists...")
   - Support multi-factor authentication (TOTP) for admin accounts [RECOMMENDED]

---

### SEC-AUTH-02: Authorization and Access Control [MANDATORY]

**Regulatory Basis**: PCI-DSS Requirement 7 (restrict access by need-to-know); GDPR principle of data minimization

**Requirement**: Implement role-based access control (RBAC) with principle of least privilege.

**Implementation Specifications**:

1. **Role Definitions** (minimum):

   | Role | Permissions |
   |------|------------|
   | **Customer** | View/edit own profile, view own orders, manage own cart, write reviews |
   | **Store Manager** | All customer permissions + manage products, view all orders, manage inventory |
   | **Store Admin** | All manager permissions + manage customers, process refunds, configure settings |
   | **Super Admin** | All store admin permissions + manage store users, view audit logs, configure payment gateway |

2. **Authorization Checks**:
   - Every API endpoint must verify the caller's role and permissions
   - Implement authorization at the service layer, not just the handler layer
   - Customers must only access their own data (enforce `customer_id` matching on all customer-facing endpoints)
   - Admin endpoints must verify admin role AND specific permission
   - All authorization failures must return HTTP 403 (not 404, to avoid information leakage, unless deliberately masking resource existence)

3. **Insecure Direct Object Reference (IDOR) Prevention**:
   - Never rely solely on user-supplied IDs for authorization
   - Always verify that the authenticated user has access to the requested resource
   - Use UUIDs (not sequential integers) for all resource IDs to reduce enumeration risk

---

## 6. Input Validation and Output Encoding

### SEC-INPUT-01: Input Validation [MANDATORY]

**Regulatory Basis**: PCI-DSS Requirement 6.5; OWASP Top 10 (Injection)

**Requirement**: All user input MUST be validated before processing.

**Implementation Specifications**:

1. **SQL Injection Prevention**:
   - Use parameterized queries exclusively (sqlx prepared statements)
   - NEVER construct SQL queries by string concatenation with user input
   - sqlx's compile-time checked queries provide additional safety

2. **Cross-Site Scripting (XSS) Prevention**:
   - Sanitize/escape all user-generated content before rendering (product descriptions, reviews, customer names)
   - Use Content-Security-Policy headers to restrict script execution
   - React's JSX escaping provides default XSS protection in the admin UI; avoid `dangerouslySetInnerHTML`

3. **Field-Level Validation**:

   | Field | Validation Rules |
   |-------|-----------------|
   | Email | Valid email format; maximum 254 characters |
   | Phone | Valid phone format; maximum 20 characters |
   | Name | Maximum 100 characters; no HTML/script tags |
   | Address | Maximum 500 characters per field; no HTML/script tags |
   | SKU | Alphanumeric + hyphens; maximum 50 characters |
   | Price | Positive decimal; maximum 99999999.99; minimum 0.01 (or 0 for free products) |
   | Quantity | Positive integer; maximum 9999 |
   | Coupon code | Alphanumeric + hyphens; maximum 50 characters |
   | Review text | Maximum 5000 characters; HTML stripped |
   | Product description | Maximum 50000 characters; allowlisted HTML tags only |
   | URL/slug | Valid URL characters; maximum 200 characters |

4. **Request Size Limits**:
   - Maximum request body: 10 MB (for product image uploads, adjust as needed)
   - Maximum JSON payload: 1 MB for API endpoints (excluding file uploads)
   - Maximum URL length: 2048 characters

---

### SEC-INPUT-02: CSRF Protection [MANDATORY]

**Regulatory Basis**: PCI-DSS Requirement 6.5; OWASP Top 10

**Requirement**: All state-changing requests (POST, PUT, PATCH, DELETE) MUST be protected against Cross-Site Request Forgery attacks.

**Implementation Specifications**:

1. For API endpoints using JWT authentication: CSRF protection is inherent if tokens are sent via `Authorization` header (not cookies)
2. For cookie-based sessions (guest checkout): Implement CSRF tokens
   - Generate a cryptographically random CSRF token per session
   - Include the token in a cookie (`SameSite=Strict`) and require it in a request header (`X-CSRF-Token`)
   - Verify token match on all state-changing requests
3. Set `SameSite` attribute on all cookies (`Strict` for session cookies; `Lax` if cross-site navigation is needed)
4. Validate `Origin` and `Referer` headers as an additional layer

---

## 7. API Security

### SEC-API-01: API Security Headers [MANDATORY]

**Regulatory Basis**: OWASP Secure Headers Project; defense in depth

**Requirement**: All API responses MUST include appropriate security headers.

**Required Headers**:

```
Strict-Transport-Security: max-age=31536000; includeSubDomains
Content-Security-Policy: default-src 'self'; script-src 'self' js.stripe.com; frame-src js.stripe.com
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 0
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: camera=(), microphone=(), geolocation=()
Cache-Control: no-store (for responses containing PII or payment data)
```

---

### SEC-API-02: API Versioning and Deprecation [RECOMMENDED]

**Regulatory Basis**: Best practice for maintaining security patch capability

**Requirement**: API endpoints should be versioned (e.g., `/api/v1/`) to allow security patches to be deployed to new versions while maintaining backward compatibility.

**Specifications**:
1. Use URL-based versioning (`/api/v1/rustcommerce/`)
2. Document deprecation timeline when new versions are released
3. Maintain security patches for deprecated versions for a minimum of 6 months
4. Communicate deprecation to store operators via admin dashboard notifications

---

### SEC-API-03: Error Handling Security [MANDATORY]

**Regulatory Basis**: OWASP (information leakage); PCI-DSS Requirement 6.5

**Requirement**: API error responses MUST NOT leak sensitive information.

**Implementation Specifications**:

1. **Production Error Responses**:
   - Return generic error messages to clients (e.g., "Payment processing failed" not "Stripe API key invalid: sk_live_...")
   - Use error codes that clients can handle programmatically
   - Never expose stack traces, database errors, file paths, or internal IPs in API responses
   - Log detailed error information server-side only

2. **Error Response Format**:
   ```json
   {
     "error": {
       "code": "PAYMENT_FAILED",
       "message": "Payment could not be processed. Please try again or use a different payment method.",
       "request_id": "req_abc123"
     }
   }
   ```

3. **Sensitive Data Redaction**:
   - Redact API keys, tokens, and secrets from all error logs
   - Redact customer PII from error reports sent to error tracking services
   - Use request IDs for correlation between client-facing errors and server-side logs

---

## 8. Infrastructure Security Requirements

### SEC-INFRA-01: Secrets Management [MANDATORY]

**Regulatory Basis**: PCI-DSS Requirement 3 (protect stored data); general security best practice

**Requirement**: All sensitive configuration values MUST be managed securely.

**Sensitive Values (must NEVER appear in source code, version control, or database)**:
- Stripe secret API key (`sk_live_*`, `sk_test_*`)
- Stripe webhook signing secret (`whsec_*`)
- Database connection string (contains password)
- JWT signing key/secret
- Encryption keys for PII at rest
- SMTP credentials (for email)
- Any third-party API keys

**Acceptable Storage Methods**:
1. Environment variables (minimum acceptable)
2. Secrets manager (HashiCorp Vault, AWS Secrets Manager, etc.) [RECOMMENDED]
3. Kubernetes secrets (if deploying on K8s)

**Prohibited Storage**:
- Source code (including configuration files checked into version control)
- Database tables
- Docker images (use runtime environment injection, not build-time)
- Client-side code (never expose secret keys to the browser)

**CI/CD Secrets**:
- Use CI platform's secrets management (GitHub Actions secrets, etc.)
- Never echo or print secrets in CI logs
- Rotate CI secrets periodically

---

### SEC-INFRA-02: Dependency Security [MANDATORY]

**Regulatory Basis**: OWASP Top 10 (A06: Vulnerable and Outdated Components); supply chain security

**Requirement**: All dependencies must be monitored for known vulnerabilities.

**Implementation Specifications**:

1. **Rust Dependencies**:
   - Run `cargo audit` in CI on every build
   - Run `cargo deny check advisories` in CI on every build
   - Pin dependency versions in `Cargo.lock` (committed to version control)
   - Review dependency updates for security implications before upgrading

2. **JavaScript Dependencies**:
   - Run `npm audit` in CI on every build
   - Use `package-lock.json` (committed to version control)
   - Consider using Socket.dev or Snyk for advanced supply chain analysis
   - Audit new dependencies before adding to `package.json`

3. **Automated Scanning**:
   - Enable Dependabot or Renovate for automated security update PRs
   - Configure to auto-merge patch-level security updates after CI passes
   - Review and merge minor/major security updates within 48 hours of notification

4. **Container Images** (if Docker is used):
   - Scan Docker images with Trivy or similar tool
   - Use minimal base images (e.g., `distroless` or `alpine`)
   - Update base images regularly

---

## 9. Security Testing Requirements

### SEC-TEST-01: Mandatory Security Testing [MANDATORY]

**Regulatory Basis**: PCI-DSS Requirement 6; OWASP Testing Guide

**Requirement**: The following security tests MUST be performed before MVP release and on an ongoing basis.

| Test Type | Frequency | Scope | Responsible |
|-----------|-----------|-------|-------------|
| Static Application Security Testing (SAST) | Every CI build | Rust + TypeScript code | DevOps (automated) |
| Dependency vulnerability scan | Every CI build | All dependencies | DevOps (automated) |
| Manual code review (security focus) | Every PR touching payment/auth/customer code | Payment, auth, customer modules | Backend Lead + QA |
| Penetration test (payment flow) | Before MVP release, then annually | Checkout, payment, webhook endpoints | External or QA Lead |
| OWASP Top 10 assessment | Before MVP release | Entire application | QA Lead |
| Stripe integration security review | Before MVP release | Stripe integration code | Backend Lead |

### SEC-TEST-02: Security Test Cases [MANDATORY]

The following specific test cases must be included in the test suite:

**Payment Security**:
- [ ] Verify no card data in database after payment
- [ ] Verify no card data in application logs after payment
- [ ] Verify forged webhooks are rejected
- [ ] Verify expired/replayed webhooks are rejected
- [ ] Verify rate limiting on checkout endpoint
- [ ] Verify payment amounts cannot be tampered with client-side

**Authentication/Authorization**:
- [ ] Verify customers cannot access other customers' orders
- [ ] Verify customers cannot access admin endpoints
- [ ] Verify admin permissions are enforced (store manager cannot access super admin endpoints)
- [ ] Verify brute force login protection works
- [ ] Verify password reset tokens expire after use or timeout
- [ ] Verify session invalidation on password change

**Input Validation**:
- [ ] Verify SQL injection attempts are blocked on all endpoints
- [ ] Verify XSS payloads are sanitized in product descriptions and reviews
- [ ] Verify CSRF protection on state-changing endpoints
- [ ] Verify file upload restrictions (type, size) on product images
- [ ] Verify price fields reject negative values and non-numeric input

**Data Protection**:
- [ ] Verify customer data export includes all PII
- [ ] Verify customer data deletion anonymizes all records
- [ ] Verify error responses do not leak sensitive information
- [ ] Verify API keys are not exposed in client-side code or API responses

---

## 10. Incident Response Preparation

### SEC-IR-01: Security Incident Response [RECOMMENDED]

**Regulatory Basis**: GDPR Articles 33-34 (72-hour breach notification); PCI-DSS Requirement 12.10

**Requirement**: Prepare incident response documentation and capabilities.

**Documentation**:
1. Provide an incident response plan template for store operators
2. Document escalation procedures for security vulnerabilities reported to the project
3. Establish a security contact (e.g., security@rustpress.net)
4. Publish a SECURITY.md file in the repository with vulnerability reporting instructions

**Technical Capabilities**:
1. Audit logs must be sufficient to determine breach scope (what data was accessed, by whom, when)
2. Admin must be able to force-logout all sessions (emergency session invalidation)
3. Admin must be able to disable payment processing (emergency kill switch)
4. Webhook endpoint must be disableable without taking down the entire plugin
5. Provide a "maintenance mode" that displays a custom message and disables checkout

---

## 11. Compliance Summary Matrix

| Requirement | PCI-DSS | GDPR | OWASP | Priority |
|------------|---------|------|-------|----------|
| SEC-PAY-01: No card storage | Req 3 | -- | A01 | MANDATORY |
| SEC-PAY-02: Payment Intents API | Req 6 | -- | -- | MANDATORY |
| SEC-PAY-03: Webhook verification | Req 6.5 | -- | A07 | MANDATORY |
| SEC-PAY-04: Rate limiting | Req 6.5 | -- | A04 | MANDATORY |
| SEC-ENC-01: TLS in transit | Req 4 | Art 32 | A02 | MANDATORY |
| SEC-ENC-02: Encryption at rest | Req 3 | Art 32 | A02 | RECOMMENDED (MANDATORY for PII) |
| SEC-ENC-03: Password hashing | Req 8 | Art 32 | A02 | MANDATORY |
| SEC-LOG-01: Financial audit log | Req 10 | -- | A09 | MANDATORY |
| SEC-LOG-02: Security event log | Req 10 | Art 33 | A09 | MANDATORY |
| SEC-LOG-03: Data access log | -- | Art 30 | A09 | RECOMMENDED |
| SEC-AUTH-01: Authentication security | Req 8 | Art 32 | A07 | MANDATORY |
| SEC-AUTH-02: Authorization (RBAC) | Req 7 | Art 25 | A01 | MANDATORY |
| SEC-INPUT-01: Input validation | Req 6.5 | -- | A03 | MANDATORY |
| SEC-INPUT-02: CSRF protection | Req 6.5 | -- | A01 | MANDATORY |
| SEC-API-01: Security headers | -- | -- | -- | MANDATORY |
| SEC-API-02: API versioning | -- | -- | -- | RECOMMENDED |
| SEC-API-03: Error handling | Req 6.5 | -- | A01 | MANDATORY |
| SEC-INFRA-01: Secrets management | Req 3 | -- | A02 | MANDATORY |
| SEC-INFRA-02: Dependency security | -- | -- | A06 | MANDATORY |
| SEC-TEST-01: Security testing | Req 6 | -- | -- | MANDATORY |
| SEC-TEST-02: Security test cases | Req 6 | -- | -- | MANDATORY |
| SEC-IR-01: Incident response | Req 12.10 | Art 33 | -- | RECOMMENDED |

---

## 12. Review and Maintenance

This document must be reviewed and updated:
- When new features are added that involve payment, authentication, or customer data
- When dependencies are significantly changed
- When new security vulnerabilities are discovered in the technology stack
- At minimum quarterly during active development
- Annually after stable release

**Approval**:

| Role | Name | Date | Status |
|------|------|------|--------|
| Legal/Compliance | Legal Agent | 2026-02-24 | Approved |
| Backend Lead | Pending | -- | -- |
| DevOps Lead | Pending | -- | -- |
| QA Lead | Pending | -- | -- |

---

*These security requirements represent the minimum acceptable security posture for an e-commerce plugin handling customer PII and facilitating financial transactions. Exceeding these requirements is encouraged. Falling below them creates unacceptable legal and regulatory risk.*
