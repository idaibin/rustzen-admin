CREATE TABLE rustzen_installation_identity (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    build_id TEXT NOT NULL,
    composition_id TEXT NOT NULL,
    schema_fingerprint TEXT NOT NULL,
    data_contract_id TEXT NOT NULL
);

CREATE TABLE monitor_nodes (
    node_id TEXT PRIMARY KEY NOT NULL,
    hostname TEXT NOT NULL,
    agent_version TEXT NOT NULL,
    current_boot_id TEXT NOT NULL,
    last_sequence INTEGER NOT NULL,
    last_report_at TEXT NOT NULL,
    last_received_at TEXT NOT NULL,
    cpu_percent REAL NOT NULL,
    memory_used_bytes INTEGER NOT NULL,
    memory_total_bytes INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE monitor_boots (
    node_id TEXT NOT NULL,
    boot_id TEXT NOT NULL,
    last_sequence INTEGER NOT NULL,
    retired_at TEXT,
    PRIMARY KEY (node_id, boot_id),
    FOREIGN KEY (node_id) REFERENCES monitor_nodes(node_id) ON DELETE CASCADE
);

CREATE TABLE resource_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id TEXT NOT NULL,
    cpu_percent REAL NOT NULL,
    memory_used_bytes INTEGER NOT NULL,
    memory_total_bytes INTEGER NOT NULL,
    collected_at TEXT NOT NULL,
    FOREIGN KEY (node_id) REFERENCES monitor_nodes(node_id) ON DELETE CASCADE
);

CREATE TABLE disk_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id TEXT NOT NULL,
    mount_point TEXT NOT NULL,
    used_bytes INTEGER NOT NULL,
    total_bytes INTEGER NOT NULL,
    collected_at TEXT NOT NULL,
    FOREIGN KEY (node_id) REFERENCES monitor_nodes(node_id) ON DELETE CASCADE
);

CREATE INDEX idx_resource_samples_node_time
ON resource_samples(node_id, collected_at DESC);

CREATE INDEX idx_disk_samples_node_mount_time
ON disk_samples(node_id, mount_point, collected_at DESC);

CREATE INDEX idx_disk_samples_node_time_mount
ON disk_samples(node_id, collected_at, mount_point);

CREATE TABLE alert_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    cpu_enabled INTEGER NOT NULL DEFAULT 1,
    cpu_threshold_percent REAL NOT NULL DEFAULT 90,
    memory_enabled INTEGER NOT NULL DEFAULT 1,
    memory_threshold_percent REAL NOT NULL DEFAULT 90,
    disk_enabled INTEGER NOT NULL DEFAULT 1,
    disk_threshold_percent REAL NOT NULL DEFAULT 90,
    offline_enabled INTEGER NOT NULL DEFAULT 1,
    offline_after_seconds INTEGER NOT NULL DEFAULT 90,
    updated_at TEXT NOT NULL
);

INSERT INTO alert_settings (id, updated_at)
VALUES (1, CURRENT_TIMESTAMP);

CREATE TABLE node_alert_settings (
    node_id TEXT PRIMARY KEY NOT NULL,
    cpu_enabled INTEGER NOT NULL DEFAULT 1,
    cpu_threshold_percent REAL NOT NULL DEFAULT 90,
    memory_enabled INTEGER NOT NULL DEFAULT 1,
    memory_threshold_percent REAL NOT NULL DEFAULT 90,
    disk_enabled INTEGER NOT NULL DEFAULT 1,
    disk_threshold_percent REAL NOT NULL DEFAULT 90,
    offline_enabled INTEGER NOT NULL DEFAULT 1,
    offline_after_seconds INTEGER NOT NULL DEFAULT 90,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (node_id) REFERENCES monitor_nodes(node_id) ON DELETE CASCADE
);

CREATE TABLE alert_counters (
    node_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    target TEXT NOT NULL,
    abnormal_count INTEGER NOT NULL DEFAULT 0,
    normal_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (node_id, kind, target),
    FOREIGN KEY (node_id) REFERENCES monitor_nodes(node_id) ON DELETE CASCADE
);

CREATE TABLE monitor_incidents (
    id TEXT PRIMARY KEY NOT NULL,
    node_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    target TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'resolved')),
    title TEXT NOT NULL,
    threshold_percent REAL,
    observed_percent REAL,
    details TEXT NOT NULL DEFAULT '{}',
    opened_at TEXT NOT NULL,
    last_observed_at TEXT NOT NULL,
    resolved_at TEXT,
    resolution_reason TEXT,
    FOREIGN KEY (node_id) REFERENCES monitor_nodes(node_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_monitor_incidents_active
ON monitor_incidents(node_id, kind, target)
WHERE status = 'active';

CREATE INDEX idx_monitor_incidents_status_time
ON monitor_incidents(status, last_observed_at DESC);

CREATE TABLE node_daily_summaries (
    node_id TEXT NOT NULL,
    summary_date TEXT NOT NULL,
    sample_count INTEGER NOT NULL,
    cpu_min REAL,
    cpu_avg REAL,
    cpu_max REAL,
    memory_min REAL,
    memory_avg REAL,
    memory_max REAL,
    disk_summary_json TEXT NOT NULL,
    coverage_percent REAL NOT NULL DEFAULT 0,
    offline_seconds INTEGER NOT NULL DEFAULT 0,
    incident_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (node_id, summary_date),
    FOREIGN KEY (node_id) REFERENCES monitor_nodes(node_id) ON DELETE CASCADE
);
