ALTER TABLE repositories
    ADD COLUMN IF NOT EXISTS issues_enabled boolean NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS projects_enabled boolean NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS wiki_enabled boolean NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS allow_forking boolean NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS web_commit_signoff_required boolean NOT NULL DEFAULT false;

CREATE TABLE IF NOT EXISTS repository_settings_audit_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    actor_user_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    event_type text NOT NULL,
    changed_fields text[] NOT NULL DEFAULT '{}',
    before_state jsonb NOT NULL DEFAULT '{}'::jsonb,
    after_state jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_settings_audit_event_type_not_blank CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX IF NOT EXISTS repository_settings_audit_events_repo_created_idx
ON repository_settings_audit_events (repository_id, created_at DESC);

CREATE INDEX IF NOT EXISTS repository_settings_audit_events_actor_created_idx
ON repository_settings_audit_events (actor_user_id, created_at DESC);
