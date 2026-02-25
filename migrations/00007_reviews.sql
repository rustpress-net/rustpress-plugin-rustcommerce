-- RustCommerce Migration 00007: Product Reviews
-- Customer product reviews and ratings

-- Review status enum
CREATE TYPE rc_review_status AS ENUM ('pending', 'approved', 'rejected', 'spam');

-- Reviews table
CREATE TABLE rc_reviews (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_id UUID NOT NULL REFERENCES rc_products(id) ON DELETE CASCADE,
    customer_id UUID REFERENCES rc_customers(id) ON DELETE SET NULL,
    order_id UUID REFERENCES rc_orders(id) ON DELETE SET NULL,
    reviewer_name VARCHAR(255) NOT NULL,
    reviewer_email VARCHAR(255) NOT NULL,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    title VARCHAR(255),
    content TEXT,
    status rc_review_status NOT NULL DEFAULT 'pending',
    is_verified_purchase BOOLEAN NOT NULL DEFAULT FALSE,
    helpful_count INTEGER NOT NULL DEFAULT 0,
    admin_response TEXT,
    admin_responded_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rc_reviews_product_id ON rc_reviews(product_id);
CREATE INDEX idx_rc_reviews_customer_id ON rc_reviews(customer_id);
CREATE INDEX idx_rc_reviews_status ON rc_reviews(status);
CREATE INDEX idx_rc_reviews_rating ON rc_reviews(rating);
CREATE INDEX idx_rc_reviews_created_at ON rc_reviews(created_at);

-- Prevent duplicate reviews per customer per product
CREATE UNIQUE INDEX idx_rc_reviews_unique_customer_product
    ON rc_reviews(customer_id, product_id)
    WHERE customer_id IS NOT NULL;

-- Trigger
CREATE TRIGGER trg_rc_reviews_updated_at
    BEFORE UPDATE ON rc_reviews
    FOR EACH ROW EXECUTE FUNCTION rc_update_timestamp();
