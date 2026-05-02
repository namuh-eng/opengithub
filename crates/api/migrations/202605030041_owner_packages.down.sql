DROP INDEX IF EXISTS package_downloads_package_idx;
DROP INDEX IF EXISTS package_repository_links_package_idx;
DROP INDEX IF EXISTS package_permissions_package_user_unique;
DROP TABLE IF EXISTS package_downloads;
DROP TABLE IF EXISTS package_repository_links;
DROP TABLE IF EXISTS package_permissions;
ALTER TABLE packages DROP CONSTRAINT IF EXISTS packages_type_check;
ALTER TABLE packages ADD CONSTRAINT packages_type_check CHECK (package_type IN ('container', 'npm', 'maven', 'generic'));
