ALTER TABLE repository_security_advisories
    ADD COLUMN IF NOT EXISTS ghsa_id text,
    ADD COLUMN IF NOT EXISTS cve_id text,
    ADD COLUMN IF NOT EXISTS cvss_vector text,
    ADD COLUMN IF NOT EXISTS cvss_score numeric(3,1),
    ADD COLUMN IF NOT EXISTS cvss_metrics jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS package_ecosystem text,
    ADD COLUMN IF NOT EXISTS affected_versions text,
    ADD COLUMN IF NOT EXISTS patched_versions text,
    ADD COLUMN IF NOT EXISTS markdown_summary text,
    ADD COLUMN IF NOT EXISTS markdown_details text,
    ADD COLUMN IF NOT EXISTS details_html text,
    ADD COLUMN IF NOT EXISTS author_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS dependency_advisory_id uuid REFERENCES dependency_advisories(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS withdrawn_at timestamptz,
    ADD COLUMN IF NOT EXISTS created_at timestamptz NOT NULL DEFAULT now();

UPDATE repository_security_advisories
SET ghsa_id = advisory_identifier
WHERE ghsa_id IS NULL OR length(trim(ghsa_id)) = 0;

ALTER TABLE repository_security_advisories
    ADD CONSTRAINT repository_security_advisories_ghsa_not_blank
    CHECK (length(trim(ghsa_id)) > 0);

ALTER TABLE repository_security_advisories
    ADD CONSTRAINT repository_security_advisories_cve_format
    CHECK (cve_id IS NULL OR cve_id ~ '^CVE-[0-9]{4}-[0-9]{4,}$');

ALTER TABLE repository_security_advisories
    ADD CONSTRAINT repository_security_advisories_cvss_score_range
    CHECK (cvss_score IS NULL OR (cvss_score >= 0 AND cvss_score <= 10));

CREATE UNIQUE INDEX IF NOT EXISTS repository_security_advisories_repository_ghsa_unique
ON repository_security_advisories (repository_id, lower(ghsa_id));

CREATE INDEX IF NOT EXISTS repository_security_advisories_repository_severity_idx
ON repository_security_advisories (repository_id, severity, COALESCE(published_at, updated_at) DESC);

CREATE INDEX IF NOT EXISTS repository_security_advisories_repository_cve_idx
ON repository_security_advisories (repository_id, lower(cve_id))
WHERE cve_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS repository_security_advisory_cwes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    advisory_id uuid NOT NULL REFERENCES repository_security_advisories(id) ON DELETE CASCADE,
    cwe_id text NOT NULL,
    name text NOT NULL DEFAULT '',
    href text,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_security_advisory_cwes_cwe_format CHECK (cwe_id ~ '^CWE-[0-9]+$')
);

CREATE UNIQUE INDEX IF NOT EXISTS repository_security_advisory_cwes_unique
ON repository_security_advisory_cwes (advisory_id, upper(cwe_id));

CREATE TABLE IF NOT EXISTS repository_security_advisory_credits (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    advisory_id uuid NOT NULL REFERENCES repository_security_advisories(id) ON DELETE CASCADE,
    user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    login text NOT NULL,
    avatar_url text,
    credit_type text NOT NULL DEFAULT 'reporter',
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_security_advisory_credits_login_not_blank CHECK (length(trim(login)) > 0),
    CONSTRAINT repository_security_advisory_credits_type_check CHECK (
        credit_type IN ('reporter', 'finder', 'analyst', 'coordinator', 'remediation_developer', 'reviewer')
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS repository_security_advisory_credits_unique
ON repository_security_advisory_credits (advisory_id, lower(login), credit_type);

CREATE TABLE IF NOT EXISTS repository_security_advisory_collaborators (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    advisory_id uuid NOT NULL REFERENCES repository_security_advisories(id) ON DELETE CASCADE,
    user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    login text NOT NULL,
    avatar_url text,
    role text NOT NULL DEFAULT 'collaborator',
    invited_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_security_advisory_collaborators_login_not_blank CHECK (length(trim(login)) > 0),
    CONSTRAINT repository_security_advisory_collaborators_role_check CHECK (
        role IN ('author', 'collaborator', 'credit_only')
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS repository_security_advisory_collaborators_unique
ON repository_security_advisory_collaborators (advisory_id, lower(login));

CREATE TABLE IF NOT EXISTS repository_security_advisory_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    advisory_id uuid NOT NULL REFERENCES repository_security_advisories(id) ON DELETE CASCADE,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    message text NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_security_advisory_events_type_not_blank CHECK (length(trim(event_type)) > 0),
    CONSTRAINT repository_security_advisory_events_message_not_blank CHECK (length(trim(message)) > 0)
);

CREATE INDEX IF NOT EXISTS repository_security_advisory_events_advisory_created_idx
ON repository_security_advisory_events (advisory_id, created_at ASC);
