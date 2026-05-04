DROP INDEX IF EXISTS sessions_elevated_until_idx;

ALTER TABLE sessions
    DROP COLUMN IF EXISTS elevated_until;
