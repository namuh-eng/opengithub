DROP INDEX IF EXISTS sbom_exports_ready_download_idx;

ALTER TABLE sbom_exports
    DROP COLUMN IF EXISTS artifact_json;
