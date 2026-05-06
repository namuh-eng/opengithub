CREATE TABLE IF NOT EXISTS wiki_revert_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    wiki_repository_id uuid NOT NULL REFERENCES wiki_repositories(id) ON DELETE CASCADE,
    page_id uuid NOT NULL REFERENCES wiki_pages(id) ON DELETE CASCADE,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    base_revision_id uuid NOT NULL REFERENCES wiki_page_revisions(id) ON DELETE RESTRICT,
    head_revision_id uuid NOT NULL REFERENCES wiki_page_revisions(id) ON DELETE RESTRICT,
    restored_revision_id uuid NOT NULL REFERENCES wiki_page_revisions(id) ON DELETE CASCADE,
    git_commit_id uuid REFERENCES wiki_git_commits(id) ON DELETE SET NULL,
    message text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT wiki_revert_events_message_not_blank CHECK (length(trim(message)) > 0),
    CONSTRAINT wiki_revert_events_distinct_revisions CHECK (base_revision_id <> head_revision_id)
);

CREATE INDEX IF NOT EXISTS wiki_revert_events_repository_created_idx
ON wiki_revert_events (repository_id, created_at DESC);

CREATE INDEX IF NOT EXISTS wiki_revert_events_page_created_idx
ON wiki_revert_events (page_id, created_at DESC);

