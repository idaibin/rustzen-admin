-- ============================================================================
-- Module: Core tables and indexes.
-- SQLite first-phase storage initialization for the sqlite-first runtime.
-- ============================================================================

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
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME
);

CREATE UNIQUE INDEX idx_menus_name ON menus(name) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_menus_code ON menus(code) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_parent_id ON menus(parent_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_sort_order ON menus(sort_order) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_status ON menus(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_deleted_at ON menus(deleted_at);
CREATE INDEX idx_resources_parent_sort ON menus(parent_id, sort_order) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_menu_type ON menus(menu_type) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_is_system ON menus(is_system) WHERE is_system = 1 AND deleted_at IS NULL;
CREATE INDEX idx_resources_parent_code ON menus(parent_code) WHERE deleted_at IS NULL;

CREATE TABLE dicts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    dict_type TEXT NOT NULL,
    label TEXT NOT NULL,
    value TEXT NOT NULL,
    status INTEGER NOT NULL DEFAULT 1 CHECK (status IN (1, 2)),
    description TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME
);

CREATE UNIQUE INDEX idx_dicts_label ON dicts(dict_type, label) WHERE deleted_at IS NULL;
CREATE INDEX idx_dicts_status ON dicts(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_dicts_deleted_at ON dicts(deleted_at);
CREATE INDEX idx_dicts_dict_type ON dicts(dict_type) WHERE deleted_at IS NULL;

CREATE TABLE operation_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    username TEXT,
    action TEXT NOT NULL,
    description TEXT,
    data TEXT,
    status TEXT NOT NULL DEFAULT 'SUCCESS',
    duration_ms INTEGER,
    ip_address TEXT,
    user_agent TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_operation_logs_user_id ON operation_logs(user_id);
CREATE INDEX idx_operation_logs_created_at ON operation_logs(created_at);

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
