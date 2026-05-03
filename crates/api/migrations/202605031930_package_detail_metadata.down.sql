DROP TABLE IF EXISTS package_about_overrides;
DROP INDEX IF EXISTS package_blobs_version_idx;
DROP INDEX IF EXISTS package_blobs_package_digest_unique;
DROP TABLE IF EXISTS package_blobs;
DROP INDEX IF EXISTS package_versions_package_platform_idx;
DROP INDEX IF EXISTS package_versions_package_digest_unique;
ALTER TABLE package_versions
DROP COLUMN IF EXISTS readme_markdown,
DROP COLUMN IF EXISTS platform_arch,
DROP COLUMN IF EXISTS platform_os,
DROP COLUMN IF EXISTS digest;
