# Legal Risk Assessment — RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Prepared By**: Legal/Compliance Attorney
**Project**: RustCommerce (RCOM-001)

---

## 1. Executive Summary

This document assesses the legal risks associated with developing and distributing the RustCommerce e-commerce plugin for RustPress CMS. Risks are categorized by domain and rated by likelihood, impact, and overall risk level. Mitigation strategies are provided for each identified risk.

**Overall Risk Profile**: MODERATE. The delegation of payment processing to Stripe significantly reduces the highest-risk area (PCI liability). The primary residual risks relate to data privacy obligations, liability boundaries between plugin and store operator, and intellectual property considerations.

---

## 2. Risk Rating Methodology

| Rating | Likelihood | Impact |
|--------|-----------|--------|
| **LOW** | Unlikely to occur | Minor consequence, easily remediated |
| **MEDIUM** | Possible; has occurred in similar projects | Moderate consequence; requires effort to remediate |
| **HIGH** | Likely or has industry precedent | Severe consequence; significant financial/legal/reputational damage |
| **CRITICAL** | Near-certain if not mitigated | Catastrophic; project-threatening |

**Overall Risk** = Likelihood x Impact

---

## 3. Payment Processing Risks

### RISK-PAY-01: Inadvertent Credit Card Data Storage

| Attribute | Assessment |
|-----------|-----------|
| **Description** | A coding error, logging misconfiguration, or debug mode causes raw credit card numbers to be stored in the database, log files, or error reports |
| **Likelihood** | MEDIUM (common in e-commerce development without strict controls) |
| **Impact** | CRITICAL (PCI-DSS violation; fines up to $500K; mandatory breach notification; reputational destruction) |
| **Overall Risk** | HIGH |
| **Mitigation** | 1. Architectural enforcement: no payment form fields on RustCommerce-served pages; Stripe Elements only. 2. Code review checklist item for all payment-related code. 3. Automated grep/scan in CI for patterns resembling card numbers (regex for 13-19 digit sequences). 4. Structured logging that explicitly excludes request bodies on payment endpoints. 5. Security testing with test card numbers to verify no storage occurs. |

### RISK-PAY-02: Stripe Webhook Spoofing

| Attribute | Assessment |
|-----------|-----------|
| **Description** | An attacker sends forged webhook events to the RustCommerce webhook endpoint, causing fraudulent order fulfillment or status changes |
| **Likelihood** | MEDIUM (webhook endpoints are publicly discoverable) |
| **Impact** | HIGH (financial loss from fraudulent order fulfillment; inventory discrepancies) |
| **Overall Risk** | HIGH |
| **Mitigation** | 1. Mandatory Stripe webhook signature verification on all incoming webhook events. 2. Reject webhooks that fail signature verification with 401 status. 3. Implement idempotency checks to prevent replay attacks. 4. Rate limit the webhook endpoint. 5. Log all rejected webhook attempts for security monitoring. |

### RISK-PAY-03: Payment Disputes and Chargebacks

