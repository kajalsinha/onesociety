-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "postgis";  -- For location-based features

-- Create logical schemas for future separation
CREATE SCHEMA IF NOT EXISTS user_schema;
CREATE SCHEMA IF NOT EXISTS product_schema;
CREATE SCHEMA IF NOT EXISTS rental_schema;
CREATE SCHEMA IF NOT EXISTS billing_schema;
CREATE SCHEMA IF NOT EXISTS audit_schema;
CREATE SCHEMA IF NOT EXISTS messaging_schema;
CREATE SCHEMA IF NOT EXISTS review_schema;
CREATE SCHEMA IF NOT EXISTS subscription_schema;

-- User Service Tables (PII isolation)
CREATE TABLE user_schema.users (
    user_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    phone_number VARCHAR(15),
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_login TIMESTAMPTZ,
    status VARCHAR(20) DEFAULT 'active'
);

CREATE TABLE user_schema.user_profiles (
    profile_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES user_schema.users(user_id),
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    address JSONB,
    verification_status VARCHAR(20) DEFAULT 'pending',
    avg_rating NUMERIC(3,2),
    total_reviews INTEGER DEFAULT 0,
    identity_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Product Categories and Tags
CREATE TABLE product_schema.categories (
    category_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    parent_category_id UUID REFERENCES product_schema.categories(category_id),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE product_schema.tags (
    tag_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(50) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Product Service Tables
CREATE TABLE product_schema.products (
    product_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_id UUID REFERENCES user_schema.users(user_id),
    category_id UUID REFERENCES product_schema.categories(category_id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    specifications JSONB,
    daily_price NUMERIC(10,2) NOT NULL,
    deposit_amount NUMERIC(10,2),
    availability_schedule TS_RANGE,
    location GEOGRAPHY(POINT),  -- PostGIS point for lat/long
    address JSONB,
    avg_rating NUMERIC(3,2),
    total_reviews INTEGER DEFAULT 0,
    insurance_required BOOLEAN DEFAULT false,
    status VARCHAR(20) DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE product_schema.product_tags (
    product_id UUID REFERENCES product_schema.products(product_id),
    tag_id UUID REFERENCES product_schema.tags(tag_id),
    PRIMARY KEY (product_id, tag_id)
);

CREATE TABLE product_schema.product_images (
    image_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_id UUID REFERENCES product_schema.products(product_id),
    image_url TEXT NOT NULL,
    is_primary BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Wishlist
CREATE TABLE product_schema.wishlists (
    wishlist_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES user_schema.users(user_id),
    product_id UUID REFERENCES product_schema.products(product_id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, product_id)
);

-- Rental Service Tables
CREATE TABLE rental_schema.rentals (
    rental_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    product_id UUID REFERENCES product_schema.products(product_id),
    renter_id UUID REFERENCES user_schema.users(user_id),
    rental_period TS_RANGE NOT NULL,
    status VARCHAR(20) DEFAULT 'requested',
    insurance_policy_id UUID,
    pickup_notes TEXT,
    return_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insurance Tables
CREATE TABLE rental_schema.insurance_policies (
    policy_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    rental_id UUID REFERENCES rental_schema.rentals(rental_id),
    provider VARCHAR(100),
    policy_number VARCHAR(100),
    coverage_amount NUMERIC(12,2),
    status VARCHAR(20),
    valid_from TIMESTAMPTZ,
    valid_until TIMESTAMPTZ,
    terms JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Review System
CREATE TABLE review_schema.reviews (
    review_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    rental_id UUID REFERENCES rental_schema.rentals(rental_id),
    reviewer_id UUID REFERENCES user_schema.users(user_id),
    reviewee_id UUID REFERENCES user_schema.users(user_id),
    product_id UUID REFERENCES product_schema.products(product_id),
    rating INTEGER CHECK (rating BETWEEN 1 AND 5),
    review_text TEXT,
    review_type VARCHAR(20) CHECK (review_type IN ('product', 'renter', 'owner')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Billing Service Tables
CREATE TABLE billing_schema.transactions (
    transaction_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    rental_id UUID REFERENCES rental_schema.rentals(rental_id),
    amount NUMERIC(12,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'EUR',
    payment_method_token TEXT,
    payment_type VARCHAR(20),
    status VARCHAR(20) DEFAULT 'pending',
    escrow_release_conditions JSONB,
    commission_amount NUMERIC(12,2),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Subscription System
CREATE TABLE subscription_schema.plans (
    plan_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    price NUMERIC(10,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'EUR',
    billing_interval VARCHAR(20),
    features JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE subscription_schema.subscriptions (
    subscription_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES user_schema.users(user_id),
    plan_id UUID REFERENCES subscription_schema.plans(plan_id),
    status VARCHAR(20) DEFAULT 'active',
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    cancel_at_period_end BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Audit Service Tables
CREATE TABLE audit_schema.audit_logs (
    audit_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    service_name VARCHAR(50) NOT NULL,
    table_name VARCHAR(50) NOT NULL,
    operation VARCHAR(10) NOT NULL,
    user_id UUID,
    old_data JSONB,
    new_data JSONB,
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- Messaging Service Tables
CREATE TABLE messaging_schema.messages (
    message_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sender_id UUID REFERENCES user_schema.users(user_id),
    receiver_id UUID REFERENCES user_schema.users(user_id),
    content TEXT NOT NULL,
    sent_at TIMESTAMPTZ DEFAULT NOW(),
    read_status BOOLEAN DEFAULT false
);

-- Audit Trigger Function
CREATE OR REPLACE FUNCTION audit_schema.log_audit_event()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        INSERT INTO audit_schema.audit_logs(
            service_name,
            table_name,
            operation,
            user_id,
            old_data
        ) VALUES (
            TG_TABLE_SCHEMA,
            TG_TABLE_NAME,
            TG_OP,
            OLD.user_id,
            row_to_json(OLD)
        );
    ELSE
        INSERT INTO audit_schema.audit_logs(
            service_name,
            table_name,
            operation,
            user_id,
            old_data,
            new_data
        ) VALUES (
            TG_TABLE_SCHEMA,
            TG_TABLE_NAME,
            TG_OP,
            NEW.user_id,
            CASE WHEN TG_OP = 'UPDATE' THEN row_to_json(OLD) ELSE NULL END,
            row_to_json(NEW)
        );
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Example Trigger for User Table
CREATE TRIGGER users_audit_trigger
AFTER INSERT OR UPDATE OR DELETE ON user_schema.users
FOR EACH ROW EXECUTE FUNCTION audit_schema.log_audit_event();
