-- ============================================================================
-- Module: Permission menu parent_code backfill.
-- ============================================================================

ALTER TABLE menus
    ADD COLUMN IF NOT EXISTS parent_code VARCHAR(100);

CREATE INDEX IF NOT EXISTS idx_resources_parent_code
    ON menus(parent_code) WHERE deleted_at IS NULL;

UPDATE menus AS child
SET parent_code = parent.code
FROM menus AS parent
WHERE child.parent_id = parent.id
  AND child.deleted_at IS NULL
  AND parent.deleted_at IS NULL;

