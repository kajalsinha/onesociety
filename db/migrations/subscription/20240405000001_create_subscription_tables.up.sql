-- Create subscription schema
CREATE SCHEMA IF NOT EXISTS subscription_schema;

-- Create subscription plans table
CREATE TABLE subscription_schema.subscription_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    price_cents INTEGER NOT NULL CHECK (price_cents >= 0),
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    billing_cycle VARCHAR(20) NOT NULL CHECK (billing_cycle IN ('monthly', 'yearly')),
    features JSONB NOT NULL DEFAULT '[]',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    max_listings INTEGER,
    max_rentals_per_month INTEGER,
    priority_support BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create subscriptions table
CREATE TABLE subscription_schema.subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user_schema.users(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES subscription_schema.subscription_plans(id) ON DELETE RESTRICT,
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'canceled', 'past_due', 'unpaid')),
    current_period_start TIMESTAMP WITH TIME ZONE NOT NULL,
    current_period_end TIMESTAMP WITH TIME ZONE NOT NULL,
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT FALSE,
    canceled_at TIMESTAMP WITH TIME ZONE,
    provider_subscription_id VARCHAR(255),
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create subscription usage table
CREATE TABLE subscription_schema.subscription_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES subscription_schema.subscriptions(id) ON DELETE CASCADE,
    usage_type VARCHAR(50) NOT NULL,
    usage_count INTEGER NOT NULL DEFAULT 0,
    usage_limit INTEGER,
    period_start TIMESTAMP WITH TIME ZONE NOT NULL,
    period_end TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_subscription_plans_is_active ON subscription_schema.subscription_plans(is_active);
CREATE INDEX idx_subscriptions_user_id ON subscription_schema.subscriptions(user_id);
CREATE INDEX idx_subscriptions_status ON subscription_schema.subscriptions(status);
CREATE INDEX idx_subscriptions_current_period_end ON subscription_schema.subscriptions(current_period_end);
CREATE INDEX idx_subscription_usage_subscription_id ON subscription_schema.subscription_usage(subscription_id);
CREATE INDEX idx_subscription_usage_period ON subscription_schema.subscription_usage(period_start, period_end);

-- Create audit triggers
CREATE TRIGGER audit_subscription_plans_trigger
    AFTER INSERT OR UPDATE OR DELETE ON subscription_schema.subscription_plans
    FOR EACH ROW EXECUTE FUNCTION audit_schema.audit_trigger();

CREATE TRIGGER audit_subscriptions_trigger
    AFTER INSERT OR UPDATE OR DELETE ON subscription_schema.subscriptions
    FOR EACH ROW EXECUTE FUNCTION audit_schema.audit_trigger();

CREATE TRIGGER audit_subscription_usage_trigger
    AFTER INSERT OR UPDATE OR DELETE ON subscription_schema.subscription_usage
    FOR EACH ROW EXECUTE FUNCTION audit_schema.audit_trigger();
