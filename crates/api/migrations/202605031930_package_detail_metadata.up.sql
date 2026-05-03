ALTER TABLE package_versions
ADD COLUMN IF NOT EXISTS digest text,
ADD COLUMN IF NOT EXISTS platform_os text,
ADD COLUMN IF NOT EXISTS platform_arch text,
ADD COLUMN IF NOT EXISTS readme_markdown text;

CREATE UNIQUE INDEX IF NOT EXISTS package_versions_package_digest_unique
ON package_versions (package_id, lower(digest))
WHERE digest IS NOT NULL;

CREATE INDEX IF NOT EXISTS package_versions_package_platform_idx
ON package_versions (package_id, platform_os, platform_arch)
WHERE platform_os IS NOT NULL OR platform_arch IS NOT NULL;

CREATE TABLE IF NOT EXISTS package_blobs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id uuid NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    package_version_id uuid REFERENCES package_versions(id) ON DELETE CASCADE,
    digest text NOT NULL,
    media_type text,
    platform_os text,
    platform_arch text,
    size_bytes bigint,
    storage_key text,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT package_blobs_digest_not_blank CHECK (length(trim(digest)) > 0),
    CONSTRAINT package_blobs_size_non_negative CHECK (size_bytes IS NULL OR size_bytes >= 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS package_blobs_package_digest_unique
ON package_blobs (package_id, lower(digest));

CREATE INDEX IF NOT EXISTS package_blobs_version_idx
ON package_blobs (package_version_id);

CREATE TABLE IF NOT EXISTS package_about_overrides (
    package_id uuid PRIMARY KEY REFERENCES packages(id) ON DELETE CASCADE,
    markdown text NOT NULL,
    updated_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT package_about_markdown_not_blank CHECK (length(trim(markdown)) > 0)
);
