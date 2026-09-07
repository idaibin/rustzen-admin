-- Rustzen Admin complete SQLite schema.
-- This project uses replaceable initialization SQL rather than patch migrations.

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
    auth_epoch INTEGER NOT NULL DEFAULT 1 CHECK (auth_epoch >= 1),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME
);

CREATE UNIQUE INDEX idx_users_username ON users(username) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_users_email ON users(email) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_deleted_at ON users(deleted_at);

CREATE TABLE access_policy_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    authz_epoch INTEGER NOT NULL CHECK (authz_epoch >= 1),
    updated_at INTEGER NOT NULL
);

INSERT INTO access_policy_state (id, authz_epoch, updated_at)
VALUES (1, 1, unixepoch());

CREATE TABLE access_sessions (
    sid TEXT PRIMARY KEY CHECK (length(sid) BETWEEN 16 AND 128),
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    auth_epoch_at_issue INTEGER NOT NULL CHECK (auth_epoch_at_issue >= 1),
    expires_at INTEGER NOT NULL,
    revoked_at INTEGER,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_access_sessions_user_active
    ON access_sessions(user_id, revoked_at, expires_at, created_at);
CREATE INDEX idx_access_sessions_expiry ON access_sessions(expires_at);
CREATE INDEX idx_access_sessions_revoked ON access_sessions(revoked_at) WHERE revoked_at IS NOT NULL;

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
    id TEXT PRIMARY KEY CHECK (id IN ('monitor', 'insights', 'reports')),
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

CREATE TRIGGER users_password_auth_epoch
AFTER UPDATE OF password_hash ON users
WHEN OLD.password_hash IS NOT NEW.password_hash
BEGIN
    UPDATE users SET auth_epoch = OLD.auth_epoch + 1 WHERE id = NEW.id;
END;

CREATE TRIGGER users_status_auth_epoch
AFTER UPDATE OF status, deleted_at ON users
WHEN OLD.status IS NOT NEW.status OR OLD.deleted_at IS NOT NEW.deleted_at
BEGIN
    UPDATE users SET auth_epoch = OLD.auth_epoch + 1 WHERE id = NEW.id;
    UPDATE access_policy_state SET authz_epoch = authz_epoch + 1, updated_at = unixepoch()
    WHERE id = 1;
END;

CREATE TRIGGER roles_policy_epoch AFTER UPDATE OF status, deleted_at ON roles
WHEN OLD.status IS NOT NEW.status OR OLD.deleted_at IS NOT NEW.deleted_at
BEGIN
    UPDATE access_policy_state SET authz_epoch = authz_epoch + 1, updated_at = unixepoch()
    WHERE id = 1;
END;

CREATE TRIGGER user_roles_insert_policy_epoch AFTER INSERT ON user_roles BEGIN
    UPDATE access_policy_state SET authz_epoch = authz_epoch + 1, updated_at = unixepoch()
    WHERE id = 1;
END;
CREATE TRIGGER user_roles_delete_policy_epoch AFTER DELETE ON user_roles BEGIN
    UPDATE access_policy_state SET authz_epoch = authz_epoch + 1, updated_at = unixepoch()
    WHERE id = 1;
END;

CREATE TRIGGER user_roles_update_policy_epoch AFTER UPDATE OF user_id, role_id ON user_roles BEGIN
    UPDATE access_policy_state SET authz_epoch = authz_epoch + 1, updated_at = unixepoch()
    WHERE id = 1;
END;
CREATE TRIGGER role_menus_insert_policy_epoch AFTER INSERT ON role_menus BEGIN
    UPDATE access_policy_state SET authz_epoch = authz_epoch + 1, updated_at = unixepoch()
    WHERE id = 1;
END;
CREATE TRIGGER role_menus_delete_policy_epoch AFTER DELETE ON role_menus BEGIN
    UPDATE access_policy_state SET authz_epoch = authz_epoch + 1, updated_at = unixepoch()
    WHERE id = 1;
END;

CREATE TRIGGER role_menus_update_policy_epoch AFTER UPDATE OF role_id, menu_id ON role_menus BEGIN
    UPDATE access_policy_state SET authz_epoch = authz_epoch + 1, updated_at = unixepoch()
    WHERE id = 1;
END;
CREATE TRIGGER menus_policy_epoch
AFTER UPDATE OF status, is_active, deleted_at ON menus
WHEN OLD.status IS NOT NEW.status OR OLD.is_active IS NOT NEW.is_active
  OR OLD.deleted_at IS NOT NEW.deleted_at
BEGIN
    UPDATE access_policy_state SET authz_epoch = authz_epoch + 1, updated_at = unixepoch()
    WHERE id = 1;
END;
CREATE TRIGGER modules_policy_epoch AFTER UPDATE OF enabled ON modules
WHEN OLD.enabled IS NOT NEW.enabled
BEGIN
    UPDATE access_policy_state SET authz_epoch = authz_epoch + 1, updated_at = unixepoch()
    WHERE id = 1;
END;

CREATE TABLE system_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    schedule_type TEXT NOT NULL CHECK(schedule_type IN ('cron')),
    schedule_json TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
    running INTEGER NOT NULL DEFAULT 0 CHECK(running IN (0, 1)),
    last_run_id INTEGER,
    last_trigger_type TEXT CHECK(last_trigger_type IN ('scheduled', 'manual') OR last_trigger_type IS NULL),
    last_status TEXT CHECK(last_status IN ('running', 'success', 'failed', 'skipped') OR last_status IS NULL),
    last_started_at DATETIME,
    last_finished_at DATETIME,
    last_error_message TEXT,
    next_run_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_system_tasks_enabled_next_run_at ON system_tasks(enabled, next_run_at);
CREATE INDEX idx_system_tasks_running ON system_tasks(running, updated_at);

CREATE TABLE system_task_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_key TEXT NOT NULL,
    trigger_type TEXT NOT NULL CHECK(trigger_type IN ('scheduled', 'manual')),
    status TEXT NOT NULL CHECK(status IN ('running', 'success', 'failed', 'skipped')),
    scheduled_for DATETIME,
    started_at DATETIME NOT NULL,
    finished_at DATETIME,
    error_message TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_key) REFERENCES system_tasks(task_key) ON DELETE CASCADE
);

