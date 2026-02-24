# E-Commerce Compliance Checklist — RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Prepared By**: Legal/Compliance Attorney
**Project**: RustCommerce (RCOM-001)

---

## 1. Executive Summary

RustCommerce enables RustPress sites to function as online stores. As an e-commerce platform, it must be built with awareness of multiple regulatory frameworks spanning payment security, data privacy, tax obligations, and accessibility. This document provides a comprehensive compliance checklist that the development team must address and that store operators must be informed about.

**Critical Distinction**: RustCommerce is a *software plugin*, not a store operator. Many compliance obligations fall on the store operator (the business using RustCommerce), not on the plugin developer. However, the plugin MUST be built to *enable* compliance and MUST NOT make compliance impossible through design choices.

---

## 2. PCI-DSS Compliance

### 2.1 What PCI-DSS Is

The Payment Card Industry Data Security Standard (PCI-DSS) is a set of security requirements for organizations that handle credit card data. Non-compliance can result in fines of $5,000 to $100,000 per month.

### 2.2 RustCommerce's PCI Scope — Delegation to Stripe

RustCommerce delegates all payment card processing to Stripe. This means:

| Aspect | Responsibility | Notes |
|--------|---------------|-------|
| Card number collection | **Stripe** (via Stripe.js / Elements) | Card data never touches RustCommerce servers |
| Card data storage | **Stripe** | RustCommerce stores ZERO card data |
| Payment processing | **Stripe** | RustCommerce sends payment intents, Stripe handles the rest |
| PCI-DSS certification | **Stripe** (Level 1 Service Provider) | Stripe maintains its own PCI compliance |
| SAQ-A eligibility | **Store Operator** | By using Stripe Elements, stores qualify for the simplest Self-Assessment Questionnaire |

### 2.3 PCI-DSS Checklist for RustCommerce

- [ ] **PC-01**: NEVER collect, transmit, or store raw credit card numbers (PAN) in any database table, log file, or API response
- [ ] **PC-02**: NEVER log card numbers, CVV, or full magnetic stripe data, even in debug/development mode
- [ ] **PC-03**: Use Stripe.js / Stripe Elements on the frontend so card data goes directly to Stripe, never to RustCommerce backend
- [ ] **PC-04**: Store only Stripe payment intent IDs, charge IDs, and last-4-digits of card (for display) in the payments table
- [ ] **PC-05**: Use HTTPS (TLS 1.2+) for all API communication, especially checkout and payment endpoints
- [ ] **PC-06**: Verify Stripe webhook signatures on all incoming webhook events
- [ ] **PC-07**: Implement rate limiting on payment-related endpoints
- [ ] **PC-08**: Document in user-facing materials that RustCommerce does NOT store card data and that Stripe handles PCI compliance
- [ ] **PC-09**: Include Stripe's PCI compliance badge/link in checkout flow (optional but recommended)
- [ ] **PC-10**: Log all payment events (creation, success, failure, refund) with timestamps for audit trail, WITHOUT including sensitive card data

### 2.4 What Store Operators Must Do

Store operators using RustCommerce should be informed they must:
- Complete SAQ-A (Self-Assessment Questionnaire A) annually if using Stripe Elements
- Maintain a valid SSL/TLS certificate on their domain
- Not customize the checkout in ways that expose card data to their server
- Report any suspected breaches to their acquiring bank and Stripe

---

## 3. GDPR Compliance (EU General Data Protection Regulation)

### 3.1 Applicability

GDPR applies to RustCommerce stores that:
- Are established in the EU/EEA, OR
- Offer goods or services to individuals in the EU/EEA, OR
- Monitor the behavior of individuals in the EU/EEA

Given that e-commerce stores are typically accessible worldwide, most stores using RustCommerce should assume GDPR applies.

### 3.2 GDPR Checklist — Plugin Architecture Requirements

#### 3.2.1 Lawful Basis for Processing

- [ ] **GD-01**: Document lawful basis for each type of data processing (contract performance for orders, legitimate interest for analytics, consent for marketing)
- [ ] **GD-02**: Implement consent collection mechanism for marketing emails and non-essential cookies
- [ ] **GD-03**: Store consent records with timestamp, scope, and method of consent

#### 3.2.2 Right to Access (Article 15)

- [ ] **GD-04**: Provide API endpoint for customers to export their personal data (`GET /api/v1/rustcommerce/customers/{id}/export`)
- [ ] **GD-05**: Export must include: profile data, order history, addresses, reviews, wishlist items
- [ ] **GD-06**: Export format should be machine-readable (JSON or CSV)
- [ ] **GD-07**: Admin UI must provide a "Data Export" button on customer detail page

