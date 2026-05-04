ALTER TABLE teams
    ADD COLUMN IF NOT EXISTS parent_team_id uuid REFERENCES teams(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS visibility text NOT NULL DEFAULT 'visible',
    ADD COLUMN IF NOT EXISTS notifications_enabled boolean NOT NULL DEFAULT true;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'teams_visibility_check'
    ) THEN
        ALTER TABLE teams
            ADD CONSTRAINT teams_visibility_check
            CHECK (visibility IN ('visible', 'secret'));
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'teams_parent_not_self'
    ) THEN
        ALTER TABLE teams
            ADD CONSTRAINT teams_parent_not_self
            CHECK (parent_team_id IS NULL OR parent_team_id <> id);
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS teams_parent_team_id_idx
ON teams (parent_team_id)
WHERE parent_team_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS teams_org_visibility_idx
ON teams (organization_id, visibility, updated_at DESC);

CREATE INDEX IF NOT EXISTS teams_org_name_trgm_idx
ON teams USING gin (name gin_trgm_ops);

CREATE INDEX IF NOT EXISTS teams_org_slug_trgm_idx
ON teams USING gin (slug gin_trgm_ops);

CREATE TABLE IF NOT EXISTS organization_team_mentions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id uuid NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    team_id uuid NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    source_kind text NOT NULL,
    source_id uuid NOT NULL,
    mentioned_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    notification_status text NOT NULL DEFAULT 'pending',
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT organization_team_mentions_source_kind_check
        CHECK (source_kind IN ('issue', 'pull_request', 'comment', 'review_request')),
    CONSTRAINT organization_team_mentions_notification_status_check
        CHECK (notification_status IN ('pending', 'sent', 'suppressed', 'failed'))
);

CREATE INDEX IF NOT EXISTS organization_team_mentions_org_team_created_idx
ON organization_team_mentions (organization_id, team_id, created_at DESC);

CREATE INDEX IF NOT EXISTS organization_team_mentions_source_idx
ON organization_team_mentions (source_kind, source_id);

CREATE INDEX IF NOT EXISTS organization_audit_events_org_team_type_created_idx
ON organization_audit_events (organization_id, event_type, created_at DESC)
WHERE event_type LIKE 'organization.team.%';
