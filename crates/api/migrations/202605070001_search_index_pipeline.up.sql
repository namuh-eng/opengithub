CREATE TABLE search_index_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type text NOT NULL,
    repository_id uuid REFERENCES repositories(id) ON DELETE CASCADE,
    resource_kind text NOT NULL,
    resource_id text NOT NULL,
    status text NOT NULL DEFAULT 'completed',
    attempts integer NOT NULL DEFAULT 1,
    last_error text,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    completed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT search_index_events_event_type_not_blank CHECK (length(trim(event_type)) > 0),
    CONSTRAINT search_index_events_resource_kind_not_blank CHECK (length(trim(resource_kind)) > 0),
    CONSTRAINT search_index_events_resource_id_not_blank CHECK (length(trim(resource_id)) > 0),
    CONSTRAINT search_index_events_status_check CHECK (status IN ('queued', 'running', 'completed', 'failed')),
    CONSTRAINT search_index_events_attempts_positive CHECK (attempts > 0)
);

CREATE INDEX search_index_events_repository_created_idx ON search_index_events (repository_id, created_at DESC);
CREATE INDEX search_index_events_status_created_idx ON search_index_events (status, created_at DESC);
CREATE INDEX search_index_events_resource_idx ON search_index_events (resource_kind, resource_id);

CREATE TRIGGER search_index_events_set_updated_at
BEFORE UPDATE ON search_index_events
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
