-- ============================================================================
-- File Number: 108007
-- File Name: func_log_operations_bulk.sql
-- Module: Functions
-- Description: Bulk log operations for better performance. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE FUNCTION log_operations_bulk(
    p_logs JSONB
)
RETURNS INTEGER AS $$
DECLARE
    log_record JSONB;
    inserted_count INTEGER := 0;
BEGIN
    -- Ensure current month partition exists
    PERFORM create_log_partition(CURRENT_DATE::DATE);
    -- Process each log record
    FOR log_record IN SELECT * FROM jsonb_array_elements(p_logs)
    LOOP
        INSERT INTO operation_logs (
            user_id, username, action, description, ip_address,
            user_agent, request_id, resource_type, resource_id,
            status, duration_ms, created_at
        ) VALUES (
            (log_record->>'user_id')::BIGINT,
            log_record->>'username',
            log_record->>'action',
            log_record->>'description',
            (log_record->>'ip_address')::INET,
            log_record->>'user_agent',
            log_record->>'request_id',
            log_record->>'resource_type',
            (log_record->>'resource_id')::BIGINT,
            COALESCE(log_record->>'status', 'SUCCESS'),
            (log_record->>'duration_ms')::INTEGER,
            CURRENT_TIMESTAMP
        );
        inserted_count := inserted_count + 1;
    END LOOP;
    RETURN inserted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION log_operations_bulk(JSONB) IS 'Bulk log operations for better performance';
