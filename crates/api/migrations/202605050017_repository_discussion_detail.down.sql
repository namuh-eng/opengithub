DROP INDEX IF EXISTS discussion_answers_comment_idx;
DROP INDEX IF EXISTS discussion_reactions_comment_idx;
DROP INDEX IF EXISTS discussion_reactions_discussion_idx;
DROP INDEX IF EXISTS discussion_comments_discussion_parent_created_idx;
DROP INDEX IF EXISTS discussion_comments_parent_created_idx;

DROP TABLE IF EXISTS discussion_answers;
DROP TABLE IF EXISTS discussion_reactions;

ALTER TABLE discussion_comments
    DROP COLUMN IF EXISTS deleted_reason,
    DROP COLUMN IF EXISTS deleted_at,
    DROP COLUMN IF EXISTS edited_at,
    DROP COLUMN IF EXISTS parent_comment_id;
