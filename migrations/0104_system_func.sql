
-- ============================================================================
-- Module: Update updated_at column trigger function and update triggers for users, roles, menus.
-- ============================================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for updating updated_at on row update
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_roles_updated_at ON roles;
CREATE TRIGGER update_roles_updated_at BEFORE UPDATE ON roles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_menus_updated_at ON menus;
CREATE TRIGGER update_menus_updated_at BEFORE UPDATE ON menus
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_dicts_updated_at ON dicts;
CREATE TRIGGER update_dicts_updated_at BEFORE UPDATE ON dicts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();


-- ============================================================================
-- Module: Get user basic info
-- ============================================================================

CREATE OR REPLACE FUNCTION get_user_basic_info(p_user_id BIGINT)
RETURNS TABLE (
    id BIGINT,
    username VARCHAR(50),
    email VARCHAR(100),
    real_name VARCHAR(50),
    avatar_url VARCHAR(255),
    is_system BOOLEAN,
    status SMALLINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.id,
        u.username,
        u.email,
        u.real_name,
        u.avatar_url,
        u.is_system,
        u.status
    FROM users u
    WHERE u.id = p_user_id
      AND u.deleted_at IS NULL
      AND u.status = 1;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_user_basic_info(BIGINT) IS 'Efficiently retrieves basic user information for authentication responses';

-- ============================================================================
-- Module: Get user permissions
-- ============================================================================

DROP FUNCTION IF EXISTS get_user_permissions(BIGINT);

CREATE FUNCTION get_user_permissions(p_user_id BIGINT)
RETURNS TEXT[] AS $$
DECLARE
    perms TEXT[];
BEGIN
    SELECT array_agg(up.menu_code)
    INTO perms
    FROM user_permissions up
    WHERE up.user_id = p_user_id;

    RETURN COALESCE(perms, ARRAY[]::TEXT[]);
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_user_permissions(BIGINT) IS 'Efficiently retrieves all permissions for a specific user';

-- ============================================================================
-- Module: Get login credentials
-- ============================================================================

DROP FUNCTION IF EXISTS get_login_credentials(VARCHAR);

CREATE FUNCTION get_login_credentials(p_username VARCHAR(50))
RETURNS TABLE (
    id BIGINT,
    password_hash VARCHAR(255),
    status SMALLINT,
    is_system BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.id,
        u.password_hash,
        u.status,
        u.is_system
    FROM users u
    WHERE u.username = p_username
      AND u.deleted_at IS NULL;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_login_credentials(VARCHAR) IS 'Efficiently retrieves user credentials for login authentication';


-- ============================================================================
-- Module: Log operation
-- ============================================================================

DROP FUNCTION IF EXISTS log_operation(
    BIGINT,
    VARCHAR,
    VARCHAR,
    TEXT,
    JSONB,
    INET,
    TEXT,
    VARCHAR,
    INTEGER
);

CREATE FUNCTION log_operation(
    p_user_id BIGINT,
    p_username VARCHAR(50),
    p_action VARCHAR(100),
    p_description TEXT,
    p_data JSONB DEFAULT NULL,
    p_ip_address INET DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL,
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
        user_id, username, action, description, data, status, duration_ms, ip_address, user_agent, created_at
    ) VALUES (
        p_user_id, p_username, p_action, p_description, p_data, p_status, p_duration_ms, p_ip_address, p_user_agent, CURRENT_TIMESTAMP
    ) RETURNING id INTO log_id;
    RETURN log_id;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION log_operation(BIGINT, VARCHAR, VARCHAR, TEXT, JSONB, INET, TEXT, VARCHAR, INTEGER) IS 'Logs a single operation with automatic partition handling, including data field.';


-- ============================================================================
-- Module: Create log partition
-- ============================================================================

DROP FUNCTION IF EXISTS create_log_partition(DATE);

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
