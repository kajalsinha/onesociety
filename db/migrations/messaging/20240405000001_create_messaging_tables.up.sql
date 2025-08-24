-- Create messaging schema
CREATE SCHEMA IF NOT EXISTS messaging_schema;

-- Create conversations table
CREATE TABLE messaging_schema.conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rental_id UUID NOT NULL REFERENCES rental_schema.rentals(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES user_schema.users(id) ON DELETE CASCADE,
    renter_id UUID NOT NULL REFERENCES user_schema.users(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived', 'blocked')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(rental_id, owner_id, renter_id)
);

-- Create messages table
CREATE TABLE messaging_schema.messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES messaging_schema.conversations(id) ON DELETE CASCADE,
    sender_id UUID NOT NULL REFERENCES user_schema.users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    message_type VARCHAR(20) NOT NULL DEFAULT 'text' CHECK (message_type IN ('text', 'image', 'file', 'system')),
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_conversations_rental_id ON messaging_schema.conversations(rental_id);
CREATE INDEX idx_conversations_owner_id ON messaging_schema.conversations(owner_id);
CREATE INDEX idx_conversations_renter_id ON messaging_schema.conversations(renter_id);
CREATE INDEX idx_conversations_status ON messaging_schema.conversations(status);
CREATE INDEX idx_messages_conversation_id ON messaging_schema.messages(conversation_id);
CREATE INDEX idx_messages_sender_id ON messaging_schema.messages(sender_id);
CREATE INDEX idx_messages_created_at ON messaging_schema.messages(created_at);
CREATE INDEX idx_messages_is_read ON messaging_schema.messages(is_read);

-- Create audit triggers
CREATE TRIGGER audit_conversations_trigger
    AFTER INSERT OR UPDATE OR DELETE ON messaging_schema.conversations
    FOR EACH ROW EXECUTE FUNCTION audit_schema.audit_trigger();

CREATE TRIGGER audit_messages_trigger
    AFTER INSERT OR UPDATE OR DELETE ON messaging_schema.messages
    FOR EACH ROW EXECUTE FUNCTION audit_schema.audit_trigger();
