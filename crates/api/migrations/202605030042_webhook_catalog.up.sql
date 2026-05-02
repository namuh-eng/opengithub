ALTER TABLE webhooks
    ALTER COLUMN repository_id DROP NOT NULL,
    ADD COLUMN IF NOT EXISTS scope_type text NOT NULL DEFAULT 'repository',
    ADD COLUMN IF NOT EXISTS scope_id uuid,
    ADD COLUMN IF NOT EXISTS content_type text NOT NULL DEFAULT 'json',
    ADD COLUMN IF NOT EXISTS secret_ciphertext text,
    ADD COLUMN IF NOT EXISTS ssl_verify boolean NOT NULL DEFAULT true;

UPDATE webhooks
SET scope_id = repository_id,
    secret_ciphertext = COALESCE(secret_ciphertext, secret_hash)
WHERE scope_id IS NULL;

ALTER TABLE webhooks
    ALTER COLUMN scope_id SET NOT NULL;

ALTER TABLE webhooks
    ADD CONSTRAINT webhooks_scope_type_check CHECK (scope_type IN ('repository', 'organization')),
    ADD CONSTRAINT webhooks_content_type_check CHECK (content_type IN ('json', 'form'));

CREATE INDEX IF NOT EXISTS webhooks_scope_active_idx ON webhooks (scope_type, scope_id, active);

ALTER TABLE webhook_deliveries
    ADD COLUMN IF NOT EXISTS request_headers jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS request_body text,
    ADD COLUMN IF NOT EXISTS response_headers jsonb NOT NULL DEFAULT '{}'::jsonb,
    ADD COLUMN IF NOT EXISTS duration_ms integer,
    ADD COLUMN IF NOT EXISTS redelivery_of uuid;

UPDATE webhook_deliveries
SET request_body = COALESCE(request_body, payload::text)
WHERE request_body IS NULL;

ALTER TABLE webhook_deliveries
    ADD CONSTRAINT webhook_deliveries_duration_non_negative CHECK (duration_ms IS NULL OR duration_ms >= 0);
