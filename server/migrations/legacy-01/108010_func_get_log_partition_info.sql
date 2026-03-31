-- ============================================================================
-- File Number: 108010
-- File Name: func_get_log_partition_info.sql
-- Module: Functions
-- Description: Provides partition info for operation_logs. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE FUNCTION get_log_partition_info()
RETURNS TABLE (
    partition_name TEXT,
    start_date DATE,
    end_date DATE,
    row_count BIGINT,
    size_mb NUMERIC
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        c.relname::TEXT as partition_name,
        pg_get_expr(c.relpartbound, c.oid)::TEXT as partition_info,
        COALESCE(pg_stat_get_live_tuples(c.oid), 0) as row_count,
        ROUND(pg_total_relation_size(c.oid) / 1024.0 / 1024.0, 2) as size_mb
    FROM pg_class c
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE c.relname LIKE 'operation_logs_%'
    AND n.nspname = 'public'
    ORDER BY c.relname;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION get_log_partition_info() IS 'Provides partition info for operation_logs';
