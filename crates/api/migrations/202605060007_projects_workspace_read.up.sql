CREATE TABLE IF NOT EXISTS project_item_field_values (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_item_id uuid NOT NULL REFERENCES project_items(id) ON DELETE CASCADE,
    project_field_id uuid NOT NULL REFERENCES project_fields(id) ON DELETE CASCADE,
    value jsonb NOT NULL DEFAULT 'null'::jsonb,
    updated_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS project_item_field_values_item_field_unique
ON project_item_field_values (project_item_id, project_field_id);

CREATE INDEX IF NOT EXISTS project_item_field_values_field_value_idx
ON project_item_field_values USING gin (value);

CREATE TRIGGER project_item_field_values_set_updated_at
BEFORE UPDATE ON project_item_field_values
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS project_item_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    project_item_id uuid REFERENCES project_items(id) ON DELETE CASCADE,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_item_events_type_not_blank CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX IF NOT EXISTS project_item_events_project_created_idx
ON project_item_events (project_id, created_at DESC);

CREATE INDEX IF NOT EXISTS project_item_events_item_created_idx
ON project_item_events (project_item_id, created_at DESC)
WHERE project_item_id IS NOT NULL;
