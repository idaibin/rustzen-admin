-- ============================================================================
-- File Number: 107002
-- File Name: view_user_menu_info.sql
-- Module: Views
-- Description: Optimized view for user menu information with proper column mapping for AuthMenuInfoEntity structure. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE VIEW user_menu_info AS
SELECT DISTINCT
    u.id as user_id,
    m.id,
    CASE WHEN m.parent_id = 0 THEN NULL ELSE m.parent_id END as parent_id,
    m.title,
    COALESCE(m.path, '') as path,
    m.component,
    m.icon,
    m.sort_order as order_num,
    CASE WHEN m.status = 1 THEN true ELSE false END as visible,
    false as keep_alive,
    m.menu_type
FROM users u
JOIN user_roles ur ON u.id = ur.user_id
JOIN roles r ON ur.role_id = r.id AND r.status = 1 AND r.deleted_at IS NULL
JOIN role_menus rm ON r.id = rm.role_id
JOIN menus m ON rm.menu_id = m.id AND m.status = 1 AND m.deleted_at IS NULL
WHERE u.deleted_at IS NULL
  AND u.status = 1
  AND m.menu_type IN (1, 2, 3)
ORDER BY u.id, m.sort_order, m.id;

COMMENT ON VIEW user_menu_info IS 'Optimized view for user menu information with proper column mapping for AuthMenuInfoEntity structure';
