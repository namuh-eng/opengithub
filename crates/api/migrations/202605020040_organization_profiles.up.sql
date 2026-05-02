ALTER TABLE organizations
    ADD COLUMN IF NOT EXISTS avatar_url text,
    ADD COLUMN IF NOT EXISTS website_url text,
    ADD COLUMN IF NOT EXISTS location text,
    ADD COLUMN IF NOT EXISTS profile_visibility text NOT NULL DEFAULT 'public',
    ADD COLUMN IF NOT EXISTS public_members_visible boolean NOT NULL DEFAULT true;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'organizations_profile_visibility_check'
    ) THEN
        ALTER TABLE organizations
            ADD CONSTRAINT organizations_profile_visibility_check
            CHECK (profile_visibility IN ('public', 'private'));
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS organization_verified_domains (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    domain text NOT NULL,
    verified_at timestamptz NOT NULL DEFAULT now(),
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT organization_verified_domains_domain_not_blank CHECK (length(trim(domain)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS organization_verified_domains_org_domain_unique
ON organization_verified_domains (organization_id, lower(domain));
CREATE INDEX IF NOT EXISTS organization_verified_domains_org_verified_idx
ON organization_verified_domains (organization_id, verified_at DESC);

CREATE TABLE IF NOT EXISTS organization_profile_pins (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    position integer NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT organization_profile_pins_position_positive CHECK (position > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS organization_profile_pins_org_repository_unique
ON organization_profile_pins (organization_id, repository_id);
CREATE UNIQUE INDEX IF NOT EXISTS organization_profile_pins_org_position_unique
ON organization_profile_pins (organization_id, position);
CREATE INDEX IF NOT EXISTS organization_profile_pins_org_position_idx
ON organization_profile_pins (organization_id, position ASC);

CREATE TRIGGER organization_profile_pins_set_updated_at
BEFORE UPDATE ON organization_profile_pins
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS repository_topics (
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    topic text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (repository_id, topic),
    CONSTRAINT repository_topics_topic_not_blank CHECK (length(trim(topic)) > 0)
);

CREATE INDEX IF NOT EXISTS repository_topics_topic_lower_idx
ON repository_topics (lower(topic));
CREATE INDEX IF NOT EXISTS repository_topics_repository_idx
ON repository_topics (repository_id);

CREATE INDEX IF NOT EXISTS repositories_org_updated_idx
ON repositories (owner_organization_id, updated_at DESC)
WHERE owner_organization_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS organization_memberships_org_role_idx
ON organization_memberships (organization_id, role);
