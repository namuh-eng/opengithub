CREATE TABLE email_deliveries (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    job_lease_id uuid NOT NULL REFERENCES job_leases(id) ON DELETE CASCADE,
    recipient text NOT NULL,
    subject text NOT NULL,
    provider text NOT NULL,
    status text NOT NULL DEFAULT 'queued',
    provider_message_id text,
    error_code text,
    error_message text,
    attempt_count integer NOT NULL DEFAULT 0,
    sent_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT email_deliveries_recipient_not_blank CHECK (length(trim(recipient)) > 0),
    CONSTRAINT email_deliveries_subject_not_blank CHECK (length(trim(subject)) > 0),
    CONSTRAINT email_deliveries_provider_check CHECK (provider IN ('noop', 'log', 'ses')),
    CONSTRAINT email_deliveries_status_check CHECK (status IN ('queued', 'sent', 'failed')),
    CONSTRAINT email_deliveries_attempt_count_non_negative CHECK (attempt_count >= 0)
);

CREATE UNIQUE INDEX email_deliveries_job_lease_unique ON email_deliveries (job_lease_id);
CREATE INDEX email_deliveries_status_updated_idx ON email_deliveries (status, updated_at DESC);
CREATE INDEX email_deliveries_recipient_created_idx ON email_deliveries (lower(recipient), created_at DESC);

CREATE TRIGGER email_deliveries_set_updated_at
BEFORE UPDATE ON email_deliveries
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
