# Infrastructure Cost Estimate — RustCommerce Plugin

**Document Version**: 1.0
**Date**: 2026-02-24
**Author**: Infrastructure Agent
**Status**: Draft

---

## 1. Development Environment

RustCommerce development uses the existing RustPress Docker Compose setup. Costs are effectively zero beyond developer workstations.

| Component | Development Setup | Monthly Cost |
|-----------|------------------|-------------|
| PostgreSQL 16 | Docker container (docker-compose.yml) | $0 |
| Redis 7 | Docker container (docker-compose.yml) | $0 |
| File storage | Local filesystem | $0 |
| Stripe | Test/sandbox mode (no real charges) | $0 |
| CDN | Not used in development | $0 |
| RustPress server | `cargo run` locally | $0 |
| **Total** | | **$0/month** |

Hardware requirements for development:
- 4+ GB RAM (PostgreSQL + Redis + Rust compilation)
- 10+ GB disk (Rust toolchain + dependencies + Docker images)
- Any modern CPU (Rust compilation benefits from more cores)

---

## 2. Production: Small Store

**Profile**: < 100 products, < 50 orders/day, < 500 daily visitors

### 2.1 Self-Hosted (VPS)

| Component | Specification | Provider Example | Monthly Cost |
|-----------|--------------|-----------------|-------------|
| VPS Server | 2 vCPU, 4 GB RAM, 80 GB SSD | Hetzner CX31 / DigitalOcean Droplet | $15-20 |
| PostgreSQL | Runs on same VPS (Docker) | Included | $0 |
| Redis | Runs on same VPS (Docker) | Included | $0 |
| File storage | Local disk on VPS | Included | $0 |
| Domain + TLS | Let's Encrypt (free) | Cloudflare Free | $0 |
| CDN | Not needed at this scale | Cloudflare Free (optional) | $0 |
| Backups | Daily snapshots | VPS provider snapshot | $2-4 |
| **Subtotal** | | | **$17-24/month** |

### 2.2 Cloud-Hosted (AWS/GCP Minimum)

| Component | Specification | Provider Example | Monthly Cost |
|-----------|--------------|-----------------|-------------|
| Compute | t3.small (2 vCPU, 2 GB) | AWS EC2 | $15 |
| Database | db.t3.micro (1 vCPU, 1 GB) | AWS RDS PostgreSQL | $15 |
| Cache | Not needed (moka in-memory) | — | $0 |
| Storage | 20 GB S3 | AWS S3 | $0.50 |
| CDN | Cloudflare Free tier | Cloudflare | $0 |
| Bandwidth | ~50 GB/month | AWS (first 100 GB free) | $0 |
| **Subtotal** | | | **~$31/month** |

### 2.3 Stripe Fees (Small Store)

| Metric | Value |
|--------|-------|
| Average order value | $50 |
| Orders per month | ~1,500 (50/day) |
| Monthly revenue | $75,000 |
| Stripe fee (2.9% + $0.30/txn) | $2,175 + $450 = **$2,625/month** |

### 2.4 Small Store Total

| Category | Self-Hosted | Cloud-Hosted |
|----------|------------|-------------|
| Infrastructure | $17-24/month | ~$31/month |
| Stripe fees | ~$2,625/month | ~$2,625/month |
| **Total** | **~$2,645/month** | **~$2,656/month** |

> Stripe fees dominate at every scale. Infrastructure costs are negligible relative to payment processing fees.

---

## 3. Production: Medium Store

**Profile**: < 10,000 products, < 500 orders/day, < 10,000 daily visitors

### 3.1 Self-Hosted (Dedicated Server)

| Component | Specification | Provider Example | Monthly Cost |
|-----------|--------------|-----------------|-------------|
| Server | 4 vCPU, 16 GB RAM, 240 GB NVMe | Hetzner AX41 / OVH | $40-60 |
| PostgreSQL | Runs on server (Docker) | Included | $0 |
| Redis | Runs on server (Docker) | Included | $0 |
| File storage | S3-compatible (Backblaze B2) | ~100 GB images | $0.50 |
| CDN | Cloudflare Pro | Product image delivery | $20 |
| Backups | Daily to S3 | Backblaze B2 | $1 |
| Monitoring | Uptime + alerting | UptimeRobot Pro / Grafana Cloud Free | $0-7 |
| **Subtotal** | | | **$62-89/month** |

### 3.2 Cloud-Hosted (AWS/GCP)

| Component | Specification | Provider Example | Monthly Cost |
|-----------|--------------|-----------------|-------------|
| Compute | t3.medium (2 vCPU, 4 GB) | AWS EC2 | $30 |
| Database | db.t3.small (2 vCPU, 2 GB) with read replica | AWS RDS PostgreSQL | $30 + $30 |
| Cache | cache.t3.micro (0.5 GB) | AWS ElastiCache Redis | $13 |
| Storage | 200 GB S3 | AWS S3 | $5 |
| CDN | CloudFront (500 GB/month) | AWS CloudFront | $43 |
| Bandwidth | ~200 GB/month | AWS | $9 |
| Monitoring | Basic CloudWatch | AWS | $0 |
| **Subtotal** | | | **~$160/month** |

