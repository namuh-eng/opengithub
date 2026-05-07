DROP TRIGGER IF EXISTS repository_watches_set_updated_at ON repository_watches;

DROP INDEX IF EXISTS repository_watches_active_repository_idx;

ALTER TABLE repository_watches
    DROP CONSTRAINT IF EXISTS repository_watches_custom_events_array,
    DROP CONSTRAINT IF EXISTS repository_watches_level_check,
    DROP CONSTRAINT IF EXISTS repository_watches_reason_check;

UPDATE repository_watches
SET reason = CASE
        WHEN level = 'participating' THEN 'subscribed'
        ELSE 'participating'
    END;

ALTER TABLE repository_watches
    DROP COLUMN IF EXISTS updated_at,
    DROP COLUMN IF EXISTS ignored_at,
    DROP COLUMN IF EXISTS custom_events,
    DROP COLUMN IF EXISTS level,
    ADD CONSTRAINT repository_watches_reason_check CHECK (reason IN ('subscribed', 'participating'));
