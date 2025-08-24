-- Create payment schema
CREATE SCHEMA IF NOT EXISTS payment_schema;

-- Create payment methods table
CREATE TABLE payment_schema.payment_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user_schema.users(id) ON DELETE CASCADE,
    payment_type VARCHAR(20) NOT NULL CHECK (payment_type IN ('card', 'bank_account', 'digital_wallet')),
    provider VARCHAR(50) NOT NULL,
    provider_payment_method_id VARCHAR(255) NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, provider_payment_method_id)
);

-- Create payment intents table
CREATE TABLE payment_schema.payment_intents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user_schema.users(id) ON DELETE CASCADE,
    rental_id UUID REFERENCES rental_schema.rentals(id) ON DELETE SET NULL,
    subscription_id UUID REFERENCES subscription_schema.subscriptions(id) ON DELETE SET NULL,
    amount_cents INTEGER NOT NULL CHECK (amount_cents > 0),
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'succeeded', 'failed', 'canceled')),
    payment_method_id UUID REFERENCES payment_schema.payment_methods(id) ON DELETE SET NULL,
    provider_payment_intent_id VARCHAR(255),
    provider_charge_id VARCHAR(255),
    failure_reason TEXT,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create payment transactions table
CREATE TABLE payment_schema.payment_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_intent_id UUID NOT NULL REFERENCES payment_schema.payment_intents(id) ON DELETE CASCADE,
    transaction_type VARCHAR(20) NOT NULL CHECK (transaction_type IN ('charge', 'refund', 'dispute')),
    amount_cents INTEGER NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'processing', 'succeeded', 'failed')),
    provider_transaction_id VARCHAR(255),
    failure_reason TEXT,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_payment_methods_user_id ON payment_schema.payment_methods(user_id);
CREATE INDEX idx_payment_methods_is_default ON payment_schema.payment_methods(is_default);
CREATE INDEX idx_payment_intents_user_id ON payment_schema.payment_intents(user_id);
CREATE INDEX idx_payment_intents_rental_id ON payment_schema.payment_intents(rental_id);
CREATE INDEX idx_payment_intents_subscription_id ON payment_schema.payment_intents(subscription_id);
CREATE INDEX idx_payment_intents_status ON payment_schema.payment_intents(status);
CREATE INDEX idx_payment_intents_provider_payment_intent_id ON payment_schema.payment_intents(provider_payment_intent_id);
CREATE INDEX idx_payment_transactions_payment_intent_id ON payment_schema.payment_transactions(payment_intent_id);
CREATE INDEX idx_payment_transactions_status ON payment_schema.payment_transactions(status);

-- Create audit triggers
CREATE TRIGGER audit_payment_methods_trigger
    AFTER INSERT OR UPDATE OR DELETE ON payment_schema.payment_methods
    FOR EACH ROW EXECUTE FUNCTION audit_schema.audit_trigger();

CREATE TRIGGER audit_payment_intents_trigger
    AFTER INSERT OR UPDATE OR DELETE ON payment_schema.payment_intents
    FOR EACH ROW EXECUTE FUNCTION audit_schema.audit_trigger();

CREATE TRIGGER audit_payment_transactions_trigger
    AFTER INSERT OR UPDATE OR DELETE ON payment_schema.payment_transactions
    FOR EACH ROW EXECUTE FUNCTION audit_schema.audit_trigger();
