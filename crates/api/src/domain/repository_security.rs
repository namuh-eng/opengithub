use chrono::{DateTime, Utc};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::{
    markdown::{render_markdown, RenderMarkdownInput},
    pulls::{create_pull_request, CreatePullRequest},
    repositories::{
        can_read_repository, can_write_repository, get_repository_by_owner_name,
        replace_repository_snapshot, CreateCommit, Repository, RepositoryError, RepositorySnapshot,
        RepositorySnapshotFile, RepositoryVisibility,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySecurityOverview {
    pub repository: RepositorySecurityRepository,
    pub viewer: SecurityViewer,
    pub policy: SecurityPolicySummary,
    pub features: Vec<SecurityFeatureCard>,
    pub advisories: Vec<RepositorySecurityAdvisorySummary>,
    pub links: SecurityOverviewLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySecurityRepository {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub visibility: String,
    pub default_branch: String,
    pub security_href: String,
    pub policy_href: String,
    pub advisories_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityViewer {
    pub permission: String,
    pub can_read: bool,
    pub can_write: bool,
    pub can_edit_policy: bool,
    pub can_view_private_alert_counts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPolicySummary {
    pub exists: bool,
    pub path: Option<String>,
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub blob_oid: Option<String>,
    pub content_sha: Option<String>,
    pub html: Option<String>,
    pub source_href: Option<String>,
    pub raw_href: Option<String>,
    pub history_href: Option<String>,
    pub edit_href: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub empty_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityFeatureCard {
    pub key: String,
    pub label: String,
    pub status: String,
    pub summary: String,
    pub alert_count: Option<i64>,
    pub private_count: Option<i64>,
    pub href: String,
    pub config_href: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySecurityAdvisorySummary {
    pub id: Uuid,
    pub identifier: String,
    pub severity: String,
    pub status: String,
    pub title: String,
    pub summary: String,
    pub package_name: Option<String>,
    pub vulnerable_range: Option<String>,
    pub href: String,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityOverviewLinks {
    pub overview_href: String,
    pub policy_href: String,
    pub advisories_href: String,
    pub dependabot_href: String,
    pub code_scanning_href: String,
    pub secret_scanning_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySecurityPolicyView {
    pub repository: RepositorySecurityRepository,
    pub viewer: SecurityViewer,
    pub policy: SecurityPolicyDocument,
    pub links: SecurityOverviewLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPolicyDocument {
    pub exists: bool,
    pub path: Option<String>,
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub blob_oid: Option<String>,
    pub content_sha: Option<String>,
    pub markdown: Option<String>,
    pub html: Option<String>,
    pub outline: Vec<SecurityPolicyHeading>,
    pub source_href: Option<String>,
    pub raw_href: Option<String>,
    pub history_href: Option<String>,
    pub edit_href: Option<String>,
    pub latest_commit: Option<SecurityPolicyCommit>,
    pub updated_at: Option<DateTime<Utc>>,
    pub empty_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPolicyHeading {
    pub id: String,
    pub level: i32,
    pub text: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPolicyCommit {
    pub oid: String,
    pub short_oid: String,
    pub message: String,
    pub committed_at: DateTime<Utc>,
    pub href: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPolicyMutation {
    pub markdown: String,
    pub commit_message: String,
    pub path: Option<String>,
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub expected_content_sha: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct DependabotAlertsQuery<'a> {
    pub state: Option<&'a str>,
    pub query: Option<&'a str>,
    pub package: Option<&'a str>,
    pub ecosystem: Option<&'a str>,
    pub manifest: Option<&'a str>,
    pub scope: Option<&'a str>,
    pub severity: Option<&'a str>,
    pub sort: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertsView {
    pub repository: RepositorySecurityRepository,
    pub viewer: SecurityViewer,
    pub availability: DependabotAlertsAvailability,
    pub filters: DependabotAlertFilters,
    pub counts: DependabotAlertCounts,
    pub alerts: Vec<DependabotAlertRow>,
    pub packages: Vec<DependabotAlertPackageFilter>,
    pub manifests: Vec<DependabotAlertManifestFilter>,
    pub links: DependabotAlertLinks,
    pub freshness: DependabotAlertFreshness,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertDetail {
    pub repository: RepositorySecurityRepository,
    pub viewer: SecurityViewer,
    pub availability: DependabotAlertsAvailability,
    pub alert: DependabotAlertRow,
    pub advisory: DependabotAdvisoryDetail,
    pub dependency: DependabotDependencyDetail,
    pub timeline: Vec<DependabotAlertTimelineEvent>,
    pub assignee_options: Vec<DependabotAlertAssignmentOption>,
    pub security_update: DependabotSecurityUpdateState,
    pub links: DependabotAlertLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertsAvailability {
    pub enabled: bool,
    pub indexed: bool,
    pub message: String,
    pub disabled_reason: Option<String>,
    pub settings_href: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertFilters {
    pub state: String,
    pub query: Option<String>,
    pub package: Option<String>,
    pub ecosystem: Option<String>,
    pub manifest: Option<String>,
    pub scope: Option<String>,
    pub severity: Option<String>,
    pub sort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertCounts {
    pub open: i64,
    pub closed: i64,
    pub total: i64,
    pub visible: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertPackage {
    pub id: Uuid,
    pub ecosystem: String,
    pub name: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertAdvisorySummary {
    pub id: Uuid,
    pub identifier: String,
    pub severity: String,
    pub title: String,
    pub href: String,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertRow {
    pub id: Uuid,
    pub number: i64,
    pub state: String,
    pub scope: String,
    pub package: DependabotAlertPackage,
    pub advisory: DependabotAlertAdvisorySummary,
    pub manifest_path: String,
    pub manifest_href: String,
    pub lockfile_path: Option<String>,
    pub lockfile_href: Option<String>,
    pub vulnerable_requirements: Option<String>,
    pub current_version: Option<String>,
    pub fixed_version: Option<String>,
    pub relationship: String,
    pub assignees: Vec<DependabotAlertAssignee>,
    pub href: String,
    pub detected_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertAssignee {
    pub id: Uuid,
    pub login: String,
    pub avatar_url: Option<String>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertPackageFilter {
    pub package: DependabotAlertPackage,
    pub open_count: i64,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertManifestFilter {
    pub path: String,
    pub ecosystem: String,
    pub href: String,
    pub open_count: i64,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAdvisoryDetail {
    pub identifier: String,
    pub severity: String,
    pub title: String,
    pub href: String,
    pub vulnerable_range: String,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotDependencyDetail {
    pub package: DependabotAlertPackage,
    pub manifest_path: String,
    pub manifest_href: String,
    pub lockfile_path: Option<String>,
    pub lockfile_href: Option<String>,
    pub current_version: Option<String>,
    pub relationship: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertTimelineEvent {
    pub id: Uuid,
    pub event_type: String,
    pub message: String,
    pub actor: Option<DependabotAlertAssignee>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertAssignmentOption {
    pub id: Uuid,
    pub kind: String,
    pub login: String,
    pub avatar_url: Option<String>,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotSecurityUpdateState {
    pub supported: bool,
    pub status: String,
    pub href: Option<String>,
    pub pull_request_href: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertLinks {
    pub list_href: String,
    pub open_href: String,
    pub closed_href: String,
    pub settings_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertFreshness {
    pub computed_at: DateTime<Utc>,
    pub cadence: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertMutation {
    pub action: String,
    pub dismissal_reason: Option<String>,
    pub dismissal_comment: Option<String>,
    pub assignee_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningAlertMutation {
    pub action: String,
    pub dismissal_reason: Option<String>,
    pub dismissal_comment: Option<String>,
    pub assignee_ids: Option<Vec<Uuid>>,
    pub linked_issue_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotBulkMutation {
    pub action: String,
    pub alert_ids: Vec<Uuid>,
    pub dismissal_reason: Option<String>,
    pub dismissal_comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotBulkMutationResult {
    pub repository: RepositorySecurityRepository,
    pub requested_count: usize,
    pub updated_count: usize,
    pub results: Vec<DependabotBulkAlertResult>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotBulkAlertResult {
    pub id: Uuid,
    pub number: i64,
    pub state: String,
    pub ok: bool,
    pub message: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotSecurityUpdateResult {
    pub alert: DependabotAlertRow,
    pub status: String,
    pub branch: String,
    pub commit_oid: Option<String>,
    pub pull_request_href: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub struct CodeScanningAlertsQuery<'a> {
    pub state: Option<&'a str>,
    pub query: Option<&'a str>,
    pub severity: Option<&'a str>,
    pub security_severity: Option<&'a str>,
    pub tool: Option<&'a str>,
    pub branch: Option<&'a str>,
    pub ref_name: Option<&'a str>,
    pub tag: Option<&'a str>,
    pub application_code: Option<&'a str>,
    pub sort: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningAlertsView {
    pub repository: RepositorySecurityRepository,
    pub viewer: SecurityViewer,
    pub availability: CodeScanningAvailability,
    pub filters: CodeScanningFilters,
    pub counts: CodeScanningAlertCounts,
    pub alerts: Vec<CodeScanningAlertRow>,
    pub tools: Vec<CodeScanningToolStatus>,
    pub branches: Vec<CodeScanningBranchFilter>,
    pub links: CodeScanningLinks,
    pub freshness: DependabotAlertFreshness,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningAlertDetail {
    pub repository: RepositorySecurityRepository,
    pub viewer: SecurityViewer,
    pub availability: CodeScanningAvailability,
    pub alert: CodeScanningAlertRow,
    pub location: CodeScanningLocation,
    pub rule: CodeScanningRuleDetail,
    pub timeline: Vec<CodeScanningTimelineEvent>,
    pub assignee_options: Vec<DependabotAlertAssignmentOption>,
    pub linked_issue: CodeScanningLinkedIssueState,
    pub links: CodeScanningLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningAvailability {
    pub enabled: bool,
    pub indexed: bool,
    pub message: String,
    pub disabled_reason: Option<String>,
    pub settings_href: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningFilters {
    pub state: String,
    pub query: Option<String>,
    pub severity: Option<String>,
    pub security_severity: Option<String>,
    pub tool: Option<String>,
    pub branch: Option<String>,
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub tag: Option<String>,
    pub application_code: Option<String>,
    pub sort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningAlertCounts {
    pub open: i64,
    pub closed: i64,
    pub total: i64,
    pub visible: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningAlertRow {
    pub id: Uuid,
    pub number: i64,
    pub state: String,
    pub rule_id: String,
    pub rule_name: String,
    pub message: String,
    pub severity: String,
    pub security_severity: Option<String>,
    pub tool_name: String,
    pub path: String,
    pub path_href: String,
    pub start_line: i32,
    pub end_line: Option<i32>,
    pub ref_name: String,
    pub branch_name: Option<String>,
    pub is_default_branch: bool,
    pub linked_issue: Option<CodeScanningIssueLink>,
    pub assignees: Vec<DependabotAlertAssignee>,
    pub href: String,
    pub detected_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningIssueLink {
    pub id: Uuid,
    pub number: i64,
    pub title: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningLocation {
    pub path: String,
    pub path_href: String,
    pub raw_href: String,
    pub start_line: i32,
    pub end_line: Option<i32>,
    pub code_snippet: Option<String>,
    pub ref_name: String,
    pub commit_oid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningRuleDetail {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub help_markdown: Option<String>,
    pub help_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningTimelineEvent {
    pub id: Uuid,
    pub event_type: String,
    pub message: String,
    pub actor: Option<DependabotAlertAssignee>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningLinkedIssueState {
    pub issue: Option<CodeScanningIssueLink>,
    pub can_link: bool,
    pub create_href: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningToolStatus {
    pub name: String,
    pub version: Option<String>,
    pub status: String,
    pub alert_count: i64,
    pub latest_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningBranchFilter {
    pub name: String,
    pub open_count: i64,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeScanningLinks {
    pub list_href: String,
    pub open_href: String,
    pub closed_href: String,
    pub upload_href: String,
    pub settings_href: String,
}

pub async fn repository_code_scanning_alerts_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    query: CodeScanningAlertsQuery<'_>,
) -> Result<Option<CodeScanningAlertsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }

    repository_code_scanning_alerts_for_repository(pool, &repository, actor_user_id, query)
        .await
        .map(Some)
}

pub async fn repository_code_scanning_alert_detail_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    alert_number: i64,
) -> Result<Option<CodeScanningAlertDetail>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    if alert_number <= 0 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "code scanning alert id must be a positive number".to_owned(),
        ));
    }

    repository_code_scanning_alert_detail_for_repository(
        pool,
        &repository,
        actor_user_id,
        alert_number,
    )
    .await
}

pub async fn update_repository_code_scanning_alert_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    alert_number: i64,
    mutation: CodeScanningAlertMutation,
) -> Result<Option<CodeScanningAlertDetail>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    if !can_write_repository(pool, &repository, actor_user_id).await? {
        return Err(RepositoryError::PermissionDenied);
    }
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    if alert_number <= 0 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "code scanning alert id must be a positive number".to_owned(),
        ));
    }

    update_repository_code_scanning_alert(pool, &repository, actor_user_id, alert_number, mutation)
        .await?;
    repository_code_scanning_alert_detail_for_repository(
        pool,
        &repository,
        actor_user_id,
        alert_number,
    )
    .await
}

pub async fn create_or_link_repository_code_scanning_issue_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    alert_number: i64,
) -> Result<Option<CodeScanningAlertDetail>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    if !can_write_repository(pool, &repository, actor_user_id).await? {
        return Err(RepositoryError::PermissionDenied);
    }
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    if alert_number <= 0 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "code scanning alert id must be a positive number".to_owned(),
        ));
    }

    create_or_link_repository_code_scanning_issue(pool, &repository, actor_user_id, alert_number)
        .await?;
    repository_code_scanning_alert_detail_for_repository(
        pool,
        &repository,
        actor_user_id,
        alert_number,
    )
    .await
}

pub async fn repository_dependabot_alerts_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    query: DependabotAlertsQuery<'_>,
) -> Result<Option<DependabotAlertsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }

    repository_dependabot_alerts_for_repository(pool, &repository, actor_user_id, query)
        .await
        .map(Some)
}

pub async fn repository_dependabot_alert_detail_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    alert_number: i64,
) -> Result<Option<DependabotAlertDetail>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    if alert_number <= 0 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "alert id must be a positive number".to_owned(),
        ));
    }

    repository_dependabot_alert_detail_for_repository(
        pool,
        &repository,
        actor_user_id,
        alert_number,
    )
    .await
}

pub async fn update_repository_dependabot_alert_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    alert_number: i64,
    mutation: DependabotAlertMutation,
) -> Result<Option<DependabotAlertDetail>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    if !can_write_repository(pool, &repository, actor_user_id).await? {
        return Err(RepositoryError::PermissionDenied);
    }
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    if alert_number <= 0 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "alert id must be a positive number".to_owned(),
        ));
    }

    update_repository_dependabot_alert(pool, &repository, actor_user_id, alert_number, mutation)
        .await?;
    repository_dependabot_alert_detail_for_repository(
        pool,
        &repository,
        actor_user_id,
        alert_number,
    )
    .await
}

pub async fn bulk_update_repository_dependabot_alerts_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    mutation: DependabotBulkMutation,
) -> Result<Option<DependabotBulkMutationResult>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    if !can_write_repository(pool, &repository, actor_user_id).await? {
        return Err(RepositoryError::PermissionDenied);
    }
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }

    bulk_update_repository_dependabot_alerts(pool, &repository, actor_user_id, mutation)
        .await
        .map(Some)
}

pub async fn create_repository_dependabot_security_update_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    alert_number: i64,
) -> Result<Option<DependabotSecurityUpdateResult>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    if !can_write_repository(pool, &repository, actor_user_id).await? {
        return Err(RepositoryError::PermissionDenied);
    }
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    if alert_number <= 0 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "alert id must be a positive number".to_owned(),
        ));
    }

    create_repository_dependabot_security_update(pool, &repository, actor_user_id, alert_number)
        .await
}

pub async fn repository_security_overview_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositorySecurityOverview>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }

    repository_security_overview_for_repository(pool, &repository, actor_user_id)
        .await
        .map(Some)
}

pub async fn repository_security_policy_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositorySecurityPolicyView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }

    repository_security_policy_for_repository(pool, &repository, actor_user_id)
        .await
        .map(Some)
}

pub async fn upsert_repository_security_policy_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    mutation: SecurityPolicyMutation,
) -> Result<Option<RepositorySecurityPolicyView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    if !can_write_repository(pool, &repository, actor_user_id).await? {
        return Err(RepositoryError::PermissionDenied);
    }
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }

    write_security_policy(pool, &repository, actor_user_id, mutation).await?;
    repository_security_policy_for_repository(pool, &repository, actor_user_id)
        .await
        .map(Some)
}

async fn repository_security_overview_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<RepositorySecurityOverview, RepositoryError> {
    let can_write = can_write_repository(pool, repository, actor_user_id).await?;
    let permission = viewer_permission(pool, repository, actor_user_id, can_write).await?;
    let links = security_links(repository);

    Ok(RepositorySecurityOverview {
        repository: RepositorySecurityRepository {
            id: repository.id,
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            visibility: repository.visibility.as_str().to_owned(),
            default_branch: repository.default_branch.clone(),
            security_href: links.overview_href.clone(),
            policy_href: links.policy_href.clone(),
            advisories_href: links.advisories_href.clone(),
        },
        viewer: SecurityViewer {
            permission,
            can_read: true,
            can_write,
            can_edit_policy: can_write && !repository.is_archived,
            can_view_private_alert_counts: can_write,
        },
        policy: security_policy_summary(pool, repository, can_write).await?,
        features: security_feature_cards(pool, repository, can_write).await?,
        advisories: published_advisories(pool, repository).await?,
        links,
    })
}

async fn repository_code_scanning_alerts_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    query: CodeScanningAlertsQuery<'_>,
) -> Result<CodeScanningAlertsView, RepositoryError> {
    let can_write = can_write_repository(pool, repository, actor_user_id).await?;
    let links = code_scanning_links(repository);
    let filters = normalize_code_scanning_filters(query)?;
    let setting = code_scanning_setting(pool, repository).await?;
    let availability = code_scanning_availability(repository, setting.as_ref());
    let mut alerts = if availability.enabled {
        code_scanning_alert_rows(pool, repository).await?
    } else {
        Vec::new()
    };
    let counts_source = alerts.clone();
    apply_code_scanning_filters(&mut alerts, &filters);
    sort_code_scanning_alerts(&mut alerts, &filters.sort);
    let visible = alerts.len() as i64;

    Ok(CodeScanningAlertsView {
        repository: security_repository(repository, &dependabot_links(repository)),
        viewer: security_viewer(pool, repository, actor_user_id, can_write).await?,
        availability,
        filters,
        counts: code_scanning_counts(&counts_source, visible),
        alerts,
        tools: code_scanning_tool_statuses(pool, repository).await?,
        branches: code_scanning_branch_filters(&counts_source, query.branch).await?,
        links,
        freshness: DependabotAlertFreshness {
            computed_at: Utc::now(),
            cadence: "updated when SARIF analysis is uploaded".to_owned(),
        },
    })
}

async fn repository_code_scanning_alert_detail_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    alert_number: i64,
) -> Result<Option<CodeScanningAlertDetail>, RepositoryError> {
    let can_write = can_write_repository(pool, repository, actor_user_id).await?;
    let links = code_scanning_links(repository);
    let setting = code_scanning_setting(pool, repository).await?;
    let availability = code_scanning_availability(repository, setting.as_ref());
    if !availability.enabled {
        return Ok(None);
    }
    let Some(alert) = code_scanning_alert_rows(pool, repository)
        .await?
        .into_iter()
        .find(|alert| alert.number == alert_number)
    else {
        return Ok(None);
    };
    let row = sqlx::query(
        r#"
        SELECT code_scanning_alerts.code_snippet,
               code_scanning_alerts.help_markdown,
               code_scanning_alerts.help_uri,
               code_scanning_alerts.rule_description,
               code_scanning_runs.commit_oid
        FROM code_scanning_alerts
        LEFT JOIN code_scanning_runs ON code_scanning_runs.id = code_scanning_alerts.run_id
        WHERE code_scanning_alerts.id = $1
        "#,
    )
    .bind(alert.id)
    .fetch_one(pool)
    .await?;

    Ok(Some(CodeScanningAlertDetail {
        repository: security_repository(repository, &dependabot_links(repository)),
        viewer: security_viewer(pool, repository, actor_user_id, can_write).await?,
        availability,
        location: CodeScanningLocation {
            path: alert.path.clone(),
            path_href: alert.path_href.clone(),
            raw_href: repository_raw_href(repository, &alert.ref_name, &alert.path),
            start_line: alert.start_line,
            end_line: alert.end_line,
            code_snippet: row.get("code_snippet"),
            ref_name: alert.ref_name.clone(),
            commit_oid: row.get("commit_oid"),
        },
        rule: CodeScanningRuleDetail {
            id: alert.rule_id.clone(),
            name: alert.rule_name.clone(),
            description: row.get("rule_description"),
            help_markdown: row.get("help_markdown"),
            help_uri: row.get("help_uri"),
        },
        timeline: code_scanning_alert_timeline(pool, alert.id).await?,
        assignee_options: code_scanning_assignment_options(pool, repository, alert.id).await?,
        linked_issue: CodeScanningLinkedIssueState {
            issue: alert.linked_issue.clone(),
            can_link: can_write && !repository.is_archived,
            create_href: (can_write && !repository.is_archived).then(|| {
                format!(
                    "/api/repos/{}/{}/security/code-scanning/{}/issue",
                    repository.owner_login, repository.name, alert.number
                )
            }),
        },
        alert,
        links,
    }))
}

async fn update_repository_code_scanning_alert(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    alert_number: i64,
    mutation: CodeScanningAlertMutation,
) -> Result<(), RepositoryError> {
    let setting = code_scanning_setting(pool, repository).await?;
    let availability = code_scanning_availability(repository, setting.as_ref());
    if !availability.enabled {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "Code scanning alerts are disabled for this repository".to_owned(),
        ));
    }

    let alert = sqlx::query(
        "SELECT id, state, fixed_at FROM code_scanning_alerts WHERE repository_id = $1 AND number = $2",
    )
    .bind(repository.id)
    .bind(alert_number)
    .fetch_optional(pool)
    .await?;
    let Some(alert) = alert else {
        return Err(RepositoryError::NotFound);
    };
    let alert_id: Uuid = alert.get("id");
    let state: String = alert.get("state");
    let fixed_at: Option<DateTime<Utc>> = alert.get("fixed_at");

    match mutation.action.as_str() {
        "dismiss" => {
            if state != "open" {
                return Err(RepositoryError::InvalidDependencyGraphQuery(
                    "only open code scanning alerts can be dismissed".to_owned(),
                ));
            }
            let reason =
                normalize_code_scanning_dismissal_reason(mutation.dismissal_reason.as_deref())?;
            let comment =
                normalize_dependabot_dismissal_comment(mutation.dismissal_comment.as_deref())?;
            sqlx::query(
                r#"
                UPDATE code_scanning_alerts
                SET state = 'dismissed',
                    dismissed_reason = $3,
                    dismissed_comment = $4,
                    dismissed_by_user_id = $5,
                    dismissed_at = now(),
                    updated_at = now()
                WHERE repository_id = $1 AND id = $2
                "#,
            )
            .bind(repository.id)
            .bind(alert_id)
            .bind(&reason)
            .bind(&comment)
            .bind(actor_user_id)
            .execute(pool)
            .await?;
            record_code_scanning_alert_event(
                pool,
                repository,
                alert_id,
                actor_user_id,
                "dismissed",
                &format!("Dismissed this alert as {reason}."),
                json!({ "reason": reason, "hasComment": comment.is_some() }),
            )
            .await?;
            notify_code_scanning_alert_assignees(
                pool,
                repository,
                alert_id,
                "Code scanning alert dismissed",
                "security_alert",
            )
            .await?;
        }
        "reopen" => {
            if fixed_at.is_some() || state == "fixed" {
                return Err(RepositoryError::InvalidDependencyGraphQuery(
                    "fixed code scanning alerts cannot be reopened".to_owned(),
                ));
            }
            if state != "dismissed" {
                return Err(RepositoryError::InvalidDependencyGraphQuery(
                    "only dismissed code scanning alerts can be reopened".to_owned(),
                ));
            }
            sqlx::query(
                r#"
                UPDATE code_scanning_alerts
                SET state = 'open',
                    dismissed_reason = NULL,
                    dismissed_comment = NULL,
                    dismissed_by_user_id = NULL,
                    dismissed_at = NULL,
                    updated_at = now()
                WHERE repository_id = $1 AND id = $2
                "#,
            )
            .bind(repository.id)
            .bind(alert_id)
            .execute(pool)
            .await?;
            record_code_scanning_alert_event(
                pool,
                repository,
                alert_id,
                actor_user_id,
                "reopened",
                "Reopened this code scanning alert.",
                json!({ "previousState": state }),
            )
            .await?;
            notify_code_scanning_alert_assignees(
                pool,
                repository,
                alert_id,
                "Code scanning alert reopened",
                "security_alert",
            )
            .await?;
        }
        "assign" => {
            let assignee_ids = mutation.assignee_ids.unwrap_or_default();
            if assignee_ids.len() > 25 {
                return Err(RepositoryError::InvalidDependencyGraphQuery(
                    "code scanning alert assignment is limited to 25 users".to_owned(),
                ));
            }
            let options = code_scanning_assignment_options(pool, repository, alert_id).await?;
            for assignee_id in &assignee_ids {
                if !options.iter().any(|option| option.id == *assignee_id) {
                    return Err(RepositoryError::InvalidDependencyGraphQuery(
                        "code scanning alert assignee must have repository access".to_owned(),
                    ));
                }
            }
            sqlx::query("DELETE FROM code_scanning_alert_assignees WHERE alert_id = $1")
                .bind(alert_id)
                .execute(pool)
                .await?;
            for assignee_id in &assignee_ids {
                sqlx::query(
                    "INSERT INTO code_scanning_alert_assignees (alert_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                )
                .bind(alert_id)
                .bind(assignee_id)
                .execute(pool)
                .await?;
            }
            record_code_scanning_alert_event(
                pool,
                repository,
                alert_id,
                actor_user_id,
                "assigned",
                if assignee_ids.is_empty() {
                    "Cleared code scanning alert assignees."
                } else {
                    "Updated code scanning alert assignees."
                },
                json!({ "assigneeCount": assignee_ids.len() }),
            )
            .await?;
            notify_code_scanning_alert_assignees(
                pool,
                repository,
                alert_id,
                "Code scanning alert assigned",
                "assign",
            )
            .await?;
        }
        "link_issue" => {
            let Some(issue_id) = mutation.linked_issue_id else {
                return Err(RepositoryError::InvalidDependencyGraphQuery(
                    "linked issue id is required".to_owned(),
                ));
            };
            let issue_number = code_scanning_link_existing_issue(
                pool,
                repository,
                alert_id,
                issue_id,
                actor_user_id,
            )
            .await?;
            record_code_scanning_alert_event(
                pool,
                repository,
                alert_id,
                actor_user_id,
                "issue_linked",
                &format!("Linked issue #{issue_number} to this alert."),
                json!({ "issueId": issue_id, "issueNumber": issue_number }),
            )
            .await?;
        }
        _ => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "code scanning alert action must be dismiss, reopen, assign, or link_issue"
                    .to_owned(),
            ))
        }
    }

    Ok(())
}

