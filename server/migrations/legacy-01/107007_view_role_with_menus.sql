CREATE OR REPLACE VIEW role_with_menus AS
SELECT
    r.id AS id,
    r.role_name,
    r.role_code,
    r.description,
    r.status,
    r.created_at,
    r.updated_at,
    r.deleted_at,
    r.is_system,
    COALESCE(
        JSON_AGG(
            JSON_BUILD_OBJECT(
                'id', m.id,
                'name', m.title,
                'code', m.permission_code
            ) ORDER BY m.id
        ) FILTER (WHERE m.id IS NOT NULL),
        '[]'::json
    ) AS menus
FROM roles r
LEFT JOIN role_menus rm ON r.id = rm.role_id
LEFT JOIN menus m ON rm.menu_id = m.id AND m.deleted_at IS NULL
WHERE r.deleted_at IS NULL
GROUP BY r.id, r.role_name, r.role_code, r.description, r.status, r.created_at, r.updated_at, r.deleted_at, r.is_system;

COMMENT ON VIEW role_with_menus IS 'Aggregated role info with menus as JSON array.';