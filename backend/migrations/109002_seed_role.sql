-- ============================================================================
-- File Number: 109002
-- File Name: seed_role.sql
-- Module: Seed Data
-- Description: Seed initial roles (system admin, user manager, auditor). Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

INSERT INTO roles (role_name, role_code, description, status, is_system, sort_order)
VALUES
    ('System Administrator', 'SYSTEM_ADMIN', 'System administrator with full access to all system functions', 1, TRUE, 1),
    ('User Manager', 'USER_MANAGER', 'Manages users, roles, and basic system configuration', 1, TRUE, 2),
    ('Auditor', 'AUDITOR', 'Read-only access for auditing and monitoring', 1, TRUE, 3)
ON CONFLICT (role_name) DO NOTHING;
