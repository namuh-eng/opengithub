DROP TABLE IF EXISTS repository_settings_audit_events;

ALTER TABLE repository_merge_settings
    DROP COLUMN IF EXISTS allow_auto_merge;

ALTER TABLE repositories
    DROP COLUMN IF EXISTS web_commit_signoff_required,
    DROP COLUMN IF EXISTS allow_forking,
    DROP COLUMN IF EXISTS has_wiki,
    DROP COLUMN IF EXISTS has_projects,
    DROP COLUMN IF EXISTS has_issues;
