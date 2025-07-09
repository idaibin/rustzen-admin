-- ============================================================================
-- File Number: 107000
-- File Name: view_user_with_roles.sql
-- Module: Views
-- Description: Aggregated view for user with roles. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE VIEW user_with_roles AS
SELECT
    u.id AS id,
    u.username,
    u.email,
    u.real_name,
    u.password_hash,
    u.avatar_url,
    u.status,
    u.is_super_admin,
    u.last_login_at::timestamptz AS last_login_at,  -- 这里加类型转换
    u.created_at::timestamptz AS created_at,
    u.updated_at::timestamptz AS updated_at,
    COALESCE(
        JSON_AGG(
            JSON_BUILD_OBJECT(
                'id', r.id,
                'role_name', r.role_name,
                'role_code', r.role_code
            ) ORDER BY r.id
        ) FILTER (WHERE r.id IS NOT NULL),
        '[]'::json
    ) AS roles
FROM users u
LEFT JOIN user_roles ur ON u.id = ur.user_id
LEFT JOIN roles r ON ur.role_id = r.id AND r.deleted_at IS NULL
WHERE u.deleted_at IS NULL
GROUP BY u.id, u.username, u.email, u.real_name, u.avatar_url, u.status, u.is_super_admin, u.last_login_at, u.created_at;

COMMENT ON VIEW user_with_roles IS 'Aggregated user info with roles as JSON array.';
