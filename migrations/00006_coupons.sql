-- RustCommerce Migration 00006: Coupons
-- Discount coupons and promotional codes

-- Discount type enum
CREATE TYPE rc_discount_type AS ENUM ('percentage', 'fixed_amount', 'free_shipping');

-- Coupons table
CREATE TABLE rc_coupons (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    discount_type rc_discount_type NOT NULL,
    discount_value DECIMAL(10, 2) NOT NULL,
    minimum_order_amount DECIMAL(10, 2),
    maximum_discount_amount DECIMAL(10, 2),
    usage_limit INTEGER,
    usage_count INTEGER NOT NULL DEFAULT 0,
    usage_limit_per_customer INTEGER,
    applies_to_product_ids UUID[] DEFAULT '{}',
    applies_to_category_ids UUID[] DEFAULT '{}',
    excluded_product_ids UUID[] DEFAULT '{}',
    excluded_category_ids UUID[] DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    starts_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_rc_coupons_discount_value CHECK (discount_value > 0),
    CONSTRAINT chk_rc_coupons_usage_limit CHECK (usage_limit IS NULL OR usage_limit > 0),
    CONSTRAINT chk_rc_coupons_dates CHECK (expires_at IS NULL OR starts_at IS NULL OR expires_at > starts_at)
);

CREATE INDEX idx_rc_coupons_code ON rc_coupons(code);
CREATE INDEX idx_rc_coupons_is_active ON rc_coupons(is_active);
CREATE INDEX idx_rc_coupons_starts_at ON rc_coupons(starts_at);
CREATE INDEX idx_rc_coupons_expires_at ON rc_coupons(expires_at);

-- Trigger
CREATE TRIGGER trg_rc_coupons_updated_at
    BEFORE UPDATE ON rc_coupons
    FOR EACH ROW EXECUTE FUNCTION rc_update_timestamp();
