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
INSERT INTO subscription_schema.plans (plan_id, name, description, price, currency, billing_interval, features)
VALUES
    ('iiiiiiii-iiii-iiii-iiii-iiiiiiiiiiii', 'Basic', 'Basic subscription plan', 9.99, 'EUR', 'monthly',
     '{"max_listings": 10, "featured_listings": 0}'::jsonb),
    ('jjjjjjjj-jjjj-jjjj-jjjj-jjjjjjjjjjjj', 'Premium', 'Premium subscription plan', 19.99, 'EUR', 'monthly',
     '{"max_listings": 50, "featured_listings": 5}'::jsonb),
    ('kkkkkkkk-kkkk-kkkk-kkkk-kkkkkkkkkkkk', 'Professional', 'Professional subscription plan', 49.99, 'EUR', 'monthly',
     '{"max_listings": -1, "featured_listings": 20}'::jsonb);

COMMIT; 