async fn create_or_link_repository_code_scanning_issue(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    alert_number: i64,
) -> Result<(), RepositoryError> {
    let Some(detail) = repository_code_scanning_alert_detail_for_repository(
        pool,
        repository,
        actor_user_id,
        alert_number,
    )
    .await?
    else {
        return Err(RepositoryError::NotFound);
    };
    if detail.linked_issue.issue.is_some() {
        return Ok(());
    }

    let issue_number = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(max(number), 0) + 1 FROM issues WHERE repository_id = $1",
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let title = format!("Code scanning: {}", detail.rule.name);
    let body = format!(
        "Code scanning reported `{}` in `{}` at line {}.\n\n{}\n\nRule: `{}`\nTool: `{}`\nSeverity: `{}`{}\n\nRemediation:\n{}",
        detail.alert.message,
        detail.location.path,
        detail.location.start_line,
        detail.rule.description.as_deref().unwrap_or("Review the affected path and apply the recommended remediation."),
        detail.rule.id,
        detail.alert.tool_name,
        detail.alert.security_severity.as_deref().unwrap_or(&detail.alert.severity),
        detail
            .rule
            .help_uri
            .as_deref()
            .map(|href| format!("\nReference: {href}"))
            .unwrap_or_default(),
        detail
            .rule
            .help_markdown
            .as_deref()
            .unwrap_or("Assess the data flow, add validation, and rerun code scanning.")
    );
    let issue_id: Uuid = sqlx::query_scalar(
        "INSERT INTO issues (repository_id, number, title, body, author_user_id) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(repository.id)
    .bind(issue_number)
    .bind(&title)
    .bind(&body)
    .bind(actor_user_id)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        "UPDATE code_scanning_alerts SET linked_issue_id = $3, updated_at = now() WHERE repository_id = $1 AND id = $2 AND linked_issue_id IS NULL",
    )
    .bind(repository.id)
    .bind(detail.alert.id)
    .bind(issue_id)
    .execute(pool)
    .await?;

    record_code_scanning_alert_event(
        pool,
        repository,
        detail.alert.id,
        actor_user_id,
        "issue_linked",
        &format!("Created and linked issue #{issue_number}."),
        json!({ "issueId": issue_id, "issueNumber": issue_number }),
    )
    .await?;
    notify_code_scanning_alert_assignees(
        pool,
        repository,
        detail.alert.id,
        "Code scanning alert linked to an issue",
        "mention",
    )
    .await?;
    Ok(())
}

async fn repository_security_policy_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<RepositorySecurityPolicyView, RepositoryError> {
    let can_write = can_write_repository(pool, repository, actor_user_id).await?;
    let permission = viewer_permission(pool, repository, actor_user_id, can_write).await?;
    let links = security_links(repository);

    Ok(RepositorySecurityPolicyView {
        repository: RepositorySecurityRepository {
            id: repository.id,
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            visibility: repository.visibility.as_str().to_owned(),
            default_branch: repository.default_branch.clone(),
            security_href: links.overview_href.clone(),
            policy_href: links.policy_href.clone(),
            advisories_href: links.advisories_href.clone(),
        },
        viewer: SecurityViewer {
            permission,
            can_read: true,
            can_write,
            can_edit_policy: can_write && !repository.is_archived,
            can_view_private_alert_counts: can_write,
        },
        policy: security_policy_document(pool, repository, can_write).await?,
        links,
    })
}

