ALTER TABLE repositories
    ADD COLUMN IF NOT EXISTS is_template boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS is_mirror boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS can_be_sponsored boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS license_template_slug text REFERENCES license_templates(slug) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS repositories_owner_updated_idx
ON repositories (owner_user_id, updated_at DESC)
WHERE owner_user_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS repositories_owner_name_idx
ON repositories (owner_user_id, lower(name))
WHERE owner_user_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS repositories_license_template_idx
ON repositories (license_template_slug)
WHERE license_template_slug IS NOT NULL;
