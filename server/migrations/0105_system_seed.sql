
-- ============================================================================
-- Module: Seed initial super admin user data.
-- ============================================================================

INSERT INTO users (username, email, password_hash, real_name, status, is_system)
VALUES (
    'superadmin',
    'superadmin@example.com',
    -- 密码为 "rustzen@123" 的 argon2id hash 示例（请根据实际安全策略替换）
    '$argon2id$v=19$m=19456,t=2,p=1$i2SSaoqEMMwYzJQPXhVHfg$k1Y5bZ/k5SxEoEroG+UFzCW8aKzK1o/DWKKDU34FiPI',
    'Super Administrator',
    1,
    TRUE
)
ON CONFLICT (username) DO NOTHING;

-- ============================================================================
-- Module: Seed initial roles (system admin, user manager, auditor).
-- ============================================================================

INSERT INTO roles (name, code, description, status, is_system, sort_order)
VALUES
    ('System Administrator', 'SYSTEM_ADMIN', 'Wildcard admin role. The "*" permission grants access to all system functions.', 1, TRUE, 1)
ON CONFLICT (code) DO NOTHING;


-- ============================================================================
-- Module: Seed initial system super-admin menu only.
-- ============================================================================

INSERT INTO menus (parent_id, name, code, menu_type, sort_order, status, is_system, is_manual)
VALUES (
    0,
    'All Permissions',
    '*',
    1,
    1,
    2,
    TRUE,
    FALSE
)
ON CONFLICT (code) DO NOTHING;


-- ============================================================================
-- Module: Seed initial role_menus data.
-- ============================================================================

INSERT INTO role_menus (role_id, menu_id, created_at)
SELECT r.id, m.id, NOW()
FROM roles r, menus m
WHERE r.code = 'SYSTEM_ADMIN' AND m.code = '*'
ON CONFLICT (role_id, menu_id) DO NOTHING;

-- ============================================================================
-- Module: Seed initial dictionary data (example types and entries).
-- ============================================================================

INSERT INTO dicts (dict_type, label, value, status, sort_order)
VALUES
    ('user_status', 'Active', '1', 1, 1),
    ('user_status', 'Disabled', '2', 1, 2),
    ('user_status', 'Pending', '3', 1, 3),
    ('user_status', 'Locked', '4', 1, 4),
    ('role_type', 'System Role', '1', 1, 1),
    ('role_type', 'Custom Role', '2', 1, 2)
ON CONFLICT DO NOTHING;
