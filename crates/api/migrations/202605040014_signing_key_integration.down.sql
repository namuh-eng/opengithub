DROP INDEX IF EXISTS repository_git_refs_signature_fingerprint_idx;
DROP INDEX IF EXISTS commits_signature_fingerprint_idx;

ALTER TABLE repository_git_refs
    DROP COLUMN IF EXISTS signature_fingerprint;

ALTER TABLE commits
    DROP COLUMN IF EXISTS signature_summary,
    DROP COLUMN IF EXISTS signature_fingerprint;
