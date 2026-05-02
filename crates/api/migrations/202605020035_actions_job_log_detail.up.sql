ALTER TABLE workflow_job_log_lines
ADD COLUMN step_id uuid REFERENCES workflow_steps(id) ON DELETE SET NULL;

CREATE INDEX workflow_job_log_lines_step_number_idx
ON workflow_job_log_lines (step_id, line_number)
WHERE step_id IS NOT NULL;

CREATE TABLE actions_log_preferences (
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    show_timestamps boolean NOT NULL DEFAULT true,
    raw_logs boolean NOT NULL DEFAULT false,
    wrap_lines boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (repository_id, user_id)
);

CREATE TRIGGER actions_log_preferences_set_updated_at
BEFORE UPDATE ON actions_log_preferences
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
