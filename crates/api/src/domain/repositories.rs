use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{types::Json, PgPool, Row};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use crate::api_types::ListEnvelope;

use super::{branch_policies::branch_pattern_matches, permissions::RepositoryRole};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryVisibility {
    #[default]
    Public,
    Private,
    Internal,
}

impl RepositoryVisibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::Internal => "internal",
        }
    }
}

impl TryFrom<&str> for RepositoryVisibility {
    type Error = RepositoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            "internal" => Ok(Self::Internal),
            other => Err(RepositoryError::InvalidVisibility(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryOwner {
    User { id: Uuid },
    Organization { id: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Organization {
    pub id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub owner_user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Repository {
    pub id: Uuid,
    pub owner_user_id: Option<Uuid>,
    pub owner_organization_id: Option<Uuid>,
    pub owner_login: String,
    pub name: String,
    pub description: Option<String>,
    pub visibility: RepositoryVisibility,
    pub default_branch: String,
    pub is_archived: bool,
    pub created_by_user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryFeatureSettings {
    pub issues_enabled: bool,
    pub projects_enabled: bool,
    pub wiki_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryMergeMethod {
    Squash,
    MergeCommit,
    Rebase,
}

impl RepositoryMergeMethod {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Squash => "squash",
            Self::MergeCommit => "merge_commit",
            Self::Rebase => "rebase",
        }
    }
}

impl TryFrom<&str> for RepositoryMergeMethod {
    type Error = RepositoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "squash" => Ok(Self::Squash),
            "merge_commit" => Ok(Self::MergeCommit),
            "rebase" => Ok(Self::Rebase),
            other => Err(RepositoryError::InvalidMergeMethod(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryMergeSettings {
    pub allow_squash: bool,
    pub allow_merge_commit: bool,
    pub allow_rebase: bool,
    pub default_method: RepositoryMergeMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDangerState {
    pub is_archived: bool,
    pub can_archive: bool,
    pub can_unarchive: bool,
    pub delete_supported: bool,
    pub transfer_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySettingsAuditEvent {
    pub id: Uuid,
    pub event_type: String,
    pub changed_fields: Vec<String>,
    pub actor_user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySettings {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub description: Option<String>,
    pub visibility: RepositoryVisibility,
    pub default_branch: String,
    pub is_template: bool,
    pub allow_forking: bool,
    pub web_commit_signoff_required: bool,
    pub features: RepositoryFeatureSettings,
    pub merge: RepositoryMergeSettings,
    pub danger: RepositoryDangerState,
    pub branches: Vec<String>,
    pub viewer_permission: String,
    pub updated_at: DateTime<Utc>,
    pub audit_events: Vec<RepositorySettingsAuditEvent>,
    pub policy_locks: Vec<RepositoryPolicyLock>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPolicyLock {
    pub field: String,
    pub reason: String,
    pub settings_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryAccessSettings {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub visibility: RepositoryVisibility,
    pub viewer_permission: String,
    pub roles: Vec<RepositoryAccessRoleDefinition>,
    pub people: Vec<RepositoryAccessPerson>,
    pub teams: Vec<RepositoryAccessTeam>,
    pub invitations: Vec<RepositoryInvitation>,
    pub invite_targets: RepositoryInviteTargets,
    pub audit_events: Vec<RepositorySettingsAuditEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryAccessRoleDefinition {
    pub role: RepositoryRole,
    pub label: String,
    pub description: String,
    pub rank: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryAccessPerson {
    pub user_id: Uuid,
    pub login: String,
    pub display_name: Option<String>,
    pub email: String,
    pub avatar_url: Option<String>,
    pub role: RepositoryRole,
    pub source: String,
    pub source_text: String,
    pub team_slug: Option<String>,
    pub team_name: Option<String>,
    pub can_edit: bool,
    pub can_remove: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryAccessTeam {
    pub team_id: Uuid,
    pub slug: String,
    pub name: String,
    pub role: RepositoryRole,
    pub source: String,
    pub source_text: String,
    pub member_count: i64,
    pub href: String,
    pub can_edit: bool,
    pub can_remove: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryInvitation {
    pub id: Uuid,
    pub invited_user_id: Option<Uuid>,
    pub invited_email: String,
    pub invited_login: Option<String>,
    pub role: RepositoryRole,
    pub status: String,
    pub email_delivery_status: String,
    pub invited_by_user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub can_cancel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryInviteTargets {
    pub users: Vec<RepositoryInviteUserTarget>,
    pub teams: Vec<RepositoryInviteTeamTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryInviteUserTarget {
    pub user_id: Uuid,
    pub login: String,
    pub display_name: Option<String>,
    pub email: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryInviteTeamTarget {
    pub team_id: Uuid,
    pub slug: String,
    pub name: String,
    pub member_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryAccessInviteRequest {
    pub email_or_login: String,
    pub role: RepositoryRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryAccessTeamGrantRequest {
    pub team_slug: String,
    pub role: RepositoryRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryAccessRolePatch {
    pub role: RepositoryRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchSettings {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub visibility: RepositoryVisibility,
    pub default_branch: String,
    pub default_branch_summary: RepositoryDefaultBranchSummary,
    pub viewer_permission: String,
    pub can_edit: bool,
    pub refs: Vec<RepositoryBranchRefSummary>,
    pub rules: Vec<RepositoryBranchRule>,
    pub rulesets: Vec<RepositoryRuleset>,
    pub status_check_suggestions: Vec<String>,
    pub audit_events: Vec<RepositorySettingsAuditEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDefaultBranchSummary {
    pub name: String,
    pub protected: bool,
    pub matching_rule_count: i64,
    pub matching_ruleset_count: i64,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchRefSummary {
    pub name: String,
    pub protected: bool,
    pub matching_rule_count: i64,
    pub matching_ruleset_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchRule {
    pub id: Uuid,
    pub pattern: String,
    pub description: Option<String>,
    pub enforcement: BranchPolicyEnforcement,
    pub matching_branches: Vec<String>,
    pub matching_branch_count: i64,
    pub requirements: BranchPolicyRequirements,
    pub bypass_actors: Vec<BypassActor>,
    pub can_edit: bool,
    pub can_delete: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryRuleset {
    pub id: Uuid,
    pub name: String,
    pub target: String,
    pub enforcement: BranchPolicyEnforcement,
    pub patterns: Vec<String>,
    pub matching_branches: Vec<String>,
    pub matching_branch_count: i64,
    pub requirements: BranchPolicyRequirements,
    pub bypass_actors: Vec<BypassActor>,
    pub can_edit: bool,
    pub can_delete: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BranchPolicyEnforcement {
    Active,
    Evaluate,
    Disabled,
}

impl BranchPolicyEnforcement {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Evaluate => "evaluate",
            Self::Disabled => "disabled",
        }
    }
}

impl TryFrom<&str> for BranchPolicyEnforcement {
    type Error = RepositoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(Self::Active),
            "evaluate" => Ok(Self::Evaluate),
            "disabled" => Ok(Self::Disabled),
            other => Err(RepositoryError::InvalidBranchPolicy(format!(
                "Unsupported enforcement state `{other}`."
            ))),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BranchPolicyRequirements {
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BypassActor {
    pub actor_type: String,
    pub actor_id: Uuid,
    pub label: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchRuleMutation {
    pub pattern: String,
    pub description: Option<String>,
    pub enforcement: Option<BranchPolicyEnforcement>,
    #[serde(flatten)]
    pub requirements: BranchPolicyRequirementsPatch,
    pub bypass_actors: Option<Vec<BypassActor>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryRulesetMutation {
    pub name: String,
    pub enforcement: Option<BranchPolicyEnforcement>,
    pub patterns: Vec<String>,
    #[serde(flatten)]
    pub requirements: BranchPolicyRequirementsPatch,
    pub bypass_actors: Option<Vec<BypassActor>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BranchPolicyRequirementsPatch {
    pub required_approving_review_count: Option<i64>,
    pub requires_up_to_date_branch: Option<bool>,
    pub required_status_checks: Option<Vec<String>>,
    pub requires_conversation_resolution: Option<bool>,
    pub requires_signed_commits: Option<bool>,
    pub requires_linear_history: Option<bool>,
    pub requires_merge_queue: Option<bool>,
    pub requires_deployments: Option<bool>,
    pub required_deployment_environments: Option<Vec<String>>,
    pub locked: Option<bool>,
    pub restricts_pushes: Option<bool>,
    pub allows_force_pushes: Option<bool>,
    pub allows_deletions: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryFeatureSettingsPatch {
    pub issues_enabled: Option<bool>,
    pub projects_enabled: Option<bool>,
    pub wiki_enabled: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryMergeSettingsPatch {
    pub allow_squash: Option<bool>,
    pub allow_merge_commit: Option<bool>,
    pub allow_rebase: Option<bool>,
    pub default_method: Option<RepositoryMergeMethod>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySettingsPatch {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub visibility: Option<RepositoryVisibility>,
    pub default_branch: Option<String>,
    pub is_template: Option<bool>,
    pub allow_forking: Option<bool>,
    pub web_commit_signoff_required: Option<bool>,
    pub is_archived: Option<bool>,
    pub features: Option<RepositoryFeatureSettingsPatch>,
    pub merge: Option<RepositoryMergeSettingsPatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryFile {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub commit_id: Uuid,
    pub path: String,
    pub content: String,
    pub oid: String,
    pub byte_size: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryTreeEntry {
    pub kind: String,
    pub name: String,
    pub path: String,
    pub href: String,
    pub byte_size: Option<i64>,
    pub latest_commit_message: Option<String>,
    pub latest_commit_href: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPathBreadcrumb {
    pub name: String,
    pub path: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPathOverview {
    #[serde(flatten)]
    pub repository: Repository,
    pub viewer_permission: Option<String>,
    pub ref_name: String,
    pub resolved_ref: RepositoryResolvedRef,
    pub default_branch_href: String,
    pub recovery_href: String,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub has_more: bool,
    pub path: String,
    pub path_name: String,
    pub breadcrumbs: Vec<RepositoryPathBreadcrumb>,
    pub parent_href: Option<String>,
    pub entries: Vec<RepositoryTreeEntry>,
    pub readme: Option<RepositoryFile>,
    pub latest_commit: Option<RepositoryLatestCommit>,
    pub history_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBlobView {
    #[serde(flatten)]
    pub repository: Repository,
    pub viewer_permission: Option<String>,
    pub ref_name: String,
    pub resolved_ref: RepositoryResolvedRef,
    pub default_branch_href: String,
    pub recovery_href: String,
    pub path: String,
    pub path_name: String,
    pub breadcrumbs: Vec<RepositoryPathBreadcrumb>,
    pub parent_href: Option<String>,
    pub file: RepositoryFile,
    pub language: Option<String>,
    pub is_binary: bool,
    pub is_large: bool,
    pub line_count: i64,
    pub loc_count: i64,
    pub size_label: String,
    pub mime_type: String,
    pub render_mode: String,
    pub display_content: Option<String>,
    pub latest_commit: Option<RepositoryLatestCommit>,
    pub latest_path_commit: Option<RepositoryLatestCommit>,
    pub history_href: String,
    pub raw_href: String,
    pub download_href: String,
    pub raw_api_href: String,
    pub download_api_href: String,
    pub permalink_href: String,
    pub symbols: Vec<RepositoryCodeSymbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCodeSymbol {
    pub kind: String,
    pub name: String,
    pub line_number: i64,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBlameView {
    #[serde(flatten)]
    pub blob: RepositoryBlobView,
    pub lines: Vec<RepositoryBlameLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBlameLine {
    pub line_number: i64,
    pub content: String,
    pub commit: RepositoryBlameCommit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBlameCommit {
    pub oid: String,
    pub short_oid: String,
    pub message: String,
    pub href: String,
    pub committed_at: DateTime<Utc>,
    pub author_login: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitHistoryView {
    pub repository: RepositoryCommitHistoryRepository,
    pub resolved_ref: RepositoryCommitResolvedRef,
    pub filters: RepositoryCommitHistoryFilters,
    pub groups: Vec<RepositoryCommitGroup>,
    pub author_options: Vec<RepositoryCommitAuthorOption>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitHistoryRepository {
    pub owner_login: String,
    pub name: String,
    pub default_branch: String,
    pub visibility: RepositoryVisibility,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitResolvedRef {
    pub short_name: String,
    pub qualified_name: String,
    pub kind: String,
    pub target_oid: Option<String>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitHistoryFilters {
    pub path: Option<String>,
    pub author: Option<String>,
    pub until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitGroup {
    pub date: String,
    pub commits: Vec<RepositoryCommitListItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitListItem {
    pub oid: String,
    pub short_oid: String,
    pub message: String,
    pub subject: String,
    pub body: Option<String>,
    pub href: String,
    pub browse_href: String,
    pub committed_at: DateTime<Utc>,
    pub author_login: Option<String>,
    pub author_avatar_url: Option<String>,
    pub pull_requests: Vec<RepositoryCommitPullRequestLink>,
    pub status: RepositoryCommitStatusSummary,
    pub verification: RepositoryCommitVerificationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitPullRequestLink {
    pub number: i64,
    pub title: String,
    pub href: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitStatusSummary {
    pub status: String,
    pub conclusion: Option<String>,
    pub total_count: i64,
    pub completed_count: i64,
    pub failed_count: i64,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitVerificationSummary {
    pub verified: bool,
    pub signature_state: super::signing_keys::SignatureVerificationState,
    pub signature_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitAuthorOption {
    pub login: String,
    pub avatar_url: Option<String>,
    pub count: i64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchesView {
    pub repository: RepositoryBranchesRepository,
    pub tabs: RepositoryBranchClassificationCounts,
    pub filters: RepositoryBranchesFilters,
    pub default_branch: Option<RepositoryBranchDirectoryRow>,
    pub branches: Vec<RepositoryBranchDirectoryRow>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub empty_state: RepositoryBranchesEmptyState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchesRepository {
    pub owner_login: String,
    pub name: String,
    pub default_branch: String,
    pub visibility: RepositoryVisibility,
    pub viewer_permission: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchesFilters {
    pub tab: String,
    pub query: Option<String>,
    pub stale_cutoff_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchClassificationCounts {
    pub overview: i64,
    pub active: i64,
    pub stale: i64,
    pub all: i64,
    pub default: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchDirectoryRow {
    pub name: String,
    pub qualified_name: String,
    pub classification: String,
    pub is_default: bool,
    pub href: String,
    pub commits_href: String,
    pub activity_href: String,
    pub latest_commit: Option<RepositoryBranchLatestCommitSummary>,
    pub checks: RepositoryBranchCheckSummary,
    pub protection: RepositoryBranchProtectionSummary,
    pub ahead: i64,
    pub behind: i64,
    pub pull_request: Option<RepositoryBranchPullRequestSummary>,
    pub capabilities: RepositoryBranchCapabilities,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchLatestCommitSummary {
    pub oid: String,
    pub short_oid: String,
    pub subject: String,
    pub href: String,
    pub committed_at: DateTime<Utc>,
    pub author_login: Option<String>,
    pub author_avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchCheckSummary {
    pub status: String,
    pub conclusion: Option<String>,
    pub total_count: i64,
    pub completed_count: i64,
    pub failed_count: i64,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchProtectionSummary {
    pub protected: bool,
    pub matching_rule_count: i64,
    pub matching_ruleset_count: i64,
    pub required_status_checks: Vec<String>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchPullRequestSummary {
    pub number: i64,
    pub title: String,
    pub state: String,
    pub draft: bool,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchCapabilities {
    pub can_copy: bool,
    pub can_view_activity: bool,
    pub can_view_rules: bool,
    pub can_delete: bool,
    pub delete_disabled_reason: Option<String>,
    pub can_restore: bool,
    pub restore_disabled_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchesEmptyState {
    pub title: String,
    pub message: String,
    pub reset_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchActivityView {
    pub repository: RepositoryBranchesRepository,
    pub branch: RepositoryBranchDirectoryRow,
    pub recent_commits: Vec<RepositoryCommitListItem>,
    pub recent_pull_requests: Vec<RepositoryBranchPullRequestSummary>,
    pub protection_events: Vec<RepositoryBranchProtectionEvent>,
    pub links: RepositoryBranchActivityLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchProtectionEvent {
    pub source_type: String,
    pub name: String,
    pub enforcement: BranchPolicyEnforcement,
    pub href: String,
    pub required_status_checks: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBranchActivityLinks {
    pub branches_href: String,
    pub tree_href: String,
    pub commits_href: String,
    pub compare_href: String,
    pub rules_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPulseView {
    pub repository: RepositoryPulseRepository,
    pub period: RepositoryPulsePeriod,
    pub metrics: Vec<RepositoryPulseMetric>,
    pub summary: RepositoryPulseSummary,
    pub top_committers: Vec<RepositoryPulseCommitter>,
    pub releases: Vec<RepositoryPulseActivityItem>,
    pub merged_pull_requests: Vec<RepositoryPulseActivityItem>,
    pub issue_activity: Vec<RepositoryPulseActivityItem>,
    pub snapshot: RepositoryPulseSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPulseRepository {
    pub owner_login: String,
    pub name: String,
    pub default_branch: String,
    pub visibility: RepositoryVisibility,
    pub viewer_permission: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPulsePeriod {
    pub key: String,
    pub label: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPulseMetric {
    pub key: String,
    pub label: String,
    pub count: i64,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPulseSummary {
    pub sentence: String,
    pub commits: i64,
    pub files_changed: i64,
    pub additions: i64,
    pub deletions: i64,
    pub authors: i64,
    pub merged_pull_requests: i64,
    pub open_pull_requests: i64,
    pub closed_issues: i64,
    pub new_issues: i64,
    pub open_issues: i64,
    pub releases: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPulseCommitter {
    pub user_id: Option<Uuid>,
    pub login: String,
    pub author_status: String,
    pub is_bot: bool,
    pub avatar_url: Option<String>,
    pub commits: i64,
    pub files_changed: i64,
    pub additions: i64,
    pub deletions: i64,
    pub profile_href: String,
    pub commits_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPulseActivityItem {
    pub kind: String,
    pub number: Option<i64>,
    pub title: String,
    pub state: String,
    pub author_login: Option<String>,
    pub author_profile_href: Option<String>,
    pub author_status: String,
    pub author_avatar_url: Option<String>,
    pub href: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPulseSnapshot {
    pub cache_key: String,
    pub computed_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitDetailView {
    pub repository: RepositoryCommitDetailRepository,
    pub commit: RepositoryCommitDetailCommit,
    pub parents: Vec<RepositoryCommitDetailParent>,
    pub branches: Vec<RepositoryCommitDetailBranchLink>,
    pub pull_requests: Vec<RepositoryCommitPullRequestLink>,
    pub status: RepositoryCommitStatusSummary,
    pub verification: RepositoryCommitVerificationSummary,
    pub diff_placeholder: RepositoryCommitDetailDiffPlaceholder,
    pub diff_summary: RepositoryCommitDetailDiffSummary,
    pub file_tree: Vec<RepositoryCommitDetailFileTreeNode>,
    pub files: Vec<RepositoryCommitDetailFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitDetailRepository {
    pub owner_login: String,
    pub name: String,
    pub default_branch: String,
    pub visibility: RepositoryVisibility,
    pub href: String,
    pub commit_history_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitDetailCommit {
    pub oid: String,
    pub short_oid: String,
    pub message: String,
    pub subject: String,
    pub body: Option<String>,
    pub href: String,
    pub browse_href: String,
    pub committed_at: DateTime<Utc>,
    pub author_login: Option<String>,
    pub author_avatar_url: Option<String>,
    pub committer_login: Option<String>,
    pub committer_avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitDetailParent {
    pub oid: String,
    pub short_oid: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitDetailBranchLink {
    pub name: String,
    pub qualified_name: String,
    pub kind: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitDetailDiffPlaceholder {
    pub state: String,
    pub message: String,
    pub next_phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitDetailDiffSummary {
    pub total_files: i64,
    pub additions: i64,
    pub deletions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitDetailFileTreeNode {
    pub path: String,
    pub name: String,
    pub depth: i64,
    pub status: String,
    pub additions: i64,
    pub deletions: i64,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitDetailFile {
    pub path: String,
    pub previous_path: Option<String>,
    pub status: String,
    pub additions: i64,
    pub deletions: i64,
    pub byte_size: i64,
    pub blob_oid: Option<String>,
    pub language: Option<String>,
    pub anchor: String,
    pub href: String,
    pub raw_href: String,
    pub view_href: String,
    pub is_binary: bool,
    pub is_large: bool,
    pub hunks: Vec<RepositoryCommitDetailHunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitDetailHunk {
    pub id: String,
    pub header: String,
    pub old_start: i64,
    pub old_lines: i64,
    pub new_start: i64,
    pub new_lines: i64,
    pub lines: Vec<RepositoryCommitDetailLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitDetailLine {
    pub kind: String,
    pub old_line: Option<i64>,
    pub new_line: Option<i64>,
    pub content: String,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitDetailContext {
    pub path: String,
    pub hunk_id: String,
    pub lines: Vec<RepositoryCommitDetailLine>,
    pub expanded: bool,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub struct RepositoryCommitDetailContextQuery<'a> {
    pub path: &'a str,
    pub hunk_id: &'a str,
    pub context_lines: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct RepositoryCommitHistoryQuery<'a> {
    pub ref_name: Option<&'a str>,
    pub path: Option<&'a str>,
    pub author: Option<&'a str>,
    pub until: Option<DateTime<Utc>>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct RepositoryBranchesQuery<'a> {
    pub tab: Option<&'a str>,
    pub query: Option<&'a str>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct RepositoryPulseQuery<'a> {
    pub period: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
pub struct RepositoryRefsQuery<'a> {
    pub query: Option<&'a str>,
    pub current_path: Option<&'a str>,
    pub active_ref: Option<&'a str>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct RepositoryPathQuery<'a> {
    pub ref_name: Option<&'a str>,
    pub path: &'a str,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct RepositoryFileFinderQuery<'a> {
    pub ref_name: Option<&'a str>,
    pub query: Option<&'a str>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryLatestCommit {
    pub oid: String,
    pub short_oid: String,
    pub message: String,
    pub href: String,
    pub committed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryLanguageSummary {
    pub language: String,
    pub color: String,
    pub byte_count: i64,
    pub percentage: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCloneUrls {
    pub https: String,
    pub git: String,
    pub zip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySidebarMetadata {
    pub about: Option<String>,
    pub website_url: Option<String>,
    pub topics: Vec<String>,
    pub stars_count: i64,
    pub watchers_count: i64,
    pub forks_count: i64,
    pub releases_count: i64,
    pub deployments_count: i64,
    pub contributors_count: i64,
    pub languages: Vec<RepositoryLanguageSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryViewerState {
    pub starred: bool,
    pub watching: bool,
    pub watch_label: String,
    pub watch_level: RepositoryWatchLevel,
    pub custom_watch_events: Vec<RepositoryWatchEvent>,
    pub forked_repository_href: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySocialState {
    pub starred: bool,
    pub watching: bool,
    pub watch_label: String,
    pub watch_level: RepositoryWatchLevel,
    pub custom_watch_events: Vec<RepositoryWatchEvent>,
    pub stars_count: i64,
    pub watchers_count: i64,
    pub forks_count: i64,
    pub forked_repository_href: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryWatchLevel {
    Participating,
    All,
    Ignore,
    Custom,
}

impl RepositoryWatchLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Participating => "participating",
            Self::All => "all",
            Self::Ignore => "ignore",
            Self::Custom => "custom",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Participating => "Participating and @mentions",
            Self::All => "All Activity",
            Self::Ignore => "Ignoring",
            Self::Custom => "Custom",
        }
    }

    fn is_active(self) -> bool {
        !matches!(self, Self::Ignore)
    }
}

impl TryFrom<&str> for RepositoryWatchLevel {
    type Error = RepositoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "participating" | "subscribed" => Ok(Self::Participating),
            "all" => Ok(Self::All),
            "ignore" => Ok(Self::Ignore),
            "custom" => Ok(Self::Custom),
            other => Err(RepositoryError::InvalidWatchLevel(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryWatchEvent {
    Issues,
    PullRequests,
    Releases,
    Discussions,
    Actions,
    SecurityAlerts,
    RepositoryInvitations,
}

impl RepositoryWatchEvent {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Issues => "issues",
            Self::PullRequests => "pull_requests",
            Self::Releases => "releases",
            Self::Discussions => "discussions",
            Self::Actions => "actions",
            Self::SecurityAlerts => "security_alerts",
            Self::RepositoryInvitations => "repository_invitations",
        }
    }
}

impl TryFrom<&str> for RepositoryWatchEvent {
    type Error = RepositoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "issues" => Ok(Self::Issues),
            "pull_requests" => Ok(Self::PullRequests),
            "releases" => Ok(Self::Releases),
            "discussions" => Ok(Self::Discussions),
            "actions" => Ok(Self::Actions),
            "security_alerts" => Ok(Self::SecurityAlerts),
            "repository_invitations" => Ok(Self::RepositoryInvitations),
            other => Err(RepositoryError::InvalidWatchEvent(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryWatchSettings {
    pub repository_id: Uuid,
    pub level: RepositoryWatchLevel,
    pub label: String,
    pub watching: bool,
    pub watchers_count: i64,
    pub custom_events: Vec<RepositoryWatchEvent>,
    pub available_events: Vec<RepositoryWatchEvent>,
    pub ignore_warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryWatchSettingsPatch {
    pub level: RepositoryWatchLevel,
    #[serde(default)]
    pub custom_events: Vec<RepositoryWatchEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryForkResult {
    pub source_repository_id: Uuid,
    pub fork_repository: Repository,
    pub fork_href: String,
    pub social: RepositorySocialState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryOverview {
    #[serde(flatten)]
    pub repository: Repository,
    pub viewer_permission: Option<String>,
    pub branch_count: i64,
    pub tag_count: i64,
    pub default_branch_ref: Option<GitRef>,
    pub latest_commit: Option<RepositoryLatestCommit>,
    pub root_entries: Vec<RepositoryTreeEntry>,
    pub files: Vec<RepositoryFile>,
    pub readme: Option<RepositoryFile>,
    pub sidebar: RepositorySidebarMetadata,
    pub viewer_state: RepositoryViewerState,
    pub clone_urls: RepositoryCloneUrls,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryPermission {
    pub repository_id: Uuid,
    pub user_id: Uuid,
    pub role: RepositoryRole,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WritableRepositoryOwner {
    pub owner_type: String,
    pub id: Uuid,
    pub login: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub visibility_options: Vec<RepositoryCreationVisibilityOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCreationVisibilityOption {
    pub visibility: RepositoryVisibility,
    pub enabled: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryTemplateOption {
    pub slug: String,
    pub display_name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitignoreTemplateOption {
    pub slug: String,
    pub display_name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LicenseTemplateOption {
    pub slug: String,
    pub display_name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCreationOptions {
    pub owners: Vec<WritableRepositoryOwner>,
    pub templates: Vec<RepositoryTemplateOption>,
    pub gitignore_templates: Vec<GitignoreTemplateOption>,
    pub license_templates: Vec<LicenseTemplateOption>,
    pub suggested_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryNameAvailability {
    pub owner_type: String,
    pub owner_id: Uuid,
    pub owner_login: String,
    pub requested_name: String,
    pub normalized_name: String,
    pub available: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Commit {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub oid: String,
    pub author_user_id: Option<Uuid>,
    pub committer_user_id: Option<Uuid>,
    pub message: String,
    pub tree_oid: Option<String>,
    pub parent_oids: Vec<String>,
    pub committed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitRef {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub name: String,
    pub kind: String,
    pub target_commit_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryResolvedRef {
    pub kind: String,
    pub short_name: String,
    pub qualified_name: String,
    pub target_oid: Option<String>,
    pub recovery_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryRefSummary {
    pub name: String,
    pub short_name: String,
    pub kind: String,
    pub href: String,
    pub same_path_href: String,
    pub active: bool,
    pub target_short_oid: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryFileFinderItem {
    pub path: String,
    pub name: String,
    pub kind: String,
    pub href: String,
    pub byte_size: i64,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryFileFinderResult {
    #[serde(flatten)]
    pub envelope: ListEnvelope<RepositoryFileFinderItem>,
    pub resolved_ref: RepositoryResolvedRef,
    pub default_branch_href: String,
    pub recovery_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrganization {
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub owner_user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepository {
    pub owner: RepositoryOwner,
    pub name: String,
    pub description: Option<String>,
    pub visibility: RepositoryVisibility,
    pub default_branch: Option<String>,
    pub created_by_user_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepositoryBootstrapRequest {
    pub initialize_readme: bool,
    pub template_slug: Option<String>,
    pub gitignore_template_slug: Option<String>,
    pub license_template_slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BootstrapFile {
    pub path: String,
    pub content: String,
    pub oid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositorySnapshotFile {
    pub path: String,
    pub content: String,
    pub oid: String,
    pub byte_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySnapshot {
    pub commit: CreateCommit,
    pub branch_name: String,
    pub files: Vec<RepositorySnapshotFile>,
}

struct IndexedSearchFile {
    path: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommit {
    pub oid: String,
    pub author_user_id: Option<Uuid>,
    pub committer_user_id: Option<Uuid>,
    pub message: String,
    pub tree_oid: Option<String>,
    pub parent_oids: Vec<String>,
    pub committed_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("repository owner was not found")]
    OwnerNotFound,
    #[error("user does not have permission to create repositories for this owner")]
    OwnerPermissionDenied,
    #[error("organization policy does not allow members to create {visibility} repositories")]
    OrganizationRepositoryCreationPolicy {
        visibility: String,
        reason: String,
        settings_href: String,
    },
    #[error("organization policy prevents this repository setting from changing")]
    OrganizationPolicyLocked {
        field: String,
        reason: String,
        settings_href: String,
    },
    #[error("user does not have repository access")]
    PermissionDenied,
    #[error("repository was not found")]
    NotFound,
    #[error("repository path was not found")]
    PathNotFound,
    #[error("repository ref was not found")]
    RefNotFound,
    #[error("repository ref `{ref_name}` was not found")]
    RefNotFoundWithRecovery {
        ref_name: String,
        recovery_href: String,
        default_branch_href: String,
    },
    #[error("repository path `{path}` was not found")]
    PathNotFoundWithRecovery {
        path: String,
        recovery_href: String,
        default_branch_href: String,
    },
    #[error("invalid repository visibility `{0}`")]
    InvalidVisibility(String),
    #[error("invalid repository name `{0}`")]
    InvalidName(String),
    #[error("invalid repository description `{0}`")]
    InvalidDescription(String),
    #[error("invalid merge method `{0}`")]
    InvalidMergeMethod(String),
    #[error("invalid repository watch level `{0}`")]
    InvalidWatchLevel(String),
    #[error("invalid repository watch event `{0}`")]
    InvalidWatchEvent(String),
    #[error("repository default branch `{0}` was not found")]
    DefaultBranchNotFound(String),
    #[error("at least one merge method must remain enabled")]
    MergeMethodRequired,
    #[error("default merge method must be enabled")]
    DefaultMergeMethodDisabled,
    #[error("archived repositories only allow unarchive settings updates")]
    ArchivedRepositoryReadOnly,
    #[error("unknown repository template `{0}`")]
    UnknownTemplate(String),
    #[error("unknown gitignore template `{0}`")]
    UnknownGitignoreTemplate(String),
    #[error("unknown license template `{0}`")]
    UnknownLicenseTemplate(String),
    #[error("repository has already been forked by this user")]
    ForkAlreadyExists,
    #[error("invalid repository access role `{0}`")]
    InvalidAccessRole(String),
    #[error("repository access target was not found")]
    AccessTargetNotFound,
    #[error("repository access grant already exists")]
    AccessGrantConflict,
    #[error("repository must keep at least one owner or admin access path")]
    LastAdminAccess,
    #[error("repository team access is only available for organization repositories")]
    TeamAccessUnsupported,
    #[error("invalid branch policy: {0}")]
    InvalidBranchPolicy(String),
    #[error("invalid branch directory query: {0}")]
    InvalidBranchDirectoryQuery(String),
    #[error("invalid repository pulse query: {0}")]
    InvalidPulseQuery(String),
    #[error("invalid commit diff context: {0}")]
    InvalidDiffContext(String),
    #[error("repository branch policy already exists")]
    BranchPolicyConflict,
    #[error("repository branch policy was not found")]
    BranchPolicyNotFound,
    #[error("repository git storage failed")]
    GitStorageFailed,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub async fn repository_creation_options(
    pool: &PgPool,
    actor_user_id: Uuid,
) -> Result<RepositoryCreationOptions, RepositoryError> {
    let owner_rows = sqlx::query(
        r#"
        SELECT 'user' AS owner_type,
               users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               COALESCE(users.display_name, users.email) AS display_name,
               users.avatar_url,
               NULL::text AS organization_role,
               true AS members_can_create_public_repositories,
               true AS members_can_create_private_repositories,
               false AS members_can_create_internal_repositories,
               0 AS sort_order
        FROM users
        WHERE users.id = $1

        UNION ALL

        SELECT 'organization' AS owner_type,
               organizations.id,
               organizations.slug AS login,
               organizations.display_name,
               NULL::text AS avatar_url,
               organization_memberships.role AS organization_role,
               COALESCE(organization_policy_settings.members_can_create_public_repositories, true) AS members_can_create_public_repositories,
               COALESCE(organization_policy_settings.members_can_create_private_repositories, true) AS members_can_create_private_repositories,
               COALESCE(organization_policy_settings.members_can_create_internal_repositories, false) AS members_can_create_internal_repositories,
               1 AS sort_order
        FROM organizations
        JOIN organization_memberships
          ON organization_memberships.organization_id = organizations.id
        LEFT JOIN organization_policy_settings
          ON organization_policy_settings.organization_id = organizations.id
        WHERE organization_memberships.user_id = $1
        ORDER BY sort_order ASC, login ASC
        "#,
    )
    .bind(actor_user_id)
    .fetch_all(pool)
    .await?;

    let owners = owner_rows
        .into_iter()
        .map(|row| WritableRepositoryOwner {
            owner_type: row.get::<String, _>("owner_type"),
            id: row.get::<Uuid, _>("id"),
            login: row.get::<String, _>("login"),
            display_name: row.get("display_name"),
            avatar_url: row.get("avatar_url"),
            visibility_options: repository_creation_visibility_options_from_row(&row),
        })
        .collect();

    let templates = sqlx::query(
        r#"
        SELECT slug, display_name, description
        FROM repository_creation_templates
        ORDER BY sort_order ASC, display_name ASC
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| RepositoryTemplateOption {
        slug: row.get("slug"),
        display_name: row.get("display_name"),
        description: row.get("description"),
    })
    .collect();

    let gitignore_templates = sqlx::query(
        r#"
        SELECT slug, display_name, description
        FROM gitignore_templates
        ORDER BY sort_order ASC, display_name ASC
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| GitignoreTemplateOption {
        slug: row.get("slug"),
        display_name: row.get("display_name"),
        description: row.get("description"),
    })
    .collect();

    let license_templates = sqlx::query(
        r#"
        SELECT slug, display_name, description
        FROM license_templates
        ORDER BY sort_order ASC, display_name ASC
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| LicenseTemplateOption {
        slug: row.get("slug"),
        display_name: row.get("display_name"),
        description: row.get("description"),
    })
    .collect();

    Ok(RepositoryCreationOptions {
        owners,
        templates,
        gitignore_templates,
        license_templates,
        suggested_name: suggested_repository_name(actor_user_id),
    })
}

pub async fn repository_name_availability(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: RepositoryOwner,
    requested_name: &str,
) -> Result<RepositoryNameAvailability, RepositoryError> {
    ensure_owner_can_create(pool, &owner, actor_user_id).await?;
    let (owner_type, owner_id, owner_login) = repository_owner_login(pool, &owner).await?;
    let normalized_name = normalize_repository_name(requested_name);
    let mut reason = validate_repository_name(&normalized_name).err();
    let exists = if reason.is_none() {
        repository_exists_for_owner(pool, &owner, &normalized_name).await?
    } else {
        false
    };
    if exists {
        reason = Some("A repository with this name already exists for this owner.".to_owned());
    }

    Ok(RepositoryNameAvailability {
        owner_type,
        owner_id,
        owner_login,
        requested_name: requested_name.to_owned(),
        normalized_name,
        available: reason.is_none() && !exists,
        reason,
    })
}

pub async fn create_organization(
    pool: &PgPool,
    input: CreateOrganization,
) -> Result<Organization, RepositoryError> {
    let row = sqlx::query(
        r#"
        INSERT INTO organizations (slug, display_name, description, owner_user_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, slug, display_name, description, owner_user_id, created_at, updated_at
        "#,
    )
    .bind(&input.slug)
    .bind(&input.display_name)
    .bind(&input.description)
    .bind(input.owner_user_id)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO organization_memberships (organization_id, user_id, role)
        VALUES ($1, $2, 'owner')
        ON CONFLICT (organization_id, user_id) DO UPDATE SET role = 'owner'
        "#,
    )
    .bind(row.get::<Uuid, _>("id"))
    .bind(input.owner_user_id)
    .execute(pool)
    .await?;

    Ok(organization_from_row(row))
}

pub async fn create_repository(
    pool: &PgPool,
    input: CreateRepository,
) -> Result<Repository, RepositoryError> {
    create_repository_with_bootstrap(pool, input, RepositoryBootstrapRequest::default()).await
}

pub async fn create_repository_with_bootstrap(
    pool: &PgPool,
    input: CreateRepository,
    bootstrap: RepositoryBootstrapRequest,
) -> Result<Repository, RepositoryError> {
    ensure_owner_can_create(pool, &input.owner, input.created_by_user_id).await?;
    ensure_owner_visibility_can_create(
        pool,
        &input.owner,
        input.created_by_user_id,
        &input.visibility,
    )
    .await?;
    let normalized_name = normalize_repository_name(&input.name);
    validate_repository_name(&normalized_name).map_err(RepositoryError::InvalidName)?;
    let description = normalize_repository_description(input.description)?;

    let (owner_user_id, owner_organization_id) = match input.owner {
        RepositoryOwner::User { id } => (Some(id), None),
        RepositoryOwner::Organization { id } => (None, Some(id)),
    };

    let repository_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO repositories (
            owner_user_id,
            owner_organization_id,
            name,
            description,
            visibility,
            default_branch,
            created_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, COALESCE($6, 'main'), $7)
        RETURNING id
        "#,
    )
    .bind(owner_user_id)
    .bind(owner_organization_id)
    .bind(&normalized_name)
    .bind(&description)
    .bind(input.visibility.as_str())
    .bind(&input.default_branch)
    .bind(input.created_by_user_id)
    .fetch_one(pool)
    .await?;

    let repository = get_repository(pool, repository_id)
        .await?
        .ok_or(RepositoryError::NotFound)?;
    grant_repository_permission(
        pool,
        repository.id,
        input.created_by_user_id,
        RepositoryRole::Owner,
        "owner",
    )
    .await?;
    ensure_default_repository_labels(pool, repository.id).await?;
    bootstrap_repository(pool, &repository, input.created_by_user_id, &bootstrap).await?;
    Ok(repository)
}

pub async fn repository_overview_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryOverview>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    Ok(Some(
        repository_overview_for_actor(pool, repository, actor_user_id).await?,
    ))
}

pub async fn repository_overview_for_viewer_by_owner_name(
    pool: &PgPool,
    actor_user_id: Option<Uuid>,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryOverview>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    repository_overview_for_viewer(pool, repository, actor_user_id)
        .await
        .map(Some)
}

pub async fn repository_refs_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    query: RepositoryRefsQuery<'_>,
) -> Result<Option<ListEnvelope<RepositoryRefSummary>>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    let normalized_query = query.query.unwrap_or("").trim().to_lowercase();
    let current_path = normalize_repository_path(query.current_path.unwrap_or(""))?;
    let active_ref = query
        .active_ref
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&repository.default_branch);
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, 100);

    let rows = sqlx::query(
        r#"
        SELECT repository_git_refs.name,
               repository_git_refs.kind,
               repository_git_refs.updated_at,
               commits.oid AS target_oid,
               commits.id AS target_commit_id
        FROM repository_git_refs
        LEFT JOIN commits ON commits.id = repository_git_refs.target_commit_id
        WHERE repository_git_refs.repository_id = $1
        ORDER BY repository_git_refs.kind ASC, lower(repository_git_refs.name) ASC
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::new();
    for row in rows {
        let name: String = row.get("name");
        let kind: String = row.get("kind");
        let short_name = name
            .strip_prefix("refs/heads/")
            .or_else(|| name.strip_prefix("refs/tags/"))
            .unwrap_or(&name)
            .to_owned();
        if !normalized_query.is_empty()
            && !short_name.to_lowercase().contains(&normalized_query)
            && !name.to_lowercase().contains(&normalized_query)
        {
            continue;
        }
        let target_commit_id = row.get::<Option<Uuid>, _>("target_commit_id");
        let same_path_href = if current_path.is_empty()
            || repository_path_exists_for_commit(
                pool,
                repository.id,
                target_commit_id,
                &current_path,
            )
            .await?
        {
            repository_tree_href(&repository, &short_name, &current_path)
        } else {
            repository_tree_href(&repository, &short_name, "")
        };
        let active = ref_matches_active(&name, &short_name, active_ref);
        items.push(RepositoryRefSummary {
            href: same_path_href.clone(),
            same_path_href,
            active,
            target_short_oid: row
                .get::<Option<String>, _>("target_oid")
                .map(|oid| oid.chars().take(7).collect()),
            updated_at: row.get("updated_at"),
            name,
            short_name,
            kind,
        });
    }
    let total = items.len() as i64;
    let offset = ((page - 1) * page_size) as usize;
    items = items
        .into_iter()
        .skip(offset)
        .take(page_size as usize)
        .collect();

    Ok(Some(ListEnvelope {
        total,
        page,
        page_size,
        items,
    }))
}

pub async fn repository_branches_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    query: RepositoryBranchesQuery<'_>,
) -> Result<Option<RepositoryBranchesView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    repository_branches_for_repository(pool, &repository, actor_user_id, query)
        .await
        .map(Some)
}

pub async fn repository_branch_activity_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    branch: &str,
) -> Result<Option<RepositoryBranchActivityView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    repository_branch_activity_for_repository(pool, &repository, actor_user_id, branch)
        .await
        .map(Some)
}

pub async fn repository_pulse_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    query: RepositoryPulseQuery<'_>,
) -> Result<Option<RepositoryPulseView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    repository_pulse_for_repository(pool, &repository, actor_user_id, query)
        .await
        .map(Some)
}

pub async fn repository_file_finder_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    query: RepositoryFileFinderQuery<'_>,
) -> Result<Option<RepositoryFileFinderResult>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    let resolved_ref = resolve_repository_ref(pool, &repository, query.ref_name).await?;
    let normalized_query = query.query.unwrap_or("").trim().to_lowercase();
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, 100);
    let files = list_repository_files_for_resolved_ref(pool, repository.id, &resolved_ref).await?;
    let mut items = files
        .into_iter()
        .filter(|file| {
            normalized_query.is_empty() || file.path.to_lowercase().contains(&normalized_query)
        })
        .map(|file| {
            let name = file
                .path
                .rsplit('/')
                .next()
                .filter(|value| !value.is_empty())
                .unwrap_or(&file.path)
                .to_owned();
            RepositoryFileFinderItem {
                href: format!(
                    "/{}/{}/blob/{}/{}",
                    repository.owner_login,
                    repository.name,
                    percent_encode_segment(&resolved_ref.short_name),
                    percent_encode_path(&file.path)
                ),
                language: language_for_path(&file.path),
                byte_size: file.byte_size,
                kind: "file".to_owned(),
                name,
                path: file.path,
            }
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.path.to_lowercase().cmp(&right.path.to_lowercase()));
    let total = items.len() as i64;
    let offset = ((page - 1) * page_size) as usize;
    items = items
        .into_iter()
        .skip(offset)
        .take(page_size as usize)
        .collect();

    Ok(Some(RepositoryFileFinderResult {
        default_branch_href: repository_default_branch_href(&repository),
        recovery_href: repository_default_branch_href(&repository),
        resolved_ref,
        envelope: ListEnvelope {
            total,
            page,
            page_size,
            items,
        },
    }))
}

pub async fn repository_overview_for_actor(
    pool: &PgPool,
    repository: Repository,
    actor_user_id: Uuid,
) -> Result<RepositoryOverview, RepositoryError> {
    repository_overview_for_viewer(pool, repository, Some(actor_user_id)).await
}

pub async fn repository_overview_for_viewer(
    pool: &PgPool,
    repository: Repository,
    actor_user_id: Option<Uuid>,
) -> Result<RepositoryOverview, RepositoryError> {
    let files = match resolve_repository_ref(pool, &repository, None).await {
        Ok(resolved_ref) => {
            list_repository_files_for_resolved_ref(pool, repository.id, &resolved_ref).await?
        }
        Err(RepositoryError::RefNotFoundWithRecovery { .. }) => Vec::new(),
        Err(error) => return Err(error),
    };
    let readme = files
        .iter()
        .find(|file| file.path.eq_ignore_ascii_case("README.md"))
        .cloned();
    let viewer_permission = match actor_user_id {
        Some(user_id) => repository_permission_for_user(pool, repository.id, user_id)
            .await?
            .map(|permission| permission.role.as_str().to_owned())
            .or_else(|| {
                if repository.visibility == RepositoryVisibility::Public {
                    Some("read".to_owned())
                } else {
                    None
                }
            }),
        None if repository.visibility == RepositoryVisibility::Public => Some("read".to_owned()),
        None => None,
    };
    if viewer_permission.is_none() {
        return Err(RepositoryError::PermissionDenied);
    }
    if let Some(user_id) = actor_user_id {
        record_recent_repository_visit(pool, user_id, repository.id).await?;
    }
    let branch_count = count_repository_refs(pool, repository.id, "branch").await?;
    let tag_count = count_repository_refs(pool, repository.id, "tag").await?;
    let default_branch_ref = get_repository_ref(
        pool,
        repository.id,
        &format!("refs/heads/{}", repository.default_branch),
    )
    .await?;
    let latest_commit = latest_commit_for_repository(pool, &repository).await?;
    let root_entries = repository_root_entries(&repository, &files, latest_commit.as_ref());
    let sidebar = repository_sidebar_metadata(pool, &repository).await?;
    let viewer_state = match actor_user_id {
        Some(user_id) => repository_viewer_state(pool, &repository, user_id).await?,
        None => RepositoryViewerState {
            starred: false,
            watching: false,
            watch_label: RepositoryWatchLevel::Participating.label().to_owned(),
            watch_level: RepositoryWatchLevel::Participating,
            custom_watch_events: Vec::new(),
            forked_repository_href: None,
        },
    };
    let clone_urls = repository_clone_urls(&repository);
    Ok(RepositoryOverview {
        repository,
        viewer_permission,
        branch_count,
        tag_count,
        default_branch_ref,
        latest_commit,
        root_entries,
        files,
        readme,
        sidebar,
        viewer_state,
        clone_urls,
    })
}

pub async fn set_repository_star_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    starred: bool,
) -> Result<Option<RepositorySocialState>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };

    if starred {
        sqlx::query(
            r#"
            INSERT INTO repository_stars (user_id, repository_id)
            VALUES ($1, $2)
            ON CONFLICT (user_id, repository_id) DO NOTHING
            "#,
        )
        .bind(actor_user_id)
        .bind(repository.id)
        .execute(pool)
        .await?;
        insert_repository_social_feed_event(pool, &repository, actor_user_id, "star").await?;
    } else {
        sqlx::query("DELETE FROM repository_stars WHERE user_id = $1 AND repository_id = $2")
            .bind(actor_user_id)
            .bind(repository.id)
            .execute(pool)
            .await?;
    }

    repository_social_state(pool, &repository, actor_user_id)
        .await
        .map(Some)
}

pub async fn set_repository_watch_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    watching: bool,
) -> Result<Option<RepositorySocialState>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };

    if watching {
        save_repository_watch_settings(
            pool,
            repository.id,
            actor_user_id,
            RepositoryWatchSettingsPatch {
                level: RepositoryWatchLevel::Participating,
                custom_events: Vec::new(),
            },
        )
        .await?;
    } else {
        sqlx::query("DELETE FROM repository_watches WHERE user_id = $1 AND repository_id = $2")
            .bind(actor_user_id)
            .bind(repository.id)
            .execute(pool)
            .await?;
    }

    repository_social_state(pool, &repository, actor_user_id)
        .await
        .map(Some)
}

pub async fn repository_watch_settings_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryWatchSettings>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };

    repository_watch_settings(pool, &repository, actor_user_id)
        .await
        .map(Some)
}

pub async fn update_repository_watch_settings_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    patch: RepositoryWatchSettingsPatch,
) -> Result<Option<RepositoryWatchSettings>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };

    save_repository_watch_settings(pool, repository.id, actor_user_id, patch).await?;
    repository_watch_settings(pool, &repository, actor_user_id)
        .await
        .map(Some)
}

pub async fn repository_watch_delivers_event_by_owner_name(
    pool: &PgPool,
    user_id: Uuid,
    owner_login: &str,
    name: &str,
    event: RepositoryWatchEvent,
) -> Result<bool, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(false);
    };

    if !can_read_repository(pool, &repository, user_id).await? {
        return Ok(false);
    }

    let (level, custom_events) = repository_watch_state(pool, repository.id, user_id).await?;
    Ok(match level {
        RepositoryWatchLevel::Participating => false,
        RepositoryWatchLevel::All => true,
        RepositoryWatchLevel::Ignore => false,
        RepositoryWatchLevel::Custom => custom_events.contains(&event),
    })
}

pub async fn fork_repository_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryForkResult>, RepositoryError> {
    let Some(source_repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };

    if existing_fork_href_for_user(pool, source_repository.id, actor_user_id)
        .await?
        .is_some()
    {
        return Err(RepositoryError::ForkAlreadyExists);
    }

    let fork_repository = create_repository_with_bootstrap(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: actor_user_id },
            name: source_repository.name.clone(),
            description: source_repository.description.clone(),
            visibility: source_repository.visibility.clone(),
            default_branch: Some(source_repository.default_branch.clone()),
            created_by_user_id: actor_user_id,
        },
        RepositoryBootstrapRequest::default(),
    )
    .await?;

    copy_repository_snapshot(pool, &source_repository, &fork_repository, actor_user_id).await?;
    sqlx::query(
        r#"
        INSERT INTO repository_forks (source_repository_id, fork_repository_id, forked_by_user_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(source_repository.id)
    .bind(fork_repository.id)
    .bind(actor_user_id)
    .execute(pool)
    .await?;
    insert_repository_fork_feed_event(pool, &source_repository, &fork_repository, actor_user_id)
        .await?;
    let fork_href = format!("/{}/{}", fork_repository.owner_login, fork_repository.name);
    let social = repository_social_state(pool, &source_repository, actor_user_id).await?;

    Ok(Some(RepositoryForkResult {
        source_repository_id: source_repository.id,
        fork_repository,
        fork_href,
        social,
    }))
}

pub async fn repository_path_overview_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    query: RepositoryPathQuery<'_>,
) -> Result<Option<RepositoryPathOverview>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    repository_path_overview_for_actor(pool, repository, actor_user_id, query)
        .await
        .map(Some)
}

pub async fn repository_blob_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    ref_name: Option<&str>,
    path: &str,
) -> Result<Option<RepositoryBlobView>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    repository_blob_for_actor(pool, repository, actor_user_id, ref_name, path)
        .await
        .map(Some)
}

pub async fn repository_blame_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    ref_name: Option<&str>,
    path: &str,
) -> Result<Option<RepositoryBlameView>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    repository_blame_for_actor(pool, repository, actor_user_id, ref_name, path)
        .await
        .map(Some)
}

pub async fn repository_commit_history_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    query: RepositoryCommitHistoryQuery<'_>,
) -> Result<Option<RepositoryCommitHistoryView>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    let resolved_ref = resolve_repository_ref(pool, &repository, query.ref_name).await?;
    let path = normalize_repository_path(query.path.unwrap_or(""))?;
    if !path.is_empty() {
        let path_prefix = format!("{path}/%");
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM repository_files
                WHERE repository_id = $1
                  AND (path = $2 OR path LIKE $3)
            )
            "#,
        )
        .bind(repository.id)
        .bind(&path)
        .bind(&path_prefix)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(repository_path_not_found_error(&repository, &path));
        }
    }
    repository_commit_history(
        pool,
        &repository,
        &resolved_ref,
        Some(path.as_str()).filter(|value| !value.is_empty()),
        query,
    )
    .await
    .map(Some)
}

pub async fn repository_commit_detail_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    sha: &str,
) -> Result<Option<RepositoryCommitDetailView>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    repository_commit_detail(pool, &repository, actor_user_id, sha)
        .await
        .map(Some)
}

pub async fn repository_commit_detail_context_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    sha: &str,
    query: RepositoryCommitDetailContextQuery<'_>,
) -> Result<Option<RepositoryCommitDetailContext>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    repository_commit_detail_context(pool, &repository, actor_user_id, sha, query)
        .await
        .map(Some)
}

async fn repository_path_overview_for_actor(
    pool: &PgPool,
    repository: Repository,
    actor_user_id: Uuid,
    query: RepositoryPathQuery<'_>,
) -> Result<RepositoryPathOverview, RepositoryError> {
    let resolved_ref = resolve_repository_ref(pool, &repository, query.ref_name).await?;
    let ref_name = resolved_ref.short_name.clone();
    let path = normalize_repository_path(query.path)?;
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, 100);
    let files = list_repository_files_for_resolved_ref(pool, repository.id, &resolved_ref).await?;
    let all_entries = repository_entries_for_path(&repository, &ref_name, &files, &path);
    let readme = readme_for_path(&files, &path);
    if !path.is_empty() && all_entries.is_empty() && readme.is_none() {
        return Err(repository_path_not_found_error(&repository, &path));
    }
    let total = all_entries.len() as i64;
    let offset = ((page - 1) * page_size) as usize;
    let entries = all_entries
        .into_iter()
        .skip(offset)
        .take(page_size as usize)
        .collect::<Vec<_>>();
    let has_more = (offset as i64) + (entries.len() as i64) < total;
    let latest_commit = latest_commit_for_repository(pool, &repository).await?;
    let viewer_permission = viewer_permission_for_user(pool, &repository, actor_user_id).await?;
    let history_href = repository_history_href(&repository, &ref_name, &path);

    Ok(RepositoryPathOverview {
        viewer_permission,
        ref_name: ref_name.clone(),
        resolved_ref,
        default_branch_href: repository_default_branch_href(&repository),
        recovery_href: repository_tree_href(&repository, &ref_name, &path),
        total,
        page,
        page_size,
        has_more,
        path_name: path
            .rsplit('/')
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or(&repository.name)
            .to_owned(),
        breadcrumbs: repository_breadcrumbs(&repository, &ref_name, &path),
        parent_href: repository_parent_tree_href(&repository, &ref_name, &path),
        entries,
        readme,
        latest_commit,
        history_href,
        path,
        repository,
    })
}

async fn repository_blob_for_actor(
    pool: &PgPool,
    repository: Repository,
    actor_user_id: Uuid,
    ref_name: Option<&str>,
    path: &str,
) -> Result<RepositoryBlobView, RepositoryError> {
    let resolved_ref = resolve_repository_ref(pool, &repository, ref_name).await?;
    let ref_name = resolved_ref.short_name.clone();
    let path = normalize_repository_path(path)?;
    if path.is_empty() {
        return Err(repository_path_not_found_error(&repository, &path));
    }
    let files = list_repository_files_for_resolved_ref(pool, repository.id, &resolved_ref).await?;
    let file = files
        .iter()
        .find(|file| file.path == path)
        .cloned()
        .ok_or_else(|| repository_path_not_found_error(&repository, &path))?;
    let latest_commit = latest_commit_for_repository(pool, &repository).await?;
    let latest_path_commit = latest_commit_for_file(pool, &repository, &file).await?;
    let viewer_permission = viewer_permission_for_user(pool, &repository, actor_user_id).await?;
    record_recent_repository_visit(pool, actor_user_id, repository.id).await?;
    let encoded_path = percent_encode_path(&path);
    let base = format!(
        "/{}/{}/{}",
        repository.owner_login, repository.name, encoded_path
    );
    let ref_segment = percent_encode_segment(&ref_name);
    let api_base = format!(
        "/api/repos/{}/{}/blobs/{}?ref={}",
        percent_encode_segment(&repository.owner_login),
        percent_encode_segment(&repository.name),
        encoded_path,
        ref_segment
    );
    let is_binary = is_probably_binary(&file.content);
    let is_large = file.byte_size > 512 * 1024;
    let display_content = if is_binary || is_large {
        None
    } else {
        Some(file.content.chars().take(256 * 1024).collect())
    };

    Ok(RepositoryBlobView {
        viewer_permission,
        ref_name: ref_name.clone(),
        resolved_ref,
        default_branch_href: repository_default_branch_href(&repository),
        recovery_href: repository_tree_href(&repository, &ref_name, parent_path(&path)),
        path_name: path
            .rsplit('/')
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or(&path)
            .to_owned(),
        breadcrumbs: repository_breadcrumbs(&repository, &ref_name, &path),
        parent_href: repository_parent_tree_href(&repository, &ref_name, &path),
        language: language_for_path(&path),
        is_binary,
        is_large,
        line_count: line_count(&file.content),
        loc_count: loc_count(&file.content),
        size_label: format_byte_size(file.byte_size),
        mime_type: mime_type_for_path(&path, is_binary),
        render_mode: render_mode(is_binary, is_large).to_owned(),
        display_content,
        history_href: repository_history_href(&repository, &ref_name, &path),
        raw_href: format!("{base}?raw=1"),
        download_href: format!("{base}?download=1"),
        raw_api_href: format!("{api_base}&raw=1"),
        download_api_href: format!("{api_base}&download=1"),
        permalink_href: latest_path_commit
            .as_ref()
            .map(|commit| {
                format!(
                    "/{}/{}/blob/{}/{}",
                    repository.owner_login,
                    repository.name,
                    percent_encode_segment(&commit.oid),
                    encoded_path
                )
            })
            .unwrap_or_else(|| repository_blob_href(&repository, &ref_name, &path)),
        symbols: if is_binary || is_large {
            Vec::new()
        } else {
            symbols_for_file(&path, &file.content)
        },
        path,
        file,
        latest_commit,
        latest_path_commit,
        repository,
    })
}

async fn repository_blame_for_actor(
    pool: &PgPool,
    repository: Repository,
    actor_user_id: Uuid,
    ref_name: Option<&str>,
    path: &str,
) -> Result<RepositoryBlameView, RepositoryError> {
    let blob =
        repository_blob_for_actor(pool, repository.clone(), actor_user_id, ref_name, path).await?;
    if blob.is_binary || blob.is_large {
        return Err(repository_path_not_found_error(&repository, &blob.path));
    }
    let attribution = blame_commit_for_file(pool, &repository, &blob.file).await?;
    let commit = attribution.or_else(|| {
        blob.latest_path_commit
            .as_ref()
            .map(|latest| RepositoryBlameCommit {
                oid: latest.oid.clone(),
                short_oid: latest.short_oid.clone(),
                message: latest.message.clone(),
                href: latest.href.clone(),
                committed_at: latest.committed_at,
                author_login: None,
            })
    });
    let commit = commit.ok_or(RepositoryError::NotFound)?;
    let content = blob
        .display_content
        .as_deref()
        .unwrap_or(blob.file.content.as_str());
    let lines = blame_lines(content)
        .into_iter()
        .enumerate()
        .map(|(index, content)| RepositoryBlameLine {
            line_number: (index + 1) as i64,
            content,
            commit: commit.clone(),
        })
        .collect();

    Ok(RepositoryBlameView { blob, lines })
}

pub async fn repository_overview(
    pool: &PgPool,
    repository: Repository,
) -> Result<RepositoryOverview, RepositoryError> {
    repository_overview_for_actor(pool, repository.clone(), repository.created_by_user_id).await
}

pub async fn list_repositories_for_user(
    pool: &PgPool,
    user_id: Uuid,
    page: i64,
    page_size: i64,
) -> Result<ListEnvelope<Repository>, RepositoryError> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(DISTINCT repositories.id)
        FROM repositories
        JOIN repository_permissions
          ON repository_permissions.repository_id = repositories.id
        WHERE repository_permissions.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT DISTINCT
            repositories.id,
            repositories.owner_user_id,
            repositories.owner_organization_id,
            COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug) AS owner_login,
            repositories.name,
            repositories.description,
            repositories.visibility,
            repositories.default_branch,
            repositories.is_archived,
            repositories.created_by_user_id,
            repositories.created_at,
            repositories.updated_at
        FROM repositories
        JOIN repository_permissions
          ON repository_permissions.repository_id = repositories.id
        LEFT JOIN users owner_user
          ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations
          ON organizations.id = repositories.owner_organization_id
        WHERE repository_permissions.user_id = $1
        ORDER BY repositories.updated_at DESC, repositories.name ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(repository_from_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListEnvelope {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn get_repository_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<Repository>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };

    if can_read_repository(pool, &repository, actor_user_id).await? {
        Ok(Some(repository))
    } else {
        Err(RepositoryError::PermissionDenied)
    }
}

pub async fn get_repository_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    name: &str,
) -> Result<Option<Repository>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT
            repositories.id,
            repositories.owner_user_id,
            repositories.owner_organization_id,
            COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug) AS owner_login,
            repositories.name,
            repositories.description,
            repositories.visibility,
            repositories.default_branch,
            repositories.is_archived,
            repositories.created_by_user_id,
            repositories.created_at,
            repositories.updated_at
        FROM repositories
        LEFT JOIN users owner_user
          ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations
          ON organizations.id = repositories.owner_organization_id
        WHERE (
            lower(COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug)) = lower($1)
            OR lower(owner_user.email) = lower($1)
        )
          AND lower(repositories.name) = lower($2)
        "#,
    )
    .bind(owner_login)
    .bind(name)
    .fetch_optional(pool)
    .await?;

    row.map(repository_from_row).transpose()
}

pub async fn get_repository(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Option<Repository>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT
            repositories.id,
            repositories.owner_user_id,
            repositories.owner_organization_id,
            COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug) AS owner_login,
            repositories.name,
            repositories.description,
            repositories.visibility,
            repositories.default_branch,
            repositories.is_archived,
            repositories.created_by_user_id,
            repositories.created_at,
            repositories.updated_at
        FROM repositories
        LEFT JOIN users owner_user
          ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations
          ON organizations.id = repositories.owner_organization_id
        WHERE repositories.id = $1
        "#,
    )
    .bind(repository_id)
    .fetch_optional(pool)
    .await?;

    row.map(repository_from_row).transpose()
}

pub async fn grant_repository_permission(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
    role: RepositoryRole,
    source: &str,
) -> Result<RepositoryPermission, RepositoryError> {
    let row = sqlx::query(
        r#"
        INSERT INTO repository_permissions (repository_id, user_id, role, source)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (repository_id, user_id)
        DO UPDATE SET role = EXCLUDED.role, source = EXCLUDED.source
        RETURNING repository_id, user_id, role, source
        "#,
    )
    .bind(repository_id)
    .bind(user_id)
    .bind(role.as_str())
    .bind(source)
    .fetch_one(pool)
    .await?;

    repository_permission_from_row(row)
}

pub async fn repository_permission_for_user(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
) -> Result<Option<RepositoryPermission>, RepositoryError> {
    let row = sqlx::query(
        r#"
        WITH RECURSIVE user_teams AS (
            SELECT teams.id, teams.parent_team_id, 0 AS depth
            FROM team_memberships
            JOIN teams ON teams.id = team_memberships.team_id
            WHERE team_memberships.user_id = $2
            UNION
            SELECT parent.id, parent.parent_team_id, user_teams.depth + 1
            FROM teams parent
            JOIN user_teams ON user_teams.parent_team_id = parent.id
            WHERE user_teams.depth < 24
        ),
        candidates AS (
            SELECT repository_id, user_id, role, source, 0 AS source_rank
            FROM repository_permissions
            WHERE repository_id = $1 AND user_id = $2
            UNION ALL
            SELECT repository_team_permissions.repository_id,
                   $2 AS user_id,
                   repository_team_permissions.role,
                   'team' AS source,
                   1 AS source_rank
            FROM repository_team_permissions
            JOIN user_teams ON user_teams.id = repository_team_permissions.team_id
            WHERE repository_team_permissions.repository_id = $1
            UNION ALL
            SELECT repositories.id AS repository_id,
                   $2 AS user_id,
                   organization_policy_settings.base_repository_permission AS role,
                   'organization' AS source,
                   2 AS source_rank
            FROM repositories
            JOIN organization_policy_settings
              ON organization_policy_settings.organization_id = repositories.owner_organization_id
            JOIN organization_memberships
              ON organization_memberships.organization_id = repositories.owner_organization_id
             AND organization_memberships.user_id = $2
            WHERE repositories.id = $1
              AND organization_policy_settings.base_repository_permission <> 'none'
        )
        SELECT repository_id, user_id, role, source
        FROM candidates
        ORDER BY
          CASE role
            WHEN 'owner' THEN 6
            WHEN 'admin' THEN 5
            WHEN 'maintain' THEN 4
            WHEN 'write' THEN 3
            WHEN 'triage' THEN 2
            WHEN 'read' THEN 1
            ELSE 0
          END DESC,
          source_rank ASC
        LIMIT 1
        "#,
    )
    .bind(repository_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    row.map(repository_permission_from_row).transpose()
}

pub async fn can_read_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<bool, RepositoryError> {
    if repository.visibility == RepositoryVisibility::Public {
        return Ok(true);
    }

    if repository.owner_user_id == Some(actor_user_id) {
        return Ok(true);
    }

    if let Some(organization_id) = repository.owner_organization_id {
        let is_org_member = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM organization_memberships
                WHERE organization_id = $1 AND user_id = $2
            )
            "#,
        )
        .bind(organization_id)
        .bind(actor_user_id)
        .fetch_one(pool)
        .await?;

        if is_org_member && repository.visibility == RepositoryVisibility::Internal {
            return Ok(true);
        }
    }

    Ok(
        repository_permission_for_user(pool, repository.id, actor_user_id)
            .await?
            .is_some_and(|permission| permission.role.can_read()),
    )
}

pub async fn can_write_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<bool, RepositoryError> {
    if repository.owner_user_id == Some(actor_user_id) {
        return Ok(true);
    }

    Ok(
        repository_permission_for_user(pool, repository.id, actor_user_id)
            .await?
            .is_some_and(|permission| permission.role.can_write()),
    )
}

pub async fn can_admin_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<bool, RepositoryError> {
    if repository.owner_user_id == Some(actor_user_id) {
        return Ok(true);
    }

    Ok(
        repository_permission_for_user(pool, repository.id, actor_user_id)
            .await?
            .is_some_and(|permission| permission.role.can_admin()),
    )
}

pub async fn repository_access_settings_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryAccessSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_access_admin(pool, &repository, actor_user_id).await?;
    repository_access_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn invite_repository_access_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    request: RepositoryAccessInviteRequest,
) -> Result<Option<RepositoryAccessSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_access_admin(pool, &repository, actor_user_id).await?;
    validate_grant_role(request.role)?;

    let target = normalize_access_target(&request.email_or_login)?;
    let invited_user = find_user_for_access_target(pool, &target).await?;
    let invited_email = invited_user
        .as_ref()
        .map(|user| user.email.clone())
        .unwrap_or_else(|| target.clone());
    if let Some(invited_user) = &invited_user {
        if repository_permission_for_user(pool, repository.id, invited_user.user_id)
            .await?
            .is_some()
        {
            return Err(RepositoryError::AccessGrantConflict);
        }
    }
    let token_hash = format!("{:x}", Sha256::digest(Uuid::new_v4().as_bytes()));

    let inserted = sqlx::query(
        r#"
        INSERT INTO repository_invitations (
            repository_id,
            invited_user_id,
            invited_email,
            role,
            token_hash,
            invited_by_user_id,
            email_delivery_status,
            expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'degraded', now() + interval '7 days')
        ON CONFLICT (repository_id, lower(invited_email)) WHERE status = 'pending'
        DO NOTHING
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(invited_user.as_ref().map(|user| user.user_id))
    .bind(&invited_email)
    .bind(request.role.as_str())
    .bind(token_hash)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?;

    if inserted.is_none() {
        return Err(RepositoryError::AccessGrantConflict);
    }
    insert_repository_access_audit_event(
        pool,
        repository.id,
        actor_user_id,
        "repository.access.invite",
        vec!["invitations".to_owned()],
        json!({}),
        json!({ "invitedEmail": invited_email, "role": request.role }),
    )
    .await?;
    repository_access_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn grant_repository_team_access_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    request: RepositoryAccessTeamGrantRequest,
) -> Result<Option<RepositoryAccessSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_access_admin(pool, &repository, actor_user_id).await?;
    validate_grant_role(request.role)?;
    let Some(organization_id) = repository.owner_organization_id else {
        return Err(RepositoryError::TeamAccessUnsupported);
    };
    let team = find_team_for_access_target(pool, organization_id, &request.team_slug)
        .await?
        .ok_or(RepositoryError::AccessTargetNotFound)?;

    let existing = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM repository_team_permissions
            WHERE repository_id = $1 AND team_id = $2
        )
        "#,
    )
    .bind(repository.id)
    .bind(team.team_id)
    .fetch_one(pool)
    .await?;
    if existing {
        return Err(RepositoryError::AccessGrantConflict);
    }

    sqlx::query(
        r#"
        INSERT INTO repository_team_permissions (repository_id, team_id, role, source, created_by_user_id)
        VALUES ($1, $2, $3, 'team', $4)
        "#,
    )
    .bind(repository.id)
    .bind(team.team_id)
    .bind(request.role.as_str())
    .bind(actor_user_id)
    .execute(pool)
    .await?;

    upsert_team_member_repository_permissions(pool, repository.id, team.team_id, request.role)
        .await?;
    insert_repository_access_audit_event(
        pool,
        repository.id,
        actor_user_id,
        "repository.access.team_grant",
        vec!["teams".to_owned()],
        json!({ "teamSlug": team.slug }),
        json!({ "teamSlug": team.slug, "role": request.role }),
    )
    .await?;
    repository_access_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn update_repository_collaborator_access_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    user_id: Uuid,
    patch: RepositoryAccessRolePatch,
) -> Result<Option<RepositoryAccessSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_access_admin(pool, &repository, actor_user_id).await?;
    validate_grant_role(patch.role)?;
    let existing = repository_permission_for_user(pool, repository.id, user_id)
        .await?
        .ok_or(RepositoryError::AccessTargetNotFound)?;
    if existing.source != "direct" {
        return Err(RepositoryError::PermissionDenied);
    }
    if existing.role.can_admin() && !patch.role.can_admin() {
        ensure_admin_path_remains(pool, repository.id, Some(user_id), None).await?;
    }

    sqlx::query(
        "UPDATE repository_permissions SET role = $3 WHERE repository_id = $1 AND user_id = $2 AND source = 'direct'",
    )
    .bind(repository.id)
    .bind(user_id)
    .bind(patch.role.as_str())
    .execute(pool)
    .await?;
    insert_repository_access_audit_event(
        pool,
        repository.id,
        actor_user_id,
        "repository.access.role_update",
        vec!["people".to_owned()],
        json!({ "userId": user_id, "role": existing.role }),
        json!({ "userId": user_id, "role": patch.role }),
    )
    .await?;
    repository_access_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn update_repository_team_access_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    team_id: Uuid,
    patch: RepositoryAccessRolePatch,
) -> Result<Option<RepositoryAccessSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_access_admin(pool, &repository, actor_user_id).await?;
    validate_grant_role(patch.role)?;
    let row = sqlx::query(
        r#"
        SELECT role, source
        FROM repository_team_permissions
        WHERE repository_id = $1 AND team_id = $2
        "#,
    )
    .bind(repository.id)
    .bind(team_id)
    .fetch_optional(pool)
    .await?
    .ok_or(RepositoryError::AccessTargetNotFound)?;
    let source: String = row.get("source");
    if source != "team" {
        return Err(RepositoryError::PermissionDenied);
    }
    let current_role = RepositoryRole::try_from(row.get::<String, _>("role").as_str())
        .map_err(|error| RepositoryError::Sqlx(sqlx::Error::Protocol(error.to_string())))?;
    if current_role.can_admin() && !patch.role.can_admin() {
        ensure_admin_path_remains(pool, repository.id, None, Some(team_id)).await?;
    }

    sqlx::query(
        r#"
        UPDATE repository_team_permissions
        SET role = $3
        WHERE repository_id = $1 AND team_id = $2 AND source = 'team'
        "#,
    )
    .bind(repository.id)
    .bind(team_id)
    .bind(patch.role.as_str())
    .execute(pool)
    .await?;

    upsert_team_member_repository_permissions(pool, repository.id, team_id, patch.role).await?;
    insert_repository_access_audit_event(
        pool,
        repository.id,
        actor_user_id,
        "repository.access.team_role_update",
        vec!["teams".to_owned()],
        json!({ "teamId": team_id }),
        json!({ "teamId": team_id, "role": patch.role }),
    )
    .await?;
    repository_access_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn remove_repository_team_access_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    team_id: Uuid,
) -> Result<Option<RepositoryAccessSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_access_admin(pool, &repository, actor_user_id).await?;
    let current = sqlx::query(
        r#"
        SELECT role, source
        FROM repository_team_permissions
        WHERE repository_id = $1 AND team_id = $2
        "#,
    )
    .bind(repository.id)
    .bind(team_id)
    .fetch_optional(pool)
    .await?
    .ok_or(RepositoryError::AccessTargetNotFound)?;
    let source: String = current.get("source");
    if source != "team" {
        return Err(RepositoryError::PermissionDenied);
    }
    let deleted_role = RepositoryRole::try_from(current.get::<String, _>("role").as_str())
        .map_err(|error| RepositoryError::Sqlx(sqlx::Error::Protocol(error.to_string())))?;
    if deleted_role.can_admin() {
        ensure_admin_path_remains(pool, repository.id, None, Some(team_id)).await?;
    }

    sqlx::query(
        r#"
        DELETE FROM repository_team_permissions
        WHERE repository_id = $1 AND team_id = $2 AND source = 'team'
        "#,
    )
    .bind(repository.id)
    .bind(team_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM repository_permissions
        WHERE repository_id = $1
          AND source = 'team'
          AND user_id IN (
              SELECT user_id FROM team_memberships WHERE team_id = $2
          )
        "#,
    )
    .bind(repository.id)
    .bind(team_id)
    .execute(pool)
    .await?;

    insert_repository_access_audit_event(
        pool,
        repository.id,
        actor_user_id,
        "repository.access.team_remove",
        vec!["teams".to_owned()],
        json!({ "teamId": team_id, "role": deleted_role }),
        json!({}),
    )
    .await?;
    repository_access_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn remove_repository_collaborator_access_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    user_id: Uuid,
) -> Result<Option<RepositoryAccessSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_access_admin(pool, &repository, actor_user_id).await?;
    let existing = repository_permission_for_user(pool, repository.id, user_id)
        .await?
        .ok_or(RepositoryError::AccessTargetNotFound)?;
    if existing.source != "direct" {
        return Err(RepositoryError::PermissionDenied);
    }
    if existing.role.can_admin() {
        ensure_admin_path_remains(pool, repository.id, Some(user_id), None).await?;
    }
    sqlx::query(
        "DELETE FROM repository_permissions WHERE repository_id = $1 AND user_id = $2 AND source = 'direct'",
    )
    .bind(repository.id)
    .bind(user_id)
    .execute(pool)
    .await?;
    insert_repository_access_audit_event(
        pool,
        repository.id,
        actor_user_id,
        "repository.access.remove",
        vec!["people".to_owned()],
        json!({ "userId": user_id, "role": existing.role }),
        json!({}),
    )
    .await?;
    repository_access_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn cancel_repository_invitation_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    invitation_id: Uuid,
) -> Result<Option<RepositoryAccessSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_access_admin(pool, &repository, actor_user_id).await?;
    let row = sqlx::query(
        r#"
        UPDATE repository_invitations
        SET status = 'canceled', canceled_at = now()
        WHERE id = $1 AND repository_id = $2 AND status = 'pending'
        RETURNING invited_email, role
        "#,
    )
    .bind(invitation_id)
    .bind(repository.id)
    .fetch_optional(pool)
    .await?
    .ok_or(RepositoryError::AccessTargetNotFound)?;
    let invited_email: String = row.get("invited_email");
    let role: String = row.get("role");
    insert_repository_access_audit_event(
        pool,
        repository.id,
        actor_user_id,
        "repository.access.invite_cancel",
        vec!["invitations".to_owned()],
        json!({ "invitedEmail": invited_email, "role": role }),
        json!({ "status": "canceled" }),
    )
    .await?;
    repository_access_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn repository_settings_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositorySettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_admin_repository(pool, &repository, actor_user_id).await? {
        return Err(RepositoryError::PermissionDenied);
    }
    repository_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn update_repository_settings_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    patch: RepositorySettingsPatch,
) -> Result<Option<RepositorySettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_admin_repository(pool, &repository, actor_user_id).await? {
        return Err(RepositoryError::PermissionDenied);
    }

    enforce_repository_policy_locks(pool, &repository, actor_user_id, &patch).await?;
    validate_settings_patch(pool, &repository, &patch).await?;
    let before = repository_settings_for_repository(pool, &repository, actor_user_id)
        .await?
        .ok_or(RepositoryError::NotFound)?;
    let changed_fields = changed_settings_fields(&before, &patch);
    if changed_fields.is_empty() {
        return Ok(Some(before));
    }

    let mut transaction = pool.begin().await?;
    let next_name = patch
        .name
        .as_deref()
        .map(str::trim)
        .unwrap_or(&before.name)
        .to_owned();
    let next_description = patch
        .description
        .clone()
        .unwrap_or(before.description.clone());
    let next_visibility = patch
        .visibility
        .clone()
        .unwrap_or(before.visibility.clone());
    let next_default_branch = patch
        .default_branch
        .as_deref()
        .map(str::trim)
        .unwrap_or(&before.default_branch)
        .to_owned();

    let features_patch = patch.features.clone().unwrap_or_default();
    let merge_patch = patch.merge.clone().unwrap_or_default();
    let next_features = RepositoryFeatureSettings {
        issues_enabled: features_patch
            .issues_enabled
            .unwrap_or(before.features.issues_enabled),
        projects_enabled: features_patch
            .projects_enabled
            .unwrap_or(before.features.projects_enabled),
        wiki_enabled: features_patch
            .wiki_enabled
            .unwrap_or(before.features.wiki_enabled),
    };
    let next_merge = RepositoryMergeSettings {
        allow_squash: merge_patch
            .allow_squash
            .unwrap_or(before.merge.allow_squash),
        allow_merge_commit: merge_patch
            .allow_merge_commit
            .unwrap_or(before.merge.allow_merge_commit),
        allow_rebase: merge_patch
            .allow_rebase
            .unwrap_or(before.merge.allow_rebase),
        default_method: merge_patch
            .default_method
            .unwrap_or(before.merge.default_method.clone()),
    };

    sqlx::query(
        r#"
        UPDATE repositories
        SET name = $2,
            description = $3,
            visibility = $4,
            default_branch = $5,
            is_archived = $6,
            is_template = $7,
            issues_enabled = $8,
            projects_enabled = $9,
            wiki_enabled = $10,
            allow_forking = $11,
            web_commit_signoff_required = $12
        WHERE id = $1
        "#,
    )
    .bind(repository.id)
    .bind(&next_name)
    .bind(&next_description)
    .bind(next_visibility.as_str())
    .bind(&next_default_branch)
    .bind(patch.is_archived.unwrap_or(before.danger.is_archived))
    .bind(patch.is_template.unwrap_or(before.is_template))
    .bind(next_features.issues_enabled)
    .bind(next_features.projects_enabled)
    .bind(next_features.wiki_enabled)
    .bind(patch.allow_forking.unwrap_or(before.allow_forking))
    .bind(
        patch
            .web_commit_signoff_required
            .unwrap_or(before.web_commit_signoff_required),
    )
    .execute(&mut *transaction)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO repository_merge_settings (
            repository_id, allow_squash, allow_merge_commit, allow_rebase, default_method
        )
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (repository_id)
        DO UPDATE SET
            allow_squash = EXCLUDED.allow_squash,
            allow_merge_commit = EXCLUDED.allow_merge_commit,
            allow_rebase = EXCLUDED.allow_rebase,
            default_method = EXCLUDED.default_method
        "#,
    )
    .bind(repository.id)
    .bind(next_merge.allow_squash)
    .bind(next_merge.allow_merge_commit)
    .bind(next_merge.allow_rebase)
    .bind(next_merge.default_method.as_str())
    .execute(&mut *transaction)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO repository_settings_audit_events (
            repository_id, actor_user_id, event_type, changed_fields, before_state, after_state
        )
        VALUES ($1, $2, 'repository.settings.update', $3, $4, $5)
        "#,
    )
    .bind(repository.id)
    .bind(actor_user_id)
    .bind(&changed_fields)
    .bind(json!(before))
    .bind(json!({
        "name": next_name,
        "description": next_description,
        "visibility": next_visibility,
        "defaultBranch": next_default_branch,
        "features": next_features,
        "merge": next_merge,
    }))
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    let updated = get_repository(pool, repository.id)
        .await?
        .ok_or(RepositoryError::NotFound)?;
    repository_settings_for_repository(pool, &updated, actor_user_id).await
}

async fn enforce_repository_policy_locks(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    patch: &RepositorySettingsPatch,
) -> Result<(), RepositoryError> {
    let locks = repository_policy_locks(pool, repository, actor_user_id).await?;
    let blocked = |field: &str| locks.iter().find(|lock| lock.field == field);
    if patch.visibility.is_some() {
        if let Some(lock) = blocked("visibility") {
            return Err(RepositoryError::OrganizationPolicyLocked {
                field: lock.field.clone(),
                reason: lock.reason.clone(),
                settings_href: lock.settings_href.clone(),
            });
        }
    }
    if patch.allow_forking == Some(true) {
        if let Some(lock) = blocked("allowForking") {
            return Err(RepositoryError::OrganizationPolicyLocked {
                field: lock.field.clone(),
                reason: lock.reason.clone(),
                settings_href: lock.settings_href.clone(),
            });
        }
    }
    Ok(())
}

pub async fn repository_branch_settings_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryBranchSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        return Err(RepositoryError::PermissionDenied);
    }
    repository_branch_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn create_repository_branch_rule_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    mutation: RepositoryBranchRuleMutation,
) -> Result<Option<RepositoryBranchSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_access_admin(pool, &repository, actor_user_id).await?;
    let rule = normalize_branch_rule_mutation(mutation)?;
    ensure_unique_branch_rule_pattern(pool, repository.id, &rule.pattern, None).await?;

    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO repository_branch_protection_rules (
            repository_id, pattern, description, enforcement,
            required_approving_review_count, requires_up_to_date_branch,
            requires_conversation_resolution, requires_signed_commits, requires_linear_history,
            requires_merge_queue, requires_deployments, required_deployment_environments,
            locked, restricts_pushes, allows_force_pushes, allows_deletions, bypass_actors
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(&rule.pattern)
    .bind(&rule.description)
    .bind(rule.enforcement.as_str())
    .bind(rule.requirements.required_approving_review_count)
    .bind(rule.requirements.requires_up_to_date_branch)
    .bind(rule.requirements.requires_conversation_resolution)
    .bind(rule.requirements.requires_signed_commits)
    .bind(rule.requirements.requires_linear_history)
    .bind(rule.requirements.requires_merge_queue)
    .bind(rule.requirements.requires_deployments)
    .bind(&rule.requirements.required_deployment_environments)
    .bind(rule.requirements.locked)
    .bind(rule.requirements.restricts_pushes)
    .bind(rule.requirements.allows_force_pushes)
    .bind(rule.requirements.allows_deletions)
    .bind(json!(rule.bypass_actors))
    .fetch_one(&mut *transaction)
    .await?;
    let rule_id: Uuid = row.get("id");
    replace_required_status_checks(
        &mut transaction,
        rule_id,
        &rule.requirements.required_status_checks,
    )
    .await?;
    insert_repository_settings_audit_event_tx(
        &mut transaction,
        repository.id,
        actor_user_id,
        "repository.branch_rule.create",
        vec!["branch_rules".to_owned()],
        json!({}),
        json!({ "id": rule_id, "pattern": rule.pattern }),
    )
    .await?;
    transaction.commit().await?;
    repository_branch_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn update_repository_branch_rule_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    rule_id: Uuid,
    mutation: RepositoryBranchRuleMutation,
) -> Result<Option<RepositoryBranchSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_access_admin(pool, &repository, actor_user_id).await?;
    let existing = branch_rule_exists(pool, repository.id, rule_id).await?;
    let rule = normalize_branch_rule_mutation(mutation)?;
    ensure_unique_branch_rule_pattern(pool, repository.id, &rule.pattern, Some(rule_id)).await?;

    let mut transaction = pool.begin().await?;
    let updated = sqlx::query(
        r#"
        UPDATE repository_branch_protection_rules
        SET pattern = $3,
            description = $4,
            enforcement = $5,
            required_approving_review_count = $6,
            requires_up_to_date_branch = $7,
            requires_conversation_resolution = $8,
            requires_signed_commits = $9,
            requires_linear_history = $10,
            requires_merge_queue = $11,
            requires_deployments = $12,
            required_deployment_environments = $13,
            locked = $14,
            restricts_pushes = $15,
            allows_force_pushes = $16,
            allows_deletions = $17,
            bypass_actors = $18
        WHERE repository_id = $1 AND id = $2
        "#,
    )
    .bind(repository.id)
    .bind(rule_id)
    .bind(&rule.pattern)
    .bind(&rule.description)
    .bind(rule.enforcement.as_str())
    .bind(rule.requirements.required_approving_review_count)
    .bind(rule.requirements.requires_up_to_date_branch)
    .bind(rule.requirements.requires_conversation_resolution)
    .bind(rule.requirements.requires_signed_commits)
    .bind(rule.requirements.requires_linear_history)
    .bind(rule.requirements.requires_merge_queue)
    .bind(rule.requirements.requires_deployments)
    .bind(&rule.requirements.required_deployment_environments)
    .bind(rule.requirements.locked)
    .bind(rule.requirements.restricts_pushes)
    .bind(rule.requirements.allows_force_pushes)
    .bind(rule.requirements.allows_deletions)
    .bind(json!(rule.bypass_actors))
    .execute(&mut *transaction)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(RepositoryError::BranchPolicyNotFound);
    }
    replace_required_status_checks(
        &mut transaction,
        rule_id,
        &rule.requirements.required_status_checks,
    )
    .await?;
    insert_repository_settings_audit_event_tx(
        &mut transaction,
        repository.id,
        actor_user_id,
        "repository.branch_rule.update",
        vec!["branch_rules".to_owned()],
        existing,
        json!({ "id": rule_id, "pattern": rule.pattern }),
    )
    .await?;
    transaction.commit().await?;
    repository_branch_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn delete_repository_branch_rule_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    rule_id: Uuid,
) -> Result<Option<RepositoryBranchSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_access_admin(pool, &repository, actor_user_id).await?;
    let existing = branch_rule_exists(pool, repository.id, rule_id).await?;
    let mut transaction = pool.begin().await?;
    let deleted = sqlx::query(
        "DELETE FROM repository_branch_protection_rules WHERE repository_id = $1 AND id = $2",
    )
    .bind(repository.id)
    .bind(rule_id)
    .execute(&mut *transaction)
    .await?;
    if deleted.rows_affected() == 0 {
        return Err(RepositoryError::BranchPolicyNotFound);
    }
    insert_repository_settings_audit_event_tx(
        &mut transaction,
        repository.id,
        actor_user_id,
        "repository.branch_rule.delete",
        vec!["branch_rules".to_owned()],
        existing,
        json!({}),
    )
    .await?;
    transaction.commit().await?;
    repository_branch_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn create_repository_ruleset_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    mutation: RepositoryRulesetMutation,
) -> Result<Option<RepositoryBranchSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_access_admin(pool, &repository, actor_user_id).await?;
    let ruleset = normalize_ruleset_mutation(mutation)?;
    ensure_unique_ruleset_name(pool, repository.id, &ruleset.name, None).await?;
    sqlx::query(
        r#"
        INSERT INTO repository_rulesets (
            repository_id, name, enforcement, patterns, required_approving_review_count,
            requires_up_to_date_branch, required_status_checks, requires_conversation_resolution,
            requires_signed_commits, requires_linear_history, requires_merge_queue,
            requires_deployments, required_deployment_environments, locked, restricts_pushes,
            allows_force_pushes, allows_deletions, bypass_actors
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
        "#,
    )
    .bind(repository.id)
    .bind(&ruleset.name)
    .bind(ruleset.enforcement.as_str())
    .bind(&ruleset.patterns)
    .bind(ruleset.requirements.required_approving_review_count)
    .bind(ruleset.requirements.requires_up_to_date_branch)
    .bind(&ruleset.requirements.required_status_checks)
    .bind(ruleset.requirements.requires_conversation_resolution)
    .bind(ruleset.requirements.requires_signed_commits)
    .bind(ruleset.requirements.requires_linear_history)
    .bind(ruleset.requirements.requires_merge_queue)
    .bind(ruleset.requirements.requires_deployments)
    .bind(&ruleset.requirements.required_deployment_environments)
    .bind(ruleset.requirements.locked)
    .bind(ruleset.requirements.restricts_pushes)
    .bind(ruleset.requirements.allows_force_pushes)
    .bind(ruleset.requirements.allows_deletions)
    .bind(json!(ruleset.bypass_actors))
    .execute(pool)
    .await?;
    insert_repository_access_audit_event(
        pool,
        repository.id,
        actor_user_id,
        "repository.ruleset.create",
        vec!["rulesets".to_owned()],
        json!({}),
        json!({ "name": ruleset.name }),
    )
    .await?;
    repository_branch_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn update_repository_ruleset_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    ruleset_id: Uuid,
    mutation: RepositoryRulesetMutation,
) -> Result<Option<RepositoryBranchSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_access_admin(pool, &repository, actor_user_id).await?;
    let ruleset = normalize_ruleset_mutation(mutation)?;
    ensure_unique_ruleset_name(pool, repository.id, &ruleset.name, Some(ruleset_id)).await?;
    let updated = sqlx::query(
        r#"
        UPDATE repository_rulesets
        SET name = $3,
            enforcement = $4,
            patterns = $5,
            required_approving_review_count = $6,
            requires_up_to_date_branch = $7,
            required_status_checks = $8,
            requires_conversation_resolution = $9,
            requires_signed_commits = $10,
            requires_linear_history = $11,
            requires_merge_queue = $12,
            requires_deployments = $13,
            required_deployment_environments = $14,
            locked = $15,
            restricts_pushes = $16,
            allows_force_pushes = $17,
            allows_deletions = $18,
            bypass_actors = $19
        WHERE repository_id = $1 AND id = $2
        "#,
    )
    .bind(repository.id)
    .bind(ruleset_id)
    .bind(&ruleset.name)
    .bind(ruleset.enforcement.as_str())
    .bind(&ruleset.patterns)
    .bind(ruleset.requirements.required_approving_review_count)
    .bind(ruleset.requirements.requires_up_to_date_branch)
    .bind(&ruleset.requirements.required_status_checks)
    .bind(ruleset.requirements.requires_conversation_resolution)
    .bind(ruleset.requirements.requires_signed_commits)
    .bind(ruleset.requirements.requires_linear_history)
    .bind(ruleset.requirements.requires_merge_queue)
    .bind(ruleset.requirements.requires_deployments)
    .bind(&ruleset.requirements.required_deployment_environments)
    .bind(ruleset.requirements.locked)
    .bind(ruleset.requirements.restricts_pushes)
    .bind(ruleset.requirements.allows_force_pushes)
    .bind(ruleset.requirements.allows_deletions)
    .bind(json!(ruleset.bypass_actors))
    .execute(pool)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(RepositoryError::BranchPolicyNotFound);
    }
    insert_repository_access_audit_event(
        pool,
        repository.id,
        actor_user_id,
        "repository.ruleset.update",
        vec!["rulesets".to_owned()],
        json!({ "id": ruleset_id }),
        json!({ "id": ruleset_id, "name": ruleset.name }),
    )
    .await?;
    repository_branch_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn delete_repository_ruleset_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    ruleset_id: Uuid,
) -> Result<Option<RepositoryBranchSettings>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_access_admin(pool, &repository, actor_user_id).await?;
    let deleted =
        sqlx::query("DELETE FROM repository_rulesets WHERE repository_id = $1 AND id = $2")
            .bind(repository.id)
            .bind(ruleset_id)
            .execute(pool)
            .await?;
    if deleted.rows_affected() == 0 {
        return Err(RepositoryError::BranchPolicyNotFound);
    }
    insert_repository_access_audit_event(
        pool,
        repository.id,
        actor_user_id,
        "repository.ruleset.delete",
        vec!["rulesets".to_owned()],
        json!({ "id": ruleset_id }),
        json!({}),
    )
    .await?;
    repository_branch_settings_for_repository(pool, &repository, actor_user_id).await
}

async fn validate_settings_patch(
    pool: &PgPool,
    repository: &Repository,
    patch: &RepositorySettingsPatch,
) -> Result<(), RepositoryError> {
    if repository.is_archived && !settings_patch_only_unarchives(patch) {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    if let Some(name) = patch.name.as_deref() {
        validate_repository_name(name.trim()).map_err(RepositoryError::InvalidName)?;
    }
    if let Some(Some(description)) = patch.description.as_ref() {
        if description.len() > 500 {
            return Err(RepositoryError::InvalidDescription(
                "Repository description must be 500 characters or fewer.".to_owned(),
            ));
        }
    }
    if let Some(default_branch) = patch.default_branch.as_deref() {
        let branch = default_branch.trim();
        if branch.is_empty() {
            return Err(RepositoryError::DefaultBranchNotFound(branch.to_owned()));
        }
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM repository_git_refs
                WHERE repository_id = $1
                  AND kind = 'branch'
                  AND (name = $2 OR name = $3)
            )
            "#,
        )
        .bind(repository.id)
        .bind(branch)
        .bind(format!("refs/heads/{branch}"))
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(RepositoryError::DefaultBranchNotFound(branch.to_owned()));
        }
    }

    if let Some(merge_patch) = &patch.merge {
        let current = repository_merge_settings_for_repository(pool, repository.id).await?;
        let next = RepositoryMergeSettings {
            allow_squash: merge_patch.allow_squash.unwrap_or(current.allow_squash),
            allow_merge_commit: merge_patch
                .allow_merge_commit
                .unwrap_or(current.allow_merge_commit),
            allow_rebase: merge_patch.allow_rebase.unwrap_or(current.allow_rebase),
            default_method: merge_patch
                .default_method
                .clone()
                .unwrap_or(current.default_method),
        };
        if !(next.allow_squash || next.allow_merge_commit || next.allow_rebase) {
            return Err(RepositoryError::MergeMethodRequired);
        }
        let default_enabled = match next.default_method {
            RepositoryMergeMethod::Squash => next.allow_squash,
            RepositoryMergeMethod::MergeCommit => next.allow_merge_commit,
            RepositoryMergeMethod::Rebase => next.allow_rebase,
        };
        if !default_enabled {
            return Err(RepositoryError::DefaultMergeMethodDisabled);
        }
    }

    Ok(())
}

fn settings_patch_only_unarchives(patch: &RepositorySettingsPatch) -> bool {
    patch.is_archived == Some(false)
        && patch.name.is_none()
        && patch.description.is_none()
        && patch.visibility.is_none()
        && patch.default_branch.is_none()
        && patch.is_template.is_none()
        && patch.allow_forking.is_none()
        && patch.web_commit_signoff_required.is_none()
        && patch.features.is_none()
        && patch.merge.is_none()
}

fn changed_settings_fields(
    before: &RepositorySettings,
    patch: &RepositorySettingsPatch,
) -> Vec<String> {
    let mut fields = Vec::new();
    if patch
        .name
        .as_deref()
        .is_some_and(|value| value.trim() != before.name)
    {
        fields.push("name".to_owned());
    }
    if patch
        .description
        .as_ref()
        .is_some_and(|value| value != &before.description)
    {
        fields.push("description".to_owned());
    }
    if patch
        .visibility
        .as_ref()
        .is_some_and(|value| value != &before.visibility)
    {
        fields.push("visibility".to_owned());
    }
    if patch
        .default_branch
        .as_deref()
        .is_some_and(|value| value.trim() != before.default_branch)
    {
        fields.push("default_branch".to_owned());
    }
    if patch
        .is_template
        .is_some_and(|value| value != before.is_template)
    {
        fields.push("is_template".to_owned());
    }
    if patch
        .allow_forking
        .is_some_and(|value| value != before.allow_forking)
    {
        fields.push("allow_forking".to_owned());
    }
    if patch
        .web_commit_signoff_required
        .is_some_and(|value| value != before.web_commit_signoff_required)
    {
        fields.push("web_commit_signoff_required".to_owned());
    }
    if let Some(features) = &patch.features {
        if features
            .issues_enabled
            .is_some_and(|value| value != before.features.issues_enabled)
        {
            fields.push("features.issues_enabled".to_owned());
        }
        if features
            .projects_enabled
            .is_some_and(|value| value != before.features.projects_enabled)
        {
            fields.push("features.projects_enabled".to_owned());
        }
        if features
            .wiki_enabled
            .is_some_and(|value| value != before.features.wiki_enabled)
        {
            fields.push("features.wiki_enabled".to_owned());
        }
    }
    if let Some(merge) = &patch.merge {
        if merge
            .allow_squash
            .is_some_and(|value| value != before.merge.allow_squash)
        {
            fields.push("merge.allow_squash".to_owned());
        }
        if merge
            .allow_merge_commit
            .is_some_and(|value| value != before.merge.allow_merge_commit)
        {
            fields.push("merge.allow_merge_commit".to_owned());
        }
        if merge
            .allow_rebase
            .is_some_and(|value| value != before.merge.allow_rebase)
        {
            fields.push("merge.allow_rebase".to_owned());
        }
        if merge
            .default_method
            .as_ref()
            .is_some_and(|value| value != &before.merge.default_method)
        {
            fields.push("merge.default_method".to_owned());
        }
    }
    if patch
        .is_archived
        .is_some_and(|value| value != before.danger.is_archived)
    {
        fields.push("is_archived".to_owned());
    }
    fields
}

async fn require_access_admin(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<(), RepositoryError> {
    if can_admin_repository(pool, repository, actor_user_id).await? {
        Ok(())
    } else {
        Err(RepositoryError::PermissionDenied)
    }
}

fn validate_grant_role(role: RepositoryRole) -> Result<(), RepositoryError> {
    match role {
        RepositoryRole::Owner => Err(RepositoryError::InvalidAccessRole(
            RepositoryRole::Owner.as_str().to_owned(),
        )),
        RepositoryRole::Admin
        | RepositoryRole::Maintain
        | RepositoryRole::Write
        | RepositoryRole::Triage
        | RepositoryRole::Read => Ok(()),
    }
}

fn normalize_access_target(value: &str) -> Result<String, RepositoryError> {
    let target = value.trim();
    if target.is_empty() {
        return Err(RepositoryError::AccessTargetNotFound);
    }
    Ok(target.chars().take(254).collect::<String>())
}

async fn ensure_admin_path_remains(
    pool: &PgPool,
    repository_id: Uuid,
    excluded_user_id: Option<Uuid>,
    excluded_team_id: Option<Uuid>,
) -> Result<(), RepositoryError> {
    let direct_admin_paths = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)::bigint
        FROM repository_permissions
        WHERE repository_id = $1
          AND role IN ('owner', 'admin')
          AND ($2::uuid IS NULL OR user_id <> $2)
        "#,
    )
    .bind(repository_id)
    .bind(excluded_user_id)
    .fetch_one(pool)
    .await?;
    let team_admin_paths = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)::bigint
        FROM repository_team_permissions
        WHERE repository_id = $1
          AND role = 'admin'
          AND source = 'team'
          AND ($2::uuid IS NULL OR team_id <> $2)
        "#,
    )
    .bind(repository_id)
    .bind(excluded_team_id)
    .fetch_one(pool)
    .await?;

    if direct_admin_paths + team_admin_paths > 0 {
        Ok(())
    } else {
        Err(RepositoryError::LastAdminAccess)
    }
}

async fn repository_access_settings_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<Option<RepositoryAccessSettings>, RepositoryError> {
    let Some(row) = sqlx::query(
        r#"
        SELECT repositories.id,
               COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug) AS owner_login,
               repositories.name,
               repositories.visibility,
               repository_permissions.role AS viewer_permission
        FROM repositories
        LEFT JOIN users owner_user
          ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations
          ON organizations.id = repositories.owner_organization_id
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = repositories.id
         AND repository_permissions.user_id = $2
        WHERE repositories.id = $1
        "#,
    )
    .bind(repository.id)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let people = repository_access_people(pool, repository).await?;
    let teams = repository_access_teams(pool, repository).await?;
    let invitations = repository_access_invitations(pool, repository.id).await?;
    let invite_targets = repository_access_invite_targets(pool, repository).await?;
    let audit_events = repository_settings_audit_events(pool, repository.id).await?;
    let viewer_permission: Option<String> = row.try_get("viewer_permission")?;

    Ok(Some(RepositoryAccessSettings {
        id: row.try_get("id")?,
        owner_login: row.try_get("owner_login")?,
        name: row.try_get("name")?,
        visibility: RepositoryVisibility::try_from(
            row.try_get::<String, _>("visibility")?.as_str(),
        )?,
        viewer_permission: viewer_permission.unwrap_or_else(|| {
            if repository.owner_user_id == Some(actor_user_id) {
                RepositoryRole::Owner.as_str().to_owned()
            } else {
                RepositoryRole::Admin.as_str().to_owned()
            }
        }),
        roles: repository_access_role_definitions(),
        people,
        teams,
        invitations,
        invite_targets,
        audit_events,
    }))
}

fn repository_access_role_definitions() -> Vec<RepositoryAccessRoleDefinition> {
    vec![
        RepositoryAccessRoleDefinition {
            role: RepositoryRole::Read,
            label: "Read".to_owned(),
            description: "Can view and clone the repository.".to_owned(),
            rank: 10,
        },
        RepositoryAccessRoleDefinition {
            role: RepositoryRole::Triage,
            label: "Triage".to_owned(),
            description: "Can manage issues and pull requests without write access.".to_owned(),
            rank: 20,
        },
        RepositoryAccessRoleDefinition {
            role: RepositoryRole::Write,
            label: "Write".to_owned(),
            description: "Can push branches and manage collaboration content.".to_owned(),
            rank: 30,
        },
        RepositoryAccessRoleDefinition {
            role: RepositoryRole::Maintain,
            label: "Maintain".to_owned(),
            description: "Can manage repository settings short of destructive ownership controls."
                .to_owned(),
            rank: 40,
        },
        RepositoryAccessRoleDefinition {
            role: RepositoryRole::Admin,
            label: "Admin".to_owned(),
            description: "Can administer repository settings and access.".to_owned(),
            rank: 50,
        },
    ]
}

async fn repository_access_people(
    pool: &PgPool,
    repository: &Repository,
) -> Result<Vec<RepositoryAccessPerson>, RepositoryError> {
    let mut people = Vec::new();
    let direct_rows = sqlx::query(
        r#"
        SELECT users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.display_name,
               users.email,
               users.avatar_url,
               repository_permissions.role,
               repository_permissions.source
        FROM repository_permissions
        JOIN users ON users.id = repository_permissions.user_id
        WHERE repository_permissions.repository_id = $1
        ORDER BY
            CASE repository_permissions.role
                WHEN 'owner' THEN 0
                WHEN 'admin' THEN 1
                WHEN 'maintain' THEN 2
                WHEN 'write' THEN 3
                WHEN 'triage' THEN 4
                ELSE 5
            END,
            lower(COALESCE(NULLIF(users.username, ''), users.email))
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;

    for row in direct_rows {
        let source: String = row.try_get("source")?;
        let role = RepositoryRole::try_from(row.try_get::<String, _>("role")?.as_str())
            .map_err(|error| RepositoryError::Sqlx(sqlx::Error::Protocol(error.to_string())))?;
        let source_text = match source.as_str() {
            "owner" => "Repository owner access".to_owned(),
            "team" => "Granted by team membership".to_owned(),
            "organization" => "Inherited from organization membership".to_owned(),
            _ => "Direct collaborator access".to_owned(),
        };
        people.push(RepositoryAccessPerson {
            user_id: row.try_get("id")?,
            login: row.try_get("login")?,
            display_name: row.try_get("display_name")?,
            email: row.try_get("email")?,
            avatar_url: row.try_get("avatar_url")?,
            role,
            source: source.clone(),
            source_text,
            team_slug: None,
            team_name: None,
            can_edit: source == "direct",
            can_remove: source == "direct",
        });
    }

    if let Some(organization_id) = repository.owner_organization_id {
        let base_role = organization_base_repository_role(pool, organization_id).await?;
        if let Some(base_role) = base_role {
            let member_rows = sqlx::query(
                r#"
                SELECT users.id,
                       COALESCE(NULLIF(users.username, ''), users.email) AS login,
                       users.display_name,
                       users.email,
                       users.avatar_url
                FROM organization_memberships
                JOIN users ON users.id = organization_memberships.user_id
                WHERE organization_memberships.organization_id = $1
                ORDER BY lower(COALESCE(NULLIF(users.username, ''), users.email))
                "#,
            )
            .bind(organization_id)
            .fetch_all(pool)
            .await?;

            for row in member_rows {
                let user_id: Uuid = row.try_get("id")?;
                if people.iter().any(|person| person.user_id == user_id) {
                    continue;
                }
                people.push(RepositoryAccessPerson {
                    user_id,
                    login: row.try_get("login")?,
                    display_name: row.try_get("display_name")?,
                    email: row.try_get("email")?,
                    avatar_url: row.try_get("avatar_url")?,
                    role: base_role,
                    source: "organization".to_owned(),
                    source_text: "Inherited from organization base permissions".to_owned(),
                    team_slug: None,
                    team_name: None,
                    can_edit: false,
                    can_remove: false,
                });
            }
        }

        let team_rows = sqlx::query(
            r#"
            SELECT users.id,
                   COALESCE(NULLIF(users.username, ''), users.email) AS login,
                   users.display_name,
                   users.email,
                   users.avatar_url,
                   repository_team_permissions.role,
                   teams.slug,
                   teams.name
            FROM repository_team_permissions
            JOIN teams ON teams.id = repository_team_permissions.team_id
            JOIN team_memberships ON team_memberships.team_id = teams.id
            JOIN users ON users.id = team_memberships.user_id
            WHERE repository_team_permissions.repository_id = $1
              AND teams.organization_id = $2
            ORDER BY lower(teams.slug), lower(COALESCE(NULLIF(users.username, ''), users.email))
            "#,
        )
        .bind(repository.id)
        .bind(organization_id)
        .fetch_all(pool)
        .await?;

        for row in team_rows {
            let user_id: Uuid = row.try_get("id")?;
            if people
                .iter()
                .any(|person| person.user_id == user_id && person.source != "team")
            {
                continue;
            }
            let role = RepositoryRole::try_from(row.try_get::<String, _>("role")?.as_str())
                .map_err(|error| RepositoryError::Sqlx(sqlx::Error::Protocol(error.to_string())))?;
            let slug: String = row.try_get("slug")?;
            let name: String = row.try_get("name")?;
            people.push(RepositoryAccessPerson {
                user_id,
                login: row.try_get("login")?,
                display_name: row.try_get("display_name")?,
                email: row.try_get("email")?,
                avatar_url: row.try_get("avatar_url")?,
                role,
                source: "team".to_owned(),
                source_text: format!("Inherited from team {slug}"),
                team_slug: Some(slug),
                team_name: Some(name),
                can_edit: false,
                can_remove: false,
            });
        }
    }

    Ok(people)
}

async fn repository_access_teams(
    pool: &PgPool,
    repository: &Repository,
) -> Result<Vec<RepositoryAccessTeam>, RepositoryError> {
    let Some(organization_id) = repository.owner_organization_id else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query(
        r#"
        SELECT teams.id,
               teams.slug,
               teams.name,
               repository_team_permissions.role,
               repository_team_permissions.source,
               COUNT(team_memberships.user_id)::bigint AS member_count
        FROM repository_team_permissions
        JOIN teams ON teams.id = repository_team_permissions.team_id
        LEFT JOIN team_memberships ON team_memberships.team_id = teams.id
        WHERE repository_team_permissions.repository_id = $1
          AND teams.organization_id = $2
        GROUP BY teams.id, teams.slug, teams.name, repository_team_permissions.role, repository_team_permissions.source
        ORDER BY lower(teams.slug)
        "#,
    )
    .bind(repository.id)
    .bind(organization_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let source: String = row.try_get("source")?;
            let slug: String = row.try_get("slug")?;
            Ok(RepositoryAccessTeam {
                team_id: row.try_get("id")?,
                slug: slug.clone(),
                name: row.try_get("name")?,
                role: RepositoryRole::try_from(row.try_get::<String, _>("role")?.as_str())
                    .map_err(|error| {
                        RepositoryError::Sqlx(sqlx::Error::Protocol(error.to_string()))
                    })?,
                source: source.clone(),
                source_text: if source == "inherited" {
                    "Inherited from organization base permissions".to_owned()
                } else {
                    "Direct team access".to_owned()
                },
                member_count: row.try_get("member_count")?,
                href: format!("/orgs/{}/teams/{slug}", repository.owner_login),
                can_edit: source == "team",
                can_remove: source == "team",
            })
        })
        .collect()
}

async fn repository_access_invitations(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<RepositoryInvitation>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT repository_invitations.id,
               repository_invitations.invited_user_id,
               repository_invitations.invited_email,
               COALESCE(NULLIF(users.username, ''), users.email) AS invited_login,
               repository_invitations.role,
               repository_invitations.status,
               repository_invitations.email_delivery_status,
               repository_invitations.invited_by_user_id,
               repository_invitations.expires_at,
               repository_invitations.created_at
        FROM repository_invitations
        LEFT JOIN users ON users.id = repository_invitations.invited_user_id
        WHERE repository_invitations.repository_id = $1
          AND repository_invitations.status = 'pending'
        ORDER BY repository_invitations.created_at DESC
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(RepositoryInvitation {
                id: row.try_get("id")?,
                invited_user_id: row.try_get("invited_user_id")?,
                invited_email: row.try_get("invited_email")?,
                invited_login: row.try_get("invited_login")?,
                role: RepositoryRole::try_from(row.try_get::<String, _>("role")?.as_str())
                    .map_err(|error| {
                        RepositoryError::Sqlx(sqlx::Error::Protocol(error.to_string()))
                    })?,
                status: row.try_get("status")?,
                email_delivery_status: row.try_get("email_delivery_status")?,
                invited_by_user_id: row.try_get("invited_by_user_id")?,
                expires_at: row.try_get("expires_at")?,
                created_at: row.try_get("created_at")?,
                can_cancel: true,
            })
        })
        .collect()
}

async fn repository_access_invite_targets(
    pool: &PgPool,
    repository: &Repository,
) -> Result<RepositoryInviteTargets, RepositoryError> {
    let user_rows = sqlx::query(
        r#"
        SELECT users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.display_name,
               users.email,
               users.avatar_url
        FROM users
        WHERE NOT EXISTS (
            SELECT 1 FROM repository_permissions
            WHERE repository_permissions.repository_id = $1
              AND repository_permissions.user_id = users.id
        )
        ORDER BY lower(COALESCE(NULLIF(users.username, ''), users.email))
        LIMIT 10
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;

    let users = user_rows
        .into_iter()
        .map(|row| RepositoryInviteUserTarget {
            user_id: row.get("id"),
            login: row.get("login"),
            display_name: row.get("display_name"),
            email: row.get("email"),
            avatar_url: row.get("avatar_url"),
        })
        .collect();

    let teams = if let Some(organization_id) = repository.owner_organization_id {
        let rows = sqlx::query(
            r#"
            SELECT teams.id,
                   teams.slug,
                   teams.name,
                   COUNT(team_memberships.user_id)::bigint AS member_count
            FROM teams
            LEFT JOIN team_memberships ON team_memberships.team_id = teams.id
            WHERE teams.organization_id = $1
              AND NOT EXISTS (
                  SELECT 1 FROM repository_team_permissions
                  WHERE repository_team_permissions.repository_id = $2
                    AND repository_team_permissions.team_id = teams.id
              )
            GROUP BY teams.id, teams.slug, teams.name
            ORDER BY lower(teams.slug)
            LIMIT 10
            "#,
        )
        .bind(organization_id)
        .bind(repository.id)
        .fetch_all(pool)
        .await?;

        rows.into_iter()
            .map(|row| RepositoryInviteTeamTarget {
                team_id: row.get("id"),
                slug: row.get("slug"),
                name: row.get("name"),
                member_count: row.get("member_count"),
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(RepositoryInviteTargets { users, teams })
}

async fn find_user_for_access_target(
    pool: &PgPool,
    target: &str,
) -> Result<Option<RepositoryInviteUserTarget>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT id,
               COALESCE(NULLIF(username, ''), email) AS login,
               display_name,
               email,
               avatar_url
        FROM users
        WHERE lower(email) = lower($1)
           OR lower(username) = lower($1)
        LIMIT 1
        "#,
    )
    .bind(target)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| RepositoryInviteUserTarget {
        user_id: row.get("id"),
        login: row.get("login"),
        display_name: row.get("display_name"),
        email: row.get("email"),
        avatar_url: row.get("avatar_url"),
    }))
}

async fn find_team_for_access_target(
    pool: &PgPool,
    organization_id: Uuid,
    team_slug: &str,
) -> Result<Option<RepositoryInviteTeamTarget>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT teams.id,
               teams.slug,
               teams.name,
               COUNT(team_memberships.user_id)::bigint AS member_count
        FROM teams
        LEFT JOIN team_memberships ON team_memberships.team_id = teams.id
        WHERE teams.organization_id = $1
          AND lower(teams.slug) = lower($2)
        GROUP BY teams.id, teams.slug, teams.name
        "#,
    )
    .bind(organization_id)
    .bind(team_slug.trim())
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| RepositoryInviteTeamTarget {
        team_id: row.get("id"),
        slug: row.get("slug"),
        name: row.get("name"),
        member_count: row.get("member_count"),
    }))
}

async fn upsert_team_member_repository_permissions(
    pool: &PgPool,
    repository_id: Uuid,
    team_id: Uuid,
    role: RepositoryRole,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO repository_permissions (repository_id, user_id, role, source)
        SELECT $1, team_memberships.user_id, $3, 'team'
        FROM team_memberships
        WHERE team_memberships.team_id = $2
        ON CONFLICT (repository_id, user_id)
        DO UPDATE SET role = EXCLUDED.role, source = EXCLUDED.source
        WHERE repository_permissions.source = 'team'
        "#,
    )
    .bind(repository_id)
    .bind(team_id)
    .bind(role.as_str())
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_repository_access_audit_event(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
    event_type: &str,
    changed_fields: Vec<String>,
    before_state: serde_json::Value,
    after_state: serde_json::Value,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO repository_settings_audit_events (
            repository_id, actor_user_id, event_type, changed_fields, before_state, after_state
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(repository_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(changed_fields)
    .bind(before_state)
    .bind(after_state)
    .execute(pool)
    .await?;
    Ok(())
}

async fn repository_settings_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<Option<RepositorySettings>, RepositoryError> {
    let Some(row) = sqlx::query(
        r#"
        SELECT repositories.id,
               COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug) AS owner_login,
               repositories.name,
               repositories.description,
               repositories.visibility,
               repositories.default_branch,
               repositories.is_archived,
               repositories.is_template,
               repositories.issues_enabled,
               repositories.projects_enabled,
               repositories.wiki_enabled,
               repositories.allow_forking,
               repositories.web_commit_signoff_required,
               repositories.updated_at,
               repository_permissions.role AS viewer_permission
        FROM repositories
        LEFT JOIN users owner_user
          ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations
          ON organizations.id = repositories.owner_organization_id
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = repositories.id
         AND repository_permissions.user_id = $2
        WHERE repositories.id = $1
        "#,
    )
    .bind(repository.id)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let merge = repository_merge_settings_for_repository(pool, repository.id).await?;
    let branches = repository_branch_names(pool, repository.id).await?;
    let audit_events = repository_settings_audit_events(pool, repository.id).await?;
    let policy_locks = repository_policy_locks(pool, repository, actor_user_id).await?;
    let viewer_permission = repository_permission_for_user(pool, repository.id, actor_user_id)
        .await?
        .map(|permission| permission.role.as_str().to_owned());

    Ok(Some(RepositorySettings {
        id: row.try_get("id")?,
        owner_login: row.try_get("owner_login")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        visibility: RepositoryVisibility::try_from(
            row.try_get::<String, _>("visibility")?.as_str(),
        )?,
        default_branch: row.try_get("default_branch")?,
        is_template: row.try_get("is_template")?,
        allow_forking: row.try_get("allow_forking")?,
        web_commit_signoff_required: row.try_get("web_commit_signoff_required")?,
        features: RepositoryFeatureSettings {
            issues_enabled: row.try_get("issues_enabled")?,
            projects_enabled: row.try_get("projects_enabled")?,
            wiki_enabled: row.try_get("wiki_enabled")?,
        },
        merge,
        danger: RepositoryDangerState {
            is_archived: row.try_get("is_archived")?,
            can_archive: !row.try_get::<bool, _>("is_archived")?,
            can_unarchive: row.try_get("is_archived")?,
            delete_supported: false,
            transfer_supported: false,
        },
        branches,
        viewer_permission: viewer_permission.unwrap_or_else(|| {
            if repository.owner_user_id == Some(actor_user_id) {
                RepositoryRole::Owner.as_str().to_owned()
            } else {
                RepositoryRole::Admin.as_str().to_owned()
            }
        }),
        updated_at: row.try_get("updated_at")?,
        audit_events,
        policy_locks,
    }))
}

async fn repository_policy_locks(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<Vec<RepositoryPolicyLock>, RepositoryError> {
    let Some(organization_id) = repository.owner_organization_id else {
        return Ok(Vec::new());
    };
    if organization_actor_is_owner_or_admin(pool, organization_id, actor_user_id).await? {
        return Ok(Vec::new());
    }
    let Some(row) = sqlx::query(
        r#"
        SELECT organizations.slug,
               COALESCE(organization_policy_settings.members_can_fork_private_repositories, true) AS members_can_fork_private_repositories,
               COALESCE(organization_policy_settings.members_can_change_repository_visibility, false) AS members_can_change_repository_visibility,
               COALESCE(organization_policy_settings.members_can_delete_repositories, false) AS members_can_delete_repositories,
               COALESCE(organization_policy_settings.members_can_transfer_repositories, false) AS members_can_transfer_repositories
        FROM organizations
        LEFT JOIN organization_policy_settings
          ON organization_policy_settings.organization_id = organizations.id
        WHERE organizations.id = $1
        "#,
    )
    .bind(organization_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(Vec::new());
    };

    let slug: String = row.try_get("slug")?;
    let settings_href = format!("/organizations/{slug}/settings/member_privileges");
    let mut locks = Vec::new();
    if !row.try_get::<bool, _>("members_can_change_repository_visibility")? {
        locks.push(RepositoryPolicyLock {
            field: "visibility".to_owned(),
            reason: "Organization policy prevents members from changing repository visibility."
                .to_owned(),
            settings_href: settings_href.clone(),
        });
    }
    if repository.visibility == RepositoryVisibility::Private
        && !row.try_get::<bool, _>("members_can_fork_private_repositories")?
    {
        locks.push(RepositoryPolicyLock {
            field: "allowForking".to_owned(),
            reason: "Organization policy prevents private repository forking.".to_owned(),
            settings_href: settings_href.clone(),
        });
    }
    if !row.try_get::<bool, _>("members_can_delete_repositories")? {
        locks.push(RepositoryPolicyLock {
            field: "deleteRepository".to_owned(),
            reason: "Organization policy prevents members from deleting repositories.".to_owned(),
            settings_href: settings_href.clone(),
        });
    }
    if !row.try_get::<bool, _>("members_can_transfer_repositories")? {
        locks.push(RepositoryPolicyLock {
            field: "transferRepository".to_owned(),
            reason: "Organization policy prevents members from transferring repositories."
                .to_owned(),
            settings_href,
        });
    }
    Ok(locks)
}

async fn organization_actor_is_owner_or_admin(
    pool: &PgPool,
    organization_id: Uuid,
    actor_user_id: Uuid,
) -> Result<bool, RepositoryError> {
    let role = sqlx::query_scalar::<_, String>(
        r#"
        SELECT role
        FROM organization_memberships
        WHERE organization_id = $1 AND user_id = $2
        "#,
    )
    .bind(organization_id)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?;
    Ok(role
        .as_deref()
        .is_some_and(|role| matches!(role, "owner" | "admin")))
}

async fn repository_branch_settings_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<Option<RepositoryBranchSettings>, RepositoryError> {
    let Some(row) = sqlx::query(
        r#"
        SELECT repositories.id,
               COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug) AS owner_login,
               repositories.name,
               repositories.visibility,
               repositories.default_branch,
               repository_permissions.role AS viewer_permission
        FROM repositories
        LEFT JOIN users owner_user
          ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations
          ON organizations.id = repositories.owner_organization_id
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = repositories.id
         AND repository_permissions.user_id = $2
        WHERE repositories.id = $1
        "#,
    )
    .bind(repository.id)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let viewer_permission = repository_permission_for_user(pool, repository.id, actor_user_id)
        .await?
        .map(|permission| permission.role.as_str().to_owned());
    let viewer_permission = viewer_permission.unwrap_or_else(|| {
        if repository.owner_user_id == Some(actor_user_id) {
            RepositoryRole::Owner.as_str().to_owned()
        } else {
            RepositoryRole::Read.as_str().to_owned()
        }
    });
    let can_edit = can_admin_repository(pool, repository, actor_user_id).await?;
    let mut refs = repository_branch_ref_summaries(pool, repository.id).await?;
    let rules = repository_branch_rules(pool, repository.id, &refs, can_edit).await?;
    let rulesets = repository_rulesets(pool, repository.id, &refs, can_edit).await?;
    for branch in &mut refs {
        branch.matching_rule_count = rules
            .iter()
            .filter(|rule| rule.enforcement != BranchPolicyEnforcement::Disabled)
            .filter(|rule| branch_pattern_matches(&rule.pattern, &branch.name))
            .count() as i64;
        branch.matching_ruleset_count = rulesets
            .iter()
            .filter(|ruleset| ruleset.enforcement != BranchPolicyEnforcement::Disabled)
            .filter(|ruleset| {
                ruleset
                    .patterns
                    .iter()
                    .any(|pattern| branch_pattern_matches(pattern, &branch.name))
            })
            .count() as i64;
        branch.protected = branch.matching_rule_count + branch.matching_ruleset_count > 0;
    }
    let status_check_suggestions = repository_status_check_suggestions(pool, repository.id).await?;
    let audit_events = repository_settings_audit_events(pool, repository.id).await?;
    let default_branch: String = row.try_get("default_branch")?;
    let default_rule_count = rules
        .iter()
        .filter(|rule| rule.enforcement != BranchPolicyEnforcement::Disabled)
        .filter(|rule| branch_pattern_matches(&rule.pattern, &default_branch))
        .count() as i64;
    let default_ruleset_count = rulesets
        .iter()
        .filter(|ruleset| ruleset.enforcement != BranchPolicyEnforcement::Disabled)
        .filter(|ruleset| {
            ruleset
                .patterns
                .iter()
                .any(|pattern| branch_pattern_matches(pattern, &default_branch))
        })
        .count() as i64;
    let owner_login: String = row.try_get("owner_login")?;
    let repo_name: String = row.try_get("name")?;

    Ok(Some(RepositoryBranchSettings {
        id: row.try_get("id")?,
        owner_login: owner_login.clone(),
        name: repo_name.clone(),
        visibility: RepositoryVisibility::try_from(
            row.try_get::<String, _>("visibility")?.as_str(),
        )?,
        default_branch: default_branch.clone(),
        default_branch_summary: RepositoryDefaultBranchSummary {
            name: default_branch.clone(),
            protected: default_rule_count + default_ruleset_count > 0,
            matching_rule_count: default_rule_count,
            matching_ruleset_count: default_ruleset_count,
            href: format!("/{owner_login}/{repo_name}/tree/{default_branch}"),
        },
        viewer_permission,
        can_edit,
        refs,
        rules,
        rulesets,
        status_check_suggestions,
        audit_events,
    }))
}

async fn repository_branch_ref_summaries(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<RepositoryBranchRefSummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT name, updated_at
        FROM repository_git_refs
        WHERE repository_id = $1 AND kind = 'branch'
        ORDER BY lower(name)
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let name: String = row.get("name");
            RepositoryBranchRefSummary {
                name: name.strip_prefix("refs/heads/").unwrap_or(&name).to_owned(),
                protected: false,
                matching_rule_count: 0,
                matching_ruleset_count: 0,
                updated_at: row.get("updated_at"),
            }
        })
        .collect())
}

async fn repository_branch_rules(
    pool: &PgPool,
    repository_id: Uuid,
    refs: &[RepositoryBranchRefSummary],
    can_edit: bool,
) -> Result<Vec<RepositoryBranchRule>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT id, pattern, description, enforcement, required_approving_review_count,
               requires_up_to_date_branch, requires_conversation_resolution,
               requires_signed_commits, requires_linear_history, requires_merge_queue,
               requires_deployments, required_deployment_environments, locked, restricts_pushes,
               allows_force_pushes, allows_deletions, bypass_actors, created_at, updated_at
        FROM repository_branch_protection_rules
        WHERE repository_id = $1
        ORDER BY lower(pattern), created_at
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    let mut rules = Vec::new();
    for row in rows {
        let rule_id: Uuid = row.get("id");
        let pattern: String = row.get("pattern");
        let required_status_checks = sqlx::query_scalar::<_, String>(
            r#"
            SELECT context
            FROM repository_required_status_checks
            WHERE branch_protection_rule_id = $1
            ORDER BY lower(context)
            "#,
        )
        .bind(rule_id)
        .fetch_all(pool)
        .await?;
        let matching_branches = matching_branches(refs, std::slice::from_ref(&pattern));
        rules.push(RepositoryBranchRule {
            id: rule_id,
            pattern,
            description: row.get("description"),
            enforcement: BranchPolicyEnforcement::try_from(
                row.get::<String, _>("enforcement").as_str(),
            )?,
            matching_branch_count: matching_branches.len() as i64,
            matching_branches,
            requirements: BranchPolicyRequirements {
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
            },
            bypass_actors: decode_bypass_actors(row.get("bypass_actors"))?,
            can_edit,
            can_delete: can_edit,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });
    }
    Ok(rules)
}

async fn repository_rulesets(
    pool: &PgPool,
    repository_id: Uuid,
    refs: &[RepositoryBranchRefSummary],
    can_edit: bool,
) -> Result<Vec<RepositoryRuleset>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, target, enforcement, patterns, required_approving_review_count,
               requires_up_to_date_branch, required_status_checks,
               requires_conversation_resolution, requires_signed_commits, requires_linear_history,
               requires_merge_queue, requires_deployments, required_deployment_environments,
               locked, restricts_pushes, allows_force_pushes, allows_deletions,
               bypass_actors, created_at, updated_at
        FROM repository_rulesets
        WHERE repository_id = $1
        ORDER BY lower(name), created_at
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let patterns: Vec<String> = row.get("patterns");
            let matching_branches = matching_branches(refs, &patterns);
            Ok(RepositoryRuleset {
                id: row.get("id"),
                name: row.get("name"),
                target: row.get("target"),
                enforcement: BranchPolicyEnforcement::try_from(
                    row.get::<String, _>("enforcement").as_str(),
                )?,
                patterns,
                matching_branch_count: matching_branches.len() as i64,
                matching_branches,
                requirements: BranchPolicyRequirements {
                    required_approving_review_count: row.get("required_approving_review_count"),
                    requires_up_to_date_branch: row.get("requires_up_to_date_branch"),
                    required_status_checks: row.get("required_status_checks"),
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
                },
                bypass_actors: decode_bypass_actors(row.get("bypass_actors"))?,
                can_edit,
                can_delete: can_edit,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        })
        .collect()
}

async fn repository_status_check_suggestions(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<String>, RepositoryError> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT repository_required_status_checks.context
        FROM repository_required_status_checks
        JOIN repository_branch_protection_rules
          ON repository_branch_protection_rules.id = repository_required_status_checks.branch_protection_rule_id
        WHERE repository_branch_protection_rules.repository_id = $1
          AND length(trim(repository_required_status_checks.context)) > 0
        ORDER BY repository_required_status_checks.context
        LIMIT 20
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    Ok(rows)
}

fn matching_branches(refs: &[RepositoryBranchRefSummary], patterns: &[String]) -> Vec<String> {
    refs.iter()
        .filter(|branch| {
            patterns
                .iter()
                .any(|pattern| branch_pattern_matches(pattern, &branch.name))
        })
        .map(|branch| branch.name.clone())
        .collect()
}

fn decode_bypass_actors(value: serde_json::Value) -> Result<Vec<BypassActor>, RepositoryError> {
    serde_json::from_value(value)
        .map_err(|error| RepositoryError::InvalidBranchPolicy(error.to_string()))
}

struct NormalizedBranchRuleMutation {
    pattern: String,
    description: Option<String>,
    enforcement: BranchPolicyEnforcement,
    requirements: BranchPolicyRequirements,
    bypass_actors: Vec<BypassActor>,
}

struct NormalizedRulesetMutation {
    name: String,
    enforcement: BranchPolicyEnforcement,
    patterns: Vec<String>,
    requirements: BranchPolicyRequirements,
    bypass_actors: Vec<BypassActor>,
}

fn normalize_branch_rule_mutation(
    mutation: RepositoryBranchRuleMutation,
) -> Result<NormalizedBranchRuleMutation, RepositoryError> {
    let pattern = normalize_branch_pattern(&mutation.pattern)?;
    let requirements = normalize_branch_requirements(mutation.requirements)?;
    if pattern == "main" && requirements.allows_deletions {
        return Err(RepositoryError::InvalidBranchPolicy(
            "The default branch cannot allow deletion in this phase.".to_owned(),
        ));
    }
    Ok(NormalizedBranchRuleMutation {
        pattern,
        description: mutation.description.and_then(|value| {
            let value = value.trim().chars().take(240).collect::<String>();
            (!value.is_empty()).then_some(value)
        }),
        enforcement: mutation
            .enforcement
            .unwrap_or(BranchPolicyEnforcement::Active),
        requirements,
        bypass_actors: normalize_bypass_actors(mutation.bypass_actors.unwrap_or_default())?,
    })
}

fn normalize_ruleset_mutation(
    mutation: RepositoryRulesetMutation,
) -> Result<NormalizedRulesetMutation, RepositoryError> {
    let name = mutation.name.trim().chars().take(120).collect::<String>();
    if name.is_empty() {
        return Err(RepositoryError::InvalidBranchPolicy(
            "Ruleset name is required.".to_owned(),
        ));
    }
    let mut patterns = Vec::new();
    let mut seen = BTreeSet::new();
    for pattern in mutation.patterns {
        let pattern = normalize_branch_pattern(&pattern)?;
        if seen.insert(pattern.to_lowercase()) {
            patterns.push(pattern);
        }
    }
    if patterns.is_empty() {
        return Err(RepositoryError::InvalidBranchPolicy(
            "At least one branch pattern is required.".to_owned(),
        ));
    }
    Ok(NormalizedRulesetMutation {
        name,
        enforcement: mutation
            .enforcement
            .unwrap_or(BranchPolicyEnforcement::Active),
        patterns,
        requirements: normalize_branch_requirements(mutation.requirements)?,
        bypass_actors: normalize_bypass_actors(mutation.bypass_actors.unwrap_or_default())?,
    })
}

fn normalize_branch_requirements(
    patch: BranchPolicyRequirementsPatch,
) -> Result<BranchPolicyRequirements, RepositoryError> {
    let required_approving_review_count = patch.required_approving_review_count.unwrap_or(0);
    if required_approving_review_count < 0 {
        return Err(RepositoryError::InvalidBranchPolicy(
            "Required approving review count cannot be negative.".to_owned(),
        ));
    }
    Ok(BranchPolicyRequirements {
        required_approving_review_count,
        requires_up_to_date_branch: patch.requires_up_to_date_branch.unwrap_or(false),
        required_status_checks: normalize_nonempty_strings(
            patch.required_status_checks.unwrap_or_default(),
            "status check context",
        )?,
        requires_conversation_resolution: patch.requires_conversation_resolution.unwrap_or(false),
        requires_signed_commits: patch.requires_signed_commits.unwrap_or(false),
        requires_linear_history: patch.requires_linear_history.unwrap_or(false),
        requires_merge_queue: patch.requires_merge_queue.unwrap_or(false),
        requires_deployments: patch.requires_deployments.unwrap_or(false),
        required_deployment_environments: normalize_nonempty_strings(
            patch.required_deployment_environments.unwrap_or_default(),
            "deployment environment",
        )?,
        locked: patch.locked.unwrap_or(false),
        restricts_pushes: patch.restricts_pushes.unwrap_or(false),
        allows_force_pushes: patch.allows_force_pushes.unwrap_or(false),
        allows_deletions: patch.allows_deletions.unwrap_or(false),
    })
}

fn normalize_nonempty_strings(
    values: Vec<String>,
    label: &str,
) -> Result<Vec<String>, RepositoryError> {
    let mut normalized = Vec::new();
    let mut seen = BTreeSet::new();
    for value in values {
        let trimmed = value.trim().chars().take(120).collect::<String>();
        if trimmed.is_empty() {
            return Err(RepositoryError::InvalidBranchPolicy(format!(
                "{label} cannot be blank."
            )));
        }
        if seen.insert(trimmed.to_lowercase()) {
            normalized.push(trimmed);
        }
    }
    Ok(normalized)
}

fn normalize_branch_pattern(pattern: &str) -> Result<String, RepositoryError> {
    let pattern = pattern.trim().trim_start_matches("refs/heads/").to_owned();
    if pattern.is_empty() {
        return Err(RepositoryError::InvalidBranchPolicy(
            "Branch pattern is required.".to_owned(),
        ));
    }
    if pattern.len() > 160 || pattern.contains("..") || pattern.contains('\\') {
        return Err(RepositoryError::InvalidBranchPolicy(
            "Branch pattern contains unsupported characters.".to_owned(),
        ));
    }
    Ok(pattern)
}

fn normalize_bypass_actors(actors: Vec<BypassActor>) -> Result<Vec<BypassActor>, RepositoryError> {
    let mut normalized = Vec::new();
    let mut seen = BTreeSet::new();
    for actor in actors {
        let actor_type = actor.actor_type.trim().to_lowercase();
        if !matches!(actor_type.as_str(), "user" | "team" | "role") {
            return Err(RepositoryError::InvalidBranchPolicy(
                "Bypass actors must be users, teams, or roles.".to_owned(),
            ));
        }
        if seen.insert((actor_type.clone(), actor.actor_id)) {
            normalized.push(BypassActor {
                actor_type,
                actor_id: actor.actor_id,
                label: actor.label.trim().chars().take(120).collect(),
            });
        }
    }
    Ok(normalized)
}

async fn ensure_unique_branch_rule_pattern(
    pool: &PgPool,
    repository_id: Uuid,
    pattern: &str,
    excluding_rule_id: Option<Uuid>,
) -> Result<(), RepositoryError> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM repository_branch_protection_rules
            WHERE repository_id = $1
              AND lower(pattern) = lower($2)
              AND ($3::uuid IS NULL OR id <> $3)
        )
        "#,
    )
    .bind(repository_id)
    .bind(pattern)
    .bind(excluding_rule_id)
    .fetch_one(pool)
    .await?;
    if exists {
        Err(RepositoryError::BranchPolicyConflict)
    } else {
        Ok(())
    }
}

async fn ensure_unique_ruleset_name(
    pool: &PgPool,
    repository_id: Uuid,
    name: &str,
    excluding_ruleset_id: Option<Uuid>,
) -> Result<(), RepositoryError> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM repository_rulesets
            WHERE repository_id = $1
              AND lower(name) = lower($2)
              AND ($3::uuid IS NULL OR id <> $3)
        )
        "#,
    )
    .bind(repository_id)
    .bind(name)
    .bind(excluding_ruleset_id)
    .fetch_one(pool)
    .await?;
    if exists {
        Err(RepositoryError::BranchPolicyConflict)
    } else {
        Ok(())
    }
}

async fn branch_rule_exists(
    pool: &PgPool,
    repository_id: Uuid,
    rule_id: Uuid,
) -> Result<serde_json::Value, RepositoryError> {
    let row = sqlx::query(
        "SELECT id, pattern, enforcement FROM repository_branch_protection_rules WHERE repository_id = $1 AND id = $2",
    )
    .bind(repository_id)
    .bind(rule_id)
    .fetch_optional(pool)
    .await?
    .ok_or(RepositoryError::BranchPolicyNotFound)?;
    Ok(json!({
        "id": row.get::<Uuid, _>("id"),
        "pattern": row.get::<String, _>("pattern"),
        "enforcement": row.get::<String, _>("enforcement")
    }))
}

async fn replace_required_status_checks(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    rule_id: Uuid,
    contexts: &[String],
) -> Result<(), RepositoryError> {
    sqlx::query(
        "DELETE FROM repository_required_status_checks WHERE branch_protection_rule_id = $1",
    )
    .bind(rule_id)
    .execute(&mut **transaction)
    .await?;
    for context in contexts {
        sqlx::query(
            r#"
            INSERT INTO repository_required_status_checks (branch_protection_rule_id, context)
            VALUES ($1, $2)
            "#,
        )
        .bind(rule_id)
        .bind(context)
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}

async fn insert_repository_settings_audit_event_tx(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    repository_id: Uuid,
    actor_user_id: Uuid,
    event_type: &str,
    changed_fields: Vec<String>,
    before_state: serde_json::Value,
    after_state: serde_json::Value,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO repository_settings_audit_events (
            repository_id, actor_user_id, event_type, changed_fields, before_state, after_state
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(repository_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(changed_fields)
    .bind(before_state)
    .bind(after_state)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

async fn repository_merge_settings_for_repository(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<RepositoryMergeSettings, RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO repository_merge_settings (repository_id)
        VALUES ($1)
        ON CONFLICT (repository_id) DO NOTHING
        "#,
    )
    .bind(repository_id)
    .execute(pool)
    .await?;

    let row = sqlx::query(
        r#"
        SELECT allow_squash, allow_merge_commit, allow_rebase, default_method
        FROM repository_merge_settings
        WHERE repository_id = $1
        "#,
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;

    Ok(RepositoryMergeSettings {
        allow_squash: row.try_get("allow_squash")?,
        allow_merge_commit: row.try_get("allow_merge_commit")?,
        allow_rebase: row.try_get("allow_rebase")?,
        default_method: RepositoryMergeMethod::try_from(
            row.try_get::<String, _>("default_method")?.as_str(),
        )?,
    })
}

async fn repository_branch_names(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<String>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT name
        FROM repository_git_refs
        WHERE repository_id = $1 AND kind = 'branch'
        ORDER BY lower(name)
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let name: String = row.get("name");
            name.strip_prefix("refs/heads/")
                .unwrap_or(name.as_str())
                .to_owned()
        })
        .collect())
}

async fn repository_settings_audit_events(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<RepositorySettingsAuditEvent>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT id, event_type, changed_fields, actor_user_id, created_at
        FROM repository_settings_audit_events
        WHERE repository_id = $1
        ORDER BY created_at DESC
        LIMIT 20
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(RepositorySettingsAuditEvent {
                id: row.try_get("id")?,
                event_type: row.try_get("event_type")?,
                changed_fields: row.try_get("changed_fields")?,
                actor_user_id: row.try_get("actor_user_id")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}

pub async fn insert_commit(
    pool: &PgPool,
    repository_id: Uuid,
    input: CreateCommit,
) -> Result<Commit, RepositoryError> {
    let row = sqlx::query(
        r#"
        INSERT INTO commits (
            repository_id,
            oid,
            author_user_id,
            committer_user_id,
            message,
            tree_oid,
            parent_oids,
            committed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
            id,
            repository_id,
            oid,
            author_user_id,
            committer_user_id,
            message,
            tree_oid,
            parent_oids,
            committed_at,
            created_at
        "#,
    )
    .bind(repository_id)
    .bind(&input.oid)
    .bind(input.author_user_id)
    .bind(input.committer_user_id)
    .bind(&input.message)
    .bind(&input.tree_oid)
    .bind(&input.parent_oids)
    .bind(input.committed_at)
    .fetch_one(pool)
    .await?;

    Ok(commit_from_row(row))
}

pub async fn upsert_git_ref(
    pool: &PgPool,
    repository_id: Uuid,
    name: &str,
    kind: &str,
    target_commit_id: Option<Uuid>,
) -> Result<GitRef, RepositoryError> {
    let row = sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (repository_id, name)
        DO UPDATE SET kind = EXCLUDED.kind, target_commit_id = EXCLUDED.target_commit_id
        RETURNING id, repository_id, name, kind, target_commit_id, created_at, updated_at
        "#,
    )
    .bind(repository_id)
    .bind(name)
    .bind(kind)
    .bind(target_commit_id)
    .fetch_one(pool)
    .await?;

    Ok(git_ref_from_row(row))
}

pub async fn replace_repository_snapshot(
    pool: &PgPool,
    repository_id: Uuid,
    snapshot: RepositorySnapshot,
) -> Result<Commit, RepositoryError> {
    let mut transaction = pool.begin().await?;
    let existing_commit_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM commits WHERE repository_id = $1 AND oid = $2",
    )
    .bind(repository_id)
    .bind(&snapshot.commit.oid)
    .fetch_optional(&mut *transaction)
    .await?;

    let commit = if let Some(commit_id) = existing_commit_id {
        sqlx::query(
            r#"
            SELECT id, repository_id, oid, author_user_id, committer_user_id, message,
                   tree_oid, parent_oids, committed_at, created_at
            FROM commits
            WHERE id = $1
            "#,
        )
        .bind(commit_id)
        .fetch_one(&mut *transaction)
        .await
        .map(commit_from_row)?
    } else {
        let row = sqlx::query(
            r#"
            INSERT INTO commits (
                repository_id,
                oid,
                author_user_id,
                committer_user_id,
                message,
                tree_oid,
                parent_oids,
                committed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id,
                repository_id,
                oid,
                author_user_id,
                committer_user_id,
                message,
                tree_oid,
                parent_oids,
                committed_at,
                created_at
            "#,
        )
        .bind(repository_id)
        .bind(&snapshot.commit.oid)
        .bind(snapshot.commit.author_user_id)
        .bind(snapshot.commit.committer_user_id)
        .bind(&snapshot.commit.message)
        .bind(&snapshot.commit.tree_oid)
        .bind(&snapshot.commit.parent_oids)
        .bind(snapshot.commit.committed_at)
        .fetch_one(&mut *transaction)
        .await?;
        commit_from_row(row)
    };

    if let Some(tree_oid) = snapshot.commit.tree_oid.as_deref() {
        sqlx::query(
            r#"
            INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
            VALUES ($1, $2, 'tree', $3)
            ON CONFLICT (repository_id, oid) DO NOTHING
            "#,
        )
        .bind(repository_id)
        .bind(tree_oid)
        .bind(snapshot.files.len() as i64)
        .execute(&mut *transaction)
        .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
        VALUES ($1, $2, 'commit', 0)
        ON CONFLICT (repository_id, oid) DO NOTHING
        "#,
    )
    .bind(repository_id)
    .bind(&commit.oid)
    .execute(&mut *transaction)
    .await?;

    sqlx::query("DELETE FROM repository_files WHERE repository_id = $1 AND commit_id = $2")
        .bind(repository_id)
        .bind(commit.id)
        .execute(&mut *transaction)
        .await?;

    for file in &snapshot.files {
        sqlx::query(
            r#"
            INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
            VALUES ($1, $2, 'blob', $3)
            ON CONFLICT (repository_id, oid) DO NOTHING
            "#,
        )
        .bind(repository_id)
        .bind(&file.oid)
        .bind(file.byte_size)
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(repository_id)
        .bind(commit.id)
        .bind(&file.path)
        .bind(&file.content)
        .bind(&file.oid)
        .bind(file.byte_size)
        .execute(&mut *transaction)
        .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, $2, 'branch', $3)
        ON CONFLICT (repository_id, name)
        DO UPDATE SET kind = EXCLUDED.kind, target_commit_id = EXCLUDED.target_commit_id
        "#,
    )
    .bind(repository_id)
    .bind(format!("refs/heads/{}", snapshot.branch_name))
    .bind(commit.id)
    .execute(&mut *transaction)
    .await?;

    let indexed_files = snapshot
        .files
        .iter()
        .map(|file| IndexedSearchFile {
            path: file.path.clone(),
            content: file.content.clone(),
        })
        .collect::<Vec<_>>();
    transaction.commit().await?;
    if let Some(repository) = get_repository(pool, repository_id).await? {
        upsert_repository_search_index(pool, &repository, &commit, &indexed_files).await?;
    }
    super::git_transport::materialize_bare_repository_by_id(pool, repository_id)
        .await
        .map_err(|_| RepositoryError::GitStorageFailed)?;
    Ok(commit)
}

async fn bootstrap_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    request: &RepositoryBootstrapRequest,
) -> Result<(), RepositoryError> {
    let files = bootstrap_files(pool, repository, request).await?;
    if files.is_empty() {
        return Ok(());
    }

    let tree_oid = deterministic_oid(
        "tree",
        &files
            .iter()
            .map(|file| format!("{}:{}", file.path, file.oid))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    let commit_oid = deterministic_oid(
        "commit",
        &format!(
            "{}:{}:{}",
            repository.id, repository.default_branch, tree_oid
        ),
    );
    let commit = insert_commit(
        pool,
        repository.id,
        CreateCommit {
            oid: commit_oid.clone(),
            author_user_id: Some(actor_user_id),
            committer_user_id: Some(actor_user_id),
            message: "Initial commit".to_owned(),
            tree_oid: Some(tree_oid.clone()),
            parent_oids: Vec::new(),
            committed_at: Utc::now(),
        },
    )
    .await?;

    sqlx::query(
        r#"
        INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
        VALUES ($1, $2, 'tree', $3)
        ON CONFLICT (repository_id, oid) DO NOTHING
        "#,
    )
    .bind(repository.id)
    .bind(&tree_oid)
    .bind(files.len() as i64)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
        VALUES ($1, $2, 'commit', 0)
        ON CONFLICT (repository_id, oid) DO NOTHING
        "#,
    )
    .bind(repository.id)
    .bind(&commit_oid)
    .execute(pool)
    .await?;

    for file in &files {
        sqlx::query(
            r#"
            INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
            VALUES ($1, $2, 'blob', $3)
            ON CONFLICT (repository_id, oid) DO NOTHING
            "#,
        )
        .bind(repository.id)
        .bind(&file.oid)
        .bind(file.content.len() as i64)
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (repository_id, commit_id, lower(path))
            DO UPDATE SET commit_id = EXCLUDED.commit_id,
                          content = EXCLUDED.content,
                          oid = EXCLUDED.oid,
                          byte_size = EXCLUDED.byte_size
            "#,
        )
        .bind(repository.id)
        .bind(commit.id)
        .bind(&file.path)
        .bind(&file.content)
        .bind(&file.oid)
        .bind(file.content.len() as i64)
        .execute(pool)
        .await?;
    }

    upsert_git_ref(
        pool,
        repository.id,
        &format!("refs/heads/{}", repository.default_branch),
        "branch",
        Some(commit.id),
    )
    .await?;

    super::git_transport::materialize_bare_repository(pool, repository)
        .await
        .map_err(|_| RepositoryError::GitStorageFailed)?;

    let indexed_files = files
        .iter()
        .map(|file| IndexedSearchFile {
            path: file.path.clone(),
            content: file.content.clone(),
        })
        .collect::<Vec<_>>();
    upsert_repository_search_index(pool, repository, &commit, &indexed_files).await?;

    Ok(())
}

async fn bootstrap_files(
    pool: &PgPool,
    repository: &Repository,
    request: &RepositoryBootstrapRequest,
) -> Result<Vec<BootstrapFile>, RepositoryError> {
    let mut files = Vec::new();

    let template_slug = request.template_slug.as_deref().unwrap_or("blank").trim();
    if !template_slug.is_empty() && template_slug != "blank" {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM repository_creation_templates WHERE slug = $1)",
        )
        .bind(template_slug)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(RepositoryError::UnknownTemplate(template_slug.to_owned()));
        }
        files.extend(template_files(template_slug, repository));
    }

    if request.initialize_readme {
        files.push(make_bootstrap_file(
            "README.md",
            &format!(
                "# {}\n\n{}{}\n",
                repository.name,
                repository
                    .description
                    .as_deref()
                    .unwrap_or("A new opengithub repository."),
                if template_slug == "blank" {
                    ""
                } else {
                    "\n\nGenerated from a repository template."
                }
            ),
        ));
    }

    if let Some(slug) = request
        .gitignore_template_slug
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let content = sqlx::query_scalar::<_, String>(
            "SELECT content FROM gitignore_templates WHERE slug = $1",
        )
        .bind(slug)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RepositoryError::UnknownGitignoreTemplate(slug.to_owned()))?;
        files.push(make_bootstrap_file(".gitignore", &content));
    }

    if let Some(slug) = request
        .license_template_slug
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let content = sqlx::query_scalar::<_, String>(
            "SELECT content FROM license_templates WHERE slug = $1",
        )
        .bind(slug)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RepositoryError::UnknownLicenseTemplate(slug.to_owned()))?;
        let owner = repository.owner_login.clone();
        files.push(make_bootstrap_file(
            "LICENSE",
            &content
                .replace("{{year}}", &Utc::now().format("%Y").to_string())
                .replace("{{owner}}", &owner),
        ));
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    files.dedup_by(|left, right| left.path.eq_ignore_ascii_case(&right.path));
    Ok(files)
}

fn template_files(slug: &str, repository: &Repository) -> Vec<BootstrapFile> {
    match slug {
        "node-typescript" => vec![
            make_bootstrap_file("package.json", &format!("{{\n  \"name\": \"{}\",\n  \"version\": \"0.1.0\",\n  \"type\": \"module\"\n}}\n", repository.name)),
            make_bootstrap_file("src/index.ts", "export function main() {\n  return \"hello from opengithub\";\n}\n"),
        ],
        "rust-axum" => vec![
            make_bootstrap_file("Cargo.toml", &format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\naxum = \"0.7\"\ntokio = {{ version = \"1\", features = [\"full\"] }}\n", repository.name.replace('-', "_"))),
            make_bootstrap_file("src/main.rs", "use axum::{routing::get, Router};\n\n#[tokio::main]\nasync fn main() {\n    let app = Router::new().route(\"/\", get(|| async { \"ok\" }));\n    let listener = tokio::net::TcpListener::bind(\"0.0.0.0:3000\").await.unwrap();\n    axum::serve(listener, app).await.unwrap();\n}\n"),
        ],
        _ => Vec::new(),
    }
}

fn make_bootstrap_file(path: &str, content: &str) -> BootstrapFile {
    BootstrapFile {
        path: path.to_owned(),
        content: content.to_owned(),
        oid: deterministic_oid("blob", &format!("{path}\0{content}")),
    }
}

fn deterministic_oid(kind: &str, content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update(b"\0");
    hasher.update(content.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

async fn list_repository_files(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<RepositoryFile>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT id, repository_id, commit_id, path, content, oid, byte_size, created_at
        FROM repository_files
        WHERE repository_id = $1
        ORDER BY lower(path) ASC
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(repository_file_from_row).collect())
}

async fn list_repository_files_for_resolved_ref(
    pool: &PgPool,
    repository_id: Uuid,
    resolved_ref: &RepositoryResolvedRef,
) -> Result<Vec<RepositoryFile>, RepositoryError> {
    let Some(target_oid) = resolved_ref.target_oid.as_deref() else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query(
        r#"
        SELECT repository_files.id,
               repository_files.repository_id,
               repository_files.commit_id,
               repository_files.path,
               repository_files.content,
               repository_files.oid,
               repository_files.byte_size,
               repository_files.created_at
        FROM repository_files
        JOIN commits ON commits.id = repository_files.commit_id
        WHERE repository_files.repository_id = $1
          AND commits.oid = $2
        ORDER BY lower(repository_files.path) ASC
        "#,
    )
    .bind(repository_id)
    .bind(target_oid)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(repository_file_from_row).collect())
}

async fn repository_path_exists_for_commit(
    pool: &PgPool,
    repository_id: Uuid,
    commit_id: Option<Uuid>,
    path: &str,
) -> Result<bool, RepositoryError> {
    let Some(commit_id) = commit_id else {
        return Ok(false);
    };
    if path.is_empty() {
        return Ok(true);
    }
    let path_prefix = format!("{path}/%");
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM repository_files
            WHERE repository_id = $1
              AND commit_id = $2
              AND (path = $3 OR path LIKE $4)
        )
        "#,
    )
    .bind(repository_id)
    .bind(commit_id)
    .bind(path)
    .bind(path_prefix)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

async fn count_repository_refs(
    pool: &PgPool,
    repository_id: Uuid,
    kind: &str,
) -> Result<i64, RepositoryError> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_git_refs WHERE repository_id = $1 AND kind = $2",
    )
    .bind(repository_id)
    .bind(kind)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

async fn get_repository_ref(
    pool: &PgPool,
    repository_id: Uuid,
    name: &str,
) -> Result<Option<GitRef>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT id, repository_id, name, kind, target_commit_id, created_at, updated_at
        FROM repository_git_refs
        WHERE repository_id = $1 AND name = $2
        "#,
    )
    .bind(repository_id)
    .bind(name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(git_ref_from_row))
}

async fn latest_commit_for_repository(
    pool: &PgPool,
    repository: &Repository,
) -> Result<Option<RepositoryLatestCommit>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT oid, message, committed_at
        FROM commits
        WHERE repository_id = $1
        ORDER BY committed_at DESC, created_at DESC
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| {
        let oid: String = row.get("oid");
        RepositoryLatestCommit {
            short_oid: oid.chars().take(7).collect(),
            href: format!(
                "/{}/{}/commit/{}",
                repository.owner_login, repository.name, oid
            ),
            oid,
            message: row.get("message"),
            committed_at: row.get("committed_at"),
        }
    }))
}

async fn latest_commit_for_file(
    pool: &PgPool,
    repository: &Repository,
    file: &RepositoryFile,
) -> Result<Option<RepositoryLatestCommit>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT oid, message, committed_at
        FROM commits
        WHERE id = $1
          AND repository_id = $2
        LIMIT 1
        "#,
    )
    .bind(file.commit_id)
    .bind(repository.id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| {
        let oid: String = row.get("oid");
        RepositoryLatestCommit {
            short_oid: oid.chars().take(7).collect(),
            href: format!(
                "/{}/{}/commit/{}",
                repository.owner_login, repository.name, oid
            ),
            oid,
            message: row.get("message"),
            committed_at: row.get("committed_at"),
        }
    }))
}

async fn blame_commit_for_file(
    pool: &PgPool,
    repository: &Repository,
    file: &RepositoryFile,
) -> Result<Option<RepositoryBlameCommit>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT commits.oid, commits.message, commits.committed_at,
               COALESCE(NULLIF(users.username, ''), users.email) AS author_login
        FROM commits
        LEFT JOIN users ON users.id = commits.author_user_id
        WHERE commits.id = $1
          AND commits.repository_id = $2
        LIMIT 1
        "#,
    )
    .bind(file.commit_id)
    .bind(repository.id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| {
        let oid: String = row.get("oid");
        RepositoryBlameCommit {
            short_oid: oid.chars().take(7).collect(),
            href: format!(
                "/{}/{}/commit/{}",
                repository.owner_login, repository.name, oid
            ),
            oid,
            message: row.get("message"),
            committed_at: row.get("committed_at"),
            author_login: row.get("author_login"),
        }
    }))
}

async fn record_recent_repository_visit(
    pool: &PgPool,
    user_id: Uuid,
    repository_id: Uuid,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO recent_repository_visits (user_id, repository_id, visited_at)
        VALUES ($1, $2, now())
        ON CONFLICT (user_id, repository_id)
        DO UPDATE SET visited_at = EXCLUDED.visited_at
        "#,
    )
    .bind(user_id)
    .bind(repository_id)
    .execute(pool)
    .await?;

    Ok(())
}

fn repository_root_entries(
    repository: &Repository,
    files: &[RepositoryFile],
    latest_commit: Option<&RepositoryLatestCommit>,
) -> Vec<RepositoryTreeEntry> {
    let mut folders: BTreeMap<String, DateTime<Utc>> = BTreeMap::new();
    let mut entries = Vec::new();

    for file in files {
        if let Some((folder, _)) = file.path.split_once('/') {
            folders
                .entry(folder.to_owned())
                .and_modify(|updated_at| {
                    if file.created_at > *updated_at {
                        *updated_at = file.created_at;
                    }
                })
                .or_insert(file.created_at);
        } else {
            entries.push(RepositoryTreeEntry {
                kind: "file".to_owned(),
                name: file.path.clone(),
                path: file.path.clone(),
                href: format!(
                    "/{}/{}/blob/{}/{}",
                    repository.owner_login, repository.name, repository.default_branch, file.path
                ),
                byte_size: Some(file.byte_size),
                latest_commit_message: latest_commit.map(|commit| commit.message.clone()),
                latest_commit_href: latest_commit.map(|commit| commit.href.clone()),
                updated_at: file.created_at,
            });
        }
    }

    for (folder, updated_at) in folders {
        entries.push(RepositoryTreeEntry {
            kind: "folder".to_owned(),
            name: folder.clone(),
            path: folder.clone(),
            href: format!(
                "/{}/{}/tree/{}/{}",
                repository.owner_login, repository.name, repository.default_branch, folder
            ),
            byte_size: None,
            latest_commit_message: latest_commit.map(|commit| commit.message.clone()),
            latest_commit_href: latest_commit.map(|commit| commit.href.clone()),
            updated_at,
        });
    }

    entries.sort_by(
        |left, right| match (left.kind.as_str(), right.kind.as_str()) {
            ("folder", "file") => std::cmp::Ordering::Less,
            ("file", "folder") => std::cmp::Ordering::Greater,
            _ => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
        },
    );
    entries
}

fn repository_entries_for_path(
    repository: &Repository,
    ref_name: &str,
    files: &[RepositoryFile],
    path: &str,
) -> Vec<RepositoryTreeEntry> {
    let prefix = if path.is_empty() {
        String::new()
    } else {
        format!("{path}/")
    };
    let mut folders: BTreeMap<String, DateTime<Utc>> = BTreeMap::new();
    let mut entries = Vec::new();

    for file in files {
        if !file.path.starts_with(&prefix) {
            continue;
        }
        let remainder = &file.path[prefix.len()..];
        if remainder.is_empty() {
            continue;
        }
        if let Some((folder, _)) = remainder.split_once('/') {
            let child_path = if path.is_empty() {
                folder.to_owned()
            } else {
                format!("{path}/{folder}")
            };
            folders
                .entry(child_path)
                .and_modify(|updated_at| {
                    if file.created_at > *updated_at {
                        *updated_at = file.created_at;
                    }
                })
                .or_insert(file.created_at);
        } else {
            entries.push(RepositoryTreeEntry {
                kind: "file".to_owned(),
                name: remainder.to_owned(),
                path: file.path.clone(),
                href: format!(
                    "/{}/{}/blob/{}/{}",
                    repository.owner_login,
                    repository.name,
                    percent_encode_segment(ref_name),
                    percent_encode_path(&file.path)
                ),
                byte_size: Some(file.byte_size),
                latest_commit_message: None,
                latest_commit_href: None,
                updated_at: file.created_at,
            });
        }
    }

    for (folder_path, updated_at) in folders {
        entries.push(RepositoryTreeEntry {
            kind: "folder".to_owned(),
            name: folder_path
                .rsplit('/')
                .next()
                .unwrap_or(folder_path.as_str())
                .to_owned(),
            path: folder_path.clone(),
            href: format!(
                "/{}/{}/tree/{}/{}",
                repository.owner_login,
                repository.name,
                percent_encode_segment(ref_name),
                percent_encode_path(&folder_path)
            ),
            byte_size: None,
            latest_commit_message: None,
            latest_commit_href: None,
            updated_at,
        });
    }

    entries.sort_by(
        |left, right| match (left.kind.as_str(), right.kind.as_str()) {
            ("folder", "file") => std::cmp::Ordering::Less,
            ("file", "folder") => std::cmp::Ordering::Greater,
            _ => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
        },
    );
    entries
}

fn readme_for_path(files: &[RepositoryFile], path: &str) -> Option<RepositoryFile> {
    let prefix = if path.is_empty() {
        String::new()
    } else {
        format!("{path}/")
    };
    files
        .iter()
        .find(|file| {
            file.path
                .strip_prefix(&prefix)
                .is_some_and(|remainder| remainder.eq_ignore_ascii_case("README.md"))
        })
        .cloned()
}

fn repository_breadcrumbs(
    repository: &Repository,
    ref_name: &str,
    path: &str,
) -> Vec<RepositoryPathBreadcrumb> {
    let mut breadcrumbs = vec![RepositoryPathBreadcrumb {
        name: repository.name.clone(),
        path: String::new(),
        href: format!(
            "/{}/{}/tree/{}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(ref_name)
        ),
    }];
    let mut current = String::new();
    for segment in path.split('/').filter(|segment| !segment.is_empty()) {
        if current.is_empty() {
            current.push_str(segment);
        } else {
            current.push('/');
            current.push_str(segment);
        }
        breadcrumbs.push(RepositoryPathBreadcrumb {
            name: segment.to_owned(),
            path: current.clone(),
            href: format!(
                "/{}/{}/tree/{}/{}",
                repository.owner_login,
                repository.name,
                percent_encode_segment(ref_name),
                percent_encode_path(&current)
            ),
        });
    }
    breadcrumbs
}

fn repository_parent_tree_href(
    repository: &Repository,
    ref_name: &str,
    path: &str,
) -> Option<String> {
    if path.is_empty() {
        return None;
    }
    let parent = path
        .rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or("");
    Some(if parent.is_empty() {
        format!(
            "/{}/{}/tree/{}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(ref_name)
        )
    } else {
        format!(
            "/{}/{}/tree/{}/{}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(ref_name),
            percent_encode_path(parent)
        )
    })
}

fn repository_default_branch_href(repository: &Repository) -> String {
    repository_tree_href(repository, &repository.default_branch, "")
}

fn repository_path_not_found_error(repository: &Repository, path: &str) -> RepositoryError {
    RepositoryError::PathNotFoundWithRecovery {
        path: path.to_owned(),
        recovery_href: repository_default_branch_href(repository),
        default_branch_href: repository_default_branch_href(repository),
    }
}

fn repository_tree_href(repository: &Repository, ref_name: &str, path: &str) -> String {
    if path.is_empty() {
        format!(
            "/{}/{}/tree/{}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(ref_name)
        )
    } else {
        format!(
            "/{}/{}/tree/{}/{}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(ref_name),
            percent_encode_path(path)
        )
    }
}

fn repository_blob_href(repository: &Repository, ref_name: &str, path: &str) -> String {
    format!(
        "/{}/{}/blob/{}/{}",
        repository.owner_login,
        repository.name,
        percent_encode_segment(ref_name),
        percent_encode_path(path)
    )
}

fn ref_matches_active(qualified_name: &str, short_name: &str, active_ref: &str) -> bool {
    let normalized = active_ref.trim();
    qualified_name == normalized
        || short_name == normalized
        || qualified_name == format!("refs/heads/{normalized}")
        || qualified_name == format!("refs/tags/{normalized}")
}

fn parent_path(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or("")
}

fn repository_history_href(repository: &Repository, ref_name: &str, path: &str) -> String {
    if path.is_empty() {
        format!(
            "/{}/{}/commits/{}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(ref_name)
        )
    } else {
        format!(
            "/{}/{}/commits/{}/{}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(ref_name),
            percent_encode_path(path)
        )
    }
}

async fn viewer_permission_for_user(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<Option<String>, RepositoryError> {
    Ok(
        repository_permission_for_user(pool, repository.id, actor_user_id)
            .await?
            .map(|permission| permission.role.as_str().to_owned())
            .or_else(|| {
                if repository.visibility == RepositoryVisibility::Public {
                    Some("read".to_owned())
                } else {
                    None
                }
            }),
    )
}

async fn resolve_repository_ref(
    pool: &PgPool,
    repository: &Repository,
    ref_name: Option<&str>,
) -> Result<RepositoryResolvedRef, RepositoryError> {
    let ref_name = ref_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&repository.default_branch);
    let normalized = normalize_repository_path(ref_name)?;
    let branch_ref = format!("refs/heads/{normalized}");
    let tag_ref = format!("refs/tags/{normalized}");
    let row = sqlx::query(
        r#"
        SELECT repository_git_refs.name,
               repository_git_refs.kind,
               commits.oid AS target_oid
        FROM repository_git_refs
        LEFT JOIN commits ON commits.id = repository_git_refs.target_commit_id
        WHERE repository_git_refs.repository_id = $1
          AND repository_git_refs.name IN ($2, $3, $4)
        ORDER BY CASE
            WHEN repository_git_refs.name = $2 THEN 0
            WHEN repository_git_refs.name = $3 THEN 1
            ELSE 2
        END
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(&branch_ref)
    .bind(&tag_ref)
    .bind(&normalized)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Err(RepositoryError::RefNotFoundWithRecovery {
            ref_name: normalized,
            recovery_href: repository_default_branch_href(repository),
            default_branch_href: repository_default_branch_href(repository),
        });
    };
    let qualified_name: String = row.get("name");
    let kind: String = row.get("kind");
    let short_name = qualified_name
        .strip_prefix("refs/heads/")
        .or_else(|| qualified_name.strip_prefix("refs/tags/"))
        .unwrap_or(&qualified_name)
        .to_owned();

    Ok(RepositoryResolvedRef {
        recovery_href: repository_tree_href(repository, &short_name, ""),
        kind,
        short_name,
        qualified_name,
        target_oid: row.get("target_oid"),
    })
}

fn normalize_repository_path(value: &str) -> Result<String, RepositoryError> {
    let trimmed = value.trim_matches('/');
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    let mut segments = Vec::new();
    for segment in trimmed.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") || segment.contains('\\') {
            return Err(RepositoryError::PathNotFound);
        }
        segments.push(segment);
    }
    Ok(segments.join("/"))
}

fn blame_lines(content: &str) -> Vec<String> {
    let mut lines = content
        .split('\n')
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

async fn repository_commit_history(
    pool: &PgPool,
    repository: &Repository,
    resolved_ref: &RepositoryResolvedRef,
    path: Option<&str>,
    query: RepositoryCommitHistoryQuery<'_>,
) -> Result<RepositoryCommitHistoryView, RepositoryError> {
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;
    let path = path.unwrap_or("");
    let path_prefix = if path.is_empty() {
        None
    } else {
        Some(format!("{path}/%"))
    };
    let normalized_author = query
        .author
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let until = query.until;
    let target_oid = resolved_ref.target_oid.as_deref();

    let author_options_rows = sqlx::query(
        r#"
        WITH RECURSIVE reachable AS (
            SELECT c.id, c.oid, c.parent_oids
            FROM commits c
            WHERE c.repository_id = $1
              AND ($2::text IS NULL OR c.oid = $2)
            UNION
            SELECT parent.id, parent.oid, parent.parent_oids
            FROM commits parent
            JOIN reachable child ON parent.oid = ANY(child.parent_oids)
            WHERE parent.repository_id = $1
        ),
        scoped AS (
            SELECT DISTINCT commits.id, commits.author_user_id
            FROM commits
            JOIN reachable ON reachable.id = commits.id
            LEFT JOIN repository_files ON repository_files.commit_id = commits.id
            WHERE commits.repository_id = $1
              AND ($3::text = '' OR repository_files.path = $3 OR repository_files.path LIKE $4)
        )
        SELECT COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.avatar_url,
               count(*)::bigint AS count
        FROM scoped
        JOIN users ON users.id = scoped.author_user_id
        GROUP BY COALESCE(NULLIF(users.username, ''), users.email), users.avatar_url
        ORDER BY lower(COALESCE(NULLIF(users.username, ''), users.email)) ASC
        "#,
    )
    .bind(repository.id)
    .bind(target_oid)
    .bind(path)
    .bind(path_prefix.as_deref().unwrap_or(""))
    .fetch_all(pool)
    .await?;

    let author_options = author_options_rows
        .into_iter()
        .map(|row| {
            let login: String = row.get("login");
            RepositoryCommitAuthorOption {
                active: normalized_author
                    .as_deref()
                    .is_some_and(|author| author.eq_ignore_ascii_case(&login)),
                login,
                avatar_url: row.get("avatar_url"),
                count: row.get("count"),
            }
        })
        .collect::<Vec<_>>();

    if let Some(author) = normalized_author.as_deref() {
        if !author_options
            .iter()
            .any(|option| option.login.eq_ignore_ascii_case(author))
        {
            return Err(RepositoryError::PathNotFound);
        }
    }

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        WITH RECURSIVE reachable AS (
            SELECT c.id, c.oid, c.parent_oids
            FROM commits c
            WHERE c.repository_id = $1
              AND ($2::text IS NULL OR c.oid = $2)
            UNION
            SELECT parent.id, parent.oid, parent.parent_oids
            FROM commits parent
            JOIN reachable child ON parent.oid = ANY(child.parent_oids)
            WHERE parent.repository_id = $1
        )
        SELECT count(DISTINCT commits.id)::bigint
        FROM commits
        JOIN reachable ON reachable.id = commits.id
        LEFT JOIN repository_files ON repository_files.commit_id = commits.id
        LEFT JOIN users ON users.id = commits.author_user_id
        WHERE commits.repository_id = $1
          AND ($3::text = '' OR repository_files.path = $3 OR repository_files.path LIKE $4)
          AND ($5::text IS NULL OR lower(COALESCE(NULLIF(users.username, ''), users.email)) = lower($5))
          AND ($6::timestamptz IS NULL OR commits.committed_at <= $6)
        "#,
    )
    .bind(repository.id)
    .bind(target_oid)
    .bind(path)
    .bind(path_prefix.as_deref().unwrap_or(""))
    .bind(normalized_author.as_deref())
    .bind(until)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        r#"
        WITH RECURSIVE reachable AS (
            SELECT c.id, c.oid, c.parent_oids
            FROM commits c
            WHERE c.repository_id = $1
              AND ($2::text IS NULL OR c.oid = $2)
            UNION
            SELECT parent.id, parent.oid, parent.parent_oids
            FROM commits parent
            JOIN reachable child ON parent.oid = ANY(child.parent_oids)
            WHERE parent.repository_id = $1
        )
        SELECT DISTINCT commits.id,
               commits.oid,
               commits.message,
               commits.committed_at,
               commits.created_at AS sort_created_at,
               COALESCE(NULLIF(users.username, ''), users.email) AS author_login,
               users.avatar_url AS author_avatar_url,
               commits.author_user_id,
               commits.signature_fingerprint,
               commits.signature_summary,
               COALESCE(statuses.status, workflow_statuses.status, 'pending') AS status,
               COALESCE(statuses.conclusion, workflow_statuses.conclusion) AS conclusion,
               COALESCE(statuses.total_count, workflow_statuses.total_count, 0)::bigint AS total_count,
               COALESCE(statuses.completed_count, workflow_statuses.completed_count, 0)::bigint AS completed_count,
               COALESCE(statuses.failed_count, workflow_statuses.failed_count, 0)::bigint AS failed_count,
               COALESCE(pr_links.links, '[]'::jsonb) AS pull_requests
        FROM commits
        JOIN reachable ON reachable.id = commits.id
        LEFT JOIN repository_files ON repository_files.commit_id = commits.id
        LEFT JOIN users ON users.id = commits.author_user_id
        LEFT JOIN repository_commit_status_summaries statuses ON statuses.commit_id = commits.id
        LEFT JOIN LATERAL (
            SELECT CASE
                    WHEN count(*) FILTER (WHERE workflow_runs.status IN ('queued', 'in_progress')) > 0 THEN 'running'
                    WHEN count(*) = 0 THEN NULL
                    ELSE 'completed'
                   END AS status,
                   CASE
                    WHEN count(*) = 0 THEN NULL
                    WHEN count(*) FILTER (WHERE workflow_runs.conclusion = 'failure') > 0 THEN 'failure'
                    WHEN count(*) FILTER (WHERE workflow_runs.conclusion = 'cancelled') > 0 THEN 'cancelled'
                    WHEN count(*) FILTER (WHERE workflow_runs.conclusion = 'success') = count(*) THEN 'success'
                    ELSE NULL
                   END AS conclusion,
                   count(*)::bigint AS total_count,
                   count(*) FILTER (WHERE workflow_runs.status = 'completed')::bigint AS completed_count,
                   count(*) FILTER (WHERE workflow_runs.conclusion = 'failure')::bigint AS failed_count
            FROM workflow_runs
            WHERE workflow_runs.repository_id = commits.repository_id
              AND (workflow_runs.commit_id = commits.id OR workflow_runs.head_sha = commits.oid)
        ) workflow_statuses ON true
        LEFT JOIN LATERAL (
            SELECT jsonb_agg(
                jsonb_build_object(
                    'number', pull_requests.number,
                    'title', pull_requests.title,
                    'href', format('/%s/%s/pull/%s', $7::text, $8::text, pull_requests.number),
                    'state', pull_requests.state
                )
                ORDER BY pull_requests.number
            ) AS links
            FROM pull_request_commits
            JOIN pull_requests ON pull_requests.id = pull_request_commits.pull_request_id
            WHERE pull_request_commits.commit_id = commits.id
        ) pr_links ON true
        WHERE commits.repository_id = $1
          AND ($3::text = '' OR repository_files.path = $3 OR repository_files.path LIKE $4)
          AND ($5::text IS NULL OR lower(COALESCE(NULLIF(users.username, ''), users.email)) = lower($5))
          AND ($6::timestamptz IS NULL OR commits.committed_at <= $6)
        ORDER BY commits.committed_at DESC, commits.created_at DESC
        LIMIT $9 OFFSET $10
        "#,
    )
    .bind(repository.id)
    .bind(target_oid)
    .bind(path)
    .bind(path_prefix.as_deref().unwrap_or(""))
    .bind(normalized_author.as_deref())
    .bind(until)
    .bind(&repository.owner_login)
    .bind(&repository.name)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let oid: String = row.get("oid");
        let message: String = row.get("message");
        let (subject, body) = split_commit_message(&message);
        let signature_fingerprint: Option<String> = row.get("signature_fingerprint");
        let stored_signature_summary: Option<String> = row.get("signature_summary");
        let signature = super::signing_keys::signature_presentation_for_user(
            pool,
            row.get("author_user_id"),
            signature_fingerprint.as_deref(),
            stored_signature_summary.as_deref(),
        )
        .await
        .map_err(|error| match error {
            super::signing_keys::SigningKeyError::Sqlx(error) => RepositoryError::Sqlx(error),
            _ => RepositoryError::GitStorageFailed,
        })?;
        let pull_requests: Vec<RepositoryCommitPullRequestLink> =
            serde_json::from_value(row.get("pull_requests")).unwrap_or_default();
        items.push(RepositoryCommitListItem {
            short_oid: oid.chars().take(7).collect(),
            href: format!(
                "/{}/{}/commit/{}",
                repository.owner_login, repository.name, oid
            ),
            browse_href: format!(
                "/{}/{}/tree/{}",
                repository.owner_login, repository.name, oid
            ),
            oid: oid.clone(),
            message,
            subject,
            body,
            committed_at: row.get("committed_at"),
            author_login: row.get("author_login"),
            author_avatar_url: row.get("author_avatar_url"),
            pull_requests,
            status: RepositoryCommitStatusSummary {
                status: row.get("status"),
                conclusion: row.get("conclusion"),
                total_count: row.get("total_count"),
                completed_count: row.get("completed_count"),
                failed_count: row.get("failed_count"),
                href: format!(
                    "/{}/{}/actions?commit={}",
                    repository.owner_login, repository.name, oid
                ),
            },
            verification: RepositoryCommitVerificationSummary {
                verified: signature.verified,
                signature_state: signature.signature_state,
                signature_summary: signature.signature_summary,
            },
        });
    }

    Ok(RepositoryCommitHistoryView {
        repository: RepositoryCommitHistoryRepository {
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            default_branch: repository.default_branch.clone(),
            visibility: repository.visibility.clone(),
        },
        resolved_ref: RepositoryCommitResolvedRef {
            short_name: resolved_ref.short_name.clone(),
            qualified_name: resolved_ref.qualified_name.clone(),
            kind: resolved_ref.kind.clone(),
            target_oid: resolved_ref.target_oid.clone(),
            href: repository_tree_href(repository, &resolved_ref.short_name, path),
        },
        filters: RepositoryCommitHistoryFilters {
            path: Some(path.to_owned()).filter(|value| !value.is_empty()),
            author: normalized_author,
            until,
        },
        groups: group_commit_history_items(items),
        author_options,
        total,
        page,
        page_size,
        has_next_page: offset + page_size < total,
        has_previous_page: page > 1,
    })
}

async fn repository_branches_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    query: RepositoryBranchesQuery<'_>,
) -> Result<RepositoryBranchesView, RepositoryError> {
    const STALE_CUTOFF_DAYS: i64 = 90;

    let tab = query
        .tab
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("overview")
        .to_lowercase();
    if !matches!(tab.as_str(), "overview" | "active" | "stale" | "all") {
        return Err(RepositoryError::InvalidBranchDirectoryQuery(format!(
            "unsupported tab `{tab}`"
        )));
    }
    let normalized_query = query
        .query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    if normalized_query
        .as_deref()
        .is_some_and(|value| value.chars().count() > 120)
    {
        return Err(RepositoryError::InvalidBranchDirectoryQuery(
            "search query must be 120 characters or fewer".to_owned(),
        ));
    }
    let query_lower = normalized_query
        .as_deref()
        .map(str::to_lowercase)
        .unwrap_or_default();
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, 100);
    let can_admin = can_admin_repository(pool, repository, actor_user_id).await?;
    let viewer_permission = viewer_permission_for_user(pool, repository, actor_user_id)
        .await?
        .unwrap_or_else(|| "read".to_owned());

    let rows = sqlx::query(
        r#"
        SELECT refs.name,
               refs.updated_at,
               commits.id AS commit_id,
               commits.oid,
               commits.message,
               commits.committed_at,
               COALESCE(NULLIF(users.username, ''), users.email) AS author_login,
               users.avatar_url AS author_avatar_url,
               COALESCE(statuses.status, workflow_statuses.status, 'pending') AS status,
               COALESCE(statuses.conclusion, workflow_statuses.conclusion) AS conclusion,
               COALESCE(statuses.total_count, workflow_statuses.total_count, 0)::bigint AS total_count,
               COALESCE(statuses.completed_count, workflow_statuses.completed_count, 0)::bigint AS completed_count,
               COALESCE(statuses.failed_count, workflow_statuses.failed_count, 0)::bigint AS failed_count,
               pr_link.link AS pull_request
        FROM repository_git_refs refs
        LEFT JOIN commits ON commits.id = refs.target_commit_id
        LEFT JOIN users ON users.id = commits.author_user_id
        LEFT JOIN repository_commit_status_summaries statuses ON statuses.commit_id = commits.id
        LEFT JOIN LATERAL (
            SELECT CASE
                    WHEN count(*) FILTER (WHERE workflow_runs.status IN ('queued', 'in_progress')) > 0 THEN 'running'
                    WHEN count(*) = 0 THEN NULL
                    ELSE 'completed'
                   END AS status,
                   CASE
                    WHEN count(*) = 0 THEN NULL
                    WHEN count(*) FILTER (WHERE workflow_runs.conclusion = 'failure') > 0 THEN 'failure'
                    WHEN count(*) FILTER (WHERE workflow_runs.conclusion = 'cancelled') > 0 THEN 'cancelled'
                    WHEN count(*) FILTER (WHERE workflow_runs.conclusion = 'success') = count(*) THEN 'success'
                    ELSE NULL
                   END AS conclusion,
                   count(*)::bigint AS total_count,
                   count(*) FILTER (WHERE workflow_runs.status = 'completed')::bigint AS completed_count,
                   count(*) FILTER (WHERE workflow_runs.conclusion = 'failure')::bigint AS failed_count
            FROM workflow_runs
            WHERE workflow_runs.repository_id = refs.repository_id
              AND (workflow_runs.commit_id = commits.id OR workflow_runs.head_sha = commits.oid)
        ) workflow_statuses ON true
        LEFT JOIN LATERAL (
            SELECT jsonb_build_object(
                'number', pull_requests.number,
                'title', pull_requests.title,
                'state', pull_requests.state,
                'draft', COALESCE(pull_requests.is_draft, false),
                'href', format('/%s/%s/pull/%s', $2::text, $3::text, pull_requests.number)
            ) AS link
            FROM pull_requests
            WHERE pull_requests.repository_id = refs.repository_id
              AND pull_requests.head_ref = regexp_replace(refs.name, '^refs/heads/', '')
              AND pull_requests.state <> 'closed'
            ORDER BY pull_requests.updated_at DESC, pull_requests.number DESC
            LIMIT 1
        ) pr_link ON true
        WHERE refs.repository_id = $1 AND refs.kind = 'branch'
        ORDER BY CASE WHEN refs.name = $4 THEN 0 ELSE 1 END,
                 COALESCE(commits.committed_at, refs.updated_at) DESC,
                 lower(refs.name) ASC
        "#,
    )
    .bind(repository.id)
    .bind(&repository.owner_login)
    .bind(&repository.name)
    .bind(format!("refs/heads/{}", repository.default_branch))
    .fetch_all(pool)
    .await?;

    let branch_refs = repository_branch_ref_summaries(pool, repository.id).await?;
    let rules = repository_branch_rules(pool, repository.id, &branch_refs, can_admin).await?;
    let rulesets = repository_rulesets(pool, repository.id, &branch_refs, can_admin).await?;
    let default_target_oid = rows
        .iter()
        .find(|row| {
            let name: String = row.get("name");
            branch_short_name(&name) == repository.default_branch
        })
        .and_then(|row| row.get::<Option<String>, _>("oid"));

    let mut all_rows = Vec::new();
    for row in rows {
        let qualified_name: String = row.get("name");
        let short_name = branch_short_name(&qualified_name);
        let is_default = short_name == repository.default_branch;
        let updated_at = row
            .get::<Option<DateTime<Utc>>, _>("committed_at")
            .unwrap_or_else(|| row.get("updated_at"));
        let classification = if is_default {
            "default"
        } else if updated_at < Utc::now() - chrono::Duration::days(STALE_CUTOFF_DAYS) {
            "stale"
        } else {
            "active"
        }
        .to_owned();
        let oid = row.get::<Option<String>, _>("oid");
        let (ahead, behind) = branch_ahead_behind(
            pool,
            repository.id,
            oid.as_deref(),
            default_target_oid.as_deref(),
        )
        .await?;
        let protection = branch_protection_summary(repository, &short_name, &rules, &rulesets);
        let latest_commit = oid.as_ref().map(|oid| {
            let message: String = row.get("message");
            let (subject, _) = split_commit_message(&message);
            RepositoryBranchLatestCommitSummary {
                short_oid: oid.chars().take(7).collect(),
                href: format!(
                    "/{}/{}/commit/{}",
                    repository.owner_login, repository.name, oid
                ),
                oid: oid.clone(),
                subject,
                committed_at: row.get("committed_at"),
                author_login: row.get("author_login"),
                author_avatar_url: row.get("author_avatar_url"),
            }
        });
        let pull_request = row
            .get::<Option<serde_json::Value>, _>("pull_request")
            .and_then(|value| serde_json::from_value(value).ok());
        all_rows.push(RepositoryBranchDirectoryRow {
            href: repository_tree_href(repository, &short_name, ""),
            commits_href: repository_history_href(repository, &short_name, ""),
            activity_href: format!(
                "/{}/{}/branches/{}",
                repository.owner_login,
                repository.name,
                percent_encode_segment(&short_name)
            ),
            checks: RepositoryBranchCheckSummary {
                status: row.get("status"),
                conclusion: row.get("conclusion"),
                total_count: row.get("total_count"),
                completed_count: row.get("completed_count"),
                failed_count: row.get("failed_count"),
                href: oid
                    .as_ref()
                    .map(|oid| {
                        format!(
                            "/{}/{}/actions?commit={}",
                            repository.owner_login, repository.name, oid
                        )
                    })
                    .unwrap_or_else(|| {
                        format!("/{}/{}/actions", repository.owner_login, repository.name)
                    }),
            },
            capabilities: RepositoryBranchCapabilities {
                can_copy: true,
                can_view_activity: true,
                can_view_rules: protection.protected || can_admin,
                can_delete: can_admin && !is_default && !protection.protected,
                delete_disabled_reason: if is_default {
                    Some("The default branch cannot be deleted.".to_owned())
                } else if protection.protected {
                    Some("Protected branches require policy changes before deletion.".to_owned())
                } else if can_admin {
                    None
                } else {
                    Some("Admin access is required to delete branches.".to_owned())
                },
                can_restore: false,
                restore_disabled_reason: Some(
                    "Branch restore is handled by a later mutation phase.".to_owned(),
                ),
            },
            protection,
            name: short_name,
            qualified_name,
            classification,
            is_default,
            latest_commit,
            ahead,
            behind,
            pull_request,
            updated_at,
        });
    }

    let counts = RepositoryBranchClassificationCounts {
        default: all_rows.iter().filter(|row| row.is_default).count() as i64,
        active: all_rows
            .iter()
            .filter(|row| row.classification == "active")
            .count() as i64,
        stale: all_rows
            .iter()
            .filter(|row| row.classification == "stale")
            .count() as i64,
        all: all_rows.len() as i64,
        overview: all_rows
            .iter()
            .filter(|row| row.is_default || row.classification == "active")
            .count() as i64,
    };
    let default_branch = all_rows.iter().find(|row| row.is_default).cloned();
    let mut filtered = all_rows
        .into_iter()
        .filter(|row| {
            query_lower.is_empty()
                || row.name.to_lowercase().contains(&query_lower)
                || row.qualified_name.to_lowercase().contains(&query_lower)
        })
        .filter(|row| match tab.as_str() {
            "overview" => !row.is_default && row.classification == "active",
            "active" => row.classification == "active",
            "stale" => row.classification == "stale",
            "all" => true,
            _ => false,
        })
        .collect::<Vec<_>>();
    filtered.sort_by(|left, right| {
        if left.is_default != right.is_default {
            return left.is_default.cmp(&right.is_default).reverse();
        }
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    let total = filtered.len() as i64;
    let offset = ((page - 1) * page_size) as usize;
    let branches = filtered
        .into_iter()
        .skip(offset)
        .take(page_size as usize)
        .collect();

    record_branch_directory_visit(
        pool,
        repository.id,
        actor_user_id,
        &tab,
        normalized_query.as_deref().unwrap_or(""),
        page,
        page_size,
    )
    .await?;

    Ok(RepositoryBranchesView {
        repository: RepositoryBranchesRepository {
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            default_branch: repository.default_branch.clone(),
            visibility: repository.visibility.clone(),
            viewer_permission,
        },
        tabs: counts,
        filters: RepositoryBranchesFilters {
            tab: tab.clone(),
            query: normalized_query.clone(),
            stale_cutoff_days: STALE_CUTOFF_DAYS,
        },
        default_branch: if tab == "overview" {
            default_branch.filter(|row| {
                query_lower.is_empty()
                    || row.name.to_lowercase().contains(&query_lower)
                    || row.qualified_name.to_lowercase().contains(&query_lower)
            })
        } else {
            None
        },
        branches,
        total,
        page,
        page_size,
        has_next_page: offset as i64 + page_size < total,
        has_previous_page: page > 1,
        empty_state: RepositoryBranchesEmptyState {
            title: if normalized_query.is_some() {
                "No branches matched this search".to_owned()
            } else {
                "No branches in this view".to_owned()
            },
            message: "Adjust the branch tab or search query to recover the repository branch list."
                .to_owned(),
            reset_href: format!("/{}/{}/branches", repository.owner_login, repository.name),
        },
    })
}

async fn repository_branch_activity_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    branch: &str,
) -> Result<RepositoryBranchActivityView, RepositoryError> {
    let branch = branch.trim();
    if branch.is_empty() || branch.chars().count() > 120 || branch.starts_with("refs/") {
        return Err(RepositoryError::InvalidBranchDirectoryQuery(
            "branch name must be a short branch ref".to_owned(),
        ));
    }
    let branch = normalize_repository_path(branch)?;
    let directory = repository_branches_for_repository(
        pool,
        repository,
        actor_user_id,
        RepositoryBranchesQuery {
            tab: Some("all"),
            query: Some(&branch),
            page: 1,
            page_size: 100,
        },
    )
    .await?;
    let Some(branch_row) = directory
        .branches
        .into_iter()
        .chain(directory.default_branch)
        .find(|row| row.name == branch)
    else {
        return Err(RepositoryError::RefNotFoundWithRecovery {
            ref_name: branch.clone(),
            recovery_href: format!("/{}/{}/branches", repository.owner_login, repository.name),
            default_branch_href: repository_tree_href(repository, &repository.default_branch, ""),
        });
    };

    let resolved_ref = resolve_repository_ref(pool, repository, Some(&branch)).await?;
    let history = repository_commit_history(
        pool,
        repository,
        &resolved_ref,
        None,
        RepositoryCommitHistoryQuery {
            ref_name: Some(&branch),
            path: None,
            author: None,
            until: None,
            page: 1,
            page_size: 6,
        },
    )
    .await?;
    let recent_commits = history
        .groups
        .into_iter()
        .flat_map(|group| group.commits)
        .take(6)
        .collect();

    let recent_pull_requests = sqlx::query(
        r#"
        SELECT number, title, state, COALESCE(is_draft, false) AS draft
        FROM pull_requests
        WHERE repository_id = $1 AND head_ref = $2
        ORDER BY updated_at DESC, number DESC
        LIMIT 6
        "#,
    )
    .bind(repository.id)
    .bind(&branch)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| RepositoryBranchPullRequestSummary {
        number: row.get("number"),
        title: row.get("title"),
        state: row.get("state"),
        draft: row.get("draft"),
        href: format!(
            "/{}/{}/pull/{}",
            repository.owner_login,
            repository.name,
            row.get::<i64, _>("number")
        ),
    })
    .collect::<Vec<_>>();

    let can_admin = can_admin_repository(pool, repository, actor_user_id).await?;
    let branch_refs = repository_branch_ref_summaries(pool, repository.id).await?;
    let rules = repository_branch_rules(pool, repository.id, &branch_refs, can_admin).await?;
    let rulesets = repository_rulesets(pool, repository.id, &branch_refs, can_admin).await?;
    let mut protection_events = Vec::new();
    for rule in rules {
        if rule.enforcement != BranchPolicyEnforcement::Disabled
            && branch_pattern_matches(&rule.pattern, &branch)
        {
            protection_events.push(RepositoryBranchProtectionEvent {
                source_type: "rule".to_owned(),
                name: rule
                    .description
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| rule.pattern.clone()),
                enforcement: rule.enforcement,
                href: format!(
                    "/{}/{}/settings/branches?branch={}",
                    repository.owner_login,
                    repository.name,
                    percent_encode_segment(&branch)
                ),
                required_status_checks: rule.requirements.required_status_checks,
                updated_at: rule.updated_at,
            });
        }
    }
    for ruleset in rulesets {
        if ruleset.enforcement != BranchPolicyEnforcement::Disabled
            && ruleset
                .patterns
                .iter()
                .any(|pattern| branch_pattern_matches(pattern, &branch))
        {
            protection_events.push(RepositoryBranchProtectionEvent {
                source_type: "ruleset".to_owned(),
                name: ruleset.name,
                enforcement: ruleset.enforcement,
                href: format!(
                    "/{}/{}/settings/branches?branch={}",
                    repository.owner_login,
                    repository.name,
                    percent_encode_segment(&branch)
                ),
                required_status_checks: ruleset.requirements.required_status_checks,
                updated_at: ruleset.updated_at,
            });
        }
    }
    protection_events.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    Ok(RepositoryBranchActivityView {
        repository: directory.repository,
        branch: branch_row,
        recent_commits,
        recent_pull_requests,
        protection_events,
        links: RepositoryBranchActivityLinks {
            branches_href: format!("/{}/{}/branches", repository.owner_login, repository.name),
            tree_href: repository_tree_href(repository, &branch, ""),
            commits_href: repository_history_href(repository, &branch, ""),
            compare_href: format!(
                "/{}/{}/compare/{}...{}",
                repository.owner_login,
                repository.name,
                percent_encode_segment(&repository.default_branch),
                percent_encode_segment(&branch)
            ),
            rules_href: format!(
                "/{}/{}/settings/branches?branch={}",
                repository.owner_login,
                repository.name,
                percent_encode_segment(&branch)
            ),
        },
    })
}

fn branch_short_name(qualified_name: &str) -> String {
    qualified_name
        .strip_prefix("refs/heads/")
        .unwrap_or(qualified_name)
        .to_owned()
}

fn branch_protection_summary(
    repository: &Repository,
    branch_name: &str,
    rules: &[RepositoryBranchRule],
    rulesets: &[RepositoryRuleset],
) -> RepositoryBranchProtectionSummary {
    let mut required_status_checks = BTreeSet::new();
    let matching_rule_count = rules
        .iter()
        .filter(|rule| rule.enforcement != BranchPolicyEnforcement::Disabled)
        .filter(|rule| branch_pattern_matches(&rule.pattern, branch_name))
        .inspect(|rule| {
            required_status_checks.extend(rule.requirements.required_status_checks.iter().cloned());
        })
        .count() as i64;
    let matching_ruleset_count = rulesets
        .iter()
        .filter(|ruleset| ruleset.enforcement != BranchPolicyEnforcement::Disabled)
        .filter(|ruleset| {
            ruleset
                .patterns
                .iter()
                .any(|pattern| branch_pattern_matches(pattern, branch_name))
        })
        .inspect(|ruleset| {
            required_status_checks
                .extend(ruleset.requirements.required_status_checks.iter().cloned());
        })
        .count() as i64;
    RepositoryBranchProtectionSummary {
        protected: matching_rule_count + matching_ruleset_count > 0,
        matching_rule_count,
        matching_ruleset_count,
        required_status_checks: required_status_checks.into_iter().collect(),
        href: format!(
            "/{}/{}/settings/branches?branch={}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(branch_name)
        ),
    }
}

async fn branch_ahead_behind(
    pool: &PgPool,
    repository_id: Uuid,
    branch_oid: Option<&str>,
    default_oid: Option<&str>,
) -> Result<(i64, i64), RepositoryError> {
    let Some(branch_oid) = branch_oid else {
        return Ok((0, 0));
    };
    let Some(default_oid) = default_oid else {
        return Ok((0, 0));
    };
    if branch_oid == default_oid {
        return Ok((0, 0));
    }
    let row = sqlx::query(
        r#"
        WITH RECURSIVE branch_ancestors AS (
            SELECT c.oid, c.parent_oids
            FROM commits c
            WHERE c.repository_id = $1 AND c.oid = $2
            UNION
            SELECT parent.oid, parent.parent_oids
            FROM commits parent
            JOIN branch_ancestors child ON parent.oid = ANY(child.parent_oids)
            WHERE parent.repository_id = $1
        ),
        default_ancestors AS (
            SELECT c.oid, c.parent_oids
            FROM commits c
            WHERE c.repository_id = $1 AND c.oid = $3
            UNION
            SELECT parent.oid, parent.parent_oids
            FROM commits parent
            JOIN default_ancestors child ON parent.oid = ANY(child.parent_oids)
            WHERE parent.repository_id = $1
        )
        SELECT
            (SELECT count(*)::bigint FROM branch_ancestors WHERE oid NOT IN (SELECT oid FROM default_ancestors)) AS ahead,
            (SELECT count(*)::bigint FROM default_ancestors WHERE oid NOT IN (SELECT oid FROM branch_ancestors)) AS behind
        "#,
    )
    .bind(repository_id)
    .bind(branch_oid)
    .bind(default_oid)
    .fetch_one(pool)
    .await?;
    Ok((row.get("ahead"), row.get("behind")))
}

async fn record_branch_directory_visit(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
    tab: &str,
    query: &str,
    page: i64,
    page_size: i64,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO repository_branch_directory_recent_visits (
            repository_id, user_id, tab, query, page, page_size, viewed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, now())
        ON CONFLICT (repository_id, user_id, tab, query) DO UPDATE SET
            page = EXCLUDED.page,
            page_size = EXCLUDED.page_size,
            viewed_at = now()
        "#,
    )
    .bind(repository_id)
    .bind(user_id)
    .bind(tab)
    .bind(query)
    .bind(page)
    .bind(page_size)
    .execute(pool)
    .await?;
    Ok(())
}

async fn repository_pulse_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    query: RepositoryPulseQuery<'_>,
) -> Result<RepositoryPulseView, RepositoryError> {
    let period_key = normalize_pulse_period(query.period)?;
    let ended_at = Utc::now();
    let started_at = pulse_period_start(period_key, ended_at);
    let viewer_permission = viewer_permission_for_user(pool, repository, actor_user_id)
        .await?
        .unwrap_or_else(|| "read".to_owned());
    let cache_key = format!(
        "{}:{}:{}",
        period_key,
        started_at.format("%Y%m%d%H%M"),
        ended_at.format("%Y%m%d%H%M")
    );

    let aggregate = sqlx::query(
        r#"
        WITH period_commits AS (
            SELECT id, author_user_id
            FROM commits
            WHERE repository_id = $1
              AND committed_at >= $2
              AND committed_at <= $3
        ),
        issue_only AS (
            SELECT issues.*
            FROM issues
            LEFT JOIN pull_requests ON pull_requests.issue_id = issues.id
            WHERE issues.repository_id = $1 AND pull_requests.id IS NULL
        )
        SELECT
            (SELECT count(*)::bigint FROM period_commits) AS commits,
            (SELECT count(DISTINCT author_user_id)::bigint FROM period_commits WHERE author_user_id IS NOT NULL) AS authors,
            (SELECT count(DISTINCT commit_file_changes.path)::bigint
             FROM commit_file_changes
             JOIN period_commits ON period_commits.id = commit_file_changes.commit_id) AS files_changed,
            (SELECT COALESCE(sum(commit_file_changes.additions), 0)::bigint
             FROM commit_file_changes
             JOIN period_commits ON period_commits.id = commit_file_changes.commit_id) AS additions,
            (SELECT COALESCE(sum(commit_file_changes.deletions), 0)::bigint
             FROM commit_file_changes
             JOIN period_commits ON period_commits.id = commit_file_changes.commit_id) AS deletions,
            (SELECT count(*)::bigint FROM pull_requests
             WHERE repository_id = $1 AND state = 'merged' AND merged_at >= $2 AND merged_at <= $3) AS merged_pull_requests,
            (SELECT count(*)::bigint FROM pull_requests
             WHERE repository_id = $1 AND state = 'open') AS open_pull_requests,
            (SELECT count(*)::bigint FROM issue_only
             WHERE state = 'closed' AND closed_at >= $2 AND closed_at <= $3) AS closed_issues,
            (SELECT count(*)::bigint FROM issue_only
             WHERE created_at >= $2 AND created_at <= $3) AS new_issues,
            (SELECT count(*)::bigint FROM issue_only
             WHERE state = 'open') AS open_issues,
            (SELECT count(*)::bigint FROM releases
             WHERE repository_id = $1 AND draft = false AND published_at >= $2 AND published_at <= $3) AS releases
        "#,
    )
    .bind(repository.id)
    .bind(started_at)
    .bind(ended_at)
    .fetch_one(pool)
    .await?;

    let summary = RepositoryPulseSummary {
        commits: aggregate.get("commits"),
        authors: aggregate.get("authors"),
        files_changed: aggregate.get("files_changed"),
        additions: aggregate.get("additions"),
        deletions: aggregate.get("deletions"),
        merged_pull_requests: aggregate.get("merged_pull_requests"),
        open_pull_requests: aggregate.get("open_pull_requests"),
        closed_issues: aggregate.get("closed_issues"),
        new_issues: aggregate.get("new_issues"),
        open_issues: aggregate.get("open_issues"),
        releases: aggregate.get("releases"),
        sentence: String::new(),
    };
    let summary = RepositoryPulseSummary {
        sentence: pulse_summary_sentence(&summary, period_key),
        ..summary
    };

    let metrics = vec![
        RepositoryPulseMetric {
            key: "merged_pull_requests".to_owned(),
            label: "Merged pull requests".to_owned(),
            count: summary.merged_pull_requests,
            href: pulse_pull_requests_href(repository, "merged", started_at, ended_at),
        },
        RepositoryPulseMetric {
            key: "open_pull_requests".to_owned(),
            label: "Open pull requests".to_owned(),
            count: summary.open_pull_requests,
            href: format!(
                "/{}/{}/pulls?state=open",
                repository.owner_login, repository.name
            ),
        },
        RepositoryPulseMetric {
            key: "closed_issues".to_owned(),
            label: "Closed issues".to_owned(),
            count: summary.closed_issues,
            href: pulse_issues_href(repository, "closed", started_at, ended_at),
        },
        RepositoryPulseMetric {
            key: "new_issues".to_owned(),
            label: "New issues".to_owned(),
            count: summary.new_issues,
            href: pulse_issues_href(repository, "created", started_at, ended_at),
        },
    ];

    let top_committers =
        repository_pulse_top_committers(pool, repository, started_at, ended_at).await?;
    let releases = repository_pulse_releases(pool, repository, started_at, ended_at).await?;
    let merged_pull_requests =
        repository_pulse_merged_pull_requests(pool, repository, started_at, ended_at).await?;
    let issue_activity =
        repository_pulse_issue_activity(pool, repository, started_at, ended_at).await?;
    let snapshot = record_repository_pulse_snapshot(
        pool,
        repository.id,
        actor_user_id,
        period_key,
        &cache_key,
        &summary,
    )
    .await?;

    Ok(RepositoryPulseView {
        repository: RepositoryPulseRepository {
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            default_branch: repository.default_branch.clone(),
            visibility: repository.visibility.clone(),
            viewer_permission,
            href: format!("/{}/{}", repository.owner_login, repository.name),
        },
        period: RepositoryPulsePeriod {
            key: period_key.to_owned(),
            label: pulse_period_label(period_key).to_owned(),
            started_at,
            ended_at,
        },
        metrics,
        summary,
        top_committers,
        releases,
        merged_pull_requests,
        issue_activity,
        snapshot,
    })
}

