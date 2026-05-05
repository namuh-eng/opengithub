ALTER TABLE sbom_exports
    ADD COLUMN IF NOT EXISTS artifact_json jsonb;

CREATE INDEX IF NOT EXISTS sbom_exports_ready_download_idx
ON sbom_exports (repository_id, id, status)
WHERE status = 'ready';
