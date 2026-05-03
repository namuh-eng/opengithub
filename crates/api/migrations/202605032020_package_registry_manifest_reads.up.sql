ALTER TABLE package_versions
ADD COLUMN IF NOT EXISTS manifest_media_type text,
ADD COLUMN IF NOT EXISTS config_digest text,
ADD COLUMN IF NOT EXISTS manifest_size_bytes bigint;

ALTER TABLE package_versions
DROP CONSTRAINT IF EXISTS package_versions_manifest_size_non_negative;

ALTER TABLE package_versions
ADD CONSTRAINT package_versions_manifest_size_non_negative
CHECK (manifest_size_bytes IS NULL OR manifest_size_bytes >= 0);

CREATE INDEX IF NOT EXISTS package_versions_manifest_digest_idx
ON package_versions (package_id, lower(digest))
WHERE digest IS NOT NULL;

CREATE TABLE IF NOT EXISTS package_registry_audit_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id uuid NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    package_version_id uuid REFERENCES package_versions(id) ON DELETE SET NULL,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    reference text,
    digest text,
    user_agent text,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT package_registry_audit_event_type_not_blank CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX IF NOT EXISTS package_registry_audit_package_idx
ON package_registry_audit_events (package_id, created_at DESC);