fn normalize_pulse_period(period: Option<&str>) -> Result<&'static str, RepositoryError> {
    match period
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("1w")
    {
        "24h" => Ok("24h"),
        "3d" => Ok("3d"),
        "1w" => Ok("1w"),
        "1m" => Ok("1m"),
        other => Err(RepositoryError::InvalidPulseQuery(format!(
            "unsupported period `{other}`"
        ))),
    }
}

fn pulse_period_start(period_key: &str, ended_at: DateTime<Utc>) -> DateTime<Utc> {
    match period_key {
        "24h" => ended_at - chrono::Duration::hours(24),
        "3d" => ended_at - chrono::Duration::days(3),
        "1m" => ended_at - chrono::Duration::days(30),
        _ => ended_at - chrono::Duration::days(7),
    }
}

fn pulse_period_label(period_key: &str) -> &'static str {
    match period_key {
        "24h" => "Last 24 hours",
        "3d" => "Last 3 days",
        "1m" => "Last month",
        _ => "Last week",
    }
}

fn pulse_summary_sentence(summary: &RepositoryPulseSummary, period_key: &str) -> String {
    format!(
        "{} authors pushed {} commits touching {} files with {} additions and {} deletions in the {} window.",
        summary.authors,
        summary.commits,
        summary.files_changed,
        summary.additions,
        summary.deletions,
        period_key
    )
}