async fn viewer_permission(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    can_write: bool,
) -> Result<String, RepositoryError> {
    if repository.owner_user_id == Some(actor_user_id) {
        return Ok("owner".to_owned());
    }
    if can_write {
        return Ok("write".to_owned());
    }
    if repository.visibility == RepositoryVisibility::Public {
        return Ok("read".to_owned());
    }
    let role =
        super::repositories::repository_permission_for_user(pool, repository.id, actor_user_id)
            .await?
            .map(|permission| permission.role.as_str().to_owned())
            .unwrap_or_else(|| "read".to_owned());
    Ok(role)
}

async fn security_policy_summary(
    pool: &PgPool,
    repository: &Repository,
    can_write: bool,
) -> Result<SecurityPolicySummary, RepositoryError> {
    if let Some(row) = sqlx::query(
        r#"
        SELECT repository_security_policies.path,
               repository_security_policies.ref_name,
               repository_security_policies.blob_oid,
               repository_security_policies.content_sha,
               repository_security_policies.markdown,
               repository_security_policies.rendered_html,
               repository_security_policies.updated_at,
               commits.oid AS commit_oid,
               commits.message AS commit_message,
               commits.committed_at AS committed_at
        FROM repository_security_policies
        LEFT JOIN commits ON commits.id = repository_security_policies.source_commit_id
        WHERE repository_security_policies.repository_id = $1
          AND repository_security_policies.published = true
        ORDER BY CASE WHEN lower(path) = 'security.md' THEN 0 ELSE 1 END, updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .fetch_optional(pool)
    .await?
    {
        return policy_from_row(repository, row, can_write).await;
    }

    let row = sqlx::query(
        r#"
        SELECT repository_files.path,
               $2::text AS ref_name,
               repository_files.oid AS blob_oid,
               repository_files.content AS markdown,
               commits.committed_at AS updated_at
        FROM repository_files
        JOIN repository_git_refs
          ON repository_git_refs.repository_id = repository_files.repository_id
         AND repository_git_refs.target_commit_id = repository_files.commit_id
        JOIN commits ON commits.id = repository_files.commit_id
        WHERE repository_files.repository_id = $1
          AND repository_git_refs.name IN ($2, 'refs/heads/' || $2)
          AND lower(repository_files.path) IN ('security.md', '.github/security.md', 'docs/security.md')
        ORDER BY CASE lower(repository_files.path)
            WHEN 'security.md' THEN 0
            WHEN '.github/security.md' THEN 1
            ELSE 2
        END
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(&repository.default_branch)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(SecurityPolicySummary {
            exists: false,
            path: None,
            ref_name: None,
            blob_oid: None,
            content_sha: None,
            html: None,
            source_href: None,
            raw_href: None,
            history_href: None,
            edit_href: if can_write {
                Some(format!(
                    "/{}/{}/security/policy/edit",
                    repository.owner_login, repository.name
                ))
            } else {
                None
            },
            updated_at: None,
            empty_state: if can_write {
                "No SECURITY.md policy has been published. Maintainers can start setup.".to_owned()
            } else {
                "No security policy has been published for this repository.".to_owned()
            },
        });
    };

    let path: String = row.get("path");
    let ref_name: String = row.get("ref_name");
    let markdown: String = row.get("markdown");
    let content_sha = markdown_sha(&markdown);
    let rendered = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            markdown: markdown.clone(),
            repository_id: Some(repository.id),
            owner: Some(repository.owner_login.clone()),
            repo: Some(repository.name.clone()),
            ref_name: Some(ref_name.clone()),
            enable_task_toggles: Some(false),
        },
    )
    .await
    .map_err(markdown_error)?;

    Ok(SecurityPolicySummary {
        exists: true,
        path: Some(path.clone()),
        ref_name: Some(ref_name.clone()),
        blob_oid: row.get("blob_oid"),
        content_sha: Some(content_sha),
        html: Some(rendered.html),
        source_href: Some(repository_blob_href(repository, &ref_name, &path)),
        raw_href: Some(repository_raw_href(repository, &ref_name, &path)),
        history_href: Some(repository_history_href(repository, &ref_name, &path)),
        edit_href: can_write.then(|| {
            format!(
                "/{}/{}/security/policy/edit?path={}",
                repository.owner_login,
                repository.name,
                percent_encode_path(&path)
            )
        }),
        updated_at: row.get("updated_at"),
        empty_state: String::new(),
    })
}

async fn security_policy_document(
    pool: &PgPool,
    repository: &Repository,
    can_write: bool,
) -> Result<SecurityPolicyDocument, RepositoryError> {
    if let Some(row) = sqlx::query(
        r#"
        SELECT repository_security_policies.path,
               repository_security_policies.ref_name,
               repository_security_policies.blob_oid,
               repository_security_policies.content_sha,
               repository_security_policies.markdown,
               repository_security_policies.rendered_html,
               repository_security_policies.updated_at,
               commits.oid AS commit_oid,
               commits.message AS commit_message,
               commits.committed_at AS committed_at
        FROM repository_security_policies
        LEFT JOIN commits ON commits.id = repository_security_policies.source_commit_id
        WHERE repository_security_policies.repository_id = $1
          AND repository_security_policies.published = true
        ORDER BY CASE WHEN lower(path) = 'security.md' THEN 0 ELSE 1 END, updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .fetch_optional(pool)
    .await?
    {
        return policy_document_from_row(repository, row, can_write);
    }

    let row = sqlx::query(
        r#"
        SELECT repository_files.path,
               $2::text AS ref_name,
               repository_files.oid AS blob_oid,
               repository_files.content AS markdown,
               commits.oid AS commit_oid,
               commits.message AS commit_message,
               commits.committed_at AS committed_at
        FROM repository_files
        JOIN repository_git_refs
          ON repository_git_refs.repository_id = repository_files.repository_id
         AND repository_git_refs.target_commit_id = repository_files.commit_id
        JOIN commits ON commits.id = repository_files.commit_id
        WHERE repository_files.repository_id = $1
          AND repository_git_refs.name IN ($2, 'refs/heads/' || $2)
          AND lower(repository_files.path) IN ('security.md', '.github/security.md', 'docs/security.md')
        ORDER BY CASE lower(repository_files.path)
            WHEN 'security.md' THEN 0
            WHEN '.github/security.md' THEN 1
            ELSE 2
        END
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(&repository.default_branch)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(SecurityPolicyDocument {
            exists: false,
            path: None,
            ref_name: None,
            blob_oid: None,
            content_sha: None,
            markdown: None,
            html: None,
            outline: Vec::new(),
            source_href: None,
            raw_href: None,
            history_href: None,
            edit_href: if can_write {
                Some(format!(
                    "/{}/{}/security/policy/edit",
                    repository.owner_login, repository.name
                ))
            } else {
                None
            },
            latest_commit: None,
            updated_at: None,
            empty_state: if can_write {
                "No SECURITY.md policy has been published. Maintainers can start setup.".to_owned()
            } else {
                "No security policy has been published for this repository.".to_owned()
            },
        });
    };

    let path: String = row.get("path");
    let ref_name: String = row.get("ref_name");
    let markdown: String = row.get("markdown");
    let rendered = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            markdown: markdown.clone(),
            repository_id: Some(repository.id),
            owner: Some(repository.owner_login.clone()),
            repo: Some(repository.name.clone()),
            ref_name: Some(ref_name.clone()),
            enable_task_toggles: Some(false),
        },
    )
    .await
    .map_err(markdown_error)?;
    let commit_oid: String = row.get("commit_oid");
    let committed_at: DateTime<Utc> = row.get("committed_at");

    Ok(SecurityPolicyDocument {
        exists: true,
        path: Some(path.clone()),
        ref_name: Some(ref_name.clone()),
        blob_oid: row.get("blob_oid"),
        content_sha: Some(rendered.content_sha.clone()),
        markdown: Some(markdown),
        outline: policy_heading_outline(&rendered.html),
        html: Some(rendered.html),
        source_href: Some(repository_blob_href(repository, &ref_name, &path)),
        raw_href: Some(repository_raw_href(repository, &ref_name, &path)),
        history_href: Some(repository_history_href(repository, &ref_name, &path)),
        edit_href: can_write.then(|| {
            format!(
                "/{}/{}/security/policy/edit?path={}",
                repository.owner_login,
                repository.name,
                percent_encode_path(&path)
            )
        }),
        latest_commit: Some(SecurityPolicyCommit {
            short_oid: commit_oid.chars().take(7).collect(),
            href: format!(
                "/{}/{}/commit/{}",
                repository.owner_login,
                repository.name,
                percent_encode_segment(&commit_oid)
            ),
            oid: commit_oid,
            message: row.get("commit_message"),
            committed_at,
        }),
        updated_at: Some(committed_at),
        empty_state: String::new(),
    })
}

async fn policy_from_row(
    repository: &Repository,
    row: sqlx::postgres::PgRow,
    can_write: bool,
) -> Result<SecurityPolicySummary, RepositoryError> {
    let path: String = row.get("path");
    let ref_name: String = row.get("ref_name");
    Ok(SecurityPolicySummary {
        exists: true,
        path: Some(path.clone()),
        ref_name: Some(ref_name.clone()),
        blob_oid: row.get("blob_oid"),
        content_sha: Some(row.get("content_sha")),
        html: Some(row.get("rendered_html")),
        source_href: Some(repository_blob_href(repository, &ref_name, &path)),
        raw_href: Some(repository_raw_href(repository, &ref_name, &path)),
        history_href: Some(repository_history_href(repository, &ref_name, &path)),
        edit_href: can_write.then(|| {
            format!(
                "/{}/{}/security/policy/edit?path={}",
                repository.owner_login,
                repository.name,
                percent_encode_path(&path)
            )
        }),
        updated_at: row.get("updated_at"),
        empty_state: String::new(),
    })
}

fn policy_document_from_row(
    repository: &Repository,
    row: sqlx::postgres::PgRow,
    can_write: bool,
) -> Result<SecurityPolicyDocument, RepositoryError> {
    let path: String = row.get("path");
    let ref_name: String = row.get("ref_name");
    let html: String = row.get("rendered_html");
    Ok(SecurityPolicyDocument {
        exists: true,
        path: Some(path.clone()),
        ref_name: Some(ref_name.clone()),
        blob_oid: row.get("blob_oid"),
        content_sha: Some(row.get("content_sha")),
        markdown: Some(row.get("markdown")),
        outline: policy_heading_outline(&html),
        html: Some(html),
        source_href: Some(repository_blob_href(repository, &ref_name, &path)),
        raw_href: Some(repository_raw_href(repository, &ref_name, &path)),
        history_href: Some(repository_history_href(repository, &ref_name, &path)),
        edit_href: can_write.then(|| {
            format!(
                "/{}/{}/security/policy/edit?path={}",
                repository.owner_login,
                repository.name,
                percent_encode_path(&path)
            )
        }),
        latest_commit: match (
            row.try_get::<Option<String>, _>("commit_oid")
                .ok()
                .flatten(),
            row.try_get::<Option<String>, _>("commit_message")
                .ok()
                .flatten(),
            row.try_get::<Option<DateTime<Utc>>, _>("committed_at")
                .ok()
                .flatten(),
        ) {
            (Some(oid), Some(message), Some(committed_at)) => Some(SecurityPolicyCommit {
                short_oid: oid.chars().take(7).collect(),
                href: format!(
                    "/{}/{}/commit/{}",
                    repository.owner_login,
                    repository.name,
                    percent_encode_segment(&oid)
                ),
                oid,
                message,
                committed_at,
            }),
            _ => None,
        },
        updated_at: row.get("updated_at"),
        empty_state: String::new(),
    })
}

async fn security_feature_cards(
    pool: &PgPool,
    repository: &Repository,
    can_view_counts: bool,
) -> Result<Vec<SecurityFeatureCard>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT feature_key, status, summary, alert_count, private_count, config_href, updated_at
        FROM repository_security_feature_settings
        WHERE repository_id = $1
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;
    let mut cards = default_feature_cards(repository);
    for row in rows {
        let key: String = row.get("feature_key");
        if let Some(card) = cards.iter_mut().find(|card| card.key == key) {
            card.status = row.get("status");
            card.summary = row.get("summary");
            let alert_count = row.get::<i64, _>("alert_count");
            let private_count = row.get::<i64, _>("private_count");
            card.alert_count = can_view_counts.then_some(alert_count);
            card.private_count = can_view_counts.then_some(private_count);
            card.config_href = row.get("config_href");
            card.updated_at = row.get("updated_at");
        }
    }
    Ok(cards)
}

fn default_feature_cards(repository: &Repository) -> Vec<SecurityFeatureCard> {
    [
        (
            "dependabot",
            "Dependabot",
            "Dependency update and vulnerability alert coverage.",
            format!(
                "/{}/{}/security/dependabot",
                repository.owner_login, repository.name
            ),
        ),
        (
            "code_scanning",
            "Code scanning",
            "Static analysis findings from configured workflows.",
            format!(
                "/{}/{}/security/code-scanning",
                repository.owner_login, repository.name
            ),
        ),
        (
            "secret_scanning",
            "Secret scanning",
            "Credential exposure detection for committed content.",
            format!(
                "/{}/{}/security/secret-scanning",
                repository.owner_login, repository.name
            ),
        ),
        (
            "private_vulnerability_reporting",
            "Private vulnerability reporting",
            "Coordinated disclosure intake for repository maintainers.",
            format!(
                "/{}/{}/security/advisories/new",
                repository.owner_login, repository.name
            ),
        ),
    ]
    .into_iter()
    .map(|(key, label, summary, href)| SecurityFeatureCard {
        key: key.to_owned(),
        label: label.to_owned(),
        status: "needs_setup".to_owned(),
        summary: summary.to_owned(),
        alert_count: can_never_count(),
        private_count: can_never_count(),
        href,
        config_href: None,
        updated_at: None,
    })
    .collect()
}

const fn can_never_count() -> Option<i64> {
    None
}

async fn published_advisories(
    pool: &PgPool,
    repository: &Repository,
) -> Result<Vec<RepositorySecurityAdvisorySummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT id, advisory_identifier, severity, status, title, summary, package_name,
               vulnerable_range, advisory_href, published_at, updated_at
        FROM repository_security_advisories
        WHERE repository_id = $1 AND status = 'published'
        ORDER BY COALESCE(published_at, updated_at) DESC, advisory_identifier ASC
        LIMIT 10
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(RepositorySecurityAdvisorySummary {
                id: row.get("id"),
                identifier: row.get("advisory_identifier"),
                severity: row.get("severity"),
                status: row.get("status"),
                title: row.get("title"),
                summary: row.get("summary"),
                package_name: row.get("package_name"),
                vulnerable_range: row.get("vulnerable_range"),
                href: row.get("advisory_href"),
                published_at: row.get("published_at"),
                updated_at: row.get("updated_at"),
            })
        })
        .collect()
}

fn security_links(repository: &Repository) -> SecurityOverviewLinks {
    let base = format!("/{}/{}", repository.owner_login, repository.name);
    SecurityOverviewLinks {
        overview_href: format!("{base}/security"),
        policy_href: format!("{base}/security/policy"),
        advisories_href: format!("{base}/security/advisories"),
        dependabot_href: format!("{base}/security/dependabot"),
        code_scanning_href: format!("{base}/security/code-scanning"),
        secret_scanning_href: format!("{base}/security/secret-scanning"),
    }
}

