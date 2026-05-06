ALTER TABLE projects
    ADD COLUMN IF NOT EXISTS deleted_at timestamptz,
    ADD COLUMN IF NOT EXISTS closed_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS deleted_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS projects_deleted_at_idx
ON projects (deleted_at)
WHERE deleted_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS projects_closed_by_user_idx
ON projects (closed_by_user_id)
WHERE closed_by_user_id IS NOT NULL;

ALTER TABLE project_repositories
    ADD COLUMN IF NOT EXISTS linked_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS updated_at timestamptz NOT NULL DEFAULT now();

CREATE INDEX IF NOT EXISTS project_repositories_project_type_idx
ON project_repositories (project_id, link_type, created_at DESC);

CREATE TRIGGER project_repositories_set_updated_at
BEFORE UPDATE ON project_repositories
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS project_team_permissions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    team_id uuid NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    role text NOT NULL,
    created_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_team_permissions_role_check CHECK (role IN ('read', 'write', 'admin'))
);

CREATE UNIQUE INDEX IF NOT EXISTS project_team_permissions_project_team_unique
ON project_team_permissions (project_id, team_id);

CREATE INDEX IF NOT EXISTS project_team_permissions_team_idx
ON project_team_permissions (team_id, role);

CREATE TRIGGER project_team_permissions_set_updated_at
BEFORE UPDATE ON project_team_permissions
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS project_readme_revisions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    author_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    body text,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS project_readme_revisions_project_latest_idx
ON project_readme_revisions (project_id, created_at DESC);

CREATE INDEX IF NOT EXISTS project_status_updates_project_status_idx
ON project_status_updates (project_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS audit_events_project_settings_idx
ON audit_events (target_type, target_id, created_at DESC)
WHERE target_type = 'project';
