CREATE TABLE workflow_run_attempts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id uuid NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    attempt_number integer NOT NULL,
    status text NOT NULL DEFAULT 'queued',
    conclusion text,
    triggered_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    trigger_kind text NOT NULL DEFAULT 'initial',
    started_at timestamptz,
    completed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT workflow_run_attempts_number_positive CHECK (attempt_number > 0),
    CONSTRAINT workflow_run_attempts_status_check CHECK (status IN ('queued', 'in_progress', 'completed', 'cancelled')),
    CONSTRAINT workflow_run_attempts_conclusion_check CHECK (
        conclusion IS NULL OR conclusion IN ('success', 'failure', 'cancelled', 'skipped', 'timed_out')
    ),
    CONSTRAINT workflow_run_attempts_trigger_kind_not_blank CHECK (length(trim(trigger_kind)) > 0)
);

CREATE UNIQUE INDEX workflow_run_attempts_run_number_unique
ON workflow_run_attempts (run_id, attempt_number);

CREATE TRIGGER workflow_run_attempts_set_updated_at
BEFORE UPDATE ON workflow_run_attempts
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE workflow_jobs
ADD COLUMN attempt_number integer NOT NULL DEFAULT 1,
ADD COLUMN group_name text,
ADD COLUMN log_storage_key text,
ADD COLUMN log_deleted_at timestamptz;

CREATE INDEX workflow_jobs_run_attempt_idx
ON workflow_jobs (run_id, attempt_number, created_at);

CREATE TABLE workflow_annotations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id uuid NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    job_id uuid REFERENCES workflow_jobs(id) ON DELETE CASCADE,
    step_id uuid REFERENCES workflow_steps(id) ON DELETE SET NULL,
    annotation_level text NOT NULL,
    path text,
    start_line integer,
    end_line integer,
    title text,
    message text NOT NULL,
    raw_details text,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT workflow_annotations_level_check CHECK (annotation_level IN ('notice', 'warning', 'failure')),
    CONSTRAINT workflow_annotations_message_not_blank CHECK (length(trim(message)) > 0),
    CONSTRAINT workflow_annotations_start_line_positive CHECK (start_line IS NULL OR start_line > 0),
    CONSTRAINT workflow_annotations_end_line_positive CHECK (end_line IS NULL OR end_line > 0)
);

CREATE INDEX workflow_annotations_run_created_idx
ON workflow_annotations (run_id, created_at);

CREATE INDEX workflow_annotations_job_created_idx
ON workflow_annotations (job_id, created_at)
WHERE job_id IS NOT NULL;

CREATE TABLE workflow_artifacts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id uuid NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    name text NOT NULL,
    digest text,
    size_bytes bigint NOT NULL DEFAULT 0,
    storage_key text,
    expired_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT workflow_artifacts_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT workflow_artifacts_size_non_negative CHECK (size_bytes >= 0)
);

CREATE UNIQUE INDEX workflow_artifacts_run_name_unique
ON workflow_artifacts (run_id, lower(name));

CREATE TRIGGER workflow_artifacts_set_updated_at
BEFORE UPDATE ON workflow_artifacts
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
