-- RustCommerce Migration 00002: Cart and Orders
-- Shopping carts, cart items, orders, and order items

-- Order status enum
CREATE TYPE rc_order_status AS ENUM (
    'pending',
    'confirmed',
    'processing',
    'shipped',
    'delivered',
    'cancelled',
    'refunded',
    'failed'
);

-- Carts table
CREATE TABLE rc_carts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    customer_id UUID,
    session_id VARCHAR(255),
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    subtotal DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    tax_total DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    shipping_total DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    discount_total DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    grand_total DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    coupon_code VARCHAR(100),
    notes TEXT,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rc_carts_customer_id ON rc_carts(customer_id);
CREATE INDEX idx_rc_carts_session_id ON rc_carts(session_id);
CREATE INDEX idx_rc_carts_expires_at ON rc_carts(expires_at);

-- Cart items table
CREATE TABLE rc_cart_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    cart_id UUID NOT NULL REFERENCES rc_carts(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    variant_id UUID REFERENCES rc_product_variants(id) ON DELETE SET NULL,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price DECIMAL(10, 2) NOT NULL,
    total_price DECIMAL(10, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (cart_id, product_id, variant_id)
);

CREATE INDEX idx_rc_cart_items_cart_id ON rc_cart_items(cart_id);
CREATE INDEX idx_rc_cart_items_product_id ON rc_cart_items(product_id);

-- Orders table
CREATE TABLE rc_orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_number VARCHAR(50) NOT NULL UNIQUE,
    customer_id UUID,
    customer_email VARCHAR(255) NOT NULL,
    customer_name VARCHAR(255) NOT NULL,
    status rc_order_status NOT NULL DEFAULT 'pending',
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    subtotal DECIMAL(10, 2) NOT NULL,
    tax_total DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    shipping_total DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    discount_total DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    grand_total DECIMAL(10, 2) NOT NULL,
    shipping_address_line1 VARCHAR(255),
    shipping_address_line2 VARCHAR(255),
    shipping_city VARCHAR(100),
    shipping_state VARCHAR(100),
    shipping_postal_code VARCHAR(20),
    shipping_country VARCHAR(2),
    billing_address_line1 VARCHAR(255),
    billing_address_line2 VARCHAR(255),
    billing_city VARCHAR(100),
    billing_state VARCHAR(100),
    billing_postal_code VARCHAR(20),
    billing_country VARCHAR(2),
    shipping_method VARCHAR(100),
    tracking_number VARCHAR(255),
    coupon_code VARCHAR(100),
    notes TEXT,
    internal_notes TEXT,
    shipped_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rc_orders_order_number ON rc_orders(order_number);
CREATE INDEX idx_rc_orders_customer_id ON rc_orders(customer_id);
CREATE INDEX idx_rc_orders_customer_email ON rc_orders(customer_email);
CREATE INDEX idx_rc_orders_status ON rc_orders(status);
CREATE INDEX idx_rc_orders_created_at ON rc_orders(created_at);

-- Order items table
CREATE TABLE rc_order_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID NOT NULL REFERENCES rc_orders(id) ON DELETE CASCADE,
    product_id UUID NOT NULL,
    variant_id UUID,
    product_name VARCHAR(255) NOT NULL,
    variant_name VARCHAR(255),
    sku VARCHAR(100),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price DECIMAL(10, 2) NOT NULL,
    total_price DECIMAL(10, 2) NOT NULL,
    tax_amount DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    discount_amount DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rc_order_items_order_id ON rc_order_items(order_id);
CREATE INDEX idx_rc_order_items_product_id ON rc_order_items(product_id);

-- Triggers
CREATE TRIGGER trg_rc_carts_updated_at
    BEFORE UPDATE ON rc_carts
    FOR EACH ROW EXECUTE FUNCTION rc_update_timestamp();

CREATE TRIGGER trg_rc_cart_items_updated_at
    BEFORE UPDATE ON rc_cart_items
    FOR EACH ROW EXECUTE FUNCTION rc_update_timestamp();

CREATE TRIGGER trg_rc_orders_updated_at
    BEFORE UPDATE ON rc_orders
    FOR EACH ROW EXECUTE FUNCTION rc_update_timestamp();
