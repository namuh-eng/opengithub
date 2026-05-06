CREATE TABLE check_runs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    workflow_run_id uuid REFERENCES workflow_runs(id) ON DELETE CASCADE,
    workflow_job_id uuid REFERENCES workflow_jobs(id) ON DELETE SET NULL,
    head_sha text NOT NULL,
    name text NOT NULL,
    status text NOT NULL DEFAULT 'queued',
    conclusion text,
    started_at timestamptz,
    completed_at timestamptz,
    output_title text,
    output_summary text,
    annotations_count bigint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT check_runs_head_sha_not_blank CHECK (length(trim(head_sha)) > 0),
    CONSTRAINT check_runs_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT check_runs_status_check CHECK (status IN ('queued', 'in_progress', 'completed')),
    CONSTRAINT check_runs_conclusion_check CHECK (
        conclusion IS NULL OR conclusion IN ('success', 'failure', 'cancelled', 'neutral', 'skipped', 'timed_out')
    ),
    CONSTRAINT check_runs_completed_has_conclusion CHECK (
        status <> 'completed' OR conclusion IS NOT NULL
    )
);

CREATE UNIQUE INDEX check_runs_repository_job_unique
ON check_runs (repository_id, workflow_job_id)
WHERE workflow_job_id IS NOT NULL;

CREATE INDEX check_runs_repository_head_status_idx
ON check_runs (repository_id, head_sha, status, updated_at DESC);

CREATE TRIGGER check_runs_set_updated_at
BEFORE UPDATE ON check_runs
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE check_annotations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    check_run_id uuid NOT NULL REFERENCES check_runs(id) ON DELETE CASCADE,
    workflow_annotation_id uuid REFERENCES workflow_annotations(id) ON DELETE SET NULL,
    path text,
    start_line integer,
    end_line integer,
    level text NOT NULL,
    message text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT check_annotations_level_check CHECK (level IN ('notice', 'warning', 'failure')),
    CONSTRAINT check_annotations_message_not_blank CHECK (length(trim(message)) > 0),
    CONSTRAINT check_annotations_start_line_positive CHECK (start_line IS NULL OR start_line > 0),
    CONSTRAINT check_annotations_end_line_positive CHECK (end_line IS NULL OR end_line > 0)
);

CREATE UNIQUE INDEX check_annotations_workflow_annotation_unique
ON check_annotations (check_run_id, workflow_annotation_id)
WHERE workflow_annotation_id IS NOT NULL;

CREATE INDEX check_annotations_check_run_created_idx
ON check_annotations (check_run_id, created_at);
