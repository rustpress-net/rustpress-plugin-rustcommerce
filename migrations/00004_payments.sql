-- RustCommerce Migration 00004: Payments
-- Payment records and tracking

-- Payment status enum
CREATE TYPE rc_payment_status AS ENUM (
    'pending',
    'processing',
    'completed',
    'failed',
    'cancelled',
    'refunded',
    'partially_refunded'
);

-- Payment method enum
CREATE TYPE rc_payment_method AS ENUM (
    'stripe',
    'paypal',
    'bank_transfer',
    'cash_on_delivery',
    'manual'
);

-- Payments table
CREATE TABLE rc_payments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID NOT NULL REFERENCES rc_orders(id) ON DELETE CASCADE,
    payment_method rc_payment_method NOT NULL,
    status rc_payment_status NOT NULL DEFAULT 'pending',
    amount DECIMAL(10, 2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    gateway_transaction_id VARCHAR(255),
    gateway_response JSONB,
    refund_amount DECIMAL(10, 2) DEFAULT 0.00,
    refund_reason TEXT,
    metadata JSONB,
    paid_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    refunded_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rc_payments_order_id ON rc_payments(order_id);
CREATE INDEX idx_rc_payments_status ON rc_payments(status);
CREATE INDEX idx_rc_payments_gateway_transaction_id ON rc_payments(gateway_transaction_id);
CREATE INDEX idx_rc_payments_created_at ON rc_payments(created_at);

-- Trigger
CREATE TRIGGER trg_rc_payments_updated_at
    BEFORE UPDATE ON rc_payments
    FOR EACH ROW EXECUTE FUNCTION rc_update_timestamp();
