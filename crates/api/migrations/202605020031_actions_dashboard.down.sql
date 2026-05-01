DROP TABLE IF EXISTS actions_recent_views;

DROP INDEX IF EXISTS workflow_runs_actor_created_idx;
DROP INDEX IF EXISTS workflow_runs_repository_branch_created_idx;
DROP INDEX IF EXISTS workflow_runs_repository_event_created_idx;

ALTER TABLE workflow_runs
DROP COLUMN IF EXISTS commit_id,
DROP COLUMN IF EXISTS pull_request_id,
DROP COLUMN IF EXISTS display_title;

ALTER TABLE actions_workflows
DROP COLUMN IF EXISTS pinned_order;
