-- ============================================================================
-- File Number: 100500
-- File Name: create_user_role.sql
-- Module: User-Role Association
-- Description: Create user_roles table, indexes, and comments. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE TABLE user_roles (
    id BIGSERIAL PRIMARY KEY, -- Unique association ID
    user_id BIGINT NOT NULL, -- User ID
    role_id BIGINT NOT NULL, -- Role ID
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Association creation timestamp
    UNIQUE(user_id, role_id)
);

-- Indexes for user_roles table
CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_userid_roleid ON user_roles(user_id, role_id);

-- Comments for user_roles table
COMMENT ON TABLE user_roles IS 'User-role association table: maps users to roles';
