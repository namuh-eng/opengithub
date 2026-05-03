DROP INDEX IF EXISTS repository_git_refs_verified_tag_idx;

ALTER TABLE repository_git_refs
    DROP COLUMN IF EXISTS signature_metadata,
    DROP COLUMN IF EXISTS signed_by_user_id,
    DROP COLUMN IF EXISTS signature_summary,
    DROP COLUMN IF EXISTS verified;

ALTER TABLE repository_archives
    DROP CONSTRAINT IF EXISTS repository_archives_format_check,
    ADD CONSTRAINT repository_archives_format_check CHECK (format IN ('zip'));
