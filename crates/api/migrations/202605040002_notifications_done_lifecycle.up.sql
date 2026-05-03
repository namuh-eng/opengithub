ALTER TABLE notifications
ADD COLUMN IF NOT EXISTS done_at timestamptz;

CREATE INDEX IF NOT EXISTS notifications_user_inbox_updated_idx
ON notifications (user_id, updated_at DESC)
WHERE done_at IS NULL;

CREATE INDEX IF NOT EXISTS notifications_user_done_updated_idx
ON notifications (user_id, done_at DESC, updated_at DESC)
WHERE done_at IS NOT NULL;
