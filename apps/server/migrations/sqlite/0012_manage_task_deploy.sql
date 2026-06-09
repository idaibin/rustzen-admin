-- ============================================================================
-- Module: Manage tasks, deploy versions.
-- ============================================================================

CREATE TABLE IF NOT EXISTS system_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    schedule_type TEXT NOT NULL CHECK(schedule_type IN ('cron')),
    schedule_json TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
    running INTEGER NOT NULL DEFAULT 0 CHECK(running IN (0, 1)),
    last_run_id INTEGER,
    last_trigger_type TEXT CHECK(last_trigger_type IN ('scheduled', 'manual') OR last_trigger_type IS NULL),
    last_status TEXT CHECK(last_status IN ('running', 'success', 'failed', 'skipped') OR last_status IS NULL),
    last_started_at DATETIME,
    last_finished_at DATETIME,
    last_error_message TEXT,
    next_run_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_system_tasks_enabled_next_run_at
    ON system_tasks(enabled, next_run_at);

CREATE INDEX IF NOT EXISTS idx_system_tasks_running
    ON system_tasks(running, updated_at);

CREATE TABLE IF NOT EXISTS system_task_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_key TEXT NOT NULL,
    trigger_type TEXT NOT NULL CHECK(trigger_type IN ('scheduled', 'manual')),
    status TEXT NOT NULL CHECK(status IN ('running', 'success', 'failed', 'skipped')),
    scheduled_for DATETIME,
    started_at DATETIME NOT NULL,
    finished_at DATETIME,
    error_message TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_key) REFERENCES system_tasks(task_key) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_system_task_runs_task_key_created_at
    ON system_task_runs(task_key, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_system_task_runs_status_created_at
    ON system_task_runs(status, created_at DESC);

CREATE TABLE IF NOT EXISTS deploy_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    component TEXT NOT NULL CHECK(component IN ('server', 'web')),
    version TEXT NOT NULL,
    arch TEXT NOT NULL DEFAULT 'x86_64' CHECK(arch IN ('x86_64', 'aarch64')),
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL CHECK(file_size > 0),
    file_hash TEXT NOT NULL,
    is_current INTEGER NOT NULL DEFAULT 0 CHECK(is_current IN (0, 1)),
    is_deployed INTEGER NOT NULL DEFAULT 0 CHECK(is_deployed IN (0, 1)),
    is_expired INTEGER NOT NULL DEFAULT 0 CHECK(is_expired IN (0, 1)),
    deployed_at DATETIME,
    expired_at DATETIME,
    deleted_at DATETIME,
    deployed_by TEXT,
    notes TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(component, version, arch)
);

CREATE INDEX IF NOT EXISTS idx_deploy_versions_component
    ON deploy_versions(component);

CREATE INDEX IF NOT EXISTS idx_deploy_versions_version
    ON deploy_versions(version);

CREATE INDEX IF NOT EXISTS idx_deploy_versions_component_version_arch
    ON deploy_versions(component, version, arch);

CREATE INDEX IF NOT EXISTS idx_deploy_versions_is_current
    ON deploy_versions(is_current);

CREATE INDEX IF NOT EXISTS idx_deploy_versions_is_deployed
    ON deploy_versions(is_deployed);

CREATE INDEX IF NOT EXISTS idx_deploy_versions_is_expired
    ON deploy_versions(is_expired);

CREATE INDEX IF NOT EXISTS idx_deploy_versions_created_at
    ON deploy_versions(created_at DESC);