async fn repository_dependabot_alerts_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    query: DependabotAlertsQuery<'_>,
) -> Result<DependabotAlertsView, RepositoryError> {
    let filters = normalize_dependabot_alert_filters(query)?;
    let can_write = can_write_repository(pool, repository, actor_user_id).await?;
    let setting = dependabot_setting(pool, repository).await?;
    let availability = dependabot_availability(repository, setting.as_ref());

    if availability.enabled {
        materialize_dependabot_alerts(pool, repository).await?;
    }

    let mut alerts = if availability.enabled {
        dependabot_alert_rows(pool, repository).await?
    } else {
        Vec::new()
    };
    let all_alerts = alerts.clone();
    apply_dependabot_alert_filters(&mut alerts, &filters);
    sort_dependabot_alerts(&mut alerts, &filters.sort);

    let links = dependabot_links(repository);
    Ok(DependabotAlertsView {
        repository: security_repository(repository, &links),
        viewer: security_viewer(pool, repository, actor_user_id, can_write).await?,
        availability,
        filters,
        counts: dependabot_counts(&all_alerts, alerts.len() as i64),
        alerts,
        packages: dependabot_package_filters(repository, &all_alerts, query.package).await?,
        manifests: dependabot_manifest_filters(repository, &all_alerts, query.manifest).await?,
        links,
        freshness: DependabotAlertFreshness {
            computed_at: Utc::now(),
            cadence: "on repository dependency graph refresh".to_owned(),
        },
    })
}

async fn repository_dependabot_alert_detail_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    alert_number: i64,
) -> Result<Option<DependabotAlertDetail>, RepositoryError> {
    let can_write = can_write_repository(pool, repository, actor_user_id).await?;
    let setting = dependabot_setting(pool, repository).await?;
    let availability = dependabot_availability(repository, setting.as_ref());
    if availability.enabled {
        materialize_dependabot_alerts(pool, repository).await?;
    }

    let Some(alert) = dependabot_alert_rows(pool, repository)
        .await?
        .into_iter()
        .find(|alert| alert.number == alert_number)
    else {
        return Ok(None);
    };
    let links = dependabot_links(repository);
    let timeline = dependabot_alert_timeline(pool, alert.id).await?;
    let assignee_options = dependabot_assignment_options(pool, repository, alert.id).await?;
    let advisory = DependabotAdvisoryDetail {
        identifier: alert.advisory.identifier.clone(),
        severity: alert.advisory.severity.clone(),
        title: alert.advisory.title.clone(),
        href: alert.advisory.href.clone(),
        vulnerable_range: alert
            .vulnerable_requirements
            .clone()
            .unwrap_or_else(|| "See advisory".to_owned()),
        published_at: alert.advisory.published_at,
    };
    let dependency = DependabotDependencyDetail {
        package: alert.package.clone(),
        manifest_path: alert.manifest_path.clone(),
        manifest_href: alert.manifest_href.clone(),
        lockfile_path: alert.lockfile_path.clone(),
        lockfile_href: alert.lockfile_href.clone(),
        current_version: alert.current_version.clone(),
        relationship: alert.relationship.clone(),
    };
    let security_update = dependabot_security_update_state(pool, repository, &alert).await?;

    Ok(Some(DependabotAlertDetail {
        repository: security_repository(repository, &links),
        viewer: security_viewer(pool, repository, actor_user_id, can_write).await?,
        availability,
        alert,
        advisory,
        dependency,
        timeline,
        assignee_options,
        security_update,
        links,
    }))
}

async fn update_repository_dependabot_alert(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    alert_number: i64,
    mutation: DependabotAlertMutation,
) -> Result<(), RepositoryError> {
    let setting = dependabot_setting(pool, repository).await?;
    let availability = dependabot_availability(repository, setting.as_ref());
    if !availability.enabled {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "Dependabot alerts are disabled for this repository".to_owned(),
        ));
    }
    materialize_dependabot_alerts(pool, repository).await?;

    let alert = sqlx::query(
        r#"
        SELECT id, state, fixed_version
        FROM dependabot_alerts
        WHERE repository_id = $1 AND number = $2
        "#,
    )
    .bind(repository.id)
    .bind(alert_number)
    .fetch_optional(pool)
    .await?;
    let Some(alert) = alert else {
        return Err(RepositoryError::NotFound);
    };
    let alert_id: Uuid = alert.get("id");
    let state: String = alert.get("state");
    let fixed_version: Option<String> = alert.get("fixed_version");

    match mutation.action.as_str() {
        "dismiss" => {
            if state != "open" {
                return Err(RepositoryError::InvalidDependencyGraphQuery(
                    "only open Dependabot alerts can be dismissed".to_owned(),
                ));
            }
            let reason =
                normalize_dependabot_dismissal_reason(mutation.dismissal_reason.as_deref())?;
            let comment =
                normalize_dependabot_dismissal_comment(mutation.dismissal_comment.as_deref())?;
            sqlx::query(
                r#"
                UPDATE dependabot_alerts
                SET state = 'dismissed',
                    dismissed_reason = $3,
                    dismissed_comment = $4,
                    dismissed_by_user_id = $5,
                    dismissed_at = now(),
                    updated_at = now()
                WHERE repository_id = $1 AND id = $2
                "#,
            )
            .bind(repository.id)
            .bind(alert_id)
            .bind(&reason)
            .bind(&comment)
            .bind(actor_user_id)
            .execute(pool)
            .await?;
            record_dependabot_alert_event(
                pool,
                repository,
                alert_id,
                actor_user_id,
                "dismissed",
                &format!("Dismissed this alert as {reason}."),
                json!({ "reason": reason, "hasComment": comment.is_some() }),
            )
            .await?;
            notify_dependabot_alert_assignees(
                pool,
                repository,
                alert_id,
                "Dependabot alert dismissed",
                "security_alert",
            )
            .await?;
        }
        "reopen" => {
            if fixed_version.is_some() || state == "fixed" {
                return Err(RepositoryError::InvalidDependencyGraphQuery(
                    "fixed Dependabot alerts cannot be reopened".to_owned(),
                ));
            }
            if state != "dismissed" {
                return Err(RepositoryError::InvalidDependencyGraphQuery(
                    "only dismissed Dependabot alerts can be reopened".to_owned(),
                ));
            }
            sqlx::query(
                r#"
                UPDATE dependabot_alerts
                SET state = 'open',
                    dismissed_reason = NULL,
                    dismissed_comment = NULL,
                    dismissed_by_user_id = NULL,
                    dismissed_at = NULL,
                    updated_at = now()
                WHERE repository_id = $1 AND id = $2
                "#,
            )
            .bind(repository.id)
            .bind(alert_id)
            .execute(pool)
            .await?;
            record_dependabot_alert_event(
                pool,
                repository,
                alert_id,
                actor_user_id,
                "reopened",
                "Reopened this Dependabot alert.",
                json!({ "previousState": state }),
            )
            .await?;
            notify_dependabot_alert_assignees(
                pool,
                repository,
                alert_id,
                "Dependabot alert reopened",
                "security_alert",
            )
            .await?;
        }
        "assign" => {
            let assignee_ids = mutation.assignee_ids.unwrap_or_default();
            if assignee_ids.len() > 25 {
                return Err(RepositoryError::InvalidDependencyGraphQuery(
                    "Dependabot alert assignment is limited to 25 users".to_owned(),
                ));
            }
            let options = dependabot_assignment_options(pool, repository, alert_id).await?;
            for assignee_id in &assignee_ids {
                if !options.iter().any(|option| option.id == *assignee_id) {
                    return Err(RepositoryError::InvalidDependencyGraphQuery(
                        "Dependabot alert assignee must have repository access".to_owned(),
                    ));
                }
            }
            sqlx::query("DELETE FROM dependabot_alert_assignees WHERE alert_id = $1")
                .bind(alert_id)
                .execute(pool)
                .await?;
            for assignee_id in &assignee_ids {
                sqlx::query(
                    "INSERT INTO dependabot_alert_assignees (alert_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                )
                .bind(alert_id)
                .bind(assignee_id)
                .execute(pool)
                .await?;
            }
            record_dependabot_alert_event(
                pool,
                repository,
                alert_id,
                actor_user_id,
                "assigned",
                if assignee_ids.is_empty() {
                    "Cleared Dependabot alert assignees."
                } else {
                    "Updated Dependabot alert assignees."
                },
                json!({ "assigneeCount": assignee_ids.len() }),
            )
            .await?;
            notify_dependabot_alert_assignees(
                pool,
                repository,
                alert_id,
                "Dependabot alert assigned",
                "assign",
            )
            .await?;
        }
        _ => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "Dependabot alert action must be dismiss, reopen, or assign".to_owned(),
            ))
        }
    }

    Ok(())
}

async fn bulk_update_repository_dependabot_alerts(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    mutation: DependabotBulkMutation,
) -> Result<DependabotBulkMutationResult, RepositoryError> {
    let alert_ids = normalize_dependabot_bulk_alert_ids(&mutation.alert_ids)?;
    let action = mutation.action.trim();
    if !matches!(action, "dismiss" | "reopen") {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "Dependabot bulk action must be dismiss or reopen".to_owned(),
        ));
    }
    let dismissal_reason = if action == "dismiss" {
        Some(normalize_dependabot_dismissal_reason(
            mutation.dismissal_reason.as_deref(),
        )?)
    } else {
        None
    };
    let dismissal_comment = if action == "dismiss" {
        normalize_dependabot_dismissal_comment(mutation.dismissal_comment.as_deref())?
    } else {
        None
    };

    let setting = dependabot_setting(pool, repository).await?;
    let availability = dependabot_availability(repository, setting.as_ref());
    if !availability.enabled {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "Dependabot alerts are disabled for this repository".to_owned(),
        ));
    }
    materialize_dependabot_alerts(pool, repository).await?;

    let mut results = Vec::new();
    for alert_id in alert_ids {
        let row = sqlx::query(
            r#"
            SELECT id, number, state, fixed_version
            FROM dependabot_alerts
            WHERE repository_id = $1 AND id = $2
            "#,
        )
        .bind(repository.id)
        .bind(alert_id)
        .fetch_optional(pool)
        .await?;
        let Some(row) = row else {
            results.push(DependabotBulkAlertResult {
                id: alert_id,
                number: 0,
                state: "hidden".to_owned(),
                ok: false,
                message: "Alert was not found or is no longer visible.".to_owned(),
                href: dependabot_links(repository).list_href,
            });
            continue;
        };
        let number: i64 = row.get("number");
        let state: String = row.get("state");
        let fixed_version: Option<String> = row.get("fixed_version");
        let href = format!(
            "/{}/{}/security/dependabot/{}",
            repository.owner_login, repository.name, number
        );

        match action {
            "dismiss" if state != "open" => {
                results.push(DependabotBulkAlertResult {
                    id: alert_id,
                    number,
                    state,
                    ok: false,
                    message: "Only open Dependabot alerts can be dismissed.".to_owned(),
                    href,
                });
            }
            "reopen" if fixed_version.is_some() || state == "fixed" => {
                results.push(DependabotBulkAlertResult {
                    id: alert_id,
                    number,
                    state,
                    ok: false,
                    message: "Fixed Dependabot alerts cannot be reopened.".to_owned(),
                    href,
                });
            }
            "reopen" if state != "dismissed" => {
                results.push(DependabotBulkAlertResult {
                    id: alert_id,
                    number,
                    state,
                    ok: false,
                    message: "Only dismissed Dependabot alerts can be reopened.".to_owned(),
                    href,
                });
            }
            "dismiss" => {
                sqlx::query(
                    r#"
                    UPDATE dependabot_alerts
                    SET state = 'dismissed',
                        dismissed_reason = $3,
                        dismissed_comment = $4,
                        dismissed_by_user_id = $5,
                        dismissed_at = now(),
                        updated_at = now()
                    WHERE repository_id = $1 AND id = $2
                    "#,
                )
                .bind(repository.id)
                .bind(alert_id)
                .bind(dismissal_reason.as_deref())
                .bind(dismissal_comment.as_deref())
                .bind(actor_user_id)
                .execute(pool)
                .await?;
                record_dependabot_alert_event(
                    pool,
                    repository,
                    alert_id,
                    actor_user_id,
                    "bulk_dismissed",
                    "Dismissed this alert from bulk triage.",
                    json!({
                        "reason": dismissal_reason,
                        "hasComment": dismissal_comment.is_some(),
                    }),
                )
                .await?;
                notify_dependabot_alert_assignees(
                    pool,
                    repository,
                    alert_id,
                    "Dependabot alert dismissed",
                    "security_alert",
                )
                .await?;
                results.push(DependabotBulkAlertResult {
                    id: alert_id,
                    number,
                    state: "dismissed".to_owned(),
                    ok: true,
                    message: "Dismissed.".to_owned(),
                    href,
                });
            }
            "reopen" => {
                sqlx::query(
                    r#"
                    UPDATE dependabot_alerts
                    SET state = 'open',
                        dismissed_reason = NULL,
                        dismissed_comment = NULL,
                        dismissed_by_user_id = NULL,
                        dismissed_at = NULL,
                        updated_at = now()
                    WHERE repository_id = $1 AND id = $2
                    "#,
                )
                .bind(repository.id)
                .bind(alert_id)
                .execute(pool)
                .await?;
                record_dependabot_alert_event(
                    pool,
                    repository,
                    alert_id,
                    actor_user_id,
                    "bulk_reopened",
                    "Reopened this alert from bulk triage.",
                    json!({ "previousState": state }),
                )
                .await?;
                notify_dependabot_alert_assignees(
                    pool,
                    repository,
                    alert_id,
                    "Dependabot alert reopened",
                    "security_alert",
                )
                .await?;
                results.push(DependabotBulkAlertResult {
                    id: alert_id,
                    number,
                    state: "open".to_owned(),
                    ok: true,
                    message: "Reopened.".to_owned(),
                    href,
                });
            }
            _ => {}
        }
    }

    let updated_count = results.iter().filter(|result| result.ok).count();
    let links = dependabot_links(repository);
    Ok(DependabotBulkMutationResult {
        repository: security_repository(repository, &links),
        requested_count: results.len(),
        updated_count,
        results,
        message: format!("{updated_count} Dependabot alerts updated."),
    })
}

