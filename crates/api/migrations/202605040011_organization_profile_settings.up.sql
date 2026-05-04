ALTER TABLE organizations
    ADD COLUMN IF NOT EXISTS public_email text,
    ADD COLUMN IF NOT EXISTS billing_email text,
    ADD COLUMN IF NOT EXISTS avatar_s3_bucket text,
    ADD COLUMN IF NOT EXISTS avatar_s3_key text,
    ADD COLUMN IF NOT EXISTS avatar_updated_at timestamptz;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'organizations_public_email_not_blank'
    ) THEN
        ALTER TABLE organizations
            ADD CONSTRAINT organizations_public_email_not_blank
            CHECK (public_email IS NULL OR length(trim(public_email)) > 0);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'organizations_billing_email_not_blank'
    ) THEN
        ALTER TABLE organizations
            ADD CONSTRAINT organizations_billing_email_not_blank
            CHECK (billing_email IS NULL OR length(trim(billing_email)) > 0);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'organizations_avatar_storage_pair'
    ) THEN
        ALTER TABLE organizations
            ADD CONSTRAINT organizations_avatar_storage_pair
            CHECK (
                (avatar_s3_bucket IS NULL AND avatar_s3_key IS NULL)
                OR (length(trim(avatar_s3_bucket)) > 0 AND length(trim(avatar_s3_key)) > 0)
            );
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS organization_social_accounts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    provider text NOT NULL,
    value text NOT NULL,
    position integer NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT organization_social_accounts_provider_check
        CHECK (provider IN ('x', 'mastodon', 'linkedin', 'bluesky')),
    CONSTRAINT organization_social_accounts_value_not_blank CHECK (length(trim(value)) > 0),
    CONSTRAINT organization_social_accounts_position_check CHECK (position BETWEEN 1 AND 4)
);

CREATE UNIQUE INDEX IF NOT EXISTS organization_social_accounts_org_provider_unique
ON organization_social_accounts (organization_id, provider);

CREATE UNIQUE INDEX IF NOT EXISTS organization_social_accounts_org_position_unique
ON organization_social_accounts (organization_id, position);

CREATE INDEX IF NOT EXISTS organization_social_accounts_org_position_idx
ON organization_social_accounts (organization_id, position ASC);

CREATE TRIGGER organization_social_accounts_set_updated_at
BEFORE UPDATE ON organization_social_accounts
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX IF NOT EXISTS organization_audit_events_org_type_created_idx
ON organization_audit_events (organization_id, event_type, created_at DESC);
