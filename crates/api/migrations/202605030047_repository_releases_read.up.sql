ALTER TABLE releases
    ADD COLUMN IF NOT EXISTS target_commit_id uuid REFERENCES commits(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS body_html text NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS rendered_body_excerpt text,
    ADD COLUMN IF NOT EXISTS is_latest boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS tag_verified boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS tag_signature_summary text,
    ADD COLUMN IF NOT EXISTS immutable boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS deleted_at timestamptz,
    ADD COLUMN IF NOT EXISTS updated_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS metadata jsonb NOT NULL DEFAULT '{}'::jsonb;

UPDATE releases
SET rendered_body_excerpt = left(regexp_replace(COALESCE(body, ''), '\s+', ' ', 'g'), 280)
WHERE rendered_body_excerpt IS NULL;

CREATE INDEX IF NOT EXISTS releases_repository_published_visible_idx
ON releases (repository_id, published_at DESC NULLS LAST, created_at DESC)
WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS releases_repository_target_commit_idx
ON releases (repository_id, target_commit_id)
WHERE target_commit_id IS NOT NULL;

CREATE TABLE release_assets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    release_id uuid NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name text NOT NULL,
    label text,
    content_type text NOT NULL DEFAULT 'application/octet-stream',
    byte_size bigint NOT NULL DEFAULT 0,
    storage_kind text NOT NULL DEFAULT 'local',
    storage_key text NOT NULL,
    checksum_sha256 text,
    download_count bigint NOT NULL DEFAULT 0,
    uploaded_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz,
    CONSTRAINT release_assets_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT release_assets_storage_key_not_blank CHECK (length(trim(storage_key)) > 0),
    CONSTRAINT release_assets_byte_size_non_negative CHECK (byte_size >= 0),
    CONSTRAINT release_assets_download_count_non_negative CHECK (download_count >= 0),
    CONSTRAINT release_assets_storage_kind_check CHECK (storage_kind IN ('local', 's3'))
);

CREATE UNIQUE INDEX release_assets_release_name_active_unique
ON release_assets (release_id, lower(name))
WHERE deleted_at IS NULL;
CREATE INDEX release_assets_repository_release_idx
ON release_assets (repository_id, release_id, created_at);

CREATE TRIGGER release_assets_set_updated_at
BEFORE UPDATE ON release_assets
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE release_reactions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    release_id uuid NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reaction text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT release_reactions_reaction_check CHECK (
        reaction IN ('thumbs_up', 'thumbs_down', 'laugh', 'hooray', 'confused', 'heart', 'rocket', 'eyes')
    )
);

CREATE UNIQUE INDEX release_reactions_release_user_reaction_unique
ON release_reactions (release_id, user_id, reaction);
CREATE INDEX release_reactions_repository_release_idx
ON release_reactions (repository_id, release_id);

CREATE TABLE release_downloads (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    release_id uuid NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
    asset_id uuid REFERENCES release_assets(id) ON DELETE SET NULL,
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    source text NOT NULL DEFAULT 'asset',
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT release_downloads_source_check CHECK (source IN ('asset', 'zipball', 'tarball'))
);

CREATE INDEX release_downloads_release_created_idx
ON release_downloads (release_id, created_at DESC);
CREATE INDEX release_downloads_asset_created_idx
ON release_downloads (asset_id, created_at DESC)
WHERE asset_id IS NOT NULL;

CREATE TABLE release_audit_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    release_id uuid REFERENCES releases(id) ON DELETE SET NULL,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    changed_fields text[] NOT NULL DEFAULT ARRAY[]::text[],
    before_state jsonb NOT NULL DEFAULT '{}'::jsonb,
    after_state jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT release_audit_events_type_not_blank CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX release_audit_events_repository_created_idx
ON release_audit_events (repository_id, created_at DESC);
CREATE INDEX release_audit_events_release_created_idx
ON release_audit_events (release_id, created_at DESC)
WHERE release_id IS NOT NULL;
