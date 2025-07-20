-- ============================================================================
-- Module: user-role association
-- ============================================================================

CREATE TABLE user_roles (
    user_id BIGINT NOT NULL, -- User ID
    role_id BIGINT NOT NULL, -- Role ID
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Association creation timestamp
    UNIQUE(user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);

CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);

COMMENT ON TABLE user_roles IS 'User-role association table: maps users to roles';

-- ============================================================================
-- Module: role-resource association
-- ============================================================================

CREATE TABLE role_menus (
    role_id BIGINT NOT NULL, -- Role ID
    menu_id BIGINT NOT NULL, -- Menu ID
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Association creation timestamp
    UNIQUE(role_id, menu_id),
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (menu_id) REFERENCES menus(id) ON DELETE CASCADE
);

CREATE INDEX idx_role_menus_role_id ON role_menus(role_id);
CREATE INDEX idx_role_menus_menu_id ON role_menus(menu_id);

COMMENT ON TABLE role_menus IS 'Role-menu association table: maps roles to menus (permissions)';