async fn repository_pulse_top_committers(
    pool: &PgPool,
    repository: &Repository,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
) -> Result<Vec<RepositoryPulseCommitter>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT commits.author_user_id,
               NULLIF(users.username, '') AS username,
               users.email,
               users.avatar_url,
               count(DISTINCT commits.id)::bigint AS commits,
               count(DISTINCT commit_file_changes.path)::bigint AS files_changed,
               COALESCE(sum(commit_file_changes.additions), 0)::bigint AS additions,
               COALESCE(sum(commit_file_changes.deletions), 0)::bigint AS deletions
        FROM commits
        LEFT JOIN users ON users.id = commits.author_user_id
        LEFT JOIN commit_file_changes ON commit_file_changes.commit_id = commits.id
        WHERE commits.repository_id = $1
          AND commits.committed_at >= $2
          AND commits.committed_at <= $3
        GROUP BY commits.author_user_id, users.username, users.email, users.avatar_url
        ORDER BY count(DISTINCT commits.id) DESC,
                 COALESCE(sum(commit_file_changes.additions), 0) DESC,
                 lower(COALESCE(NULLIF(users.username, ''), users.email, 'unmatched author')) ASC
        LIMIT 10
        "#,
    )
    .bind(repository.id)
    .bind(started_at)
    .bind(ended_at)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let user_id: Option<Uuid> = row.get("author_user_id");
            let username: Option<String> = row.get("username");
            let email: Option<String> = row.get("email");
            let login = username
                .or(email)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "Unmatched author".to_owned());
            let is_bot = pulse_login_is_bot(&login);
            let author_status = if user_id.is_none() {
                "unmatched"
            } else if is_bot {
                "bot"
            } else {
                "active"
            };
            let commits_href = if user_id.is_some() {
                format!(
                    "/{}/{}/commits/{}?author={}&until={}",
                    repository.owner_login,
                    repository.name,
                    percent_encode_segment(&repository.default_branch),
                    percent_encode_segment(&login),
                    percent_encode_segment(&ended_at.to_rfc3339())
                )
            } else {
                format!(
                    "/{}/{}/commits/{}?until={}",
                    repository.owner_login,
                    repository.name,
                    percent_encode_segment(&repository.default_branch),
                    percent_encode_segment(&ended_at.to_rfc3339())
                )
            };
            RepositoryPulseCommitter {
                user_id,
                profile_href: if user_id.is_some() {
                    format!("/{login}")
                } else {
                    format!("/{}/{}", repository.owner_login, repository.name)
                },
                commits_href,
                login,
                author_status: author_status.to_owned(),
                is_bot,
                avatar_url: row.get("avatar_url"),
                commits: row.get("commits"),
                files_changed: row.get("files_changed"),
                additions: row.get("additions"),
                deletions: row.get("deletions"),
            }
        })
        .collect())
}

