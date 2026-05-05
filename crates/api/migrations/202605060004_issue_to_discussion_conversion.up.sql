ALTER TABLE issues
    ADD COLUMN IF NOT EXISTS converted_discussion_id uuid REFERENCES discussions(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS converted_to_discussion_at timestamptz,
    ADD COLUMN IF NOT EXISTS converted_to_discussion_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL;

ALTER TABLE discussion_comments
    ADD COLUMN IF NOT EXISTS converted_issue_comment_id uuid REFERENCES comments(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS issues_converted_discussion_idx
    ON issues(converted_discussion_id)
    WHERE converted_discussion_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS discussion_comments_converted_issue_comment_idx
    ON discussion_comments(converted_issue_comment_id)
    WHERE converted_issue_comment_id IS NOT NULL;
