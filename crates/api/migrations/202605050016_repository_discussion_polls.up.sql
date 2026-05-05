CREATE TABLE IF NOT EXISTS discussion_polls (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    discussion_id uuid NOT NULL UNIQUE REFERENCES discussions(id) ON DELETE CASCADE,
    question text NOT NULL,
    allows_multiple boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT discussion_polls_question_not_blank CHECK (length(trim(question)) > 0)
);

CREATE TABLE IF NOT EXISTS discussion_poll_options (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    poll_id uuid NOT NULL REFERENCES discussion_polls(id) ON DELETE CASCADE,
    position integer NOT NULL,
    label text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (poll_id, position),
    CONSTRAINT discussion_poll_options_label_not_blank CHECK (length(trim(label)) > 0),
    CONSTRAINT discussion_poll_options_position_nonnegative CHECK (position >= 0)
);

CREATE INDEX IF NOT EXISTS discussion_poll_options_poll_idx
    ON discussion_poll_options(poll_id, position);
