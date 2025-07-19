-- ============================================================================
-- File Number: 100400
-- File Name: create_log.sql
-- Module: Log Management
-- Description: Create operation_logs table (partitioned), indexes, and comments. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE TABLE operation_logs (
    id BIGSERIAL,
    user_id BIGINT,
    username VARCHAR(50),
    action VARCHAR(100) NOT NULL,
    description TEXT,
    ip_address INET,
    user_agent TEXT,
    request_id VARCHAR(100),
    resource_type VARCHAR(50),
    resource_id BIGINT,
    status VARCHAR(20) DEFAULT 'SUCCESS',
    duration_ms INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

COMMENT ON TABLE operation_logs IS 'Operation logs table (partitioned by month): stores operation logs for auditing';
COMMENT ON COLUMN operation_logs.user_agent IS 'User agent string';
COMMENT ON COLUMN operation_logs.request_id IS 'Request ID for tracing';
COMMENT ON COLUMN operation_logs.resource_type IS 'Type of operated resource';
COMMENT ON COLUMN operation_logs.resource_id IS 'ID of operated resource';
COMMENT ON COLUMN operation_logs.status IS 'Operation status';
COMMENT ON COLUMN operation_logs.duration_ms IS 'Operation duration in milliseconds';
