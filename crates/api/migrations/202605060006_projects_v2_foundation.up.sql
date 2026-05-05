CREATE TABLE IF NOT EXISTS projects (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id uuid REFERENCES users(id) ON DELETE CASCADE,
    owner_organization_id uuid REFERENCES organizations(id) ON DELETE CASCADE,
    number bigint NOT NULL,
    title text NOT NULL,
    short_description text,
    readme text,
    state text NOT NULL DEFAULT 'open',
    visibility text NOT NULL DEFAULT 'private',
    is_template boolean NOT NULL DEFAULT false,
    default_repository_id uuid REFERENCES repositories(id) ON DELETE SET NULL,
    created_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    closed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT projects_exactly_one_owner CHECK (
        (owner_user_id IS NOT NULL AND owner_organization_id IS NULL)
        OR (owner_user_id IS NULL AND owner_organization_id IS NOT NULL)
    ),
    CONSTRAINT projects_title_not_blank CHECK (length(trim(title)) > 0),
    CONSTRAINT projects_number_positive CHECK (number > 0),
    CONSTRAINT projects_state_check CHECK (state IN ('open', 'closed')),
    CONSTRAINT projects_visibility_check CHECK (visibility IN ('public', 'private'))
);

ALTER TABLE organization_policy_settings
    ADD COLUMN IF NOT EXISTS projects_enabled boolean NOT NULL DEFAULT true;

CREATE UNIQUE INDEX IF NOT EXISTS projects_user_owner_number_unique
ON projects (owner_user_id, number)
WHERE owner_user_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS projects_org_owner_number_unique
ON projects (owner_organization_id, number)
WHERE owner_organization_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS projects_user_scope_idx
ON projects (owner_user_id, state, updated_at DESC)
WHERE owner_user_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS projects_org_scope_idx
ON projects (owner_organization_id, state, updated_at DESC)
WHERE owner_organization_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS projects_title_trgm_idx
ON projects USING gin (title gin_trgm_ops);

CREATE TRIGGER projects_set_updated_at
BEFORE UPDATE ON projects
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS project_repositories (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    link_type text NOT NULL DEFAULT 'linked',
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_repositories_link_type_check CHECK (link_type IN ('default', 'linked'))
);

CREATE UNIQUE INDEX IF NOT EXISTS project_repositories_project_repo_unique
ON project_repositories (project_id, repository_id);

CREATE INDEX IF NOT EXISTS project_repositories_repository_idx
ON project_repositories (repository_id, project_id);

CREATE TABLE IF NOT EXISTS project_permissions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role text NOT NULL,
    source text NOT NULL DEFAULT 'direct',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_permissions_role_check CHECK (role IN ('read', 'write', 'admin')),
    CONSTRAINT project_permissions_source_check CHECK (source IN ('owner', 'organization', 'team', 'direct'))
);

CREATE UNIQUE INDEX IF NOT EXISTS project_permissions_project_user_unique
ON project_permissions (project_id, user_id);

CREATE INDEX IF NOT EXISTS project_permissions_user_idx
ON project_permissions (user_id, role);

CREATE TRIGGER project_permissions_set_updated_at
BEFORE UPDATE ON project_permissions
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS project_status_updates (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    author_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    status text NOT NULL,
    body text,
    start_date date,
    target_date date,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_status_updates_status_check CHECK (status IN ('on_track', 'at_risk', 'off_track', 'complete'))
);

CREATE INDEX IF NOT EXISTS project_status_updates_latest_idx
ON project_status_updates (project_id, created_at DESC);

CREATE TABLE IF NOT EXISTS project_templates (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title text NOT NULL,
    description text,
    is_public boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_templates_title_not_blank CHECK (length(trim(title)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS project_templates_project_unique
ON project_templates (project_id);

CREATE TABLE IF NOT EXISTS project_views (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name text NOT NULL,
    layout text NOT NULL DEFAULT 'table',
    position integer NOT NULL DEFAULT 1,
    configuration jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_views_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT project_views_layout_check CHECK (layout IN ('table', 'board', 'roadmap'))
);

CREATE INDEX IF NOT EXISTS project_views_project_position_idx
ON project_views (project_id, position);

CREATE TRIGGER project_views_set_updated_at
BEFORE UPDATE ON project_views
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS project_fields (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name text NOT NULL,
    field_type text NOT NULL,
    position integer NOT NULL DEFAULT 1,
    settings jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_fields_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT project_fields_type_check CHECK (field_type IN ('title', 'assignees', 'labels', 'milestone', 'repository', 'status', 'single_select', 'iteration', 'date', 'text', 'number'))
);

CREATE INDEX IF NOT EXISTS project_fields_project_position_idx
ON project_fields (project_id, position);

CREATE TRIGGER project_fields_set_updated_at
BEFORE UPDATE ON project_fields
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS project_workflows (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name text NOT NULL,
    enabled boolean NOT NULL DEFAULT false,
    trigger_event text NOT NULL,
    configuration jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_workflows_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT project_workflows_trigger_not_blank CHECK (length(trim(trigger_event)) > 0)
);

CREATE INDEX IF NOT EXISTS project_workflows_project_idx
ON project_workflows (project_id, enabled);

CREATE TRIGGER project_workflows_set_updated_at
BEFORE UPDATE ON project_workflows
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS project_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    item_type text NOT NULL DEFAULT 'draft_issue',
    issue_id uuid,
    pull_request_id uuid,
    title text,
    body text,
    position numeric NOT NULL DEFAULT 0,
    archived_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_items_type_check CHECK (item_type IN ('draft_issue', 'issue', 'pull_request')),
    CONSTRAINT project_items_draft_title CHECK (item_type <> 'draft_issue' OR length(trim(coalesce(title, ''))) > 0)
);

CREATE INDEX IF NOT EXISTS project_items_project_active_idx
ON project_items (project_id, position)
WHERE archived_at IS NULL;

CREATE TRIGGER project_items_set_updated_at
BEFORE UPDATE ON project_items
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
