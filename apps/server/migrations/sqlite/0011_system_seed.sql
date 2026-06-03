-- ============================================================================
-- Module: Initial seed data.
-- ============================================================================

INSERT INTO users (username, email, password_hash, real_name, status, is_system)
VALUES (
    'superadmin',
    'superadmin@example.com',
    '$argon2id$v=19$m=19456,t=2,p=1$i2SSaoqEMMwYzJQPXhVHfg$k1Y5bZ/k5SxEoEroG+UFzCW8aKzK1o/DWKKDU34FiPI',
    'Super Administrator',
    1,
    1
)
ON CONFLICT DO NOTHING;

INSERT INTO roles (name, code, description, status, is_system, sort_order)
VALUES (
    'System Administrator',
    'SYSTEM_ADMIN',
    'Wildcard admin role. The "*" permission grants access to all system functions.',
    1,
    1,
    1
)
ON CONFLICT DO NOTHING;

INSERT INTO menus (parent_id, name, code, menu_type, sort_order, status, is_system, is_manual)
VALUES (
    0,
    'All Permissions',
    '*',
    1,
    1,
    2,
    1,
    0
)
ON CONFLICT DO NOTHING;

INSERT INTO role_menus (role_id, menu_id, created_at)
SELECT r.id, m.id, CURRENT_TIMESTAMP
FROM roles r
CROSS JOIN menus m
WHERE r.code = 'SYSTEM_ADMIN'
  AND m.code = '*'
ON CONFLICT DO NOTHING;

INSERT INTO dicts (dict_type, label, value, status, sort_order)
VALUES
    ('user_status', 'Active', '1', 1, 1),
    ('user_status', 'Disabled', '2', 1, 2),
    ('user_status', 'Pending', '3', 1, 3),
    ('user_status', 'Locked', '4', 1, 4),
    ('role_type', 'System Role', '1', 1, 1),
    ('role_type', 'Custom Role', '2', 1, 2)
ON CONFLICT DO NOTHING;
