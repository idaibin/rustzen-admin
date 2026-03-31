-- ============================================================================
-- File Number: 107003
-- File Name: view_user_info_summary.sql
-- Module: Views
-- Description: Comprehensive user information with menu, role, and permission counts for dashboard display. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE VIEW user_info_summary AS
SELECT
    u.id,
    u.username,
    u.email,
    u.real_name,
    u.avatar_url,
    u.status,
    u.is_super_admin,
    u.last_login_at,
    u.created_at,
    COUNT(DISTINCT m.id) as menu_count,
    COUNT(DISTINCT r.id) as role_count,
    COUNT(DISTINCT CASE WHEN m.permission_code IS NOT NULL THEN m.id END) as permission_count
FROM users u
LEFT JOIN user_roles ur ON u.id = ur.user_id
LEFT JOIN roles r ON ur.role_id = r.id AND r.status = 1 AND r.deleted_at IS NULL
LEFT JOIN role_menus rm ON r.id = rm.role_id
LEFT JOIN menus m ON rm.menu_id = m.id AND m.status = 1 AND m.deleted_at IS NULL
WHERE u.deleted_at IS NULL
GROUP BY u.id, u.username, u.email, u.real_name, u.avatar_url, u.status, u.is_super_admin, u.last_login_at, u.created_at;

COMMENT ON VIEW user_info_summary IS 'Comprehensive user information with menu, role, and permission counts for dashboard display';
