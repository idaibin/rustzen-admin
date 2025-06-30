-- ============================================================================
-- User Info Query Optimization
-- This migration optimizes user information queries for better performance
-- and resolves column mapping issues between database schema and application models
-- ============================================================================

-- ============================================================================
-- 1. Drop and Recreate User-Related Views with Optimizations
-- ============================================================================

-- Drop existing views to recreate with optimizations
DROP VIEW IF EXISTS user_menu_tree;
DROP VIEW IF EXISTS user_permissions;

-- ============================================================================
-- 2. Enhanced User Permissions View
-- ============================================================================

-- Enhanced user permissions view with additional identifiers and better performance
CREATE OR REPLACE VIEW user_permissions AS
SELECT DISTINCT
    u.id as user_id,
    u.username,
    m.permission_code,
    m.menu_type,
    r.role_code,
    m.id as menu_id,
    r.id as role_id
FROM users u
JOIN user_roles ur ON u.id = ur.user_id
JOIN roles r ON ur.role_id = r.id AND r.status = 1 AND r.deleted_at IS NULL
JOIN role_menus rm ON r.id = rm.role_id
JOIN menus m ON rm.menu_id = m.id AND m.status = 1 AND m.deleted_at IS NULL
WHERE u.deleted_at IS NULL
  AND u.status = 1
  AND m.permission_code IS NOT NULL;

COMMENT ON VIEW user_permissions IS 'Enhanced user permissions view with additional menu and role identifiers for better performance';

-- ============================================================================
-- 3. User Menu Info View (Optimized for AuthMenuInfoEntity)
-- ============================================================================

-- Create optimized user menu info view that matches AuthMenuInfoEntity structure
CREATE OR REPLACE VIEW user_menu_info AS
SELECT DISTINCT
    u.id as user_id,
    m.id,
    CASE WHEN m.parent_id = 0 THEN NULL ELSE m.parent_id END as parent_id,
    m.title,
    COALESCE(m.path, '') as path,  -- Ensure non-null path
    m.component,
    m.icon,
    m.sort_order as order_num,     -- Map sort_order to order_num for Rust struct
    CASE WHEN m.status = 1 THEN true ELSE false END as visible,  -- Map status to visible boolean
    false as keep_alive,           -- Default to false since this column doesn't exist in schema
    m.menu_type
FROM users u
JOIN user_roles ur ON u.id = ur.user_id
JOIN roles r ON ur.role_id = r.id AND r.status = 1 AND r.deleted_at IS NULL
JOIN role_menus rm ON r.id = rm.role_id
JOIN menus m ON rm.menu_id = m.id AND m.status = 1 AND m.deleted_at IS NULL
WHERE u.deleted_at IS NULL
  AND u.status = 1
  AND m.menu_type IN (1, 2, 3)  -- Only include valid menu types
ORDER BY u.id, m.sort_order, m.id;

COMMENT ON VIEW user_menu_info IS 'Optimized view for user menu information with proper column mapping for AuthMenuInfoEntity structure';

-- ============================================================================
-- 4. User Info Summary View
-- ============================================================================

-- Create a comprehensive user info view that combines user data with statistics
CREATE OR REPLACE VIEW user_info_summary AS
SELECT
    u.id,
    u.username,
    u.email,
    u.real_name,
    u.avatar_url,
    u.status,
    u.is_super_admin,
    u.last_login_at,
    u.created_at,
    COUNT(DISTINCT m.id) as menu_count,
    COUNT(DISTINCT r.id) as role_count,
    COUNT(DISTINCT CASE WHEN m.permission_code IS NOT NULL THEN m.id END) as permission_count
FROM users u
LEFT JOIN user_roles ur ON u.id = ur.user_id
LEFT JOIN roles r ON ur.role_id = r.id AND r.status = 1 AND r.deleted_at IS NULL
LEFT JOIN role_menus rm ON r.id = rm.role_id
LEFT JOIN menus m ON rm.menu_id = m.id AND m.status = 1 AND m.deleted_at IS NULL
WHERE u.deleted_at IS NULL
GROUP BY u.id, u.username, u.email, u.real_name, u.avatar_url, u.status, u.is_super_admin, u.last_login_at, u.created_at;

COMMENT ON VIEW user_info_summary IS 'Comprehensive user information with menu, role, and permission counts for dashboard display';

-- ============================================================================
-- 5. Performance Indexes for User Authentication and Authorization
-- ============================================================================

-- Create composite index for user authentication lookups (username + status check)
CREATE INDEX IF NOT EXISTS idx_users_auth_lookup
ON users(username, status)
WHERE deleted_at IS NULL;

-- Create composite index for user email lookups
CREATE INDEX IF NOT EXISTS idx_users_email_lookup
ON users(email, status)
WHERE deleted_at IS NULL;

-- Create composite index for user roles lookup optimization
CREATE INDEX IF NOT EXISTS idx_user_roles_composite
ON user_roles(user_id, role_id);

-- Create composite index for role menus lookup optimization
CREATE INDEX IF NOT EXISTS idx_role_menus_composite
ON role_menus(role_id, menu_id);

-- Create composite index for active menus with hierarchy support
CREATE INDEX IF NOT EXISTS idx_menus_active_hierarchy
ON menus(parent_id, sort_order, menu_type, status)
WHERE deleted_at IS NULL;

