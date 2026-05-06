DROP INDEX IF EXISTS sessions_user_active_last_active_idx;

ALTER TABLE sessions
    DROP COLUMN IF EXISTS last_active_at,
    DROP COLUMN IF EXISTS ip_inet,
    DROP COLUMN IF EXISTS user_agent;
