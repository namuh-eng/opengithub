DROP INDEX IF EXISTS package_versions_active_package_idx;
DROP INDEX IF EXISTS packages_active_owner_org_idx;
DROP INDEX IF EXISTS packages_active_owner_user_idx;

ALTER TABLE package_versions
DROP COLUMN IF EXISTS restored_by_user_id,
DROP COLUMN IF EXISTS restored_at,
DROP COLUMN IF EXISTS deleted_by_user_id,
DROP COLUMN IF EXISTS deleted_at;

ALTER TABLE packages
DROP COLUMN IF EXISTS restored_by_user_id,
DROP COLUMN IF EXISTS restored_at,
DROP COLUMN IF EXISTS deleted_by_user_id,
DROP COLUMN IF EXISTS deleted_at;
