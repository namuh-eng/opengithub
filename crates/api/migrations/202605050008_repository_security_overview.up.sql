CREATE TABLE IF NOT EXISTS repository_security_feature_settings (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    feature_key text NOT NULL,
    status text NOT NULL DEFAULT 'disabled',
    summary text NOT NULL DEFAULT '',
    alert_count bigint NOT NULL DEFAULT 0,
    private_count bigint NOT NULL DEFAULT 0,
    config_href text,
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_security_feature_key_check CHECK (
        feature_key IN ('dependabot', 'code_scanning', 'secret_scanning', 'private_vulnerability_reporting')
    ),
    CONSTRAINT repository_security_feature_status_check CHECK (
        status IN ('enabled', 'disabled', 'needs_setup', 'unavailable')
    ),
    CONSTRAINT repository_security_feature_counts_non_negative CHECK (
        alert_count >= 0 AND private_count >= 0
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS repository_security_feature_settings_unique
ON repository_security_feature_settings (repository_id, feature_key);

CREATE INDEX IF NOT EXISTS repository_security_feature_settings_repository_idx
ON repository_security_feature_settings (repository_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS repository_security_policies (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    path text NOT NULL DEFAULT 'SECURITY.md',
    ref_name text NOT NULL DEFAULT 'main',
    source_commit_id uuid REFERENCES commits(id) ON DELETE SET NULL,
    blob_oid text,
    content_sha text NOT NULL,
    markdown text NOT NULL,
    rendered_html text NOT NULL,
    published boolean NOT NULL DEFAULT true,
    updated_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_security_policies_path_not_blank CHECK (length(trim(path)) > 0),
    CONSTRAINT repository_security_policies_ref_not_blank CHECK (length(trim(ref_name)) > 0),
    CONSTRAINT repository_security_policies_sha_not_blank CHECK (length(trim(content_sha)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS repository_security_policies_repository_path_unique
ON repository_security_policies (repository_id, lower(path));

CREATE INDEX IF NOT EXISTS repository_security_policies_repository_updated_idx
ON repository_security_policies (repository_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS repository_security_advisories (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    advisory_identifier text NOT NULL,
    severity text NOT NULL,
    status text NOT NULL DEFAULT 'published',
    title text NOT NULL,
    summary text NOT NULL DEFAULT '',
    package_name text,
    vulnerable_range text,
    advisory_href text NOT NULL,
    published_at timestamptz,
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_security_advisories_identifier_not_blank CHECK (length(trim(advisory_identifier)) > 0),
    CONSTRAINT repository_security_advisories_severity_check CHECK (severity IN ('low', 'moderate', 'high', 'critical')),
    CONSTRAINT repository_security_advisories_status_check CHECK (status IN ('draft', 'published', 'withdrawn')),
    CONSTRAINT repository_security_advisories_title_not_blank CHECK (length(trim(title)) > 0),
    CONSTRAINT repository_security_advisories_href_not_blank CHECK (length(trim(advisory_href)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS repository_security_advisories_unique
ON repository_security_advisories (repository_id, advisory_identifier);

CREATE INDEX IF NOT EXISTS repository_security_advisories_repository_status_idx
ON repository_security_advisories (repository_id, status, COALESCE(published_at, updated_at) DESC);

CREATE TABLE IF NOT EXISTS security_audit_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    target_type text,
    target_id text,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT security_audit_events_event_type_not_blank CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX IF NOT EXISTS security_audit_events_actor_created_idx
ON security_audit_events (actor_user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS security_audit_events_event_type_idx
ON security_audit_events (event_type);
