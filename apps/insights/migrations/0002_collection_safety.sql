ALTER TABLE insights_projects
ADD COLUMN collection_enabled INTEGER NOT NULL DEFAULT 0
CHECK (collection_enabled IN (0, 1));

-- The 0001 hash was a development seed, not an operator-configured public
-- routing identifier. Clear it so first enable must explicitly register a new
-- key through the authenticated collection-policy API.
UPDATE insights_projects
SET project_key_hash = ''
WHERE id = 'default'
  AND project_key_hash = '6ab538c2b9772ed3ea67476cf10035de9a31718833b1ab27c2d28c269f9a5b95';

UPDATE insights_settings
SET max_batch_events = MIN(max_batch_events, 50);