async fn create_repository_dependabot_security_update(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    alert_number: i64,
) -> Result<Option<DependabotSecurityUpdateResult>, RepositoryError> {
    let setting = dependabot_setting(pool, repository).await?;
    let availability = dependabot_availability(repository, setting.as_ref());
    if !availability.enabled {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "Dependabot alerts are disabled for this repository".to_owned(),
        ));
    }
    materialize_dependabot_alerts(pool, repository).await?;
    let Some(alert) = dependabot_alert_rows(pool, repository)
        .await?
        .into_iter()
        .find(|alert| alert.number == alert_number)
    else {
        return Ok(None);
    };
    if alert.state != "open" {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "security update pull requests require an open Dependabot alert".to_owned(),
        ));
    }
    if !matches!(alert.package.ecosystem.as_str(), "npm" | "cargo" | "pip") {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "security update pull requests are unsupported for this ecosystem".to_owned(),
        ));
    }

    if let Some(existing) = dependabot_existing_security_update_pr(pool, repository, &alert).await?
    {
        return Ok(Some(DependabotSecurityUpdateResult {
            alert,
            status: "linked".to_owned(),
            branch: existing.0,
            commit_oid: None,
            pull_request_href: Some(existing.1),
            message: "A security update pull request already exists for this alert.".to_owned(),
        }));
    }

    let branch = dependabot_security_update_branch(&alert);
    let default_commit = current_branch_commit(pool, repository.id, &repository.default_branch)
        .await?
        .ok_or_else(|| RepositoryError::DefaultBranchNotFound(repository.default_branch.clone()))?;
    let mut files = current_branch_files(pool, repository.id, Some(default_commit.id)).await?;
    let target_version = dependabot_security_update_version(&alert)?;
    let Some(manifest) = files
        .iter_mut()
        .find(|file| file.path.eq_ignore_ascii_case(&alert.manifest_path))
    else {
        return Err(RepositoryError::PathNotFound);
    };
    let next_content = update_dependency_manifest_content(
        &alert.package.ecosystem,
        &manifest.content,
        &alert.package.name,
        &target_version,
    )?;
    if next_content == manifest.content {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "security update did not change the dependency manifest".to_owned(),
        ));
    }
    manifest.content = next_content;
    manifest.oid = deterministic_content_oid("blob", &manifest.content);
    manifest.byte_size = manifest.content.len() as i64;

    let tree_oid = deterministic_content_oid(
        "tree",
        &files
            .iter()
            .map(|file| format!("{}:{}:{}", file.path, file.oid, file.byte_size))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    let title = format!("Bump {} to {}", alert.package.name, target_version);
    let body = format!(
        "Security update for {}.\n\n- Alert: {}\n- Advisory: {}\n- Manifest: `{}`\n",
        alert.package.name, alert.href, alert.advisory.identifier, alert.manifest_path
    );
    let commit_oid = deterministic_content_oid(
        "commit",
        &format!(
            "{}:{}:{}:{}:{}",
            repository.id, branch, tree_oid, alert.id, target_version
        ),
    );
    let commit = replace_repository_snapshot(
        pool,
        repository.id,
        RepositorySnapshot {
            commit: CreateCommit {
                oid: commit_oid.clone(),
                author_user_id: Some(actor_user_id),
                committer_user_id: Some(actor_user_id),
                message: title.clone(),
                tree_oid: Some(tree_oid),
                parent_oids: vec![default_commit.oid],
                committed_at: Utc::now(),
            },
            branch_name: branch.clone(),
            files,
        },
    )
    .await?;

    let pull = create_pull_request(
        pool,
        CreatePullRequest {
            repository_id: repository.id,
            actor_user_id,
            title,
            body: Some(body),
            head_ref: branch.clone(),
            base_ref: repository.default_branch.clone(),
            head_repository_id: None,
            is_draft: false,
            label_ids: Vec::new(),
            milestone_id: None,
            assignee_user_ids: Vec::new(),
            reviewer_user_ids: Vec::new(),
            template_slug: None,
        },
    )
    .await
    .map_err(collaboration_to_repository_error)?;

    sqlx::query(
        r#"
        UPDATE dependabot_alerts
        SET security_update_pull_request_id = $3, updated_at = now()
        WHERE repository_id = $1 AND id = $2
        "#,
    )
    .bind(repository.id)
    .bind(alert.id)
    .bind(pull.pull_request.id)
    .execute(pool)
    .await?;
    record_dependabot_alert_event(
        pool,
        repository,
        alert.id,
        actor_user_id,
        "security_update_opened",
        "Opened a security update pull request.",
        json!({
            "pullRequestId": pull.pull_request.id,
            "pullRequestNumber": pull.pull_request.number,
            "branch": branch,
            "commitOid": commit.oid,
        }),
    )
    .await?;
    notify_dependabot_alert_assignees(
        pool,
        repository,
        alert.id,
        "Dependabot security update opened",
        "security_alert",
    )
    .await?;

    Ok(Some(DependabotSecurityUpdateResult {
        alert,
        status: "created".to_owned(),
        branch,
        commit_oid: Some(commit.oid),
        pull_request_href: Some(pull.href),
        message: "Security update pull request created.".to_owned(),
    }))
}

async fn security_viewer(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    can_write: bool,
) -> Result<SecurityViewer, RepositoryError> {
    Ok(SecurityViewer {
        permission: viewer_permission(pool, repository, actor_user_id, can_write).await?,
        can_read: true,
        can_write,
        can_edit_policy: can_write && !repository.is_archived,
        can_view_private_alert_counts: can_write,
    })
}

fn security_repository(
    repository: &Repository,
    _links: &DependabotAlertLinks,
) -> RepositorySecurityRepository {
    RepositorySecurityRepository {
        id: repository.id,
        owner_login: repository.owner_login.clone(),
        name: repository.name.clone(),
        visibility: repository.visibility.as_str().to_owned(),
        default_branch: repository.default_branch.clone(),
        security_href: format!("/{}/{}/security", repository.owner_login, repository.name),
        policy_href: format!(
            "/{}/{}/security/policy",
            repository.owner_login, repository.name
        ),
        advisories_href: format!(
            "/{}/{}/security/advisories",
            repository.owner_login, repository.name
        ),
    }
}

async fn dependabot_setting(
    pool: &PgPool,
    repository: &Repository,
) -> Result<Option<sqlx::postgres::PgRow>, RepositoryError> {
    sqlx::query(
        r#"
        SELECT status, summary, config_href
        FROM repository_security_feature_settings
        WHERE repository_id = $1 AND feature_key = 'dependabot'
        "#,
    )
    .bind(repository.id)
    .fetch_optional(pool)
    .await
    .map_err(RepositoryError::from)
}

async fn code_scanning_setting(
    pool: &PgPool,
    repository: &Repository,
) -> Result<Option<sqlx::postgres::PgRow>, RepositoryError> {
    sqlx::query(
        r#"
        SELECT status, summary, config_href
        FROM repository_security_feature_settings
        WHERE repository_id = $1 AND feature_key = 'code_scanning'
        "#,
    )
    .bind(repository.id)
    .fetch_optional(pool)
    .await
    .map_err(RepositoryError::from)
}

fn code_scanning_availability(
    repository: &Repository,
    setting: Option<&sqlx::postgres::PgRow>,
) -> CodeScanningAvailability {
    let status = setting
        .map(|row| row.get::<String, _>("status"))
        .unwrap_or_else(|| "disabled".to_owned());
    let enabled = status == "enabled";
    CodeScanningAvailability {
        enabled,
        indexed: enabled,
        message: if enabled {
            "Code scanning alerts are normalized from SARIF analysis and Actions runs.".to_owned()
        } else {
            setting
                .map(|row| row.get::<String, _>("summary"))
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "Code scanning is not enabled for this repository.".to_owned())
        },
        disabled_reason: (!enabled).then_some(status),
        settings_href: setting
            .and_then(|row| row.get::<Option<String>, _>("config_href"))
            .or_else(|| {
                Some(format!(
                    "/{}/{}/security/code-scanning/setup",
                    repository.owner_login, repository.name
                ))
            }),
    }
}

fn dependabot_availability(
    repository: &Repository,
    setting: Option<&sqlx::postgres::PgRow>,
) -> DependabotAlertsAvailability {
    let status = setting
        .map(|row| row.get::<String, _>("status"))
        .unwrap_or_else(|| "enabled".to_owned());
    let enabled = status == "enabled";
    DependabotAlertsAvailability {
        enabled,
        indexed: enabled,
        message: if enabled {
            "Dependabot alerts are derived from indexed dependency manifests and advisories."
                .to_owned()
        } else {
            setting
                .map(|row| row.get::<String, _>("summary"))
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "Dependabot alerts are disabled for this repository.".to_owned())
        },
        disabled_reason: (!enabled).then_some(status),
        settings_href: setting
            .and_then(|row| row.get::<Option<String>, _>("config_href"))
            .or_else(|| {
                Some(format!(
                    "/{}/{}/settings/security_analysis",
                    repository.owner_login, repository.name
                ))
            }),
    }
}

async fn materialize_dependabot_alerts(
    pool: &PgPool,
    repository: &Repository,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        WITH candidates AS (
            SELECT repository_dependencies.id AS repository_dependency_id,
                   dependency_advisories.id AS dependency_advisory_id,
                   repository_dependencies.relationship,
                   repository_dependencies.package_version,
                   dependency_advisories.advisory_identifier,
                   row_number() OVER (
                       ORDER BY dependency_advisories.severity DESC,
                                lower(dependency_advisories.advisory_identifier),
                                repository_dependencies.id
                   ) AS ordinal
            FROM repository_dependencies
            JOIN dependency_advisories ON dependency_advisories.package_id = repository_dependencies.package_id
            WHERE repository_dependencies.repository_id = $1
        ),
        numbered AS (
            SELECT candidates.*,
                   COALESCE((SELECT max(number) FROM dependabot_alerts WHERE repository_id = $1), 0)
                   + candidates.ordinal AS generated_number
            FROM candidates
        )
        INSERT INTO dependabot_alerts (
            repository_id,
            repository_dependency_id,
            dependency_advisory_id,
            number,
            scope,
            vulnerable_requirements,
            fixed_version
        )
        SELECT $1,
               repository_dependency_id,
               dependency_advisory_id,
               generated_number,
               CASE WHEN relationship = 'direct' THEN 'production' ELSE 'development' END,
               COALESCE(package_version, 'installed version'),
               NULL
        FROM numbered
        ON CONFLICT (repository_id, repository_dependency_id, dependency_advisory_id) DO NOTHING
        "#,
    )
    .bind(repository.id)
    .execute(pool)
    .await?;
    Ok(())
}

fn normalize_dependabot_alert_filters(
    query: DependabotAlertsQuery<'_>,
) -> Result<DependabotAlertFilters, RepositoryError> {
    let state = match query.state.map(str::trim).filter(|value| !value.is_empty()) {
        Some(state @ ("open" | "closed" | "dismissed" | "fixed" | "all")) => state.to_owned(),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported dependabot alert state `{other}`"
            )))
        }
        None => "open".to_owned(),
    };
    let query_text = normalize_optional_filter(query.query, "q", 120)?;
    let package = normalize_optional_filter(query.package, "package", 160)?;
    let manifest = normalize_optional_filter(query.manifest, "manifest", 240)?;
    let ecosystem = match query
        .ecosystem
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(ecosystem @ ("npm" | "cargo" | "pip")) => Some(ecosystem.to_owned()),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported ecosystem `{other}`"
            )))
        }
        None => None,
    };
    let scope = match query.scope.map(str::trim).filter(|value| !value.is_empty()) {
        Some(scope @ ("production" | "development")) => Some(scope.to_owned()),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported scope `{other}`"
            )))
        }
        None => None,
    };
    let severity = match query
        .severity
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(severity @ ("low" | "moderate" | "high" | "critical")) => Some(severity.to_owned()),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported severity `{other}`"
            )))
        }
        None => None,
    };
    let sort = match query.sort.map(str::trim).filter(|value| !value.is_empty()) {
        Some(sort @ ("most_important" | "recently_detected" | "package" | "manifest")) => {
            sort.to_owned()
        }
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported sort `{other}`"
            )))
        }
        None => "most_important".to_owned(),
    };
    Ok(DependabotAlertFilters {
        state,
        query: query_text,
        package,
        ecosystem,
        manifest,
        scope,
        severity,
        sort,
    })
}

fn normalize_optional_filter(
    value: Option<&str>,
    label: &str,
    max_chars: usize,
) -> Result<Option<String>, RepositoryError> {
    let value = value.map(str::trim).filter(|value| !value.is_empty());
    if let Some(value) = value {
        if value.chars().count() > max_chars {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "{label} must be {max_chars} characters or fewer"
            )));
        }
        return Ok(Some(value.to_owned()));
    }
    Ok(None)
}

fn normalize_code_scanning_filters(
    query: CodeScanningAlertsQuery<'_>,
) -> Result<CodeScanningFilters, RepositoryError> {
    let state = match query.state.map(str::trim).filter(|value| !value.is_empty()) {
        Some(state @ ("open" | "closed" | "dismissed" | "fixed" | "all")) => state.to_owned(),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported code scanning alert state `{other}`"
            )))
        }
        None => "open".to_owned(),
    };
    let severity = match query
        .severity
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(severity @ ("note" | "warning" | "error")) => Some(severity.to_owned()),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported code scanning severity `{other}`"
            )))
        }
        None => None,
    };
    let security_severity = match query
        .security_severity
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(severity @ ("low" | "medium" | "high" | "critical")) => Some(severity.to_owned()),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported code scanning security severity `{other}`"
            )))
        }
        None => None,
    };
    let application_code = match query
        .application_code
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value @ ("true" | "false")) => Some(value.to_owned()),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported application_code filter `{other}`"
            )))
        }
        None => None,
    };
    let sort = match query.sort.map(str::trim).filter(|value| !value.is_empty()) {
        Some(sort @ ("most_important" | "recently_detected" | "tool" | "path")) => sort.to_owned(),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported code scanning sort `{other}`"
            )))
        }
        None => "most_important".to_owned(),
    };
    Ok(CodeScanningFilters {
        state,
        query: normalize_optional_filter(query.query, "q", 120)?,
        severity,
        security_severity,
        tool: normalize_optional_filter(query.tool, "tool", 120)?,
        branch: normalize_optional_filter(query.branch, "branch", 160)?,
        ref_name: normalize_optional_filter(query.ref_name, "ref", 200)?,
        tag: normalize_optional_filter(query.tag, "tag", 160)?,
        application_code,
        sort,
    })
}

fn normalize_dependabot_dismissal_reason(value: Option<&str>) -> Result<String, RepositoryError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "dismissal reason is required".to_owned(),
        ));
    };
    match value {
        "fix_started" | "inaccurate" | "no_bandwidth" | "not_used" | "tolerable_risk" => {
            Ok(value.to_owned())
        }
        other => Err(RepositoryError::InvalidDependencyGraphQuery(format!(
            "unsupported dismissal reason `{other}`"
        ))),
    }
}

fn normalize_dependabot_dismissal_comment(
    value: Option<&str>,
) -> Result<Option<String>, RepositoryError> {
    let value = value.map(str::trim).filter(|value| !value.is_empty());
    if let Some(value) = value {
        if value.chars().count() > 500 {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "dismissal comment must be 500 characters or fewer".to_owned(),
            ));
        }
        return Ok(Some(value.to_owned()));
    }
    Ok(None)
}

fn normalize_code_scanning_dismissal_reason(
    value: Option<&str>,
) -> Result<String, RepositoryError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "dismissal reason is required".to_owned(),
        ));
    };
    match value {
        "false_positive" | "won_t_fix" | "used_in_tests" | "not_used" => Ok(value.to_owned()),
        other => Err(RepositoryError::InvalidDependencyGraphQuery(format!(
            "unsupported code scanning dismissal reason `{other}`"
        ))),
    }
}

