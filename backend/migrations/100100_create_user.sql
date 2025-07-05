-- ============================================================================
-- File Number: 100100
-- File Name: create_user.sql
-- Module: User Management
-- Description: Create users table, indexes, and comments. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY, -- Unique user ID
    username VARCHAR(50) UNIQUE NOT NULL, -- Unique username
    email VARCHAR(100) UNIQUE NOT NULL, -- Unique email address
    password_hash VARCHAR(255) NOT NULL, -- Hashed password
    real_name VARCHAR(50), -- Real name
    avatar_url VARCHAR(255), -- Avatar image URL
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2, 3)), -- 1: active, 2: disabled, 3: pending
    is_super_admin BOOLEAN DEFAULT FALSE, -- Super administrator flag
    last_login_at TIMESTAMP, -- Last login timestamp
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Creation timestamp
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Last update timestamp
    deleted_at TIMESTAMP -- Soft delete timestamp (NULL means not deleted)
);

CREATE UNIQUE INDEX idx_users_username ON users(username) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_users_email ON users(email) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_deleted_at ON users(deleted_at);

COMMENT ON TABLE users IS 'Users table: stores user account information';
COMMENT ON COLUMN users.username IS 'Unique username';
COMMENT ON COLUMN users.email IS 'Unique email address';
COMMENT ON COLUMN users.real_name IS 'Real name';
COMMENT ON COLUMN users.status IS 'User status: 1=active, 2=disabled, 3=pending';
COMMENT ON COLUMN users.deleted_at IS 'Soft delete timestamp, NULL means not deleted';
COMMENT ON COLUMN users.is_super_admin IS 'Super administrator flag: TRUE for super admin users';
