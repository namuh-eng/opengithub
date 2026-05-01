CREATE TABLE repository_merge_settings (
    repository_id uuid PRIMARY KEY REFERENCES repositories(id) ON DELETE CASCADE,
    allow_squash boolean NOT NULL DEFAULT true,
    allow_merge_commit boolean NOT NULL DEFAULT true,
    allow_rebase boolean NOT NULL DEFAULT true,
    default_method text NOT NULL DEFAULT 'squash',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_merge_settings_default_method_check CHECK (
        default_method IN ('squash', 'merge_commit', 'rebase')
    ),
    CONSTRAINT repository_merge_settings_at_least_one_method CHECK (
        allow_squash OR allow_merge_commit OR allow_rebase
    )
);

CREATE TRIGGER repository_merge_settings_set_updated_at
BEFORE UPDATE ON repository_merge_settings
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE repository_branch_protection_rules (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    repository_id uuid NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    pattern text NOT NULL,
    required_approving_review_count bigint NOT NULL DEFAULT 0,
    requires_up_to_date_branch boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_branch_protection_rules_pattern_not_blank CHECK (length(trim(pattern)) > 0),
    CONSTRAINT repository_branch_protection_rules_reviews_nonnegative CHECK (
        required_approving_review_count >= 0
    )
);

CREATE INDEX repository_branch_protection_rules_repo_idx
ON repository_branch_protection_rules (repository_id);

CREATE TRIGGER repository_branch_protection_rules_set_updated_at
BEFORE UPDATE ON repository_branch_protection_rules
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TABLE repository_required_status_checks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_protection_rule_id uuid NOT NULL REFERENCES repository_branch_protection_rules(id) ON DELETE CASCADE,
    context text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT repository_required_status_checks_context_not_blank CHECK (length(trim(context)) > 0)
);

CREATE UNIQUE INDEX repository_required_status_checks_rule_context_unique
ON repository_required_status_checks (branch_protection_rule_id, lower(context));