| Attribute | Assessment |
|-----------|-----------|
| **Description** | Store operators face chargebacks and need evidence to dispute them; plugin does not retain sufficient transaction records |
| **Likelihood** | HIGH (chargebacks are routine in e-commerce) |
| **Impact** | MEDIUM (financial loss for store operator; not directly RustCommerce's liability) |
| **Overall Risk** | MEDIUM |
| **Mitigation** | 1. Store comprehensive order/payment audit trail (timestamps, IP addresses, shipping tracking). 2. Retain Stripe event history and payment intent metadata. 3. Document best practices for chargeback evidence in store operator guide. 4. Store shipping confirmation and tracking data linked to orders. |

### RISK-PAY-04: Payment Gateway Downtime

| Attribute | Assessment |
|-----------|-----------|
| **Description** | Stripe experiences an outage, preventing customers from completing purchases |
| **Likelihood** | LOW (Stripe has 99.99%+ uptime, but outages do occur) |
| **Impact** | MEDIUM (lost sales during outage) |
| **Overall Risk** | LOW |
| **Mitigation** | 1. Implement graceful error handling with clear customer-facing messages. 2. Design extensible payment gateway interface to allow future addition of backup gateways. 3. Consider order queuing mechanism for retry after gateway recovery. |

---

## 4. Data Privacy and Breach Risks

### RISK-DATA-01: Data Breach Exposing Customer PII

| Attribute | Assessment |
|-----------|-----------|
| **Description** | Unauthorized access to the database exposes customer names, emails, addresses, phone numbers, and order history |
| **Likelihood** | MEDIUM (data breaches are common; risk depends on store operator's security posture) |
| **Impact** | CRITICAL (GDPR fines up to 4% of global revenue or 20M EUR; CCPA fines; mandatory notifications; class action lawsuits; reputational damage) |
| **Overall Risk** | HIGH |
| **Mitigation** | 1. Encrypt PII at rest in the database. 2. Implement role-based access control with principle of least privilege. 3. Comprehensive audit logging of data access. 4. Provide breach detection guidance and incident response template for store operators. 5. Data minimization: collect only what is necessary. 6. See SECURITY_REQUIREMENTS.md for full security architecture. |

### RISK-DATA-02: Failure to Honor Data Deletion Requests

| Attribute | Assessment |
|-----------|-----------|
| **Description** | A customer exercises their GDPR right to erasure, but the plugin fails to fully delete/anonymize their data due to foreign key constraints, backup retention, or incomplete implementation |
| **Likelihood** | MEDIUM (complex data relationships make complete deletion difficult) |
| **Impact** | HIGH (GDPR non-compliance; regulatory investigation; fines) |
| **Overall Risk** | HIGH |
| **Mitigation** | 1. Design the data model with deletion in mind from the start (soft-delete with anonymization, not hard-delete). 2. Map all personal data fields across all tables. 3. Implement a comprehensive anonymization function that handles all tables. 4. Test deletion thoroughly with integration tests. 5. Document what is retained (anonymized order records for tax compliance) and the legal basis for retention. |

### RISK-DATA-03: Inadequate Consent Management

| Attribute | Assessment |
|-----------|-----------|
| **Description** | The plugin collects or processes data without proper consent, or fails to provide adequate consent management tools |
| **Likelihood** | MEDIUM |
| **Impact** | MEDIUM (regulatory complaints; fines under GDPR; loss of customer trust) |
| **Overall Risk** | MEDIUM |
| **Mitigation** | 1. Implement granular consent management (marketing, analytics, cookies). 2. Store consent records with timestamp and scope. 3. Default to opt-out for marketing; require explicit opt-in in EU. 4. Provide consent withdrawal mechanism. 5. Separate transactional from marketing communications. |

### RISK-DATA-04: Cross-Border Data Transfer Violations

| Attribute | Assessment |
|-----------|-----------|
| **Description** | Customer data from EU residents is transferred to or processed in countries without adequate data protection (e.g., US servers without SCCs) |
| **Likelihood** | MEDIUM (common with cloud hosting) |
| **Impact** | HIGH (GDPR violation; data transfer suspension orders) |
| **Overall Risk** | MEDIUM |
| **Mitigation** | 1. Document that store operators are responsible for ensuring lawful data transfers. 2. Provide guidance on Standard Contractual Clauses (SCCs) and other transfer mechanisms. 3. Privacy Policy template includes data transfer disclosure section. 4. Recommend EU-hosted infrastructure for stores primarily serving EU customers. |

---

## 5. Intellectual Property Risks

### RISK-IP-01: Open-Source License Contamination

| Attribute | Assessment |
|-----------|-----------|
| **Description** | A dependency (direct or transitive) with a copyleft license (GPL, AGPL) contaminates the MIT-licensed project, requiring RustCommerce to be relicensed |
| **Likelihood** | LOW (Rust ecosystem is predominantly MIT/Apache-2.0) |
| **Impact** | HIGH (may require relicensing or removing the dependency; delays release) |
| **Overall Risk** | MEDIUM |
| **Mitigation** | 1. Implement cargo-deny with license allow/deny lists (see LICENSE_REVIEW.md). 2. Run license checks in CI on every PR. 3. Review all new dependencies before adding to Cargo.toml. 4. Audit transitive dependencies quarterly. |

### RISK-IP-02: Patent Claims from Payment/E-Commerce Methods

| Attribute | Assessment |
|-----------|-----------|
| **Description** | A patent holder claims that specific e-commerce features (one-click checkout, specific cart algorithms, recommendation engines) infringe on their patents |
| **Likelihood** | LOW (open-source projects are rarely targeted; most e-commerce patents are broadly licensed or expired) |
| **Impact** | HIGH (injunction; licensing fees; feature removal) |
| **Overall Risk** | LOW |
| **Mitigation** | 1. Use standard, well-established e-commerce patterns rather than novel approaches. 2. Apache-2.0 licensed dependencies include patent grants that provide additional protection. 3. Document that RustCommerce implements standard e-commerce functionality, not novel patented methods. 4. Consider contributing to the Open Invention Network (OIN) for patent non-aggression. |

### RISK-IP-03: Trademark Infringement

| Attribute | Assessment |
|-----------|-----------|
| **Description** | The name "RustCommerce" or product imagery infringes on existing trademarks |
| **Likelihood** | LOW (preliminary assessment; formal search recommended) |
| **Impact** | MEDIUM (rebranding costs; legal fees) |
| **Overall Risk** | LOW |
| **Mitigation** | 1. Conduct a trademark search for "RustCommerce" in relevant classes (Class 9: software, Class 35: retail services, Class 42: SaaS). 2. Note: "Rust" as used in Rust programming language context has specific trademark guidelines from the Rust Foundation. 3. Ensure compliance with Rust trademark policy for use of "Rust" in the project name. 4. File trademark application if desired. |

### RISK-IP-04: Third-Party Code Incorporation Without Attribution

| Attribute | Assessment |
|-----------|-----------|
| **Description** | Developers copy code from Stack Overflow, blog posts, or other projects without proper attribution or license compliance |
| **Likelihood** | MEDIUM (common in software development) |
| **Impact** | MEDIUM (copyright infringement claims; license violations) |
| **Overall Risk** | MEDIUM |
| **Mitigation** | 1. Establish contribution guidelines requiring original code or properly attributed/licensed code. 2. Code review must verify no substantial copied code without attribution. 3. Maintain a NOTICES file for any incorporated third-party code. 4. AI-generated code: document that AI tools were used and review for potential IP issues. |

---

## 6. Liability and Terms of Service Risks

### RISK-LIA-01: Store Operator Holds RustCommerce Liable for Business Losses

| Attribute | Assessment |
|-----------|-----------|
| **Description** | A store operator suffers financial loss (e.g., from a bug that miscalculates prices, fails to process payments, or causes a security breach) and seeks damages from the RustCommerce project |
| **Likelihood** | MEDIUM (bugs happen; litigious operators exist) |
| **Impact** | HIGH (lawsuits; financial damages; project viability) |
| **Overall Risk** | HIGH |
| **Mitigation** | 1. MIT license includes "AS IS" disclaimer and limitation of liability. 2. Create a clear DISCLAIMER.md or include in README that RustCommerce is provided without warranty. 3. Recommend store operators use the plugin with appropriate business insurance. 4. Document that store operators are responsible for testing in their environment before production use. 5. Consider additional terms of use beyond the MIT license. |

### RISK-LIA-02: Tax Calculation Errors

| Attribute | Assessment |
|-----------|-----------|
| **Description** | The tax calculation engine produces incorrect tax amounts, resulting in store operators under-collecting or over-collecting taxes |
| **Likelihood** | HIGH (tax rules are complex and change frequently; especially US nexus, EU VAT rates) |
| **Impact** | MEDIUM (store operator tax liability; customer complaints; regulatory penalties fall on store operator, not plugin) |
| **Overall Risk** | MEDIUM |
| **Mitigation** | 1. Clearly document that store operators are responsible for tax compliance and rate configuration. 2. Provide integration points for professional tax calculation services (TaxJar, Avalara). 3. Do not represent the built-in tax calculation as legally compliant; label it as "basic" and recommend third-party services for complex scenarios. 4. Include prominent disclaimer in tax settings: "Consult a tax professional for your specific obligations." |

### RISK-LIA-03: Inventory/Pricing Errors Leading to Obligation to Fulfill

| Attribute | Assessment |
|-----------|-----------|
| **Description** | A bug causes incorrect pricing display (e.g., $0 price, missing decimal) and customers place orders at the wrong price; consumer protection laws may require fulfillment |
| **Likelihood** | LOW (with proper QA) |
| **Impact** | MEDIUM (financial loss for store operator) |
| **Overall Risk** | LOW |
| **Mitigation** | 1. Implement input validation and sanity checks on prices (minimum price, maximum discount percentage). 2. Provide order review step before payment processing. 3. Document that store operators should configure order review workflows for unusual orders. 4. Include terms of service template that reserves the right to cancel orders with obvious pricing errors. |

---

## 7. Regulatory and Compliance Risks

### RISK-REG-01: Non-Compliance with Evolving Privacy Regulations

| Attribute | Assessment |
|-----------|-----------|
| **Description** | New privacy regulations (or amendments to existing ones) impose requirements the plugin cannot meet without significant changes |
| **Likelihood** | HIGH (privacy regulation is rapidly evolving globally; new US state laws, EU Digital Services Act, AI Act implications) |
| **Impact** | MEDIUM (feature development to achieve compliance; store operators unable to use plugin in certain jurisdictions) |
| **Overall Risk** | MEDIUM |
| **Mitigation** | 1. Design data handling with privacy-by-design principles, making it adaptable. 2. Abstract consent and data management into configurable modules. 3. Monitor regulatory developments quarterly. 4. Maintain a modular architecture that allows privacy features to be updated independently. |

### RISK-REG-02: Accessibility Lawsuit

| Attribute | Assessment |
|-----------|-----------|
| **Description** | A store using RustCommerce is sued for ADA non-compliance due to inaccessible checkout or product pages |
| **Likelihood** | MEDIUM (ADA website lawsuits have increased significantly; over 4,000 filed annually in US) |
| **Impact** | MEDIUM (primarily store operator's liability, but reputational impact on RustCommerce; feature development cost) |
| **Overall Risk** | MEDIUM |
| **Mitigation** | 1. Build admin UI to WCAG 2.1 AA standards. 2. Provide accessible API responses that enable theme developers to build accessible storefronts. 3. Document accessibility best practices for theme developers. 4. Include accessibility testing in QA process. 5. Note that storefront accessibility is the theme developer's and store operator's responsibility, but provide enabling tools. |

### RISK-REG-03: PSD2/SCA Compliance for EU Payments

| Attribute | Assessment |
|-----------|-----------|
| **Description** | EU's Payment Services Directive 2 (PSD2) requires Strong Customer Authentication (SCA) for online payments; non-compliance results in declined transactions |
| **Likelihood** | HIGH (SCA is mandatory in EU/EEA since 2021) |
| **Impact** | MEDIUM (declined transactions; lost sales for EU stores) |
| **Overall Risk** | MEDIUM |
| **Mitigation** | 1. Stripe handles SCA/3D Secure automatically when using Payment Intents API. 2. Ensure RustCommerce uses Stripe Payment Intents (not the legacy Charges API). 3. Support the client-side flow for 3D Secure authentication redirects. 4. Test with Stripe's SCA test cards. |

---

## 8. Operational Risks with Legal Implications

### RISK-OPS-01: Supply Chain Attack via Compromised Dependency

| Attribute | Assessment |
|-----------|-----------|
| **Description** | A dependency (Rust crate or npm package) is compromised with malicious code that exfiltrates customer data or payment tokens |
| **Likelihood** | LOW-MEDIUM (supply chain attacks are increasing; see event-stream, ua-parser-js incidents) |
| **Impact** | CRITICAL (data breach; PCI violation; loss of trust) |
| **Overall Risk** | HIGH |
| **Mitigation** | 1. Pin dependency versions and review changelogs before upgrading. 2. Use cargo-deny to restrict dependency sources. 3. Run cargo-audit regularly for known vulnerabilities. 4. For npm: use npm audit, lockfile-lint, and consider Socket.dev. 5. Minimize dependency count where possible. 6. Review new transitive dependencies introduced by upgrades. |

### RISK-OPS-02: Open-Source Contributor Legal Issues

| Attribute | Assessment |
|-----------|-----------|
| **Description** | An external contributor submits code they do not have the right to contribute (copied from a proprietary codebase or incompatibly licensed project) |
| **Likelihood** | LOW (small contributor base initially) |
| **Impact** | HIGH (IP infringement claims; forced code removal) |
| **Overall Risk** | LOW-MEDIUM |
| **Mitigation** | 1. Require a Developer Certificate of Origin (DCO) sign-off on all contributions. 2. Consider a Contributor License Agreement (CLA) for significant contributions. 3. Code review must include basic IP review (no suspicious large code blocks, no proprietary headers). 4. CONTRIBUTING.md should clearly state the license terms contributions will be under. |

---

## 9. Terms of Service Recommendations for Store Operators

RustCommerce should provide a Terms of Service template or guidance document that store operators can adapt. Key clauses to recommend:

### 9.1 Mandatory Clauses

1. **Limitation of Liability**: Cap liability to the purchase price of the order in question
2. **Pricing Errors**: Reserve the right to cancel orders with obvious pricing errors
3. **Order Acceptance**: Clarify that order confirmation is not acceptance; acceptance occurs upon shipment
4. **Payment Terms**: Specify that payment is due at checkout and is processed by a third-party payment processor
5. **Refund/Return Policy**: Define return window, conditions, and refund method
6. **Intellectual Property**: Assert ownership of store content; restrict unauthorized use
7. **User Accounts**: Terms for account creation, responsibilities, termination

### 9.2 Recommended Clauses

8. **Governing Law and Jurisdiction**: Specify applicable law and courts
9. **Dispute Resolution**: Consider arbitration clause or mediation-first requirement
10. **Force Majeure**: Exclude liability for events beyond reasonable control
11. **Modification of Terms**: Right to update terms with notice to customers
12. **Age Restrictions**: Minimum age for account creation and purchases
13. **Prohibited Uses**: Restrict fraudulent orders, automated scraping, etc.
14. **Third-Party Links**: Disclaimer for external links and services

### 9.3 Jurisdiction-Specific Clauses

15. **EU/EEA**: 14-day cooling-off period for distance selling (Directive 2011/83/EU)
16. **UK**: Consumer Rights Act 2015 compliance; 14-day cancellation right
17. **Australia**: Australian Consumer Law guarantees cannot be excluded
18. **California**: Automatic renewal/subscription disclosure requirements (if subscriptions are offered)

---

## 10. Risk Summary Matrix

| Risk ID | Risk | Likelihood | Impact | Overall | Priority |
|---------|------|-----------|--------|---------|----------|
| RISK-PAY-01 | Credit card data storage | MEDIUM | CRITICAL | HIGH | P0 |
| RISK-PAY-02 | Webhook spoofing | MEDIUM | HIGH | HIGH | P0 |
| RISK-PAY-03 | Chargeback evidence | HIGH | MEDIUM | MEDIUM | P1 |
| RISK-PAY-04 | Gateway downtime | LOW | MEDIUM | LOW | P2 |
| RISK-DATA-01 | Customer data breach | MEDIUM | CRITICAL | HIGH | P0 |
| RISK-DATA-02 | Failed data deletion | MEDIUM | HIGH | HIGH | P0 |
| RISK-DATA-03 | Consent management gaps | MEDIUM | MEDIUM | MEDIUM | P1 |
| RISK-DATA-04 | Cross-border transfers | MEDIUM | HIGH | MEDIUM | P1 |
| RISK-IP-01 | License contamination | LOW | HIGH | MEDIUM | P1 |
| RISK-IP-02 | Patent claims | LOW | HIGH | LOW | P2 |
| RISK-IP-03 | Trademark infringement | LOW | MEDIUM | LOW | P2 |
| RISK-IP-04 | Code without attribution | MEDIUM | MEDIUM | MEDIUM | P1 |
| RISK-LIA-01 | Operator liability claims | MEDIUM | HIGH | HIGH | P0 |
| RISK-LIA-02 | Tax calculation errors | HIGH | MEDIUM | MEDIUM | P1 |
| RISK-LIA-03 | Pricing errors | LOW | MEDIUM | LOW | P2 |
| RISK-REG-01 | Evolving regulations | HIGH | MEDIUM | MEDIUM | P1 |
| RISK-REG-02 | Accessibility lawsuit | MEDIUM | MEDIUM | MEDIUM | P1 |
| RISK-REG-03 | PSD2/SCA compliance | HIGH | MEDIUM | MEDIUM | P1 |
| RISK-OPS-01 | Supply chain attack | LOW-MEDIUM | CRITICAL | HIGH | P0 |
| RISK-OPS-02 | Contributor IP issues | LOW | HIGH | LOW-MEDIUM | P2 |

### P0 Risks (Must Mitigate Before MVP Release)

1. **RISK-PAY-01**: Architectural enforcement of no card data storage
2. **RISK-PAY-02**: Mandatory webhook signature verification
3. **RISK-DATA-01**: Encryption, access controls, audit logging
4. **RISK-DATA-02**: Comprehensive data deletion/anonymization
5. **RISK-LIA-01**: Clear disclaimers, MIT license, documentation
6. **RISK-OPS-01**: Dependency security scanning in CI

---

## 11. Recommendations

### Immediate Actions

1. **Engage trademark counsel** to search "RustCommerce" and assess Rust Foundation trademark policy compliance
2. **Draft DISCLAIMER section** for README.md covering warranty exclusion, use-at-own-risk, and store operator responsibilities
3. **Implement DCO** requirement for external contributions (via CONTRIBUTING.md and CI check)
4. **Configure cargo-deny and npm audit** in CI pipeline immediately

### Pre-Release Actions

5. **Conduct penetration test** on payment flow before public release
6. **Complete GDPR Data Protection Impact Assessment (DPIA)** for the plugin's data processing
7. **Prepare incident response plan** template for store operators
8. **Review Terms of Service template** with qualified e-commerce attorney
9. **Verify SCA/3D Secure flow** with Stripe test environment

### Post-Release Actions

10. **Establish vulnerability disclosure program** (security@rustpress.net or similar)
11. **Monitor legal developments** in e-commerce regulation quarterly
12. **Maintain compliance documentation** as features evolve
13. **Consider cyber liability insurance** for the project/organization

---

*This risk assessment should be reviewed quarterly and updated when new features are added, new jurisdictions are targeted, or regulatory changes occur. Next scheduled review: before Milestone 3 completion.*