async fn dependabot_alert_rows(
    pool: &PgPool,
    repository: &Repository,
) -> Result<Vec<DependabotAlertRow>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT dependabot_alerts.id,
               dependabot_alerts.number,
               dependabot_alerts.state,
               dependabot_alerts.scope,
               dependabot_alerts.vulnerable_requirements,
               dependabot_alerts.fixed_version,
               dependabot_alerts.created_at,
               dependabot_alerts.updated_at,
               repository_dependencies.package_version,
               repository_dependencies.relationship,
               repository_dependencies.lockfile_path,
               dependency_manifests.path AS manifest_path,
               dependency_packages.id AS package_id,
               dependency_packages.ecosystem,
               dependency_packages.name,
               dependency_packages.package_href,
               dependency_advisories.id AS advisory_id,
               dependency_advisories.advisory_identifier,
               dependency_advisories.severity,
               dependency_advisories.title,
               dependency_advisories.advisory_href,
               dependency_advisories.published_at
        FROM dependabot_alerts
        JOIN repository_dependencies ON repository_dependencies.id = dependabot_alerts.repository_dependency_id
        JOIN dependency_manifests ON dependency_manifests.id = repository_dependencies.manifest_id
        JOIN dependency_packages ON dependency_packages.id = repository_dependencies.package_id
        JOIN dependency_advisories ON dependency_advisories.id = dependabot_alerts.dependency_advisory_id
        WHERE dependabot_alerts.repository_id = $1
        ORDER BY dependabot_alerts.number ASC
        LIMIT 250
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;

    let mut alerts = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let ecosystem: String = row.get("ecosystem");
        let package_name: String = row.get("name");
        let manifest_path: String = row.get("manifest_path");
        let lockfile_path: Option<String> = row.get("lockfile_path");
        let package_href = row
            .get::<Option<String>, _>("package_href")
            .unwrap_or_else(|| package_href(&ecosystem, &package_name));
        alerts.push(DependabotAlertRow {
            id,
            number: row.get("number"),
            state: row.get("state"),
            scope: row.get("scope"),
            package: DependabotAlertPackage {
                id: row.get("package_id"),
                ecosystem: ecosystem.clone(),
                name: package_name.clone(),
                href: package_href,
            },
            advisory: DependabotAlertAdvisorySummary {
                id: row.get("advisory_id"),
                identifier: row.get("advisory_identifier"),
                severity: row.get("severity"),
                title: row.get("title"),
                href: row.get("advisory_href"),
                published_at: row.get("published_at"),
            },
            manifest_path: manifest_path.clone(),
            manifest_href: repository_blob_href(
                repository,
                &repository.default_branch,
                &manifest_path,
            ),
            lockfile_path: lockfile_path.clone(),
            lockfile_href: lockfile_path
                .as_deref()
                .map(|path| repository_blob_href(repository, &repository.default_branch, path)),
            vulnerable_requirements: row.get("vulnerable_requirements"),
            current_version: row.get("package_version"),
            fixed_version: row.get("fixed_version"),
            relationship: row.get("relationship"),
            assignees: dependabot_alert_assignees(pool, id).await?,
            href: format!(
                "/{}/{}/security/dependabot/{}",
                repository.owner_login,
                repository.name,
                row.get::<i64, _>("number")
            ),
            detected_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });
    }
    Ok(alerts)
}

async fn code_scanning_alert_rows(
    pool: &PgPool,
    repository: &Repository,
) -> Result<Vec<CodeScanningAlertRow>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT code_scanning_alerts.id,
               code_scanning_alerts.number,
               code_scanning_alerts.state,
               code_scanning_alerts.rule_id,
               code_scanning_alerts.rule_name,
               code_scanning_alerts.message,
               code_scanning_alerts.severity,
               code_scanning_alerts.security_severity,
               code_scanning_alerts.tool_name,
               code_scanning_alerts.path,
               code_scanning_alerts.start_line,
               code_scanning_alerts.end_line,
               code_scanning_alerts.ref_name,
               code_scanning_alerts.branch_name,
               code_scanning_alerts.linked_issue_id,
               code_scanning_alerts.created_at,
               code_scanning_alerts.updated_at,
               issues.number AS issue_number,
               issues.title AS issue_title
        FROM code_scanning_alerts
        LEFT JOIN issues ON issues.id = code_scanning_alerts.linked_issue_id
        WHERE code_scanning_alerts.repository_id = $1
        ORDER BY code_scanning_alerts.number ASC
        LIMIT 250
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;

    let mut alerts = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let path: String = row.get("path");
        let ref_name: String = row.get("ref_name");
        let number: i64 = row.get("number");
        let linked_issue = match (
            row.try_get::<Option<Uuid>, _>("linked_issue_id")
                .ok()
                .flatten(),
            row.try_get::<Option<i64>, _>("issue_number").ok().flatten(),
            row.try_get::<Option<String>, _>("issue_title")
                .ok()
                .flatten(),
        ) {
            (Some(id), Some(number), Some(title)) => Some(CodeScanningIssueLink {
                id,
                number,
                title,
                href: format!(
                    "/{}/{}/issues/{}",
                    repository.owner_login, repository.name, number
                ),
            }),
            _ => None,
        };
        alerts.push(CodeScanningAlertRow {
            id,
            number,
            state: row.get("state"),
            rule_id: row.get("rule_id"),
            rule_name: row.get("rule_name"),
            message: row.get("message"),
            severity: row.get("severity"),
            security_severity: row.get("security_severity"),
            tool_name: row.get("tool_name"),
            path: path.clone(),
            path_href: format!(
                "{}#L{}",
                repository_blob_href(repository, &ref_name, &path),
                row.get::<i32, _>("start_line")
            ),
            start_line: row.get("start_line"),
            end_line: row.get("end_line"),
            is_default_branch: branch_name_matches_default(
                row.get::<Option<String>, _>("branch_name").as_deref(),
                &ref_name,
                &repository.default_branch,
            ),
            ref_name,
            branch_name: row.get("branch_name"),
            linked_issue,
            assignees: code_scanning_alert_assignees(pool, id).await?,
            href: format!(
                "/{}/{}/security/code-scanning/{}",
                repository.owner_login, repository.name, number
            ),
            detected_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });
    }
    Ok(alerts)
}

async fn dependabot_alert_assignees(
    pool: &PgPool,
    alert_id: Uuid,
) -> Result<Vec<DependabotAlertAssignee>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.avatar_url
        FROM dependabot_alert_assignees
        JOIN users ON users.id = dependabot_alert_assignees.user_id
        WHERE dependabot_alert_assignees.alert_id = $1
        ORDER BY lower(COALESCE(NULLIF(users.username, ''), users.email)) ASC
        "#,
    )
    .bind(alert_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let login: String = row.get("login");
            DependabotAlertAssignee {
                id: row.get("id"),
                href: format!("/{login}"),
                login,
                avatar_url: row.get("avatar_url"),
            }
        })
        .collect())
}

async fn code_scanning_alert_assignees(
    pool: &PgPool,
    alert_id: Uuid,
) -> Result<Vec<DependabotAlertAssignee>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.avatar_url
        FROM code_scanning_alert_assignees
        JOIN users ON users.id = code_scanning_alert_assignees.user_id
        WHERE code_scanning_alert_assignees.alert_id = $1
        ORDER BY lower(COALESCE(NULLIF(users.username, ''), users.email)) ASC
        "#,
    )
    .bind(alert_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let login: String = row.get("login");
            DependabotAlertAssignee {
                id: row.get("id"),
                href: format!("/{login}"),
                login,
                avatar_url: row.get("avatar_url"),
            }
        })
        .collect())
}

fn apply_dependabot_alert_filters(
    alerts: &mut Vec<DependabotAlertRow>,
    filters: &DependabotAlertFilters,
) {
    alerts.retain(|alert| match filters.state.as_str() {
        "open" => alert.state == "open",
        "closed" => alert.state == "dismissed" || alert.state == "fixed",
        "dismissed" => alert.state == "dismissed",
        "fixed" => alert.state == "fixed",
        "all" => true,
        _ => true,
    });
    if let Some(query) = filters.query.as_deref() {
        let needle = query.to_lowercase();
        alerts.retain(|alert| {
            alert.package.name.to_lowercase().contains(&needle)
                || alert.advisory.title.to_lowercase().contains(&needle)
                || alert.advisory.identifier.to_lowercase().contains(&needle)
                || alert.manifest_path.to_lowercase().contains(&needle)
        });
    }
    if let Some(package) = filters.package.as_deref() {
        alerts.retain(|alert| {
            alert.package.name.eq_ignore_ascii_case(package)
                || format!("{}:{}", alert.package.ecosystem, alert.package.name)
                    .eq_ignore_ascii_case(package)
        });
    }
    if let Some(ecosystem) = filters.ecosystem.as_deref() {
        alerts.retain(|alert| alert.package.ecosystem == ecosystem);
    }
    if let Some(manifest) = filters.manifest.as_deref() {
        alerts.retain(|alert| alert.manifest_path.eq_ignore_ascii_case(manifest));
    }
    if let Some(scope) = filters.scope.as_deref() {
        alerts.retain(|alert| alert.scope == scope);
    }
    if let Some(severity) = filters.severity.as_deref() {
        alerts.retain(|alert| alert.advisory.severity == severity);
    }
}

fn apply_code_scanning_filters(
    alerts: &mut Vec<CodeScanningAlertRow>,
    filters: &CodeScanningFilters,
) {
    alerts.retain(|alert| match filters.state.as_str() {
        "open" => alert.state == "open",
        "closed" => alert.state == "dismissed" || alert.state == "fixed",
        "dismissed" => alert.state == "dismissed",
        "fixed" => alert.state == "fixed",
        "all" => true,
        _ => true,
    });
    if let Some(query) = filters.query.as_deref() {
        let needle = query.to_lowercase();
        alerts.retain(|alert| {
            alert.rule_name.to_lowercase().contains(&needle)
                || alert.rule_id.to_lowercase().contains(&needle)
                || alert.message.to_lowercase().contains(&needle)
                || alert.path.to_lowercase().contains(&needle)
                || alert.tool_name.to_lowercase().contains(&needle)
        });
    }
    if let Some(severity) = filters.severity.as_deref() {
        alerts.retain(|alert| alert.severity == severity);
    }
    if let Some(security_severity) = filters.security_severity.as_deref() {
        alerts.retain(|alert| {
            alert
                .security_severity
                .as_deref()
                .is_some_and(|value| value == security_severity)
        });
    }
    if let Some(tool) = filters.tool.as_deref() {
        alerts.retain(|alert| alert.tool_name.eq_ignore_ascii_case(tool));
    }
    if let Some(branch) = filters.branch.as_deref() {
        alerts.retain(|alert| {
            alert
                .branch_name
                .as_deref()
                .is_some_and(|value| value.eq_ignore_ascii_case(branch))
        });
    }
    if let Some(ref_name) = filters.ref_name.as_deref() {
        alerts.retain(|alert| alert.ref_name.eq_ignore_ascii_case(ref_name));
    }
    if let Some(tag) = filters.tag.as_deref() {
        alerts.retain(|alert| {
            alert
                .ref_name
                .eq_ignore_ascii_case(&format!("refs/tags/{tag}"))
        });
    }
    if filters.application_code.as_deref() == Some("true") {
        alerts.retain(|alert| {
            !alert.path.contains("vendor/") && !alert.path.contains("node_modules/")
        });
    }
}

fn sort_dependabot_alerts(alerts: &mut [DependabotAlertRow], sort: &str) {
    alerts.sort_by(|left, right| match sort {
        "recently_detected" => right
            .detected_at
            .cmp(&left.detected_at)
            .then(left.number.cmp(&right.number)),
        "package" => left
            .package
            .name
            .to_lowercase()
            .cmp(&right.package.name.to_lowercase()),
        "manifest" => left
            .manifest_path
            .to_lowercase()
            .cmp(&right.manifest_path.to_lowercase()),
        _ => severity_rank(&left.advisory.severity)
            .cmp(&severity_rank(&right.advisory.severity))
            .then(left.number.cmp(&right.number)),
    });
}

fn sort_code_scanning_alerts(alerts: &mut [CodeScanningAlertRow], sort: &str) {
    alerts.sort_by(|left, right| match sort {
        "recently_detected" => right
            .detected_at
            .cmp(&left.detected_at)
            .then(left.number.cmp(&right.number)),
        "tool" => left
            .tool_name
            .to_lowercase()
            .cmp(&right.tool_name.to_lowercase())
            .then(left.number.cmp(&right.number)),
        "path" => left
            .path
            .to_lowercase()
            .cmp(&right.path.to_lowercase())
            .then(left.start_line.cmp(&right.start_line)),
        _ => code_scanning_severity_rank(left.security_severity.as_deref(), &left.severity)
            .cmp(&code_scanning_severity_rank(
                right.security_severity.as_deref(),
                &right.severity,
            ))
            .then(left.number.cmp(&right.number)),
    });
}

fn code_scanning_severity_rank(security_severity: Option<&str>, severity: &str) -> i32 {
    match security_severity.or(Some(severity)).unwrap_or_default() {
        "critical" | "error" => 0,
        "high" => 1,
        "medium" | "warning" => 2,
        "low" | "note" => 3,
        _ => 4,
    }
}

fn severity_rank(severity: &str) -> i32 {
    match severity {
        "critical" => 0,
        "high" => 1,
        "moderate" => 2,
        "low" => 3,
        _ => 4,
    }
}

fn dependabot_counts(alerts: &[DependabotAlertRow], visible: i64) -> DependabotAlertCounts {
    let open = alerts.iter().filter(|alert| alert.state == "open").count() as i64;
    let closed = alerts
        .iter()
        .filter(|alert| alert.state == "dismissed" || alert.state == "fixed")
        .count() as i64;
    DependabotAlertCounts {
        open,
        closed,
        total: alerts.len() as i64,
        visible,
    }
}

fn code_scanning_counts(alerts: &[CodeScanningAlertRow], visible: i64) -> CodeScanningAlertCounts {
    let open = alerts.iter().filter(|alert| alert.state == "open").count() as i64;
    let closed = alerts
        .iter()
        .filter(|alert| alert.state == "dismissed" || alert.state == "fixed")
        .count() as i64;
    CodeScanningAlertCounts {
        open,
        closed,
        total: alerts.len() as i64,
        visible,
    }
}

async fn dependabot_package_filters(
    _repository: &Repository,
    alerts: &[DependabotAlertRow],
    selected: Option<&str>,
) -> Result<Vec<DependabotAlertPackageFilter>, RepositoryError> {
    let mut packages = Vec::<DependabotAlertPackageFilter>::new();
    for alert in alerts.iter().filter(|alert| alert.state == "open") {
        if let Some(existing) = packages
            .iter_mut()
            .find(|entry| entry.package.id == alert.package.id)
        {
            existing.open_count += 1;
        } else {
            packages.push(DependabotAlertPackageFilter {
                package: alert.package.clone(),
                open_count: 1,
                selected: selected
                    .map(|value| {
                        value.eq_ignore_ascii_case(&alert.package.name)
                            || value.eq_ignore_ascii_case(&format!(
                                "{}:{}",
                                alert.package.ecosystem, alert.package.name
                            ))
                    })
                    .unwrap_or(false),
            });
        }
    }
    packages.sort_by(|left, right| {
        right
            .open_count
            .cmp(&left.open_count)
            .then(left.package.name.cmp(&right.package.name))
    });
    Ok(packages)
}

async fn dependabot_manifest_filters(
    repository: &Repository,
    alerts: &[DependabotAlertRow],
    selected: Option<&str>,
) -> Result<Vec<DependabotAlertManifestFilter>, RepositoryError> {
    let mut manifests = Vec::<DependabotAlertManifestFilter>::new();
    for alert in alerts.iter().filter(|alert| alert.state == "open") {
        if let Some(existing) = manifests
            .iter_mut()
            .find(|entry| entry.path == alert.manifest_path)
        {
            existing.open_count += 1;
        } else {
            manifests.push(DependabotAlertManifestFilter {
                path: alert.manifest_path.clone(),
                ecosystem: alert.package.ecosystem.clone(),
                href: repository_blob_href(
                    repository,
                    &repository.default_branch,
                    &alert.manifest_path,
                ),
                open_count: 1,
                selected: selected
                    .map(|value| value.eq_ignore_ascii_case(&alert.manifest_path))
                    .unwrap_or(false),
            });
        }
    }
    manifests.sort_by(|left, right| left.path.to_lowercase().cmp(&right.path.to_lowercase()));
    Ok(manifests)
}

