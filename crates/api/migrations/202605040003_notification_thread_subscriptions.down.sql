DROP INDEX IF EXISTS notifications_thread_user_idx;

ALTER TABLE notifications
DROP COLUMN IF EXISTS thread_id;

DROP TABLE IF EXISTS notification_subscriptions;
DROP TABLE IF EXISTS notification_threads;
