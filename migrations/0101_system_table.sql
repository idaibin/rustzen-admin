-- ============================================================================
-- Module: User Management
-- Description: Create users table, indexes, and comments. Zen migration style.
-- ============================================================================

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY, -- Unique user ID
    username VARCHAR(50) UNIQUE NOT NULL, -- Unique username
    email VARCHAR(100) UNIQUE NOT NULL, -- Unique email address
    password_hash VARCHAR(255) NOT NULL, -- Hashed password
    real_name VARCHAR(50), -- Real name
    avatar_url VARCHAR(255), -- Avatar image URL
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2, 3)), -- 1: active, 2: disabled, 3: pending
    is_system BOOLEAN DEFAULT FALSE, -- System built-in user flag
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
COMMENT ON COLUMN users.is_system IS 'System built-in user flag: TRUE for system built-in users';


-- ============================================================================
-- Module: Role Management
-- Description: Create roles table, indexes, and comments. Zen migration style.
-- ============================================================================

CREATE TABLE roles (
    id BIGSERIAL PRIMARY KEY, -- Unique role ID
    name VARCHAR(50) UNIQUE NOT NULL, -- Role display name
    code VARCHAR(50) UNIQUE NOT NULL, -- Role code for programmatic access
    description TEXT, -- Role description
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2)), -- 1: active, 2: disabled
    is_system BOOLEAN DEFAULT FALSE, -- System built-in role flag
    sort_order INTEGER DEFAULT 0, -- Sort order for display
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Creation timestamp
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Last update timestamp
    deleted_at TIMESTAMP -- Soft delete timestamp
);

CREATE UNIQUE INDEX idx_roles_name ON roles(name) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_roles_code ON roles(code) WHERE deleted_at IS NULL;
CREATE INDEX idx_roles_status ON roles(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_roles_deleted_at ON roles(deleted_at);
CREATE INDEX idx_roles_sort_order ON roles(sort_order) WHERE deleted_at IS NULL;

COMMENT ON TABLE roles IS 'Roles table: stores user roles';
COMMENT ON COLUMN roles.name IS 'Role display name';
COMMENT ON COLUMN roles.code IS 'Role code for programmatic access';
COMMENT ON COLUMN roles.description IS 'Role description';
COMMENT ON COLUMN roles.status IS 'Role status: 1=active, 2=disabled';
COMMENT ON COLUMN roles.is_system IS 'System built-in role flag';
COMMENT ON COLUMN roles.sort_order IS 'Sort order for display';
COMMENT ON COLUMN roles.deleted_at IS 'Soft delete timestamp, NULL means not deleted';


-- ============================================================================
-- Module: Resource Management
-- Description: Create menus table, indexes, and comments. Zen migration style.
-- ============================================================================

CREATE TABLE menus (
    id BIGSERIAL PRIMARY KEY, -- Unique menu ID
    parent_id BIGINT DEFAULT 0, -- Parent menu ID (0 for root)
    name VARCHAR(100) UNIQUE NOT NULL, -- Menu title
    code VARCHAR(100) UNIQUE NOT NULL, -- Unique permission code for menu/button
    menu_type SMALLINT DEFAULT 2 CHECK (menu_type IN (1, 2, 3)), -- 1=directory, 2=menu, 3=button
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2)), -- 1: visible, 2: hidden
    is_system BOOLEAN DEFAULT FALSE, -- System built-in resource flag
    sort_order INTEGER DEFAULT 0, -- Sort order
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Creation timestamp
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Last update timestamp
    deleted_at TIMESTAMP -- Soft delete timestamp
);

CREATE INDEX idx_resources_parent_id ON menus(parent_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_sort_order ON menus(sort_order) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_status ON menus(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_deleted_at ON menus(deleted_at);
CREATE INDEX idx_resources_parent_sort ON menus(parent_id, sort_order) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_menu_type ON menus(menu_type) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_is_system ON menus(is_system) WHERE is_system = TRUE AND deleted_at IS NULL;

COMMENT ON TABLE menus IS 'Resources table: stores resource definitions';
COMMENT ON COLUMN menus.name IS 'Resource name';
COMMENT ON COLUMN menus.code IS 'Resource code';
COMMENT ON COLUMN menus.menu_type IS 'Resource menu_type: 1=directory, 2=menu, 3=button';
COMMENT ON COLUMN menus.status IS 'Resource status: 1=visible, 2=hidden';
COMMENT ON COLUMN menus.is_system IS 'System built-in resource flag';
COMMENT ON COLUMN menus.deleted_at IS 'Soft delete timestamp, NULL means not deleted';


-- ============================================================================
-- Module: Dictionary
-- Description: Create dicts table, indexes, and comments. Zen migration style.
-- ============================================================================

CREATE TABLE dicts (
    id BIGSERIAL PRIMARY KEY, -- Unique dict entry ID
    dict_type VARCHAR(50) NOT NULL, -- Dictionary type/category
    label VARCHAR(100) NOT NULL, -- Dictionary label
    value VARCHAR(255) NOT NULL, -- Dictionary value
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2)), -- 1: active, 2: inactive
    description TEXT, -- Dictionary description
    sort_order INTEGER DEFAULT 0, -- Sort order
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Creation timestamp
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Last update timestamp
    deleted_at TIMESTAMP -- Soft delete timestamp
);

CREATE UNIQUE INDEX idx_dicts_label ON dicts(dict_type, label) WHERE deleted_at IS NULL;
CREATE INDEX idx_dicts_status ON dicts(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_dicts_deleted_at ON dicts(deleted_at);
CREATE INDEX idx_dicts_dict_type ON dicts(dict_type) WHERE deleted_at IS NULL;

COMMENT ON TABLE dicts IS 'Dictionary table for key-value pairs and types';
COMMENT ON COLUMN dicts.dict_type IS 'Dictionary type';
COMMENT ON COLUMN dicts.label IS 'Dictionary label';
COMMENT ON COLUMN dicts.value IS 'Dictionary value';
COMMENT ON COLUMN dicts.description IS 'Dictionary description';
COMMENT ON COLUMN dicts.status IS 'Dictionary entry status: 1=active, 2=inactive';
COMMENT ON COLUMN dicts.sort_order IS 'Sort order for display';
COMMENT ON COLUMN dicts.deleted_at IS 'Soft delete timestamp, NULL means not deleted';


-- ============================================================================
-- Module: Log Management
-- Description: Create operation_logs table (partitioned), indexes, and comments.
-- ============================================================================

CREATE TABLE operation_logs (
    id BIGSERIAL,
    user_id BIGINT,
    username VARCHAR(50),
    action VARCHAR(100) NOT NULL,
    description TEXT,
    data JSONB,
    status VARCHAR(20) DEFAULT 'SUCCESS',
    duration_ms INTEGER,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

COMMENT ON TABLE operation_logs IS 'Operation logs table (partitioned by month): stores operation logs for auditing';
COMMENT ON COLUMN operation_logs.user_id IS 'User ID';
COMMENT ON COLUMN operation_logs.username IS 'Username';
COMMENT ON COLUMN operation_logs.action IS 'Request action';
COMMENT ON COLUMN operation_logs.description IS 'Request description';
COMMENT ON COLUMN operation_logs.data IS 'Request data';
COMMENT ON COLUMN operation_logs.status IS 'Request status';
COMMENT ON COLUMN operation_logs.duration_ms IS 'Request duration in milliseconds';
COMMENT ON COLUMN operation_logs.ip_address IS 'Request IP address';
COMMENT ON COLUMN operation_logs.user_agent IS 'Request user agent string';

