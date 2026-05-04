ALTER TABLE organization_memberships
    ADD COLUMN IF NOT EXISTS membership_visibility text NOT NULL DEFAULT 'public',
    ADD COLUMN IF NOT EXISTS outside_collaborator boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS security_manager boolean NOT NULL DEFAULT false;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'organization_memberships_visibility_check'
    ) THEN
        ALTER TABLE organization_memberships
            ADD CONSTRAINT organization_memberships_visibility_check
            CHECK (membership_visibility IN ('public', 'private'));
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS organization_memberships_org_visibility_idx
ON organization_memberships (organization_id, membership_visibility);

CREATE INDEX IF NOT EXISTS organization_memberships_org_outside_idx
ON organization_memberships (organization_id, outside_collaborator)
WHERE outside_collaborator = true;

CREATE INDEX IF NOT EXISTS organization_memberships_org_security_manager_idx
ON organization_memberships (organization_id, security_manager)
WHERE security_manager = true;

CREATE TABLE IF NOT EXISTS organization_invitations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    invited_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    invited_email text NOT NULL,
    role text NOT NULL,
    team_ids uuid[] NOT NULL DEFAULT ARRAY[]::uuid[],
    status text NOT NULL DEFAULT 'pending',
    token_hash text NOT NULL,
    invited_by_user_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    email_delivery_status text NOT NULL DEFAULT 'degraded',
    email_delivery_error text,
    failed_at timestamptz,
    expires_at timestamptz NOT NULL,
    accepted_at timestamptz,
    canceled_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT organization_invitations_email_not_blank CHECK (length(trim(invited_email)) > 0),
    CONSTRAINT organization_invitations_role_check CHECK (role IN ('owner', 'admin', 'member')),
    CONSTRAINT organization_invitations_status_check CHECK (status IN ('pending', 'accepted', 'canceled', 'expired', 'failed')),
    CONSTRAINT organization_invitations_email_delivery_status_check CHECK (email_delivery_status IN ('queued', 'sent', 'degraded', 'failed'))
);

CREATE UNIQUE INDEX IF NOT EXISTS organization_invitations_pending_email_unique
ON organization_invitations (organization_id, lower(invited_email))
WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS organization_invitations_org_status_idx
ON organization_invitations (organization_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS organization_invitations_invited_user_idx
ON organization_invitations (invited_user_id)
WHERE invited_user_id IS NOT NULL;

DROP TRIGGER IF EXISTS organization_invitations_set_updated_at ON organization_invitations;
CREATE TRIGGER organization_invitations_set_updated_at
BEFORE UPDATE ON organization_invitations
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
