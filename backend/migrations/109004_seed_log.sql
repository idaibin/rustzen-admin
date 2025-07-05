-- ============================================================================
-- File Number: 109004
-- File Name: seed_log.sql
-- Module: Seed Data
-- Description: Seed initial log data (optional, for demo/testing). Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

-- Ensure current month partition exists
SELECT create_log_partition(CURRENT_DATE);

-- Example: Insert a demo log entry for testing
INSERT INTO operation_logs (user_id, username, action, description, status, created_at)
VALUES (1, 'superadmin', 'LOGIN', 'Superadmin login for demo', 'SUCCESS', CURRENT_TIMESTAMP)
ON CONFLICT DO NOTHING;
