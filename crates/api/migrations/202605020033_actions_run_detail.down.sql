DROP TABLE IF EXISTS workflow_artifacts;
DROP TABLE IF EXISTS workflow_annotations;

DROP INDEX IF EXISTS workflow_jobs_run_attempt_idx;

ALTER TABLE workflow_jobs
DROP COLUMN IF EXISTS attempt_number,
DROP COLUMN IF EXISTS group_name,
DROP COLUMN IF EXISTS log_storage_key,
DROP COLUMN IF EXISTS log_deleted_at;

DROP TABLE IF EXISTS workflow_run_attempts;
