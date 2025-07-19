-- ============================================================================
-- File Number: 100200
-- File Name: create_role.sql
-- Module: Role Management
-- Description: Create roles table, indexes, and comments. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE TABLE roles (
    id BIGSERIAL PRIMARY KEY, -- Unique role ID
    role_name VARCHAR(50) UNIQUE NOT NULL, -- Role display name
    role_code VARCHAR(50) NOT NULL, -- Role code for programmatic access
    description TEXT, -- Role description
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2)), -- 1: active, 2: disabled
    is_system BOOLEAN DEFAULT FALSE, -- System built-in role flag
    sort_order INTEGER DEFAULT 0, -- Sort order for display
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Creation timestamp
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Last update timestamp
    deleted_at TIMESTAMP -- Soft delete timestamp
);

CREATE UNIQUE INDEX idx_roles_name ON roles(role_name) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_roles_code ON roles(role_code) WHERE deleted_at IS NULL;
CREATE INDEX idx_roles_status ON roles(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_roles_deleted_at ON roles(deleted_at);
CREATE INDEX idx_roles_sort_order ON roles(sort_order) WHERE deleted_at IS NULL;

COMMENT ON TABLE roles IS 'Roles table: stores user roles';
COMMENT ON COLUMN roles.role_name IS 'Role display name';
COMMENT ON COLUMN roles.role_code IS 'Role code for programmatic access';
COMMENT ON COLUMN roles.description IS 'Role description';
COMMENT ON COLUMN roles.status IS 'Role status: 1=active, 2=disabled';
COMMENT ON COLUMN roles.is_system IS 'System built-in role flag';
COMMENT ON COLUMN roles.sort_order IS 'Sort order for display';
COMMENT ON COLUMN roles.deleted_at IS 'Soft delete timestamp, NULL means not deleted';
