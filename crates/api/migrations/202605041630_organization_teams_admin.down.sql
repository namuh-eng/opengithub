DROP INDEX IF EXISTS organization_audit_events_org_team_type_created_idx;
DROP TABLE IF EXISTS organization_team_mentions;

DROP INDEX IF EXISTS teams_org_slug_trgm_idx;
DROP INDEX IF EXISTS teams_org_name_trgm_idx;
DROP INDEX IF EXISTS teams_org_visibility_idx;
DROP INDEX IF EXISTS teams_parent_team_id_idx;

ALTER TABLE teams
    DROP CONSTRAINT IF EXISTS teams_parent_not_self,
    DROP CONSTRAINT IF EXISTS teams_visibility_check,
    DROP COLUMN IF EXISTS notifications_enabled,
    DROP COLUMN IF EXISTS visibility,
    DROP COLUMN IF EXISTS parent_team_id;
