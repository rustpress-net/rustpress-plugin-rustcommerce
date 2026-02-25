-- RustCommerce Migration 00001: Core E-commerce Tables
-- Products, categories, variants, images, and junction tables

-- Enable UUID extension if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Product status enum
CREATE TYPE rc_product_status AS ENUM ('draft', 'active', 'archived', 'out_of_stock');

-- Categories table (self-referencing for hierarchy)
CREATE TABLE rc_categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    parent_id UUID REFERENCES rc_categories(id) ON DELETE SET NULL,
    image_url TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    meta_title VARCHAR(255),
    meta_description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rc_categories_parent_id ON rc_categories(parent_id);
CREATE INDEX idx_rc_categories_slug ON rc_categories(slug);
CREATE INDEX idx_rc_categories_is_active ON rc_categories(is_active);

-- Products table
CREATE TABLE rc_products (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    short_description TEXT,
    sku VARCHAR(100) UNIQUE,
    price DECIMAL(10, 2) NOT NULL,
    compare_at_price DECIMAL(10, 2),
    cost_price DECIMAL(10, 2),
    status rc_product_status NOT NULL DEFAULT 'draft',
    is_featured BOOLEAN NOT NULL DEFAULT FALSE,
    is_digital BOOLEAN NOT NULL DEFAULT FALSE,
    weight DECIMAL(8, 3),
    width DECIMAL(8, 2),
    height DECIMAL(8, 2),
    depth DECIMAL(8, 2),
    stock_quantity INTEGER NOT NULL DEFAULT 0,
    low_stock_threshold INTEGER NOT NULL DEFAULT 5,
    track_inventory BOOLEAN NOT NULL DEFAULT TRUE,
    allow_backorders BOOLEAN NOT NULL DEFAULT FALSE,
    meta_title VARCHAR(255),
    meta_description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rc_products_slug ON rc_products(slug);
CREATE INDEX idx_rc_products_sku ON rc_products(sku);
CREATE INDEX idx_rc_products_status ON rc_products(status);
CREATE INDEX idx_rc_products_is_featured ON rc_products(is_featured);
CREATE INDEX idx_rc_products_price ON rc_products(price);
CREATE INDEX idx_rc_products_created_at ON rc_products(created_at);

-- Product variants table
CREATE TABLE rc_product_variants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_id UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    sku VARCHAR(100) UNIQUE,
    price DECIMAL(10, 2) NOT NULL,
    compare_at_price DECIMAL(10, 2),
    cost_price DECIMAL(10, 2),
    stock_quantity INTEGER NOT NULL DEFAULT 0,
    weight DECIMAL(8, 3),
    option1_name VARCHAR(100),
    option1_value VARCHAR(255),
    option2_name VARCHAR(100),
    option2_value VARCHAR(255),
    option3_name VARCHAR(100),
    option3_value VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rc_product_variants_product_id ON rc_product_variants(product_id);
CREATE INDEX idx_rc_product_variants_sku ON rc_product_variants(sku);

-- Product images table
CREATE TABLE rc_product_images (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_id UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    variant_id UUID REFERENCES rc_product_variants(id) ON DELETE SET NULL,
    url TEXT NOT NULL,
    alt_text VARCHAR(255),
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rc_product_images_product_id ON rc_product_images(product_id);
CREATE INDEX idx_rc_product_images_variant_id ON rc_product_images(variant_id);

-- Product-Category junction table (many-to-many)
CREATE TABLE rc_product_categories (
    product_id UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    category_id UUID NOT NULL REFERENCES rc_categories(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (product_id, category_id)
);

CREATE INDEX idx_rc_product_categories_category_id ON rc_product_categories(category_id);

-- Trigger function for updating updated_at timestamps
CREATE OR REPLACE FUNCTION rc_update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_rc_categories_updated_at
    BEFORE UPDATE ON rc_categories
    FOR EACH ROW EXECUTE FUNCTION rc_update_timestamp();

CREATE TRIGGER trg_rc_products_updated_at
    BEFORE UPDATE ON rc_products
    FOR EACH ROW EXECUTE FUNCTION rc_update_timestamp();

CREATE TRIGGER trg_rc_product_variants_updated_at
    BEFORE UPDATE ON rc_product_variants
    FOR EACH ROW EXECUTE FUNCTION rc_update_timestamp();
