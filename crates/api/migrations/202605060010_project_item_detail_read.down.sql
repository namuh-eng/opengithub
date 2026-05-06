DROP TRIGGER IF EXISTS project_item_comments_set_updated_at ON project_item_comments;
DROP TABLE IF EXISTS project_item_comments;

DROP INDEX IF EXISTS project_items_project_type_archived_idx;
DROP INDEX IF EXISTS project_items_project_archived_idx;

ALTER TABLE project_items
DROP COLUMN IF EXISTS source_sync_version,
DROP COLUMN IF EXISTS source_synced_at,
DROP COLUMN IF EXISTS restored_by_user_id,
DROP COLUMN IF EXISTS restored_at,
DROP COLUMN IF EXISTS archived_by_user_id;
