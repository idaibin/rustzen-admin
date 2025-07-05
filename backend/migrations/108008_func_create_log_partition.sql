-- ============================================================================
-- File Number: 108008
-- File Name: func_create_log_partition.sql
-- Module: Functions
-- Description: Creates a new partition for operation_logs table for the specified month. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE FUNCTION create_log_partition(partition_date DATE)
RETURNS VOID AS $$
DECLARE
    partition_name TEXT;
    start_date DATE;
    end_date DATE;
BEGIN
    partition_name := 'operation_logs_' || to_char(partition_date, 'YYYY_MM');
    start_date := date_trunc('month', partition_date);
    end_date := start_date + INTERVAL '1 month';
    -- Create partition table (if not exists)
    EXECUTE format('CREATE TABLE IF NOT EXISTS %I PARTITION OF operation_logs
                    FOR VALUES FROM (%L) TO (%L)',
                   partition_name, start_date, end_date);
    -- Create indexes for partition table (if not exists)
    EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%s_user_id ON %I(user_id)',
                   partition_name, partition_name);
    EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%s_action ON %I(action)',
                   partition_name, partition_name);
    EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%s_created_at ON %I(created_at)',
                   partition_name, partition_name);
    RAISE NOTICE 'Created partition: % for period % to %', partition_name, start_date, end_date;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION create_log_partition(DATE) IS 'Creates a new partition for operation_logs table for the specified month';
