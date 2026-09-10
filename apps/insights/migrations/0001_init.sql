CREATE TABLE insights_projects (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    project_key_hash TEXT NOT NULL UNIQUE,
    collection_enabled INTEGER NOT NULL DEFAULT 0 CHECK (collection_enabled IN (0, 1)),
    allowed_origins TEXT NOT NULL DEFAULT '[]',
    archived_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE insights_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL,
    event_name TEXT NOT NULL,
    visitor_id TEXT NOT NULL,
    user_id TEXT,
    session_id TEXT,
    platform TEXT,
    page_path TEXT,
    referrer TEXT,
    api_path TEXT,
    api_method TEXT,
    status_code INTEGER,
    duration_ms INTEGER,
    is_error INTEGER NOT NULL DEFAULT 0 CHECK(is_error IN (0, 1)),
    properties TEXT NOT NULL DEFAULT '{}',
    occurred_at TEXT NOT NULL,
    received_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES insights_projects(id) ON DELETE CASCADE
);

CREATE INDEX idx_insights_events_project_time
ON insights_events(project_id, occurred_at DESC);
CREATE INDEX idx_insights_events_project_name_time
ON insights_events(project_id, event_name, occurred_at DESC);
CREATE INDEX idx_insights_events_project_page_time
ON insights_events(project_id, page_path, occurred_at DESC);
CREATE INDEX idx_insights_events_project_api_time
ON insights_events(project_id, api_path, occurred_at DESC);
CREATE INDEX idx_insights_events_project_visitor_time
ON insights_events(project_id, visitor_id, occurred_at DESC);

CREATE TABLE insights_settings (
    singleton INTEGER PRIMARY KEY NOT NULL DEFAULT 1 CHECK(singleton = 1),
    event_retention_days INTEGER NOT NULL DEFAULT 30,
    default_query_days INTEGER NOT NULL DEFAULT 7,
    max_query_days INTEGER NOT NULL DEFAULT 90,
    max_batch_events INTEGER NOT NULL DEFAULT 50,
    business_timezone TEXT NOT NULL DEFAULT 'UTC',
    updated_at TEXT NOT NULL
);

INSERT INTO insights_settings (singleton, updated_at)
VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

INSERT INTO insights_projects (
    id, name, project_key_hash, allowed_origins, created_at, updated_at
)
VALUES (
    'default', '默认', '', '[]',
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
);
