CREATE TABLE repository_ref_files (
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    ref text NOT NULL,
    paths jsonb NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (repository_id, ref),
    CONSTRAINT repository_ref_files_ref_not_blank CHECK (length(trim(ref)) > 0),
    CONSTRAINT repository_ref_files_paths_array CHECK (jsonb_typeof(paths) = 'array')
);

CREATE INDEX repository_ref_files_updated_at_idx
ON repository_ref_files (updated_at DESC);
