
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
    ('System Administrator', 'SYSTEM_ADMIN', 'System administrator with full access to all system functions', 1, TRUE, 1)
ON CONFLICT (code) DO NOTHING;


-- ============================================================================
-- Module: Seed initial system menu structure.
-- ============================================================================

INSERT INTO menus (parent_id, name, code, menu_type, sort_order, status, is_system)
VALUES
    (0, 'System Super Admin', '*', 1, 1, 2, TRUE),  -- 系统超级管理员 id: 1
    (0, 'System Management', 'system:*', 1, 1, 1, TRUE), -- 系统管理 id: 2

    (2, 'User Management', 'system:user:*', 1, 1, 1, TRUE), -- 用户管理 id: 3
    (3, 'User List', 'system:user:list', 2, 1, 1, TRUE), -- 用户列表 id: 4
    (3, 'User Create', 'system:user:create', 3, 2, 1, TRUE), -- 用户创建 id: 5
    (3, 'User Update', 'system:user:update', 3, 3, 1, TRUE), -- 用户更新 id: 6
    (3, 'User Detail', 'system:user:detail', 3, 4, 1, TRUE), -- 用户详情 id: 7
    (3, 'User Delete', 'system:user:delete', 3, 5, 1, TRUE), -- 用户删除 id: 8

    (2, 'Role Management', 'system:role:*', 1, 1, 1, TRUE), -- 角色管理 id: 9
    (9, 'Role List', 'system:role:list', 2, 1, 1, TRUE), -- 角色列表 id: 10
    (9, 'Role Create', 'system:role:create', 3, 2, 1, TRUE), -- 角色创建 id: 11
    (9, 'Role Update', 'system:role:update', 3, 3, 1, TRUE), -- 角色更新 id: 12
    (9, 'Role Detail', 'system:role:detail', 3, 4, 1, TRUE), -- 角色详情 id: 13
    (9, 'Role Delete', 'system:role:delete', 3, 5, 1, TRUE), -- 角色删除 id: 14

    (2, 'Menu Management', 'system:menu:*', 1, 1, 1, TRUE), -- 菜单管理 id: 15
    (15, 'Menu List', 'system:menu:list', 2, 1, 1, TRUE), -- 菜单列表 id: 16
    (15, 'Menu Create', 'system:menu:create', 3, 2, 1, TRUE), -- 菜单创建 id: 17
    (15, 'Menu Update', 'system:menu:update', 3, 3, 1, TRUE), -- 菜单更新 id: 18
    (15, 'Menu Detail', 'system:menu:detail', 3, 4, 1, TRUE), -- 菜单详情 id: 19
    (15, 'Menu Delete', 'system:menu:delete', 3, 5, 1, TRUE), -- 菜单删除 id: 20

    (2, 'Dictionary Management', 'system:dict:*', 1, 1, 1, TRUE), -- 字典管理 id: 21
    (21, 'Dictionary List', 'system:dict:list', 2, 1, 1, TRUE), -- 字典列表 id: 22
    (21, 'Dictionary Create', 'system:dict:create', 3, 2, 1, TRUE), -- 字典创建 id: 23
    (21, 'Dictionary Update', 'system:dict:update', 3, 3, 1, TRUE), -- 字典更新 id: 24
    (21, 'Dictionary Detail', 'system:dict:detail', 3, 4, 1, TRUE), -- 字典详情 id: 25
    (21, 'Dictionary Delete', 'system:dict:delete', 3, 5, 1, TRUE), -- 字典删除 id: 26

    (2, 'Operation Logs', 'system:log:*', 1, 1, 1, TRUE), -- 操作日志 id: 27
    (27, 'Operation Logs List', 'system:log:list', 2, 1, 1, TRUE), -- 操作日志列表 id: 28
    (27, 'Operation Logs Detail', 'system:log:detail', 3, 2, 1, TRUE) -- 操作日志详情 id: 29
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

