ALTER TABLE organizations
    ADD COLUMN IF NOT EXISTS contact_email text,
    ADD COLUMN IF NOT EXISTS terms_of_service_type text,
    ADD COLUMN IF NOT EXISTS company_name text,
    ADD COLUMN IF NOT EXISTS ownership_type text NOT NULL DEFAULT 'personal';

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'organizations_contact_email_not_blank'
    ) THEN
        ALTER TABLE organizations
            ADD CONSTRAINT organizations_contact_email_not_blank
            CHECK (contact_email IS NULL OR length(trim(contact_email)) > 0);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'organizations_terms_type_not_blank'
    ) THEN
        ALTER TABLE organizations
            ADD CONSTRAINT organizations_terms_type_not_blank
            CHECK (terms_of_service_type IS NULL OR length(trim(terms_of_service_type)) > 0);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'organizations_company_name_not_blank'
    ) THEN
        ALTER TABLE organizations
            ADD CONSTRAINT organizations_company_name_not_blank
            CHECK (company_name IS NULL OR length(trim(company_name)) > 0);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'organizations_ownership_type_check'
    ) THEN
        ALTER TABLE organizations
            ADD CONSTRAINT organizations_ownership_type_check
            CHECK (ownership_type IN ('personal', 'business'));
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS reserved_slugs (
    slug text PRIMARY KEY,
    reason text NOT NULL DEFAULT 'reserved',
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT reserved_slugs_slug_not_blank CHECK (length(trim(slug)) > 0),
    CONSTRAINT reserved_slugs_reason_not_blank CHECK (length(trim(reason)) > 0)
);

INSERT INTO reserved_slugs (slug, reason)
VALUES
    ('about', 'system_route'),
    ('account', 'system_route'),
    ('admin', 'system_route'),
    ('api', 'system_route'),
    ('apps', 'system_route'),
    ('assets', 'system_route'),
    ('billing', 'system_route'),
    ('dashboard', 'system_route'),
    ('docs', 'system_route'),
    ('explore', 'system_route'),
    ('features', 'system_route'),
    ('help', 'system_route'),
    ('login', 'system_route'),
    ('logout', 'system_route'),
    ('new', 'system_route'),
    ('notifications', 'system_route'),
    ('orgs', 'system_route'),
    ('organizations', 'system_route'),
    ('pricing', 'system_route'),
    ('search', 'system_route'),
    ('settings', 'system_route'),
    ('signup', 'system_route'),
    ('support', 'system_route'),
    ('users', 'system_route')
ON CONFLICT (slug) DO NOTHING;

CREATE TABLE IF NOT EXISTS organization_policy_settings (
    organization_id uuid PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    base_repository_permission text NOT NULL DEFAULT 'read',
    members_can_create_public_repositories boolean NOT NULL DEFAULT true,
    members_can_create_private_repositories boolean NOT NULL DEFAULT true,
    members_can_create_internal_repositories boolean NOT NULL DEFAULT false,
    members_can_fork_private_repositories boolean NOT NULL DEFAULT true,
    repository_discussions_enabled boolean NOT NULL DEFAULT true,
    projects_base_permission text NOT NULL DEFAULT 'write',
    pages_public_publishing boolean NOT NULL DEFAULT true,
    pages_private_publishing boolean NOT NULL DEFAULT true,
    app_access_request_policy text NOT NULL DEFAULT 'owners_and_members',
    members_can_change_repository_visibility boolean NOT NULL DEFAULT false,
    members_can_delete_repositories boolean NOT NULL DEFAULT false,
    members_can_transfer_repositories boolean NOT NULL DEFAULT false,
    members_can_delete_issues boolean NOT NULL DEFAULT false,
    members_can_create_teams boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT organization_policy_base_permission_check
        CHECK (base_repository_permission IN ('none', 'read', 'write', 'admin')),
    CONSTRAINT organization_policy_projects_permission_check
        CHECK (projects_base_permission IN ('none', 'read', 'write', 'admin')),
    CONSTRAINT organization_policy_app_access_check
        CHECK (app_access_request_policy IN ('owners_only', 'owners_and_members'))
);

CREATE TRIGGER organization_policy_settings_set_updated_at
BEFORE UPDATE ON organization_policy_settings
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS organization_audit_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT organization_audit_events_type_not_blank CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX IF NOT EXISTS organization_audit_events_org_created_idx
ON organization_audit_events (organization_id, created_at DESC);

CREATE INDEX IF NOT EXISTS organization_audit_events_actor_created_idx
ON organization_audit_events (actor_user_id, created_at DESC);
