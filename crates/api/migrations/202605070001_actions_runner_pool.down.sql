ALTER TABLE workflow_jobs DROP CONSTRAINT IF EXISTS workflow_jobs_runner_id_fk;
DROP TABLE IF EXISTS workflow_job_assignments;
DROP TABLE IF EXISTS actions_runner_settings;
DROP TABLE IF EXISTS actions_runners;
ALTER TABLE workflow_jobs
    DROP COLUMN IF EXISTS assigned_at,
    DROP COLUMN IF EXISTS runner_id;
