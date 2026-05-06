ALTER TABLE project_workflows
    ADD COLUMN IF NOT EXISTS workflow_key text,
    ADD COLUMN IF NOT EXISTS description text,
    ADD COLUMN IF NOT EXISTS actor_label text NOT NULL DEFAULT '@opengithub-project-automation',
    ADD COLUMN IF NOT EXISTS source text NOT NULL DEFAULT 'system',
    ADD COLUMN IF NOT EXISTS last_run_at timestamptz,
    ADD COLUMN IF NOT EXISTS last_run_status text,
    ADD COLUMN IF NOT EXISTS last_run_message text;

UPDATE project_workflows
SET workflow_key = lower(regexp_replace(trigger_event || '-' || name, '[^a-z0-9]+', '-', 'g'))
WHERE workflow_key IS NULL;

ALTER TABLE project_workflows
    ALTER COLUMN workflow_key SET NOT NULL,
    ADD CONSTRAINT project_workflows_key_not_blank CHECK (length(trim(workflow_key)) > 0),
    ADD CONSTRAINT project_workflows_source_check CHECK (source IN ('system', 'ui', 'actions', 'graphql')),
    ADD CONSTRAINT project_workflows_last_status_check CHECK (
        last_run_status IS NULL OR last_run_status IN ('success', 'skipped', 'failed')
    );

CREATE UNIQUE INDEX IF NOT EXISTS project_workflows_project_key_unique
ON project_workflows (project_id, workflow_key);

CREATE INDEX IF NOT EXISTS project_workflows_project_event_idx
ON project_workflows (project_id, trigger_event, enabled);

CREATE TABLE IF NOT EXISTS project_workflow_rules (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_workflow_id uuid NOT NULL REFERENCES project_workflows(id) ON DELETE CASCADE,
    rule_type text NOT NULL,
    configuration jsonb NOT NULL DEFAULT '{}'::jsonb,
    position integer NOT NULL DEFAULT 1,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_workflow_rules_type_not_blank CHECK (length(trim(rule_type)) > 0)
);

CREATE INDEX IF NOT EXISTS project_workflow_rules_workflow_position_idx
ON project_workflow_rules (project_workflow_id, position);

CREATE TRIGGER project_workflow_rules_set_updated_at
BEFORE UPDATE ON project_workflow_rules
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS project_workflow_repository_targets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_workflow_id uuid NOT NULL REFERENCES project_workflows(id) ON DELETE CASCADE,
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS project_workflow_repository_targets_unique
ON project_workflow_repository_targets (project_workflow_id, repository_id);

CREATE INDEX IF NOT EXISTS project_workflow_repository_targets_repository_idx
ON project_workflow_repository_targets (repository_id, project_workflow_id);

CREATE TABLE IF NOT EXISTS workflow_execution_logs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    project_workflow_id uuid REFERENCES project_workflows(id) ON DELETE SET NULL,
    project_item_id uuid REFERENCES project_items(id) ON DELETE SET NULL,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    source text NOT NULL DEFAULT 'system',
    event_type text NOT NULL,
    status text NOT NULL,
    message text,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT workflow_execution_logs_source_check CHECK (source IN ('system', 'ui', 'actions', 'graphql')),
    CONSTRAINT workflow_execution_logs_status_check CHECK (status IN ('success', 'skipped', 'failed')),
    CONSTRAINT workflow_execution_logs_event_not_blank CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX IF NOT EXISTS workflow_execution_logs_project_created_idx
ON workflow_execution_logs (project_id, created_at DESC);

CREATE INDEX IF NOT EXISTS workflow_execution_logs_workflow_created_idx
ON workflow_execution_logs (project_workflow_id, created_at DESC);
