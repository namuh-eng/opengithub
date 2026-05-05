CREATE TABLE IF NOT EXISTS discussion_categories (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    slug text NOT NULL,
    name text NOT NULL,
    emoji text NOT NULL DEFAULT '💬',
    description text,
    position integer NOT NULL DEFAULT 0,
    accepts_answers boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (repository_id, slug)
);

CREATE TABLE IF NOT EXISTS discussions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    category_id uuid NOT NULL REFERENCES discussion_categories(id) ON DELETE RESTRICT,
    number bigint NOT NULL,
    title text NOT NULL,
    body text NOT NULL DEFAULT '',
    state text NOT NULL DEFAULT 'open',
    answered boolean NOT NULL DEFAULT false,
    locked boolean NOT NULL DEFAULT false,
    author_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    answer_comment_id uuid,
    comments_count bigint NOT NULL DEFAULT 0,
    votes_count bigint NOT NULL DEFAULT 0,
    last_activity_at timestamptz NOT NULL DEFAULT now(),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (state IN ('open', 'closed')),
    UNIQUE (repository_id, number)
);

CREATE TABLE IF NOT EXISTS discussion_comments (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    discussion_id uuid NOT NULL REFERENCES discussions(id) ON DELETE CASCADE,
    author_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    body text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS discussion_labels (
    discussion_id uuid NOT NULL REFERENCES discussions(id) ON DELETE CASCADE,
    label_id uuid NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (discussion_id, label_id)
);

CREATE TABLE IF NOT EXISTS discussion_votes (
    discussion_id uuid NOT NULL REFERENCES discussions(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (discussion_id, user_id)
);

CREATE TABLE IF NOT EXISTS discussion_pins (
    discussion_id uuid PRIMARY KEY REFERENCES discussions(id) ON DELETE CASCADE,
    pinned_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS repository_community_links (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    label text NOT NULL,
    href text NOT NULL,
    kind text NOT NULL DEFAULT 'custom',
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS discussion_activity_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    discussion_id uuid NOT NULL REFERENCES discussions(id) ON DELETE CASCADE,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS discussion_categories_repository_position_idx
    ON discussion_categories(repository_id, position, name);
CREATE INDEX IF NOT EXISTS discussions_repository_state_activity_idx
    ON discussions(repository_id, state, last_activity_at DESC);
CREATE INDEX IF NOT EXISTS discussions_repository_category_activity_idx
    ON discussions(repository_id, category_id, last_activity_at DESC);
CREATE INDEX IF NOT EXISTS discussions_repository_votes_idx
    ON discussions(repository_id, votes_count DESC, comments_count DESC);
CREATE INDEX IF NOT EXISTS discussions_title_trgm_idx
    ON discussions USING gin (title gin_trgm_ops);
CREATE INDEX IF NOT EXISTS discussion_comments_discussion_created_idx
    ON discussion_comments(discussion_id, created_at);
CREATE INDEX IF NOT EXISTS discussion_votes_user_idx
    ON discussion_votes(user_id, discussion_id);
CREATE INDEX IF NOT EXISTS discussion_pins_position_idx
    ON discussion_pins(position, created_at);
CREATE INDEX IF NOT EXISTS repository_community_links_position_idx
    ON repository_community_links(repository_id, position, label);
