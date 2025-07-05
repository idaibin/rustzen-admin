-- ============================================================================
-- System Initial Data Seed
-- This file contains initial data for roles, menus, and super admin user
-- Run this after the schema migration to populate basic system data
-- ============================================================================

DO $$
DECLARE
    -- Menu IDs
    system_root_menu_id BIGINT;
    user_mgmt_menu_id BIGINT;
    user_detail_menu_id BIGINT;
    user_create_menu_id BIGINT;
    user_update_menu_id BIGINT;
    user_delete_menu_id BIGINT;
    role_mgmt_menu_id BIGINT;
    role_detail_menu_id BIGINT;
    role_create_menu_id BIGINT;
    role_update_menu_id BIGINT;
    role_delete_menu_id BIGINT;
    menu_mgmt_menu_id BIGINT;
    menu_detail_menu_id BIGINT;
    menu_create_menu_id BIGINT;
    menu_update_menu_id BIGINT;
    menu_delete_menu_id BIGINT;
    dict_mgmt_menu_id BIGINT;
    dict_detail_menu_id BIGINT;
    dict_create_menu_id BIGINT;
    dict_update_menu_id BIGINT;
    dict_delete_menu_id BIGINT;
    log_mgmt_menu_id BIGINT;
    log_detail_menu_id BIGINT;
    
    -- Role IDs
    system_admin_role_id BIGINT;
    user_manager_role_id BIGINT;
    auditor_role_id BIGINT;
    
    -- User IDs
    superadmin_user_id BIGINT;