### 3.3 Stripe Fees (Medium Store)

| Metric | Value |
|--------|-------|
| Average order value | $60 |
| Orders per month | ~15,000 (500/day) |
| Monthly revenue | $900,000 |
| Stripe fee (2.9% + $0.30/txn) | $26,100 + $4,500 = **$30,600/month** |

> At this volume, Stripe offers negotiated rates (typically 2.5% + $0.25). With negotiated rates: ~$22,500 + $3,750 = **~$26,250/month**.

### 3.4 Medium Store Total

| Category | Self-Hosted | Cloud-Hosted |
|----------|------------|-------------|
| Infrastructure | $62-89/month | ~$160/month |
| Stripe fees (negotiated) | ~$26,250/month | ~$26,250/month |
| **Total** | **~$26,320/month** | **~$26,410/month** |

---

## 4. Production: Large Store

**Profile**: < 100,000 products, < 5,000 orders/day, < 100,000 daily visitors

### 4.1 Cloud-Hosted (AWS/GCP — Recommended)

| Component | Specification | Provider Example | Monthly Cost |
|-----------|--------------|-----------------|-------------|
| Compute | 2x c6g.large (2 vCPU, 4 GB) behind ALB | AWS EC2 + ALB | $124 + $20 |
| Database | db.r6g.large (2 vCPU, 16 GB) Multi-AZ | AWS RDS PostgreSQL | $410 |
| Read replica | db.r6g.medium (1 vCPU, 8 GB) | AWS RDS | $148 |
| Cache | cache.r6g.large (13 GB) | AWS ElastiCache Redis | $199 |
| Storage | 1 TB S3 + 500K images | AWS S3 | $23 |
| CDN | CloudFront (2 TB/month) | AWS CloudFront | $170 |
| Bandwidth | ~500 GB/month (non-CDN) | AWS | $45 |
| Monitoring | CloudWatch + alarms | AWS | $20 |
| WAF | AWS WAF (basic rules) | AWS WAF | $10 |
| Backups | Automated RDS snapshots + S3 cross-region | AWS | $30 |
| **Subtotal** | | | **~$1,199/month** |

### 4.2 Stripe Fees (Large Store)

| Metric | Value |
|--------|-------|
| Average order value | $75 |
| Orders per month | ~150,000 (5,000/day) |
| Monthly revenue | $11,250,000 |
| Stripe fee (negotiated ~2.2% + $0.20) | $247,500 + $30,000 = **~$277,500/month** |

> At this volume, custom Stripe enterprise pricing may yield further reductions.

### 4.3 Large Store Total

| Category | Cloud-Hosted |
|----------|-------------|
| Infrastructure | ~$1,199/month |
| Stripe fees (negotiated) | ~$277,500/month |
| **Total** | **~$278,699/month** |

---

## 5. Stripe Fee Structure Reference

### 5.1 Standard Pricing (US)

| Fee Type | Rate |
|----------|------|
| Domestic cards | 2.9% + $0.30 per transaction |
| International cards | 3.9% + $0.30 per transaction |
| Currency conversion | +1% |
| Disputes/chargebacks | $15 per dispute |
| Refunds | Processing fee not returned; $0.30 per-transaction fee is not refunded |
| Payouts | Free (standard: 2 business days) or $0.50 (instant) |

### 5.2 Volume Discount Thresholds

| Monthly Volume | Typical Negotiated Rate |
|----------------|----------------------|
| < $50K | 2.9% + $0.30 (standard) |
| $50K - $250K | 2.7% + $0.25 |
| $250K - $1M | 2.5% + $0.25 |
| $1M - $5M | 2.2% + $0.20 |
| $5M+ | Custom enterprise pricing |

### 5.3 Additional Stripe Costs (Optional Features)

| Feature | Cost | RustCommerce Usage |
|---------|------|-------------------|
| Stripe Radar (fraud) | $0.05/txn (standard free, advanced paid) | Included in standard |
| Stripe Tax | 0.5% of volume | Optional (P2 feature) |
| Stripe Billing (subscriptions) | 0.5% of volume | Optional (P2 feature) |
| Stripe Connect (marketplace) | 0.25% + fee/txn | Out of scope (single-store only) |

---

## 6. Database Storage Estimates

### 6.1 Per-Record Size Estimates

