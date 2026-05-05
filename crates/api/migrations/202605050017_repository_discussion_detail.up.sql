ALTER TABLE discussion_comments
    ADD COLUMN IF NOT EXISTS parent_comment_id uuid REFERENCES discussion_comments(id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS edited_at timestamptz,
    ADD COLUMN IF NOT EXISTS deleted_at timestamptz,
    ADD COLUMN IF NOT EXISTS deleted_reason text;

CREATE TABLE IF NOT EXISTS discussion_reactions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    discussion_id uuid NOT NULL REFERENCES discussions(id) ON DELETE CASCADE,
    comment_id uuid REFERENCES discussion_comments(id) ON DELETE CASCADE,
    user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    content text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (content IN ('+1', '-1', 'laugh', 'hooray', 'confused', 'heart', 'rocket', 'eyes')),
    UNIQUE (discussion_id, comment_id, user_id, content)
);

CREATE TABLE IF NOT EXISTS discussion_answers (
    discussion_id uuid PRIMARY KEY REFERENCES discussions(id) ON DELETE CASCADE,
    comment_id uuid NOT NULL REFERENCES discussion_comments(id) ON DELETE CASCADE,
    marked_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    marked_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS discussion_comments_parent_created_idx
    ON discussion_comments(parent_comment_id, created_at);
CREATE INDEX IF NOT EXISTS discussion_comments_discussion_parent_created_idx
    ON discussion_comments(discussion_id, parent_comment_id, created_at);
CREATE INDEX IF NOT EXISTS discussion_reactions_discussion_idx
    ON discussion_reactions(discussion_id, content);
CREATE INDEX IF NOT EXISTS discussion_reactions_comment_idx
    ON discussion_reactions(comment_id, content);
CREATE INDEX IF NOT EXISTS discussion_answers_comment_idx
    ON discussion_answers(comment_id);
