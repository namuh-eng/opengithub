CREATE TABLE IF NOT EXISTS milestone_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    milestone_id uuid REFERENCES milestones(id) ON DELETE SET NULL,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    event_type text NOT NULL,
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT milestone_events_type_not_blank CHECK (length(trim(event_type)) > 0)
);

CREATE INDEX IF NOT EXISTS milestone_events_milestone_created_idx
ON milestone_events (milestone_id, created_at DESC);

CREATE INDEX IF NOT EXISTS milestone_events_repository_created_idx
ON milestone_events (repository_id, created_at DESC);

CREATE TABLE IF NOT EXISTS milestone_item_order (
    milestone_id uuid NOT NULL REFERENCES milestones(id) ON DELETE CASCADE,
    issue_id uuid NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    position integer NOT NULL,
    updated_by_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (milestone_id, issue_id),
    CONSTRAINT milestone_item_order_position_nonnegative CHECK (position >= 0)
);

CREATE INDEX IF NOT EXISTS milestone_item_order_milestone_position_idx
ON milestone_item_order (milestone_id, position ASC);

CREATE INDEX IF NOT EXISTS milestones_repository_updated_idx
ON milestones (repository_id, updated_at DESC);

CREATE INDEX IF NOT EXISTS milestones_repository_due_idx
ON milestones (repository_id, due_on ASC NULLS LAST);

CREATE INDEX IF NOT EXISTS issues_repository_milestone_state_idx
ON issues (repository_id, milestone_id, state);
