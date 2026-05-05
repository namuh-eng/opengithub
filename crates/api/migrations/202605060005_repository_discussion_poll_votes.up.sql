ALTER TABLE discussion_polls
    ADD COLUMN IF NOT EXISTS allows_vote_changes boolean NOT NULL DEFAULT true;

CREATE TABLE IF NOT EXISTS discussion_poll_votes (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    poll_id uuid NOT NULL REFERENCES discussion_polls(id) ON DELETE CASCADE,
    option_id uuid NOT NULL REFERENCES discussion_poll_options(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    replaced_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS discussion_poll_votes_active_option_user_idx
    ON discussion_poll_votes(option_id, user_id)
    WHERE replaced_at IS NULL;

CREATE INDEX IF NOT EXISTS discussion_poll_votes_poll_user_active_idx
    ON discussion_poll_votes(poll_id, user_id, created_at)
    WHERE replaced_at IS NULL;

CREATE INDEX IF NOT EXISTS discussion_poll_votes_poll_option_active_idx
    ON discussion_poll_votes(poll_id, option_id)
    WHERE replaced_at IS NULL;
