ALTER TABLE packages
ADD COLUMN IF NOT EXISTS deleted_at timestamptz,
ADD COLUMN IF NOT EXISTS deleted_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS restored_at timestamptz,
ADD COLUMN IF NOT EXISTS restored_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL;

ALTER TABLE package_versions
ADD COLUMN IF NOT EXISTS deleted_at timestamptz,
ADD COLUMN IF NOT EXISTS deleted_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS restored_at timestamptz,
ADD COLUMN IF NOT EXISTS restored_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS packages_active_owner_user_idx
ON packages (owner_user_id, package_type, lower(name))
WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS packages_active_owner_org_idx
ON packages (owner_organization_id, package_type, lower(name))
WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS package_versions_active_package_idx
ON package_versions (package_id, created_at DESC, id DESC)
WHERE deleted_at IS NULL;
