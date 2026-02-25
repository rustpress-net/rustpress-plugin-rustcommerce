-- RustCommerce Migration 00005: Shipping and Tax
-- Shipping zones, methods, and tax rates

-- Shipping type enum
CREATE TYPE rc_shipping_type AS ENUM ('flat_rate', 'free', 'weight_based', 'price_based', 'per_item');

-- Tax type enum
CREATE TYPE rc_tax_type AS ENUM ('percentage', 'fixed');

-- Shipping zones table
CREATE TABLE rc_shipping_zones (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    countries TEXT[] NOT NULL DEFAULT '{}',
    states TEXT[] NOT NULL DEFAULT '{}',
    postal_codes TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rc_shipping_zones_is_active ON rc_shipping_zones(is_active);

-- Shipping methods table
CREATE TABLE rc_shipping_methods (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    zone_id UUID NOT NULL REFERENCES rc_shipping_zones(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    shipping_type rc_shipping_type NOT NULL DEFAULT 'flat_rate',
    price DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    min_order_amount DECIMAL(10, 2),
    max_order_amount DECIMAL(10, 2),
    min_weight DECIMAL(8, 3),
    max_weight DECIMAL(8, 3),
    estimated_days_min INTEGER,
    estimated_days_max INTEGER,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rc_shipping_methods_zone_id ON rc_shipping_methods(zone_id);
CREATE INDEX idx_rc_shipping_methods_is_active ON rc_shipping_methods(is_active);

-- Tax rates table
CREATE TABLE rc_tax_rates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    rate DECIMAL(6, 4) NOT NULL,
    tax_type rc_tax_type NOT NULL DEFAULT 'percentage',
    country VARCHAR(2) NOT NULL,
    state VARCHAR(100),
    postal_code VARCHAR(20),
    priority INTEGER NOT NULL DEFAULT 0,
    is_compound BOOLEAN NOT NULL DEFAULT FALSE,
    is_shipping_taxed BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rc_tax_rates_country ON rc_tax_rates(country);
CREATE INDEX idx_rc_tax_rates_state ON rc_tax_rates(state);
CREATE INDEX idx_rc_tax_rates_is_active ON rc_tax_rates(is_active);

-- Triggers
CREATE TRIGGER trg_rc_shipping_zones_updated_at
    BEFORE UPDATE ON rc_shipping_zones
    FOR EACH ROW EXECUTE FUNCTION rc_update_timestamp();

CREATE TRIGGER trg_rc_shipping_methods_updated_at
    BEFORE UPDATE ON rc_shipping_methods
    FOR EACH ROW EXECUTE FUNCTION rc_update_timestamp();

CREATE TRIGGER trg_rc_tax_rates_updated_at
    BEFORE UPDATE ON rc_tax_rates
    FOR EACH ROW EXECUTE FUNCTION rc_update_timestamp();
