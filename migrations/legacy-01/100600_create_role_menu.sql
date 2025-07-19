-- ============================================================================
-- File Number: 100600
-- File Name: create_role_menu.sql
-- Module: Role-Menu Association
-- Description: Create role_menus association table, indexes, and comments. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE TABLE role_menus (
    id BIGSERIAL PRIMARY KEY, -- Unique association ID
    role_id BIGINT NOT NULL, -- Role ID
    menu_id BIGINT NOT NULL, -- Menu ID
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Association creation timestamp
    UNIQUE(role_id, menu_id)
);

CREATE INDEX idx_role_menus_role_id ON role_menus(role_id);
CREATE INDEX idx_role_menus_menu_id ON role_menus(menu_id);
CREATE INDEX idx_role_menus_roleid_menuid ON role_menus(role_id, menu_id);

COMMENT ON TABLE role_menus IS 'Role-menu association table: maps roles to menus (permissions)';
