-- ============================================================================
-- Module: Menu manual flag.
-- ============================================================================

ALTER TABLE menus
    ADD COLUMN IF NOT EXISTS is_manual BOOLEAN DEFAULT TRUE;

COMMENT ON COLUMN menus.is_manual IS 'TRUE when the menu row has been manually maintained and must not be overwritten by sync';

UPDATE menus
SET is_manual = FALSE
WHERE is_system = TRUE;
