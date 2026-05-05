ALTER TABLE discussions
    ADD COLUMN IF NOT EXISTS deleted_at timestamptz,
    ADD COLUMN IF NOT EXISTS deleted_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS deleted_reason text,
    ADD COLUMN IF NOT EXISTS transferred_from_repository_id uuid REFERENCES repositories(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS transferred_from_number bigint;

CREATE TABLE IF NOT EXISTS discussion_deletion_tombstones (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    discussion_id uuid NOT NULL REFERENCES discussions(id) ON DELETE CASCADE,
    discussion_number bigint NOT NULL,
    deleted_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    reason text,
    title_sha256 text NOT NULL,
    body_sha256 text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (discussion_id)
);

CREATE INDEX IF NOT EXISTS discussions_repository_deleted_idx
    ON discussions(repository_id, deleted_at, number);
CREATE INDEX IF NOT EXISTS discussion_deletion_tombstones_repository_created_idx
    ON discussion_deletion_tombstones(repository_id, created_at DESC);
