ALTER TABLE sessions
    ADD COLUMN IF NOT EXISTS elevated_until timestamptz;

CREATE INDEX IF NOT EXISTS sessions_elevated_until_idx
ON sessions (user_id, elevated_until DESC)
WHERE elevated_until IS NOT NULL AND revoked_at IS NULL;
