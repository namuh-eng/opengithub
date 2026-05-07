ALTER TABLE commits
    ADD COLUMN IF NOT EXISTS signature_fingerprint text,
    ADD COLUMN IF NOT EXISTS signature_summary text;

ALTER TABLE repository_git_refs
    ADD COLUMN IF NOT EXISTS signature_fingerprint text;

CREATE INDEX IF NOT EXISTS commits_signature_fingerprint_idx
ON commits (repository_id, signature_fingerprint)
WHERE signature_fingerprint IS NOT NULL;

CREATE INDEX IF NOT EXISTS repository_git_refs_signature_fingerprint_idx
ON repository_git_refs (repository_id, signature_fingerprint)
WHERE signature_fingerprint IS NOT NULL;
