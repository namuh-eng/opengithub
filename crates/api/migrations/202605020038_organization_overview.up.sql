CREATE TABLE organization_verified_domains (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    domain text NOT NULL,
    verified_at timestamptz NOT NULL DEFAULT now(),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT organization_verified_domains_domain_not_blank CHECK (length(trim(domain)) > 0)
);

CREATE UNIQUE INDEX organization_verified_domains_org_domain_unique
ON organization_verified_domains (organization_id, lower(domain));

CREATE TRIGGER organization_verified_domains_set_updated_at
BEFORE UPDATE ON organization_verified_domains
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE profile_pins (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id uuid REFERENCES users(id) ON DELETE CASCADE,
    owner_organization_id uuid REFERENCES organizations(id) ON DELETE CASCADE,
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT profile_pins_exactly_one_owner CHECK (
        (owner_user_id IS NOT NULL AND owner_organization_id IS NULL)
        OR (owner_user_id IS NULL AND owner_organization_id IS NOT NULL)
    ),
    CONSTRAINT profile_pins_position_non_negative CHECK (position >= 0)
);

CREATE UNIQUE INDEX profile_pins_user_repo_unique
ON profile_pins (owner_user_id, repository_id)
WHERE owner_user_id IS NOT NULL;

CREATE UNIQUE INDEX profile_pins_org_repo_unique
ON profile_pins (owner_organization_id, repository_id)
WHERE owner_organization_id IS NOT NULL;

CREATE INDEX profile_pins_org_position_idx
ON profile_pins (owner_organization_id, position)
WHERE owner_organization_id IS NOT NULL;

CREATE TRIGGER profile_pins_set_updated_at
BEFORE UPDATE ON profile_pins
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE repository_topics (
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    topic text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (repository_id, topic),
    CONSTRAINT repository_topics_topic_not_blank CHECK (length(trim(topic)) > 0)
);

CREATE INDEX repository_topics_topic_idx ON repository_topics (lower(topic));
