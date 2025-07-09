-- ============================================================================
-- File Number: 107005
-- File Name: view_recent_operations.sql
-- Module: Views
-- Description: View of recent operations for monitoring. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE VIEW recent_operations AS
SELECT
    id, user_id, username, action, description, ip_address,
    status, duration_ms, created_at
FROM operation_logs
ORDER BY created_at DESC
LIMIT 1000;

COMMENT ON VIEW recent_operations IS 'View of recent operations for monitoring';
