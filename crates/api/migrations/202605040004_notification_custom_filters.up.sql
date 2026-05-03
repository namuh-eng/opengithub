CREATE TABLE notification_custom_filters (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name text NOT NULL,
    query_string text NOT NULL,
    position integer NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT notification_custom_filters_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT notification_custom_filters_query_not_blank CHECK (length(trim(query_string)) > 0),
    CONSTRAINT notification_custom_filters_position_positive CHECK (position > 0)
);

CREATE UNIQUE INDEX notification_custom_filters_user_name_unique
ON notification_custom_filters (user_id, lower(name));

CREATE UNIQUE INDEX notification_custom_filters_user_position_unique
ON notification_custom_filters (user_id, position);

CREATE INDEX notification_custom_filters_user_updated_idx
ON notification_custom_filters (user_id, position ASC, updated_at DESC);

CREATE TRIGGER notification_custom_filters_set_updated_at
BEFORE UPDATE ON notification_custom_filters
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
