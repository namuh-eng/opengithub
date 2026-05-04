CREATE TABLE IF NOT EXISTS commit_file_changes (
    commit_id uuid NOT NULL REFERENCES commits(id) ON DELETE CASCADE,
    path text NOT NULL,
    status text NOT NULL DEFAULT 'modified',
    additions bigint NOT NULL DEFAULT 0,
    deletions bigint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (commit_id, path),
    CONSTRAINT commit_file_changes_path_not_blank CHECK (length(trim(path)) > 0),
    CONSTRAINT commit_file_changes_status_check CHECK (status IN ('added', 'modified', 'removed', 'renamed')),
    CONSTRAINT commit_file_changes_counts_nonnegative CHECK (additions >= 0 AND deletions >= 0)
);

CREATE INDEX IF NOT EXISTS commit_file_changes_path_idx
ON commit_file_changes (path);

CREATE TABLE IF NOT EXISTS repository_insight_snapshots (
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    period_key text NOT NULL,
    cache_key text NOT NULL,
    snapshot jsonb NOT NULL DEFAULT '{}'::jsonb,
    computed_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL DEFAULT (now() + interval '10 minutes'),
    PRIMARY KEY (repository_id, period_key, cache_key),
    CONSTRAINT repository_insight_snapshots_period_key_check
        CHECK (period_key IN ('24h', '3d', '1w', '1m')),
    CONSTRAINT repository_insight_snapshots_cache_key_not_blank CHECK (length(trim(cache_key)) > 0),
    CONSTRAINT repository_insight_snapshots_snapshot_object CHECK (jsonb_typeof(snapshot) = 'object')
);

CREATE INDEX IF NOT EXISTS repository_insight_snapshots_repository_computed_idx
ON repository_insight_snapshots (repository_id, computed_at DESC);

CREATE TABLE IF NOT EXISTS recent_insight_views (
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    period_key text NOT NULL,
    viewed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (repository_id, user_id, period_key),
    CONSTRAINT recent_insight_views_period_key_check
        CHECK (period_key IN ('24h', '3d', '1w', '1m'))
);

CREATE INDEX IF NOT EXISTS recent_insight_views_user_viewed_idx
ON recent_insight_views (user_id, viewed_at DESC);
