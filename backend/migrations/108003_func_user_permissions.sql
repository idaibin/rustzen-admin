-- ============================================================================
-- File Number: 108003
-- File Name: func_user_permissions.sql
-- Module: Functions
-- Description: Efficiently retrieves all permissions for a specific user. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE FUNCTION get_user_permissions(p_user_id BIGINT)
RETURNS TABLE (
    permission_code VARCHAR(100),
    menu_type SMALLINT,
    role_code VARCHAR(50)
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        up.permission_code,
        up.menu_type,
        up.role_code
    FROM user_permissions up
    WHERE up.user_id = p_user_id;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_user_permissions(BIGINT) IS 'Efficiently retrieves all permissions for a specific user';
