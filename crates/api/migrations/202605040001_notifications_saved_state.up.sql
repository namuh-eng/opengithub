ALTER TABLE notifications
ADD COLUMN IF NOT EXISTS saved boolean NOT NULL DEFAULT false,
ADD COLUMN IF NOT EXISTS saved_at timestamptz;

CREATE INDEX IF NOT EXISTS notifications_user_saved_updated_idx
ON notifications (user_id, saved, updated_at DESC);
