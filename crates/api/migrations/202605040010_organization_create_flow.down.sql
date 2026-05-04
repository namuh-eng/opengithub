DROP INDEX IF EXISTS organization_audit_events_actor_created_idx;
DROP INDEX IF EXISTS organization_audit_events_org_created_idx;
DROP TABLE IF EXISTS organization_audit_events;
DROP TRIGGER IF EXISTS organization_policy_settings_set_updated_at ON organization_policy_settings;
DROP TABLE IF EXISTS organization_policy_settings;
DROP TABLE IF EXISTS reserved_slugs;

ALTER TABLE organizations
    DROP CONSTRAINT IF EXISTS organizations_ownership_type_check,
    DROP CONSTRAINT IF EXISTS organizations_company_name_not_blank,
    DROP CONSTRAINT IF EXISTS organizations_terms_type_not_blank,
    DROP CONSTRAINT IF EXISTS organizations_contact_email_not_blank,
    DROP COLUMN IF EXISTS ownership_type,
    DROP COLUMN IF EXISTS company_name,
    DROP COLUMN IF EXISTS terms_of_service_type,
    DROP COLUMN IF EXISTS contact_email;
