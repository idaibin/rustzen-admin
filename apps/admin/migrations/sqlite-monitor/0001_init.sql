-- Rustzen Admin Access + Monitor Host final SQLite schema.
-- Fresh initialization only; optional Admin owners are intentionally absent.

CREATE TABLE rustzen_installation_identity (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    build_id TEXT NOT NULL,
    composition_id TEXT NOT NULL,
    schema_fingerprint TEXT NOT NULL,
    data_contract_id TEXT NOT NULL
);




CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    email TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    real_name TEXT,
    avatar_url TEXT,
    status INTEGER NOT NULL DEFAULT 1 CHECK (status IN (1, 2, 3, 4)),
    is_system INTEGER NOT NULL DEFAULT 0,
    last_login_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME
);

CREATE UNIQUE INDEX idx_users_username ON users(username) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_users_email ON users(email) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_deleted_at ON users(deleted_at);

CREATE TABLE roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    description TEXT,
    status INTEGER NOT NULL DEFAULT 1 CHECK (status IN (1, 2)),
    is_system INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME
);

CREATE UNIQUE INDEX idx_roles_name ON roles(name) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_roles_code ON roles(code) WHERE deleted_at IS NULL;
CREATE INDEX idx_roles_status ON roles(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_roles_deleted_at ON roles(deleted_at);
CREATE INDEX idx_roles_sort_order ON roles(sort_order) WHERE deleted_at IS NULL;

CREATE TABLE modules (
    id TEXT PRIMARY KEY CHECK (id = 'monitor'),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1))
);

CREATE TABLE module_navigation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id TEXT NOT NULL REFERENCES modules(id),
    module_menu_code TEXT NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    icon TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    status INTEGER NOT NULL DEFAULT 1 CHECK (status IN (1, 2)),
    is_manual INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(module_id, module_menu_code)
);

CREATE TABLE menus (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER NOT NULL DEFAULT 0,
    parent_code TEXT,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    menu_type INTEGER NOT NULL DEFAULT 2 CHECK (menu_type IN (1, 2, 3)),
    status INTEGER NOT NULL DEFAULT 1 CHECK (status IN (1, 2)),
    is_system INTEGER NOT NULL DEFAULT 0,
    is_manual INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    path TEXT,
    icon TEXT,
    module_id TEXT REFERENCES modules(id),
    module_menu_code TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME
);

CREATE UNIQUE INDEX idx_menus_name
    ON menus(COALESCE(module_id, ''), name)
    WHERE deleted_at IS NULL AND is_active = 1;
CREATE UNIQUE INDEX idx_menus_code ON menus(code) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_menus_module_menu_code
    ON menus(module_id, module_menu_code)
    WHERE module_id IS NOT NULL
      AND module_menu_code IS NOT NULL
      AND is_active = 1
      AND deleted_at IS NULL;
CREATE INDEX idx_resources_parent_id ON menus(parent_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_sort_order ON menus(sort_order) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_status ON menus(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_deleted_at ON menus(deleted_at);
CREATE INDEX idx_resources_parent_sort ON menus(parent_id, sort_order) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_menu_type ON menus(menu_type) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_is_system ON menus(is_system) WHERE is_system = 1 AND deleted_at IS NULL;
CREATE INDEX idx_resources_parent_code ON menus(parent_code) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_module_active
    ON menus(module_id, is_active)
    WHERE module_id IS NOT NULL AND deleted_at IS NULL;

CREATE TABLE user_roles (
    user_id INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);

CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);

CREATE TABLE role_menus (
    role_id INTEGER NOT NULL,
    menu_id INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(role_id, menu_id),
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (menu_id) REFERENCES menus(id) ON DELETE CASCADE
);

CREATE INDEX idx_role_menus_role_id ON role_menus(role_id);
CREATE INDEX idx_role_menus_menu_id ON role_menus(menu_id);

INSERT INTO modules (id, enabled)
VALUES ('monitor', 1);

-- The only initial account is disabled and has a deliberately non-PHC hash.
-- `rz-admin bootstrap-owner` replaces it with the operator secret and enables it.
INSERT INTO users (username, email, password_hash, real_name, status, is_system)
VALUES ('owner', 'owner@example.com', '!bootstrap-required!', '所有者', 2, 1);

INSERT INTO roles (name, code, description, status, is_system, sort_order)
VALUES
    ('所有者', 'owner', '内置所有者角色，拥有全部权限。', 1, 1, 1),
    ('管理员', 'admin', '内置管理员角色，拥有日常管理权限。', 1, 1, 2),
    ('查看者', 'viewer', '内置查看者角色，仅拥有只读权限。', 1, 1, 3);

INSERT INTO menus (parent_id, name, code, menu_type, sort_order, status, is_system, is_manual)
VALUES (0, '全部权限', '*', 1, 1, 1, 1, 0);

INSERT INTO role_menus (role_id, menu_id, created_at)
SELECT r.id, m.id, CURRENT_TIMESTAMP
FROM roles r
CROSS JOIN menus m
WHERE r.code = 'owner' AND m.code = '*';

INSERT INTO user_roles (user_id, role_id, created_at)
SELECT u.id, r.id, CURRENT_TIMESTAMP
FROM users u
INNER JOIN roles r ON r.code = u.username
WHERE u.username = 'owner';

CREATE VIEW user_with_roles AS
SELECT
    u.id AS id,
    u.username,
    u.email,
    u.real_name,
    u.password_hash,
    u.avatar_url,
    u.status,
    u.is_system,
    u.last_login_at,
    u.created_at,
    u.updated_at,
    COALESCE(
        (
            SELECT json_group_array(json_object('label', ro.name, 'value', ro.id))
            FROM (
                SELECT r.name, r.id
                FROM user_roles ur
                INNER JOIN roles r ON ur.role_id = r.id AND r.deleted_at IS NULL
                WHERE ur.user_id = u.id
                ORDER BY r.id
            ) ro
        ),
        '[]'
    ) AS roles
FROM users u
WHERE u.deleted_at IS NULL;

CREATE VIEW user_permissions AS
SELECT DISTINCT
    u.id AS user_id,
    u.username,
    m.code AS menu_code,
    m.menu_type,
    r.code AS role_code,
    m.id AS menu_id,
    r.id AS role_id
FROM users u
INNER JOIN user_roles ur ON u.id = ur.user_id
INNER JOIN roles r ON ur.role_id = r.id AND r.status = 1 AND r.deleted_at IS NULL
INNER JOIN role_menus rm ON r.id = rm.role_id
INNER JOIN menus m ON rm.menu_id = m.id AND m.deleted_at IS NULL
WHERE u.deleted_at IS NULL
  AND u.status = 1
  AND m.is_active = 1
  AND m.code IS NOT NULL;

CREATE VIEW role_with_menus AS
SELECT
    r.id AS id,
    r.name,
    r.code,
    r.description,
    r.status,
    r.created_at,
    r.updated_at,
    r.deleted_at,
    r.is_system,
    COALESCE(
        (
            SELECT json_group_array(json_object('label', mo.name, 'value', mo.id))
            FROM (
                SELECT m.name, m.id
                FROM role_menus rm
                INNER JOIN menus m ON rm.menu_id = m.id
                    AND m.deleted_at IS NULL
                    AND m.is_active = 1
                WHERE rm.role_id = r.id
                ORDER BY m.id
            ) mo
        ),
        '[]'
    ) AS menus
FROM roles r
WHERE r.deleted_at IS NULL;
