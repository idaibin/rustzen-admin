-- ============================================================================
-- File Number: 109003
-- File Name: seed_dict.sql
-- Module: Seed Data
-- Description: Seed initial dictionary data (example types and entries). Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

INSERT INTO dicts (type, key, value, status, sort_order)
VALUES
    ('user_status', 'active', 'Active', 1, 1),
    ('user_status', 'disabled', 'Disabled', 1, 2),
    ('user_status', 'pending', 'Pending', 1, 3),
    ('role_type', 'system', 'System Role', 1, 1),
    ('role_type', 'custom', 'Custom Role', 1, 2)
ON CONFLICT DO NOTHING;
