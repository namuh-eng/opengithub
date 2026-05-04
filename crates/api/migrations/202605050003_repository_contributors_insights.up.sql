CREATE TABLE IF NOT EXISTS repository_contributors_weekly (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    period_key text NOT NULL,
    cache_key text NOT NULL,
    bucket_start timestamptz NOT NULL,
    bucket_end timestamptz NOT NULL,
    author_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    author_login text NOT NULL,
    commits bigint NOT NULL DEFAULT 0,
    additions bigint NOT NULL DEFAULT 0,
    deletions bigint NOT NULL DEFAULT 0,
    computed_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_contributors_weekly_period_key_check
        CHECK (period_key IN ('24h', '3d', '1w', '1m')),
    CONSTRAINT repository_contributors_weekly_cache_key_not_blank CHECK (length(trim(cache_key)) > 0),
    CONSTRAINT repository_contributors_weekly_author_login_not_blank CHECK (length(trim(author_login)) > 0),
    CONSTRAINT repository_contributors_weekly_counts_nonnegative
        CHECK (commits >= 0 AND additions >= 0 AND deletions >= 0),
    CONSTRAINT repository_contributors_weekly_bucket_order CHECK (bucket_end > bucket_start)
);

CREATE INDEX IF NOT EXISTS repository_contributors_weekly_repo_cache_idx
ON repository_contributors_weekly (repository_id, period_key, cache_key, bucket_start);

CREATE INDEX IF NOT EXISTS repository_contributors_weekly_author_idx
ON repository_contributors_weekly (repository_id, author_user_id, bucket_start);
