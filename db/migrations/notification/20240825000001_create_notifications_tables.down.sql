-- Rollback: create_notifications_tables
-- Service: notification

BEGIN;

DROP TABLE IF EXISTS notification_schema.notifications;
DROP TABLE IF EXISTS notification_schema.user_settings;
DROP TABLE IF EXISTS notification_schema.templates;
DROP TABLE IF EXISTS notification_schema.channels;
DROP SCHEMA IF EXISTS notification_schema;

COMMIT;

