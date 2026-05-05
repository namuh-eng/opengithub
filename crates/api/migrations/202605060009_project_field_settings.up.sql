ALTER TABLE project_fields
    ADD COLUMN IF NOT EXISTS deleted_at timestamptz,
    ADD COLUMN IF NOT EXISTS cache_version bigint NOT NULL DEFAULT 1;

CREATE INDEX IF NOT EXISTS project_fields_project_active_position_idx
ON project_fields (project_id, position)
WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS project_field_options (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_field_id uuid NOT NULL REFERENCES project_fields(id) ON DELETE CASCADE,
    name text NOT NULL,
    color text NOT NULL DEFAULT 'gray',
    position integer NOT NULL DEFAULT 1,
    description text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_field_options_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT project_field_options_color_check CHECK (color IN ('gray', 'red', 'orange', 'yellow', 'green', 'blue', 'purple', 'pink'))
);

CREATE UNIQUE INDEX IF NOT EXISTS project_field_options_field_name_unique
ON project_field_options (project_field_id, lower(name));

CREATE INDEX IF NOT EXISTS project_field_options_field_position_idx
ON project_field_options (project_field_id, position);

CREATE TRIGGER project_field_options_set_updated_at
BEFORE UPDATE ON project_field_options
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS project_iterations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_field_id uuid NOT NULL REFERENCES project_fields(id) ON DELETE CASCADE,
    name text NOT NULL,
    start_date date NOT NULL,
    duration_days integer NOT NULL,
    position integer NOT NULL DEFAULT 1,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_iterations_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT project_iterations_duration_positive CHECK (duration_days > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS project_iterations_field_name_unique
ON project_iterations (project_field_id, lower(name));

CREATE INDEX IF NOT EXISTS project_iterations_field_position_idx
ON project_iterations (project_field_id, position);

CREATE INDEX IF NOT EXISTS project_iterations_field_dates_idx
ON project_iterations (project_field_id, start_date);

CREATE TRIGGER project_iterations_set_updated_at
BEFORE UPDATE ON project_iterations
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS project_iteration_breaks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_field_id uuid NOT NULL REFERENCES project_fields(id) ON DELETE CASCADE,
    name text NOT NULL,
    start_date date NOT NULL,
    duration_days integer NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_iteration_breaks_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT project_iteration_breaks_duration_positive CHECK (duration_days > 0)
);

CREATE INDEX IF NOT EXISTS project_iteration_breaks_field_dates_idx
ON project_iteration_breaks (project_field_id, start_date);

CREATE TRIGGER project_iteration_breaks_set_updated_at
BEFORE UPDATE ON project_iteration_breaks
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
