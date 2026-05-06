CREATE TABLE IF NOT EXISTS project_charts (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    owner_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    title text NOT NULL,
    description text,
    chart_type text NOT NULL DEFAULT 'burn_up',
    filter text,
    x_field_id uuid REFERENCES project_fields(id) ON DELETE SET NULL,
    y_field_id uuid REFERENCES project_fields(id) ON DELETE SET NULL,
    group_field_id uuid REFERENCES project_fields(id) ON DELETE SET NULL,
    visibility text NOT NULL DEFAULT 'private',
    share_slug text,
    cache_version bigint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT project_charts_title_not_blank CHECK (length(trim(title)) > 0),
    CONSTRAINT project_charts_type_check CHECK (chart_type IN ('burn_up', 'bar', 'line', 'stacked_area', 'number')),
    CONSTRAINT project_charts_visibility_check CHECK (visibility IN ('private', 'project'))
);

CREATE UNIQUE INDEX IF NOT EXISTS project_charts_project_share_slug_unique
ON project_charts (project_id, share_slug)
WHERE share_slug IS NOT NULL;

CREATE INDEX IF NOT EXISTS project_charts_project_visibility_idx
ON project_charts (project_id, visibility, updated_at DESC);

CREATE TRIGGER project_charts_set_updated_at
BEFORE UPDATE ON project_charts
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS project_chart_revisions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    chart_id uuid NOT NULL REFERENCES project_charts(id) ON DELETE CASCADE,
    author_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    configuration jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS project_chart_revisions_chart_latest_idx
ON project_chart_revisions (chart_id, created_at DESC);

CREATE TABLE IF NOT EXISTS project_chart_series_cache (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    chart_id uuid REFERENCES project_charts(id) ON DELETE CASCADE,
    project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    cache_key text NOT NULL,
    range_key text NOT NULL,
    filter text,
    series jsonb NOT NULL DEFAULT '[]'::jsonb,
    data_rows jsonb NOT NULL DEFAULT '[]'::jsonb,
    matching_item_count bigint NOT NULL DEFAULT 0,
    computed_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS project_chart_series_cache_key_unique
ON project_chart_series_cache (project_id, cache_key);

CREATE INDEX IF NOT EXISTS project_chart_series_cache_project_computed_idx
ON project_chart_series_cache (project_id, computed_at DESC);

CREATE INDEX IF NOT EXISTS project_items_project_created_idx
ON project_items (project_id, created_at, updated_at DESC);

CREATE INDEX IF NOT EXISTS project_item_field_values_project_item_idx
ON project_item_field_values (project_item_id);
