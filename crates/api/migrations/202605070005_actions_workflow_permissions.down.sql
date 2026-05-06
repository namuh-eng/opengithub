DROP TRIGGER IF EXISTS actions_environments_set_updated_at ON actions_environments;
DROP INDEX IF EXISTS actions_environments_repository_name_unique;
DROP TABLE IF EXISTS actions_environments;

ALTER TABLE actions_runner_settings
    DROP CONSTRAINT IF EXISTS actions_runner_settings_token_permission_check,
    DROP COLUMN IF EXISTS allow_actions_approve_pull_requests,
    DROP COLUMN IF EXISTS workflow_token_permission;
