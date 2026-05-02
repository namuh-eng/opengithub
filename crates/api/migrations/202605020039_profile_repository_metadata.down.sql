DROP INDEX IF EXISTS repositories_license_template_idx;
DROP INDEX IF EXISTS repositories_owner_name_idx;
DROP INDEX IF EXISTS repositories_owner_updated_idx;

ALTER TABLE repositories
    DROP COLUMN IF EXISTS license_template_slug,
    DROP COLUMN IF EXISTS can_be_sponsored,
    DROP COLUMN IF EXISTS is_mirror,
    DROP COLUMN IF EXISTS is_template;
