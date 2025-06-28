-- ============================================================================
-- 1. Users Table
-- ============================================================================
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    real_name VARCHAR(50),
    avatar_url VARCHAR(255),
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2, 3, 4, 5)), -- 1: active 2: disabled 3: pending 4: locked 5: deleted
    last_login_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP -- Soft delete field
);

-- User table indexes
CREATE UNIQUE INDEX idx_users_username ON users(username) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_users_email ON users(email) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_status ON users(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_deleted_at ON users(deleted_at);

-- User table comments
COMMENT ON TABLE users IS 'Users table';
COMMENT ON COLUMN users.username IS 'Username';
COMMENT ON COLUMN users.email IS 'Email';
COMMENT ON COLUMN users.real_name IS 'Real name';
COMMENT ON COLUMN users.status IS 'Status 1: active 2: disabled 3: pending 4: locked 5: deleted';
COMMENT ON COLUMN users.deleted_at IS 'Deletion time, NULL means not deleted';

-- ============================================================================
-- 2. Roles Table
-- ============================================================================
CREATE TABLE roles (
    id BIGSERIAL PRIMARY KEY,
    role_name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2)), -- 1: active 2: disabled
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP -- Soft delete field
);

-- Role table indexes
CREATE UNIQUE INDEX idx_roles_name ON roles(role_name) WHERE deleted_at IS NULL;
CREATE INDEX idx_roles_status ON roles(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_roles_deleted_at ON roles(deleted_at);

-- Role table comments
COMMENT ON TABLE roles IS 'Roles table';
COMMENT ON COLUMN roles.role_name IS 'Role name';
COMMENT ON COLUMN roles.description IS 'Role description';
COMMENT ON COLUMN roles.status IS 'Status 1: active 2: disabled';
COMMENT ON COLUMN roles.deleted_at IS 'Deletion time, NULL means not deleted';

-- ============================================================================
-- 3. User-Role Association Table
-- ============================================================================
CREATE TABLE user_roles (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, role_id)
);

-- User-role association table indexes
CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);

-- User-role association table comments
COMMENT ON TABLE user_roles IS 'User-role association table';

-- ============================================================================
-- 4. Menus Table
-- ============================================================================
CREATE TABLE menus (
    id BIGSERIAL PRIMARY KEY,
    parent_id BIGINT DEFAULT 0,
    title VARCHAR(100) NOT NULL,
    path VARCHAR(255),
    component VARCHAR(100),
    icon VARCHAR(100),
    sort_order INTEGER DEFAULT 0,
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2)), -- 1: visible 2: hidden
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP -- Soft delete field
);

-- Menu table indexes
CREATE INDEX idx_menus_parent_id ON menus(parent_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_sort_order ON menus(sort_order) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_status ON menus(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_deleted_at ON menus(deleted_at);

-- Menu table comments
COMMENT ON TABLE menus IS 'Menus table';
COMMENT ON COLUMN menus.title IS 'Menu title';
COMMENT ON COLUMN menus.path IS 'Menu path';
COMMENT ON COLUMN menus.component IS 'Component name';
COMMENT ON COLUMN menus.status IS 'Status 1: visible 2: hidden';
COMMENT ON COLUMN menus.deleted_at IS 'Deletion time, NULL means not deleted';

-- ============================================================================
-- 5. Role-Menu Association Table (Permission Control)
-- ============================================================================
CREATE TABLE role_menus (
    id BIGSERIAL PRIMARY KEY,
    role_id BIGINT NOT NULL,
    menu_id BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(role_id, menu_id)
);

-- Role-menu association table indexes
CREATE INDEX idx_role_menus_role_id ON role_menus(role_id);
CREATE INDEX idx_role_menus_menu_id ON role_menus(menu_id);

-- Role-menu association table comments
COMMENT ON TABLE role_menus IS 'Role-menu association table - permission control';

-- ============================================================================
-- 6. Operation Logs Table
-- ============================================================================
CREATE TABLE operation_logs (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT,
    username VARCHAR(50),
    action VARCHAR(100) NOT NULL,
    description TEXT,
    ip_address INET,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Operation logs table indexes
CREATE INDEX idx_operation_logs_user_id ON operation_logs(user_id);
CREATE INDEX idx_operation_logs_created_at ON operation_logs(created_at);
CREATE INDEX idx_operation_logs_action ON operation_logs(action);

-- Operation logs table comments
COMMENT ON TABLE operation_logs IS 'Operation logs table';
COMMENT ON COLUMN operation_logs.action IS 'Operation type';
COMMENT ON COLUMN operation_logs.description IS 'Operation description';

-- ============================================================================
-- 7. Foreign Key Constraints
-- ============================================================================

-- User-role association table foreign keys
ALTER TABLE user_roles
ADD CONSTRAINT fk_user_roles_user_id
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE user_roles
ADD CONSTRAINT fk_user_roles_role_id
FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE;

-- Role-menu association table foreign keys
ALTER TABLE role_menus
ADD CONSTRAINT fk_role_menus_role_id
FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE;

ALTER TABLE role_menus
ADD CONSTRAINT fk_role_menus_menu_id
FOREIGN KEY (menu_id) REFERENCES menus(id) ON DELETE CASCADE;

-- Operation logs table foreign key (soft association)
ALTER TABLE operation_logs
ADD CONSTRAINT fk_operation_logs_user_id
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL;

-- ============================================================================
-- 8. Add permission_code field to menus table
-- ============================================================================
ALTER TABLE menus ADD COLUMN permission_code VARCHAR(100) UNIQUE; 

-- ============================================================================
-- 9. Initialize zen_admin permission, role, user, and associations
-- ============================================================================

DO $$
DECLARE
    zen_admin_menu_id BIGINT;
    zen_admin_role_id BIGINT;
    zen_admin_user_id BIGINT;
BEGIN
    -- Insert menu and get ID
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, permission_code)
    VALUES (0, 'Zen Admin Permission', NULL, NULL, NULL, 0, 2, 'zen_admin')
    RETURNING id INTO zen_admin_menu_id;
    
    -- Insert role and get ID
    INSERT INTO roles (role_name, description, status)
    VALUES ('ZenAdmin', 'Zen administrator with all permissions', 1)
    RETURNING id INTO zen_admin_role_id;
    
    -- Insert user and get ID
    INSERT INTO users (username, email, password_hash, real_name, status)
    VALUES ('zenadmin', 'zenadmin@example.com', '$argon2id$v=19$m=19456,t=2,p=1$4sTQ/mQzCFey6tdni97/fQ$k32+fZocOR47QMPWK8HUhAFpnY81sOZAsJUxoSLGWb4', 'Zen Administrator', 1)
    RETURNING id INTO zen_admin_user_id;
    
    -- Associate role and menu
    INSERT INTO role_menus (role_id, menu_id)
    VALUES (zen_admin_role_id, zen_admin_menu_id);
    
    -- Associate user and role
    INSERT INTO user_roles (user_id, role_id)
    VALUES (zen_admin_user_id, zen_admin_role_id);
END $$;


