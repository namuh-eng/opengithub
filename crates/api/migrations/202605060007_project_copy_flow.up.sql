CREATE TABLE IF NOT EXISTS project_recent_visits (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason text NOT NULL DEFAULT 'view',
    viewed_at timestamptz NOT NULL DEFAULT now(),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT project_recent_visits_reason_not_blank CHECK (length(trim(reason)) > 0),
    CONSTRAINT project_recent_visits_metadata_object CHECK (jsonb_typeof(metadata) = 'object')
);

CREATE UNIQUE INDEX IF NOT EXISTS project_recent_visits_project_user_reason_unique
ON project_recent_visits (project_id, user_id, reason);

CREATE INDEX IF NOT EXISTS project_recent_visits_user_viewed_idx
ON project_recent_visits (user_id, viewed_at DESC);