#### 3.2.3 Right to Erasure / Right to Be Forgotten (Article 17)

- [ ] **GD-08**: Provide API endpoint for customer data deletion (`DELETE /api/v1/rustcommerce/customers/{id}`)
- [ ] **GD-09**: Deletion must anonymize or remove: name, email, phone, addresses, IP addresses
- [ ] **GD-10**: Order records may be retained for legal/tax purposes but must be anonymized (replace customer name with "Deleted Customer", remove personal details)
- [ ] **GD-11**: Payment records referencing Stripe may be retained (transaction IDs are not personal data)
- [ ] **GD-12**: Reviews must be anonymized or deleted upon customer deletion request
- [ ] **GD-13**: Admin UI must provide a "Delete Customer Data" button with confirmation dialog explaining what will be retained for legal reasons

#### 3.2.4 Right to Rectification (Article 16)

- [ ] **GD-14**: Customers must be able to update their profile information (name, email, addresses)
- [ ] **GD-15**: Admin must be able to update customer data on their behalf

#### 3.2.5 Right to Data Portability (Article 20)

- [ ] **GD-16**: Data export (GD-04 through GD-07) satisfies this requirement
- [ ] **GD-17**: Export format must be structured, commonly used, and machine-readable

#### 3.2.6 Data Minimization (Article 5(1)(c))

- [ ] **GD-18**: Collect only data necessary for order fulfillment (do not require fields like date of birth, gender, etc.)
- [ ] **GD-19**: Mark optional fields clearly in checkout and account forms
- [ ] **GD-20**: Do not collect IP addresses beyond what is needed for fraud prevention

#### 3.2.7 Data Protection by Design (Article 25)

- [ ] **GD-21**: Encrypt customer PII at rest in the database (or document encryption strategy)
- [ ] **GD-22**: All API communication over HTTPS (TLS 1.2+)
- [ ] **GD-23**: Implement role-based access control for customer data (not all admin roles need access)
- [ ] **GD-24**: Implement data retention policies with automatic anonymization after retention period

#### 3.2.8 Data Processing Records (Article 30)

- [ ] **GD-25**: Provide documentation template for store operators to maintain their Records of Processing Activities (ROPA)
- [ ] **GD-26**: Log data access events for audit purposes

#### 3.2.9 Data Breach Notification (Articles 33-34)

- [ ] **GD-27**: Implement audit logging that would enable breach scope assessment
- [ ] **GD-28**: Document incident response procedures for store operators (72-hour notification requirement to supervisory authority)

---

## 4. CCPA Compliance (California Consumer Privacy Act / CPRA)

### 4.1 Applicability

CCPA/CPRA applies to businesses that:
- Have annual gross revenues exceeding $25 million, OR
- Buy, sell, or share personal information of 100,000+ California residents, OR
- Derive 50%+ of annual revenue from selling/sharing personal information

### 4.2 CCPA Checklist

- [ ] **CC-01**: Implement "Do Not Sell or Share My Personal Information" link capability in storefront footer
- [ ] **CC-02**: Support opt-out of personal information "sale" (broadly defined to include sharing with ad networks)
- [ ] **CC-03**: Provide data access and deletion capabilities (overlaps with GDPR requirements GD-04 through GD-13)
- [ ] **CC-04**: Do not discriminate against customers who exercise privacy rights (e.g., no price differences)
- [ ] **CC-05**: Support "Limit the Use of My Sensitive Personal Information" requests
- [ ] **CC-06**: Include CCPA-specific disclosures in the Privacy Policy template (see PRIVACY_POLICY_TEMPLATE.md)
- [ ] **CC-07**: Implement data retention limits and disclose retention periods

### 4.3 Implementation Notes

Most CCPA requirements overlap with GDPR implementation. The key CCPA-specific items are:
- The "Do Not Sell" link (which the storefront theme must support)
- The broader definition of "sale" (includes sharing data for advertising purposes)
- The right to opt out applies to businesses, not just data processors

---

## 5. Cookie Consent

### 5.1 Requirements by Jurisdiction

| Jurisdiction | Requirement | Standard |
|-------------|-------------|----------|
| EU/EEA | Prior consent for non-essential cookies | ePrivacy Directive / GDPR |
| UK | Prior consent for non-essential cookies | UK GDPR / PECR |
| California | Disclosure of cookies; opt-out for tracking | CCPA/CPRA |
| Brazil | Consent for non-essential cookies | LGPD |
| Canada | Implied consent with disclosure | PIPEDA |

