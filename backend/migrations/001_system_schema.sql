-- ============================================================================
-- 1. 用户表 (users) 
-- ============================================================================
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    real_name VARCHAR(50),
    avatar_url VARCHAR(255),
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2)), -- 1:正常 2:禁用
    last_login_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP -- 软删除字段
);

-- 用户表索引
CREATE UNIQUE INDEX idx_users_username ON users(username) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_users_email ON users(email) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_status ON users(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_deleted_at ON users(deleted_at);

-- 用户表注释
COMMENT ON TABLE users IS '用户表';
COMMENT ON COLUMN users.username IS '用户名';
COMMENT ON COLUMN users.email IS '邮箱';
COMMENT ON COLUMN users.real_name IS '真实姓名';
COMMENT ON COLUMN users.status IS '状态 1:正常 2:禁用';
COMMENT ON COLUMN users.deleted_at IS '删除时间，NULL表示未删除';

-- ============================================================================
-- 2. 角色表 (roles) 
-- ============================================================================
CREATE TABLE roles (
    id BIGSERIAL PRIMARY KEY,
    role_name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2)), -- 1:正常 2:禁用
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP -- 软删除字段
);

-- 角色表索引
CREATE UNIQUE INDEX idx_roles_name ON roles(role_name) WHERE deleted_at IS NULL;
CREATE INDEX idx_roles_status ON roles(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_roles_deleted_at ON roles(deleted_at);

-- 角色表注释
COMMENT ON TABLE roles IS '角色表';
COMMENT ON COLUMN roles.role_name IS '角色名称';
COMMENT ON COLUMN roles.description IS '角色描述';
COMMENT ON COLUMN roles.status IS '状态 1:正常 2:禁用';
COMMENT ON COLUMN roles.deleted_at IS '删除时间，NULL表示未删除';

-- ============================================================================
-- 3. 用户角色关联表 (user_roles)
-- ============================================================================
CREATE TABLE user_roles (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, role_id)
);

-- 用户角色关联表索引
CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);

-- 用户角色关联表注释
COMMENT ON TABLE user_roles IS '用户角色关联表';

-- ============================================================================
-- 4. 菜单表 (menus) 
-- ============================================================================
CREATE TABLE menus (
    id BIGSERIAL PRIMARY KEY,
    parent_id BIGINT DEFAULT 0,
    title VARCHAR(100) NOT NULL,
    path VARCHAR(255),
    component VARCHAR(100),
    icon VARCHAR(100),
    sort_order INTEGER DEFAULT 0,
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2)), -- 1:显示 2:隐藏
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP -- 软删除字段
);

-- 菜单表索引
CREATE INDEX idx_menus_parent_id ON menus(parent_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_sort_order ON menus(sort_order) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_status ON menus(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_deleted_at ON menus(deleted_at);

-- 菜单表注释
COMMENT ON TABLE menus IS '菜单表';
COMMENT ON COLUMN menus.title IS '菜单标题';
COMMENT ON COLUMN menus.path IS '菜单路径';
COMMENT ON COLUMN menus.component IS '组件名称';
COMMENT ON COLUMN menus.status IS '状态 1:显示 2:隐藏';
COMMENT ON COLUMN menus.deleted_at IS '删除时间，NULL表示未删除';

-- ============================================================================
-- 5. 角色菜单关联表 (role_menus) - 这就是权限控制
-- ============================================================================
CREATE TABLE role_menus (
    id BIGSERIAL PRIMARY KEY,
    role_id BIGINT NOT NULL,
    menu_id BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(role_id, menu_id)
);

-- 角色菜单关联表索引
CREATE INDEX idx_role_menus_role_id ON role_menus(role_id);
CREATE INDEX idx_role_menus_menu_id ON role_menus(menu_id);

-- 角色菜单关联表注释
COMMENT ON TABLE role_menus IS '角色菜单关联表 - 权限控制';

-- ============================================================================
-- 6. 操作日志表 (operation_logs) 
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

-- 操作日志表索引
CREATE INDEX idx_operation_logs_user_id ON operation_logs(user_id);
CREATE INDEX idx_operation_logs_created_at ON operation_logs(created_at);
CREATE INDEX idx_operation_logs_action ON operation_logs(action);

-- 操作日志表注释
COMMENT ON TABLE operation_logs IS '操作日志表';
COMMENT ON COLUMN operation_logs.action IS '操作类型';
COMMENT ON COLUMN operation_logs.description IS '操作描述';

-- ============================================================================
-- 7. 外键约束
-- ============================================================================

-- 用户角色关联表外键
ALTER TABLE user_roles
ADD CONSTRAINT fk_user_roles_user_id
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE user_roles
ADD CONSTRAINT fk_user_roles_role_id
FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE;

-- 角色菜单关联表外键
ALTER TABLE role_menus
ADD CONSTRAINT fk_role_menus_role_id
FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE;

ALTER TABLE role_menus
ADD CONSTRAINT fk_role_menus_menu_id
FOREIGN KEY (menu_id) REFERENCES menus(id) ON DELETE CASCADE;

-- 操作日志表外键（软关联）
ALTER TABLE operation_logs
ADD CONSTRAINT fk_operation_logs_user_id
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL; 