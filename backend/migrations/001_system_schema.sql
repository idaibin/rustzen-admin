-- ============================================================================
-- System Database Schema Definition
-- This file contains only core table structures, indexes, constraints, and functions
-- Log-related functionality is handled separately in 003_log_system.sql
-- ============================================================================

-- ============================================================================
-- 1. Users Table: Stores user account information
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

-- Indexes for users table
CREATE UNIQUE INDEX idx_users_username ON users(username) WHERE deleted_at IS NULL; -- Unique username for active users
CREATE UNIQUE INDEX idx_users_email ON users(email) WHERE deleted_at IS NULL; -- Unique email for active users
CREATE INDEX idx_users_deleted_at ON users(deleted_at); -- Index for soft delete

-- Comments for users table and columns
COMMENT ON TABLE users IS 'Users table: stores user account information';
COMMENT ON COLUMN users.username IS 'Unique username';
COMMENT ON COLUMN users.email IS 'Unique email address';
COMMENT ON COLUMN users.real_name IS 'Real name';
COMMENT ON COLUMN users.status IS 'User status: 1=active, 2=disabled, 3=pending';
COMMENT ON COLUMN users.deleted_at IS 'Soft delete timestamp, NULL means not deleted';
COMMENT ON COLUMN users.is_super_admin IS 'Super administrator flag: TRUE for super admin users';

-- ============================================================================
-- 2. Roles Table: Stores user roles
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

-- Indexes for roles table
CREATE UNIQUE INDEX idx_roles_name ON roles(role_name) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_roles_code ON roles(role_code) WHERE deleted_at IS NULL;
CREATE INDEX idx_roles_status ON roles(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_roles_deleted_at ON roles(deleted_at);
CREATE INDEX idx_roles_sort_order ON roles(sort_order) WHERE deleted_at IS NULL;

-- Comments for roles table and columns
COMMENT ON TABLE roles IS 'Roles table: stores user roles';
COMMENT ON COLUMN roles.role_name IS 'Role display name';
COMMENT ON COLUMN roles.role_code IS 'Role code for programmatic access';
COMMENT ON COLUMN roles.description IS 'Role description';
COMMENT ON COLUMN roles.status IS 'Role status: 1=active, 2=disabled';
COMMENT ON COLUMN roles.is_system IS 'System built-in role flag';
COMMENT ON COLUMN roles.sort_order IS 'Sort order for display';
COMMENT ON COLUMN roles.deleted_at IS 'Soft delete timestamp, NULL means not deleted';

-- ============================================================================
-- 3. User-Role Association Table: Maps users to roles (many-to-many)
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

-- ============================================================================
-- 4. Menus Table: Stores menu and permission definitions
-- ============================================================================
CREATE TABLE menus (
    id BIGSERIAL PRIMARY KEY, -- Unique menu ID
    parent_id BIGINT DEFAULT 0, -- Parent menu ID (0 for root)
    title VARCHAR(100) NOT NULL, -- Menu title
    path VARCHAR(255), -- Route path
    component VARCHAR(100), -- Frontend component name
    icon VARCHAR(100), -- Menu icon
    sort_order INTEGER DEFAULT 0, -- Sort order
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2)), -- 1: visible, 2: hidden
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Creation timestamp
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Last update timestamp
    deleted_at TIMESTAMP, -- Soft delete timestamp
    menu_type SMALLINT DEFAULT 2 CHECK (menu_type IN (1, 2, 3)), -- 1=directory, 2=menu, 3=button
    is_system BOOLEAN DEFAULT FALSE, -- System built-in menu flag
    meta_data JSONB, -- Extended metadata
    permission_code VARCHAR(100) UNIQUE -- Unique permission code for menu/button
);

