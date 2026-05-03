DROP TABLE IF EXISTS release_audit_events;
DROP TABLE IF EXISTS release_downloads;
DROP TABLE IF EXISTS release_reactions;
DROP TABLE IF EXISTS release_assets;

DROP INDEX IF EXISTS releases_repository_target_commit_idx;
DROP INDEX IF EXISTS releases_repository_published_visible_idx;

ALTER TABLE releases
    DROP COLUMN IF EXISTS metadata,
    DROP COLUMN IF EXISTS updated_by_user_id,
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS immutable,
    DROP COLUMN IF EXISTS tag_signature_summary,
    DROP COLUMN IF EXISTS tag_verified,
    DROP COLUMN IF EXISTS is_latest,
    DROP COLUMN IF EXISTS rendered_body_excerpt,
    DROP COLUMN IF EXISTS body_html,
    DROP COLUMN IF EXISTS target_commit_id;
