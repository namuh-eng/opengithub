ALTER TABLE repository_permissions
    DROP CONSTRAINT IF EXISTS repository_permissions_role_check;

ALTER TABLE repository_permissions
    ADD CONSTRAINT repository_permissions_role_check
    CHECK (role IN ('owner', 'admin', 'maintain', 'write', 'triage', 'read'));

CREATE TABLE IF NOT EXISTS repository_team_permissions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    team_id uuid NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    role text NOT NULL,
    source text NOT NULL DEFAULT 'team',
    created_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_team_permissions_role_check CHECK (role IN ('admin', 'maintain', 'write', 'triage', 'read')),
    CONSTRAINT repository_team_permissions_source_check CHECK (source IN ('team', 'inherited'))
);

CREATE UNIQUE INDEX IF NOT EXISTS repository_team_permissions_repo_team_unique
ON repository_team_permissions (repository_id, team_id);
CREATE INDEX IF NOT EXISTS repository_team_permissions_team_id_idx
ON repository_team_permissions (team_id);

DROP TRIGGER IF EXISTS repository_team_permissions_set_updated_at ON repository_team_permissions;
CREATE TRIGGER repository_team_permissions_set_updated_at
BEFORE UPDATE ON repository_team_permissions
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS repository_invitations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    invited_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    invited_email text NOT NULL,
    role text NOT NULL,
    status text NOT NULL DEFAULT 'pending',
    token_hash text NOT NULL,
    invited_by_user_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    email_delivery_status text NOT NULL DEFAULT 'degraded',
    expires_at timestamptz NOT NULL,
    accepted_at timestamptz,
    canceled_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_invitations_email_not_blank CHECK (length(trim(invited_email)) > 0),
    CONSTRAINT repository_invitations_role_check CHECK (role IN ('admin', 'maintain', 'write', 'triage', 'read')),
    CONSTRAINT repository_invitations_status_check CHECK (status IN ('pending', 'accepted', 'canceled', 'expired')),
    CONSTRAINT repository_invitations_email_delivery_status_check CHECK (email_delivery_status IN ('queued', 'sent', 'degraded', 'failed'))
);

CREATE UNIQUE INDEX IF NOT EXISTS repository_invitations_pending_email_unique
ON repository_invitations (repository_id, lower(invited_email))
WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS repository_invitations_repo_status_idx
ON repository_invitations (repository_id, status, created_at DESC);
CREATE INDEX IF NOT EXISTS repository_invitations_invited_user_idx
ON repository_invitations (invited_user_id)
WHERE invited_user_id IS NOT NULL;

DROP TRIGGER IF EXISTS repository_invitations_set_updated_at ON repository_invitations;
CREATE TRIGGER repository_invitations_set_updated_at
BEFORE UPDATE ON repository_invitations
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
