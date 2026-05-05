DROP INDEX IF EXISTS discussion_deletion_tombstones_repository_created_idx;
DROP INDEX IF EXISTS discussions_repository_deleted_idx;
DROP TABLE IF EXISTS discussion_deletion_tombstones;
ALTER TABLE discussions
    DROP COLUMN IF EXISTS transferred_from_number,
    DROP COLUMN IF EXISTS transferred_from_repository_id,
    DROP COLUMN IF EXISTS deleted_reason,
    DROP COLUMN IF EXISTS deleted_by_user_id,
    DROP COLUMN IF EXISTS deleted_at;