### 5.2 Cookie Consent Checklist

- [ ] **CK-01**: Identify all cookies set by RustCommerce (session, cart, preferences, analytics)
- [ ] **CK-02**: Classify cookies as strictly necessary, functional, analytics, or marketing
- [ ] **CK-03**: Provide a cookie consent banner/modal integration point for the storefront theme
- [ ] **CK-04**: Do NOT set non-essential cookies before consent is obtained
- [ ] **CK-05**: Store consent preferences and allow users to modify them
- [ ] **CK-06**: Provide a cookie policy page template listing all cookies, their purpose, and duration
- [ ] **CK-07**: Cart session cookies are "strictly necessary" and do not require consent
- [ ] **CK-08**: Analytics/tracking cookies MUST require consent before activation

### 5.3 RustCommerce Cookie Inventory

| Cookie | Purpose | Classification | Consent Required |
|--------|---------|---------------|-----------------|
| `rc_session` | User session identifier | Strictly Necessary | No |
| `rc_cart` | Cart contents (guest users) | Strictly Necessary | No |
| `rc_currency` | Selected currency preference | Functional | Yes (EU) |
| `rc_recently_viewed` | Recently viewed products | Functional | Yes (EU) |
| Any analytics cookies | Store analytics | Analytics | Yes |

---

## 6. Tax Compliance

### 6.1 Overview

RustCommerce must support tax calculation and collection. Tax compliance is the store operator's legal obligation, but the plugin must provide the tooling to make compliance possible.

### 6.2 EU VAT (Value Added Tax)

- [ ] **TX-01**: Support configurable VAT rates per EU member state
- [ ] **TX-02**: Support VAT rate categories (standard, reduced, zero-rated, exempt)
- [ ] **TX-03**: Support EU VAT MOSS (Mini One-Stop Shop) for digital services
- [ ] **TX-04**: Display prices inclusive or exclusive of VAT (configurable per store)
- [ ] **TX-05**: Generate VAT-compliant invoices with: seller VAT number, buyer details, VAT amount, VAT rate, net amount
- [ ] **TX-06**: Support reverse charge mechanism for B2B cross-border EU sales
- [ ] **TX-07**: Validate EU VAT numbers via VIES (VAT Information Exchange System) API
- [ ] **TX-08**: Support the EU's 2024+ ViDA (VAT in the Digital Age) requirements for digital reporting

### 6.3 US Sales Tax

- [ ] **TX-09**: Support sales tax calculation based on destination (most US states are destination-based)
- [ ] **TX-10**: Support tax-exempt products and product categories
- [ ] **TX-11**: Support tax nexus configuration (which states the store has nexus in)
- [ ] **TX-12**: Provide integration points for third-party tax calculation services (TaxJar, Avalara, etc.)
- [ ] **TX-13**: Support economic nexus thresholds (post-Wayfair, most states: $100K revenue or 200 transactions)
- [ ] **TX-14**: Generate sales tax reports for state filing

### 6.4 General Tax Requirements

- [ ] **TX-15**: Store tax amounts separately from product prices in order records
- [ ] **TX-16**: Support tax-inclusive and tax-exclusive pricing modes
- [ ] **TX-17**: Support tax zones/regions with configurable rates
- [ ] **TX-18**: Maintain audit trail of tax calculations for each order
- [ ] **TX-19**: Document that store operators are responsible for tax registration, filing, and remittance
- [ ] **TX-20**: Provide tax report exports (CSV/PDF) for accounting purposes

---

## 7. Accessibility Requirements

### 7.1 Legal Framework

| Law / Standard | Jurisdiction | Requirement |
|---------------|-------------|-------------|
| ADA (Americans with Disabilities Act) | USA | Public accommodations must be accessible; courts increasingly apply to websites |
| Section 508 | USA (federal) | Federal agencies must use accessible technology |
| EAA (European Accessibility Act) | EU | Digital services must be accessible by June 2025 |
| AODA | Ontario, Canada | Websites must meet WCAG 2.0 Level AA |
| WCAG 2.1/2.2 | International standard | De facto global accessibility benchmark |

### 7.2 Accessibility Checklist — Admin UI

- [ ] **AC-01**: All admin UI components must be keyboard navigable (tab, enter, escape, arrow keys)
- [ ] **AC-02**: All form inputs must have associated labels
- [ ] **AC-03**: Color contrast ratios must meet WCAG 2.1 AA (4.5:1 for normal text, 3:1 for large text)
- [ ] **AC-04**: All images/icons must have alt text or aria-label
- [ ] **AC-05**: Error messages must be programmatically associated with form fields
- [ ] **AC-06**: Admin dashboard data visualizations must have text alternatives
- [ ] **AC-07**: Follow existing RustPress admin UI accessibility patterns and components

