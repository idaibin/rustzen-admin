-- ============================================================================
-- File Number: 109000
-- File Name: seed_user.sql
-- Module: Seed Data
-- Description: Seed initial super admin user data. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

INSERT INTO users (username, email, password_hash, real_name, status, is_super_admin)
VALUES (
    'superadmin',
    'superadmin@example.com',
    -- 密码为 "rustzen@123" 的 argon2id hash 示例（请根据实际安全策略替换）
    '$argon2id$v=19$m=19456,t=2,p=1$i2SSaoqEMMwYzJQPXhVHfg$k1Y5bZ/k5SxEoEroG+UFzCW8aKzK1o/DWKKDU34FiPI',
    'Super Administrator',
    1,
    TRUE
)
