CREATE TABLE IF NOT EXISTS package_workflow_tokens (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash text NOT NULL,
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    workflow_run_id uuid NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    workflow_job_id uuid REFERENCES workflow_jobs(id) ON DELETE SET NULL,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    scopes text[] NOT NULL DEFAULT ARRAY[]::text[],
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz,
    last_used_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT package_workflow_tokens_hash_not_blank CHECK (length(trim(token_hash)) > 0),
    CONSTRAINT package_workflow_tokens_scopes_not_empty CHECK (array_length(scopes, 1) IS NOT NULL)
);

CREATE UNIQUE INDEX IF NOT EXISTS package_workflow_tokens_hash_unique
ON package_workflow_tokens (token_hash);

CREATE INDEX IF NOT EXISTS package_workflow_tokens_run_active_idx
ON package_workflow_tokens (workflow_run_id, revoked_at, expires_at);

ALTER TABLE package_versions
ADD COLUMN IF NOT EXISTS source_repository_id uuid REFERENCES repositories(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS workflow_run_id uuid REFERENCES workflow_runs(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS workflow_job_id uuid REFERENCES workflow_jobs(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS oci_annotations jsonb NOT NULL DEFAULT '{}'::jsonb;

CREATE INDEX IF NOT EXISTS package_versions_workflow_run_idx
ON package_versions (workflow_run_id)
WHERE workflow_run_id IS NOT NULL;

ALTER TABLE package_registry_audit_events
ADD COLUMN IF NOT EXISTS actor_kind text NOT NULL DEFAULT 'pat',
ADD COLUMN IF NOT EXISTS repository_id uuid REFERENCES repositories(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS workflow_run_id uuid REFERENCES workflow_runs(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS workflow_job_id uuid REFERENCES workflow_jobs(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS metadata jsonb NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE package_registry_audit_events
DROP CONSTRAINT IF EXISTS package_registry_audit_actor_kind_check;

ALTER TABLE package_registry_audit_events
ADD CONSTRAINT package_registry_audit_actor_kind_check
CHECK (actor_kind IN ('anonymous', 'pat', 'workflow'));

CREATE UNIQUE INDEX IF NOT EXISTS package_repository_links_package_repo_type_unique
ON package_repository_links (package_id, repository_id, link_type);
