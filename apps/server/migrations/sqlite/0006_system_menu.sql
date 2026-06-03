-- ============================================================================
-- Module: Menus table and related indexes.
-- ============================================================================

CREATE TABLE IF NOT EXISTS menus (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER NOT NULL DEFAULT 0,
    parent_code TEXT,
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    menu_type INTEGER NOT NULL DEFAULT 2 CHECK (menu_type IN (1, 2, 3)),
    status INTEGER NOT NULL DEFAULT 1 CHECK (status IN (1, 2)),
    is_system INTEGER NOT NULL DEFAULT 0,
    is_manual INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at DATETIME
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_menus_name ON menus(name) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_menus_code ON menus(code) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_resources_parent_id ON menus(parent_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_resources_sort_order ON menus(sort_order) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_resources_status ON menus(status) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_resources_deleted_at ON menus(deleted_at);
CREATE INDEX IF NOT EXISTS idx_resources_parent_sort ON menus(parent_id, sort_order) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_resources_menu_type ON menus(menu_type) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_resources_is_system ON menus(is_system) WHERE is_system = 1 AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_resources_parent_code ON menus(parent_code) WHERE deleted_at IS NULL;
