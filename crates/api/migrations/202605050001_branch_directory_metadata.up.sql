CREATE TABLE IF NOT EXISTS repository_branch_directory_recent_visits (
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tab text NOT NULL DEFAULT 'overview',
    query text NOT NULL DEFAULT '',
    page bigint NOT NULL DEFAULT 1,
    page_size bigint NOT NULL DEFAULT 30,
    viewed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (repository_id, user_id, tab, query),
    CONSTRAINT repository_branch_directory_recent_visits_tab_check
        CHECK (tab IN ('overview', 'active', 'stale', 'all')),
    CONSTRAINT repository_branch_directory_recent_visits_query_length
        CHECK (char_length(query) <= 120),
    CONSTRAINT repository_branch_directory_recent_visits_page_positive
        CHECK (page > 0 AND page_size > 0)
);

CREATE INDEX IF NOT EXISTS repository_branch_directory_recent_visits_user_viewed_idx
ON repository_branch_directory_recent_visits (user_id, viewed_at DESC);
