CREATE TABLE IF NOT EXISTS repository_label_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    label_id uuid REFERENCES labels(id) ON DELETE SET NULL,
    actor_user_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    event_type text NOT NULL,
    before_state jsonb,
    after_state jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_label_events_type_not_blank CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX IF NOT EXISTS repository_label_events_repo_created_idx
ON repository_label_events (repository_id, created_at DESC);

CREATE INDEX IF NOT EXISTS repository_label_events_label_created_idx
ON repository_label_events (label_id, created_at DESC)
WHERE label_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS labels_repository_name_count_idx
ON labels (repository_id, lower(name), updated_at DESC);

CREATE INDEX IF NOT EXISTS issue_labels_issue_label_idx
ON issue_labels (issue_id, label_id);

CREATE INDEX IF NOT EXISTS discussion_labels_label_id_idx
ON discussion_labels (label_id);
