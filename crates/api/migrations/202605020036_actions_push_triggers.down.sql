DROP INDEX IF EXISTS workflow_runs_repository_concurrency_active_idx;

ALTER TABLE workflow_runs
DROP CONSTRAINT IF EXISTS workflow_runs_matrix_object,
DROP CONSTRAINT IF EXISTS workflow_runs_event_payload_object,
DROP COLUMN IF EXISTS workflow_matrix,
DROP COLUMN IF EXISTS concurrency_group,
DROP COLUMN IF EXISTS event_payload;
