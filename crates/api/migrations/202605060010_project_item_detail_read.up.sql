ALTER TABLE project_items
ADD COLUMN IF NOT EXISTS archived_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS restored_at timestamptz,
ADD COLUMN IF NOT EXISTS restored_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS source_synced_at timestamptz,
ADD COLUMN IF NOT EXISTS source_sync_version bigint NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS project_items_project_archived_idx
ON project_items (project_id, archived_at DESC, updated_at DESC)
WHERE archived_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS project_items_project_type_archived_idx
ON project_items (project_id, item_type, archived_at DESC)
WHERE archived_at IS NOT NULL;

CREATE TABLE IF NOT EXISTS project_item_comments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    project_item_id uuid NOT NULL REFERENCES project_items(id) ON DELETE CASCADE,
    author_user_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    body text NOT NULL,
    is_deleted boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_item_comments_body_not_blank CHECK (length(trim(body)) > 0)
);

CREATE INDEX IF NOT EXISTS project_item_comments_item_created_idx
ON project_item_comments (project_item_id, created_at);

CREATE TRIGGER project_item_comments_set_updated_at
BEFORE UPDATE ON project_item_comments
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