fn pulse_login_is_bot(login: &str) -> bool {
    let normalized = login.trim().to_ascii_lowercase();
    normalized.ends_with("[bot]")
        || normalized.ends_with("-bot")
        || normalized.contains(" bot")
        || normalized.contains("automation")
}

async fn repository_pulse_releases(
    pool: &PgPool,
    repository: &Repository,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
) -> Result<Vec<RepositoryPulseActivityItem>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT releases.tag_name,
               COALESCE(NULLIF(releases.name, ''), releases.tag_name) AS title,
               releases.prerelease,
               releases.published_at,
               COALESCE(NULLIF(users.username, ''), users.email) AS author_login,
               users.avatar_url AS author_avatar_url
        FROM releases
        LEFT JOIN users ON users.id = releases.author_user_id
        WHERE releases.repository_id = $1
          AND releases.draft = false
          AND releases.published_at >= $2
          AND releases.published_at <= $3
        ORDER BY releases.published_at DESC, lower(releases.tag_name)
        LIMIT 10
        "#,
    )
    .bind(repository.id)
    .bind(started_at)
    .bind(ended_at)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let tag: String = row.get("tag_name");
            let author_login: Option<String> = row.get("author_login");
            let author_status = pulse_author_status(author_login.as_deref());
            RepositoryPulseActivityItem {
                kind: "release".to_owned(),
                number: None,
                title: row.get("title"),
                state: if row.get::<bool, _>("prerelease") {
                    "prerelease".to_owned()
                } else {
                    "published".to_owned()
                },
                author_profile_href: author_login.as_ref().map(|login| format!("/{login}")),
                author_status,
                author_login,
                author_avatar_url: row.get("author_avatar_url"),
                href: format!(
                    "/{}/{}/releases/tag/{}",
                    repository.owner_login,
                    repository.name,
                    percent_encode_segment(&tag)
                ),
                occurred_at: row
                    .get::<Option<DateTime<Utc>>, _>("published_at")
                    .unwrap_or_else(Utc::now),
            }
        })
        .collect())
}

