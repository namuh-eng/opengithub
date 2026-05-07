ALTER TABLE users
    ADD COLUMN IF NOT EXISTS public_email_id uuid,
    ADD COLUMN IF NOT EXISTS pronouns text,
    ADD COLUMN IF NOT EXISTS display_local_time boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS time_zone text NOT NULL DEFAULT 'UTC',
    ADD COLUMN IF NOT EXISTS private_profile boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS show_private_contribution_count boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS achievements_enabled boolean NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS preferred_language text NOT NULL DEFAULT 'en';

CREATE TABLE IF NOT EXISTS user_email_addresses (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email text NOT NULL,
    is_primary boolean NOT NULL DEFAULT false,
    is_public boolean NOT NULL DEFAULT false,
    verified_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT user_email_addresses_email_not_blank CHECK (length(trim(email)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS user_email_addresses_user_email_lower_unique
ON user_email_addresses (user_id, lower(email));
CREATE INDEX IF NOT EXISTS user_email_addresses_user_idx
ON user_email_addresses (user_id, is_primary DESC, created_at ASC);

CREATE TRIGGER user_email_addresses_set_updated_at
BEFORE UPDATE ON user_email_addresses
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

INSERT INTO user_email_addresses (user_id, email, is_primary, is_public, verified_at)
SELECT u.id, u.email, true, true, now()
FROM users u
WHERE NOT EXISTS (
    SELECT 1 FROM user_email_addresses e WHERE e.user_id = u.id AND lower(e.email) = lower(u.email)
);

UPDATE users u
SET public_email_id = e.id
FROM user_email_addresses e
WHERE e.user_id = u.id
  AND e.is_primary = true
  AND u.public_email_id IS NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'users_public_email_id_fkey'
    ) THEN
        ALTER TABLE users
            ADD CONSTRAINT users_public_email_id_fkey
            FOREIGN KEY (public_email_id) REFERENCES user_email_addresses(id) ON DELETE SET NULL;
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS user_social_accounts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider text NOT NULL,
    handle_or_url text NOT NULL DEFAULT '',
    position integer NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT user_social_accounts_provider_not_blank CHECK (length(trim(provider)) > 0),
    CONSTRAINT user_social_accounts_position_check CHECK (position BETWEEN 1 AND 4)
);

CREATE UNIQUE INDEX IF NOT EXISTS user_social_accounts_user_position_unique
ON user_social_accounts (user_id, position);
CREATE INDEX IF NOT EXISTS user_social_accounts_user_idx
ON user_social_accounts (user_id, position ASC);

CREATE TRIGGER user_social_accounts_set_updated_at
BEFORE UPDATE ON user_social_accounts
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS user_avatars (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    s3_bucket text NOT NULL DEFAULT 'opengithub-user-avatars',
    s3_key text NOT NULL,
    public_url text NOT NULL,
    content_type text NOT NULL,
    byte_size integer NOT NULL,
    active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT user_avatars_key_not_blank CHECK (length(trim(s3_key)) > 0),
    CONSTRAINT user_avatars_url_not_blank CHECK (length(trim(public_url)) > 0),
    CONSTRAINT user_avatars_size_positive CHECK (byte_size > 0)
);

CREATE INDEX IF NOT EXISTS user_avatars_user_active_idx
ON user_avatars (user_id, active, created_at DESC);

CREATE TABLE IF NOT EXISTS security_audit_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    target_type text NOT NULL DEFAULT 'user',
    target_id uuid,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT security_audit_events_event_type_not_blank CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX IF NOT EXISTS security_audit_events_actor_created_idx
ON security_audit_events (actor_user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS security_audit_events_event_type_idx
ON security_audit_events (event_type);
