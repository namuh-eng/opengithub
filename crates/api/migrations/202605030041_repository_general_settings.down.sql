DROP TABLE IF EXISTS repository_settings_audit_events;

ALTER TABLE repositories
    DROP COLUMN IF EXISTS web_commit_signoff_required,
    DROP COLUMN IF EXISTS allow_forking,
    DROP COLUMN IF EXISTS wiki_enabled,
    DROP COLUMN IF EXISTS projects_enabled,
    DROP COLUMN IF EXISTS issues_enabled;
