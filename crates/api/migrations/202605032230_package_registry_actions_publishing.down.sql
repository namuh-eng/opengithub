DROP INDEX IF EXISTS package_repository_links_package_repo_type_unique;

ALTER TABLE package_registry_audit_events
DROP CONSTRAINT IF EXISTS package_registry_audit_actor_kind_check;

ALTER TABLE package_registry_audit_events
DROP COLUMN IF EXISTS metadata,
DROP COLUMN IF EXISTS workflow_job_id,
DROP COLUMN IF EXISTS workflow_run_id,
DROP COLUMN IF EXISTS repository_id,
DROP COLUMN IF EXISTS actor_kind;

DROP INDEX IF EXISTS package_versions_workflow_run_idx;

ALTER TABLE package_versions
DROP COLUMN IF EXISTS oci_annotations,
DROP COLUMN IF EXISTS workflow_job_id,
DROP COLUMN IF EXISTS workflow_run_id,
DROP COLUMN IF EXISTS source_repository_id;

DROP INDEX IF EXISTS package_workflow_tokens_run_active_idx;
DROP INDEX IF EXISTS package_workflow_tokens_hash_unique;
DROP TABLE IF EXISTS package_workflow_tokens;
