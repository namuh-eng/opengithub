CREATE TABLE gists (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    description text,
    is_public boolean NOT NULL DEFAULT true,
    forked_from_gist_id uuid REFERENCES gists(id) ON DELETE SET NULL,
    git_storage_kind text NOT NULL DEFAULT 'local_bare',
    git_storage_path text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT gists_storage_kind_check CHECK (git_storage_kind IN ('local_bare', 's3_bare'))
);

CREATE INDEX gists_owner_updated_idx ON gists (owner_id, updated_at DESC);
CREATE INDEX gists_public_updated_idx ON gists (updated_at DESC) WHERE is_public = true;
CREATE INDEX gists_forked_from_idx ON gists (forked_from_gist_id);

CREATE TRIGGER gists_set_updated_at
BEFORE UPDATE ON gists
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE gist_files (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    gist_id uuid NOT NULL REFERENCES gists(id) ON DELETE CASCADE,
    filename text NOT NULL,
    language text,
    size_bytes bigint NOT NULL DEFAULT 0,
    content_sha text NOT NULL,
    content text NOT NULL,
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT gist_files_filename_not_blank CHECK (length(trim(filename)) > 0),
    CONSTRAINT gist_files_content_sha_not_blank CHECK (length(trim(content_sha)) > 0),
    CONSTRAINT gist_files_size_non_negative CHECK (size_bytes >= 0)
);

CREATE UNIQUE INDEX gist_files_gist_filename_unique ON gist_files (gist_id, lower(filename));
CREATE INDEX gist_files_gist_position_idx ON gist_files (gist_id, position, filename);
CREATE INDEX gist_files_language_idx ON gist_files (language);

CREATE TRIGGER gist_files_set_updated_at
BEFORE UPDATE ON gist_files
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE gist_revisions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    gist_id uuid NOT NULL REFERENCES gists(id) ON DELETE CASCADE,
    author_user_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    version bigint NOT NULL,
    description text,
    files_snapshot jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT gist_revisions_version_positive CHECK (version > 0)
);

CREATE UNIQUE INDEX gist_revisions_gist_version_unique ON gist_revisions (gist_id, version);
CREATE INDEX gist_revisions_gist_created_idx ON gist_revisions (gist_id, created_at DESC);

CREATE TABLE gist_comments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    gist_id uuid NOT NULL REFERENCES gists(id) ON DELETE CASCADE,
    author_user_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    body text NOT NULL,
    is_minimized boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT gist_comments_body_not_blank CHECK (length(trim(body)) > 0)
);

CREATE INDEX gist_comments_gist_created_idx ON gist_comments (gist_id, created_at);

CREATE TRIGGER gist_comments_set_updated_at
BEFORE UPDATE ON gist_comments
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE gist_stars (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    gist_id uuid NOT NULL REFERENCES gists(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, gist_id)
);

CREATE INDEX gist_stars_gist_idx ON gist_stars (gist_id);

CREATE TABLE gist_forks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    source_gist_id uuid NOT NULL REFERENCES gists(id) ON DELETE CASCADE,
    fork_gist_id uuid NOT NULL REFERENCES gists(id) ON DELETE CASCADE,
    forked_by_user_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT gist_forks_distinct CHECK (source_gist_id <> fork_gist_id)
);

CREATE UNIQUE INDEX gist_forks_source_fork_unique ON gist_forks (source_gist_id, fork_gist_id);
CREATE INDEX gist_forks_source_created_idx ON gist_forks (source_gist_id, created_at DESC);
