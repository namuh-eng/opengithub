DROP INDEX IF EXISTS repository_popular_content_daily_repo_path_idx;
DROP INDEX IF EXISTS repository_popular_content_daily_repo_date_idx;
DROP TABLE IF EXISTS repository_popular_content_daily;

DROP INDEX IF EXISTS repository_referrers_daily_repo_referrer_idx;
DROP INDEX IF EXISTS repository_referrers_daily_repo_date_idx;
DROP TABLE IF EXISTS repository_referrers_daily;

DROP INDEX IF EXISTS repository_traffic_daily_repo_date_idx;
DROP TABLE IF EXISTS repository_traffic_daily;

DELETE FROM repository_insight_snapshots WHERE period_key = '14d';
DELETE FROM recent_insight_views WHERE period_key = '14d';

ALTER TABLE repository_insight_snapshots
DROP CONSTRAINT IF EXISTS repository_insight_snapshots_period_key_check;

ALTER TABLE repository_insight_snapshots
ADD CONSTRAINT repository_insight_snapshots_period_key_check
    CHECK (period_key IN ('24h', '3d', '1w', '1m'));

ALTER TABLE recent_insight_views
DROP CONSTRAINT IF EXISTS recent_insight_views_period_key_check;

ALTER TABLE recent_insight_views
ADD CONSTRAINT recent_insight_views_period_key_check
    CHECK (period_key IN ('24h', '3d', '1w', '1m'));
