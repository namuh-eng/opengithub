CREATE TABLE IF NOT EXISTS wiki_repositories (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    git_storage_kind text NOT NULL DEFAULT 'local_bare',
    git_storage_path text,
    default_branch text NOT NULL DEFAULT 'master',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT wiki_repositories_storage_kind_not_blank CHECK (length(trim(git_storage_kind)) > 0),
    CONSTRAINT wiki_repositories_default_branch_not_blank CHECK (length(trim(default_branch)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS wiki_repositories_repository_unique
ON wiki_repositories (repository_id);

DROP TRIGGER IF EXISTS wiki_repositories_set_updated_at ON wiki_repositories;
CREATE TRIGGER wiki_repositories_set_updated_at
BEFORE UPDATE ON wiki_repositories
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS wiki_pages (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    wiki_repository_id uuid NOT NULL REFERENCES wiki_repositories(id) ON DELETE CASCADE,
    title text NOT NULL,
    slug text NOT NULL,
    path text NOT NULL,
    latest_revision_id uuid,
    is_sidebar boolean NOT NULL DEFAULT false,
    is_footer boolean NOT NULL DEFAULT false,
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT wiki_pages_title_not_blank CHECK (length(trim(title)) > 0),
    CONSTRAINT wiki_pages_slug_not_blank CHECK (length(trim(slug)) > 0),
    CONSTRAINT wiki_pages_path_not_blank CHECK (length(trim(path)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS wiki_pages_repository_slug_unique
ON wiki_pages (wiki_repository_id, lower(slug));

CREATE INDEX IF NOT EXISTS wiki_pages_repository_position_idx
ON wiki_pages (wiki_repository_id, position, lower(title));

DROP TRIGGER IF EXISTS wiki_pages_set_updated_at ON wiki_pages;
CREATE TRIGGER wiki_pages_set_updated_at
BEFORE UPDATE ON wiki_pages
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS wiki_page_revisions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    page_id uuid NOT NULL REFERENCES wiki_pages(id) ON DELETE CASCADE,
    author_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    commit_oid text,
    message text NOT NULL DEFAULT 'Update wiki page',
    markdown text NOT NULL,
    content_sha text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT wiki_page_revisions_markdown_not_blank CHECK (length(trim(markdown)) > 0),
    CONSTRAINT wiki_page_revisions_content_sha_not_blank CHECK (length(trim(content_sha)) > 0)
);

CREATE INDEX IF NOT EXISTS wiki_page_revisions_page_latest_idx
ON wiki_page_revisions (page_id, created_at DESC);

CREATE INDEX IF NOT EXISTS wiki_page_revisions_author_idx
ON wiki_page_revisions (author_user_id, created_at DESC)
WHERE author_user_id IS NOT NULL;

ALTER TABLE wiki_pages
    DROP CONSTRAINT IF EXISTS wiki_pages_latest_revision_fk;

ALTER TABLE wiki_pages
    ADD CONSTRAINT wiki_pages_latest_revision_fk
    FOREIGN KEY (latest_revision_id) REFERENCES wiki_page_revisions(id) ON DELETE SET NULL;

CREATE TABLE IF NOT EXISTS wiki_page_toc_cache (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    page_id uuid NOT NULL REFERENCES wiki_pages(id) ON DELETE CASCADE,
    revision_id uuid NOT NULL REFERENCES wiki_page_revisions(id) ON DELETE CASCADE,
    outline jsonb NOT NULL DEFAULT '[]'::jsonb,
    generated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS wiki_page_toc_cache_revision_unique
ON wiki_page_toc_cache (revision_id);