BEGIN
    -- Check if seed data already exists
    IF (SELECT COUNT(*) FROM users WHERE username = 'superadmin') > 0 THEN
        RAISE NOTICE 'Seed data already exists, skipping initialization';
        RETURN;
    END IF;

    RAISE NOTICE 'Starting system data initialization...';

    -- ========================================================================
    -- Create Menu Structure
    -- ========================================================================
    
    -- System Management (Root Directory)
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (0, 'System Management', '/system', NULL, 'system', 1, 1, 1, TRUE, NULL)
    RETURNING id INTO system_root_menu_id;
    
    -- User Management
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (system_root_menu_id, 'User Management', '/system/users', 'UserList', 'user', 1, 1, 2, TRUE, 'system:user:list')
    RETURNING id INTO user_mgmt_menu_id;
    
    -- User Management Buttons (separate INSERT statements)
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (user_mgmt_menu_id, 'User Detail', NULL, NULL, NULL, 1, 1, 3, TRUE, 'system:user:detail')
    RETURNING id INTO user_detail_menu_id;
    
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (user_mgmt_menu_id, 'Create User', NULL, NULL, NULL, 2, 1, 3, TRUE, 'system:user:create')
    RETURNING id INTO user_create_menu_id;
    
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (user_mgmt_menu_id, 'Update User', NULL, NULL, NULL, 3, 1, 3, TRUE, 'system:user:update')
    RETURNING id INTO user_update_menu_id;
    
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (user_mgmt_menu_id, 'Delete User', NULL, NULL, NULL, 4, 1, 3, TRUE, 'system:user:delete')
    RETURNING id INTO user_delete_menu_id;
    
    -- Role Management
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (system_root_menu_id, 'Role Management', '/system/roles', 'RoleList', 'team', 2, 1, 2, TRUE, 'system:role:list')
    RETURNING id INTO role_mgmt_menu_id;
    
    -- Role Management Buttons (separate INSERT statements)
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (role_mgmt_menu_id, 'Role Detail', NULL, NULL, NULL, 1, 1, 3, TRUE, 'system:role:detail')
    RETURNING id INTO role_detail_menu_id;
    
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (role_mgmt_menu_id, 'Create Role', NULL, NULL, NULL, 2, 1, 3, TRUE, 'system:role:create')
    RETURNING id INTO role_create_menu_id;
    
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (role_mgmt_menu_id, 'Update Role', NULL, NULL, NULL, 3, 1, 3, TRUE, 'system:role:update')
    RETURNING id INTO role_update_menu_id;
    
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (role_mgmt_menu_id, 'Delete Role', NULL, NULL, NULL, 4, 1, 3, TRUE, 'system:role:delete')
    RETURNING id INTO role_delete_menu_id;
    
    -- Menu Management
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (system_root_menu_id, 'Menu Management', '/system/menus', 'MenuList', 'menu', 3, 1, 2, TRUE, 'system:menu:list')
    RETURNING id INTO menu_mgmt_menu_id;
    
    -- Menu Management Buttons (separate INSERT statements)
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (menu_mgmt_menu_id, 'Menu Detail', NULL, NULL, NULL, 1, 1, 3, TRUE, 'system:menu:detail')
    RETURNING id INTO menu_detail_menu_id;
    
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (menu_mgmt_menu_id, 'Create Menu', NULL, NULL, NULL, 2, 1, 3, TRUE, 'system:menu:create')
    RETURNING id INTO menu_create_menu_id;
    
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (menu_mgmt_menu_id, 'Update Menu', NULL, NULL, NULL, 3, 1, 3, TRUE, 'system:menu:update')
    RETURNING id INTO menu_update_menu_id;
    
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (menu_mgmt_menu_id, 'Delete Menu', NULL, NULL, NULL, 4, 1, 3, TRUE, 'system:menu:delete')
    RETURNING id INTO menu_delete_menu_id;
    
    -- Dictionary Management
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (system_root_menu_id, 'Dictionary Management', '/system/dicts', 'DictList', 'book', 4, 1, 2, TRUE, 'system:dict:list')
    RETURNING id INTO dict_mgmt_menu_id;
    
    -- Dictionary Management Buttons (separate INSERT statements)
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (dict_mgmt_menu_id, 'Dictionary Detail', NULL, NULL, NULL, 1, 1, 3, TRUE, 'system:dict:detail')
    RETURNING id INTO dict_detail_menu_id;
    
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (dict_mgmt_menu_id, 'Create Dictionary', NULL, NULL, NULL, 2, 1, 3, TRUE, 'system:dict:create')
    RETURNING id INTO dict_create_menu_id;
    
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (dict_mgmt_menu_id, 'Update Dictionary', NULL, NULL, NULL, 3, 1, 3, TRUE, 'system:dict:update')
    RETURNING id INTO dict_update_menu_id;
    
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (dict_mgmt_menu_id, 'Delete Dictionary', NULL, NULL, NULL, 4, 1, 3, TRUE, 'system:dict:delete')
    RETURNING id INTO dict_delete_menu_id;
    
    -- Operation Logs
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (system_root_menu_id, 'Operation Logs', '/system/logs', 'LogList', 'file-text', 5, 1, 2, TRUE, 'system:log:list')
    RETURNING id INTO log_mgmt_menu_id;
    
    -- Log Management Buttons
    INSERT INTO menus (parent_id, title, path, component, icon, sort_order, status, menu_type, is_system, permission_code)
    VALUES (log_mgmt_menu_id, 'Log Detail', NULL, NULL, NULL, 1, 1, 3, TRUE, 'system:log:detail')
    RETURNING id INTO log_detail_menu_id;
    
    -- ========================================================================
    -- Create Roles
    -- ========================================================================
    
    -- System Administrator (Full Access)
    INSERT INTO roles (role_name, role_code, description, status, is_system, sort_order)
    VALUES ('System Administrator', 'SYSTEM_ADMIN', 'System administrator with full access to all system functions', 1, TRUE, 1)
    RETURNING id INTO system_admin_role_id;
    
    -- User Manager (User and Role Management)
    INSERT INTO roles (role_name, role_code, description, status, is_system, sort_order)
    VALUES ('User Manager', 'USER_MANAGER', 'Manages users, roles, and basic system configuration', 1, TRUE, 2)
    RETURNING id INTO user_manager_role_id;
    
    -- Auditor (Read-only Access)
    INSERT INTO roles (role_name, role_code, description, status, is_system, sort_order)
    VALUES ('Auditor', 'AUDITOR', 'Read-only access for auditing and monitoring', 1, TRUE, 3)
    RETURNING id INTO auditor_role_id;
    
    -- ========================================================================
    -- Create Super Admin User
    -- ========================================================================
    
    INSERT INTO users (username, email, password_hash, real_name, status, is_super_admin)
    VALUES ('superadmin', 'superadmin@example.com', '$argon2id$v=19$m=19456,t=2,p=1$i2SSaoqEMMwYzJQPXhVHfg$k1Y5bZ/k5SxEoEroG+UFzCW8aKzK1o/DWKKDU34FiPI', 'Super Administrator', 1, TRUE)
    RETURNING id INTO superadmin_user_id;
    
    -- ========================================================================
    -- Role-Menu Associations
    -- ========================================================================
    
    -- System Administrator: All permissions
    INSERT INTO role_menus (role_id, menu_id) 
    SELECT system_admin_role_id, id FROM menus WHERE is_system = TRUE AND menu_type IN (2, 3);
    
    -- User Manager: User, role, and dictionary management
    INSERT INTO role_menus (role_id, menu_id) VALUES
    (user_manager_role_id, user_mgmt_menu_id),
    (user_manager_role_id, user_detail_menu_id),
    (user_manager_role_id, user_create_menu_id),
    (user_manager_role_id, user_update_menu_id),
    (user_manager_role_id, user_delete_menu_id),
    (user_manager_role_id, role_mgmt_menu_id),
    (user_manager_role_id, role_detail_menu_id),
    (user_manager_role_id, role_create_menu_id),
    (user_manager_role_id, role_update_menu_id),
    (user_manager_role_id, role_delete_menu_id),
    (user_manager_role_id, dict_mgmt_menu_id),
    (user_manager_role_id, dict_detail_menu_id),
    (user_manager_role_id, dict_create_menu_id),
    (user_manager_role_id, dict_update_menu_id),
    (user_manager_role_id, dict_delete_menu_id);
    
    -- Auditor: Read-only access
    INSERT INTO role_menus (role_id, menu_id) VALUES
    (auditor_role_id, user_mgmt_menu_id),
    (auditor_role_id, user_detail_menu_id),
    (auditor_role_id, role_mgmt_menu_id),
    (auditor_role_id, role_detail_menu_id),
    (auditor_role_id, dict_mgmt_menu_id),
    (auditor_role_id, dict_detail_menu_id),
    (auditor_role_id, log_mgmt_menu_id),
    (auditor_role_id, log_detail_menu_id);
    
    -- ========================================================================
    -- User-Role Associations
    -- ========================================================================
    
    -- Assign System Administrator role to superadmin user
    INSERT INTO user_roles (user_id, role_id)
    VALUES (superadmin_user_id, system_admin_role_id);
    
    -- ========================================================================
    -- Output initialization summary
    -- ========================================================================
    
    RAISE NOTICE 'System initialization completed successfully!';
    RAISE NOTICE 'Created % menus, % roles, and 1 super admin user', 
        (SELECT COUNT(*) FROM menus WHERE is_system = TRUE),
        (SELECT COUNT(*) FROM roles WHERE is_system = TRUE);
    RAISE NOTICE 'Super admin credentials: username=superadmin, password=rustzen@123';
    
END $$;
