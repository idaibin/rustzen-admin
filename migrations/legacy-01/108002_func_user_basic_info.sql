-- ============================================================================
-- File Number: 108002
-- File Name: func_user_basic_info.sql
-- Module: Functions
-- Description: Efficiently retrieves basic user information for authentication responses. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE FUNCTION get_user_basic_info(p_user_id BIGINT)
RETURNS TABLE (
    id BIGINT,
    username VARCHAR(50),
    email VARCHAR(100),
    real_name VARCHAR(50),
    avatar_url VARCHAR(255),
    status SMALLINT,
    is_system BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.id,
        u.username,
        u.email,
        u.real_name,
        u.avatar_url,
        u.status,
        u.is_system
    FROM users u
    WHERE u.id = p_user_id
      AND u.deleted_at IS NULL
      AND u.status = 1;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_user_basic_info(BIGINT) IS 'Efficiently retrieves basic user information for authentication responses';
