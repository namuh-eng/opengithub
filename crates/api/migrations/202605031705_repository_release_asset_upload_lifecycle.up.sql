CREATE TABLE release_asset_upload_intents (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    release_id uuid REFERENCES releases(id) ON DELETE CASCADE,
    asset_id uuid REFERENCES release_assets(id) ON DELETE SET NULL,
    asset_name text NOT NULL,
    content_type text NOT NULL DEFAULT 'application/octet-stream',
    byte_size bigint NOT NULL,
    checksum_sha256 text,
    storage_kind text NOT NULL DEFAULT 'local',
    storage_key text NOT NULL,
    status text NOT NULL DEFAULT 'pending',
    handoff_token text NOT NULL,
    created_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    completed_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    cancelled_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    expires_at timestamptz NOT NULL,
    completed_at timestamptz,
    cancelled_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT release_asset_upload_intents_asset_name_not_blank CHECK (length(trim(asset_name)) > 0),
    CONSTRAINT release_asset_upload_intents_storage_key_not_blank CHECK (length(trim(storage_key)) > 0),
    CONSTRAINT release_asset_upload_intents_handoff_token_not_blank CHECK (length(trim(handoff_token)) > 0),
    CONSTRAINT release_asset_upload_intents_byte_size_positive CHECK (byte_size > 0),
    CONSTRAINT release_asset_upload_intents_storage_kind_check CHECK (storage_kind IN ('local', 's3')),
    CONSTRAINT release_asset_upload_intents_status_check CHECK (status IN ('pending', 'completed', 'cancelled', 'expired'))
);

CREATE INDEX release_asset_upload_intents_repository_status_idx
ON release_asset_upload_intents (repository_id, status, expires_at);

CREATE INDEX release_asset_upload_intents_release_idx
ON release_asset_upload_intents (release_id, created_at)
WHERE release_id IS NOT NULL;

CREATE UNIQUE INDEX release_asset_upload_intents_handoff_token_unique
ON release_asset_upload_intents (handoff_token);

CREATE TRIGGER release_asset_upload_intents_set_updated_at
BEFORE UPDATE ON release_asset_upload_intents
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
