ALTER TABLE discussion_pins
    DROP CONSTRAINT IF EXISTS discussion_pins_pkey;

ALTER TABLE discussion_pins
    ADD COLUMN IF NOT EXISTS pin_scope text NOT NULL DEFAULT 'global',
    ADD COLUMN IF NOT EXISTS category_id uuid REFERENCES discussion_categories(id) ON DELETE CASCADE,
    ADD COLUMN IF NOT EXISTS custom_title text,
    ADD COLUMN IF NOT EXISTS custom_body text,
    ADD COLUMN IF NOT EXISTS updated_at timestamptz NOT NULL DEFAULT now();

ALTER TABLE discussion_pins
    DROP CONSTRAINT IF EXISTS discussion_pins_scope_check;

ALTER TABLE discussion_pins
    ADD CONSTRAINT discussion_pins_scope_check
    CHECK (pin_scope IN ('global', 'category'));

UPDATE discussion_pins
SET pin_scope = 'global'
WHERE pin_scope IS NULL OR pin_scope = '';

DELETE FROM discussion_pins
WHERE pin_scope = 'category' AND category_id IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS discussion_pins_discussion_scope_global_idx
    ON discussion_pins(discussion_id)
    WHERE pin_scope = 'global';

CREATE UNIQUE INDEX IF NOT EXISTS discussion_pins_discussion_scope_category_idx
    ON discussion_pins(discussion_id, category_id)
    WHERE pin_scope = 'category';

CREATE UNIQUE INDEX IF NOT EXISTS discussion_pins_global_position_idx
    ON discussion_pins(discussion_id, position)
    WHERE pin_scope = 'global';

CREATE UNIQUE INDEX IF NOT EXISTS discussion_pins_category_position_idx
    ON discussion_pins(category_id, position)
    WHERE pin_scope = 'category';

CREATE INDEX IF NOT EXISTS discussion_pins_scope_lookup_idx
    ON discussion_pins(pin_scope, category_id, position, created_at DESC);

ALTER TABLE discussions
    ADD COLUMN IF NOT EXISTS locked_at timestamptz,
    ADD COLUMN IF NOT EXISTS locked_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS lock_allows_reactions boolean NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS closed_reason text;

ALTER TABLE discussions
    DROP CONSTRAINT IF EXISTS discussions_closed_reason_check;

ALTER TABLE discussions
    ADD CONSTRAINT discussions_closed_reason_check
    CHECK (closed_reason IS NULL OR closed_reason IN ('resolved', 'duplicate', 'outdated', 'off-topic'));

CREATE INDEX IF NOT EXISTS discussions_repository_locked_idx
    ON discussions(repository_id, locked, locked_at DESC);

CREATE INDEX IF NOT EXISTS discussion_activity_events_discussion_type_created_idx
    ON discussion_activity_events(discussion_id, event_type, created_at DESC);

CREATE INDEX IF NOT EXISTS audit_events_discussion_moderation_idx
    ON audit_events(event_type, target_type, created_at DESC)
    WHERE target_type = 'repository_discussion';
