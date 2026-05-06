DROP INDEX IF EXISTS webhooks_organization_updated_idx;
DROP INDEX IF EXISTS webhooks_organization_active_idx;

ALTER TABLE webhooks
    DROP CONSTRAINT IF EXISTS webhooks_exactly_one_scope_check,
    DROP COLUMN IF EXISTS organization_id;

ALTER TABLE webhooks
    ALTER COLUMN repository_id SET NOT NULL;
