DROP INDEX IF EXISTS notifications_user_saved_updated_idx;

ALTER TABLE notifications
DROP COLUMN IF EXISTS saved_at,
DROP COLUMN IF EXISTS saved;
