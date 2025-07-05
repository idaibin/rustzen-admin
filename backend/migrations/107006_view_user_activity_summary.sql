-- ============================================================================
-- File Number: 107006
-- File Name: view_user_activity_summary.sql
-- Module: Views
-- Description: Summary of user activity for the last 30 days. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE OR REPLACE VIEW user_activity_summary AS
SELECT
    user_id,
    username,
    COUNT(*) as total_operations,
    COUNT(CASE WHEN status = 'SUCCESS' THEN 1 END) as successful_operations,
    COUNT(CASE WHEN status != 'SUCCESS' THEN 1 END) as failed_operations,
    AVG(duration_ms) as avg_duration_ms,
    MAX(created_at) as last_activity
FROM operation_logs
WHERE created_at >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY user_id, username
ORDER BY total_operations DESC;

COMMENT ON VIEW user_activity_summary IS 'Summary of user activity for the last 30 days';
