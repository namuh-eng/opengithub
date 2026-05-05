DROP TABLE IF EXISTS project_iteration_breaks;
DROP TABLE IF EXISTS project_iterations;
DROP TABLE IF EXISTS project_field_options;

DROP INDEX IF EXISTS project_fields_project_active_position_idx;

ALTER TABLE project_fields
    DROP COLUMN IF EXISTS cache_version,
    DROP COLUMN IF EXISTS deleted_at;
