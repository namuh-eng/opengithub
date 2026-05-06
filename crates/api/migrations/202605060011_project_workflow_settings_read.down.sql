DROP INDEX IF EXISTS workflow_execution_logs_workflow_created_idx;
DROP INDEX IF EXISTS workflow_execution_logs_project_created_idx;
DROP TABLE IF EXISTS workflow_execution_logs;

DROP INDEX IF EXISTS project_workflow_repository_targets_repository_idx;
DROP INDEX IF EXISTS project_workflow_repository_targets_unique;
DROP TABLE IF EXISTS project_workflow_repository_targets;

DROP TRIGGER IF EXISTS project_workflow_rules_set_updated_at ON project_workflow_rules;
DROP INDEX IF EXISTS project_workflow_rules_workflow_position_idx;
DROP TABLE IF EXISTS project_workflow_rules;

DROP INDEX IF EXISTS project_workflows_project_event_idx;
DROP INDEX IF EXISTS project_workflows_project_key_unique;

ALTER TABLE project_workflows
    DROP CONSTRAINT IF EXISTS project_workflows_last_status_check,
    DROP CONSTRAINT IF EXISTS project_workflows_source_check,
    DROP CONSTRAINT IF EXISTS project_workflows_key_not_blank,
    DROP COLUMN IF EXISTS last_run_message,
    DROP COLUMN IF EXISTS last_run_status,
    DROP COLUMN IF EXISTS last_run_at,
    DROP COLUMN IF EXISTS source,
    DROP COLUMN IF EXISTS actor_label,
    DROP COLUMN IF EXISTS description,
    DROP COLUMN IF EXISTS workflow_key;
