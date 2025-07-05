-- ============================================================================
-- File Number: 107001
-- File Name: view_user_permissions.sql
-- Module: Views
-- Description: Enhanced user permissions view with additional identifiers and better performance. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE VIEW user_permissions AS
SELECT DISTINCT
    u.id as user_id,
    u.username,
    m.permission_code,
    m.menu_type,
    r.role_code,
    m.id as menu_id,
    r.id as role_id
FROM users u
JOIN user_roles ur ON u.id = ur.user_id
JOIN roles r ON ur.role_id = r.id AND r.status = 1 AND r.deleted_at IS NULL
JOIN role_menus rm ON r.id = rm.role_id
JOIN menus m ON rm.menu_id = m.id AND m.status = 1 AND m.deleted_at IS NULL
WHERE u.deleted_at IS NULL
  AND u.status = 1
  AND m.permission_code IS NOT NULL;

COMMENT ON VIEW user_permissions IS 'Enhanced user permissions view with additional menu and role identifiers for better performance';
