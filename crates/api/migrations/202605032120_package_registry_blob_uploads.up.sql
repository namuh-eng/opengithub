CREATE TABLE IF NOT EXISTS package_registry_uploads (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id uuid NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    expected_digest text,
    storage_kind text NOT NULL DEFAULT 'local',
    storage_key text NOT NULL,
    size_bytes bigint NOT NULL DEFAULT 0,
    status text NOT NULL DEFAULT 'active',
    expires_at timestamptz NOT NULL DEFAULT now() + interval '1 hour',
    completed_at timestamptz,
    cancelled_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT package_registry_uploads_size_non_negative CHECK (size_bytes >= 0),
    CONSTRAINT package_registry_uploads_status_check CHECK (status IN ('active', 'completed', 'cancelled', 'expired')),
    CONSTRAINT package_registry_uploads_storage_kind_check CHECK (storage_kind IN ('local', 's3'))
);

CREATE INDEX IF NOT EXISTS package_registry_uploads_package_idx
ON package_registry_uploads (package_id, created_at DESC);

CREATE INDEX IF NOT EXISTS package_registry_uploads_status_expiry_idx
ON package_registry_uploads (status, expires_at);

CREATE TRIGGER package_registry_uploads_set_updated_at
BEFORE UPDATE ON package_registry_uploads
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

ALTER TABLE package_blobs
ADD COLUMN IF NOT EXISTS storage_kind text NOT NULL DEFAULT 'local',
ADD COLUMN IF NOT EXISTS byte_size bigint;

ALTER TABLE package_blobs
DROP CONSTRAINT IF EXISTS package_blobs_storage_kind_check;

ALTER TABLE package_blobs
ADD CONSTRAINT package_blobs_storage_kind_check CHECK (storage_kind IN ('local', 's3'));

UPDATE package_blobs
SET byte_size = size_bytes
WHERE byte_size IS NULL AND size_bytes IS NOT NULL;

CREATE INDEX IF NOT EXISTS package_versions_package_created_id_idx
ON package_versions (package_id, created_at DESC, id DESC);
