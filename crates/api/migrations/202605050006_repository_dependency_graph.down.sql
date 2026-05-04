DROP INDEX IF EXISTS sbom_exports_actor_created_idx;
DROP INDEX IF EXISTS sbom_exports_repository_created_idx;
DROP TABLE IF EXISTS sbom_exports;

DROP INDEX IF EXISTS repository_dependents_source_package_idx;
DROP TABLE IF EXISTS repository_dependents;

DROP INDEX IF EXISTS dependency_advisories_package_identifier_unique;
DROP TABLE IF EXISTS dependency_advisories;

DROP INDEX IF EXISTS repository_dependencies_lockfile_path_idx;
DROP INDEX IF EXISTS repository_dependencies_repository_relationship_idx;
DROP INDEX IF EXISTS repository_dependencies_repository_detected_idx;
DROP INDEX IF EXISTS repository_dependencies_manifest_package_relationship_unique;
DROP TABLE IF EXISTS repository_dependencies;

DROP INDEX IF EXISTS dependency_packages_ecosystem_name_unique;
DROP TABLE IF EXISTS dependency_packages;

DROP INDEX IF EXISTS dependency_manifests_repository_ecosystem_idx;
DROP INDEX IF EXISTS dependency_manifests_repository_path_unique;
DROP TABLE IF EXISTS dependency_manifests;
