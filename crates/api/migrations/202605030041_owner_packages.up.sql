ALTER TABLE packages DROP CONSTRAINT IF EXISTS packages_type_check;
ALTER TABLE packages ADD CONSTRAINT packages_type_check CHECK (package_type IN ('container', 'npm', 'rubygems', 'maven', 'nuget', 'generic'));

CREATE TABLE IF NOT EXISTS package_permissions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id uuid NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role text NOT NULL DEFAULT 'read',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT package_permissions_role_check CHECK (role IN ('read', 'write', 'admin'))
);

CREATE UNIQUE INDEX IF NOT EXISTS package_permissions_package_user_unique
ON package_permissions (package_id, user_id);

CREATE TRIGGER package_permissions_set_updated_at
BEFORE UPDATE ON package_permissions
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS package_repository_links (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id uuid NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    link_type text NOT NULL DEFAULT 'source',
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT package_repository_links_type_check CHECK (link_type IN ('source', 'workflow', 'release', 'manual'))
);

CREATE INDEX IF NOT EXISTS package_repository_links_package_idx
ON package_repository_links (package_id, created_at DESC);

CREATE TABLE IF NOT EXISTS package_downloads (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id uuid NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    package_version_id uuid REFERENCES package_versions(id) ON DELETE SET NULL,
    downloaded_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    download_count bigint NOT NULL DEFAULT 1,
    downloaded_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT package_downloads_count_positive CHECK (download_count > 0)
);

CREATE INDEX IF NOT EXISTS package_downloads_package_idx
ON package_downloads (package_id, downloaded_at DESC);
