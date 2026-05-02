DROP TABLE IF EXISTS repository_rule_evaluations;
DROP TABLE IF EXISTS repository_rulesets;
DROP INDEX IF EXISTS repository_branch_protection_rules_repo_pattern_unique;

ALTER TABLE repository_branch_protection_rules
    DROP CONSTRAINT IF EXISTS repository_branch_protection_rules_enforcement_check,
    DROP COLUMN IF EXISTS description,
    DROP COLUMN IF EXISTS bypass_actors,
    DROP COLUMN IF EXISTS allows_deletions,
    DROP COLUMN IF EXISTS allows_force_pushes,
    DROP COLUMN IF EXISTS restricts_pushes,
    DROP COLUMN IF EXISTS locked,
    DROP COLUMN IF EXISTS required_deployment_environments,
    DROP COLUMN IF EXISTS requires_deployments,
    DROP COLUMN IF EXISTS requires_merge_queue,
    DROP COLUMN IF EXISTS requires_linear_history,
    DROP COLUMN IF EXISTS requires_signed_commits,
    DROP COLUMN IF EXISTS requires_conversation_resolution,
    DROP COLUMN IF EXISTS enforcement;
