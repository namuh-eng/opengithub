CREATE TABLE saved_searches (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name text NOT NULL,
    query text NOT NULL,
    scope text NOT NULL DEFAULT 'all',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT saved_searches_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT saved_searches_query_not_blank CHECK (length(trim(query)) > 0),
    CONSTRAINT saved_searches_scope_not_blank CHECK (length(trim(scope)) > 0)
);

CREATE UNIQUE INDEX saved_searches_user_name_lower_unique
ON saved_searches (user_id, lower(name));
CREATE INDEX saved_searches_user_updated_idx ON saved_searches (user_id, updated_at DESC);
CREATE INDEX saved_searches_query_trgm_idx ON saved_searches USING gin (query gin_trgm_ops);

CREATE TRIGGER saved_searches_set_updated_at
BEFORE UPDATE ON saved_searches
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE recent_searches (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    query text NOT NULL,
    scope text NOT NULL DEFAULT 'all',
    result_type text,
    searched_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT recent_searches_query_not_blank CHECK (length(trim(query)) > 0),
    CONSTRAINT recent_searches_scope_not_blank CHECK (length(trim(scope)) > 0)
);

CREATE UNIQUE INDEX recent_searches_user_query_scope_type_unique
ON recent_searches (user_id, lower(query), scope, COALESCE(result_type, ''));
CREATE INDEX recent_searches_user_searched_idx ON recent_searches (user_id, searched_at DESC);
CREATE INDEX recent_searches_query_trgm_idx ON recent_searches USING gin (query gin_trgm_ops);

CREATE TABLE search_telemetry_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    query text,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT search_telemetry_events_type_not_blank CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX search_telemetry_events_user_created_idx
ON search_telemetry_events (user_id, created_at DESC);
CREATE INDEX search_telemetry_events_type_idx ON search_telemetry_events (event_type);
