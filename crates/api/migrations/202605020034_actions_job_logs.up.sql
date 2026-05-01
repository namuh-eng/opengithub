CREATE TABLE workflow_job_log_lines (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id uuid NOT NULL REFERENCES workflow_jobs(id) ON DELETE CASCADE,
    line_number integer NOT NULL,
    timestamp timestamptz,
    content text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT workflow_job_log_lines_number_positive CHECK (line_number > 0)
);

CREATE UNIQUE INDEX workflow_job_log_lines_job_number_unique
ON workflow_job_log_lines (job_id, line_number);

CREATE INDEX workflow_job_log_lines_job_content_trgm
ON workflow_job_log_lines USING gin (content gin_trgm_ops);
