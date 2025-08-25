-- Rollback: add_roles_and_admin
-- Service: user

BEGIN;

DROP TABLE IF EXISTS user_schema.role_permissions;
DROP TABLE IF EXISTS user_schema.permissions;
DROP TABLE IF EXISTS user_schema.user_roles;
DROP TABLE IF EXISTS user_schema.roles;

COMMIT;

