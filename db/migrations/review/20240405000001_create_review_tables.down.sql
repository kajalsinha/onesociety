-- Drop audit triggers
DROP TRIGGER IF EXISTS audit_user_reviews_trigger ON review_schema.user_reviews;
DROP TRIGGER IF EXISTS audit_product_reviews_trigger ON review_schema.product_reviews;

-- Drop indexes
DROP INDEX IF EXISTS idx_user_reviews_rating;
DROP INDEX IF EXISTS idx_user_reviews_review_type;
DROP INDEX IF EXISTS idx_user_reviews_rental_id;
DROP INDEX IF EXISTS idx_user_reviews_reviewer_id;
DROP INDEX IF EXISTS idx_user_reviews_reviewed_user_id;
DROP INDEX IF EXISTS idx_product_reviews_rating;
DROP INDEX IF EXISTS idx_product_reviews_reviewer_id;
DROP INDEX IF EXISTS idx_product_reviews_product_id;

-- Drop tables
DROP TABLE IF EXISTS review_schema.user_reviews;
DROP TABLE IF EXISTS review_schema.product_reviews;

-- Drop schema
DROP SCHEMA IF EXISTS review_schema CASCADE;
