DROP TABLE IF EXISTS actions_log_preferences;

DROP INDEX IF EXISTS workflow_job_log_lines_step_number_idx;

ALTER TABLE workflow_job_log_lines
DROP COLUMN IF EXISTS step_id;