-- Create index for permission code lookups
CREATE INDEX IF NOT EXISTS idx_menus_permission_code
ON menus(permission_code)
WHERE permission_code IS NOT NULL AND deleted_at IS NULL AND status = 1;

-- Create index for active roles
CREATE INDEX IF NOT EXISTS idx_roles_active
ON roles(status, is_system)
WHERE deleted_at IS NULL;

-- ============================================================================
-- 6. Helper Functions for Optimized User Info Queries
-- ============================================================================

-- Function to get user menu data efficiently with proper column mapping
CREATE OR REPLACE FUNCTION get_user_menu_data(p_user_id BIGINT)
RETURNS TABLE (
    id BIGINT,
    parent_id BIGINT,
    title VARCHAR(100),
    path VARCHAR(255),
    component VARCHAR(100),
    icon VARCHAR(100),
    order_num INTEGER,
    visible BOOLEAN,
    keep_alive BOOLEAN,
    menu_type SMALLINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        umi.id,
        umi.parent_id,
        umi.title,
        umi.path,
        umi.component,
        umi.icon,
        umi.order_num,
        umi.visible,
        umi.keep_alive,
        umi.menu_type
    FROM user_menu_info umi
    WHERE umi.user_id = p_user_id
    ORDER BY umi.order_num, umi.id;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get user basic info efficiently for authentication
CREATE OR REPLACE FUNCTION get_user_basic_info(p_user_id BIGINT)
RETURNS TABLE (
    id BIGINT,
    username VARCHAR(50),
    email VARCHAR(100),
    real_name VARCHAR(50),
    avatar_url VARCHAR(255),
    status SMALLINT,
    is_super_admin BOOLEAN
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
        u.is_super_admin
    FROM users u
    WHERE u.id = p_user_id
      AND u.deleted_at IS NULL
      AND u.status = 1;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get user permissions efficiently
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

-- Function to check if user has specific permission
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

-- ============================================================================
-- 7. User Login Optimization Function
-- ============================================================================

-- Function to get login credentials efficiently
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

-- ============================================================================
-- 8. Statistics and Monitoring Views
-- ============================================================================

-- Create a view for monitoring user info system performance
CREATE OR REPLACE VIEW user_info_stats AS
SELECT
    'total_users' as metric,
    COUNT(*) as value,
    'count' as unit
FROM users
WHERE deleted_at IS NULL

UNION ALL

SELECT
    'active_users' as metric,
    COUNT(*) as value,
    'count' as unit
FROM users
WHERE deleted_at IS NULL AND status = 1

UNION ALL

SELECT
    'super_admin_users' as metric,
    COUNT(*) as value,
    'count' as unit
FROM users
WHERE deleted_at IS NULL AND is_super_admin = TRUE

UNION ALL

SELECT
    'total_user_menu_mappings' as metric,
    COUNT(*) as value,
    'count' as unit
FROM user_menu_info

UNION ALL

SELECT
    'total_user_permissions' as metric,
    COUNT(*) as value,
    'count' as unit
FROM user_permissions

UNION ALL

SELECT
    'avg_menus_per_user' as metric,
    ROUND(AVG(menu_count), 2) as value,
    'average' as unit
FROM user_info_summary
WHERE menu_count > 0

UNION ALL

SELECT
    'avg_permissions_per_user' as metric,
    ROUND(AVG(permission_count), 2) as value,
    'average' as unit
FROM user_info_summary
WHERE permission_count > 0;

COMMENT ON VIEW user_info_stats IS 'Statistics view for monitoring user info system performance and health';

-- ============================================================================
-- 9. Comments and Documentation
-- ============================================================================

COMMENT ON FUNCTION get_user_menu_data(BIGINT) IS 'Efficiently retrieves menu data for a specific user with proper column mapping for AuthMenuInfoEntity';
COMMENT ON FUNCTION get_user_basic_info(BIGINT) IS 'Efficiently retrieves basic user information for authentication responses';
COMMENT ON FUNCTION get_user_permissions(BIGINT) IS 'Efficiently retrieves all permissions for a specific user';
COMMENT ON FUNCTION user_has_permission(BIGINT, VARCHAR) IS 'Checks if a user has a specific permission (returns boolean)';
COMMENT ON FUNCTION get_login_credentials(VARCHAR) IS 'Efficiently retrieves user credentials for login authentication';

-- ============================================================================
-- 10. Performance Analysis (Optional - for monitoring)
-- ============================================================================

-- Create a function to analyze user query performance
CREATE OR REPLACE FUNCTION analyze_user_query_performance()
RETURNS TABLE (
    view_name TEXT,
    avg_rows BIGINT,
    estimated_cost TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        'user_permissions'::TEXT as view_name,
        COUNT(*) as avg_rows,
        'Low - indexed joins'::TEXT as estimated_cost
    FROM user_permissions

    UNION ALL

    SELECT
        'user_menu_info'::TEXT as view_name,
        COUNT(*) as avg_rows,
        'Low - indexed joins'::TEXT as estimated_cost
    FROM user_menu_info

    UNION ALL

    SELECT
        'user_info_summary'::TEXT as view_name,
        COUNT(*) as avg_rows,
        'Medium - aggregation'::TEXT as estimated_cost
    FROM user_info_summary;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION analyze_user_query_performance() IS 'Provides performance analysis of user-related views for monitoring';