async fn repository_pulse_merged_pull_requests(
    pool: &PgPool,
    repository: &Repository,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
) -> Result<Vec<RepositoryPulseActivityItem>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT pull_requests.number,
               pull_requests.title,
               pull_requests.state,
               pull_requests.merged_at,
               COALESCE(NULLIF(users.username, ''), users.email) AS author_login,
               users.avatar_url AS author_avatar_url
        FROM pull_requests
        LEFT JOIN users ON users.id = pull_requests.author_user_id
        WHERE pull_requests.repository_id = $1
          AND pull_requests.state = 'merged'
          AND pull_requests.merged_at >= $2
          AND pull_requests.merged_at <= $3
        ORDER BY pull_requests.merged_at DESC, pull_requests.number DESC
        LIMIT 10
        "#,
    )
    .bind(repository.id)
    .bind(started_at)
    .bind(ended_at)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let number: i64 = row.get("number");
            let author_login: Option<String> = row.get("author_login");
            let author_status = pulse_author_status(author_login.as_deref());
            RepositoryPulseActivityItem {
                kind: "pull_request".to_owned(),
                number: Some(number),
                title: row.get("title"),
                state: row.get("state"),
                author_profile_href: author_login.as_ref().map(|login| format!("/{login}")),
                author_status,
                author_login,
                author_avatar_url: row.get("author_avatar_url"),
                href: format!(
                    "/{}/{}/pull/{number}",
                    repository.owner_login, repository.name
                ),
                occurred_at: row
                    .get::<Option<DateTime<Utc>>, _>("merged_at")
                    .unwrap_or_else(Utc::now),
            }
        })
        .collect())
}

