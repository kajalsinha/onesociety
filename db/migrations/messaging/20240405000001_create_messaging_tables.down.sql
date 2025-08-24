-- Drop audit triggers
DROP TRIGGER IF EXISTS audit_messages_trigger ON messaging_schema.messages;
DROP TRIGGER IF EXISTS audit_conversations_trigger ON messaging_schema.conversations;

-- Drop indexes
DROP INDEX IF EXISTS idx_messages_is_read;
DROP INDEX IF EXISTS idx_messages_created_at;
DROP INDEX IF EXISTS idx_messages_sender_id;
DROP INDEX IF EXISTS idx_messages_conversation_id;
DROP INDEX IF EXISTS idx_conversations_status;
DROP INDEX IF EXISTS idx_conversations_renter_id;
DROP INDEX IF EXISTS idx_conversations_owner_id;
DROP INDEX IF EXISTS idx_conversations_rental_id;

-- Drop tables
DROP TABLE IF EXISTS messaging_schema.messages;
DROP TABLE IF EXISTS messaging_schema.conversations;

-- Drop schema
DROP SCHEMA IF EXISTS messaging_schema CASCADE;
