DROP TABLE IF EXISTS sudo_grants;
DROP TABLE IF EXISTS personal_access_token_repositories;

DROP INDEX IF EXISTS personal_access_tokens_owner_org_idx;
DROP INDEX IF EXISTS personal_access_tokens_owner_user_idx;

ALTER TABLE personal_access_tokens
    DROP CONSTRAINT IF EXISTS personal_access_tokens_status_check,
    DROP CONSTRAINT IF EXISTS personal_access_tokens_owner_check,
    DROP CONSTRAINT IF EXISTS personal_access_tokens_repository_access_check,
    DROP CONSTRAINT IF EXISTS personal_access_tokens_token_type_check;

ALTER TABLE personal_access_tokens
    DROP COLUMN IF EXISTS revoked_reason,
    DROP COLUMN IF EXISTS approved_at,
    DROP COLUMN IF EXISTS status,
    DROP COLUMN IF EXISTS repository_access,
    DROP COLUMN IF EXISTS resource_owner_organization_id,
    DROP COLUMN IF EXISTS resource_owner_user_id,
    DROP COLUMN IF EXISTS token_type,
    DROP COLUMN IF EXISTS description;
