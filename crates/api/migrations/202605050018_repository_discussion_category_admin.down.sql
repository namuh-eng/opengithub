DROP INDEX IF EXISTS discussion_categories_repository_section_position_idx;
DROP INDEX IF EXISTS discussion_category_sections_repository_position_idx;

ALTER TABLE discussion_categories
    DROP CONSTRAINT IF EXISTS discussion_categories_format_check,
    DROP COLUMN IF EXISTS template_path,
    DROP COLUMN IF EXISTS is_default,
    DROP COLUMN IF EXISTS format,
    DROP COLUMN IF EXISTS section_id;

DROP TABLE IF EXISTS discussion_category_sections;
