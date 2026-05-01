ALTER TABLE actions_workflows
ADD COLUMN source_blob_id uuid REFERENCES git_objects(id) ON DELETE SET NULL,
ADD COLUMN source_sha text,
ADD COLUMN source_branch text,
ADD COLUMN yaml_parse_error text,
ADD COLUMN dispatch_inputs jsonb NOT NULL DEFAULT '[]'::jsonb,
ADD COLUMN dispatch_enabled boolean NOT NULL DEFAULT false,
ADD CONSTRAINT actions_workflows_dispatch_inputs_array CHECK (jsonb_typeof(dispatch_inputs) = 'array');

CREATE INDEX actions_workflows_dispatch_enabled_idx
ON actions_workflows (repository_id, dispatch_enabled);
