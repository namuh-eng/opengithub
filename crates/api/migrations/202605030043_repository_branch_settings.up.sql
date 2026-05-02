ALTER TABLE repository_branch_protection_rules
    ADD COLUMN IF NOT EXISTS enforcement text NOT NULL DEFAULT 'active',
    ADD COLUMN IF NOT EXISTS requires_conversation_resolution boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS requires_signed_commits boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS requires_linear_history boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS requires_merge_queue boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS requires_deployments boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS required_deployment_environments text[] NOT NULL DEFAULT ARRAY[]::text[],
    ADD COLUMN IF NOT EXISTS locked boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS restricts_pushes boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS allows_force_pushes boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS allows_deletions boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS bypass_actors jsonb NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS description text,
    ADD CONSTRAINT repository_branch_protection_rules_enforcement_check
        CHECK (enforcement IN ('active', 'evaluate', 'disabled'));

CREATE UNIQUE INDEX IF NOT EXISTS repository_branch_protection_rules_repo_pattern_unique
ON repository_branch_protection_rules (repository_id, lower(pattern));

CREATE TABLE repository_rulesets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name text NOT NULL,
    target text NOT NULL DEFAULT 'branch',
    enforcement text NOT NULL DEFAULT 'active',
    patterns text[] NOT NULL DEFAULT ARRAY[]::text[],
    required_approving_review_count bigint NOT NULL DEFAULT 0,
    requires_up_to_date_branch boolean NOT NULL DEFAULT false,
    required_status_checks text[] NOT NULL DEFAULT ARRAY[]::text[],
    requires_conversation_resolution boolean NOT NULL DEFAULT false,
    requires_signed_commits boolean NOT NULL DEFAULT false,
    requires_linear_history boolean NOT NULL DEFAULT false,
    requires_merge_queue boolean NOT NULL DEFAULT false,
    requires_deployments boolean NOT NULL DEFAULT false,
    required_deployment_environments text[] NOT NULL DEFAULT ARRAY[]::text[],
    locked boolean NOT NULL DEFAULT false,
    restricts_pushes boolean NOT NULL DEFAULT false,
    allows_force_pushes boolean NOT NULL DEFAULT false,
    allows_deletions boolean NOT NULL DEFAULT false,
    bypass_actors jsonb NOT NULL DEFAULT '[]'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_rulesets_name_not_blank CHECK (length(trim(name)) > 0),
    CONSTRAINT repository_rulesets_target_check CHECK (target IN ('branch')),
    CONSTRAINT repository_rulesets_enforcement_check CHECK (enforcement IN ('active', 'evaluate', 'disabled')),
    CONSTRAINT repository_rulesets_reviews_nonnegative CHECK (required_approving_review_count >= 0),
    CONSTRAINT repository_rulesets_patterns_not_empty CHECK (array_length(patterns, 1) IS NOT NULL)
);

CREATE UNIQUE INDEX repository_rulesets_repo_name_unique
ON repository_rulesets (repository_id, lower(name));
CREATE INDEX repository_rulesets_repo_idx ON repository_rulesets (repository_id);

CREATE TRIGGER repository_rulesets_set_updated_at
BEFORE UPDATE ON repository_rulesets
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE repository_rule_evaluations (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    branch_protection_rule_id uuid REFERENCES repository_branch_protection_rules(id) ON DELETE SET NULL,
    ruleset_id uuid REFERENCES repository_rulesets(id) ON DELETE SET NULL,
    actor_user_id uuid REFERENCES users(id) ON DELETE SET NULL,
    ref_name text NOT NULL,
    operation text NOT NULL,
    outcome text NOT NULL,
    reasons text[] NOT NULL DEFAULT ARRAY[]::text[],
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_rule_evaluations_ref_not_blank CHECK (length(trim(ref_name)) > 0),
    CONSTRAINT repository_rule_evaluations_operation_check CHECK (operation IN ('push', 'merge')),
    CONSTRAINT repository_rule_evaluations_outcome_check CHECK (outcome IN ('passed', 'blocked', 'evaluated'))
);

CREATE INDEX repository_rule_evaluations_repo_created_idx
ON repository_rule_evaluations (repository_id, created_at DESC);
