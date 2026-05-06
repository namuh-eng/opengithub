DROP TABLE IF EXISTS wiki_page_toc_cache;
ALTER TABLE wiki_pages DROP CONSTRAINT IF EXISTS wiki_pages_latest_revision_fk;
DROP TABLE IF EXISTS wiki_page_revisions;
DROP TRIGGER IF EXISTS wiki_pages_set_updated_at ON wiki_pages;
DROP TABLE IF EXISTS wiki_pages;
DROP TRIGGER IF EXISTS wiki_repositories_set_updated_at ON wiki_repositories;
DROP TABLE IF EXISTS wiki_repositories;
