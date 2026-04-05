-- One-time repair script for databases that already ran older migrations.
-- Run this manually before starting the service on an existing deployment.

ALTER TABLE menus
    ADD COLUMN IF NOT EXISTS parent_code VARCHAR(100);

CREATE INDEX IF NOT EXISTS idx_resources_parent_code
    ON menus(parent_code) WHERE deleted_at IS NULL;

ALTER TABLE menus
    ADD COLUMN IF NOT EXISTS is_manual BOOLEAN DEFAULT TRUE;

COMMENT ON COLUMN menus.parent_code IS 'Parent menu permission code for hierarchy during sync';
COMMENT ON COLUMN menus.is_manual IS 'TRUE when the menu row has been manually maintained and must not be overwritten by sync';

UPDATE menus AS child
SET parent_code = parent.code
FROM menus AS parent
WHERE child.parent_id = parent.id
  AND child.deleted_at IS NULL
  AND parent.deleted_at IS NULL;

UPDATE menus
SET is_manual = FALSE
WHERE is_system = TRUE;