CREATE INDEX idx_system_task_runs_task_key_created_at
    ON system_task_runs(task_key, created_at DESC);
CREATE INDEX idx_system_task_runs_status_created_at
    ON system_task_runs(status, created_at DESC);

CREATE TABLE deploy_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    component TEXT NOT NULL DEFAULT 'release' CHECK(component = 'release'),
    version TEXT NOT NULL,
    arch TEXT NOT NULL CHECK(arch IN ('x86_64', 'aarch64')),
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL CHECK(file_size > 0),
    file_hash TEXT NOT NULL,
    is_current INTEGER NOT NULL DEFAULT 0 CHECK(is_current IN (0, 1)),
    is_deployed INTEGER NOT NULL DEFAULT 0 CHECK(is_deployed IN (0, 1)),
    is_expired INTEGER NOT NULL DEFAULT 0 CHECK(is_expired IN (0, 1)),
    deployed_at DATETIME,
    expired_at DATETIME,
    deleted_at DATETIME,
    deployed_by TEXT,
    notes TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(component, version, arch)
);

CREATE INDEX idx_release_deploy_versions_component ON deploy_versions(component);
CREATE INDEX idx_release_deploy_versions_current ON deploy_versions(is_current);
CREATE INDEX idx_release_deploy_versions_expired ON deploy_versions(is_expired);
CREATE INDEX idx_release_deploy_versions_created ON deploy_versions(created_at DESC);
CREATE UNIQUE INDEX idx_release_deploy_versions_one_current
    ON deploy_versions(is_current)
    WHERE is_current = 1 AND deleted_at IS NULL;

INSERT INTO modules (id, enabled)
VALUES ('monitor', 1), ('insights', 1), ('reports', 1);

