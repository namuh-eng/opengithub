ALTER TABLE workflow_artifacts
ADD COLUMN IF NOT EXISTS storage_kind text NOT NULL DEFAULT 'local_s3_compatible',
ADD COLUMN IF NOT EXISTS content_type text NOT NULL DEFAULT 'application/zip',
ADD COLUMN IF NOT EXISTS retention_days integer NOT NULL DEFAULT 90,
ADD COLUMN IF NOT EXISTS deleted_at timestamptz,
ADD CONSTRAINT workflow_artifacts_retention_days_positive CHECK (retention_days > 0),
ADD CONSTRAINT workflow_artifacts_storage_kind_not_blank CHECK (length(trim(storage_kind)) > 0),
ADD CONSTRAINT workflow_artifacts_content_type_not_blank CHECK (length(trim(content_type)) > 0);

CREATE INDEX IF NOT EXISTS workflow_artifacts_run_active_idx
ON workflow_artifacts (run_id, created_at)
WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS workflow_dependency_caches (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    cache_key text NOT NULL,
    version text NOT NULL,
    scope text NOT NULL DEFAULT 'refs/heads/main',
    storage_key text NOT NULL,
    storage_kind text NOT NULL DEFAULT 'local_s3_compatible',
    size_bytes bigint NOT NULL DEFAULT 0,
    last_used_at timestamptz NOT NULL DEFAULT now(),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz,
    CONSTRAINT workflow_dependency_caches_key_not_blank CHECK (length(trim(cache_key)) > 0),
    CONSTRAINT workflow_dependency_caches_version_not_blank CHECK (length(trim(version)) > 0),
    CONSTRAINT workflow_dependency_caches_scope_not_blank CHECK (length(trim(scope)) > 0),
    CONSTRAINT workflow_dependency_caches_storage_key_not_blank CHECK (length(trim(storage_key)) > 0),
    CONSTRAINT workflow_dependency_caches_size_non_negative CHECK (size_bytes >= 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS workflow_dependency_caches_repo_key_version_active_unique
ON workflow_dependency_caches (repository_id, cache_key, version)
WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS workflow_dependency_caches_repo_last_used_idx
ON workflow_dependency_caches (repository_id, last_used_at)
WHERE deleted_at IS NULL;

CREATE TRIGGER workflow_dependency_caches_set_updated_at
BEFORE UPDATE ON workflow_dependency_caches
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
