CREATE TABLE automation_systems (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE automation_flows (
    id TEXT PRIMARY KEY NOT NULL,
    system_id TEXT NOT NULL,
    name TEXT NOT NULL,
    steps_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (system_id) REFERENCES automation_systems(id) ON DELETE RESTRICT
);

CREATE INDEX idx_automation_flows_system ON automation_flows(system_id, created_at DESC);

CREATE TABLE automation_runs (
    id TEXT PRIMARY KEY NOT NULL,
    flow_id TEXT NOT NULL,
    retry_source_run_id TEXT UNIQUE,
    status TEXT NOT NULL CHECK(status IN ('queued', 'running', 'succeeded', 'failed', 'cancelled')),
    input_json TEXT NOT NULL DEFAULT '{}',
    error TEXT,
    created_at TEXT NOT NULL,
    started_at TEXT,
    finished_at TEXT,
    cancel_requested_at TEXT,
    FOREIGN KEY (flow_id) REFERENCES automation_flows(id) ON DELETE RESTRICT,
    FOREIGN KEY (retry_source_run_id) REFERENCES automation_runs(id) ON DELETE SET NULL
);

CREATE INDEX idx_automation_runs_status_created ON automation_runs(status, created_at);

CREATE TABLE automation_run_steps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    step_index INTEGER NOT NULL,
    action TEXT NOT NULL,
    status TEXT NOT NULL,
    duration_ms INTEGER,
    message TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES automation_runs(id) ON DELETE CASCADE
);

CREATE TABLE automation_artifacts (
    id TEXT PRIMARY KEY NOT NULL,
    run_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    file_name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES automation_runs(id) ON DELETE CASCADE
);

CREATE TABLE automation_settings (
    singleton INTEGER PRIMARY KEY NOT NULL DEFAULT 1 CHECK(singleton = 1),
    run_retention_days INTEGER NOT NULL DEFAULT 30,
    artifact_retention_days INTEGER NOT NULL DEFAULT 30,
    default_step_timeout_seconds INTEGER NOT NULL DEFAULT 30,
    max_run_timeout_seconds INTEGER NOT NULL DEFAULT 600,
    updated_at TEXT NOT NULL
);

INSERT INTO automation_settings (singleton, updated_at)
VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

CREATE TABLE automation_schedules (
    id TEXT PRIMARY KEY NOT NULL,
    flow_id TEXT NOT NULL,
    cadence TEXT NOT NULL CHECK(cadence IN ('daily', 'weekly')),
    weekday INTEGER CHECK(weekday IS NULL OR (weekday >= 0 AND weekday <= 6)),
    due_time TEXT NOT NULL,
    input_json TEXT NOT NULL DEFAULT '{}',
    description TEXT NOT NULL DEFAULT '',
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
    effective_at TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 0 CHECK(revision >= 0),
    updated_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (flow_id) REFERENCES automation_flows(id) ON DELETE RESTRICT,
    CHECK((cadence = 'daily' AND weekday IS NULL) OR (cadence = 'weekly' AND weekday IS NOT NULL))
);

CREATE INDEX idx_automation_schedules_enabled ON automation_schedules(enabled, updated_at);

CREATE TABLE automation_schedule_occurrences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    schedule_id TEXT NOT NULL,
    occurrence_key TEXT NOT NULL,
    due_local TEXT NOT NULL,
    due_at TEXT,
    decided_at TEXT NOT NULL,
    decision TEXT NOT NULL CHECK(decision IN ('enqueued', 'skipped')),
    reason TEXT,
    run_id TEXT UNIQUE,
    run_id_snapshot TEXT,
    FOREIGN KEY (schedule_id) REFERENCES automation_schedules(id) ON DELETE CASCADE,
    FOREIGN KEY (run_id) REFERENCES automation_runs(id) ON DELETE SET NULL,
    CHECK(
        (decision = 'enqueued' AND run_id_snapshot IS NOT NULL)
        OR (decision = 'skipped' AND run_id IS NULL AND run_id_snapshot IS NULL)
    ),
    UNIQUE(schedule_id, occurrence_key)
);

CREATE INDEX idx_automation_schedule_occurrences_schedule
    ON automation_schedule_occurrences(schedule_id, due_local DESC, id DESC);
