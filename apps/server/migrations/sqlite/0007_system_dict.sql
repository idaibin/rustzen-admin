-- ============================================================================
-- Module: Dictionary entries table and related indexes.
-- ============================================================================

CREATE TABLE IF NOT EXISTS dicts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    dict_type TEXT NOT NULL,
    label TEXT NOT NULL,
    value TEXT NOT NULL,
    status INTEGER NOT NULL DEFAULT 1 CHECK (status IN (1, 2)),
    description TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_dicts_label ON dicts(dict_type, label) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_dicts_status ON dicts(status) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_dicts_deleted_at ON dicts(deleted_at);
CREATE INDEX IF NOT EXISTS idx_dicts_dict_type ON dicts(dict_type) WHERE deleted_at IS NULL;
