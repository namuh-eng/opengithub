DROP INDEX IF EXISTS audit_events_discussion_moderation_idx;
DROP INDEX IF EXISTS discussion_activity_events_discussion_type_created_idx;
DROP INDEX IF EXISTS discussions_repository_locked_idx;

ALTER TABLE discussions
    DROP CONSTRAINT IF EXISTS discussions_closed_reason_check,
    DROP COLUMN IF EXISTS closed_reason,
    DROP COLUMN IF EXISTS lock_allows_reactions,
    DROP COLUMN IF EXISTS locked_by_user_id,
    DROP COLUMN IF EXISTS locked_at;

DROP INDEX IF EXISTS discussion_pins_scope_lookup_idx;
DROP INDEX IF EXISTS discussion_pins_category_position_idx;
DROP INDEX IF EXISTS discussion_pins_global_position_idx;
DROP INDEX IF EXISTS discussion_pins_discussion_scope_category_idx;
DROP INDEX IF EXISTS discussion_pins_discussion_scope_global_idx;

DELETE FROM discussion_pins
WHERE pin_scope <> 'global';

ALTER TABLE discussion_pins
    DROP CONSTRAINT IF EXISTS discussion_pins_scope_check,
    DROP COLUMN IF EXISTS updated_at,
    DROP COLUMN IF EXISTS custom_body,
    DROP COLUMN IF EXISTS custom_title,
    DROP COLUMN IF EXISTS category_id,
    DROP COLUMN IF EXISTS pin_scope;

ALTER TABLE discussion_pins
    ADD PRIMARY KEY (discussion_id);
