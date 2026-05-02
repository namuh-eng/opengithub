CREATE UNLOGGED TABLE rate_limit_buckets (
    bucket_key text NOT NULL,
    token_id text,
    ip inet,
    resource text NOT NULL,
    window_start timestamptz NOT NULL DEFAULT now(),
    request_count bigint NOT NULL DEFAULT 0,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (bucket_key, resource),
    CONSTRAINT rate_limit_buckets_identity_present CHECK (token_id IS NOT NULL OR ip IS NOT NULL),
    CONSTRAINT rate_limit_buckets_resource_not_blank CHECK (length(trim(resource)) > 0),
    CONSTRAINT rate_limit_buckets_request_count_nonnegative CHECK (request_count >= 0)
);

CREATE INDEX rate_limit_buckets_token_idx ON rate_limit_buckets (token_id, resource) WHERE token_id IS NOT NULL;
CREATE INDEX rate_limit_buckets_ip_idx ON rate_limit_buckets (ip, resource) WHERE ip IS NOT NULL;
CREATE INDEX rate_limit_buckets_window_idx ON rate_limit_buckets (window_start);

CREATE TRIGGER rate_limit_buckets_set_updated_at
BEFORE UPDATE ON rate_limit_buckets
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
