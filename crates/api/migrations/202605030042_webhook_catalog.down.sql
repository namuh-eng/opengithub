ALTER TABLE webhook_deliveries
    DROP COLUMN IF EXISTS redelivery_of,
    DROP COLUMN IF EXISTS duration_ms,
    DROP COLUMN IF EXISTS response_headers,
    DROP COLUMN IF EXISTS request_body,
    DROP COLUMN IF EXISTS request_headers;

ALTER TABLE webhooks
    DROP COLUMN IF EXISTS ssl_verify,
    DROP COLUMN IF EXISTS secret_ciphertext,
    DROP COLUMN IF EXISTS content_type,
    DROP COLUMN IF EXISTS scope_id,
    DROP COLUMN IF EXISTS scope_type;
