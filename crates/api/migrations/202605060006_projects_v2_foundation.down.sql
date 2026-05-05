DROP TABLE IF EXISTS project_items;
DROP TABLE IF EXISTS project_workflows;
DROP TABLE IF EXISTS project_fields;
DROP TABLE IF EXISTS project_views;
DROP TABLE IF EXISTS project_templates;
DROP TABLE IF EXISTS project_status_updates;
DROP TABLE IF EXISTS project_permissions;
DROP TABLE IF EXISTS project_repositories;
DROP TABLE IF EXISTS projects;
ALTER TABLE organization_policy_settings
    DROP COLUMN IF EXISTS projects_enabled;
