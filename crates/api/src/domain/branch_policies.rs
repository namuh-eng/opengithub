use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::collections::BTreeSet;
use uuid::Uuid;

use super::permissions::RepositoryRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchPolicyOperation {
    Merge,
    Push {
        force: bool,
        deletion: bool,
        creation: bool,
    },
}

impl BranchPolicyOperation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Merge => "merge",
            Self::Push { .. } => "push",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BranchPolicySummary {
    pub protected: bool,
    pub pattern: Option<String>,
    pub source_count: i64,
    pub active_rule_count: i64,
    pub active_ruleset_count: i64,
    pub evaluate_rule_count: i64,
    pub evaluate_ruleset_count: i64,
    pub required_approving_review_count: i64,
    pub requires_up_to_date_branch: bool,
    pub required_status_checks: Vec<String>,
    pub requires_conversation_resolution: bool,
    pub requires_signed_commits: bool,
    pub requires_linear_history: bool,
    pub requires_merge_queue: bool,
    pub requires_deployments: bool,
    pub required_deployment_environments: Vec<String>,
    pub locked: bool,
    pub restricts_pushes: bool,
    pub allows_force_pushes: bool,
    pub allows_deletions: bool,
    pub bypassed: bool,
    pub active_sources: Vec<BranchPolicySourceSummary>,
    pub evaluate_sources: Vec<BranchPolicySourceSummary>,
    pub blocking_reasons: Vec<String>,
}

