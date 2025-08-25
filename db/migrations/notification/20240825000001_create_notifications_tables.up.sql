-- Migration: create_notifications_tables
-- Service: notification
-- Created at: 2025-08-25 00:00:00 UTC

BEGIN;

-- Create schema if missing
CREATE SCHEMA IF NOT EXISTS notification_schema;

-- Notification channels and templates (for extensibility)
CREATE TABLE IF NOT EXISTS notification_schema.channels (
	channel_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
	name VARCHAR(50) UNIQUE NOT NULL,
	description TEXT
);

CREATE TABLE IF NOT EXISTS notification_schema.templates (
	template_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
	name VARCHAR(100) UNIQUE NOT NULL,
	channel VARCHAR(50) NOT NULL,
	subject TEXT,
	body TEXT,
	metadata JSONB
);

-- User notification settings
CREATE TABLE IF NOT EXISTS notification_schema.user_settings (
	user_id UUID PRIMARY KEY REFERENCES user_schema.users(user_id) ON DELETE CASCADE,
	preferences JSONB NOT NULL DEFAULT '{}'::jsonb,
	created_at TIMESTAMPTZ DEFAULT NOW(),
	updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Notifications
CREATE TABLE IF NOT EXISTS notification_schema.notifications (
	notification_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
	user_id UUID NOT NULL REFERENCES user_schema.users(user_id) ON DELETE CASCADE,
	title TEXT NOT NULL,
	message TEXT NOT NULL,
	category VARCHAR(50),
	data JSONB,
	is_read BOOLEAN DEFAULT false,
	sent_via JSONB, -- record delivery channels and statuses
	created_at TIMESTAMPTZ DEFAULT NOW(),
	read_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_notifications_user_id ON notification_schema.notifications(user_id);
CREATE INDEX IF NOT EXISTS idx_notifications_is_read ON notification_schema.notifications(is_read);

COMMIT;

