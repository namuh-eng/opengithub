DROP INDEX IF EXISTS audit_events_project_settings_idx;
DROP INDEX IF EXISTS project_status_updates_project_status_idx;
DROP TABLE IF EXISTS project_readme_revisions;
DROP TABLE IF EXISTS project_team_permissions;
DROP TRIGGER IF EXISTS project_repositories_set_updated_at ON project_repositories;
DROP INDEX IF EXISTS project_repositories_project_type_idx;
ALTER TABLE project_repositories
    DROP COLUMN IF EXISTS updated_at,
    DROP COLUMN IF EXISTS linked_by_user_id;
DROP INDEX IF EXISTS projects_closed_by_user_idx;
DROP INDEX IF EXISTS projects_deleted_at_idx;
ALTER TABLE projects
    DROP COLUMN IF EXISTS deleted_by_user_id,
    DROP COLUMN IF EXISTS closed_by_user_id,
    DROP COLUMN IF EXISTS deleted_at;
