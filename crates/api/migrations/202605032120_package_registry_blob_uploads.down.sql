DROP INDEX IF EXISTS package_versions_package_created_id_idx;

ALTER TABLE package_blobs
DROP CONSTRAINT IF EXISTS package_blobs_storage_kind_check;

ALTER TABLE package_blobs
DROP COLUMN IF EXISTS byte_size,
DROP COLUMN IF EXISTS storage_kind;

DROP TRIGGER IF EXISTS package_registry_uploads_set_updated_at ON package_registry_uploads;
DROP INDEX IF EXISTS package_registry_uploads_status_expiry_idx;
DROP INDEX IF EXISTS package_registry_uploads_package_idx;
DROP TABLE IF EXISTS package_registry_uploads;
