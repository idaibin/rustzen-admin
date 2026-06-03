-- ============================================================================
-- Module: User/role/menu view layer.
-- ============================================================================

CREATE VIEW IF NOT EXISTS user_with_roles AS
SELECT
    u.id AS id,
    u.username,
    u.email,
    u.real_name,
    u.password_hash,
    u.avatar_url,
    u.status,
    u.is_system,
    u.last_login_at,
    u.created_at,
    u.updated_at,
    COALESCE(
        (
            SELECT json_group_array(json_object('label', ro.name, 'value', ro.id))
            FROM (
                SELECT r.name, r.id
                FROM user_roles ur
                INNER JOIN roles r ON ur.role_id = r.id AND r.deleted_at IS NULL
                WHERE ur.user_id = u.id
                ORDER BY r.id
            ) ro
        ),
        '[]'
    ) AS roles
FROM users u
WHERE u.deleted_at IS NULL;

CREATE VIEW IF NOT EXISTS user_permissions AS
SELECT DISTINCT
    u.id AS user_id,
    u.username,
    m.code AS menu_code,
    m.menu_type,
    r.code AS role_code,
    m.id AS menu_id,
    r.id AS role_id
FROM users u
INNER JOIN user_roles ur ON u.id = ur.user_id
INNER JOIN roles r ON ur.role_id = r.id AND r.status = 1 AND r.deleted_at IS NULL
INNER JOIN role_menus rm ON r.id = rm.role_id
INNER JOIN menus m ON rm.menu_id = m.id AND m.deleted_at IS NULL
WHERE u.deleted_at IS NULL
  AND u.status = 1
  AND m.code IS NOT NULL;

CREATE VIEW IF NOT EXISTS role_with_menus AS
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
        (
            SELECT json_group_array(json_object('label', mo.name, 'value', mo.id))
            FROM (
                SELECT m.name, m.id
                FROM role_menus rm
                INNER JOIN menus m ON rm.menu_id = m.id AND m.deleted_at IS NULL
                WHERE rm.role_id = r.id
                ORDER BY m.id
            ) mo
        ),
        '[]'
    ) AS menus
FROM roles r
WHERE r.deleted_at IS NULL;
