CREATE TABLE IF NOT EXISTS wiki_assets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    wiki_repository_id uuid NOT NULL REFERENCES wiki_repositories(id) ON DELETE CASCADE,
    page_id uuid REFERENCES wiki_pages(id) ON DELETE CASCADE,
    revision_id uuid REFERENCES wiki_page_revisions(id) ON DELETE CASCADE,
    source_url text NOT NULL,
    alt_text text NOT NULL DEFAULT '',
    storage_kind text NOT NULL DEFAULT 'remote_url',
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT wiki_assets_source_url_not_blank CHECK (length(trim(source_url)) > 0),
    CONSTRAINT wiki_assets_storage_kind_valid CHECK (storage_kind IN ('remote_url'))
);

CREATE INDEX IF NOT EXISTS wiki_assets_page_created_idx
ON wiki_assets (page_id, created_at DESC);

CREATE INDEX IF NOT EXISTS wiki_assets_revision_idx
ON wiki_assets (revision_id);
