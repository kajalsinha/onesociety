-- Migration: create_users
-- Service: user
-- Created at: 2024-04-05 00:00:01 UTC

BEGIN;

-- Enable required extensions if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossg";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create users table in user_schema
CREATE TABLE IF NOT EXISTS user_schema.users (
    user_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    phone_number VARCHAR(15),
    password_hash TEXT NOT NULL,
    status VARCHAR(20) DEFAULT 'active',
    last_login TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Create user_profiles table in user_schema
CREATE TABLE IF NOT EXISTS user_schema.user_profiles (
    profile_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user_schema.users(user_id) ON DELETE CASCADE,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    address JSONB,
    verification_status VARCHAR(20) DEFAULT 'pending',
    avg_rating NUMERIC(3,2),
    total_reviews INTEGER DEFAULT 0,
    identity_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_users_email ON user_schema.users(email);
CREATE INDEX idx_user_profiles_user_id ON user_schema.user_profiles(user_id);
CREATE INDEX idx_users_status ON user_schema.users(status);
CREATE INDEX idx_user_profiles_verification_status ON user_schema.user_profiles(verification_status);

-- Create trigger for updated_at
CREATE OR REPLACE FUNCTION user_schema.set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_set_updated_at
    BEFORE UPDATE ON user_schema.users
    FOR EACH ROW
    EXECUTE FUNCTION user_schema.set_updated_at();

CREATE TRIGGER user_profiles_set_updated_at
    BEFORE UPDATE ON user_schema.user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION user_schema.set_updated_at();

-- Create audit trigger
CREATE TRIGGER users_audit_trigger
    AFTER INSERT OR UPDATE OR DELETE ON user_schema.users
    FOR EACH ROW
    EXECUTE FUNCTION audit_schema.log_audit_event();

CREATE TRIGGER user_profiles_audit_trigger
    AFTER INSERT OR UPDATE OR DELETE ON user_schema.user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION audit_schema.log_audit_event();

COMMIT; 