DROP INDEX IF EXISTS release_assets_release_active_idx;
DROP INDEX IF EXISTS releases_repository_tag_active_unique;

CREATE UNIQUE INDEX IF NOT EXISTS releases_repository_tag_unique
ON releases (repository_id, lower(tag_name));
