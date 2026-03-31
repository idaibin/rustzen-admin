-- ============================================================================
-- File Number: 109001
-- File Name: seed_menu.sql
-- Module: Seed Data
-- Description: Seed initial system menu structure. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

-- Example: Insert root and main system menus (expand as needed)
INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
VALUES
    (0, 'System Management', '/system', NULL, 'system', 1, 1, 1, TRUE, 'system:*'),
    (1, 'User Management', '/system/users', 'UserList', 'user', 1, 1, 2, TRUE, 'system:user:list'),
    (1, 'Role Management', '/system/roles', 'RoleList', 'team', 2, 1, 2, TRUE, 'system:role:list'),
    (1, 'Menu Management', '/system/menus', 'MenuList', 'menu', 3, 1, 2, TRUE, 'system:menu:list'),
    (1, 'Dictionary Management', '/system/dicts', 'DictList', 'book', 4, 1, 2, TRUE, 'system:dict:list'),
    (1, 'Operation Logs', '/system/logs', 'LogList', 'file-text', 5, 1, 2, TRUE, 'system:log:list')
ON CONFLICT (permission_code) DO NOTHING;
