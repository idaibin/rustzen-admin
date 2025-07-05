-- ============================================================================
-- File Number: 108011
-- File Name: func_analyze_user_query_performance.sql
-- Module: Functions
-- Description: Provides performance analysis of user-related views for monitoring. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

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
