DROP TABLE IF EXISTS security_audit_events;
DROP TABLE IF EXISTS user_avatars;

DROP TRIGGER IF EXISTS user_social_accounts_set_updated_at ON user_social_accounts;
DROP TABLE IF EXISTS user_social_accounts;

ALTER TABLE users DROP CONSTRAINT IF EXISTS users_public_email_id_fkey;

DROP TRIGGER IF EXISTS user_email_addresses_set_updated_at ON user_email_addresses;
DROP TABLE IF EXISTS user_email_addresses;

ALTER TABLE users
    DROP COLUMN IF EXISTS preferred_language,
    DROP COLUMN IF EXISTS achievements_enabled,
    DROP COLUMN IF EXISTS show_private_contribution_count,
    DROP COLUMN IF EXISTS private_profile,
    DROP COLUMN IF EXISTS time_zone,
    DROP COLUMN IF EXISTS display_local_time,
    DROP COLUMN IF EXISTS pronouns,
    DROP COLUMN IF EXISTS public_email_id;
