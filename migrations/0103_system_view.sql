-- ============================================================================
-- Module: Aggregated view for user with roles.
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
    u.is_system,
    u.last_login_at ,
    u.created_at,
    u.updated_at,
    COALESCE(
        JSON_AGG(
            JSON_BUILD_OBJECT(
                'label', r.name,
                'value', r.id
            ) ORDER BY r.id
        ) FILTER (WHERE r.id IS NOT NULL),
        '[]'::json
    ) AS roles
FROM users u
LEFT JOIN user_roles ur ON u.id = ur.user_id
LEFT JOIN roles r ON ur.role_id = r.id AND r.deleted_at IS NULL
WHERE u.deleted_at IS NULL
GROUP BY u.id, u.username, u.email, u.real_name, u.avatar_url, u.status, u.is_system, u.last_login_at, u.created_at;

COMMENT ON VIEW user_with_roles IS 'Aggregated user info with roles as JSON array.';


-- ============================================================================
-- Module: Enhanced user permissions view with additional identifiers and better performance.
-- ============================================================================

CREATE OR REPLACE VIEW user_permissions AS
SELECT DISTINCT
    u.id as user_id,
    u.username,
    m.code as menu_code,
    m.menu_type,
    r.code as role_code,
    m.id as menu_id,
    r.id as role_id
FROM users u
JOIN user_roles ur ON u.id = ur.user_id
JOIN roles r ON ur.role_id = r.id AND r.status = 1 AND r.deleted_at IS NULL
JOIN role_menus rm ON r.id = rm.role_id
JOIN menus m ON rm.menu_id = m.id AND m.deleted_at IS NULL
WHERE u.deleted_at IS NULL
  AND u.status = 1
  AND m.code IS NOT NULL;

COMMENT ON VIEW user_permissions IS 'Enhanced user permissions view with additional menu and role identifiers for better performance';


-- ============================================================================
-- Module: Aggregated role info with menus as JSON array.
-- ============================================================================

CREATE OR REPLACE VIEW role_with_menus AS
SELECT
    r.id AS id,
    r.name,
    r.code,
    r.description,
    r.status,
    r.created_at,
    r.updated_at,
    r.deleted_at,
    r.is_system,
    COALESCE(
        JSON_AGG(
            JSON_BUILD_OBJECT(
                'label', m.name,
                'value', m.id
            ) ORDER BY m.id
        ) FILTER (WHERE m.id IS NOT NULL),
        '[]'::json
    ) AS menus
FROM roles r
LEFT JOIN role_menus rm ON r.id = rm.role_id
LEFT JOIN menus m ON rm.menu_id = m.id AND m.deleted_at IS NULL
WHERE r.deleted_at IS NULL
GROUP BY r.id, r.name, r.code, r.description, r.status, r.created_at, r.updated_at, r.deleted_at, r.is_system;

COMMENT ON VIEW role_with_menus IS 'Aggregated role info with menus as JSON array.';
