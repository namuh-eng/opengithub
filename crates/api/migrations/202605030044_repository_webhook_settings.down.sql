DROP INDEX IF EXISTS webhook_deliveries_redelivery_idx;
DROP INDEX IF EXISTS webhook_deliveries_guid_unique;

ALTER TABLE webhook_deliveries
    DROP COLUMN IF EXISTS terminal_error,
    DROP COLUMN IF EXISTS redelivery_of_id,
    DROP COLUMN IF EXISTS duration_ms,
    DROP COLUMN IF EXISTS response_body_storage_key,
    DROP COLUMN IF EXISTS response_headers,
    DROP COLUMN IF EXISTS request_body_storage_key,
    DROP COLUMN IF EXISTS request_body_excerpt,
    DROP COLUMN IF EXISTS request_headers,
    DROP COLUMN IF EXISTS delivery_guid;

DROP INDEX IF EXISTS webhooks_repository_updated_idx;

ALTER TABLE webhooks
    DROP CONSTRAINT IF EXISTS webhooks_event_selection_check,
    DROP CONSTRAINT IF EXISTS webhooks_content_type_check,
    DROP COLUMN IF EXISTS disabled_reason,
    DROP COLUMN IF EXISTS secret_updated_at,
    DROP COLUMN IF EXISTS secret_configured,
    DROP COLUMN IF EXISTS event_selection,
    DROP COLUMN IF EXISTS ssl_verify,
    DROP COLUMN IF EXISTS content_type;
