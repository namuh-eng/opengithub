CREATE TABLE job_logs (
    job_id uuid PRIMARY KEY REFERENCES workflow_jobs(id) ON DELETE CASCADE,
    run_id uuid NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    job_name text NOT NULL,
    s3_key text NOT NULL,
    bytes_written bigint NOT NULL DEFAULT 0,
    finalized_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT job_logs_job_name_not_blank CHECK (length(trim(job_name)) > 0),
    CONSTRAINT job_logs_s3_key_not_blank CHECK (length(trim(s3_key)) > 0),
    CONSTRAINT job_logs_bytes_written_non_negative CHECK (bytes_written >= 0)
);

CREATE UNIQUE INDEX job_logs_s3_key_unique ON job_logs (s3_key);
CREATE INDEX job_logs_run_idx ON job_logs (run_id, finalized_at);

CREATE TRIGGER job_logs_set_updated_at
BEFORE UPDATE ON job_logs
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
