ALTER TABLE sessions
    ADD COLUMN IF NOT EXISTS user_agent text,
    ADD COLUMN IF NOT EXISTS ip_inet inet,
    ADD COLUMN IF NOT EXISTS last_active_at timestamptz NOT NULL DEFAULT now();

UPDATE sessions
SET last_active_at = last_seen_at
WHERE last_active_at IS NULL;

CREATE INDEX IF NOT EXISTS sessions_user_active_last_active_idx
ON sessions (user_id, last_active_at DESC)
WHERE revoked_at IS NULL;
