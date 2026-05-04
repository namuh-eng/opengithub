CREATE TABLE IF NOT EXISTS repository_commit_recent_visits (
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ref_name text NOT NULL,
    path text NOT NULL DEFAULT '',
    filters jsonb NOT NULL DEFAULT '{}'::jsonb,
    viewed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (repository_id, user_id, ref_name, path),
    CONSTRAINT repository_commit_recent_visits_ref_not_blank CHECK (length(trim(ref_name)) > 0),
    CONSTRAINT repository_commit_recent_visits_filters_object CHECK (jsonb_typeof(filters) = 'object')
);

CREATE INDEX IF NOT EXISTS repository_commit_recent_visits_user_viewed_idx
ON repository_commit_recent_visits (user_id, viewed_at DESC);

CREATE TABLE IF NOT EXISTS repository_commit_status_summaries (
    commit_id uuid PRIMARY KEY REFERENCES commits(id) ON DELETE CASCADE,
    status text NOT NULL DEFAULT 'pending',
    conclusion text,
    total_count bigint NOT NULL DEFAULT 0,
    completed_count bigint NOT NULL DEFAULT 0,
    failed_count bigint NOT NULL DEFAULT 0,
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_commit_status_summaries_status_check
        CHECK (status IN ('pending', 'running', 'completed', 'success', 'failure')),
    CONSTRAINT repository_commit_status_summaries_conclusion_check
        CHECK (conclusion IS NULL OR conclusion IN ('success', 'failure', 'cancelled', 'skipped', 'timed_out')),
    CONSTRAINT repository_commit_status_summaries_counts_nonnegative
        CHECK (total_count >= 0 AND completed_count >= 0 AND failed_count >= 0)
);

CREATE INDEX IF NOT EXISTS repository_commit_status_summaries_status_idx
ON repository_commit_status_summaries (status, updated_at DESC);
