-- Drop audit triggers
DROP TRIGGER IF EXISTS audit_subscription_usage_trigger ON subscription_schema.subscription_usage;
DROP TRIGGER IF EXISTS audit_subscriptions_trigger ON subscription_schema.subscriptions;
DROP TRIGGER IF EXISTS audit_subscription_plans_trigger ON subscription_schema.subscription_plans;

-- Drop indexes
DROP INDEX IF EXISTS idx_subscription_usage_period;
DROP INDEX IF EXISTS idx_subscription_usage_subscription_id;
DROP INDEX IF EXISTS idx_subscriptions_current_period_end;
DROP INDEX IF EXISTS idx_subscriptions_status;
DROP INDEX IF EXISTS idx_subscriptions_user_id;
DROP INDEX IF EXISTS idx_subscription_plans_is_active;

-- Drop tables
DROP TABLE IF EXISTS subscription_schema.subscription_usage;
DROP TABLE IF EXISTS subscription_schema.subscriptions;
DROP TABLE IF EXISTS subscription_schema.subscription_plans;

-- Drop schema
DROP SCHEMA IF EXISTS subscription_schema CASCADE;