impl BranchPolicySummary {
    pub fn unprotected() -> Self {
        Self {
            protected: false,
            pattern: None,
            source_count: 0,
            active_rule_count: 0,
            active_ruleset_count: 0,
            evaluate_rule_count: 0,
            evaluate_ruleset_count: 0,
            required_approving_review_count: 0,
            requires_up_to_date_branch: false,
            required_status_checks: Vec::new(),
            requires_conversation_resolution: false,
            requires_signed_commits: false,
            requires_linear_history: false,
            requires_merge_queue: false,
            requires_deployments: false,
            required_deployment_environments: Vec::new(),
            locked: false,
            restricts_pushes: false,
            allows_force_pushes: false,
            allows_deletions: false,
            bypassed: false,
            active_sources: Vec::new(),
            evaluate_sources: Vec::new(),
            blocking_reasons: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BranchPolicySourceSummary {
    pub id: Uuid,
    pub source_type: String,
    pub name: String,
    pub pattern: String,
    pub enforcement: String,
}

#[derive(Debug, Clone)]
struct BranchPolicySource {
    id: Uuid,
    source_type: BranchPolicySourceType,
    name: String,
    patterns: Vec<String>,
    enforcement: BranchPolicyEnforcement,
    requirements: BranchPolicyRequirements,
    bypass_actors: Vec<BypassActor>,
}

impl BranchPolicySource {
    fn summary_for(&self, branch: &str) -> BranchPolicySourceSummary {
        BranchPolicySourceSummary {
            id: self.id,
            source_type: self.source_type.as_str().to_owned(),
            name: self.name.clone(),
            pattern: self
                .patterns
                .iter()
                .find(|pattern| branch_pattern_matches(pattern, branch))
                .cloned()
                .unwrap_or_else(|| self.patterns.join(", ")),
            enforcement: self.enforcement.as_str().to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BranchPolicySourceType {
    Rule,
    Ruleset,
}

impl BranchPolicySourceType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Rule => "rule",
            Self::Ruleset => "ruleset",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BranchPolicyEnforcement {
    Active,
    Evaluate,
    Disabled,
}

impl BranchPolicyEnforcement {
    fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Evaluate => "evaluate",
            Self::Disabled => "disabled",
        }
    }
}

impl From<&str> for BranchPolicyEnforcement {
    fn from(value: &str) -> Self {
        match value {
            "evaluate" => Self::Evaluate,
            "disabled" => Self::Disabled,
            _ => Self::Active,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct BranchPolicyRequirements {
    required_approving_review_count: i64,
    requires_up_to_date_branch: bool,
    required_status_checks: Vec<String>,
    requires_conversation_resolution: bool,
    requires_signed_commits: bool,
    requires_linear_history: bool,
    requires_merge_queue: bool,
    requires_deployments: bool,
    required_deployment_environments: Vec<String>,
    locked: bool,
    restricts_pushes: bool,
    allows_force_pushes: bool,
    allows_deletions: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BypassActor {
    actor_type: String,
    actor_id: Uuid,
    label: String,
}

pub async fn evaluate_branch_policy(
    pool: &PgPool,
    repository_id: Uuid,
    branch: &str,
    actor_user_id: Option<Uuid>,
    operation: BranchPolicyOperation,
) -> Result<BranchPolicySummary, sqlx::Error> {
    let sources = matching_sources(pool, repository_id, branch).await?;
    let Some(actor_user_id) = actor_user_id else {
        let summary = aggregate_policy(branch, &sources, None, None, operation);
        maybe_record_evaluations(
            pool,
            repository_id,
            branch,
            None,
            operation,
            &sources,
            &summary,
        )
        .await?;
        return Ok(summary);
    };
    let role = repository_role_for_actor(pool, repository_id, actor_user_id).await?;
    let summary = aggregate_policy(branch, &sources, Some(actor_user_id), Some(role), operation);
    maybe_record_evaluations(
        pool,
        repository_id,
        branch,
        Some(actor_user_id),
        operation,
        &sources,
        &summary,
    )
    .await?;
    Ok(summary)
}

pub fn branch_pattern_matches(pattern: &str, branch: &str) -> bool {
    if pattern == branch {
        return true;
    }
    let mut remainder = branch;
    let mut first = true;
    for part in pattern.split('*') {
        if part.is_empty() {
            first = false;
            continue;
        }
        let Some(index) = remainder.find(part) else {
            return false;
        };
        if first && !pattern.starts_with('*') && index != 0 {
            return false;
        }
        remainder = &remainder[index + part.len()..];
        first = false;
    }
    pattern.ends_with('*') || remainder.is_empty()
}

async fn matching_sources(
    pool: &PgPool,
    repository_id: Uuid,
    branch: &str,
) -> Result<Vec<BranchPolicySource>, sqlx::Error> {
    let rule_rows = sqlx::query(
        r#"
        SELECT id, pattern, description, enforcement, required_approving_review_count,
               requires_up_to_date_branch, requires_conversation_resolution,
               requires_signed_commits, requires_linear_history, requires_merge_queue,
               requires_deployments, required_deployment_environments, locked, restricts_pushes,
               allows_force_pushes, allows_deletions, bypass_actors
        FROM repository_branch_protection_rules
        WHERE repository_id = $1
        ORDER BY lower(pattern), created_at
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    let mut sources = Vec::new();
    for row in rule_rows {
        let pattern: String = row.get("pattern");
        if !branch_pattern_matches(&pattern, branch) {
            continue;
        }
        let id: Uuid = row.get("id");
        let required_status_checks = sqlx::query_scalar::<_, String>(
            r#"
            SELECT context
            FROM repository_required_status_checks
            WHERE branch_protection_rule_id = $1
            ORDER BY lower(context)
            "#,
        )
        .bind(id)
        .fetch_all(pool)
        .await?;
        sources.push(BranchPolicySource {
            id,
            source_type: BranchPolicySourceType::Rule,
            name: row
                .get::<Option<String>, _>("description")
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| pattern.clone()),
            patterns: vec![pattern],
            enforcement: BranchPolicyEnforcement::from(
                row.get::<String, _>("enforcement").as_str(),
            ),
            requirements: requirements_from_row(&row, required_status_checks),
            bypass_actors: decode_bypass_actors(row.get("bypass_actors")),
        });
    }

    let ruleset_rows = sqlx::query(
        r#"
        SELECT id, name, enforcement, patterns, required_approving_review_count,
               requires_up_to_date_branch, required_status_checks,
               requires_conversation_resolution, requires_signed_commits, requires_linear_history,
               requires_merge_queue, requires_deployments, required_deployment_environments,
               locked, restricts_pushes, allows_force_pushes, allows_deletions, bypass_actors
        FROM repository_rulesets
        WHERE repository_id = $1
        ORDER BY lower(name), created_at
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    for row in ruleset_rows {
        let patterns: Vec<String> = row.get("patterns");
        if !patterns
            .iter()
            .any(|pattern| branch_pattern_matches(pattern, branch))
        {
            continue;
        }
        sources.push(BranchPolicySource {
            id: row.get("id"),
            source_type: BranchPolicySourceType::Ruleset,
            name: row.get("name"),
            patterns,
            enforcement: BranchPolicyEnforcement::from(
                row.get::<String, _>("enforcement").as_str(),
            ),
            requirements: requirements_from_row(&row, row.get("required_status_checks")),
            bypass_actors: decode_bypass_actors(row.get("bypass_actors")),
        });
    }
    Ok(sources)
}

fn requirements_from_row(
    row: &sqlx::postgres::PgRow,
    required_status_checks: Vec<String>,
) -> BranchPolicyRequirements {
    BranchPolicyRequirements {
        required_approving_review_count: row.get("required_approving_review_count"),
        requires_up_to_date_branch: row.get("requires_up_to_date_branch"),
        required_status_checks,
        requires_conversation_resolution: row.get("requires_conversation_resolution"),
        requires_signed_commits: row.get("requires_signed_commits"),
        requires_linear_history: row.get("requires_linear_history"),
        requires_merge_queue: row.get("requires_merge_queue"),
        requires_deployments: row.get("requires_deployments"),
        required_deployment_environments: row.get("required_deployment_environments"),
        locked: row.get("locked"),
        restricts_pushes: row.get("restricts_pushes"),
        allows_force_pushes: row.get("allows_force_pushes"),
        allows_deletions: row.get("allows_deletions"),
    }
}

fn decode_bypass_actors(value: Value) -> Vec<BypassActor> {
    serde_json::from_value(value).unwrap_or_default()
}

fn aggregate_policy(
    branch: &str,
    sources: &[BranchPolicySource],
    actor_user_id: Option<Uuid>,
    actor_role: Option<RepositoryRole>,
    operation: BranchPolicyOperation,
) -> BranchPolicySummary {
    let mut summary = BranchPolicySummary::unprotected();
    let mut blocking_summary = BranchPolicySummary::unprotected();
    let mut checks = BTreeSet::new();
    let mut blocking_checks = BTreeSet::new();
    let mut deployment_environments = BTreeSet::new();
    let mut blocking_deployment_environments = BTreeSet::new();
    let mut patterns = Vec::new();

    for source in sources {
        match source.enforcement {
            BranchPolicyEnforcement::Disabled => continue,
            BranchPolicyEnforcement::Evaluate => {
                if source.source_type == BranchPolicySourceType::Rule {
                    summary.evaluate_rule_count += 1;
                } else {
                    summary.evaluate_ruleset_count += 1;
                }
                summary.evaluate_sources.push(source.summary_for(branch));
                continue;
            }
            BranchPolicyEnforcement::Active => {}
        }
        if source.source_type == BranchPolicySourceType::Rule {
            summary.active_rule_count += 1;
        } else {
            summary.active_ruleset_count += 1;
        }
        summary.active_sources.push(source.summary_for(branch));
        patterns.extend(source.patterns.iter().cloned());
        aggregate_requirements(
            &mut summary,
            source,
            &mut checks,
            &mut deployment_environments,
        );
        let source_is_bypassed = actor_user_id
            .zip(actor_role)
            .is_some_and(|(user_id, role)| source_bypassed(source, user_id, role));
        if !source_is_bypassed {
            blocking_summary.source_count += 1;
            aggregate_requirements(
                &mut blocking_summary,
                source,
                &mut blocking_checks,
                &mut blocking_deployment_environments,
            );
        }
    }

    summary.required_status_checks = checks.into_iter().collect();
    summary.required_deployment_environments = deployment_environments.into_iter().collect();
    summary.source_count = summary.active_rule_count + summary.active_ruleset_count;
    summary.protected = summary.source_count > 0;
    summary.pattern = patterns.into_iter().next();
    summary.bypassed = summary.protected && blocking_summary.source_count < summary.source_count;

    blocking_summary.required_status_checks = blocking_checks.into_iter().collect();
    blocking_summary.required_deployment_environments =
        blocking_deployment_environments.into_iter().collect();
    blocking_summary.protected = blocking_summary.source_count > 0;

    if summary.protected {
        summary.blocking_reasons = blocking_reasons(&blocking_summary, operation);
    }
    summary
}

fn aggregate_requirements(
    summary: &mut BranchPolicySummary,
    source: &BranchPolicySource,
    checks: &mut BTreeSet<String>,
    deployment_environments: &mut BTreeSet<String>,
) {
    summary.required_approving_review_count = summary
        .required_approving_review_count
        .max(source.requirements.required_approving_review_count);
    summary.requires_up_to_date_branch |= source.requirements.requires_up_to_date_branch;
    summary.requires_conversation_resolution |=
        source.requirements.requires_conversation_resolution;
    summary.requires_signed_commits |= source.requirements.requires_signed_commits;
    summary.requires_linear_history |= source.requirements.requires_linear_history;
    summary.requires_merge_queue |= source.requirements.requires_merge_queue;
    summary.requires_deployments |= source.requirements.requires_deployments;
    summary.locked |= source.requirements.locked;
    summary.restricts_pushes |= source.requirements.restricts_pushes;
    summary.allows_force_pushes |= source.requirements.allows_force_pushes;
    summary.allows_deletions |= source.requirements.allows_deletions;
    checks.extend(source.requirements.required_status_checks.iter().cloned());
    deployment_environments.extend(
        source
            .requirements
            .required_deployment_environments
            .iter()
            .cloned(),
    );
}

fn blocking_reasons(
    summary: &BranchPolicySummary,
    operation: BranchPolicyOperation,
) -> Vec<String> {
    let mut reasons = Vec::new();
    match operation {
        BranchPolicyOperation::Merge => {
            if summary.requires_conversation_resolution {
                reasons.push("conversation resolution is required".to_owned());
            }
            if summary.requires_signed_commits {
                reasons.push("signed commits are required".to_owned());
            }
            if summary.requires_merge_queue {
                reasons.push("merge queue is required".to_owned());
            }
            if summary.requires_deployments {
                reasons.push("required deployments must pass".to_owned());
            }
            if summary.locked {
                reasons.push("branch is locked".to_owned());
            }
        }
        BranchPolicyOperation::Push {
            force,
            deletion,
            creation,
        } => {
            if summary.locked {
                reasons.push("branch is locked".to_owned());
            }
            if summary.restricts_pushes {
                reasons.push("pushes to this branch are restricted".to_owned());
            }
            if deletion && !summary.allows_deletions {
                reasons.push("branch deletion is blocked by branch protection".to_owned());
            }
            if force && !summary.allows_force_pushes {
                reasons.push("force pushes are blocked by branch protection".to_owned());
            }
            if creation && summary.restricts_pushes {
                reasons.push("new matching branches require a bypass actor".to_owned());
            }
        }
    }
    reasons
}

fn source_bypassed(source: &BranchPolicySource, actor_user_id: Uuid, role: RepositoryRole) -> bool {
    source.bypass_actors.iter().any(|actor| {
        let actor_type = actor.actor_type.as_str();
        if actor_type == "user" && actor.actor_id == actor_user_id {
            return true;
        }
        if actor_type == "role" {
            return RepositoryRole::try_from(actor.label.to_lowercase().as_str())
                .map(|bypass_role| role >= bypass_role)
                .unwrap_or(false);
        }
        false
    })
}

async fn repository_role_for_actor(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
) -> Result<RepositoryRole, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT owner_user_id,
               COALESCE(repository_permissions.role, 'read') AS role
        FROM repositories
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = repositories.id
         AND repository_permissions.user_id = $2
        WHERE repositories.id = $1
        "#,
    )
    .bind(repository_id)
    .bind(actor_user_id)
    .fetch_one(pool)
    .await?;
    let owner_user_id: Option<Uuid> = row.get("owner_user_id");
    if owner_user_id == Some(actor_user_id) {
        return Ok(RepositoryRole::Owner);
    }
    Ok(
        RepositoryRole::try_from(row.get::<String, _>("role").as_str())
            .unwrap_or(RepositoryRole::Read),
    )
}

async fn maybe_record_evaluations(
    pool: &PgPool,
    repository_id: Uuid,
    branch: &str,
    actor_user_id: Option<Uuid>,
    operation: BranchPolicyOperation,
    sources: &[BranchPolicySource],
    summary: &BranchPolicySummary,
) -> Result<(), sqlx::Error> {
    let evaluate_sources = sources
        .iter()
        .filter(|source| source.enforcement == BranchPolicyEnforcement::Evaluate)
        .collect::<Vec<_>>();
    if evaluate_sources.is_empty() {
        return Ok(());
    }
    let outcome = if summary.blocking_reasons.is_empty() {
        "evaluated"
    } else {
        "blocked"
    };
    for source in evaluate_sources {
        let (rule_id, ruleset_id) = match source.source_type {
            BranchPolicySourceType::Rule => (Some(source.id), None),
            BranchPolicySourceType::Ruleset => (None, Some(source.id)),
        };
        sqlx::query(
            r#"
            INSERT INTO repository_rule_evaluations (
                repository_id, branch_protection_rule_id, ruleset_id, actor_user_id,
                ref_name, operation, outcome, reasons
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(repository_id)
        .bind(rule_id)
        .bind(ruleset_id)
        .bind(actor_user_id)
        .bind(branch)
        .bind(operation.as_str())
        .bind(outcome)
        .bind(&summary.blocking_reasons)
        .execute(pool)
        .await?;
    }
    Ok(())
}
