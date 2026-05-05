CREATE TABLE IF NOT EXISTS project_board_column_settings (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_view_id uuid NOT NULL REFERENCES project_views(id) ON DELETE CASCADE,
    project_field_id uuid NOT NULL REFERENCES project_fields(id) ON DELETE CASCADE,
    option_key text NOT NULL,
    label text NOT NULL,
    position integer NOT NULL DEFAULT 1,
    item_limit integer,
    visible boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_board_column_settings_key_not_blank CHECK (length(trim(option_key)) > 0),
    CONSTRAINT project_board_column_settings_label_not_blank CHECK (length(trim(label)) > 0),
    CONSTRAINT project_board_column_settings_limit_positive CHECK (item_limit IS NULL OR item_limit > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS project_board_column_settings_view_field_key_unique
ON project_board_column_settings (project_view_id, project_field_id, option_key);

CREATE INDEX IF NOT EXISTS project_board_column_settings_view_position_idx
ON project_board_column_settings (project_view_id, position);

CREATE TRIGGER project_board_column_settings_set_updated_at
BEFORE UPDATE ON project_board_column_settings
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS project_roadmap_settings (
    project_view_id uuid PRIMARY KEY REFERENCES project_views(id) ON DELETE CASCADE,
    start_field_id uuid REFERENCES project_fields(id) ON DELETE SET NULL,
    target_field_id uuid REFERENCES project_fields(id) ON DELETE SET NULL,
    marker_field_ids uuid[] NOT NULL DEFAULT ARRAY[]::uuid[],
    zoom text NOT NULL DEFAULT 'month',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_roadmap_settings_zoom_check CHECK (zoom IN ('month', 'quarter', 'year'))
);

CREATE INDEX IF NOT EXISTS project_roadmap_settings_start_field_idx
ON project_roadmap_settings (start_field_id)
WHERE start_field_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS project_roadmap_settings_target_field_idx
ON project_roadmap_settings (target_field_id)
WHERE target_field_id IS NOT NULL;

CREATE TRIGGER project_roadmap_settings_set_updated_at
BEFORE UPDATE ON project_roadmap_settings
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
