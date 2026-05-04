ALTER TABLE repository_insight_snapshots
DROP CONSTRAINT IF EXISTS repository_insight_snapshots_period_key_check;

ALTER TABLE repository_insight_snapshots
ADD CONSTRAINT repository_insight_snapshots_period_key_check
    CHECK (period_key IN ('24h', '3d', '1w', '1m', '14d'));

ALTER TABLE recent_insight_views
DROP CONSTRAINT IF EXISTS recent_insight_views_period_key_check;

ALTER TABLE recent_insight_views
ADD CONSTRAINT recent_insight_views_period_key_check
    CHECK (period_key IN ('24h', '3d', '1w', '1m', '14d'));

CREATE TABLE IF NOT EXISTS repository_traffic_daily (
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    traffic_date date NOT NULL,
    clones_total bigint NOT NULL DEFAULT 0,
    clones_unique bigint NOT NULL DEFAULT 0,
    visitors_total bigint NOT NULL DEFAULT 0,
    visitors_unique bigint NOT NULL DEFAULT 0,
    computed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (repository_id, traffic_date),
    CONSTRAINT repository_traffic_daily_counts_nonnegative
        CHECK (clones_total >= 0 AND clones_unique >= 0 AND visitors_total >= 0 AND visitors_unique >= 0),
    CONSTRAINT repository_traffic_daily_unique_bounds
        CHECK (clones_unique <= clones_total AND visitors_unique <= visitors_total)
);

CREATE INDEX IF NOT EXISTS repository_traffic_daily_repo_date_idx
ON repository_traffic_daily (repository_id, traffic_date DESC);

CREATE TABLE IF NOT EXISTS repository_referrers_daily (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    traffic_date date NOT NULL,
    referrer text NOT NULL,
    total_views bigint NOT NULL DEFAULT 0,
    unique_visitors bigint NOT NULL DEFAULT 0,
    computed_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_referrers_daily_referrer_not_blank CHECK (length(trim(referrer)) > 0),
    CONSTRAINT repository_referrers_daily_counts_nonnegative
        CHECK (total_views >= 0 AND unique_visitors >= 0),
    CONSTRAINT repository_referrers_daily_unique_bounds CHECK (unique_visitors <= total_views)
);

CREATE INDEX IF NOT EXISTS repository_referrers_daily_repo_date_idx
ON repository_referrers_daily (repository_id, traffic_date DESC);

CREATE INDEX IF NOT EXISTS repository_referrers_daily_repo_referrer_idx
ON repository_referrers_daily (repository_id, lower(referrer));

CREATE TABLE IF NOT EXISTS repository_popular_content_daily (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    traffic_date date NOT NULL,
    path text NOT NULL,
    title text,
    total_views bigint NOT NULL DEFAULT 0,
    unique_visitors bigint NOT NULL DEFAULT 0,
    computed_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_popular_content_daily_path_not_blank CHECK (length(trim(path)) > 0),
    CONSTRAINT repository_popular_content_daily_counts_nonnegative
        CHECK (total_views >= 0 AND unique_visitors >= 0),
    CONSTRAINT repository_popular_content_daily_unique_bounds CHECK (unique_visitors <= total_views)
);

CREATE INDEX IF NOT EXISTS repository_popular_content_daily_repo_date_idx
ON repository_popular_content_daily (repository_id, traffic_date DESC);

CREATE INDEX IF NOT EXISTS repository_popular_content_daily_repo_path_idx
ON repository_popular_content_daily (repository_id, path);
