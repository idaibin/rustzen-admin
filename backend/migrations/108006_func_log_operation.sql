-- ============================================================================
-- File Number: 108006
-- File Name: func_log_operation.sql
-- Module: Functions
-- Description: Logs a single operation with automatic partition handling. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE FUNCTION log_operation(
    p_user_id BIGINT,
    p_username VARCHAR(50),
    p_action VARCHAR(100),
    p_description TEXT,
    p_ip_address INET DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL,
    p_request_id VARCHAR(100) DEFAULT NULL,
    p_resource_type VARCHAR(50) DEFAULT NULL,
    p_resource_id BIGINT DEFAULT NULL,
    p_status VARCHAR(20) DEFAULT 'SUCCESS',
    p_duration_ms INTEGER DEFAULT NULL
)
RETURNS BIGINT AS $$
DECLARE
    log_id BIGINT;
BEGIN
    -- Ensure current month partition exists
    PERFORM create_log_partition(CURRENT_DATE::DATE);
    -- Insert log record
    INSERT INTO operation_logs (
        user_id, username, action, description, ip_address,
        user_agent, request_id, resource_type, resource_id,
        status, duration_ms, created_at
    ) VALUES (
        p_user_id, p_username, p_action, p_description, p_ip_address,
        p_user_agent, p_request_id, p_resource_type, p_resource_id,
        p_status, p_duration_ms, CURRENT_TIMESTAMP
    ) RETURNING id INTO log_id;
    RETURN log_id;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION log_operation(BIGINT, VARCHAR, VARCHAR, TEXT, INET, TEXT, VARCHAR, VARCHAR, BIGINT, VARCHAR, INTEGER) IS 'Logs a single operation with automatic partition handling';
