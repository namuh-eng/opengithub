CREATE TABLE IF NOT EXISTS dependency_manifests (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    path text NOT NULL,
    ecosystem text NOT NULL,
    lockfile_path text,
    dependency_count bigint NOT NULL DEFAULT 0,
    detected_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT dependency_manifests_path_not_blank CHECK (length(trim(path)) > 0),
    CONSTRAINT dependency_manifests_ecosystem_check CHECK (ecosystem IN ('npm', 'cargo', 'pip')),
    CONSTRAINT dependency_manifests_dependency_count_nonnegative CHECK (dependency_count >= 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS dependency_manifests_repository_path_unique
ON dependency_manifests (repository_id, lower(path));

CREATE INDEX IF NOT EXISTS dependency_manifests_repository_ecosystem_idx
ON dependency_manifests (repository_id, ecosystem, updated_at DESC);

CREATE TABLE IF NOT EXISTS dependency_packages (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    ecosystem text NOT NULL,
    name text NOT NULL,
    package_href text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT dependency_packages_ecosystem_check CHECK (ecosystem IN ('npm', 'cargo', 'pip')),
    CONSTRAINT dependency_packages_name_not_blank CHECK (length(trim(name)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS dependency_packages_ecosystem_name_unique
ON dependency_packages (ecosystem, lower(name));

CREATE TABLE IF NOT EXISTS repository_dependencies (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    manifest_id uuid NOT NULL REFERENCES dependency_manifests(id) ON DELETE CASCADE,
    package_id uuid NOT NULL REFERENCES dependency_packages(id) ON DELETE CASCADE,
    package_version text,
    relationship text NOT NULL DEFAULT 'direct',
    license text,
    lockfile_path text,
    detected_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_dependencies_relationship_check CHECK (relationship IN ('direct', 'transitive')),
    CONSTRAINT repository_dependencies_version_not_blank CHECK (package_version IS NULL OR length(trim(package_version)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS repository_dependencies_manifest_package_relationship_unique
ON repository_dependencies (manifest_id, package_id, relationship);

CREATE INDEX IF NOT EXISTS repository_dependencies_repository_detected_idx
ON repository_dependencies (repository_id, detected_at DESC);

CREATE INDEX IF NOT EXISTS repository_dependencies_repository_relationship_idx
ON repository_dependencies (repository_id, relationship);

CREATE INDEX IF NOT EXISTS repository_dependencies_lockfile_path_idx
ON repository_dependencies (repository_id, lower(lockfile_path));

CREATE TABLE IF NOT EXISTS dependency_advisories (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id uuid NOT NULL REFERENCES dependency_packages(id) ON DELETE CASCADE,
    advisory_identifier text NOT NULL,
    severity text NOT NULL DEFAULT 'moderate',
    title text NOT NULL,
    advisory_href text NOT NULL,
    published_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT dependency_advisories_identifier_not_blank CHECK (length(trim(advisory_identifier)) > 0),
    CONSTRAINT dependency_advisories_severity_check CHECK (severity IN ('low', 'moderate', 'high', 'critical')),
    CONSTRAINT dependency_advisories_title_not_blank CHECK (length(trim(title)) > 0),
    CONSTRAINT dependency_advisories_href_not_blank CHECK (length(trim(advisory_href)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS dependency_advisories_package_identifier_unique
ON dependency_advisories (package_id, advisory_identifier);

CREATE TABLE IF NOT EXISTS repository_dependents (
    source_repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    dependent_repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    package_id uuid NOT NULL REFERENCES dependency_packages(id) ON DELETE CASCADE,
    manifest_path text,
    detected_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (source_repository_id, dependent_repository_id, package_id),
    CONSTRAINT repository_dependents_distinct_repositories CHECK (source_repository_id <> dependent_repository_id)
);

CREATE INDEX IF NOT EXISTS repository_dependents_source_package_idx
ON repository_dependents (source_repository_id, package_id, detected_at DESC);

CREATE TABLE IF NOT EXISTS sbom_exports (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    actor_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status text NOT NULL DEFAULT 'pending',
    format text NOT NULL DEFAULT 'spdx-json',
    artifact_key text,
    artifact_sha256 text,
    artifact_byte_size bigint NOT NULL DEFAULT 0,
    download_expires_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz,
    CONSTRAINT sbom_exports_status_check CHECK (status IN ('pending', 'ready', 'failed')),
    CONSTRAINT sbom_exports_format_check CHECK (format IN ('spdx-json')),
    CONSTRAINT sbom_exports_size_nonnegative CHECK (artifact_byte_size >= 0)
);

CREATE INDEX IF NOT EXISTS sbom_exports_repository_created_idx
ON sbom_exports (repository_id, created_at DESC);

CREATE INDEX IF NOT EXISTS sbom_exports_actor_created_idx
ON sbom_exports (actor_user_id, created_at DESC);
