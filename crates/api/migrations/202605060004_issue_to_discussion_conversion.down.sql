DROP INDEX IF EXISTS discussion_comments_converted_issue_comment_idx;
DROP INDEX IF EXISTS issues_converted_discussion_idx;

ALTER TABLE discussion_comments
    DROP COLUMN IF EXISTS converted_issue_comment_id;

ALTER TABLE issues
    DROP COLUMN IF EXISTS converted_to_discussion_by_user_id,
    DROP COLUMN IF EXISTS converted_to_discussion_at,
    DROP COLUMN IF EXISTS converted_discussion_id;
