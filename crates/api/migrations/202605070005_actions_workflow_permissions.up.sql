ALTER TABLE actions_runner_settings
    ADD COLUMN IF NOT EXISTS workflow_token_permission text NOT NULL DEFAULT 'read',
    ADD COLUMN IF NOT EXISTS allow_actions_approve_pull_requests boolean NOT NULL DEFAULT false,
    ADD CONSTRAINT actions_runner_settings_token_permission_check
        CHECK (workflow_token_permission IN ('read', 'write'));

CREATE TABLE actions_environments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name text NOT NULL,
    protection_rules_enabled boolean NOT NULL DEFAULT false,
    required_reviewers jsonb NOT NULL DEFAULT '[]'::jsonb,
    deployment_branch_policy jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT actions_environments_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT actions_environments_reviewers_array CHECK (jsonb_typeof(required_reviewers) = 'array'),
    CONSTRAINT actions_environments_branch_policy_object CHECK (jsonb_typeof(deployment_branch_policy) = 'object')
);

CREATE UNIQUE INDEX actions_environments_repository_name_unique
    ON actions_environments (repository_id, lower(name));

CREATE TRIGGER actions_environments_set_updated_at
BEFORE UPDATE ON actions_environments
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
