-- ============================================================================
-- File Number: 108000
-- File Name: func_user_role.sql
-- Module: Functions
-- Description: Get all permissions for a specific role. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE FUNCTION get_role_permissions(p_role_id BIGINT)
RETURNS TABLE (
    permission_code VARCHAR(100),
    menu_id BIGINT,
    menu_title VARCHAR(100)
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        m.permission_code,
        m.id,
        m.title
    FROM role_menus rm
    JOIN menus m ON rm.menu_id = m.id AND m.deleted_at IS NULL
    WHERE rm.role_id = p_role_id;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_role_permissions(BIGINT) IS 'Get all permissions for a specific role.';
