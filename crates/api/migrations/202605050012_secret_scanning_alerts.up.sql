CREATE TABLE IF NOT EXISTS secret_scanning_patterns (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    slug text NOT NULL,
    provider text NOT NULL DEFAULT 'generic',
    secret_type text NOT NULL,
    display_name text NOT NULL,
    result_kind text NOT NULL DEFAULT 'provider',
    push_protection_enabled boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT secret_scanning_patterns_slug_not_blank CHECK (length(trim(slug)) > 0),
    CONSTRAINT secret_scanning_patterns_provider_not_blank CHECK (length(trim(provider)) > 0),
    CONSTRAINT secret_scanning_patterns_secret_type_not_blank CHECK (length(trim(secret_type)) > 0),
    CONSTRAINT secret_scanning_patterns_result_kind_check CHECK (result_kind IN ('provider', 'generic'))
);

CREATE UNIQUE INDEX IF NOT EXISTS secret_scanning_patterns_slug_unique
ON secret_scanning_patterns (lower(slug));

CREATE INDEX IF NOT EXISTS secret_scanning_patterns_provider_type_idx
ON secret_scanning_patterns (lower(provider), lower(secret_type));

CREATE TABLE IF NOT EXISTS secret_scanning_alerts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    pattern_id uuid NOT NULL REFERENCES secret_scanning_patterns(id) ON DELETE RESTRICT,
    number bigint NOT NULL,
    state text NOT NULL DEFAULT 'open',
    resolution text,
    resolution_comment text,
    resolved_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    resolved_at timestamptz,
    fingerprint text NOT NULL,
    secret_hash text NOT NULL,
    redacted_secret text NOT NULL,
    redacted_context text,
    result_kind text NOT NULL DEFAULT 'provider',
    validity_state text NOT NULL DEFAULT 'unknown',
    first_seen_at timestamptz NOT NULL DEFAULT now(),
    last_seen_at timestamptz NOT NULL DEFAULT now(),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT secret_scanning_alerts_number_positive CHECK (number > 0),
    CONSTRAINT secret_scanning_alerts_state_check CHECK (state IN ('open', 'resolved')),
    CONSTRAINT secret_scanning_alerts_resolution_check CHECK (
        resolution IS NULL OR resolution IN ('revoked', 'false_positive', 'used_in_tests', 'wont_fix')
    ),
    CONSTRAINT secret_scanning_alerts_result_kind_check CHECK (result_kind IN ('provider', 'generic')),
    CONSTRAINT secret_scanning_alerts_validity_state_check CHECK (
        validity_state IN ('unknown', 'active', 'inactive', 'checking', 'unsupported')
    ),
    CONSTRAINT secret_scanning_alerts_fingerprint_not_blank CHECK (length(trim(fingerprint)) > 0),
    CONSTRAINT secret_scanning_alerts_secret_hash_not_blank CHECK (length(trim(secret_hash)) > 0),
    CONSTRAINT secret_scanning_alerts_redacted_secret_not_blank CHECK (length(trim(redacted_secret)) > 0),
    CONSTRAINT secret_scanning_alerts_no_plaintext_guard CHECK (redacted_secret !~ '[A-Za-z0-9_/-]{24,}')
);

CREATE UNIQUE INDEX IF NOT EXISTS secret_scanning_alerts_repository_number_unique
ON secret_scanning_alerts (repository_id, number);

CREATE UNIQUE INDEX IF NOT EXISTS secret_scanning_alerts_repository_fingerprint_unique
ON secret_scanning_alerts (repository_id, fingerprint);

CREATE INDEX IF NOT EXISTS secret_scanning_alerts_repository_state_updated_idx
ON secret_scanning_alerts (repository_id, state, updated_at DESC);

CREATE INDEX IF NOT EXISTS secret_scanning_alerts_repository_kind_idx
ON secret_scanning_alerts (repository_id, result_kind);

CREATE TABLE IF NOT EXISTS secret_scanning_alert_locations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id uuid NOT NULL REFERENCES secret_scanning_alerts(id) ON DELETE CASCADE,
    repository_file_id uuid REFERENCES repository_files(id) ON DELETE SET NULL,
    commit_id uuid REFERENCES commits(id) ON DELETE SET NULL,
    ref_name text NOT NULL,
    branch_name text,
    path text NOT NULL,
    start_line integer NOT NULL DEFAULT 1,
    end_line integer,
    redacted_snippet text,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT secret_scanning_alert_locations_line_positive CHECK (
        start_line > 0 AND (end_line IS NULL OR end_line >= start_line)
    ),
    CONSTRAINT secret_scanning_alert_locations_ref_not_blank CHECK (length(trim(ref_name)) > 0),
    CONSTRAINT secret_scanning_alert_locations_path_not_blank CHECK (length(trim(path)) > 0)
);

CREATE INDEX IF NOT EXISTS secret_scanning_alert_locations_alert_created_idx
ON secret_scanning_alert_locations (alert_id, created_at DESC);

CREATE TABLE IF NOT EXISTS secret_scanning_alert_assignees (
    alert_id uuid NOT NULL REFERENCES secret_scanning_alerts(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    assigned_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (alert_id, user_id)
);

CREATE INDEX IF NOT EXISTS secret_scanning_alert_assignees_user_idx
ON secret_scanning_alert_assignees (user_id, assigned_at DESC);

CREATE TABLE IF NOT EXISTS secret_scanning_validity_checks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id uuid NOT NULL REFERENCES secret_scanning_alerts(id) ON DELETE CASCADE,
    provider text NOT NULL,
    status text NOT NULL DEFAULT 'unknown',
    checked_at timestamptz NOT NULL DEFAULT now(),
    message text,
    CONSTRAINT secret_scanning_validity_checks_status_check CHECK (
        status IN ('unknown', 'active', 'inactive', 'checking', 'unsupported', 'error')
    )
);

CREATE INDEX IF NOT EXISTS secret_scanning_validity_checks_alert_checked_idx
ON secret_scanning_validity_checks (alert_id, checked_at DESC);

CREATE TABLE IF NOT EXISTS push_protection_bypasses (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    alert_id uuid REFERENCES secret_scanning_alerts(id) ON DELETE SET NULL,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    ref_name text NOT NULL,
    commit_oid text,
    path text,
    reason text NOT NULL,
    status text NOT NULL DEFAULT 'accepted',
    redacted_snippet text,
    expires_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT push_protection_bypasses_reason_not_blank CHECK (length(trim(reason)) > 0),
    CONSTRAINT push_protection_bypasses_status_check CHECK (status IN ('accepted', 'pending_review', 'rejected', 'expired'))
);

CREATE INDEX IF NOT EXISTS push_protection_bypasses_repository_created_idx
ON push_protection_bypasses (repository_id, created_at DESC);

CREATE INDEX IF NOT EXISTS push_protection_bypasses_alert_created_idx
ON push_protection_bypasses (alert_id, created_at DESC)
WHERE alert_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS secret_scanning_alert_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    alert_id uuid NOT NULL REFERENCES secret_scanning_alerts(id) ON DELETE CASCADE,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    message text NOT NULL DEFAULT '',
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT secret_scanning_alert_events_event_type_not_blank CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX IF NOT EXISTS secret_scanning_alert_events_alert_created_idx
ON secret_scanning_alert_events (alert_id, created_at ASC);

CREATE INDEX IF NOT EXISTS secret_scanning_alert_events_repository_created_idx
ON secret_scanning_alert_events (repository_id, created_at DESC);
