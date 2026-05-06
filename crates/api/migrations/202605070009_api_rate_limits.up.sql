CREATE UNLOGGED TABLE IF NOT EXISTS rate_limit_buckets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_type text NOT NULL,
    subject_key text NOT NULL,
    resource text NOT NULL,
    window_start timestamptz NOT NULL,
    request_count integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT rate_limit_buckets_subject_type_check
        CHECK (subject_type IN ('token', 'session', 'ip')),
    CONSTRAINT rate_limit_buckets_subject_key_not_blank
        CHECK (length(trim(subject_key)) > 0),
    CONSTRAINT rate_limit_buckets_resource_check
        CHECK (resource IN ('core', 'search')),
    CONSTRAINT rate_limit_buckets_request_count_non_negative
        CHECK (request_count >= 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS rate_limit_buckets_subject_resource_window_unique
ON rate_limit_buckets (subject_type, subject_key, resource, window_start);

CREATE INDEX IF NOT EXISTS rate_limit_buckets_resource_window_idx
ON rate_limit_buckets (resource, window_start DESC);

CREATE INDEX IF NOT EXISTS rate_limit_buckets_updated_idx
ON rate_limit_buckets (updated_at DESC);

DROP TRIGGER IF EXISTS rate_limit_buckets_set_updated_at ON rate_limit_buckets;
CREATE TRIGGER rate_limit_buckets_set_updated_at
BEFORE UPDATE ON rate_limit_buckets
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