### 7.3 Accessibility Checklist — Storefront API / Theme Support

- [ ] **AC-08**: API responses must include sufficient data for themes to render accessible HTML
- [ ] **AC-09**: Product data must support alt text for product images
- [ ] **AC-10**: Checkout flow API must support accessible error reporting (field-level errors with codes)
- [ ] **AC-11**: Cart operations must return data suitable for ARIA live region updates
- [ ] **AC-12**: Document accessibility best practices for theme developers building storefront UIs on the RustCommerce API

### 7.4 Accessibility Checklist — Storefront Templates/Examples

- [ ] **AC-13**: Any example storefront markup must use semantic HTML (nav, main, article, section, etc.)
- [ ] **AC-14**: Form elements in examples must include proper label associations and error states
- [ ] **AC-15**: Focus management must be handled in multi-step checkout flows
- [ ] **AC-16**: Price and quantity information must be readable by screen readers

---

## 8. Additional Regulatory Considerations

### 8.1 Consumer Protection

- [ ] **CP-01**: Support clear display of total price including all taxes, fees, and shipping before final purchase confirmation
- [ ] **CP-02**: Support order confirmation emails with complete order details
- [ ] **CP-03**: Support cancellation/return policy display (EU 14-day cooling-off period for distance selling)
- [ ] **CP-04**: Do not use dark patterns (pre-checked boxes, hidden fees, confusing opt-outs)
- [ ] **CP-05**: Display clear terms and conditions during checkout

### 8.2 Email Marketing (CAN-SPAM / GDPR)

- [ ] **EM-01**: Marketing emails must require explicit opt-in (GDPR) or provide opt-out (CAN-SPAM)
- [ ] **EM-02**: Every marketing email must include an unsubscribe mechanism
- [ ] **EM-03**: Store consent status for each customer (opted-in, opted-out, not asked)
- [ ] **EM-04**: Transactional emails (order confirmation, shipping updates) do not require opt-in but must not contain marketing

### 8.3 Age Restrictions

- [ ] **AR-01**: Provide configuration option for age-restricted products
- [ ] **AR-02**: Support age verification gate for stores selling age-restricted goods
- [ ] **AR-03**: Document that store operators are responsible for compliance with age restriction laws in their jurisdiction

### 8.4 Product Safety and Labeling

- [ ] **PS-01**: Provide fields for regulatory compliance information (e.g., CE marking, safety warnings)
- [ ] **PS-02**: Document that store operators are responsible for product safety compliance
- [ ] **PS-03**: Support product recall notification capability (via email system)

---

## 9. Compliance Responsibility Matrix

| Area | Plugin Developer (RustCommerce) | Store Operator |
|------|-------------------------------|----------------|
| PCI-DSS architecture (no card storage) | RESPONSIBLE | N/A |
| PCI-DSS SAQ-A completion | N/A | RESPONSIBLE |
| GDPR-enabling features (export, delete) | RESPONSIBLE | N/A |
| GDPR Data Controller obligations | N/A | RESPONSIBLE |
| Privacy Policy creation | TEMPLATE PROVIDED | RESPONSIBLE (customize) |
| Cookie consent mechanism | INTEGRATION POINT | RESPONSIBLE (configure) |
| Tax calculation tooling | RESPONSIBLE | N/A |
| Tax registration and filing | N/A | RESPONSIBLE |
| Accessibility of plugin UI | RESPONSIBLE | N/A |
| Accessibility of storefront | GUIDANCE PROVIDED | RESPONSIBLE (theme) |
| Terms of Service for store | TEMPLATE GUIDANCE | RESPONSIBLE |
| Consumer protection compliance | TOOLING PROVIDED | RESPONSIBLE |

---

## 10. Compliance Review Schedule

| Checkpoint | When | Reviewer |
|-----------|------|----------|
| PCI-DSS architecture review | Before Milestone 2 (Stripe integration) | Legal + Security |
| GDPR feature completeness | Before Milestone 3 (Customer management) | Legal |
| Accessibility audit | Before Milestone 4 (Storefront) | QA + Legal |
| Full compliance review | Before Milestone 5 (Release) | Legal |
| Ongoing compliance monitoring | Quarterly post-release | Legal |

---

*This checklist should be treated as a living document. Regulatory requirements evolve, and this document must be updated accordingly. Store operators should be advised to seek their own legal counsel for jurisdiction-specific compliance.*
