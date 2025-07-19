-- ============================================================================
-- File Number: 107004
-- File Name: view_user_info_stats.sql
-- Module: Views
-- Description: Statistics view for monitoring user info system performance and health. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE VIEW user_info_stats AS
SELECT
    'total_users' as metric,
    COUNT(*) as value,
    'count' as unit
FROM users
WHERE deleted_at IS NULL

UNION ALL

SELECT
    'active_users' as metric,
    COUNT(*) as value,
    'count' as unit
FROM users
WHERE deleted_at IS NULL AND status = 1

UNION ALL

SELECT
    'super_admin_users' as metric,
    COUNT(*) as value,
    'count' as unit
FROM users
WHERE deleted_at IS NULL AND is_super_admin = TRUE

UNION ALL

SELECT
    'total_user_menu_mappings' as metric,
    COUNT(*) as value,
    'count' as unit
FROM user_menu_info

UNION ALL

SELECT
    'total_user_permissions' as metric,
    COUNT(*) as value,
    'count' as unit
FROM user_permissions

UNION ALL

SELECT
    'avg_menus_per_user' as metric,
    ROUND(AVG(menu_count), 2) as value,
    'average' as unit
FROM user_info_summary
WHERE menu_count > 0

UNION ALL

SELECT
    'avg_permissions_per_user' as metric,
    ROUND(AVG(permission_count), 2) as value,
    'average' as unit
FROM user_info_summary
WHERE permission_count > 0;

COMMENT ON VIEW user_info_stats IS 'Statistics view for monitoring user info system performance and health';
