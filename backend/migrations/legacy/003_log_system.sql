-- ============================================================================
-- Log System Implementation
-- This file contains all log-related functionality including:
-- - Operation logs table with partitioning
-- - Partition management functions
-- - Log recording functions
-- - Monitoring views
-- ============================================================================

-- ============================================================================
-- 1. Operation Logs Table (Partitioned)
-- ============================================================================
CREATE TABLE operation_logs (
    id BIGSERIAL, -- Unique log ID
    user_id BIGINT, -- Operator user ID
    username VARCHAR(50), -- Operator username
    action VARCHAR(100) NOT NULL, -- Operation type
    description TEXT, -- Operation description
    ip_address INET, -- Operator IP address
    user_agent TEXT, -- User agent string
    request_id VARCHAR(100), -- Request ID for tracing
    resource_type VARCHAR(50), -- Type of operated resource
    resource_id BIGINT, -- ID of operated resource
    status VARCHAR(20) DEFAULT 'SUCCESS', -- Operation status
    duration_ms INTEGER, -- Operation duration in milliseconds
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Operation timestamp
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- Comments for operation_logs table and columns
COMMENT ON TABLE operation_logs IS 'Operation logs table (partitioned by month): stores operation logs for auditing';
COMMENT ON COLUMN operation_logs.user_agent IS 'User agent string';
COMMENT ON COLUMN operation_logs.request_id IS 'Request ID for tracing';
COMMENT ON COLUMN operation_logs.resource_type IS 'Type of operated resource';
COMMENT ON COLUMN operation_logs.resource_id IS 'ID of operated resource';
COMMENT ON COLUMN operation_logs.status IS 'Operation status';
COMMENT ON COLUMN operation_logs.duration_ms IS 'Operation duration in milliseconds';

-- ============================================================================
-- 2. Partition Management Functions
-- ============================================================================

-- Function: automatically create log partitions
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

-- Function: comprehensive partition management
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

-- Function: get partition info for monitoring
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

-- ============================================================================
-- 3. Log Recording Functions
-- ============================================================================

-- Function: log operation with automatic partition handling
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

-- Function: bulk log operations (for performance)
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

-- ============================================================================
-- 4. Monitoring Views
-- ============================================================================

-- View: recent operations (last 1000 records)
CREATE OR REPLACE VIEW recent_operations AS
SELECT 
    id, user_id, username, action, description, ip_address,
    status, duration_ms, created_at
FROM operation_logs
ORDER BY created_at DESC
LIMIT 1000;

-- View: user activity summary
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

-- ============================================================================
-- 5. Initial Partition Creation
-- ============================================================================

-- Create initial partitions for current and next month
SELECT create_log_partition(CURRENT_DATE::DATE);
SELECT create_log_partition((CURRENT_DATE + INTERVAL '1 month')::DATE);

-- ============================================================================
-- 6. Comments and Documentation
-- ============================================================================

COMMENT ON FUNCTION create_log_partition(DATE) IS 'Creates a new partition for operation_logs table for the specified month';
COMMENT ON FUNCTION manage_log_partitions() IS 'Comprehensive partition management: creates future partitions and cleans up old ones';
COMMENT ON FUNCTION log_operation(BIGINT, VARCHAR, VARCHAR, TEXT, INET, TEXT, VARCHAR, VARCHAR, BIGINT, VARCHAR, INTEGER) IS 'Logs a single operation with automatic partition handling';
COMMENT ON FUNCTION log_operations_bulk(JSONB) IS 'Bulk log operations for better performance';
COMMENT ON VIEW recent_operations IS 'View of recent operations for monitoring';
COMMENT ON VIEW user_activity_summary IS 'Summary of user activity for the last 30 days';
