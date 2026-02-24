# Key Messaging Framework — RustCommerce

**Document Version**: 1.0
**Date**: 2026-02-24
**Author**: Marketing Strategist

---

## 1. Tagline

**Primary Tagline**: "Blazing-fast e-commerce, forged in Rust."

**Alternative Taglines**:
- "E-commerce at the speed of Rust."
- "Your store. Your data. Your performance. Zero compromises."
- "The e-commerce plugin RustPress deserves."

---

## 2. Elevator Pitch

**30-Second Pitch**:

> RustCommerce is the first production-grade e-commerce plugin built entirely in Rust for the RustPress CMS. It gives any RustPress site a complete online store — products, cart, checkout, Stripe payments, order management, and a full admin dashboard — all running natively within the RustPress ecosystem. No separate deployment. No runtime interpreter. No garbage collector. Just compiled Rust serving sub-100ms responses while handling hundreds of concurrent shoppers. It is open source, MIT-licensed, and charges zero platform fees.

**One-Liner** (for social media, bios, and quick introductions):

> A full-featured, Rust-native e-commerce plugin for RustPress CMS — fast, secure, and open source.

---

## 3. Key Messages by Audience

### 3.1 Store Owners and Merchants

**Primary Message**: "Add a complete online store to your RustPress site in minutes — no separate platform, no monthly fees, no compromises on speed."

**Supporting Messages**:
- **Easy Setup**: Install the plugin, configure your store settings, add products, and start selling. The admin dashboard is intuitive and built right into RustPress.
- **Performance You Can Feel**: Product pages load instantly. Checkout completes in under 3 seconds. Your customers never wait.
- **No Platform Fees**: RustCommerce is free and open source. You pay only for hosting and Stripe payment processing. No $39/month subscription. No 2% transaction tax.
- **Total Data Ownership**: Your products, customers, and orders live in your PostgreSQL database. You own everything. Export anytime. No vendor lock-in.
- **Secure by Design**: Built in Rust, a language engineered for safety. Your store is protected by compile-time memory safety guarantees and PCI-DSS-aware architecture that never stores credit card data locally.

**Call to Action**: "Install RustCommerce and launch your store today."

### 3.2 Developers

**Primary Message**: "Finally, e-commerce in Rust. Clean architecture, type-safe APIs, and a codebase you will enjoy reading."

**Supporting Messages**:
- **Rust All the Way Down**: Models, repositories, services, and handlers — all in idiomatic Rust with async/await, proper error handling, and zero unsafe code in business logic.
- **Clean Layered Architecture**: `models/ -> repositories/ -> services/ -> handlers/` with clear separation of concerns. Every function returns `Result<T, Error>`. Every database query is compile-time checked with sqlx.
- **Full REST API**: Complete CRUD under `/api/v1/rustcommerce/` following RustPress conventions. Build headless storefronts, mobile apps, or integrate with any frontend.
- **Extensible by Design**: RustPress hooks fire on every key commerce event — `order_created`, `payment_completed`, `product_updated`, `cart_updated`, and more. Write plugins that react to commerce events.
- **Modern Frontend Stack**: React 18 + TypeScript + Tailwind CSS + Zustand for the admin UI. Lazy-loaded routes, design system components, and a typed API client.
- **Stripe Integration Done Right**: Payment Intent API with webhook verification. Extensible gateway interface for adding PayPal, Square, or custom processors.

**Call to Action**: "Clone the repo, read the docs, and start building."

### 3.3 Agencies

**Primary Message**: "One platform for content and commerce. Faster delivery, simpler maintenance, happier clients."

**Supporting Messages**:
- **Single Deployment**: No separate e-commerce platform to manage. RustCommerce runs inside RustPress. One server, one database, one admin panel.
- **Faster Project Delivery**: Install the plugin, configure settings, and hand off to the client. The admin dashboard is ready out of the box.
- **Lower Maintenance Burden**: One platform to update, one codebase to monitor, one security surface to protect. Rust's compile-time guarantees mean fewer production incidents.
- **Client Confidence**: Demonstrate sub-100ms API responses and handle 100+ concurrent users per instance. Clients see the performance difference.
- **Custom Builds Made Easy**: Extend with custom payment gateways, shipping providers, or storefront themes using clean Rust APIs and RustPress hooks.
- **Open Source Economics**: No per-client licensing fees. Deploy for as many clients as you want under the MIT license.

**Call to Action**: "Build your next client's store on RustCommerce."

---

## 4. Feature Highlights and Benefits

### 4.1 Product Management
- **Feature**: Full CRUD for products with variants, categories, tags, images, SKU tracking, and product status management.
- **Benefit**: Manage your entire catalog from one intuitive admin interface. Support for product variants (size, color) means fewer product entries and cleaner organization.

### 4.2 Shopping Cart
- **Feature**: Server-side persistent cart for logged-in users, client-side cart for guests. Real-time tax and shipping preview.
- **Benefit**: Customers never lose their cart. Session portability means they can start on mobile and finish on desktop. Transparent pricing at every step reduces cart abandonment.

