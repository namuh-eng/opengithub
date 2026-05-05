CREATE TABLE IF NOT EXISTS dependabot_alerts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    repository_dependency_id uuid NOT NULL REFERENCES repository_dependencies(id) ON DELETE CASCADE,
    dependency_advisory_id uuid NOT NULL REFERENCES dependency_advisories(id) ON DELETE CASCADE,
    number bigint NOT NULL,
    state text NOT NULL DEFAULT 'open',
    scope text NOT NULL DEFAULT 'production',
    vulnerable_requirements text,
    fixed_version text,
    dismissed_reason text,
    dismissed_comment text,
    dismissed_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    dismissed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT dependabot_alerts_number_positive CHECK (number > 0),
    CONSTRAINT dependabot_alerts_state_check CHECK (state IN ('open', 'dismissed', 'fixed')),
    CONSTRAINT dependabot_alerts_scope_check CHECK (scope IN ('production', 'development')),
    CONSTRAINT dependabot_alerts_fixed_version_not_blank CHECK (fixed_version IS NULL OR length(trim(fixed_version)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS dependabot_alerts_repository_number_unique
ON dependabot_alerts (repository_id, number);

CREATE UNIQUE INDEX IF NOT EXISTS dependabot_alerts_repository_dependency_advisory_unique
ON dependabot_alerts (repository_id, repository_dependency_id, dependency_advisory_id);

CREATE INDEX IF NOT EXISTS dependabot_alerts_repository_state_updated_idx
ON dependabot_alerts (repository_id, state, updated_at DESC);

CREATE INDEX IF NOT EXISTS dependabot_alerts_repository_dependency_idx
ON dependabot_alerts (repository_dependency_id);

CREATE TABLE IF NOT EXISTS dependabot_alert_assignees (
    alert_id uuid NOT NULL REFERENCES dependabot_alerts(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    assigned_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (alert_id, user_id)
);

CREATE INDEX IF NOT EXISTS dependabot_alert_assignees_user_idx
ON dependabot_alert_assignees (user_id, assigned_at DESC);

CREATE TABLE IF NOT EXISTS security_alert_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    alert_id uuid NOT NULL REFERENCES dependabot_alerts(id) ON DELETE CASCADE,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    message text NOT NULL DEFAULT '',
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT security_alert_events_event_type_not_blank CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX IF NOT EXISTS security_alert_events_alert_created_idx
ON security_alert_events (alert_id, created_at ASC);

CREATE INDEX IF NOT EXISTS security_alert_events_repository_created_idx
ON security_alert_events (repository_id, created_at DESC);