INSERT INTO users (username, email, password_hash, real_name, status, is_system)
VALUES
    ('owner', 'owner@example.com', '$argon2id$v=19$m=19456,t=2,p=1$i2SSaoqEMMwYzJQPXhVHfg$k1Y5bZ/k5SxEoEroG+UFzCW8aKzK1o/DWKKDU34FiPI', '所有者', 1, 1),
    ('admin', 'admin@example.com', '$argon2id$v=19$m=19456,t=2,p=1$i2SSaoqEMMwYzJQPXhVHfg$k1Y5bZ/k5SxEoEroG+UFzCW8aKzK1o/DWKKDU34FiPI', '管理员', 1, 1),
    ('viewer', 'viewer@example.com', '$argon2id$v=19$m=19456,t=2,p=1$i2SSaoqEMMwYzJQPXhVHfg$k1Y5bZ/k5SxEoEroG+UFzCW8aKzK1o/DWKKDU34FiPI', '查看者', 1, 1);

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
WHERE u.username IN ('owner', 'admin', 'viewer');

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
-- Optional Admin notification schema for fresh compositions selecting notifications.

CREATE TABLE notification_accounting (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    message_count INTEGER NOT NULL DEFAULT 0 CHECK (message_count >= 0),
    recipient_count INTEGER NOT NULL DEFAULT 0 CHECK (recipient_count >= 0),
    receipt_count INTEGER NOT NULL DEFAULT 0 CHECK (receipt_count >= 0),
    charged_bytes INTEGER NOT NULL DEFAULT 0 CHECK (charged_bytes >= 0)
);
INSERT INTO notification_accounting (id) VALUES (1);

CREATE TABLE notification_receipts (
    producer TEXT NOT NULL CHECK (producer IN ('monitor', 'reports')),
    event_id TEXT NOT NULL CHECK (length(CAST(event_id AS BLOB)) BETWEEN 1 AND 128),
    payload_sha256 TEXT NOT NULL CHECK (
        length(payload_sha256) = 64 AND payload_sha256 NOT GLOB '*[^0-9a-f]*'
    ),
    accepted_at DATETIME NOT NULL,
    expires_at DATETIME NOT NULL,
    retain_until DATETIME NOT NULL,
    result TEXT NOT NULL CHECK (result IN ('stored', 'no-recipients')),
    charged_bytes INTEGER NOT NULL DEFAULT 512 CHECK (charged_bytes >= 512),
    PRIMARY KEY (producer, event_id)
);
CREATE INDEX idx_notification_receipts_retain_until
    ON notification_receipts(retain_until);

CREATE TABLE notifications (
    inbox_seq INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE CHECK (length(CAST(id AS BLOB)) BETWEEN 1 AND 64),
    producer TEXT NOT NULL CHECK (producer IN ('monitor', 'reports')),
    event_id TEXT NOT NULL CHECK (length(CAST(event_id AS BLOB)) BETWEEN 1 AND 128),
    topic TEXT NOT NULL CHECK (length(CAST(topic AS BLOB)) BETWEEN 1 AND 128),
    subject_kind TEXT NOT NULL CHECK (length(CAST(subject_kind AS BLOB)) BETWEEN 1 AND 64),
    subject_id TEXT NOT NULL CHECK (length(CAST(subject_id AS BLOB)) BETWEEN 1 AND 256),
    subject_revision INTEGER NOT NULL CHECK (subject_revision > 0),
    occurred_at DATETIME NOT NULL,
    accepted_at DATETIME NOT NULL,
    title TEXT NOT NULL CHECK (length(CAST(title AS BLOB)) BETWEEN 1 AND 256),
    summary TEXT NOT NULL CHECK (length(CAST(summary AS BLOB)) <= 1024),
    required_capability TEXT NOT NULL CHECK (
        length(CAST(required_capability AS BLOB)) BETWEEN 1 AND 128
    ),
    charged_bytes INTEGER NOT NULL DEFAULT 1024 CHECK (charged_bytes >= 1024),
    UNIQUE (producer, event_id),
    FOREIGN KEY (producer, event_id)
        REFERENCES notification_receipts(producer, event_id) ON DELETE CASCADE
);
CREATE INDEX idx_notifications_accepted_order ON notifications(inbox_seq DESC);
CREATE INDEX idx_notifications_accepted_at ON notifications(accepted_at, inbox_seq);

