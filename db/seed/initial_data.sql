-- Initial seed data for OneSociety database

BEGIN;

-- Insert some test users
INSERT INTO user_schema.users (user_id, email, phone_number, password_hash, status)
VALUES
    ('11111111-1111-1111-1111-111111111111', 'admin@example.com', '+1234567890', 
     crypt('admin123', gen_salt('bf')), 'active'),
    ('22222222-2222-2222-2222-222222222222', 'user@example.com', '+1234567891',
     crypt('user123', gen_salt('bf')), 'active');

-- Insert user profiles
INSERT INTO user_schema.user_profiles (profile_id, user_id, first_name, last_name, verification_status)
VALUES
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '11111111-1111-1111-1111-111111111111',
     'Admin', 'User', 'verified'),
    ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '22222222-2222-2222-2222-222222222222',
     'Test', 'User', 'verified');

-- Insert product categories
INSERT INTO product_schema.categories (category_id, name, description)
VALUES
    ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'Electronics', 'Electronic devices and gadgets'),
    ('dddddddd-dddd-dddd-dddd-dddddddddddd', 'Tools', 'Hand and power tools'),
    ('eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 'Sports', 'Sports and outdoor equipment');

-- Insert some tags
INSERT INTO product_schema.tags (tag_id, name)
VALUES
    ('ffffffff-ffff-ffff-ffff-ffffffffffff', 'premium'),
    ('gggggggg-gggg-gggg-gggg-gggggggggggg', 'popular'),
    ('hhhhhhhh-hhhh-hhhh-hhhh-hhhhhhhhhhhh', 'new');

-- Insert subscription plans
INSERT INTO subscription_schema.subscription_plans (id, name, description, price_cents, currency, billing_cycle, features, max_listings, max_rentals_per_month, priority_support)
VALUES
    ('iiiiiiii-iiii-iiii-iiii-iiiiiiiiiiii', 'Basic', 'Basic subscription plan for casual users', 999, 'USD', 'monthly',
     '["basic_support", "standard_listings"]'::jsonb, 10, 5, false),
    ('jjjjjjjj-jjjj-jjjj-jjjj-jjjjjjjjjjjj', 'Premium', 'Premium subscription plan for active users', 1999, 'USD', 'monthly',
     '["priority_support", "featured_listings", "analytics"]'::jsonb, 50, 20, true),
    ('kkkkkkkk-kkkk-kkkk-kkkk-kkkkkkkkkkkk', 'Professional', 'Professional subscription plan for power users', 4999, 'USD', 'monthly',
     '["dedicated_support", "unlimited_listings", "advanced_analytics", "api_access"]'::jsonb, null, null, true);

COMMIT; 