ALTER TABLE users
    ADD COLUMN IF NOT EXISTS vigilant_mode boolean NOT NULL DEFAULT false;

CREATE TABLE IF NOT EXISTS ssh_keys (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title text NOT NULL,
    key_type text NOT NULL,
    public_key text NOT NULL,
    fingerprint_sha256 text NOT NULL,
    access_mode text NOT NULL DEFAULT 'read_write',
    source text NOT NULL DEFAULT 'settings',
    last_used_at timestamptz,
    revoked_at timestamptz,
    revoked_reason text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT ssh_keys_title_not_blank CHECK (length(trim(title)) > 0),
    CONSTRAINT ssh_keys_key_type_check CHECK (
        key_type IN (
            'ssh-ed25519',
            'ssh-rsa',
            'ecdsa-sha2-nistp256',
            'ecdsa-sha2-nistp384',
            'ecdsa-sha2-nistp521',
            'sk-ssh-ed25519@openssh.com',
            'sk-ecdsa-sha2-nistp256@openssh.com'
        )
    ),
    CONSTRAINT ssh_keys_public_key_not_blank CHECK (length(trim(public_key)) > 0),
    CONSTRAINT ssh_keys_fingerprint_not_blank CHECK (length(trim(fingerprint_sha256)) > 0),
    CONSTRAINT ssh_keys_access_mode_check CHECK (access_mode IN ('read_only', 'read_write')),
    CONSTRAINT ssh_keys_source_not_blank CHECK (length(trim(source)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS ssh_keys_user_active_fingerprint_unique
ON ssh_keys (user_id, fingerprint_sha256)
WHERE revoked_at IS NULL;

CREATE INDEX IF NOT EXISTS ssh_keys_user_created_idx
ON ssh_keys (user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS ssh_keys_user_active_idx
ON ssh_keys (user_id, created_at DESC)
WHERE revoked_at IS NULL;

CREATE TRIGGER ssh_keys_set_updated_at
BEFORE UPDATE ON ssh_keys
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS gpg_keys (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title text NOT NULL,
    armored_public_key text NOT NULL,
    primary_fingerprint text NOT NULL,
    key_id text,
    source text NOT NULL DEFAULT 'settings',
    last_used_at timestamptz,
    revoked_at timestamptz,
    revoked_reason text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT gpg_keys_title_not_blank CHECK (length(trim(title)) > 0),
    CONSTRAINT gpg_keys_armored_not_blank CHECK (length(trim(armored_public_key)) > 0),
    CONSTRAINT gpg_keys_fingerprint_not_blank CHECK (length(trim(primary_fingerprint)) > 0),
    CONSTRAINT gpg_keys_source_not_blank CHECK (length(trim(source)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS gpg_keys_user_active_fingerprint_unique
ON gpg_keys (user_id, primary_fingerprint)
WHERE revoked_at IS NULL;

CREATE INDEX IF NOT EXISTS gpg_keys_user_created_idx
ON gpg_keys (user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS gpg_keys_user_active_idx
ON gpg_keys (user_id, created_at DESC)
WHERE revoked_at IS NULL;

CREATE TRIGGER gpg_keys_set_updated_at
BEFORE UPDATE ON gpg_keys
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS gpg_key_emails (
    gpg_key_id uuid NOT NULL REFERENCES gpg_keys(id) ON DELETE CASCADE,
    email text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (gpg_key_id, email),
    CONSTRAINT gpg_key_emails_email_not_blank CHECK (length(trim(email)) > 0)
);

CREATE INDEX IF NOT EXISTS gpg_key_emails_email_lower_idx
ON gpg_key_emails (lower(email));

CREATE INDEX IF NOT EXISTS security_audit_events_signing_keys_idx
ON security_audit_events (actor_user_id, event_type, created_at DESC)
WHERE event_type LIKE 'signing_key.%' OR event_type = 'vigilant_mode.update';
