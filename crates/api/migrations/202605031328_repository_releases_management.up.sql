DROP INDEX IF EXISTS releases_repository_tag_unique;

CREATE UNIQUE INDEX IF NOT EXISTS releases_repository_tag_active_unique
ON releases (repository_id, lower(tag_name))
WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS release_assets_release_active_idx
ON release_assets (release_id, created_at)
WHERE deleted_at IS NULL;
