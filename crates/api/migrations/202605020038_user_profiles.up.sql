ALTER TABLE users
    ADD COLUMN IF NOT EXISTS bio text,
    ADD COLUMN IF NOT EXISTS company text,
    ADD COLUMN IF NOT EXISTS location text,
    ADD COLUMN IF NOT EXISTS website_url text,
    ADD COLUMN IF NOT EXISTS profile_visibility text NOT NULL DEFAULT 'public';

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'users_profile_visibility_check'
    ) THEN
        ALTER TABLE users
            ADD CONSTRAINT users_profile_visibility_check
            CHECK (profile_visibility IN ('public', 'private'));
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS user_profile_readmes (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    body text NOT NULL DEFAULT '',
    rendered_html text,
    updated_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TRIGGER user_profile_readmes_set_updated_at
BEFORE UPDATE ON user_profile_readmes
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS profile_pins (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    position integer NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT profile_pins_position_positive CHECK (position > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS profile_pins_user_repository_unique
ON profile_pins (user_id, repository_id);
CREATE UNIQUE INDEX IF NOT EXISTS profile_pins_user_position_unique
ON profile_pins (user_id, position);
CREATE INDEX IF NOT EXISTS profile_pins_user_position_idx
ON profile_pins (user_id, position ASC);

CREATE TRIGGER profile_pins_set_updated_at
BEFORE UPDATE ON profile_pins
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS profile_contribution_days (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    day date NOT NULL,
    contribution_count integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, day),
    CONSTRAINT profile_contribution_days_count_non_negative CHECK (contribution_count >= 0)
);

CREATE INDEX IF NOT EXISTS profile_contribution_days_user_day_desc_idx
ON profile_contribution_days (user_id, day DESC);

CREATE TRIGGER profile_contribution_days_set_updated_at
BEFORE UPDATE ON profile_contribution_days
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS profile_contribution_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    repository_id uuid REFERENCES repositories(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    title text NOT NULL,
    target_href text,
    occurred_at timestamptz NOT NULL DEFAULT now(),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT profile_contribution_events_type_not_blank CHECK (length(trim(event_type)) > 0),
    CONSTRAINT profile_contribution_events_title_not_blank CHECK (length(trim(title)) > 0)
);

CREATE INDEX IF NOT EXISTS profile_contribution_events_user_occurred_idx
ON profile_contribution_events (user_id, occurred_at DESC);

CREATE TABLE IF NOT EXISTS achievements (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    slug text NOT NULL,
    name text NOT NULL,
    description text,
    icon text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT achievements_slug_not_blank CHECK (length(trim(slug)) > 0),
    CONSTRAINT achievements_name_not_blank CHECK (length(trim(name)) > 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS achievements_slug_lower_unique
ON achievements (lower(slug));

CREATE TRIGGER achievements_set_updated_at
BEFORE UPDATE ON achievements
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE IF NOT EXISTS user_achievements (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    achievement_id uuid NOT NULL REFERENCES achievements(id) ON DELETE CASCADE,
    awarded_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, achievement_id)
);

CREATE INDEX IF NOT EXISTS user_achievements_user_awarded_idx
ON user_achievements (user_id, awarded_at DESC);

CREATE TABLE IF NOT EXISTS user_blocks (
    blocker_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    blocked_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason text,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (blocker_user_id, blocked_user_id),
    CONSTRAINT user_blocks_no_self CHECK (blocker_user_id <> blocked_user_id)
);

CREATE INDEX IF NOT EXISTS user_blocks_blocked_user_idx
ON user_blocks (blocked_user_id);

CREATE TABLE IF NOT EXISTS user_reports (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    reported_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason text NOT NULL,
    details text,
    status text NOT NULL DEFAULT 'open',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT user_reports_reason_not_blank CHECK (length(trim(reason)) > 0),
    CONSTRAINT user_reports_status_check CHECK (status IN ('open', 'reviewed', 'dismissed'))
);

CREATE INDEX IF NOT EXISTS user_reports_reported_status_idx
ON user_reports (reported_user_id, status, created_at DESC);

CREATE TRIGGER user_reports_set_updated_at
BEFORE UPDATE ON user_reports
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