async fn code_scanning_branch_filters(
    alerts: &[CodeScanningAlertRow],
    selected: Option<&str>,
) -> Result<Vec<CodeScanningBranchFilter>, RepositoryError> {
    let mut branches = Vec::<CodeScanningBranchFilter>::new();
    for alert in alerts.iter().filter(|alert| alert.state == "open") {
        let Some(branch) = alert.branch_name.as_deref() else {
            continue;
        };
        if let Some(existing) = branches.iter_mut().find(|entry| entry.name == branch) {
            existing.open_count += 1;
        } else {
            branches.push(CodeScanningBranchFilter {
                name: branch.to_owned(),
                open_count: 1,
                selected: selected
                    .map(|value| value.eq_ignore_ascii_case(branch))
                    .unwrap_or(false),
            });
        }
    }
    branches.sort_by(|left, right| {
        right
            .open_count
            .cmp(&left.open_count)
            .then(left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    Ok(branches)
}

async fn code_scanning_tool_statuses(
    pool: &PgPool,
    repository: &Repository,
) -> Result<Vec<CodeScanningToolStatus>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT runs.tool_name,
               max(runs.tool_version) AS tool_version,
               COALESCE(max(runs.status), 'completed') AS status,
               count(DISTINCT alerts.id) FILTER (WHERE alerts.state = 'open') AS alert_count,
               max(runs.completed_at) AS latest_run_at
        FROM code_scanning_runs runs
        LEFT JOIN code_scanning_alerts alerts ON alerts.run_id = runs.id
        WHERE runs.repository_id = $1
        GROUP BY runs.tool_name
        ORDER BY lower(runs.tool_name) ASC
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| CodeScanningToolStatus {
            name: row.get("tool_name"),
            version: row.get("tool_version"),
            status: row.get("status"),
            alert_count: row.get("alert_count"),
            latest_run_at: row.get("latest_run_at"),
        })
        .collect())
}

async fn dependabot_alert_timeline(
    pool: &PgPool,
    alert_id: Uuid,
) -> Result<Vec<DependabotAlertTimelineEvent>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT security_alert_events.id,
               security_alert_events.event_type,
               security_alert_events.message,
               security_alert_events.created_at,
               users.id AS actor_id,
               COALESCE(NULLIF(users.username, ''), users.email) AS actor_login,
               users.avatar_url AS actor_avatar_url
        FROM security_alert_events
        LEFT JOIN users ON users.id = security_alert_events.actor_user_id
        WHERE security_alert_events.alert_id = $1
        ORDER BY security_alert_events.created_at ASC
        "#,
    )
    .bind(alert_id)
    .fetch_all(pool)
    .await?;

    let mut events = Vec::new();
    for row in rows {
        let actor = match (
            row.try_get::<Option<Uuid>, _>("actor_id").ok().flatten(),
            row.try_get::<Option<String>, _>("actor_login")
                .ok()
                .flatten(),
        ) {
            (Some(id), Some(login)) => Some(DependabotAlertAssignee {
                id,
                href: format!("/{login}"),
                login,
                avatar_url: row.get("actor_avatar_url"),
            }),
            _ => None,
        };
        events.push(DependabotAlertTimelineEvent {
            id: row.get("id"),
            event_type: row.get("event_type"),
            message: row.get("message"),
            actor,
            created_at: row.get("created_at"),
        });
    }
    if events.is_empty() {
        events.push(DependabotAlertTimelineEvent {
            id: alert_id,
            event_type: "created".to_owned(),
            message: "Dependabot opened this alert from the dependency graph.".to_owned(),
            actor: None,
            created_at: Utc::now(),
        });
    }
    Ok(events)
}

async fn code_scanning_alert_timeline(
    pool: &PgPool,
    alert_id: Uuid,
) -> Result<Vec<CodeScanningTimelineEvent>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT code_scanning_alert_events.id,
               code_scanning_alert_events.event_type,
               code_scanning_alert_events.message,
               code_scanning_alert_events.created_at,
               users.id AS actor_id,
               COALESCE(NULLIF(users.username, ''), users.email) AS actor_login,
               users.avatar_url AS actor_avatar_url
        FROM code_scanning_alert_events
        LEFT JOIN users ON users.id = code_scanning_alert_events.actor_user_id
        WHERE code_scanning_alert_events.alert_id = $1
        ORDER BY code_scanning_alert_events.created_at ASC
        "#,
    )
    .bind(alert_id)
    .fetch_all(pool)
    .await?;

    let mut events = Vec::new();
    for row in rows {
        let actor = match (
            row.try_get::<Option<Uuid>, _>("actor_id").ok().flatten(),
            row.try_get::<Option<String>, _>("actor_login")
                .ok()
                .flatten(),
        ) {
            (Some(id), Some(login)) => Some(DependabotAlertAssignee {
                id,
                href: format!("/{login}"),
                login,
                avatar_url: row.get("actor_avatar_url"),
            }),
            _ => None,
        };
        events.push(CodeScanningTimelineEvent {
            id: row.get("id"),
            event_type: row.get("event_type"),
            message: row.get("message"),
            actor,
            created_at: row.get("created_at"),
        });
    }
    if events.is_empty() {
        events.push(CodeScanningTimelineEvent {
            id: alert_id,
            event_type: "created".to_owned(),
            message: "Code scanning opened this alert from analysis results.".to_owned(),
            actor: None,
            created_at: Utc::now(),
        });
    }
    Ok(events)
}

async fn record_dependabot_alert_event(
    pool: &PgPool,
    repository: &Repository,
    alert_id: Uuid,
    actor_user_id: Uuid,
    event_type: &str,
    message: &str,
    metadata: serde_json::Value,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO security_alert_events (
            repository_id, alert_id, actor_user_id, event_type, message, metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(repository.id)
    .bind(alert_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(message)
    .bind(metadata.clone())
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO security_audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'repository.dependabot_alert.update', 'repository', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(repository.id)
    .bind(json!({
        "repositoryId": repository.id,
        "alertId": alert_id,
        "alertEvent": event_type,
        "metadata": metadata,
    }))
    .execute(pool)
    .await?;

    Ok(())
}

async fn record_code_scanning_alert_event(
    pool: &PgPool,
    repository: &Repository,
    alert_id: Uuid,
    actor_user_id: Uuid,
    event_type: &str,
    message: &str,
    metadata: serde_json::Value,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO code_scanning_alert_events (
            repository_id, alert_id, actor_user_id, event_type, message, metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(repository.id)
    .bind(alert_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(message)
    .bind(metadata.clone())
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO security_audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'repository.code_scanning_alert.update', 'repository', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(repository.id)
    .bind(json!({
        "repositoryId": repository.id,
        "alertId": alert_id,
        "alertEvent": event_type,
        "metadata": metadata,
    }))
    .execute(pool)
    .await?;

    Ok(())
}

async fn notify_dependabot_alert_assignees(
    pool: &PgPool,
    repository: &Repository,
    alert_id: Uuid,
    title: &str,
    reason: &str,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO notifications (
            user_id, repository_id, subject_type, subject_id, title, reason
        )
        SELECT dependabot_alert_assignees.user_id,
               $2,
               'dependabot_alert',
               $1,
               $3,
               $4
        FROM dependabot_alert_assignees
        WHERE dependabot_alert_assignees.alert_id = $1
        "#,
    )
    .bind(alert_id)
    .bind(repository.id)
    .bind(title)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

async fn notify_code_scanning_alert_assignees(
    pool: &PgPool,
    repository: &Repository,
    alert_id: Uuid,
    title: &str,
    reason: &str,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO notifications (
            user_id, repository_id, subject_type, subject_id, title, reason
        )
        SELECT code_scanning_alert_assignees.user_id,
               $2,
               'code_scanning_alert',
               $1,
               $3,
               $4
        FROM code_scanning_alert_assignees
        WHERE code_scanning_alert_assignees.alert_id = $1
        "#,
    )
    .bind(alert_id)
    .bind(repository.id)
    .bind(title)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

async fn code_scanning_link_existing_issue(
    pool: &PgPool,
    repository: &Repository,
    alert_id: Uuid,
    issue_id: Uuid,
    actor_user_id: Uuid,
) -> Result<i64, RepositoryError> {
    let issue = sqlx::query("SELECT number FROM issues WHERE repository_id = $1 AND id = $2")
        .bind(repository.id)
        .bind(issue_id)
        .fetch_optional(pool)
        .await?;
    let Some(issue) = issue else {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "linked issue must belong to this repository".to_owned(),
        ));
    };
    let issue_number: i64 = issue.get("number");
    sqlx::query(
        "UPDATE code_scanning_alerts SET linked_issue_id = $3, updated_at = now() WHERE repository_id = $1 AND id = $2",
    )
    .bind(repository.id)
    .bind(alert_id)
    .bind(issue_id)
    .execute(pool)
    .await?;
    notify_code_scanning_alert_assignees(
        pool,
        repository,
        alert_id,
        "Code scanning alert linked to an issue",
        "mention",
    )
    .await?;
    sqlx::query(
        "INSERT INTO notifications (user_id, repository_id, subject_type, subject_id, title, reason) VALUES ($1, $2, 'issue', $3, $4, 'mention')",
    )
    .bind(actor_user_id)
    .bind(repository.id)
    .bind(issue_id)
    .bind(format!("Issue #{issue_number} linked to a code scanning alert"))
    .execute(pool)
    .await?;
    Ok(issue_number)
}

async fn dependabot_assignment_options(
    pool: &PgPool,
    repository: &Repository,
    alert_id: Uuid,
) -> Result<Vec<DependabotAlertAssignmentOption>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.avatar_url,
               EXISTS (
                   SELECT 1 FROM dependabot_alert_assignees
                   WHERE alert_id = $2 AND user_id = users.id
               ) AS selected
        FROM users
        WHERE users.id = $3
           OR EXISTS (
               SELECT 1 FROM repository_permissions
               WHERE repository_permissions.repository_id = $1
                 AND repository_permissions.user_id = users.id
           )
        ORDER BY lower(COALESCE(NULLIF(users.username, ''), users.email)) ASC
        LIMIT 25
        "#,
    )
    .bind(repository.id)
    .bind(alert_id)
    .bind(repository.created_by_user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| DependabotAlertAssignmentOption {
            id: row.get("id"),
            kind: "user".to_owned(),
            login: row.get("login"),
            avatar_url: row.get("avatar_url"),
            selected: row.get("selected"),
        })
        .collect())
}

async fn code_scanning_assignment_options(
    pool: &PgPool,
    repository: &Repository,
    alert_id: Uuid,
) -> Result<Vec<DependabotAlertAssignmentOption>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT users.id,
               'user' AS kind,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.avatar_url,
               EXISTS (
                   SELECT 1 FROM code_scanning_alert_assignees
                   WHERE code_scanning_alert_assignees.alert_id = $2
                     AND code_scanning_alert_assignees.user_id = users.id
               ) AS selected
        FROM repository_permissions
        JOIN users ON users.id = repository_permissions.user_id
        WHERE repository_permissions.repository_id = $1
          AND repository_permissions.role IN ('read', 'triage', 'write', 'maintain', 'admin')
        UNION
        SELECT users.id,
               'user' AS kind,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.avatar_url,
               EXISTS (
                   SELECT 1 FROM code_scanning_alert_assignees
                   WHERE code_scanning_alert_assignees.alert_id = $2
                     AND code_scanning_alert_assignees.user_id = users.id
               ) AS selected
        FROM repositories
        JOIN users ON users.id = repositories.owner_user_id
        WHERE repositories.id = $1
        ORDER BY login ASC
        LIMIT 50
        "#,
    )
    .bind(repository.id)
    .bind(alert_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| DependabotAlertAssignmentOption {
            id: row.get("id"),
            kind: row.get("kind"),
            login: row.get("login"),
            avatar_url: row.get("avatar_url"),
            selected: row.get("selected"),
        })
        .collect())
}

async fn dependabot_security_update_state(
    pool: &PgPool,
    repository: &Repository,
    alert: &DependabotAlertRow,
) -> Result<DependabotSecurityUpdateState, RepositoryError> {
    let existing_href = dependabot_existing_security_update_pr(pool, repository, alert)
        .await?
        .map(|(_, href)| href);
    let supported = alert.state == "open"
        && matches!(alert.package.ecosystem.as_str(), "npm" | "cargo" | "pip");
    Ok(DependabotSecurityUpdateState {
        supported,
        status: if existing_href.is_some() {
            "linked"
        } else if supported {
            "available"
        } else {
            "unsupported"
        }
        .to_owned(),
        href: (supported && existing_href.is_none()).then(|| {
            format!(
                "/api/repos/{}/{}/security/dependabot/{}/security-update",
                percent_encode_segment(&repository.owner_login),
                percent_encode_segment(&repository.name),
                alert.number
            )
        }),
        pull_request_href: existing_href.clone(),
        message: if existing_href.is_some() {
            "A security update pull request is already linked to this alert.".to_owned()
        } else if supported {
            "A security update pull request can be prepared for this manifest.".to_owned()
        } else {
            "Security update pull requests are unavailable for this alert state or ecosystem."
                .to_owned()
        },
    })
}

fn normalize_dependabot_bulk_alert_ids(alert_ids: &[Uuid]) -> Result<Vec<Uuid>, RepositoryError> {
    if alert_ids.is_empty() {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "at least one Dependabot alert must be selected".to_owned(),
        ));
    }
    if alert_ids.len() > 100 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "bulk triage is limited to 100 Dependabot alerts".to_owned(),
        ));
    }
    let mut normalized = Vec::new();
    for alert_id in alert_ids {
        if !normalized.contains(alert_id) {
            normalized.push(*alert_id);
        }
    }
    Ok(normalized)
}

fn dependabot_security_update_branch(alert: &DependabotAlertRow) -> String {
    let package = Regex::new(r"[^A-Za-z0-9._-]+")
        .expect("branch package regex")
        .replace_all(&alert.package.name, "-")
        .trim_matches('-')
        .to_ascii_lowercase();
    format!(
        "dependabot/{}/{}-{}",
        alert.package.ecosystem,
        if package.is_empty() {
            "package"
        } else {
            &package
        },
        alert.number
    )
}

fn dependabot_security_update_version(
    alert: &DependabotAlertRow,
) -> Result<String, RepositoryError> {
    if let Some(version) = alert
        .fixed_version
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(version.to_owned());
    }
    let Some(current) = alert
        .current_version
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "security update requires a current dependency version".to_owned(),
        ));
    };
    Ok(format!("{current}-security"))
}

fn update_dependency_manifest_content(
    ecosystem: &str,
    content: &str,
    package: &str,
    version: &str,
) -> Result<String, RepositoryError> {
    match ecosystem {
        "npm" => update_json_dependency_manifest(content, package, version),
        "cargo" => update_line_dependency_manifest(content, package, version, " = "),
        "pip" => update_requirements_manifest(content, package, version),
        _ => Err(RepositoryError::InvalidDependencyGraphQuery(
            "security update pull requests are unsupported for this ecosystem".to_owned(),
        )),
    }
}