### 4.3 Checkout Flow
- **Feature**: Multi-step checkout — shipping address, shipping method, payment, order confirmation. Guest checkout supported.
- **Benefit**: A streamlined, familiar checkout experience that does not require account creation. Fewer steps, fewer dropoffs, higher conversion.

### 4.4 Stripe Payment Integration
- **Feature**: Stripe Payment Intent API with secure webhook verification. No credit card data stored locally.
- **Benefit**: Accept credit cards, debit cards, and Stripe-supported payment methods worldwide. PCI-compliant by architecture.

### 4.5 Order Management
- **Feature**: Full order lifecycle — Pending, Processing, Shipped, Delivered, Cancelled, Refunded. Admin order management with filtering and detail views.
- **Benefit**: Track every order from placement to delivery. Process refunds, update statuses, and view complete order history from the admin dashboard.

### 4.6 Inventory Management
- **Feature**: Stock tracking per product and variant, low-stock alerts, backorder configuration, and stock reservation during checkout.
- **Benefit**: Never oversell. Automatic stock updates on order completion. Get alerted before items run out.

### 4.7 Store Settings
- **Feature**: Configurable currency, tax rules (flat rate and zone-based), shipping methods (flat rate, free over threshold, weight-based), and store policies.
- **Benefit**: Configure once, apply everywhere. Tax and shipping calculated automatically at checkout.

### 4.8 Admin Dashboard
- **Feature**: Store metrics dashboard with revenue, order count, average order value, best sellers, and recent orders. Full management UI for products, orders, customers, and settings.
- **Benefit**: See how your store is performing at a glance. Manage everything from a single, integrated admin panel.

### 4.9 REST API
- **Feature**: Complete CRUD API for all entities under `/api/v1/rustcommerce/` with proper authentication, pagination, and filtering.
- **Benefit**: Build custom storefronts, mobile apps, or third-party integrations. API-first architecture means your data is accessible from anywhere.

### 4.10 Hook System Integration
- **Feature**: RustPress hooks fire on key commerce events — order creation, payment completion, product updates, cart changes.
- **Benefit**: Other plugins can react to commerce events. Build custom notifications, analytics integrations, or workflow automations.

---

## 5. Messaging Tone and Voice

### Tone
- **Confident but not arrogant**: We know Rust is fast and safe. We state facts, not hype.
- **Technical but accessible**: Developers should feel at home. Store owners should not feel alienated.
- **Direct**: Short sentences. Clear claims. Specific numbers (< 100ms, 100+ concurrent users).
- **Community-oriented**: Open source. MIT license. Contributions welcome.

### Voice Guidelines
- Use "RustCommerce" (one word, camelCase) consistently. Never "Rust Commerce" or "rust-commerce" in marketing copy.
- Lead with performance and safety when addressing developers. Lead with simplicity and cost when addressing store owners.
- Avoid jargon when speaking to non-technical audiences. "Fast" is better than "zero-cost abstractions" for store owners.
- Use concrete numbers: "< 100ms API responses", "100+ concurrent shoppers", "$0 platform fees" — not vague claims like "really fast" or "super affordable."

### Words to Use
- Performance, speed, fast, instant, responsive
- Safe, secure, reliable, stable
- Native, integrated, unified, seamless
- Open source, free, community, MIT license
- Modern, production-grade, full-featured
- Compile-time, type-safe, memory-safe

### Words to Avoid
- "Blazing fast" (overused in Rust marketing — use it only in the tagline)
- "Revolutionary" or "disruptive" (overpromise)
- "Simple" when describing something complex (be honest)
- "Enterprise-grade" without qualification (too vague)
- "Lightweight" (we are full-featured; lightweight implies missing features)

---

## 6. Objection Handling

### "RustPress ecosystem is too small / immature."
**Response**: Every ecosystem starts somewhere. WordPress was once a simple blog tool. RustPress is production-ready, and RustCommerce is the kind of plugin that attracts both users and contributors. Being early means being first — and that is a competitive advantage.

### "Rust is too hard to learn. I cannot customize this."
**Response**: The admin UI is React + TypeScript — the same stack millions of developers know. The REST API works with any frontend. You only need Rust knowledge if you are extending the backend plugin itself. And even then, the clean layered architecture and comprehensive documentation make it approachable.

### "WooCommerce has thousands of plugins. Why would I switch?"
**Response**: WooCommerce's plugin ecosystem is also its biggest liability — plugin conflicts, security vulnerabilities, and performance degradation from stacking PHP plugins. RustCommerce provides the core features most stores need out of the box, and the hook system enables clean extensions without the fragility.

### "Shopify just works. Why would I self-host?"
**Response**: Shopify works until you need something it does not offer, or until the fees scale with your revenue. RustCommerce gives you complete control — your data, your customization, your server, your economics. Zero platform fees mean your margins improve as you grow.

### "This has no track record in production."
**Response**: Fair point. That is why we are running a beta program with real stores and publishing performance benchmarks openly. The technology (Rust, PostgreSQL, Stripe, React) is all battle-tested. What is new is the combination — and that combination is compelling enough to try.

---

*This messaging framework should be referenced when creating any marketing content, documentation, or public communications about RustCommerce.*
