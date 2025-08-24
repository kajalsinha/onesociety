-- Create review schema
CREATE SCHEMA IF NOT EXISTS review_schema;

-- Create product reviews table
CREATE TABLE review_schema.product_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES product_schema.products(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL REFERENCES user_schema.users(id) ON DELETE CASCADE,
    rental_id UUID REFERENCES rental_schema.rentals(id) ON DELETE SET NULL,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    title VARCHAR(200),
    content TEXT,
    is_verified_rental BOOLEAN NOT NULL DEFAULT FALSE,
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'hidden', 'flagged')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(product_id, reviewer_id, rental_id)
);

-- Create user reviews table (for renter/owner ratings)
CREATE TABLE review_schema.user_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reviewed_user_id UUID NOT NULL REFERENCES user_schema.users(id) ON DELETE CASCADE,
    reviewer_id UUID NOT NULL REFERENCES user_schema.users(id) ON DELETE CASCADE,
    rental_id UUID NOT NULL REFERENCES rental_schema.rentals(id) ON DELETE CASCADE,
    review_type VARCHAR(20) NOT NULL CHECK (review_type IN ('renter', 'owner')),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    title VARCHAR(200),
    content TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'hidden', 'flagged')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(reviewed_user_id, reviewer_id, rental_id, review_type)
);

-- Create indexes
CREATE INDEX idx_product_reviews_product_id ON review_schema.product_reviews(product_id);
CREATE INDEX idx_product_reviews_reviewer_id ON review_schema.product_reviews(reviewer_id);
CREATE INDEX idx_product_reviews_rating ON review_schema.product_reviews(rating);
CREATE INDEX idx_user_reviews_reviewed_user_id ON review_schema.user_reviews(reviewed_user_id);
CREATE INDEX idx_user_reviews_reviewer_id ON review_schema.user_reviews(reviewer_id);
CREATE INDEX idx_user_reviews_rental_id ON review_schema.user_reviews(rental_id);
CREATE INDEX idx_user_reviews_review_type ON review_schema.user_reviews(review_type);
CREATE INDEX idx_user_reviews_rating ON review_schema.user_reviews(rating);

-- Create audit triggers
CREATE TRIGGER audit_product_reviews_trigger
    AFTER INSERT OR UPDATE OR DELETE ON review_schema.product_reviews
    FOR EACH ROW EXECUTE FUNCTION audit_schema.audit_trigger();

CREATE TRIGGER audit_user_reviews_trigger
    AFTER INSERT OR UPDATE OR DELETE ON review_schema.user_reviews
    FOR EACH ROW EXECUTE FUNCTION audit_schema.audit_trigger();