fn update_json_dependency_manifest(
    content: &str,
    package: &str,
    version: &str,
) -> Result<String, RepositoryError> {
    let mut document: serde_json::Value = serde_json::from_str(content).map_err(|_| {
        RepositoryError::InvalidDependencyGraphQuery(
            "package.json must be valid JSON before a security update can be prepared".to_owned(),
        )
    })?;
    for section in [
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "peerDependencies",
    ] {
        if let Some(dependencies) = document
            .get_mut(section)
            .and_then(|value| value.as_object_mut())
        {
            if dependencies.contains_key(package) {
                dependencies.insert(
                    package.to_owned(),
                    serde_json::Value::String(version.to_owned()),
                );
                return serde_json::to_string_pretty(&document)
                    .map(|json| format!("{json}\n"))
                    .map_err(|_| {
                        RepositoryError::InvalidDependencyGraphQuery(
                            "package.json could not be serialized after the security update"
                                .to_owned(),
                        )
                    });
            }
        }
    }
    Err(RepositoryError::InvalidDependencyGraphQuery(format!(
        "package `{package}` was not found in package.json"
    )))
}

fn update_line_dependency_manifest(
    content: &str,
    package: &str,
    version: &str,
    separator: &str,
) -> Result<String, RepositoryError> {
    let mut changed = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if !changed
            && (trimmed.starts_with(&format!("{package}{separator}"))
                || trimmed.starts_with(&format!("{package}=")))
        {
            let indent_len = line.len() - trimmed.len();
            let indent = &line[..indent_len];
            lines.push(format!("{indent}{package}{separator}\"{version}\""));
            changed = true;
        } else {
            lines.push(line.to_owned());
        }
    }
    if !changed {
        return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
            "package `{package}` was not found in the manifest"
        )));
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn update_requirements_manifest(
    content: &str,
    package: &str,
    version: &str,
) -> Result<String, RepositoryError> {
    let mut changed = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if !changed
            && trimmed
                .to_ascii_lowercase()
                .starts_with(&package.to_ascii_lowercase())
        {
            let indent_len = line.len() - trimmed.len();
            let indent = &line[..indent_len];
            lines.push(format!("{indent}{package}=={version}"));
            changed = true;
        } else {
            lines.push(line.to_owned());
        }
    }
    if !changed {
        return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
            "package `{package}` was not found in requirements.txt"
        )));
    }
    Ok(format!("{}\n", lines.join("\n")))
}

async fn dependabot_existing_security_update_pr(
    pool: &PgPool,
    repository: &Repository,
    alert: &DependabotAlertRow,
) -> Result<Option<(String, String)>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT pull_requests.head_ref,
               pull_requests.number
        FROM dependabot_alerts
        JOIN pull_requests ON pull_requests.id = dependabot_alerts.security_update_pull_request_id
        WHERE dependabot_alerts.id = $1
          AND pull_requests.state = 'open'
        LIMIT 1
        "#,
    )
    .bind(alert.id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| {
        let number: i64 = row.get("number");
        (
            row.get("head_ref"),
            format!(
                "/{}/{}/pull/{}",
                repository.owner_login, repository.name, number
            ),
        )
    }))
}

fn collaboration_to_repository_error(error: super::issues::CollaborationError) -> RepositoryError {
    match error {
        super::issues::CollaborationError::RepositoryNotFound
        | super::issues::CollaborationError::IssueNotFound
        | super::issues::CollaborationError::PullRequestNotFound => RepositoryError::NotFound,
        super::issues::CollaborationError::RepositoryAccessDenied => {
            RepositoryError::PermissionDenied
        }
        super::issues::CollaborationError::InvalidState(message)
        | super::issues::CollaborationError::InvalidReaction(message)
        | super::issues::CollaborationError::InvalidIssueFilter(message)
        | super::issues::CollaborationError::InvalidIssueAttachment(message) => {
            RepositoryError::InvalidDependencyGraphQuery(message)
        }
        super::issues::CollaborationError::InvalidIssueField { message, .. } => {
            RepositoryError::InvalidDependencyGraphQuery(message)
        }
        super::issues::CollaborationError::Sqlx(error) => RepositoryError::Sqlx(error),
    }
}

fn dependabot_links(repository: &Repository) -> DependabotAlertLinks {
    let base = format!(
        "/{}/{}/security/dependabot",
        repository.owner_login, repository.name
    );
    DependabotAlertLinks {
        list_href: base.clone(),
        open_href: format!("{base}?state=open"),
        closed_href: format!("{base}?state=closed"),
        settings_href: format!(
            "/{}/{}/settings/security_analysis",
            repository.owner_login, repository.name
        ),
    }
}

fn code_scanning_links(repository: &Repository) -> CodeScanningLinks {
    let base = format!(
        "/{}/{}/security/code-scanning",
        repository.owner_login, repository.name
    );
    CodeScanningLinks {
        list_href: base.clone(),
        open_href: format!("{base}?state=open"),
        closed_href: format!("{base}?state=closed"),
        upload_href: format!(
            "/api/repos/{}/{}/code-scanning/sarifs",
            repository.owner_login, repository.name
        ),
        settings_href: format!(
            "/{}/{}/security/code-scanning/setup",
            repository.owner_login, repository.name
        ),
    }
}

fn branch_name_matches_default(
    branch_name: Option<&str>,
    ref_name: &str,
    default_branch: &str,
) -> bool {
    branch_name
        .map(|branch| branch == default_branch)
        .unwrap_or(false)
        || ref_name == default_branch
        || ref_name == format!("refs/heads/{default_branch}")
}

fn package_href(ecosystem: &str, name: &str) -> String {
    match ecosystem {
        "npm" => format!(
            "https://www.npmjs.com/package/{}",
            percent_encode_segment(name)
        ),
        "cargo" => format!("https://crates.io/crates/{}", percent_encode_segment(name)),
        "pip" => format!("https://pypi.org/project/{}", percent_encode_segment(name)),
        _ => format!(
            "/packages/{}/{}",
            percent_encode_segment(ecosystem),
            percent_encode_segment(name)
        ),
    }
}

fn markdown_sha(markdown: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(markdown.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn markdown_error(error: super::markdown::MarkdownError) -> RepositoryError {
    RepositoryError::Sqlx(sqlx::Error::Protocol(error.to_string()))
}

async fn write_security_policy(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    mutation: SecurityPolicyMutation,
) -> Result<(), RepositoryError> {
    let path = normalize_policy_path(mutation.path.as_deref())?;
    let ref_name = normalize_policy_ref(repository, mutation.ref_name.as_deref())?;
    let markdown = normalize_policy_markdown(&mutation.markdown)?;
    let commit_message = normalize_policy_commit_message(&mutation.commit_message)?;
    let rendered = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            markdown: markdown.clone(),
            repository_id: Some(repository.id),
            owner: Some(repository.owner_login.clone()),
            repo: Some(repository.name.clone()),
            ref_name: Some(ref_name.clone()),
            enable_task_toggles: Some(false),
        },
    )
    .await
    .map_err(markdown_error)?;
    let content_sha = markdown_sha(&markdown);

    let current_ref = current_branch_commit(pool, repository.id, &ref_name).await?;
    let existing_policy = current_security_policy_file(pool, repository, &ref_name).await?;
    if let Some(expected) = mutation
        .expected_content_sha
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let current_sha = existing_policy
            .as_ref()
            .map(|file| markdown_sha(&file.content))
            .unwrap_or_default();
        if expected != current_sha {
            return Err(RepositoryError::SecurityPolicyConflict);
        }
    }

    let mut files =
        current_branch_files(pool, repository.id, current_ref.as_ref().map(|r| r.id)).await?;
    if let Some(file) = files
        .iter_mut()
        .find(|file| file.path.eq_ignore_ascii_case(&path))
    {
        file.content = markdown.clone();
        file.oid = deterministic_content_oid("blob", &markdown);
        file.byte_size = markdown.len() as i64;
    } else {
        files.push(RepositorySnapshotFile {
            path: path.clone(),
            content: markdown.clone(),
            oid: deterministic_content_oid("blob", &markdown),
            byte_size: markdown.len() as i64,
        });
    }
    files.sort_by(|left, right| left.path.to_lowercase().cmp(&right.path.to_lowercase()));

    let tree_oid = deterministic_content_oid(
        "tree",
        &files
            .iter()
            .map(|file| format!("{}:{}:{}", file.path, file.oid, file.byte_size))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    let parent_oids = current_ref
        .as_ref()
        .map(|commit| vec![commit.oid.clone()])
        .unwrap_or_default();
    let commit_oid = deterministic_content_oid(
        "commit",
        &format!(
            "{}:{}:{}:{}:{}",
            repository.id, ref_name, tree_oid, commit_message, content_sha
        ),
    );
    let commit = replace_repository_snapshot(
        pool,
        repository.id,
        RepositorySnapshot {
            commit: CreateCommit {
                oid: commit_oid.clone(),
                author_user_id: Some(actor_user_id),
                committer_user_id: Some(actor_user_id),
                message: commit_message.clone(),
                tree_oid: Some(tree_oid),
                parent_oids,
                committed_at: Utc::now(),
            },
            branch_name: ref_name.clone(),
            files,
        },
    )
    .await?;

    sqlx::query(
        r#"
        INSERT INTO repository_security_policies (
            repository_id, path, ref_name, source_commit_id, blob_oid, content_sha,
            markdown, rendered_html, published, updated_by_user_id, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, now())
        ON CONFLICT (repository_id, lower(path))
        DO UPDATE SET ref_name = EXCLUDED.ref_name,
                      source_commit_id = EXCLUDED.source_commit_id,
                      blob_oid = EXCLUDED.blob_oid,
                      content_sha = EXCLUDED.content_sha,
                      markdown = EXCLUDED.markdown,
                      rendered_html = EXCLUDED.rendered_html,
                      published = true,
                      updated_by_user_id = EXCLUDED.updated_by_user_id,
                      updated_at = now()
        "#,
    )
    .bind(repository.id)
    .bind(&path)
    .bind(&ref_name)
    .bind(commit.id)
    .bind(deterministic_content_oid("blob", &markdown))
    .bind(&content_sha)
    .bind(&markdown)
    .bind(&rendered.html)
    .bind(actor_user_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO security_audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'repository.security_policy.upsert', 'repository', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(repository.id)
    .bind(json!({
        "repositoryId": repository.id,
        "path": path,
        "ref": ref_name,
        "commitOid": commit.oid,
        "contentSha": content_sha,
    }))
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(Debug)]
struct CurrentPolicyFile {
    content: String,
}

#[derive(Debug)]
struct CurrentBranchCommit {
    id: Uuid,
    oid: String,
}

async fn current_branch_commit(
    pool: &PgPool,
    repository_id: Uuid,
    ref_name: &str,
) -> Result<Option<CurrentBranchCommit>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT commits.id, commits.oid
        FROM repository_git_refs
        JOIN commits ON commits.id = repository_git_refs.target_commit_id
        WHERE repository_git_refs.repository_id = $1
          AND repository_git_refs.name IN ($2, 'refs/heads/' || $2)
        ORDER BY CASE WHEN repository_git_refs.name = 'refs/heads/' || $2 THEN 0 ELSE 1 END
        LIMIT 1
        "#,
    )
    .bind(repository_id)
    .bind(ref_name)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| CurrentBranchCommit {
        id: row.get("id"),
        oid: row.get("oid"),
    }))
}

async fn current_branch_files(
    pool: &PgPool,
    repository_id: Uuid,
    commit_id: Option<Uuid>,
) -> Result<Vec<RepositorySnapshotFile>, RepositoryError> {
    let Some(commit_id) = commit_id else {
        return Ok(Vec::new());
    };
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
        .map(|row| RepositorySnapshotFile {
            path: row.get("path"),
            content: row.get("content"),
            oid: row.get("oid"),
            byte_size: row.get("byte_size"),
        })
        .collect())
}

async fn current_security_policy_file(
    pool: &PgPool,
    repository: &Repository,
    ref_name: &str,
) -> Result<Option<CurrentPolicyFile>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT repository_files.content
        FROM repository_files
        JOIN repository_git_refs
          ON repository_git_refs.repository_id = repository_files.repository_id
         AND repository_git_refs.target_commit_id = repository_files.commit_id
        WHERE repository_files.repository_id = $1
          AND repository_git_refs.name IN ($2, 'refs/heads/' || $2)
          AND lower(repository_files.path) IN ('security.md', '.github/security.md', 'docs/security.md')
        ORDER BY CASE lower(repository_files.path)
            WHEN 'security.md' THEN 0
            WHEN '.github/security.md' THEN 1
            ELSE 2
        END
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(ref_name)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| CurrentPolicyFile {
        content: row.get("content"),
    }))
}

fn normalize_policy_path(path: Option<&str>) -> Result<String, RepositoryError> {
    let path = path.unwrap_or("SECURITY.md").trim();
    let normalized = if path.is_empty() { "SECURITY.md" } else { path };
    match normalized.to_ascii_lowercase().as_str() {
        "security.md" | ".github/security.md" | "docs/security.md" => Ok(normalized.to_owned()),
        _ => Err(RepositoryError::InvalidSecurityPolicy(
            "policy path must be SECURITY.md, .github/SECURITY.md, or docs/SECURITY.md".to_owned(),
        )),
    }
}

fn normalize_policy_ref(
    repository: &Repository,
    ref_name: Option<&str>,
) -> Result<String, RepositoryError> {
    let ref_name = ref_name.unwrap_or(&repository.default_branch).trim();
    let ref_name = ref_name.strip_prefix("refs/heads/").unwrap_or(ref_name);
    if ref_name.is_empty() || ref_name.contains("..") || ref_name.starts_with('/') {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "policy branch is invalid".to_owned(),
        ));
    }
    Ok(ref_name.to_owned())
}

fn normalize_policy_markdown(markdown: &str) -> Result<String, RepositoryError> {
    let markdown = markdown.trim();
    if markdown.is_empty() {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "policy markdown must not be empty".to_owned(),
        ));
    }
    if markdown.len() > 128 * 1024 {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "policy markdown must be 128 KiB or smaller".to_owned(),
        ));
    }
    Ok(markdown.to_owned())
}

fn normalize_policy_commit_message(message: &str) -> Result<String, RepositoryError> {
    let message = message.trim();
    if message.is_empty() {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "commit message must not be empty".to_owned(),
        ));
    }
    if message.len() > 240 {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "commit message must be 240 characters or fewer".to_owned(),
        ));
    }
    Ok(message.to_owned())
}

fn deterministic_content_oid(kind: &str, content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update([0]);
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn policy_heading_outline(html: &str) -> Vec<SecurityPolicyHeading> {
    Regex::new(r#"<h([1-6]) id="([^"]+)">(.*?)</h[1-6]>"#)
        .expect("heading outline regex")
        .captures_iter(html)
        .map(|captures| {
            let level = captures[1].parse::<i32>().unwrap_or(1);
            let id = captures[2].to_owned();
            let text = strip_tags(&captures[3])
                .trim()
                .trim_start_matches('#')
                .trim()
                .to_owned();
            SecurityPolicyHeading {
                href: format!("#{id}"),
                id,
                level,
                text,
            }
        })
        .collect()
}

fn strip_tags(value: &str) -> String {
    Regex::new(r"<[^>]+>")
        .expect("tag regex")
        .replace_all(value, |captures: &Captures<'_>| {
            if captures[0].starts_with("</") {
                " "
            } else {
                ""
            }
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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

fn repository_raw_href(repository: &Repository, ref_name: &str, path: &str) -> String {
    format!(
        "/{}/{}/raw/{}/{}",
        repository.owner_login,
        repository.name,
        percent_encode_segment(ref_name),
        percent_encode_path(path)
    )
}

fn repository_history_href(repository: &Repository, ref_name: &str, path: &str) -> String {
    format!(
        "/{}/{}/commits/{}/{}",
        repository.owner_login,
        repository.name,
        percent_encode_segment(ref_name),
        percent_encode_path(path)
    )
}

fn percent_encode_segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn percent_encode_path(path: &str) -> String {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .map(percent_encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}
