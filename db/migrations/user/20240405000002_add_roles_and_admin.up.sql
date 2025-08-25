-- Migration: add_roles_and_admin
-- Service: user
-- Created at: 2025-08-25 00:00:00 UTC

BEGIN;

-- Roles table
CREATE TABLE IF NOT EXISTS user_schema.roles (
	role_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
	name VARCHAR(50) UNIQUE NOT NULL,
	description TEXT,
	created_at TIMESTAMPTZ DEFAULT NOW()
);

-- User roles join
CREATE TABLE IF NOT EXISTS user_schema.user_roles (
	user_id UUID NOT NULL REFERENCES user_schema.users(user_id) ON DELETE CASCADE,
	role_id UUID NOT NULL REFERENCES user_schema.roles(role_id) ON DELETE CASCADE,
	assigned_at TIMESTAMPTZ DEFAULT NOW(),
	PRIMARY KEY (user_id, role_id)
);

-- Basic permissions (coarse-grained for MVP)
CREATE TABLE IF NOT EXISTS user_schema.permissions (
	permission_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
	name VARCHAR(100) UNIQUE NOT NULL,
	description TEXT
);

CREATE TABLE IF NOT EXISTS user_schema.role_permissions (
	role_id UUID NOT NULL REFERENCES user_schema.roles(role_id) ON DELETE CASCADE,
	permission_id UUID NOT NULL REFERENCES user_schema.permissions(permission_id) ON DELETE CASCADE,
	PRIMARY KEY (role_id, permission_id)
);

-- Seed admin role and permissions
INSERT INTO user_schema.roles (name, description)
VALUES ('admin', 'Platform administrator with moderation capabilities')
ON CONFLICT (name) DO NOTHING;

INSERT INTO user_schema.permissions (name, description) VALUES
('user.moderate', 'Suspend/activate users'),
('product.moderate', 'Approve/reject products'),
('review.moderate', 'Remove inappropriate reviews')
ON CONFLICT (name) DO NOTHING;

-- Grant permissions to admin role
INSERT INTO user_schema.role_permissions (role_id, permission_id)
SELECT r.role_id, p.permission_id
FROM user_schema.roles r, user_schema.permissions p
WHERE r.name = 'admin' AND p.name IN ('user.moderate','product.moderate','review.moderate')
ON CONFLICT DO NOTHING;

-- Assign admin role to seeded admin@example.com if exists
INSERT INTO user_schema.user_roles (user_id, role_id)
SELECT u.user_id, r.role_id
FROM user_schema.users u CROSS JOIN user_schema.roles r
WHERE u.email = 'admin@example.com' AND r.name = 'admin'
ON CONFLICT DO NOTHING;

COMMIT;

