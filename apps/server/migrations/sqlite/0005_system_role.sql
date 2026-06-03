-- ============================================================================
-- Module: Roles table and related indexes.
-- ============================================================================

CREATE TABLE IF NOT EXISTS roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    description TEXT,
    status INTEGER NOT NULL DEFAULT 1 CHECK (status IN (1, 2)),
    is_system INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_roles_name ON roles(name) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_roles_code ON roles(code) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_roles_status ON roles(status) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_roles_deleted_at ON roles(deleted_at);
CREATE INDEX IF NOT EXISTS idx_roles_sort_order ON roles(sort_order) WHERE deleted_at IS NULL;