async fn repository_pulse_issue_activity(
    pool: &PgPool,
    repository: &Repository,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
) -> Result<Vec<RepositoryPulseActivityItem>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT issues.number,
               issues.title,
               issues.state,
               COALESCE(issues.closed_at, issues.created_at) AS occurred_at,
               COALESCE(NULLIF(users.username, ''), users.email) AS author_login,
               users.avatar_url AS author_avatar_url
        FROM issues
        LEFT JOIN pull_requests ON pull_requests.issue_id = issues.id
        LEFT JOIN users ON users.id = issues.author_user_id
        WHERE issues.repository_id = $1
          AND pull_requests.id IS NULL
          AND (
            (issues.created_at >= $2 AND issues.created_at <= $3)
            OR (issues.closed_at >= $2 AND issues.closed_at <= $3)
          )
        ORDER BY occurred_at DESC, issues.number DESC
        LIMIT 10
        "#,
    )
    .bind(repository.id)
    .bind(started_at)
    .bind(ended_at)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let number: i64 = row.get("number");
            let author_login: Option<String> = row.get("author_login");
            let author_status = pulse_author_status(author_login.as_deref());
            RepositoryPulseActivityItem {
                kind: "issue".to_owned(),
                number: Some(number),
                title: row.get("title"),
                state: row.get("state"),
                author_profile_href: author_login.as_ref().map(|login| format!("/{login}")),
                author_status,
                author_login,
                author_avatar_url: row.get("author_avatar_url"),
                href: format!(
                    "/{}/{}/issues/{number}",
                    repository.owner_login, repository.name
                ),
                occurred_at: row
                    .get::<Option<DateTime<Utc>>, _>("occurred_at")
                    .unwrap_or_else(Utc::now),
            }
        })
        .collect())
}

