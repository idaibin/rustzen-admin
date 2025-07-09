-- ============================================================================
-- File Number: 100300
-- File Name: create_menu.sql
-- Module: Menu Management
-- Description: Create menus table, indexes, and comments. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE TABLE menus (
    id BIGSERIAL PRIMARY KEY, -- Unique menu ID
    parent_id BIGINT DEFAULT 0, -- Parent menu ID (0 for root)
    title VARCHAR(100) NOT NULL, -- Menu title
    path VARCHAR(255), -- Route path
    component VARCHAR(100), -- Frontend component name
    icon VARCHAR(100), -- Menu icon
    sort_order INTEGER DEFAULT 0, -- Sort order
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2)), -- 1: visible, 2: hidden
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Creation timestamp
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Last update timestamp
    deleted_at TIMESTAMP, -- Soft delete timestamp
    menu_type SMALLINT DEFAULT 2 CHECK (menu_type IN (1, 2, 3)), -- 1=directory, 2=menu, 3=button
    is_system BOOLEAN DEFAULT FALSE, -- System built-in menu flag
    meta_data JSONB, -- Extended metadata
    permission_code VARCHAR(100) UNIQUE -- Unique permission code for menu/button
);

CREATE INDEX idx_menus_parent_id ON menus(parent_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_sort_order ON menus(sort_order) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_status ON menus(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_deleted_at ON menus(deleted_at);
CREATE INDEX idx_menus_parent_sort ON menus(parent_id, sort_order) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_menu_type ON menus(menu_type) WHERE deleted_at IS NULL;
CREATE INDEX idx_menus_is_system ON menus(is_system) WHERE is_system = TRUE AND deleted_at IS NULL;

COMMENT ON TABLE menus IS 'Menus table: stores menu and permission definitions';
COMMENT ON COLUMN menus.title IS 'Menu title';
COMMENT ON COLUMN menus.path IS 'Route path';
COMMENT ON COLUMN menus.component IS 'Frontend component name';
COMMENT ON COLUMN menus.status IS 'Menu status: 1=visible, 2=hidden';
COMMENT ON COLUMN menus.deleted_at IS 'Soft delete timestamp, NULL means not deleted';
COMMENT ON COLUMN menus.menu_type IS 'Menu type: 1=directory, 2=menu, 3=button';
COMMENT ON COLUMN menus.is_system IS 'System built-in menu flag';
COMMENT ON COLUMN menus.meta_data IS 'Extended metadata in JSON format';
