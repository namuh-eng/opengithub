CREATE TABLE IF NOT EXISTS code_scanning_runs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    tool_name text NOT NULL,
    tool_version text,
    category text,
    ref_name text NOT NULL,
    commit_oid text,
    status text NOT NULL DEFAULT 'completed',
    source text NOT NULL DEFAULT 'sarif',
    started_at timestamptz,
    completed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT code_scanning_runs_tool_name_not_blank CHECK (length(trim(tool_name)) > 0),
    CONSTRAINT code_scanning_runs_ref_name_not_blank CHECK (length(trim(ref_name)) > 0),
    CONSTRAINT code_scanning_runs_status_check CHECK (status IN ('queued', 'processing', 'completed', 'failed'))
);

CREATE INDEX IF NOT EXISTS code_scanning_runs_repository_created_idx
ON code_scanning_runs (repository_id, created_at DESC);

CREATE TABLE IF NOT EXISTS code_scanning_alerts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    run_id uuid REFERENCES code_scanning_runs(id) ON DELETE SET NULL,
    number bigint NOT NULL,
    state text NOT NULL DEFAULT 'open',
    rule_id text NOT NULL,
    rule_name text NOT NULL,
    rule_description text,
    message text NOT NULL,
    severity text NOT NULL DEFAULT 'warning',
    security_severity text,
    tool_name text NOT NULL,
    ref_name text NOT NULL,
    branch_name text,
    path text NOT NULL,
    start_line integer NOT NULL DEFAULT 1,
    end_line integer,
    fingerprint text NOT NULL,
    code_snippet text,
    help_markdown text,
    help_uri text,
    fixed_at timestamptz,
    dismissed_reason text,
    dismissed_comment text,
    dismissed_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    dismissed_at timestamptz,
    linked_issue_id uuid REFERENCES issues(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT code_scanning_alerts_number_positive CHECK (number > 0),
    CONSTRAINT code_scanning_alerts_line_positive CHECK (start_line > 0 AND (end_line IS NULL OR end_line >= start_line)),
    CONSTRAINT code_scanning_alerts_state_check CHECK (state IN ('open', 'dismissed', 'fixed')),
    CONSTRAINT code_scanning_alerts_rule_id_not_blank CHECK (length(trim(rule_id)) > 0),
    CONSTRAINT code_scanning_alerts_message_not_blank CHECK (length(trim(message)) > 0),
    CONSTRAINT code_scanning_alerts_tool_name_not_blank CHECK (length(trim(tool_name)) > 0),
    CONSTRAINT code_scanning_alerts_path_not_blank CHECK (length(trim(path)) > 0),
    CONSTRAINT code_scanning_alerts_fingerprint_not_blank CHECK (length(trim(fingerprint)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS code_scanning_alerts_repository_number_unique
ON code_scanning_alerts (repository_id, number);

CREATE UNIQUE INDEX IF NOT EXISTS code_scanning_alerts_repository_fingerprint_unique
ON code_scanning_alerts (repository_id, rule_id, path, start_line, fingerprint, ref_name);

CREATE INDEX IF NOT EXISTS code_scanning_alerts_repository_state_updated_idx
ON code_scanning_alerts (repository_id, state, updated_at DESC);

CREATE INDEX IF NOT EXISTS code_scanning_alerts_repository_tool_idx
ON code_scanning_alerts (repository_id, lower(tool_name));

CREATE TABLE IF NOT EXISTS code_scanning_alert_instances (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id uuid NOT NULL REFERENCES code_scanning_alerts(id) ON DELETE CASCADE,
    run_id uuid REFERENCES code_scanning_runs(id) ON DELETE SET NULL,
    ref_name text NOT NULL,
    commit_oid text,
    path text NOT NULL,
    start_line integer NOT NULL DEFAULT 1,
    end_line integer,
    message text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS code_scanning_alert_instances_alert_created_idx
ON code_scanning_alert_instances (alert_id, created_at DESC);

CREATE TABLE IF NOT EXISTS code_scanning_alert_assignees (
    alert_id uuid NOT NULL REFERENCES code_scanning_alerts(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    assigned_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (alert_id, user_id)
);

CREATE INDEX IF NOT EXISTS code_scanning_alert_assignees_user_idx
ON code_scanning_alert_assignees (user_id, assigned_at DESC);

CREATE TABLE IF NOT EXISTS code_scanning_sarif_uploads (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    run_id uuid REFERENCES code_scanning_runs(id) ON DELETE SET NULL,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    artifact_storage_key text,
    artifact_sha256 text,
    status text NOT NULL DEFAULT 'processed',
    error_message text,
    created_at timestamptz NOT NULL DEFAULT now(),
    processed_at timestamptz,
    CONSTRAINT code_scanning_sarif_uploads_status_check CHECK (status IN ('queued', 'processing', 'processed', 'failed'))
);

CREATE INDEX IF NOT EXISTS code_scanning_sarif_uploads_repository_created_idx
ON code_scanning_sarif_uploads (repository_id, created_at DESC);

CREATE TABLE IF NOT EXISTS code_scanning_alert_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    alert_id uuid NOT NULL REFERENCES code_scanning_alerts(id) ON DELETE CASCADE,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    message text NOT NULL DEFAULT '',
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT code_scanning_alert_events_event_type_not_blank CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX IF NOT EXISTS code_scanning_alert_events_alert_created_idx
ON code_scanning_alert_events (alert_id, created_at ASC);

CREATE INDEX IF NOT EXISTS code_scanning_alert_events_repository_created_idx
ON code_scanning_alert_events (repository_id, created_at DESC);
