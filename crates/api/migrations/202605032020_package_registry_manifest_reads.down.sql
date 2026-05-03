DROP INDEX IF EXISTS package_registry_audit_package_idx;
DROP TABLE IF EXISTS package_registry_audit_events;
DROP INDEX IF EXISTS package_versions_manifest_digest_idx;
ALTER TABLE package_versions
DROP CONSTRAINT IF EXISTS package_versions_manifest_size_non_negative;
ALTER TABLE package_versions
DROP COLUMN IF EXISTS manifest_size_bytes,
DROP COLUMN IF EXISTS config_digest,
DROP COLUMN IF EXISTS manifest_media_type;