-- Indexes for menus table
CREATE INDEX idx_menus_parent_id ON menus(parent_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_sort_order ON menus(sort_order) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_status ON menus(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_deleted_at ON menus(deleted_at);
CREATE INDEX IF NOT EXISTS idx_menus_parent_sort ON menus(parent_id, sort_order) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_menus_menu_type ON menus(menu_type) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_menus_is_system ON menus(is_system) WHERE is_system = TRUE AND deleted_at IS NULL;

-- Comments for menus table and columns
COMMENT ON TABLE menus IS 'Menus table: stores menu and permission definitions';
COMMENT ON COLUMN menus.title IS 'Menu title';
COMMENT ON COLUMN menus.path IS 'Route path';
COMMENT ON COLUMN menus.component IS 'Frontend component name';
COMMENT ON COLUMN menus.status IS 'Menu status: 1=visible, 2=hidden';
COMMENT ON COLUMN menus.deleted_at IS 'Soft delete timestamp, NULL means not deleted';
COMMENT ON COLUMN menus.menu_type IS 'Menu type: 1=directory, 2=menu, 3=button';
COMMENT ON COLUMN menus.is_system IS 'System built-in menu flag';
COMMENT ON COLUMN menus.meta_data IS 'Extended metadata in JSON format';

-- ============================================================================
-- 5. Role-Menu Association Table: Maps roles to menus (permissions)
-- ============================================================================
CREATE TABLE role_menus (
    id BIGSERIAL PRIMARY KEY, -- Unique association ID
    role_id BIGINT NOT NULL, -- Role ID
    menu_id BIGINT NOT NULL, -- Menu ID
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Association creation timestamp
    UNIQUE(role_id, menu_id)
);

-- Indexes for role_menus table
CREATE INDEX idx_role_menus_role_id ON role_menus(role_id);
CREATE INDEX idx_role_menus_menu_id ON role_menus(menu_id);

-- Comments for role_menus table
COMMENT ON TABLE role_menus IS 'Role-menu association table: maps roles to menus (permissions)';

-- ============================================================================
-- 6. Foreign Key Constraints
-- ============================================================================
-- Foreign keys for user_roles
ALTER TABLE user_roles
ADD CONSTRAINT fk_user_roles_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE user_roles
ADD CONSTRAINT fk_user_roles_role_id FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE;
-- Foreign keys for role_menus
ALTER TABLE role_menus
ADD CONSTRAINT fk_role_menus_role_id FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE;
ALTER TABLE role_menus
ADD CONSTRAINT fk_role_menus_menu_id FOREIGN KEY (menu_id) REFERENCES menus(id) ON DELETE CASCADE;

-- ============================================================================
-- 8. Performance Views and Triggers
-- ============================================================================
-- User permissions view: provides all permissions for each user
CREATE OR REPLACE VIEW user_permissions AS
SELECT DISTINCT 
    u.id as user_id,
    u.username,
    m.permission_code,
    m.menu_type,
    r.role_code
FROM users u
JOIN user_roles ur ON u.id = ur.user_id
JOIN roles r ON ur.role_id = r.id AND r.status = 1 AND r.deleted_at IS NULL
JOIN role_menus rm ON r.id = rm.role_id  
JOIN menus m ON rm.menu_id = m.id AND m.status = 1 AND m.deleted_at IS NULL
WHERE u.deleted_at IS NULL 
  AND u.status = 1
  AND m.permission_code IS NOT NULL;
-- User menu tree view: provides hierarchical menu structure for each user
CREATE OR REPLACE VIEW user_menu_tree AS
WITH RECURSIVE menu_tree AS (
    SELECT 
        m.id, m.parent_id, m.title, m.path, m.component, 
        m.icon, m.sort_order, m.menu_type, m.permission_code,
        u.id as user_id, 0 as level,
        ARRAY[m.sort_order, m.id] as sort_path
    FROM menus m
    JOIN role_menus rm ON m.id = rm.menu_id
    JOIN user_roles ur ON rm.role_id = ur.role_id
    JOIN users u ON ur.user_id = u.id
    WHERE m.parent_id = 0 
      AND m.status = 1 AND m.deleted_at IS NULL
      AND u.deleted_at IS NULL AND u.status = 1
    UNION ALL
    SELECT 
        m.id, m.parent_id, m.title, m.path, m.component,
        m.icon, m.sort_order, m.menu_type, m.permission_code,
        mt.user_id, mt.level + 1,
        mt.sort_path || ARRAY[m.sort_order, m.id]
    FROM menus m
    JOIN menu_tree mt ON m.parent_id = mt.id
    JOIN role_menus rm ON m.id = rm.menu_id
    JOIN user_roles ur ON rm.role_id = ur.role_id AND ur.user_id = mt.user_id
    WHERE m.status = 1 AND m.deleted_at IS NULL
)
SELECT * FROM menu_tree ORDER BY user_id, sort_path;

-- Triggers: automatically update updated_at on row update
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_roles_updated_at BEFORE UPDATE ON roles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_menus_updated_at BEFORE UPDATE ON menus
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();