CREATE TABLE notification_recipients (
    notification_id TEXT NOT NULL CHECK (
        length(CAST(notification_id AS BLOB)) BETWEEN 1 AND 64
    ),
    user_id INTEGER NOT NULL,
    read_at DATETIME,
    created_at DATETIME NOT NULL,
    charged_bytes INTEGER NOT NULL DEFAULT 256 CHECK (charged_bytes >= 256),
    PRIMARY KEY (notification_id, user_id),
    FOREIGN KEY (notification_id) REFERENCES notifications(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX idx_notification_recipients_user_read
    ON notification_recipients(user_id, read_at, notification_id);

CREATE TABLE notification_user_state (
    user_id INTEGER PRIMARY KEY,
    revision INTEGER NOT NULL DEFAULT 0 CHECK (revision >= 0),
    charged_bytes INTEGER NOT NULL DEFAULT 128 CHECK (charged_bytes >= 128),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TRIGGER notification_receipts_account_insert
AFTER INSERT ON notification_receipts BEGIN
    UPDATE notification_accounting SET
        receipt_count = receipt_count + 1,
        charged_bytes = charged_bytes + NEW.charged_bytes WHERE id = 1;
END;
CREATE TRIGGER notification_receipts_account_delete
AFTER DELETE ON notification_receipts BEGIN
    UPDATE notification_accounting SET
        receipt_count = receipt_count - 1,
        charged_bytes = charged_bytes - OLD.charged_bytes WHERE id = 1;
END;
CREATE TRIGGER notifications_account_insert
AFTER INSERT ON notifications BEGIN
    UPDATE notification_accounting SET
        message_count = message_count + 1,
        charged_bytes = charged_bytes + NEW.charged_bytes WHERE id = 1;
END;
CREATE TRIGGER notifications_account_delete
AFTER DELETE ON notifications BEGIN
    UPDATE notification_accounting SET
        message_count = message_count - 1,
        charged_bytes = charged_bytes - OLD.charged_bytes WHERE id = 1;
END;
CREATE TRIGGER notification_recipients_account_insert
AFTER INSERT ON notification_recipients BEGIN
    UPDATE notification_accounting SET
        recipient_count = recipient_count + 1,
        charged_bytes = charged_bytes + NEW.charged_bytes WHERE id = 1;
END;
CREATE TRIGGER notification_recipients_account_delete
AFTER DELETE ON notification_recipients BEGIN
    UPDATE notification_accounting SET
        recipient_count = recipient_count - 1,
        charged_bytes = charged_bytes - OLD.charged_bytes WHERE id = 1;
END;
CREATE TRIGGER notification_user_state_account_insert
AFTER INSERT ON notification_user_state BEGIN
    UPDATE notification_accounting SET
        charged_bytes = charged_bytes + NEW.charged_bytes WHERE id = 1;
END;
CREATE TRIGGER notification_user_state_account_delete
AFTER DELETE ON notification_user_state BEGIN
    UPDATE notification_accounting SET
        charged_bytes = charged_bytes - OLD.charged_bytes WHERE id = 1;
END;

CREATE TRIGGER notification_receipts_charge_immutable
BEFORE UPDATE OF charged_bytes ON notification_receipts BEGIN
    SELECT RAISE(ABORT, 'notification receipt charge is immutable');
END;
CREATE TRIGGER notifications_charge_immutable
BEFORE UPDATE OF charged_bytes ON notifications BEGIN
    SELECT RAISE(ABORT, 'notification message charge is immutable');
END;
CREATE TRIGGER notification_recipients_charge_immutable
BEFORE UPDATE OF charged_bytes ON notification_recipients BEGIN
    SELECT RAISE(ABORT, 'notification recipient charge is immutable');
END;
CREATE TRIGGER notification_user_state_charge_immutable
BEFORE UPDATE OF charged_bytes ON notification_user_state BEGIN
    SELECT RAISE(ABORT, 'notification user-state charge is immutable');
END;
