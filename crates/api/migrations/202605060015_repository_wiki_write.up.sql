ALTER TABLE wiki_repositories
    ADD COLUMN IF NOT EXISTS latest_commit_oid text,
    ADD COLUMN IF NOT EXISTS latest_published_at timestamptz;

CREATE TABLE IF NOT EXISTS wiki_git_commits (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    wiki_repository_id uuid NOT NULL REFERENCES wiki_repositories(id) ON DELETE CASCADE,
    page_id uuid REFERENCES wiki_pages(id) ON DELETE SET NULL,
    revision_id uuid REFERENCES wiki_page_revisions(id) ON DELETE SET NULL,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    branch text NOT NULL,
    commit_oid text NOT NULL,
    parent_oid text,
    message text NOT NULL,
    storage_path text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT wiki_git_commits_branch_not_blank CHECK (length(trim(branch)) > 0),
    CONSTRAINT wiki_git_commits_oid_not_blank CHECK (length(trim(commit_oid)) > 0),
    CONSTRAINT wiki_git_commits_message_not_blank CHECK (length(trim(message)) > 0),
    CONSTRAINT wiki_git_commits_storage_path_not_blank CHECK (length(trim(storage_path)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS wiki_git_commits_oid_unique
ON wiki_git_commits (wiki_repository_id, commit_oid);

CREATE INDEX IF NOT EXISTS wiki_git_commits_repository_created_idx
ON wiki_git_commits (wiki_repository_id, created_at DESC);

CREATE TABLE IF NOT EXISTS repository_activity_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    target_type text,
    target_id uuid,
    message text NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_activity_events_type_not_blank CHECK (length(trim(event_type)) > 0),
    CONSTRAINT repository_activity_events_message_not_blank CHECK (length(trim(message)) > 0)
);

CREATE INDEX IF NOT EXISTS repository_activity_events_repo_created_idx
ON repository_activity_events (repository_id, created_at DESC);

CREATE INDEX IF NOT EXISTS repository_activity_events_type_created_idx
ON repository_activity_events (event_type, created_at DESC);
