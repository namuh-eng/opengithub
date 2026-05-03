ALTER TABLE personal_access_tokens
    ADD COLUMN IF NOT EXISTS description text NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS token_type text NOT NULL DEFAULT 'classic',
    ADD COLUMN IF NOT EXISTS resource_owner_user_id uuid REFERENCES users(id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS resource_owner_organization_id uuid REFERENCES organizations(id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS repository_access text NOT NULL DEFAULT 'all',
    ADD COLUMN IF NOT EXISTS status text NOT NULL DEFAULT 'active',
    ADD COLUMN IF NOT EXISTS approved_at timestamptz,
    ADD COLUMN IF NOT EXISTS revoked_reason text;

UPDATE personal_access_tokens
SET resource_owner_user_id = user_id
WHERE resource_owner_user_id IS NULL
  AND resource_owner_organization_id IS NULL;

ALTER TABLE personal_access_tokens
    DROP CONSTRAINT IF EXISTS personal_access_tokens_token_type_check,
    DROP CONSTRAINT IF EXISTS personal_access_tokens_repository_access_check,
    DROP CONSTRAINT IF EXISTS personal_access_tokens_owner_check,
    DROP CONSTRAINT IF EXISTS personal_access_tokens_status_check;

ALTER TABLE personal_access_tokens
    ADD CONSTRAINT personal_access_tokens_token_type_check
    CHECK (token_type IN ('classic', 'fine_grained')),
    ADD CONSTRAINT personal_access_tokens_repository_access_check
    CHECK (repository_access IN ('all', 'selected', 'none')),
    ADD CONSTRAINT personal_access_tokens_owner_check
    CHECK (
        (resource_owner_user_id IS NOT NULL AND resource_owner_organization_id IS NULL)
        OR (resource_owner_user_id IS NULL AND resource_owner_organization_id IS NOT NULL)
    ),
    ADD CONSTRAINT personal_access_tokens_status_check
    CHECK (status IN ('active', 'pending', 'revoked'));

CREATE INDEX IF NOT EXISTS personal_access_tokens_owner_user_idx
ON personal_access_tokens (resource_owner_user_id, created_at DESC)
WHERE resource_owner_user_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS personal_access_tokens_owner_org_idx
ON personal_access_tokens (resource_owner_organization_id, created_at DESC)
WHERE resource_owner_organization_id IS NOT NULL;

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

CREATE TABLE IF NOT EXISTS personal_access_token_repositories (
    token_id uuid NOT NULL REFERENCES personal_access_tokens(id) ON DELETE CASCADE,
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    selected_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (token_id, repository_id)
);

CREATE INDEX IF NOT EXISTS personal_access_token_repositories_repository_idx
ON personal_access_token_repositories (repository_id);

CREATE TABLE IF NOT EXISTS sudo_grants (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id text NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    method text NOT NULL DEFAULT 'session_confirmation',
    granted_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT sudo_grants_method_not_blank CHECK (length(trim(method)) > 0),
    CONSTRAINT sudo_grants_expiry_after_grant CHECK (expires_at > granted_at)
);

CREATE INDEX IF NOT EXISTS sudo_grants_session_active_idx
ON sudo_grants (session_id, user_id, expires_at DESC)
WHERE revoked_at IS NULL;
