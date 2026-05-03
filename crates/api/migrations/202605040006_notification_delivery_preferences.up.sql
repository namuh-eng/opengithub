CREATE TABLE IF NOT EXISTS notification_delivery_preferences (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    preference_key text NOT NULL,
    channels text[] NOT NULL DEFAULT ARRAY['web']::text[],
    default_email_id uuid REFERENCES user_email_addresses(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, preference_key),
    CONSTRAINT notification_delivery_preferences_key_not_blank CHECK (length(trim(preference_key)) > 0),
    CONSTRAINT notification_delivery_preferences_channels_not_empty CHECK (array_length(channels, 1) IS NOT NULL),
    CONSTRAINT notification_delivery_preferences_channels_check CHECK (channels <@ ARRAY['web', 'email', 'cli']::text[])
);

CREATE INDEX IF NOT EXISTS notification_delivery_preferences_user_updated_idx
ON notification_delivery_preferences (user_id, updated_at DESC);

CREATE TRIGGER notification_delivery_preferences_set_updated_at
BEFORE UPDATE ON notification_delivery_preferences
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS notification_email_deliveries (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    email_id uuid REFERENCES user_email_addresses(id) ON DELETE SET NULL,
    preference_key text NOT NULL,
    subject_type text NOT NULL,
    delivery_status text NOT NULL DEFAULT 'queued',
    provider_message_id text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT notification_email_deliveries_status_check CHECK (delivery_status IN ('queued', 'sent', 'degraded', 'failed')),
    CONSTRAINT notification_email_deliveries_preference_key_not_blank CHECK (length(trim(preference_key)) > 0)
);

CREATE INDEX IF NOT EXISTS notification_email_deliveries_user_created_idx
ON notification_email_deliveries (user_id, created_at DESC);

CREATE TRIGGER notification_email_deliveries_set_updated_at
BEFORE UPDATE ON notification_email_deliveries
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
