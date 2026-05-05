CREATE TABLE IF NOT EXISTS discussion_category_sections (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name text NOT NULL,
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (repository_id, name),
    CONSTRAINT discussion_category_sections_name_not_blank CHECK (length(trim(name)) > 0)
);

ALTER TABLE discussion_categories
    ADD COLUMN IF NOT EXISTS section_id uuid REFERENCES discussion_category_sections(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS format text NOT NULL DEFAULT 'question_and_answer',
    ADD COLUMN IF NOT EXISTS is_default boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS template_path text;

ALTER TABLE discussion_categories
    DROP CONSTRAINT IF EXISTS discussion_categories_format_check;

ALTER TABLE discussion_categories
    ADD CONSTRAINT discussion_categories_format_check
    CHECK (format IN ('announcement', 'open_ended', 'poll', 'question_and_answer'));

CREATE INDEX IF NOT EXISTS discussion_category_sections_repository_position_idx
    ON discussion_category_sections(repository_id, position, name);
CREATE INDEX IF NOT EXISTS discussion_categories_repository_section_position_idx
    ON discussion_categories(repository_id, section_id, position, name);

UPDATE discussion_categories
SET format = 'poll', accepts_answers = false
WHERE slug IN ('poll', 'polls');

UPDATE discussion_categories
SET format = 'question_and_answer', accepts_answers = true
WHERE slug IN ('q-a', 'qa', 'question-and-answer', 'questions');
