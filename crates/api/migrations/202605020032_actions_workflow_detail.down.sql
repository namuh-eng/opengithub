DROP INDEX IF EXISTS actions_workflows_dispatch_enabled_idx;

ALTER TABLE actions_workflows
DROP CONSTRAINT IF EXISTS actions_workflows_dispatch_inputs_array,
DROP COLUMN IF EXISTS dispatch_enabled,
DROP COLUMN IF EXISTS dispatch_inputs,
DROP COLUMN IF EXISTS yaml_parse_error,
DROP COLUMN IF EXISTS source_branch,
DROP COLUMN IF EXISTS source_sha,
DROP COLUMN IF EXISTS source_blob_id;
