-- ============================================================================
-- File Number: 108005
-- File Name: func_login_credentials.sql
-- Module: Functions
-- Description: Efficiently retrieves user credentials for login authentication. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE FUNCTION get_login_credentials(p_username VARCHAR(50))
RETURNS TABLE (
    id BIGINT,
    username VARCHAR(50),
    email VARCHAR(100),
    password_hash VARCHAR(255),
    real_name VARCHAR(50),
    avatar_url VARCHAR(255),
    status SMALLINT,
    is_super_admin BOOLEAN,
    last_login_at TIMESTAMP
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.id,
        u.username,
        u.email,
        u.password_hash,
        u.real_name,
        u.avatar_url,
        u.status,
        u.is_super_admin,
        u.last_login_at
    FROM users u
    WHERE u.username = p_username
      AND u.deleted_at IS NULL;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_login_credentials(VARCHAR) IS 'Efficiently retrieves user credentials for login authentication';
