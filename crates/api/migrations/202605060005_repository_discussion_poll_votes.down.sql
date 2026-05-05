DROP INDEX IF EXISTS discussion_poll_votes_poll_option_active_idx;
DROP INDEX IF EXISTS discussion_poll_votes_poll_user_active_idx;
DROP INDEX IF EXISTS discussion_poll_votes_active_option_user_idx;
DROP TABLE IF EXISTS discussion_poll_votes;

ALTER TABLE discussion_polls
    DROP COLUMN IF EXISTS allows_vote_changes;
