ALTER TABLE repository_git_refs
    ADD COLUMN IF NOT EXISTS verified boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS signature_summary text,
    ADD COLUMN IF NOT EXISTS signed_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS signature_metadata jsonb NOT NULL DEFAULT '{}'::jsonb;

CREATE INDEX IF NOT EXISTS repository_git_refs_verified_tag_idx
ON repository_git_refs (repository_id, verified, updated_at DESC)
WHERE kind = 'tag';

ALTER TABLE repository_archives
    DROP CONSTRAINT IF EXISTS repository_archives_format_check,
    ADD CONSTRAINT repository_archives_format_check CHECK (format IN ('zip', 'tar'));