| Table | Avg Row Size | Notes |
|-------|-------------|-------|
| `rc_products` | ~2 KB | Title, description, metadata |
| `rc_product_variants` | ~500 B | SKU, price, stock per variant |
| `rc_product_images` | ~200 B | References only (not image data) |
| `rc_categories` | ~300 B | Name, slug, parent reference |
| `rc_orders` | ~1 KB | Status, totals, addresses |
| `rc_order_items` | ~300 B | Product ref, quantity, price |
| `rc_cart_items` | ~200 B | Ephemeral; cleaned up regularly |
| `rc_customers` | ~500 B | Name, email, preferences |
| `rc_addresses` | ~400 B | Full address fields |
| `rc_payments` | ~500 B | Stripe IDs, status, amount |
| `rc_reviews` | ~1 KB | Rating, body text |
| `rc_coupons` | ~500 B | Code, rules, limits |

### 6.2 Storage Projections

| Scale | Products | Orders/Year | Estimated DB Size | Index Overhead | Total DB |
|-------|----------|-------------|------------------|----------------|----------|
| Small | 100 | 18K | ~30 MB | ~15 MB | ~50 MB |
| Medium | 10K | 180K | ~400 MB | ~200 MB | ~600 MB |
| Large | 100K | 1.8M | ~5 GB | ~3 GB | ~8 GB |

> These estimates include data only. WAL (Write-Ahead Log), temporary files, and VACUUM overhead add 20-50% during normal operation.

### 6.3 Growth Rate

At steady state, database growth is driven by orders (products are relatively stable):

| Scale | Daily Growth | Monthly Growth | Annual Growth |
|-------|-------------|---------------|--------------|
| Small | ~2 MB | ~60 MB | ~700 MB |
| Medium | ~20 MB | ~600 MB | ~7 GB |
| Large | ~200 MB | ~6 GB | ~72 GB |

---

## 7. CDN and Image Storage Costs

### 7.1 Image Storage Estimates

Assuming an average of 5 images per product, with 4 size variants each (original + 3 thumbnails):

| Scale | Products | Images | Avg Size/Image | Raw Storage | With Variants (4x) |
|-------|----------|--------|---------------|-------------|---------------------|
| Small | 100 | 500 | 300 KB | 150 MB | 600 MB |
| Medium | 10K | 50K | 300 KB | 15 GB | 60 GB |
| Large | 100K | 500K | 300 KB | 150 GB | 600 GB |

### 7.2 CDN Bandwidth Estimates

Assuming 10 product page views per visitor, 5 images per page, average 100 KB per delivered image (WebP, cached):

| Scale | Daily Visitors | Page Views/Day | Images Served/Day | Bandwidth/Day | Bandwidth/Month |
|-------|---------------|---------------|-------------------|---------------|-----------------|
| Small | 500 | 5K | 25K | 2.5 GB | 75 GB |
| Medium | 10K | 100K | 500K | 50 GB | 1.5 TB |
| Large | 100K | 1M | 5M | 500 GB | 15 TB |

> CDN cache hit rates are typically 85-95% for product images. These estimates are for origin bandwidth; CDN-served bandwidth is typically 10-100x cheaper.

### 7.3 CDN Cost Comparison

| Provider | Free Tier | Pricing Model | Small | Medium | Large |
|----------|-----------|--------------|-------|--------|-------|
| **Cloudflare** (Free/Pro) | Unlimited bandwidth | Plan-based | $0 | $20/mo | $200/mo (Business) |
| **AWS CloudFront** | 1 TB/mo (12 months) | $0.085/GB | $6 | $128 | $1,275 |
| **BunnyCDN** | None | $0.01/GB | $0.75 | $15 | $150 |
| **Cloudflare R2** (storage) | 10 GB | $0.015/GB/mo | $0 | $0.75 | $9 |

**Recommendation**: Cloudflare (Free or Pro) for CDN delivery combined with S3 or R2 for storage offers the best cost-performance ratio at all scales.

---

## 8. Cost Summary by Scale

### Monthly Recurring Costs (Infrastructure Only, Excluding Stripe)

| Component | Small | Medium | Large |
|-----------|-------|--------|-------|
| Compute | $15-20 | $40-60 | $144 |
| Database | $0 (on VPS) | $0-60 | $558 |
| Cache | $0 (moka) | $0-13 | $199 |
| Storage | $0 (local) | $1-5 | $23 |
| CDN | $0 | $0-20 | $150-200 |
| Monitoring | $0 | $0-7 | $20 |
| Backups | $2-4 | $1-5 | $30 |
| Other (WAF, DNS) | $0 | $0 | $10 |
| **Total Infrastructure** | **$17-24** | **$42-170** | **$1,134-1,184** |

### Annual Infrastructure Cost

| Scale | Low Estimate | High Estimate |
|-------|-------------|--------------|
| Small | $204/year | $288/year |
| Medium | $504/year | $2,040/year |
| Large | $13,608/year | $14,208/year |

> These are infrastructure costs only. Stripe processing fees are proportional to revenue and are typically 50-200x the infrastructure cost, making infrastructure costs a negligible fraction of total cost of ownership for any store generating meaningful revenue.