fn pulse_author_status(author_login: Option<&str>) -> String {
    match author_login {
        Some(login) if pulse_login_is_bot(login) => "bot".to_owned(),
        Some(_) => "active".to_owned(),
        None => "unavailable".to_owned(),
    }
}

async fn record_repository_pulse_snapshot(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
    period_key: &str,
    cache_key: &str,
    summary: &RepositoryPulseSummary,
) -> Result<RepositoryPulseSnapshot, RepositoryError> {
    let snapshot = serde_json::to_value(summary).unwrap_or_else(|_| json!({}));
    let row = sqlx::query(
        r#"
        INSERT INTO repository_insight_snapshots (
            repository_id, period_key, cache_key, snapshot, computed_at, expires_at
        )
        VALUES ($1, $2, $3, $4, now(), now() + interval '10 minutes')
        ON CONFLICT (repository_id, period_key, cache_key) DO UPDATE SET
            snapshot = EXCLUDED.snapshot,
            computed_at = now(),
            expires_at = now() + interval '10 minutes'
        RETURNING computed_at, expires_at
        "#,
    )
    .bind(repository_id)
    .bind(period_key)
    .bind(cache_key)
    .bind(Json(snapshot))
    .fetch_one(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO recent_insight_views (repository_id, user_id, period_key, viewed_at)
        VALUES ($1, $2, $3, now())
        ON CONFLICT (repository_id, user_id, period_key)
        DO UPDATE SET viewed_at = now()
        "#,
    )
    .bind(repository_id)
    .bind(user_id)
    .bind(period_key)
    .execute(pool)
    .await?;

    let expires_at: DateTime<Utc> = row.get("expires_at");
    Ok(RepositoryPulseSnapshot {
        cache_key: cache_key.to_owned(),
        computed_at: row.get("computed_at"),
        expires_at,
        stale: expires_at <= Utc::now(),
    })
}

fn pulse_pull_requests_href(
    repository: &Repository,
    state: &str,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
) -> String {
    format!(
        "/{}/{}/pulls?state={state}&since={}&until={}",
        repository.owner_login,
        repository.name,
        started_at.to_rfc3339(),
        ended_at.to_rfc3339()
    )
}

fn pulse_issues_href(
    repository: &Repository,
    state: &str,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
) -> String {
    let (state, sort) = if state == "created" {
        ("open", "&sort=created-desc")
    } else {
        (state, "")
    };
    format!(
        "/{}/{}/issues?state={state}{sort}&since={}&until={}",
        repository.owner_login,
        repository.name,
        started_at.to_rfc3339(),
        ended_at.to_rfc3339()
    )
}

async fn repository_commit_detail(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    sha: &str,
) -> Result<RepositoryCommitDetailView, RepositoryError> {
    let normalized_sha = sha.trim();
    if normalized_sha.len() < 4 || normalized_sha.contains('/') || normalized_sha.contains('\\') {
        return Err(RepositoryError::NotFound);
    }

    let rows = sqlx::query(
        r#"
        SELECT commits.id,
               commits.oid,
               commits.message,
               commits.parent_oids,
               commits.committed_at,
               COALESCE(NULLIF(author.username, ''), author.email) AS author_login,
               author.avatar_url AS author_avatar_url,
               COALESCE(NULLIF(committer.username, ''), committer.email) AS committer_login,
               committer.avatar_url AS committer_avatar_url,
               commits.author_user_id,
               commits.signature_fingerprint,
               commits.signature_summary,
               COALESCE(statuses.status, workflow_statuses.status, 'pending') AS status,
               COALESCE(statuses.conclusion, workflow_statuses.conclusion) AS conclusion,
               COALESCE(statuses.total_count, workflow_statuses.total_count, 0)::bigint AS total_count,
               COALESCE(statuses.completed_count, workflow_statuses.completed_count, 0)::bigint AS completed_count,
               COALESCE(statuses.failed_count, workflow_statuses.failed_count, 0)::bigint AS failed_count,
               COALESCE(pr_links.links, '[]'::jsonb) AS pull_requests
        FROM commits
        LEFT JOIN users author ON author.id = commits.author_user_id
        LEFT JOIN users committer ON committer.id = commits.committer_user_id
        LEFT JOIN repository_commit_status_summaries statuses ON statuses.commit_id = commits.id
        LEFT JOIN LATERAL (
            SELECT CASE
                    WHEN count(*) FILTER (WHERE workflow_runs.status IN ('queued', 'in_progress')) > 0 THEN 'running'
                    WHEN count(*) = 0 THEN NULL
                    ELSE 'completed'
                   END AS status,
                   CASE
                    WHEN count(*) = 0 THEN NULL
                    WHEN count(*) FILTER (WHERE workflow_runs.conclusion = 'failure') > 0 THEN 'failure'
                    WHEN count(*) FILTER (WHERE workflow_runs.conclusion = 'cancelled') > 0 THEN 'cancelled'
                    WHEN count(*) FILTER (WHERE workflow_runs.conclusion = 'success') = count(*) THEN 'success'
                    ELSE NULL
                   END AS conclusion,
                   count(*)::bigint AS total_count,
                   count(*) FILTER (WHERE workflow_runs.status = 'completed')::bigint AS completed_count,
                   count(*) FILTER (WHERE workflow_runs.conclusion = 'failure')::bigint AS failed_count
            FROM workflow_runs
            WHERE workflow_runs.repository_id = commits.repository_id
              AND (workflow_runs.commit_id = commits.id OR workflow_runs.head_sha = commits.oid)
        ) workflow_statuses ON true
        LEFT JOIN LATERAL (
            SELECT jsonb_agg(
                jsonb_build_object(
                    'number', pull_requests.number,
                    'title', pull_requests.title,
                    'href', format('/%s/%s/pull/%s', $3::text, $4::text, pull_requests.number),
                    'state', pull_requests.state
                )
                ORDER BY pull_requests.number
            ) AS links
            FROM pull_request_commits
            JOIN pull_requests ON pull_requests.id = pull_request_commits.pull_request_id
            WHERE pull_request_commits.commit_id = commits.id
        ) pr_links ON true
        WHERE commits.repository_id = $1
          AND (commits.oid = $2 OR commits.oid LIKE ($2 || '%'))
        ORDER BY (commits.oid = $2) DESC, commits.committed_at DESC
        LIMIT 2
        "#,
    )
    .bind(repository.id)
    .bind(normalized_sha)
    .bind(&repository.owner_login)
    .bind(&repository.name)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() || (rows.len() > 1 && rows[0].get::<String, _>("oid") != normalized_sha) {
        return Err(RepositoryError::NotFound);
    }

    let row = &rows[0];
    let commit_id: Uuid = row.get("id");
    let oid: String = row.get("oid");
    let message: String = row.get("message");
    let (subject, body) = split_commit_message(&message);
    let signature = super::signing_keys::signature_presentation_for_user(
        pool,
        row.get("author_user_id"),
        row.get::<Option<String>, _>("signature_fingerprint")
            .as_deref(),
        row.get::<Option<String>, _>("signature_summary").as_deref(),
    )
    .await
    .map_err(|error| match error {
        super::signing_keys::SigningKeyError::Sqlx(error) => RepositoryError::Sqlx(error),
        _ => RepositoryError::GitStorageFailed,
    })?;
    let pull_requests: Vec<RepositoryCommitPullRequestLink> =
        serde_json::from_value(row.get("pull_requests")).unwrap_or_default();
    let parent_oids: Vec<String> = row.get("parent_oids");
    let parents = parent_oids
        .into_iter()
        .map(|parent_oid| RepositoryCommitDetailParent {
            short_oid: parent_oid.chars().take(7).collect(),
            href: format!(
                "/{}/{}/commit/{}",
                repository.owner_login, repository.name, parent_oid
            ),
            oid: parent_oid,
        })
        .collect::<Vec<_>>();
    let branch_rows = sqlx::query(
        r#"
        SELECT name, kind
        FROM repository_git_refs
        WHERE repository_id = $1 AND target_commit_id = $2
        ORDER BY CASE WHEN name = $3 THEN 0 ELSE 1 END, lower(name)
        "#,
    )
    .bind(repository.id)
    .bind(commit_id)
    .bind(format!("refs/heads/{}", repository.default_branch))
    .fetch_all(pool)
    .await?;
    let branches = branch_rows
        .into_iter()
        .map(|branch| {
            let qualified_name: String = branch.get("name");
            let name = qualified_name
                .strip_prefix("refs/heads/")
                .or_else(|| qualified_name.strip_prefix("refs/tags/"))
                .unwrap_or(&qualified_name)
                .to_owned();
            RepositoryCommitDetailBranchLink {
                href: repository_history_href(repository, &name, ""),
                name,
                qualified_name,
                kind: branch.get("kind"),
            }
        })
        .collect::<Vec<_>>();

    let _ = sqlx::query(
        r#"
        INSERT INTO repository_commit_recent_visits (repository_id, user_id, ref_name, path, filters)
        VALUES ($1, $2, $3, '', jsonb_build_object('commit', $3::text))
        ON CONFLICT (repository_id, user_id, ref_name, path)
        DO UPDATE SET viewed_at = now(), filters = EXCLUDED.filters
        "#,
    )
    .bind(repository.id)
    .bind(actor_user_id)
    .bind(&oid)
    .execute(pool)
    .await;

    let diff_files = repository_commit_detail_files(
        pool,
        repository,
        commit_id,
        parents.first().map(|parent| parent.oid.as_str()),
        &oid,
    )
    .await?;
    let diff_summary = RepositoryCommitDetailDiffSummary {
        total_files: diff_files.len() as i64,
        additions: diff_files.iter().map(|file| file.additions).sum(),
        deletions: diff_files.iter().map(|file| file.deletions).sum(),
    };
    let file_tree = diff_files
        .iter()
        .map(|file| RepositoryCommitDetailFileTreeNode {
            path: file.path.clone(),
            name: file
                .path
                .rsplit('/')
                .next()
                .unwrap_or(&file.path)
                .to_owned(),
            depth: file.path.matches('/').count() as i64,
            status: file.status.clone(),
            additions: file.additions,
            deletions: file.deletions,
            href: format!("#{}", file.anchor),
        })
        .collect::<Vec<_>>();

    Ok(RepositoryCommitDetailView {
        repository: RepositoryCommitDetailRepository {
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            default_branch: repository.default_branch.clone(),
            visibility: repository.visibility.clone(),
            href: format!("/{}/{}", repository.owner_login, repository.name),
            commit_history_href: repository_history_href(
                repository,
                &repository.default_branch,
                "",
            ),
        },
        commit: RepositoryCommitDetailCommit {
            short_oid: oid.chars().take(7).collect(),
            href: format!(
                "/{}/{}/commit/{}",
                repository.owner_login, repository.name, oid
            ),
            browse_href: format!(
                "/{}/{}/tree/{}",
                repository.owner_login, repository.name, oid
            ),
            oid: oid.clone(),
            message,
            subject,
            body,
            committed_at: row.get("committed_at"),
            author_login: row.get("author_login"),
            author_avatar_url: row.get("author_avatar_url"),
            committer_login: row.get("committer_login"),
            committer_avatar_url: row.get("committer_avatar_url"),
        },
        parents,
        branches,
        pull_requests,
        status: RepositoryCommitStatusSummary {
            status: row.get("status"),
            conclusion: row.get("conclusion"),
            total_count: row.get("total_count"),
            completed_count: row.get("completed_count"),
            failed_count: row.get("failed_count"),
            href: format!(
                "/{}/{}/actions?commit={}",
                repository.owner_login, repository.name, oid
            ),
        },
        verification: RepositoryCommitVerificationSummary {
            verified: signature.verified,
            signature_state: signature.signature_state,
            signature_summary: signature.signature_summary,
        },
        diff_placeholder: RepositoryCommitDetailDiffPlaceholder {
            state: if diff_summary.total_files == 0 {
                "empty".to_owned()
            } else {
                "ready".to_owned()
            },
            message: if diff_summary.total_files == 0 {
                "No file changes were recorded for this commit.".to_owned()
            } else {
                "Diff file tree and unified rows are available.".to_owned()
            },
            next_phase: "Phase 3: Diff Filter, In-Page Search, and Focus Behavior".to_owned(),
        },
        diff_summary,
        file_tree,
        files: diff_files,
    })
}

async fn repository_commit_detail_context(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    sha: &str,
    query: RepositoryCommitDetailContextQuery<'_>,
) -> Result<RepositoryCommitDetailContext, RepositoryError> {
    let normalized_path = normalize_repository_path(query.path)?;
    if normalized_path.is_empty() {
        return Err(RepositoryError::InvalidDiffContext(
            "path is required".to_owned(),
        ));
    }
    let normalized_hunk_id = query.hunk_id.trim();
    if normalized_hunk_id.is_empty() || normalized_hunk_id.len() > 180 {
        return Err(RepositoryError::InvalidDiffContext(
            "hunkId is invalid".to_owned(),
        ));
    }

    let detail = repository_commit_detail(pool, repository, actor_user_id, sha).await?;
    let file = detail
        .files
        .iter()
        .find(|file| file.path == normalized_path)
        .ok_or(RepositoryError::PathNotFound)?;
    if file.is_binary || file.is_large {
        return Err(RepositoryError::InvalidDiffContext(
            "binary and large file diffs cannot expand inline context".to_owned(),
        ));
    }
    let hunk = file
        .hunks
        .iter()
        .find(|hunk| hunk.id == normalized_hunk_id)
        .ok_or_else(|| RepositoryError::InvalidDiffContext("hunk was not found".to_owned()))?;

    let bounded_lines =
        bounded_commit_detail_context_lines(&hunk.lines, query.context_lines.clamp(3, 200));
    Ok(RepositoryCommitDetailContext {
        path: file.path.clone(),
        hunk_id: hunk.id.clone(),
        lines: bounded_lines,
        expanded: true,
        message: "Expanded context lines loaded.".to_owned(),
    })
}

fn bounded_commit_detail_context_lines(
    lines: &[RepositoryCommitDetailLine],
    context_lines: i64,
) -> Vec<RepositoryCommitDetailLine> {
    if lines.len() <= 400 {
        return lines.to_vec();
    }
    let context = context_lines.max(3) as usize;
    let mut keep = BTreeSet::new();
    for (index, line) in lines.iter().enumerate() {
        if line.kind == "added" || line.kind == "removed" {
            let start = index.saturating_sub(context);
            let end = (index + context + 1).min(lines.len());
            keep.extend(start..end);
        }
    }
    if keep.is_empty() {
        keep.extend(0..lines.len().min(400));
    }
    keep.into_iter()
        .take(400)
        .filter_map(|index| lines.get(index).cloned())
        .collect()
}

#[derive(Debug, Clone)]
struct CommitDetailSnapshotFile {
    content: String,
    oid: String,
    byte_size: i64,
}

async fn repository_commit_detail_files(
    pool: &PgPool,
    repository: &Repository,
    commit_id: Uuid,
    parent_oid: Option<&str>,
    commit_oid: &str,
) -> Result<Vec<RepositoryCommitDetailFile>, RepositoryError> {
    let current_files = commit_detail_snapshot_files(pool, repository.id, commit_id).await?;
    let parent_files = if let Some(parent_oid) = parent_oid {
        if let Some(parent_commit_id) = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM commits WHERE repository_id = $1 AND oid = $2",
        )
        .bind(repository.id)
        .bind(parent_oid)
        .fetch_optional(pool)
        .await?
        {
            commit_detail_snapshot_files(pool, repository.id, parent_commit_id).await?
        } else {
            BTreeMap::new()
        }
    } else {
        BTreeMap::new()
    };

    let added_paths = current_files
        .keys()
        .filter(|path| !parent_files.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    let removed_paths = parent_files
        .keys()
        .filter(|path| !current_files.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    let mut renamed_paths = BTreeMap::new();
    let mut consumed_removed = BTreeSet::new();
    for added_path in &added_paths {
        let Some(current_file) = current_files.get(added_path) else {
            continue;
        };
        if let Some(removed_path) = removed_paths.iter().find(|removed_path| {
            !consumed_removed.contains(*removed_path)
                && parent_files
                    .get(*removed_path)
                    .map(|file| file.oid.as_str())
                    == Some(current_file.oid.as_str())
        }) {
            renamed_paths.insert(added_path.clone(), removed_path.clone());
            consumed_removed.insert(removed_path.clone());
        }
    }

    let mut paths = current_files.keys().cloned().collect::<BTreeSet<_>>();
    paths.extend(parent_files.keys().cloned());
    for removed_path in &consumed_removed {
        paths.remove(removed_path);
    }
    let mut files = Vec::new();
    for path in paths {
        let current = current_files.get(&path);
        let previous_path = renamed_paths.get(&path).cloned();
        let parent = previous_path
            .as_ref()
            .and_then(|renamed_path| parent_files.get(renamed_path))
            .or_else(|| parent_files.get(&path));
        if current.map(|file| file.oid.as_str()) == parent.map(|file| file.oid.as_str())
            && previous_path.is_none()
        {
            continue;
        }
        let status = match (parent, current) {
            (Some(_), Some(_)) if previous_path.is_some() => "renamed",
            (None, Some(_)) => "added",
            (Some(_), None) => "removed",
            (Some(_), Some(_)) => "modified",
            (None, None) => continue,
        }
        .to_owned();
        let old_content = parent.map(|file| file.content.as_str()).unwrap_or("");
        let new_content = current.map(|file| file.content.as_str()).unwrap_or("");
        let is_binary = old_content.contains('\0') || new_content.contains('\0');
        let byte_size = current.or(parent).map(|file| file.byte_size).unwrap_or(0);
        let is_large = byte_size > 200_000;
        let (hunks, additions, deletions) = if is_binary || is_large {
            (Vec::new(), 0, 0)
        } else {
            commit_detail_hunks_for_file(&path, old_content, new_content)
        };
        let anchor = commit_detail_anchor_for_path(&path);
        let encoded_path = percent_encode_path(&path);
        let view_href = format!(
            "/{}/{}/blob/{}/{}",
            repository.owner_login, repository.name, commit_oid, encoded_path
        );
        files.push(RepositoryCommitDetailFile {
            path: path.clone(),
            previous_path,
            status,
            additions,
            deletions,
            byte_size,
            blob_oid: current.or(parent).map(|file| file.oid.clone()),
            language: language_for_path(&path),
            href: format!(
                "/{}/{}/commit/{}#{}",
                repository.owner_login, repository.name, commit_oid, anchor
            ),
            raw_href: format!(
                "/{}/{}/raw/{}/{}",
                repository.owner_login, repository.name, commit_oid, encoded_path
            ),
            view_href,
            anchor,
            is_binary,
            is_large,
            hunks,
        });
    }
    Ok(files)
}

async fn commit_detail_snapshot_files(
    pool: &PgPool,
    repository_id: Uuid,
    commit_id: Uuid,
) -> Result<BTreeMap<String, CommitDetailSnapshotFile>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT path, content, oid, byte_size
        FROM repository_files
        WHERE repository_id = $1 AND commit_id = $2
        ORDER BY lower(path)
        "#,
    )
    .bind(repository_id)
    .bind(commit_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let path: String = row.get("path");
            (
                path.clone(),
                CommitDetailSnapshotFile {
                    content: row.get("content"),
                    oid: row.get("oid"),
                    byte_size: row.get("byte_size"),
                },
            )
        })
        .collect())
}

fn commit_detail_hunks_for_file(
    path: &str,
    old_content: &str,
    new_content: &str,
) -> (Vec<RepositoryCommitDetailHunk>, i64, i64) {
    let old_lines = old_content.lines().collect::<Vec<_>>();
    let new_lines = new_content.lines().collect::<Vec<_>>();
    let mut lines = Vec::new();
    let mut old_line = 1_i64;
    let mut new_line = 1_i64;
    let mut position = 1_i64;
    let mut additions = 0_i64;
    let mut deletions = 0_i64;
    let max_len = old_lines.len().max(new_lines.len());
    for index in 0..max_len {
        match (old_lines.get(index), new_lines.get(index)) {
            (Some(old), Some(new)) if old == new => {
                lines.push(RepositoryCommitDetailLine {
                    kind: "context".to_owned(),
                    old_line: Some(old_line),
                    new_line: Some(new_line),
                    content: (*old).to_owned(),
                    position,
                });
                old_line += 1;
                new_line += 1;
                position += 1;
            }
            (Some(old), Some(new)) => {
                lines.push(RepositoryCommitDetailLine {
                    kind: "removed".to_owned(),
                    old_line: Some(old_line),
                    new_line: None,
                    content: (*old).to_owned(),
                    position,
                });
                old_line += 1;
                position += 1;
                deletions += 1;
                lines.push(RepositoryCommitDetailLine {
                    kind: "added".to_owned(),
                    old_line: None,
                    new_line: Some(new_line),
                    content: (*new).to_owned(),
                    position,
                });
                new_line += 1;
                position += 1;
                additions += 1;
            }
            (Some(old), None) => {
                lines.push(RepositoryCommitDetailLine {
                    kind: "removed".to_owned(),
                    old_line: Some(old_line),
                    new_line: None,
                    content: (*old).to_owned(),
                    position,
                });
                old_line += 1;
                position += 1;
                deletions += 1;
            }
            (None, Some(new)) => {
                lines.push(RepositoryCommitDetailLine {
                    kind: "added".to_owned(),
                    old_line: None,
                    new_line: Some(new_line),
                    content: (*new).to_owned(),
                    position,
                });
                new_line += 1;
                position += 1;
                additions += 1;
            }
            (None, None) => {}
        }
    }
    if lines.is_empty() {
        return (Vec::new(), additions, deletions);
    }
    let header = format!(
        "@@ -1,{} +1,{} @@ {}",
        old_lines.len(),
        new_lines.len(),
        path
    );
    (
        vec![RepositoryCommitDetailHunk {
            id: format!("{}-hunk-1", commit_detail_anchor_for_path(path)),
            header,
            old_start: 1,
            old_lines: old_lines.len() as i64,
            new_start: 1,
            new_lines: new_lines.len() as i64,
            lines,
        }],
        additions,
        deletions,
    )
}

fn commit_detail_anchor_for_path(path: &str) -> String {
    let mut output = String::from("diff-");
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() {
            output.push(byte as char);
        } else {
            output.push('-');
        }
    }
    output.trim_end_matches('-').to_owned()
}

fn split_commit_message(message: &str) -> (String, Option<String>) {
    let mut parts = message.splitn(2, '\n');
    let subject = parts.next().unwrap_or("").trim().to_owned();
    let body = parts
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    (subject, body)
}

fn group_commit_history_items(items: Vec<RepositoryCommitListItem>) -> Vec<RepositoryCommitGroup> {
    let mut groups: Vec<RepositoryCommitGroup> = Vec::new();
    for item in items {
        let date = item.committed_at.date_naive().to_string();
        if let Some(group) = groups.last_mut().filter(|group| group.date == date) {
            group.commits.push(item);
        } else {
            groups.push(RepositoryCommitGroup {
                date,
                commits: vec![item],
            });
        }
    }
    groups
}

