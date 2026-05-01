ALTER TABLE actions_workflows
ADD COLUMN pinned_order integer;

ALTER TABLE workflow_runs
ADD COLUMN display_title text,
ADD COLUMN pull_request_id uuid REFERENCES pull_requests(id) ON DELETE SET NULL,
ADD COLUMN commit_id uuid REFERENCES commits(id) ON DELETE SET NULL;

CREATE INDEX workflow_runs_repository_event_created_idx
ON workflow_runs (repository_id, event, created_at DESC);

CREATE INDEX workflow_runs_repository_branch_created_idx
ON workflow_runs (repository_id, head_branch, created_at DESC);

CREATE INDEX workflow_runs_actor_created_idx
ON workflow_runs (actor_user_id, created_at DESC);

CREATE TABLE actions_recent_views (
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    workflow_id uuid REFERENCES actions_workflows(id) ON DELETE SET NULL,
    filters jsonb NOT NULL DEFAULT '{}'::jsonb,
    viewed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (repository_id, user_id)
);

CREATE INDEX actions_recent_views_user_viewed_idx
ON actions_recent_views (user_id, viewed_at DESC);
