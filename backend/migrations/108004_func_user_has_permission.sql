-- ============================================================================
-- File Number: 108004
-- File Name: func_user_has_permission.sql
-- Module: Functions
-- Description: Checks if a user has a specific permission (returns boolean). Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE FUNCTION user_has_permission(p_user_id BIGINT, p_permission_code VARCHAR(100))
RETURNS BOOLEAN AS $$
DECLARE
    has_permission BOOLEAN := FALSE;
BEGIN
    SELECT EXISTS(
        SELECT 1
        FROM user_permissions up
        WHERE up.user_id = p_user_id
          AND up.permission_code = p_permission_code
    ) INTO has_permission;

    RETURN has_permission;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION user_has_permission(BIGINT, VARCHAR) IS 'Checks if a user has a specific permission (returns boolean)';
