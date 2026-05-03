DROP INDEX IF EXISTS notifications_user_done_updated_idx;
DROP INDEX IF EXISTS notifications_user_inbox_updated_idx;

ALTER TABLE notifications
DROP COLUMN IF EXISTS done_at;
