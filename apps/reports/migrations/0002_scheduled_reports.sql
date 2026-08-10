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
