ALTER TABLE users
    ADD COLUMN IF NOT EXISTS bio text,
    ADD COLUMN IF NOT EXISTS company text,
    ADD COLUMN IF NOT EXISTS location text,
    ADD COLUMN IF NOT EXISTS website_url text,
    ADD COLUMN IF NOT EXISTS private_profile boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS achievements_enabled boolean NOT NULL DEFAULT true;

CREATE TABLE profile_readmes (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    body text NOT NULL DEFAULT '',
    rendered_body text,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE profile_pins (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    repository_id uuid REFERENCES repositories(id) ON DELETE CASCADE,
    gist_title text,
    gist_description text,
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT profile_pins_has_target CHECK (repository_id IS NOT NULL OR length(trim(coalesce(gist_title, ''))) > 0),
    CONSTRAINT profile_pins_position_non_negative CHECK (position >= 0)
);

CREATE INDEX profile_pins_user_position_idx ON profile_pins (user_id, position, created_at);

CREATE TABLE profile_contribution_days (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    contribution_date date NOT NULL,
    contribution_count integer NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, contribution_date),
    CONSTRAINT profile_contribution_days_non_negative CHECK (contribution_count >= 0)
);

CREATE INDEX profile_contribution_days_user_date_idx ON profile_contribution_days (user_id, contribution_date DESC);

CREATE TABLE profile_contribution_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    repository_id uuid REFERENCES repositories(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    title text NOT NULL,
    target_href text,
    occurred_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT profile_contribution_events_type_not_blank CHECK (length(trim(event_type)) > 0),
    CONSTRAINT profile_contribution_events_title_not_blank CHECK (length(trim(title)) > 0)
);

CREATE INDEX profile_contribution_events_user_time_idx ON profile_contribution_events (user_id, occurred_at DESC);

CREATE TABLE achievements (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    slug text NOT NULL UNIQUE,
    name text NOT NULL,
    description text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT achievements_slug_not_blank CHECK (length(trim(slug)) > 0),
    CONSTRAINT achievements_name_not_blank CHECK (length(trim(name)) > 0)
);

CREATE TABLE user_achievements (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    achievement_id uuid NOT NULL REFERENCES achievements(id) ON DELETE CASCADE,
    awarded_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, achievement_id)
);

CREATE TABLE user_blocks (
    blocker_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    blocked_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (blocker_user_id, blocked_user_id),
    CONSTRAINT user_blocks_no_self CHECK (blocker_user_id <> blocked_user_id)
);

CREATE INDEX user_blocks_blocked_idx ON user_blocks (blocked_user_id);

CREATE TABLE user_reports (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reported_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason text NOT NULL,
    details text,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT user_reports_no_self CHECK (reporter_user_id <> reported_user_id),
    CONSTRAINT user_reports_reason_not_blank CHECK (length(trim(reason)) > 0)
);

CREATE INDEX user_reports_reported_idx ON user_reports (reported_user_id, created_at DESC);

INSERT INTO achievements (slug, name, description)
VALUES
    ('first-repository', 'First repository', 'Published a first public repository.'),
    ('steady-contributor', 'Steady contributor', 'Contributed on multiple days this year.'),
    ('community-signal', 'Community signal', 'Earned followers or repository stars.')
ON CONFLICT (slug) DO NOTHING;
