ALTER TABLE workflow_runs
ADD COLUMN event_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
ADD COLUMN concurrency_group text,
ADD COLUMN workflow_matrix jsonb NOT NULL DEFAULT '{}'::jsonb,
ADD CONSTRAINT workflow_runs_event_payload_object CHECK (jsonb_typeof(event_payload) = 'object'),
ADD CONSTRAINT workflow_runs_matrix_object CHECK (jsonb_typeof(workflow_matrix) = 'object');

CREATE INDEX workflow_runs_repository_concurrency_active_idx
ON workflow_runs (repository_id, concurrency_group)
WHERE concurrency_group IS NOT NULL
  AND status IN ('queued', 'in_progress');
