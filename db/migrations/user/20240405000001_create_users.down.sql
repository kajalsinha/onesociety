-- Migration: create_users
-- Service: user
-- Created at: 2024-04-05 00:00:01 UTC

BEGIN;

-- Drop triggers first
DROP TRIGGER IF EXISTS user_profiles_audit_trigger ON user_schema.user_profiles;
DROP TRIGGER IF EXISTS users_audit_trigger ON user_schema.users;
DROP TRIGGER IF EXISTS user_profiles_set_updated_at ON user_schema.user_profiles;
DROP TRIGGER IF EXISTS users_set_updated_at ON user_schema.users;

-- Drop functions
DROP FUNCTION IF EXISTS user_schema.set_updated_at();

-- Drop indexes
DROP INDEX IF EXISTS user_schema.idx_user_profiles_verification_status;
DROP INDEX IF EXISTS user_schema.idx_users_status;
DROP INDEX IF EXISTS user_schema.idx_user_profiles_user_id;
DROP INDEX IF EXISTS user_schema.idx_users_email;

-- Drop tables
DROP TABLE IF EXISTS user_schema.user_profiles;
DROP TABLE IF EXISTS user_schema.users;

COMMIT; 