async fn upsert_repository_search_index(
    pool: &PgPool,
    repository: &Repository,
    commit: &Commit,
    files: &[IndexedSearchFile],
) -> Result<(), RepositoryError> {
    let author_login = if let Some(author_user_id) = commit.author_user_id {
        sqlx::query_scalar::<_, String>(
            "SELECT COALESCE(NULLIF(username, ''), email) FROM users WHERE id = $1",
        )
        .bind(author_user_id)
        .fetch_optional(pool)
        .await?
    } else {
        None
    };
    let href = format!(
        "/{}/{}/commit/{}",
        percent_encode_segment(&repository.owner_login),
        percent_encode_segment(&repository.name),
        percent_encode_segment(&commit.oid)
    );

    sqlx::query(
        r#"
        INSERT INTO search_documents (
            repository_id,
            owner_user_id,
            owner_organization_id,
            kind,
            resource_id,
            title,
            body,
            visibility,
            metadata,
            indexed_at
        )
        VALUES ($1, $2, $3, 'commit', $4, $5, $6, $7, $8, now())
        ON CONFLICT (kind, resource_id) DO UPDATE SET
            repository_id = EXCLUDED.repository_id,
            owner_user_id = EXCLUDED.owner_user_id,
            owner_organization_id = EXCLUDED.owner_organization_id,
            title = EXCLUDED.title,
            body = EXCLUDED.body,
            visibility = EXCLUDED.visibility,
            metadata = EXCLUDED.metadata,
            indexed_at = now()
        "#,
    )
    .bind(repository.id)
    .bind(repository.owner_user_id)
    .bind(repository.owner_organization_id)
    .bind(&commit.oid)
    .bind(commit.message.lines().next().unwrap_or(&commit.message))
    .bind(&commit.message)
    .bind(repository.visibility.as_str())
    .bind(json!({
        "href": href,
        "ownerLogin": repository.owner_login,
        "repositoryName": repository.name,
        "authorLogin": author_login,
        "committedAt": commit.committed_at,
    }))
    .execute(pool)
    .await?;

    sqlx::query(
        "DELETE FROM search_documents WHERE repository_id = $1 AND kind = 'code' AND branch = $2",
    )
    .bind(repository.id)
    .bind(&repository.default_branch)
    .execute(pool)
    .await?;

    for file in files {
        let Some((line_number, fragment)) = first_searchable_line(&file.content) else {
            continue;
        };
        let href = format!(
            "/{}/{}/blob/{}/{}#L{}",
            percent_encode_segment(&repository.owner_login),
            percent_encode_segment(&repository.name),
            percent_encode_segment(&repository.default_branch),
            percent_encode_path(&file.path),
            line_number
        );
        sqlx::query(
            r#"
            INSERT INTO search_documents (
                repository_id,
                owner_user_id,
                owner_organization_id,
                kind,
                resource_id,
                title,
                body,
                path,
                language,
                branch,
                visibility,
                metadata,
                indexed_at
            )
            VALUES ($1, $2, $3, 'code', $4, $5, $6, $7, $8, $9, $10, $11, now())
            ON CONFLICT (kind, resource_id) DO UPDATE SET
                repository_id = EXCLUDED.repository_id,
                owner_user_id = EXCLUDED.owner_user_id,
                owner_organization_id = EXCLUDED.owner_organization_id,
                title = EXCLUDED.title,
                body = EXCLUDED.body,
                path = EXCLUDED.path,
                language = EXCLUDED.language,
                branch = EXCLUDED.branch,
                visibility = EXCLUDED.visibility,
                metadata = EXCLUDED.metadata,
                indexed_at = now()
            "#,
        )
        .bind(repository.id)
        .bind(repository.owner_user_id)
        .bind(repository.owner_organization_id)
        .bind(format!(
            "{}:{}:{}",
            repository.id, repository.default_branch, file.path
        ))
        .bind(&file.path)
        .bind(&file.content)
        .bind(&file.path)
        .bind(language_for_path(&file.path))
        .bind(&repository.default_branch)
        .bind(repository.visibility.as_str())
        .bind(json!({
            "href": href,
            "ownerLogin": repository.owner_login,
            "repositoryName": repository.name,
            "lineNumber": line_number,
            "fragment": fragment,
            "commitOid": commit.oid,
        }))
        .execute(pool)
        .await?;
    }

    Ok(())
}

fn first_searchable_line(content: &str) -> Option<(i64, String)> {
    content.lines().enumerate().find_map(|(index, line)| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some((index as i64 + 1, trimmed.chars().take(240).collect()))
        }
    })
}

fn language_for_path(path: &str) -> Option<String> {
    let extension = path.rsplit('.').next()?;
    let language = match extension.to_ascii_lowercase().as_str() {
        "md" | "markdown" => "Markdown",
        "rs" => "Rust",
        "ts" | "tsx" => "TypeScript",
        "js" | "jsx" => "JavaScript",
        "json" => "JSON",
        "toml" => "TOML",
        "yml" | "yaml" => "YAML",
        "sql" => "SQL",
        "css" => "CSS",
        "html" => "HTML",
        _ => return None,
    };
    Some(language.to_owned())
}

fn mime_type_for_path(path: &str, is_binary: bool) -> String {
    if is_binary {
        return "application/octet-stream".to_owned();
    }
    let mime_type = match path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "md" | "markdown" => "text/markdown; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "jsx" | "ts" | "tsx" => "text/plain; charset=utf-8",
        "toml" | "yml" | "yaml" | "sql" | "rs" => "text/plain; charset=utf-8",
        _ => "text/plain; charset=utf-8",
    };
    mime_type.to_owned()
}

fn is_probably_binary(content: &str) -> bool {
    let mut control_count = 0usize;
    let mut total_count = 0usize;
    for character in content.chars() {
        total_count += 1;
        if character.is_control() && !matches!(character, '\n' | '\r' | '\t') {
            control_count += 1;
        }
    }
    total_count > 0 && control_count * 3 >= total_count
}

fn render_mode(is_binary: bool, is_large: bool) -> &'static str {
    if is_binary {
        "binary"
    } else if is_large {
        "large"
    } else {
        "text"
    }
}

fn line_count(content: &str) -> i64 {
    if content.is_empty() {
        0
    } else {
        content.lines().count() as i64 + i64::from(content.ends_with('\n'))
    }
}

fn loc_count(content: &str) -> i64 {
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count() as i64
}

fn symbols_for_file(path: &str, content: &str) -> Vec<RepositoryCodeSymbol> {
    let language = language_for_path(path)
        .unwrap_or_default()
        .to_ascii_lowercase();
    content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            symbol_for_line(&language, line).map(|(kind, name)| RepositoryCodeSymbol {
                kind,
                name,
                line_number: (index + 1) as i64,
                preview: line.trim().chars().take(120).collect(),
            })
        })
        .take(50)
        .collect()
}

fn symbol_for_line(language: &str, line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    if language == "markdown" {
        return markdown_symbol(trimmed);
    }
    if language == "json" {
        return json_symbol(trimmed);
    }
    if matches!(language, "rust" | "typescript" | "javascript" | "python") {
        return function_symbol(trimmed);
    }
    None
}

fn markdown_symbol(trimmed: &str) -> Option<(String, String)> {
    let hashes = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=6).contains(&hashes) || !trimmed.chars().nth(hashes).is_some_and(char::is_whitespace) {
        return None;
    }
    let name = trimmed[hashes..].trim();
    if name.is_empty() {
        None
    } else {
        Some(("heading".to_owned(), name.chars().take(80).collect()))
    }
}

fn json_symbol(trimmed: &str) -> Option<(String, String)> {
    if !trimmed.starts_with('"') || trimmed.starts_with("\"$schema\"") {
        return None;
    }
    let end = trimmed[1..].find('"')? + 1;
    if !trimmed[end + 1..].trim_start().starts_with(':') {
        return None;
    }
    Some(("key".to_owned(), trimmed[1..end].chars().take(80).collect()))
}

fn function_symbol(trimmed: &str) -> Option<(String, String)> {
    let candidates = [
        "pub async fn ",
        "pub fn ",
        "async fn ",
        "fn ",
        "export async function ",
        "export function ",
        "function ",
        "export const ",
        "const ",
        "def ",
        "async def ",
    ];
    let candidate = candidates
        .iter()
        .find_map(|prefix| trimmed.strip_prefix(prefix).map(|rest| (*prefix, rest)))?;
    let mut name = candidate.1.trim_start();
    if candidate.0.ends_with("const ") {
        name = name.split('=').next()?.trim();
    } else {
        name = name
            .split(|character: char| character == '(' || character.is_whitespace())
            .next()?;
    }
    if name.is_empty() {
        None
    } else {
        Some(("function".to_owned(), name.chars().take(80).collect()))
    }
}

fn format_byte_size(byte_size: i64) -> String {
    if byte_size < 1024 {
        return format!("{byte_size} bytes");
    }
    let kib = byte_size as f64 / 1024.0;
    if kib < 1024.0 {
        return format!("{kib:.1} KB");
    }
    let mib = kib / 1024.0;
    format!("{mib:.1} MB")
}

fn percent_encode_path(path: &str) -> String {
    path.split('/')
        .map(percent_encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}

fn percent_encode_segment(segment: &str) -> String {
    let mut encoded = String::new();
    for byte in segment.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

async fn repository_sidebar_metadata(
    pool: &PgPool,
    repository: &Repository,
) -> Result<RepositorySidebarMetadata, RepositoryError> {
    let stars_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_stars WHERE repository_id = $1",
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let watchers_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_watches WHERE repository_id = $1 AND level <> 'ignore'",
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let forks_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_forks WHERE source_repository_id = $1",
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let releases_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM releases WHERE repository_id = $1")
            .bind(repository.id)
            .fetch_one(pool)
            .await?;
    let contributors_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(DISTINCT contributor_id)
        FROM (
            SELECT author_user_id AS contributor_id FROM commits WHERE repository_id = $1
            UNION
            SELECT committer_user_id AS contributor_id FROM commits WHERE repository_id = $1
        ) contributors
        WHERE contributor_id IS NOT NULL
        "#,
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let language_rows = sqlx::query(
        r#"
        SELECT language, color, byte_count
        FROM repository_languages
        WHERE repository_id = $1
        ORDER BY byte_count DESC, language ASC
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;
    let total_language_bytes = language_rows
        .iter()
        .map(|row| row.get::<i64, _>("byte_count"))
        .sum::<i64>()
        .max(1);
    let languages = language_rows
        .into_iter()
        .map(|row| {
            let byte_count = row.get::<i64, _>("byte_count");
            RepositoryLanguageSummary {
                language: row.get("language"),
                color: row.get("color"),
                byte_count,
                percentage: byte_count * 100 / total_language_bytes,
            }
        })
        .collect();

    Ok(RepositorySidebarMetadata {
        about: repository.description.clone(),
        website_url: None,
        topics: Vec::new(),
        stars_count,
        watchers_count,
        forks_count,
        releases_count,
        deployments_count: 0,
        contributors_count,
        languages,
    })
}

async fn repository_viewer_state(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<RepositoryViewerState, RepositoryError> {
    let starred = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM repository_stars WHERE user_id = $1 AND repository_id = $2
        )
        "#,
    )
    .bind(actor_user_id)
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let (watch_level, custom_watch_events) =
        repository_watch_state(pool, repository.id, actor_user_id).await?;
    let watching = watch_level.is_active()
        && sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM repository_watches WHERE user_id = $1 AND repository_id = $2
            )
            "#,
        )
        .bind(actor_user_id)
        .bind(repository.id)
        .fetch_one(pool)
        .await?;
    let forked_repository_href =
        existing_fork_href_for_user(pool, repository.id, actor_user_id).await?;

    Ok(RepositoryViewerState {
        starred,
        watching,
        watch_label: watch_level.label().to_owned(),
        watch_level,
        custom_watch_events,
        forked_repository_href,
    })
}

async fn repository_social_state(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<RepositorySocialState, RepositoryError> {
    let sidebar = repository_sidebar_metadata(pool, repository).await?;
    let viewer_state = repository_viewer_state(pool, repository, actor_user_id).await?;
    Ok(RepositorySocialState {
        starred: viewer_state.starred,
        watching: viewer_state.watching,
        watch_label: viewer_state.watch_label,
        watch_level: viewer_state.watch_level,
        custom_watch_events: viewer_state.custom_watch_events,
        stars_count: sidebar.stars_count,
        watchers_count: sidebar.watchers_count,
        forks_count: sidebar.forks_count,
        forked_repository_href: viewer_state.forked_repository_href,
    })
}

async fn repository_watch_settings(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<RepositoryWatchSettings, RepositoryError> {
    let sidebar = repository_sidebar_metadata(pool, repository).await?;
    let (level, custom_events) = repository_watch_state(pool, repository.id, actor_user_id).await?;

    Ok(RepositoryWatchSettings {
        repository_id: repository.id,
        level,
        label: level.label().to_owned(),
        watching: level.is_active()
            && sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS (SELECT 1 FROM repository_watches WHERE user_id = $1 AND repository_id = $2)",
            )
            .bind(actor_user_id)
            .bind(repository.id)
            .fetch_one(pool)
            .await?,
        watchers_count: sidebar.watchers_count,
        custom_events,
        available_events: repository_watch_events(),
        ignore_warning:
            "Ignoring this repository suppresses repository watch notifications until you choose another watch level."
                .to_owned(),
    })
}

async fn repository_watch_state(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(RepositoryWatchLevel, Vec<RepositoryWatchEvent>), RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT level, custom_events
        FROM repository_watches
        WHERE user_id = $1 AND repository_id = $2
        "#,
    )
    .bind(actor_user_id)
    .bind(repository_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok((RepositoryWatchLevel::Participating, Vec::new()));
    };
    let level = RepositoryWatchLevel::try_from(row.get::<String, _>("level").as_str())?;
    let custom_events = watch_events_from_json(row.get::<serde_json::Value, _>("custom_events"))?;
    Ok((level, custom_events))
}

async fn save_repository_watch_settings(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
    patch: RepositoryWatchSettingsPatch,
) -> Result<(), RepositoryError> {
    let custom_events = normalize_watch_events(patch.custom_events, patch.level)?;
    let reason = match patch.level {
        RepositoryWatchLevel::Participating => "subscribed",
        other => other.as_str(),
    };

    sqlx::query(
        r#"
        INSERT INTO repository_watches (user_id, repository_id, reason, level, custom_events, ignored_at)
        VALUES ($1, $2, $3, $4, $5, CASE WHEN $4 = 'ignore' THEN now() ELSE NULL END)
        ON CONFLICT (user_id, repository_id)
        DO UPDATE SET reason = EXCLUDED.reason,
                      level = EXCLUDED.level,
                      custom_events = EXCLUDED.custom_events,
                      ignored_at = CASE
                          WHEN EXCLUDED.level = 'ignore' THEN COALESCE(repository_watches.ignored_at, now())
                          ELSE NULL
                      END
        "#,
    )
    .bind(actor_user_id)
    .bind(repository_id)
    .bind(reason)
    .bind(patch.level.as_str())
    .bind(Json(custom_events.iter().map(|event| event.as_str()).collect::<Vec<_>>()))
    .execute(pool)
    .await?;

    Ok(())
}

fn repository_watch_events() -> Vec<RepositoryWatchEvent> {
    vec![
        RepositoryWatchEvent::Issues,
        RepositoryWatchEvent::PullRequests,
        RepositoryWatchEvent::Releases,
        RepositoryWatchEvent::Discussions,
        RepositoryWatchEvent::Actions,
        RepositoryWatchEvent::SecurityAlerts,
        RepositoryWatchEvent::RepositoryInvitations,
    ]
}

fn normalize_watch_events(
    events: Vec<RepositoryWatchEvent>,
    level: RepositoryWatchLevel,
) -> Result<Vec<RepositoryWatchEvent>, RepositoryError> {
    if level != RepositoryWatchLevel::Custom {
        return Ok(Vec::new());
    }
    if events.is_empty() {
        return Err(RepositoryError::InvalidWatchEvent(
            "custom watch level requires at least one event".to_owned(),
        ));
    }
    let mut unique = BTreeSet::new();
    for event in events {
        unique.insert(event);
    }
    Ok(unique.into_iter().collect())
}

fn watch_events_from_json(
    value: serde_json::Value,
) -> Result<Vec<RepositoryWatchEvent>, RepositoryError> {
    value
        .as_array()
        .ok_or_else(|| {
            RepositoryError::InvalidWatchEvent("custom_events must be an array".to_owned())
        })?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| {
                    RepositoryError::InvalidWatchEvent(
                        "custom_events must contain strings".to_owned(),
                    )
                })
                .and_then(RepositoryWatchEvent::try_from)
        })
        .collect::<Result<Vec<_>, _>>()
}

async fn existing_fork_href_for_user(
    pool: &PgPool,
    source_repository_id: Uuid,
    actor_user_id: Uuid,
) -> Result<Option<String>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug) AS owner_login,
               forks.name
        FROM repository_forks
        JOIN repositories forks ON forks.id = repository_forks.fork_repository_id
        LEFT JOIN users owner_user ON owner_user.id = forks.owner_user_id
        LEFT JOIN organizations ON organizations.id = forks.owner_organization_id
        WHERE repository_forks.source_repository_id = $1
          AND repository_forks.forked_by_user_id = $2
        ORDER BY repository_forks.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(source_repository_id)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| {
        format!(
            "/{}/{}",
            row.get::<String, _>("owner_login"),
            row.get::<String, _>("name")
        )
    }))
}

async fn copy_repository_snapshot(
    pool: &PgPool,
    source: &Repository,
    destination: &Repository,
    actor_user_id: Uuid,
) -> Result<(), RepositoryError> {
    let files = list_repository_files(pool, source.id).await?;
    if files.is_empty() {
        return Ok(());
    }
    let source_commit = latest_commit_for_repository(pool, source).await?;
    let source_oid = source_commit
        .as_ref()
        .map(|commit| commit.oid.as_str())
        .unwrap_or("empty");
    let files_for_hash = files
        .iter()
        .map(|file| format!("{}:{}", file.path, file.oid))
        .collect::<Vec<_>>()
        .join("\n");
    let tree_oid = deterministic_oid(
        "fork-tree",
        &format!("{}:{}:{}", destination.id, source_oid, files_for_hash),
    );
    let commit_oid = deterministic_oid(
        "fork-commit",
        &format!("{}:{}:{}", destination.id, source.id, source_oid),
    );
    let snapshot = RepositorySnapshot {
        branch_name: destination.default_branch.clone(),
        commit: CreateCommit {
            oid: commit_oid,
            author_user_id: Some(actor_user_id),
            committer_user_id: Some(actor_user_id),
            message: format!("Forked from {}/{}", source.owner_login, source.name),
            tree_oid: Some(tree_oid),
            parent_oids: source_commit
                .as_ref()
                .map(|commit| vec![commit.oid.clone()])
                .unwrap_or_default(),
            committed_at: Utc::now(),
        },
        files: files
            .into_iter()
            .map(|file| RepositorySnapshotFile {
                path: file.path,
                content: file.content,
                oid: file.oid,
                byte_size: file.byte_size,
            })
            .collect(),
    };
    replace_repository_snapshot(pool, destination.id, snapshot).await?;
    Ok(())
}

fn repository_clone_urls(repository: &Repository) -> RepositoryCloneUrls {
    let path = format!("{}/{}", repository.owner_login, repository.name);
    let (https_base, ssh_host) = clone_url_hosts();
    RepositoryCloneUrls {
        https: format!("{https_base}/{path}.git"),
        git: format!("git@{ssh_host}:{path}.git"),
        zip: format!(
            "/{path}/archive/refs/heads/{}.zip",
            repository.default_branch
        ),
    }
}

fn clone_url_hosts() -> (String, String) {
    let raw = std::env::var("API_URL")
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .filter(|s| {
            url::Url::parse(s)
                .ok()
                .and_then(|u| {
                    u.host_str()
                        .map(|host| !matches!(host, "localhost" | "127.0.0.1"))
                })
                .unwrap_or(true)
        })
        .unwrap_or_else(|| "https://opengithub.namuh.co".to_owned());
    let https_base = raw.trim_end_matches('/').to_owned();
    let ssh_host = url::Url::parse(&https_base)
        .ok()
        .and_then(|u| u.host_str().map(String::from))
        .unwrap_or_else(|| "opengithub.namuh.co".to_owned());
    (https_base, ssh_host)
}

async fn ensure_owner_can_create(
    pool: &PgPool,
    owner: &RepositoryOwner,
    actor_user_id: Uuid,
) -> Result<(), RepositoryError> {
    match owner {
        RepositoryOwner::User { id } => {
            if *id == actor_user_id {
                Ok(())
            } else {
                Err(RepositoryError::OwnerPermissionDenied)
            }
        }
        RepositoryOwner::Organization { id } => {
            let allowed = sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM organization_memberships
                    WHERE organization_id = $1
                      AND user_id = $2
                )
                "#,
            )
            .bind(id)
            .bind(actor_user_id)
            .fetch_one(pool)
            .await?;

            if allowed {
                Ok(())
            } else {
                Err(RepositoryError::OwnerPermissionDenied)
            }
        }
    }
}

async fn ensure_owner_visibility_can_create(
    pool: &PgPool,
    owner: &RepositoryOwner,
    actor_user_id: Uuid,
    visibility: &RepositoryVisibility,
) -> Result<(), RepositoryError> {
    let RepositoryOwner::Organization { id } = owner else {
        if *visibility == RepositoryVisibility::Internal {
            return Err(RepositoryError::InvalidVisibility(
                "internal repositories require an organization owner".to_owned(),
            ));
        }
        return Ok(());
    };

    let Some(row) = sqlx::query(
        r#"
        SELECT organizations.slug,
               organization_memberships.role,
               COALESCE(organization_policy_settings.members_can_create_public_repositories, true) AS members_can_create_public_repositories,
               COALESCE(organization_policy_settings.members_can_create_private_repositories, true) AS members_can_create_private_repositories,
               COALESCE(organization_policy_settings.members_can_create_internal_repositories, false) AS members_can_create_internal_repositories
        FROM organization_memberships
        JOIN organizations ON organizations.id = organization_memberships.organization_id
        LEFT JOIN organization_policy_settings
          ON organization_policy_settings.organization_id = organizations.id
        WHERE organization_memberships.organization_id = $1
          AND organization_memberships.user_id = $2
        "#,
    )
    .bind(id)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?
    else {
        return Err(RepositoryError::OwnerPermissionDenied);
    };

    let role: String = row.try_get("role")?;
    if matches!(role.as_str(), "owner" | "admin") {
        return Ok(());
    }

    let enabled = match visibility {
        RepositoryVisibility::Public => row.try_get("members_can_create_public_repositories")?,
        RepositoryVisibility::Private => row.try_get("members_can_create_private_repositories")?,
        RepositoryVisibility::Internal => {
            row.try_get("members_can_create_internal_repositories")?
        }
    };

    if enabled {
        Ok(())
    } else {
        let slug: String = row.try_get("slug")?;
        Err(RepositoryError::OrganizationRepositoryCreationPolicy {
            visibility: visibility.as_str().to_owned(),
            reason: format!(
                "Organization policy prevents members from creating {} repositories.",
                visibility.as_str()
            ),
            settings_href: format!("/organizations/{slug}/settings/member_privileges"),
        })
    }
}

fn repository_creation_visibility_options_from_row(
    row: &sqlx::postgres::PgRow,
) -> Vec<RepositoryCreationVisibilityOption> {
    let owner_type: String = row.get("owner_type");
    if owner_type == "user" {
        return vec![
            RepositoryCreationVisibilityOption {
                visibility: RepositoryVisibility::Public,
                enabled: true,
                reason: None,
            },
            RepositoryCreationVisibilityOption {
                visibility: RepositoryVisibility::Private,
                enabled: true,
                reason: None,
            },
        ];
    }

    let role: Option<String> = row.try_get("organization_role").ok();
    let is_admin = role
        .as_deref()
        .is_some_and(|value| matches!(value, "owner" | "admin"));
    let policy_reason = |visibility: RepositoryVisibility| {
        Some(format!(
            "Organization policy prevents members from creating {} repositories.",
            visibility.as_str()
        ))
    };
    let values = [
        (
            RepositoryVisibility::Public,
            row.try_get("members_can_create_public_repositories")
                .unwrap_or(true),
        ),
        (
            RepositoryVisibility::Private,
            row.try_get("members_can_create_private_repositories")
                .unwrap_or(true),
        ),
        (
            RepositoryVisibility::Internal,
            row.try_get("members_can_create_internal_repositories")
                .unwrap_or(false),
        ),
    ];

    values
        .into_iter()
        .map(|(visibility, policy_enabled)| {
            let enabled = is_admin || policy_enabled;
            RepositoryCreationVisibilityOption {
                reason: if enabled {
                    None
                } else {
                    policy_reason(visibility.clone())
                },
                visibility,
                enabled,
            }
        })
        .collect()
}

async fn organization_base_repository_role(
    pool: &PgPool,
    organization_id: Uuid,
) -> Result<Option<RepositoryRole>, RepositoryError> {
    let role = sqlx::query_scalar::<_, String>(
        r#"
        SELECT base_repository_permission
        FROM organization_policy_settings
        WHERE organization_id = $1
        "#,
    )
    .bind(organization_id)
    .fetch_optional(pool)
    .await?
    .unwrap_or_else(|| "read".to_owned());

    if role == "none" {
        return Ok(None);
    }

    RepositoryRole::try_from(role.as_str())
        .map(Some)
        .map_err(|error| RepositoryError::Sqlx(sqlx::Error::Protocol(error.to_string())))
}

async fn repository_owner_login(
    pool: &PgPool,
    owner: &RepositoryOwner,
) -> Result<(String, Uuid, String), RepositoryError> {
    match owner {
        RepositoryOwner::User { id } => {
            let login = sqlx::query_scalar::<_, String>(
                "SELECT COALESCE(NULLIF(username, ''), email) FROM users WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(RepositoryError::OwnerNotFound)?;
            Ok(("user".to_owned(), *id, login))
        }
        RepositoryOwner::Organization { id } => {
            let login =
                sqlx::query_scalar::<_, String>("SELECT slug FROM organizations WHERE id = $1")
                    .bind(id)
                    .fetch_optional(pool)
                    .await?
                    .ok_or(RepositoryError::OwnerNotFound)?;
            Ok(("organization".to_owned(), *id, login))
        }
    }
}

async fn repository_exists_for_owner(
    pool: &PgPool,
    owner: &RepositoryOwner,
    name: &str,
) -> Result<bool, RepositoryError> {
    let exists = match owner {
        RepositoryOwner::User { id } => {
            sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS (
                    SELECT 1 FROM repositories
                    WHERE owner_user_id = $1 AND lower(name) = lower($2)
                )
                "#,
            )
            .bind(id)
            .bind(name)
            .fetch_one(pool)
            .await?
        }
        RepositoryOwner::Organization { id } => {
            sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS (
                    SELECT 1 FROM repositories
                    WHERE owner_organization_id = $1 AND lower(name) = lower($2)
                )
                "#,
            )
            .bind(id)
            .bind(name)
            .fetch_one(pool)
            .await?
        }
    };

    Ok(exists)
}

pub fn normalize_repository_name(value: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_hyphen = false;

    for character in value.trim().chars() {
        let next = if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
            character
        } else {
            '-'
        };

        if next == '-' {
            if previous_was_hyphen {
                continue;
            }
            previous_was_hyphen = true;
        } else {
            previous_was_hyphen = false;
        }
        normalized.push(next);
    }

    normalized.trim_matches('-').to_owned()
}

fn normalize_repository_description(
    value: Option<String>,
) -> Result<Option<String>, RepositoryError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let trimmed = value.trim().to_owned();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > 350 {
        return Err(RepositoryError::InvalidDescription(
            "Repository description must be 350 characters or fewer.".to_owned(),
        ));
    }
    Ok(Some(trimmed))
}

fn validate_repository_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Repository name is required.".to_owned());
    }
    if name.len() > 100 {
        return Err("Repository name must be 100 characters or fewer.".to_owned());
    }
    if name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
    {
        Ok(())
    } else {
        Err(
            "Repository names can only include letters, numbers, dots, underscores, and hyphens."
                .to_owned(),
        )
    }
}

async fn ensure_default_repository_labels(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<(), RepositoryError> {
    const DEFAULT_LABELS: [(&str, &str, &str); 4] = [
        ("bug", "d73a4a", "Something is not working"),
        (
            "documentation",
            "0075ca",
            "Improvements or additions to documentation",
        ),
        ("enhancement", "a2eeef", "New feature or request"),
        ("good first issue", "7057ff", "Good for newcomers"),
    ];

    for (name, color, description) in DEFAULT_LABELS {
        sqlx::query(
            r#"
            INSERT INTO labels (repository_id, name, color, description, is_default)
            VALUES ($1, $2, $3, $4, true)
            ON CONFLICT (repository_id, lower(name)) DO NOTHING
            "#,
        )
        .bind(repository_id)
        .bind(name)
        .bind(color)
        .bind(description)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn insert_repository_create_feed_event(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO feed_events (
            actor_user_id,
            repository_id,
            event_type,
            title,
            excerpt,
            target_href,
            subject_type,
            subject_id,
            metadata
        )
        VALUES ($1, $2, 'repository_create', $3, $4, $5, 'repository', $2, $6)
        "#,
    )
    .bind(actor_user_id)
    .bind(repository.id)
    .bind(format!(
        "Created repository {}/{}",
        repository.owner_login, repository.name
    ))
    .bind(repository.description.as_deref())
    .bind(format!("/{}/{}", repository.owner_login, repository.name))
    .bind(serde_json::json!({
        "visibility": repository.visibility.as_str(),
        "defaultBranch": repository.default_branch,
    }))
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_repository_social_feed_event(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    event_type: &str,
) -> Result<(), RepositoryError> {
    let (title, excerpt) = match event_type {
        "star" => (
            format!("Starred {}/{}", repository.owner_login, repository.name),
            "Repository starred from the Code tab.",
        ),
        _ => return Ok(()),
    };
    sqlx::query(
        r#"
        INSERT INTO feed_events (
            actor_user_id,
            repository_id,
            event_type,
            title,
            excerpt,
            target_href,
            subject_type,
            subject_id,
            metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'repository', $2, $7)
        "#,
    )
    .bind(actor_user_id)
    .bind(repository.id)
    .bind(event_type)
    .bind(title)
    .bind(excerpt)
    .bind(format!("/{}/{}", repository.owner_login, repository.name))
    .bind(serde_json::json!({
        "source": "repository_header",
        "repository": format!("{}/{}", repository.owner_login, repository.name),
    }))
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_repository_fork_feed_event(
    pool: &PgPool,
    source_repository: &Repository,
    fork_repository: &Repository,
    actor_user_id: Uuid,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO feed_events (
            actor_user_id,
            repository_id,
            event_type,
            title,
            excerpt,
            target_href,
            subject_type,
            subject_id,
            metadata
        )
        VALUES ($1, $2, 'fork', $3, $4, $5, 'repository', $6, $7)
        "#,
    )
    .bind(actor_user_id)
    .bind(source_repository.id)
    .bind(format!(
        "Forked {}/{}",
        source_repository.owner_login, source_repository.name
    ))
    .bind(format!(
        "Created fork {}/{}",
        fork_repository.owner_login, fork_repository.name
    ))
    .bind(format!(
        "/{}/{}",
        fork_repository.owner_login, fork_repository.name
    ))
    .bind(fork_repository.id)
    .bind(serde_json::json!({
        "sourceRepository": format!("{}/{}", source_repository.owner_login, source_repository.name),
        "forkRepository": format!("{}/{}", fork_repository.owner_login, fork_repository.name),
    }))
    .execute(pool)
    .await?;

    Ok(())
}

fn suggested_repository_name(actor_user_id: Uuid) -> String {
    let words = [
        "silver-train",
        "probable-octo",
        "refactored-disco",
        "friendly-engine",
    ];
    let index = actor_user_id.as_bytes()[0] as usize % words.len();
    words[index].to_owned()
}

fn organization_from_row(row: sqlx::postgres::PgRow) -> Organization {
    Organization {
        id: row.get("id"),
        slug: row.get("slug"),
        display_name: row.get("display_name"),
        description: row.get("description"),
        owner_user_id: row.get("owner_user_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn repository_from_row(row: sqlx::postgres::PgRow) -> Result<Repository, RepositoryError> {
    let visibility: String = row.get("visibility");
    Ok(Repository {
        id: row.get("id"),
        owner_user_id: row.get("owner_user_id"),
        owner_organization_id: row.get("owner_organization_id"),
        owner_login: row.get("owner_login"),
        name: row.get("name"),
        description: row.get("description"),
        visibility: RepositoryVisibility::try_from(visibility.as_str())?,
        default_branch: row.get("default_branch"),
        is_archived: row.get("is_archived"),
        created_by_user_id: row.get("created_by_user_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn repository_permission_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<RepositoryPermission, RepositoryError> {
    let role: String = row.get("role");
    Ok(RepositoryPermission {
        repository_id: row.get("repository_id"),
        user_id: row.get("user_id"),
        role: RepositoryRole::try_from(role.as_str())
            .map_err(|error| RepositoryError::Sqlx(sqlx::Error::Protocol(error.to_string())))?,
        source: row.get("source"),
    })
}

fn commit_from_row(row: sqlx::postgres::PgRow) -> Commit {
    Commit {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        oid: row.get("oid"),
        author_user_id: row.get("author_user_id"),
        committer_user_id: row.get("committer_user_id"),
        message: row.get("message"),
        tree_oid: row.get("tree_oid"),
        parent_oids: row.get("parent_oids"),
        committed_at: row.get("committed_at"),
        created_at: row.get("created_at"),
    }
}

fn git_ref_from_row(row: sqlx::postgres::PgRow) -> GitRef {
    GitRef {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        name: row.get("name"),
        kind: row.get("kind"),
        target_commit_id: row.get("target_commit_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn repository_file_from_row(row: sqlx::postgres::PgRow) -> RepositoryFile {
    RepositoryFile {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        commit_id: row.get("commit_id"),
        path: row.get("path"),
        content: row.get("content"),
        oid: row.get("oid"),
        byte_size: row.get("byte_size"),
        created_at: row.get("created_at"),
    }
}
