CREATE TABLE notification_threads (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid REFERENCES repositories(id) ON DELETE CASCADE,
    repository_key text NOT NULL,
    subject_type text NOT NULL,
    subject_id uuid,
    subject_key text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT notification_threads_subject_type_not_blank CHECK (length(trim(subject_type)) > 0),
    CONSTRAINT notification_threads_subject_key_not_blank CHECK (length(trim(subject_key)) > 0)
);

CREATE UNIQUE INDEX notification_threads_identity_unique
ON notification_threads (repository_key, subject_type, subject_key);

CREATE INDEX notification_threads_repository_idx
ON notification_threads (repository_id);

CREATE TRIGGER notification_threads_set_updated_at
BEFORE UPDATE ON notification_threads
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE notification_subscriptions (
    thread_id uuid NOT NULL REFERENCES notification_threads(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    state text NOT NULL DEFAULT 'subscribed',
    reason text NOT NULL DEFAULT 'subscribed',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (thread_id, user_id),
    CONSTRAINT notification_subscriptions_state_check CHECK (state IN ('subscribed', 'unsubscribed', 'participating')),
    CONSTRAINT notification_subscriptions_reason_not_blank CHECK (length(trim(reason)) > 0)
);

CREATE INDEX notification_subscriptions_user_state_idx
ON notification_subscriptions (user_id, state, updated_at DESC);

CREATE TRIGGER notification_subscriptions_set_updated_at
BEFORE UPDATE ON notification_subscriptions
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE notifications
ADD COLUMN IF NOT EXISTS thread_id uuid REFERENCES notification_threads(id) ON DELETE SET NULL;

INSERT INTO notification_threads (repository_id, repository_key, subject_type, subject_id, subject_key)
SELECT DISTINCT
       repository_id,
       COALESCE(repository_id::text, 'global') AS repository_key,
       subject_type,
       subject_id,
       COALESCE(subject_id::text, id::text) AS subject_key
FROM notifications
ON CONFLICT (repository_key, subject_type, subject_key) DO NOTHING;

UPDATE notifications
SET thread_id = notification_threads.id
FROM notification_threads
WHERE notification_threads.repository_key = COALESCE(notifications.repository_id::text, 'global')
  AND notification_threads.subject_type = notifications.subject_type
  AND notification_threads.subject_key = COALESCE(notifications.subject_id::text, notifications.id::text)
  AND notifications.thread_id IS NULL;

INSERT INTO notification_subscriptions (thread_id, user_id, state, reason)
SELECT DISTINCT notifications.thread_id, notifications.user_id, 'subscribed', 'repository_watch'
FROM notifications
JOIN repository_watches
  ON repository_watches.user_id = notifications.user_id
 AND repository_watches.repository_id = notifications.repository_id
WHERE notifications.thread_id IS NOT NULL
ON CONFLICT (thread_id, user_id) DO NOTHING;

CREATE INDEX notifications_thread_user_idx
ON notifications (thread_id, user_id);
