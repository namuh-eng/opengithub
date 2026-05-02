DROP INDEX IF EXISTS organization_memberships_org_role_idx;
DROP INDEX IF EXISTS repositories_org_updated_idx;
DROP INDEX IF EXISTS repository_topics_repository_idx;
DROP INDEX IF EXISTS repository_topics_topic_lower_idx;
DROP TABLE IF EXISTS repository_topics;
DROP TRIGGER IF EXISTS organization_profile_pins_set_updated_at ON organization_profile_pins;
DROP TABLE IF EXISTS organization_profile_pins;
DROP INDEX IF EXISTS organization_verified_domains_org_verified_idx;
DROP INDEX IF EXISTS organization_verified_domains_org_domain_unique;
DROP TABLE IF EXISTS organization_verified_domains;

ALTER TABLE organizations
    DROP CONSTRAINT IF EXISTS organizations_profile_visibility_check,
    DROP COLUMN IF EXISTS public_members_visible,
    DROP COLUMN IF EXISTS profile_visibility,
    DROP COLUMN IF EXISTS location,
    DROP COLUMN IF EXISTS website_url,
    DROP COLUMN IF EXISTS avatar_url;
