-- ============================================================================
-- File Number: 108009
-- File Name: func_manage_log_partitions.sql
-- Module: Functions
-- Description: Comprehensive partition management: creates future partitions and cleans up old ones. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE FUNCTION manage_log_partitions()
RETURNS VOID AS $$
DECLARE
    future_month DATE;
    partition_name TEXT;
    partition_exists BOOLEAN;
    months_ahead INTEGER := 6;  -- Create partitions 6 months ahead
    months_keep INTEGER := 12;  -- Keep data for 12 months
    old_month DATE;
BEGIN
    RAISE NOTICE 'Starting log partition management...';
    -- Create partitions for future months
    FOR i IN 0..months_ahead LOOP
        future_month := (CURRENT_DATE + (i || ' months')::INTERVAL)::DATE;
        partition_name := 'operation_logs_' || to_char(future_month, 'YYYY_MM');
        -- Check if partition exists
        SELECT EXISTS (
            SELECT 1 FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE c.relname = partition_name
            AND n.nspname = 'public'
        ) INTO partition_exists;
        -- Create if not exists
        IF NOT partition_exists THEN
            PERFORM create_log_partition(future_month);
        ELSE
            RAISE NOTICE 'Partition % already exists', partition_name;
        END IF;
    END LOOP;
    -- Clean up old partitions (delete partitions older than retention period)
    FOR i IN months_keep..(months_keep + 12) LOOP
        old_month := (CURRENT_DATE - (i || ' months')::INTERVAL)::DATE;
        partition_name := 'operation_logs_' || to_char(old_month, 'YYYY_MM');
        -- Check if partition exists
        SELECT EXISTS (
            SELECT 1 FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE c.relname = partition_name
            AND n.nspname = 'public'
        ) INTO partition_exists;
        -- Delete if exists
        IF partition_exists THEN
            EXECUTE format('DROP TABLE IF EXISTS %I', partition_name);
            RAISE NOTICE 'Dropped old partition: %', partition_name;
        END IF;
    END LOOP;
    RAISE NOTICE 'Log partition management completed';
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION manage_log_partitions() IS 'Comprehensive partition management: creates future partitions and cleans up old ones';
