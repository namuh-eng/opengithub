ALTER TABLE webhooks
    ALTER COLUMN repository_id DROP NOT NULL,
    ADD COLUMN IF NOT EXISTS organization_id uuid REFERENCES organizations(id) ON DELETE CASCADE;

UPDATE webhooks
SET organization_id = NULL
WHERE organization_id IS NOT NULL AND repository_id IS NOT NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'webhooks_exactly_one_scope_check'
    ) THEN
        ALTER TABLE webhooks
            ADD CONSTRAINT webhooks_exactly_one_scope_check
            CHECK (
                (repository_id IS NOT NULL AND organization_id IS NULL)
                OR (repository_id IS NULL AND organization_id IS NOT NULL)
            );
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS webhooks_organization_active_idx
ON webhooks (organization_id, active)
WHERE organization_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS webhooks_organization_updated_idx
ON webhooks (organization_id, updated_at DESC)
WHERE organization_id IS NOT NULL;
