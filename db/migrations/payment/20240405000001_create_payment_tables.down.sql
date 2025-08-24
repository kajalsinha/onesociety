-- Drop audit triggers
DROP TRIGGER IF EXISTS audit_payment_transactions_trigger ON payment_schema.payment_transactions;
DROP TRIGGER IF EXISTS audit_payment_intents_trigger ON payment_schema.payment_intents;
DROP TRIGGER IF EXISTS audit_payment_methods_trigger ON payment_schema.payment_methods;

-- Drop indexes
DROP INDEX IF EXISTS idx_payment_transactions_status;
DROP INDEX IF EXISTS idx_payment_transactions_payment_intent_id;
DROP INDEX IF EXISTS idx_payment_intents_provider_payment_intent_id;
DROP INDEX IF EXISTS idx_payment_intents_status;
DROP INDEX IF EXISTS idx_payment_intents_subscription_id;
DROP INDEX IF EXISTS idx_payment_intents_rental_id;
DROP INDEX IF EXISTS idx_payment_intents_user_id;
DROP INDEX IF EXISTS idx_payment_methods_is_default;
DROP INDEX IF EXISTS idx_payment_methods_user_id;

-- Drop tables
DROP TABLE IF EXISTS payment_schema.payment_transactions;
DROP TABLE IF EXISTS payment_schema.payment_intents;
DROP TABLE IF EXISTS payment_schema.payment_methods;

-- Drop schema
DROP SCHEMA IF EXISTS payment_schema CASCADE;
