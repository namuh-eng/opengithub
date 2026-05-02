ALTER TABLE webhooks
    ADD COLUMN IF NOT EXISTS content_type text NOT NULL DEFAULT 'application/json',
    ADD COLUMN IF NOT EXISTS ssl_verify boolean NOT NULL DEFAULT true,
    ADD COLUMN IF NOT EXISTS event_selection text NOT NULL DEFAULT 'selected',
    ADD COLUMN IF NOT EXISTS secret_configured boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS secret_updated_at timestamptz,
    ADD COLUMN IF NOT EXISTS disabled_reason text;

UPDATE webhooks
SET secret_configured = secret_hash IS NOT NULL,
    secret_updated_at = CASE WHEN secret_hash IS NULL THEN secret_updated_at ELSE COALESCE(secret_updated_at, updated_at) END,
    event_selection = CASE
        WHEN events = ARRAY['*']::text[] THEN 'everything'
        WHEN events = ARRAY['push']::text[] THEN 'push'
        ELSE 'selected'
    END;

ALTER TABLE webhooks
    ADD CONSTRAINT webhooks_content_type_check
        CHECK (content_type IN ('application/json', 'application/x-www-form-urlencoded')),
    ADD CONSTRAINT webhooks_event_selection_check
        CHECK (event_selection IN ('push', 'everything', 'selected'));

CREATE INDEX IF NOT EXISTS webhooks_repository_updated_idx
ON webhooks (repository_id, updated_at DESC);

ALTER TABLE webhook_deliveries
    ADD COLUMN IF NOT EXISTS delivery_guid uuid NOT NULL DEFAULT gen_random_uuid(),
    ADD COLUMN IF NOT EXISTS request_headers jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS request_body_excerpt text,
    ADD COLUMN IF NOT EXISTS request_body_storage_key text,
    ADD COLUMN IF NOT EXISTS response_headers jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS response_body_storage_key text,
    ADD COLUMN IF NOT EXISTS duration_ms bigint,
    ADD COLUMN IF NOT EXISTS redelivery_of_id uuid REFERENCES webhook_deliveries(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS terminal_error text;

CREATE UNIQUE INDEX IF NOT EXISTS webhook_deliveries_guid_unique
ON webhook_deliveries (delivery_guid);

CREATE INDEX IF NOT EXISTS webhook_deliveries_redelivery_idx
ON webhook_deliveries (redelivery_of_id)
WHERE redelivery_of_id IS NOT NULL;
