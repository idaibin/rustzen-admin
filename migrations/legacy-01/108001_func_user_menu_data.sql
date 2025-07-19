-- ============================================================================
-- File Number: 108001
-- File Name: func_user_menu_data.sql
-- Module: Functions
-- Description: Efficiently retrieves menu data for a specific user with proper column mapping for AuthMenuInfoEntity. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

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

COMMENT ON FUNCTION get_user_menu_data(BIGINT) IS 'Efficiently retrieves menu data for a specific user with proper column mapping for AuthMenuInfoEntity';
