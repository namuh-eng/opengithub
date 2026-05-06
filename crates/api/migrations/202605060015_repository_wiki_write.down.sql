DROP INDEX IF EXISTS repository_activity_events_type_created_idx;
DROP INDEX IF EXISTS repository_activity_events_repo_created_idx;
DROP TABLE IF EXISTS repository_activity_events;
DROP INDEX IF EXISTS wiki_git_commits_repository_created_idx;
DROP INDEX IF EXISTS wiki_git_commits_oid_unique;
DROP TABLE IF EXISTS wiki_git_commits;
ALTER TABLE wiki_repositories
    DROP COLUMN IF EXISTS latest_published_at,
    DROP COLUMN IF EXISTS latest_commit_oid;
