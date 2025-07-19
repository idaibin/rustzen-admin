-- ============================================================================
-- File Number: 100700
-- File Name: create_dict.sql
-- Module: Dictionary
-- Description: Create dicts table, indexes, and comments. Zen migration style.
-- Author: Bruce Dai
-- Date: 2025-07-06
-- ============================================================================

CREATE TABLE dicts (
    id BIGSERIAL PRIMARY KEY, -- Unique dict entry ID
    type VARCHAR(50) NOT NULL, -- Dictionary type/category
    key VARCHAR(100) NOT NULL, -- Dictionary key
    value VARCHAR(255) NOT NULL, -- Dictionary value
    status SMALLINT DEFAULT 1 CHECK (status IN (1, 2)), -- 1: active, 2: inactive
    sort_order INTEGER DEFAULT 0, -- Sort order
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Creation timestamp
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, -- Last update timestamp
    deleted_at TIMESTAMP -- Soft delete timestamp
);

CREATE UNIQUE INDEX idx_dicts_type_key ON dicts(type, key) WHERE deleted_at IS NULL;
CREATE INDEX idx_dicts_status ON dicts(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_dicts_deleted_at ON dicts(deleted_at);
CREATE INDEX idx_dicts_type ON dicts(type) WHERE deleted_at IS NULL;

COMMENT ON TABLE dicts IS 'Dictionary table for key-value pairs and types';
COMMENT ON COLUMN dicts.type IS 'Dictionary type/category';
COMMENT ON COLUMN dicts.key IS 'Dictionary key, unique within type';
COMMENT ON COLUMN dicts.value IS 'Dictionary value';
COMMENT ON COLUMN dicts.status IS 'Dictionary entry status: 1=active, 2=inactive';
COMMENT ON COLUMN dicts.sort_order IS 'Sort order for display';
COMMENT ON COLUMN dicts.deleted_at IS 'Soft delete timestamp, NULL means not deleted';
