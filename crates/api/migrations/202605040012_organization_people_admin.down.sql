DROP TABLE IF EXISTS organization_invitations;

DROP INDEX IF EXISTS organization_memberships_org_security_manager_idx;
DROP INDEX IF EXISTS organization_memberships_org_outside_idx;
DROP INDEX IF EXISTS organization_memberships_org_visibility_idx;

ALTER TABLE organization_memberships
    DROP CONSTRAINT IF EXISTS organization_memberships_visibility_check,
    DROP COLUMN IF EXISTS security_manager,
    DROP COLUMN IF EXISTS outside_collaborator,
    DROP COLUMN IF EXISTS membership_visibility;
