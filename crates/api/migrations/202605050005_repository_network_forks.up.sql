CREATE TABLE IF NOT EXISTS repository_network_forks (
    source_repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    fork_repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    pushed_at timestamptz NOT NULL,
    stars_count bigint NOT NULL DEFAULT 0,
    forks_count bigint NOT NULL DEFAULT 0,
    open_issues_count bigint NOT NULL DEFAULT 0,
    open_pull_requests_count bigint NOT NULL DEFAULT 0,
    is_active boolean NOT NULL DEFAULT true,
    is_archived boolean NOT NULL DEFAULT false,
    is_starred_by_actor boolean NOT NULL DEFAULT false,
    computed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (source_repository_id, fork_repository_id),
    CONSTRAINT repository_network_forks_distinct_repositories CHECK (source_repository_id <> fork_repository_id),
    CONSTRAINT repository_network_forks_counts_nonnegative
        CHECK (stars_count >= 0 AND forks_count >= 0 AND open_issues_count >= 0 AND open_pull_requests_count >= 0)
);

CREATE INDEX IF NOT EXISTS repository_network_forks_source_pushed_idx
ON repository_network_forks (source_repository_id, pushed_at DESC);

CREATE INDEX IF NOT EXISTS repository_network_forks_source_computed_idx
ON repository_network_forks (source_repository_id, computed_at DESC);

CREATE TABLE IF NOT EXISTS saved_fork_filter_defaults (
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    period_key text NOT NULL DEFAULT '1m',
    repository_type text NOT NULL DEFAULT 'all',
    sort_key text NOT NULL DEFAULT 'most_starred',
    saved_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (repository_id, user_id),
    CONSTRAINT saved_fork_filter_defaults_period_check
        CHECK (period_key IN ('24h', '3d', '1w', '1m', 'all')),
    CONSTRAINT saved_fork_filter_defaults_type_check
        CHECK (repository_type IN ('all', 'active', 'inactive', 'archived', 'starred')),
    CONSTRAINT saved_fork_filter_defaults_sort_check
        CHECK (sort_key IN ('most_starred', 'recently_pushed', 'recently_created', 'recently_updated', 'name'))
);

CREATE INDEX IF NOT EXISTS saved_fork_filter_defaults_user_saved_idx
ON saved_fork_filter_defaults (user_id, saved_at DESC);
