ALTER TABLE workflow_jobs
    ADD COLUMN IF NOT EXISTS runner_id uuid,
    ADD COLUMN IF NOT EXISTS assigned_at timestamptz;

CREATE TABLE actions_runners (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid REFERENCES repositories(id) ON DELETE CASCADE,
    name text NOT NULL,
    labels jsonb NOT NULL DEFAULT '[]'::jsonb,
    status text NOT NULL DEFAULT 'offline',
    last_heartbeat timestamptz,
    busy_since timestamptz,
    registration_token text NOT NULL,
    created_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT actions_runners_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT actions_runners_labels_array CHECK (jsonb_typeof(labels) = 'array'),
    CONSTRAINT actions_runners_status_check CHECK (status IN ('online', 'offline', 'busy'))
);

CREATE UNIQUE INDEX actions_runners_repository_name_unique
    ON actions_runners (repository_id, lower(name));
CREATE INDEX actions_runners_repository_status_idx
    ON actions_runners (repository_id, status, last_heartbeat DESC);
CREATE INDEX actions_runners_labels_gin_idx ON actions_runners USING gin (labels);

CREATE TRIGGER actions_runners_set_updated_at
BEFORE UPDATE ON actions_runners
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE actions_runner_settings (
    repository_id uuid PRIMARY KEY REFERENCES repositories(id) ON DELETE CASCADE,
    concurrency_limit integer NOT NULL DEFAULT 4,
    cancel_in_progress boolean NOT NULL DEFAULT false,
    updated_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT actions_runner_settings_concurrency_check CHECK (concurrency_limit BETWEEN 1 AND 64)
);

CREATE TRIGGER actions_runner_settings_set_updated_at
BEFORE UPDATE ON actions_runner_settings
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE workflow_job_assignments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id uuid NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    job_id uuid NOT NULL REFERENCES workflow_jobs(id) ON DELETE CASCADE,
    job_name text NOT NULL,
    runner_id uuid NOT NULL REFERENCES actions_runners(id) ON DELETE CASCADE,
    started_at timestamptz NOT NULL DEFAULT now(),
    status text NOT NULL DEFAULT 'in_progress',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT workflow_job_assignments_status_check CHECK (status IN ('in_progress', 'completed', 'cancelled', 'failed', 'timed_out'))
);

CREATE UNIQUE INDEX workflow_job_assignments_job_unique ON workflow_job_assignments (job_id);
CREATE INDEX workflow_job_assignments_runner_status_idx
    ON workflow_job_assignments (runner_id, status, started_at DESC);

CREATE TRIGGER workflow_job_assignments_set_updated_at
BEFORE UPDATE ON workflow_job_assignments
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE workflow_jobs
    ADD CONSTRAINT workflow_jobs_runner_id_fk
    FOREIGN KEY (runner_id) REFERENCES actions_runners(id) ON DELETE SET NULL;
