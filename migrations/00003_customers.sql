-- RustCommerce Migration 00003: Customers
-- Customer profiles and addresses

-- Customers table
CREATE TABLE rc_customers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID,
    email VARCHAR(255) NOT NULL UNIQUE,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    phone VARCHAR(50),
    company VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    accepts_marketing BOOLEAN NOT NULL DEFAULT FALSE,
    total_orders INTEGER NOT NULL DEFAULT 0,
    total_spent DECIMAL(12, 2) NOT NULL DEFAULT 0.00,
    notes TEXT,
    last_order_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rc_customers_user_id ON rc_customers(user_id);
CREATE INDEX idx_rc_customers_email ON rc_customers(email);
CREATE INDEX idx_rc_customers_is_active ON rc_customers(is_active);
CREATE INDEX idx_rc_customers_created_at ON rc_customers(created_at);

-- Customer addresses table
CREATE TABLE rc_customer_addresses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    customer_id UUID NOT NULL REFERENCES rc_customers(id) ON DELETE CASCADE,
    label VARCHAR(50),
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    company VARCHAR(255),
    address_line1 VARCHAR(255) NOT NULL,
    address_line2 VARCHAR(255),
    city VARCHAR(100) NOT NULL,
    state VARCHAR(100),
    postal_code VARCHAR(20) NOT NULL,
    country VARCHAR(2) NOT NULL,
    phone VARCHAR(50),
    is_default_shipping BOOLEAN NOT NULL DEFAULT FALSE,
    is_default_billing BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rc_customer_addresses_customer_id ON rc_customer_addresses(customer_id);

-- Add foreign key from carts to customers
ALTER TABLE rc_carts
    ADD CONSTRAINT fk_rc_carts_customer
    FOREIGN KEY (customer_id) REFERENCES rc_customers(id) ON DELETE SET NULL;

-- Add foreign key from orders to customers
ALTER TABLE rc_orders
    ADD CONSTRAINT fk_rc_orders_customer
    FOREIGN KEY (customer_id) REFERENCES rc_customers(id) ON DELETE SET NULL;

-- Triggers
CREATE TRIGGER trg_rc_customers_updated_at
    BEFORE UPDATE ON rc_customers
    FOR EACH ROW EXECUTE FUNCTION rc_update_timestamp();

CREATE TRIGGER trg_rc_customer_addresses_updated_at
    BEFORE UPDATE ON rc_customer_addresses
    FOR EACH ROW EXECUTE FUNCTION rc_update_timestamp();
