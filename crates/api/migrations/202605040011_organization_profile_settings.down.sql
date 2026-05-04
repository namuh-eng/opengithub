DROP INDEX IF EXISTS organization_audit_events_org_type_created_idx;
DROP TRIGGER IF EXISTS organization_social_accounts_set_updated_at ON organization_social_accounts;
DROP INDEX IF EXISTS organization_social_accounts_org_position_idx;
DROP INDEX IF EXISTS organization_social_accounts_org_position_unique;
DROP INDEX IF EXISTS organization_social_accounts_org_provider_unique;
DROP TABLE IF EXISTS organization_social_accounts;

ALTER TABLE organizations
    DROP CONSTRAINT IF EXISTS organizations_avatar_storage_pair,
    DROP CONSTRAINT IF EXISTS organizations_billing_email_not_blank,
    DROP CONSTRAINT IF EXISTS organizations_public_email_not_blank,
    DROP COLUMN IF EXISTS avatar_updated_at,
    DROP COLUMN IF EXISTS avatar_s3_key,
    DROP COLUMN IF EXISTS avatar_s3_bucket,
    DROP COLUMN IF EXISTS billing_email,
    DROP COLUMN IF EXISTS public_email;
