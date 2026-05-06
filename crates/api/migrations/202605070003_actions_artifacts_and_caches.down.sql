DROP TRIGGER IF EXISTS workflow_dependency_caches_set_updated_at ON workflow_dependency_caches;
DROP TABLE IF EXISTS workflow_dependency_caches;

DROP INDEX IF EXISTS workflow_artifacts_run_active_idx;

ALTER TABLE workflow_artifacts
DROP CONSTRAINT IF EXISTS workflow_artifacts_content_type_not_blank,
DROP CONSTRAINT IF EXISTS workflow_artifacts_storage_kind_not_blank,
DROP CONSTRAINT IF EXISTS workflow_artifacts_retention_days_positive,
DROP COLUMN IF EXISTS deleted_at,
DROP COLUMN IF EXISTS retention_days,
DROP COLUMN IF EXISTS content_type,
DROP COLUMN IF EXISTS storage_kind;
