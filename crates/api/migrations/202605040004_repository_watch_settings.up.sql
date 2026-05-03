ALTER TABLE repository_watches
    DROP CONSTRAINT IF EXISTS repository_watches_reason_check;

ALTER TABLE repository_watches
    ADD COLUMN level text NOT NULL DEFAULT 'participating',
    ADD COLUMN custom_events jsonb NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN ignored_at timestamptz,
    ADD COLUMN updated_at timestamptz NOT NULL DEFAULT now(),
    ADD CONSTRAINT repository_watches_reason_check CHECK (reason IN ('subscribed', 'participating', 'all', 'ignore', 'custom')),
    ADD CONSTRAINT repository_watches_level_check CHECK (level IN ('participating', 'all', 'ignore', 'custom')),
    ADD CONSTRAINT repository_watches_custom_events_array CHECK (jsonb_typeof(custom_events) = 'array');

UPDATE repository_watches
SET level = CASE
        WHEN reason = 'subscribed' THEN 'participating'
        WHEN reason = 'participating' THEN 'participating'
        ELSE reason
    END,
    ignored_at = CASE WHEN reason = 'ignore' THEN COALESCE(ignored_at, updated_at, created_at) ELSE ignored_at END,
    updated_at = now();

CREATE INDEX repository_watches_active_repository_idx
ON repository_watches (repository_id)
WHERE level <> 'ignore';

CREATE TRIGGER repository_watches_set_updated_at
BEFORE UPDATE ON repository_watches
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
