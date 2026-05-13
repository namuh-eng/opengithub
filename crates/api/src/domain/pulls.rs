use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Postgres, Row, Transaction};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use crate::api_types::ListEnvelope;

use super::{
    actions::{trigger_workflows_for_pull_request, TriggerWorkflowsForPullRequest},
    branch_policies::{evaluate_branch_policy, BranchPolicyOperation, BranchPolicySummary},
    issues::{
        append_timeline_event, insert_issue_with_number, issue_from_row, next_issue_number,
        reaction_summaries, repository_for_actor, search_error_to_collaboration, user_login,
        CollaborationError, CreateComment, CreateIssue, Issue, IssueListLabel,
        IssueListMetadataOption, IssueListMilestone, IssueListUser, IssueState, ReactionSummary,
        TimelineEvent,
    },
    markdown::{render_markdown, RenderMarkdownInput},
    notifications::{
        create_notification, should_deliver_notification, CreateNotification,
        NotificationDeliveryCheck,
    },
    permissions::RepositoryRole,
    projects::{
        auto_add_project_items_for_repository_event, run_project_item_automation,
        ProjectAutomationEvent, ProjectAutomationInput,
    },
    repositories::{
        get_repository, repository_permission_for_user, Repository, RepositoryVisibility,
        RepositoryWatchEvent,
    },
    search::{upsert_search_document, SearchDocumentKind, UpsertSearchDocument},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestState {
    #[default]
    Open,
    Closed,
    Merged,
}

impl PullRequestState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Merged => "merged",
        }
    }
}

impl TryFrom<&str> for PullRequestState {
    type Error = CollaborationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "open" => Ok(Self::Open),
            "closed" => Ok(Self::Closed),
            "merged" => Ok(Self::Merged),
            other => Err(CollaborationError::InvalidState(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequest {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub issue_id: Uuid,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: PullRequestState,
    pub is_draft: bool,
    pub author_user_id: Uuid,
    pub head_ref: String,
    pub base_ref: String,
    pub head_repository_id: Option<Uuid>,
    pub base_repository_id: Option<Uuid>,
    pub merge_commit_id: Option<Uuid>,
    pub merged_by_user_id: Option<Uuid>,
    pub merged_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePullRequest {
    pub repository_id: Uuid,
    pub actor_user_id: Uuid,
    pub title: String,
    pub body: Option<String>,
    pub head_ref: String,
    pub base_ref: String,
    pub head_repository_id: Option<Uuid>,
    pub is_draft: bool,
    pub label_ids: Vec<Uuid>,
    pub milestone_id: Option<Uuid>,
    pub assignee_user_ids: Vec<Uuid>,
    pub reviewer_user_ids: Vec<Uuid>,
    pub template_slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePullRequestState {
    pub actor_user_id: Uuid,
    pub state: PullRequestState,
    pub merge_commit_id: Option<Uuid>,
    pub method: Option<MergeMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePullRequestDraftState {
    pub actor_user_id: Uuid,
    pub is_draft: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePullRequestMetadata {
    pub actor_user_id: Uuid,
    pub label_ids: Vec<Uuid>,
    pub assignee_user_ids: Vec<Uuid>,
    pub milestone_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePullRequestReviewRequests {
    pub actor_user_id: Uuid,
    pub reviewer_user_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePullRequestSubscription {
    pub actor_user_id: Uuid,
    pub subscribed: bool,
    pub custom_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestDetail {
    pub pull_request: PullRequest,
    pub issue: Issue,
    pub href: String,
}

#[derive(Debug, Clone)]
pub struct ComparePullRequestRefsInput<'a> {
    pub repository_id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub base_ref: &'a str,
    pub head_ref: &'a str,
    pub head_repository_id: Option<Uuid>,
    pub commit_limit: i64,
    pub file_limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestReviewSummary {
    pub state: String,
    pub required: bool,
    pub requested_reviewers: Vec<IssueListUser>,
    pub reviewer_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestChecksSummary {
    pub status: String,
    pub conclusion: Option<String>,
    pub total_count: i64,
    pub completed_count: i64,
    pub failed_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestChecksView {
    pub repository: PullRequestDetailRepository,
    pub pull_request: PullRequestChecksPullRequest,
    pub summary: PullRequestChecksSummary,
    pub required_status_checks: Vec<String>,
    pub check_runs: Vec<PullRequestCheckRun>,
    pub can_rerun: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestChecksPullRequest {
    pub id: Uuid,
    pub number: i64,
    pub title: String,
    pub state: PullRequestState,
    pub head_ref: String,
    pub base_ref: String,
    pub head_sha: Option<String>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestCheckRun {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub required: bool,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output_title: Option<String>,
    pub output_summary: Option<String>,
    pub annotations_count: i64,
    pub details_href: Option<String>,
    pub rerun_href: Option<String>,
    pub annotations: Vec<PullRequestCheckAnnotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestCheckAnnotation {
    pub id: Uuid,
    pub path: Option<String>,
    pub start_line: Option<i32>,
    pub end_line: Option<i32>,
    pub level: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestTaskProgress {
    pub completed: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LinkedIssueHint {
    pub number: i64,
    pub state: String,
    pub title: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestListItem {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub repository_owner: String,
    pub repository_name: String,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: PullRequestState,
    pub is_draft: bool,
    pub author: IssueListUser,
    pub author_role: String,
    pub labels: Vec<IssueListLabel>,
    pub milestone: Option<IssueListMilestone>,
    pub comment_count: i64,
    pub linked_issues: Vec<LinkedIssueHint>,
    pub review: PullRequestReviewSummary,
    pub checks: PullRequestChecksSummary,
    pub task_progress: PullRequestTaskProgress,
    pub head_ref: String,
    pub base_ref: String,
    pub href: String,
    pub checks_href: String,
    pub reviews_href: String,
    pub comments_href: String,
    pub linked_issues_href: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestListCounts {
    pub open: i64,
    pub closed: i64,
    pub merged: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestListFilters {
    pub query: String,
    pub state: PullRequestState,
    pub author: Option<String>,
    pub labels: Vec<String>,
    pub milestone: Option<String>,
    pub no_milestone: bool,
    pub assignee: Option<String>,
    pub no_assignee: bool,
    pub project: Option<String>,
    pub review: Option<String>,
    pub checks: Option<String>,
    pub sort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestFilterOptions {
    pub labels: Vec<IssueListLabel>,
    pub users: Vec<IssueListUser>,
    pub milestones: Vec<IssueListMilestone>,
    pub projects: Vec<IssueListMetadataOption>,
    pub review_states: Vec<String>,
    pub check_states: Vec<String>,
    pub sort_options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestListRepository {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub visibility: RepositoryVisibility,
    pub default_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestListPreferences {
    pub dismissed_contributor_banner: bool,
    pub dismissed_contributor_banner_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestListView {
    pub items: Vec<PullRequestListItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub open_count: i64,
    pub closed_count: i64,
    pub merged_count: i64,
    pub counts: PullRequestListCounts,
    pub filters: PullRequestListFilters,
    pub filter_options: PullRequestFilterOptions,
    pub viewer_permission: Option<String>,
    pub repository: PullRequestListRepository,
    pub preferences: PullRequestListPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GlobalPullRequestScope {
    Created,
    Assigned,
    Mentioned,
    ReviewRequests,
}

impl GlobalPullRequestScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Assigned => "assigned",
            Self::Mentioned => "mentioned",
            Self::ReviewRequests => "review_requests",
        }
    }
}

impl TryFrom<&str> for GlobalPullRequestScope {
    type Error = CollaborationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "created" => Ok(Self::Created),
            "assigned" => Ok(Self::Assigned),
            "mentioned" => Ok(Self::Mentioned),
            "review_requests" | "review-requests" | "reviewRequests" => Ok(Self::ReviewRequests),
            other => Err(CollaborationError::InvalidIssueFilter(format!(
                "scope must be created, assigned, mentioned, or review_requests; got {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalPullRequestListQuery {
    pub scope: GlobalPullRequestScope,
    pub query: Option<String>,
    pub state: Option<PullRequestState>,
    pub repository: Option<String>,
    pub labels: Vec<String>,
    pub milestone: Option<String>,
    pub sort: String,
}

impl Default for GlobalPullRequestListQuery {
    fn default() -> Self {
        Self {
            scope: GlobalPullRequestScope::Created,
            query: Some("is:pr is:open".to_owned()),
            state: Some(PullRequestState::Open),
            repository: None,
            labels: Vec::new(),
            milestone: None,
            sort: "updated-desc".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalPullRequestCounts {
    pub created: i64,
    pub assigned: i64,
    pub mentioned: i64,
    pub review_requests: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalPullRequestFilters {
    pub scope: GlobalPullRequestScope,
    pub query: String,
    pub state: Option<PullRequestState>,
    pub repository: Option<String>,
    pub labels: Vec<String>,
    pub milestone: Option<String>,
    pub sort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalPullRequestRepositoryOption {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub full_name: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalPullRequestFilterOptions {
    pub repositories: Vec<GlobalPullRequestRepositoryOption>,
    pub labels: Vec<IssueListLabel>,
    pub milestones: Vec<IssueListMilestone>,
    pub sort_options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalPullRequestListView {
    pub items: Vec<PullRequestListItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub counts: GlobalPullRequestCounts,
    pub filters: GlobalPullRequestFilters,
    pub filter_options: GlobalPullRequestFilterOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestDetailRepository {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub visibility: RepositoryVisibility,
    pub default_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestDetailStats {
    pub commits: i64,
    pub files: i64,
    pub additions: i64,
    pub deletions: i64,
    pub comments: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestReviewStatus {
    pub reviewer: IssueListUser,
    pub state: String,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestSubscriptionState {
    pub subscribed: bool,
    pub reason: String,
    pub custom_events: Vec<String>,
    pub can_customize: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MergeMethod {
    #[default]
    Squash,
    MergeCommit,
    Rebase,
}

impl MergeMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Squash => "squash",
            Self::MergeCommit => "merge_commit",
            Self::Rebase => "rebase",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestMergeBlocker {
    pub code: String,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone)]
pub struct MergePullRequestInput {
    pub actor_user_id: Uuid,
    pub method: MergeMethod,
    pub commit_title: Option<String>,
    pub commit_body: Option<String>,
    pub delete_branch: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum MergePullRequestError {
    #[error(transparent)]
    Collaboration(#[from] CollaborationError),
    #[error("pull request merge is blocked")]
    Blocked {
        summary: String,
        blockers: Vec<PullRequestMergeBlocker>,
    },
}

impl From<sqlx::Error> for MergePullRequestError {
    fn from(error: sqlx::Error) -> Self {
        Self::Collaboration(CollaborationError::Sqlx(error))
    }
}

pub type BranchProtectionSummary = BranchPolicySummary;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestMergeability {
    pub state: String,
    pub can_merge: bool,
    pub can_close: bool,
    pub can_reopen: bool,
    pub can_mark_ready: bool,
    pub default_method: MergeMethod,
    pub methods: Vec<MergeMethod>,
    pub branch_protection: BranchProtectionSummary,
    pub blockers: Vec<PullRequestMergeBlocker>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestDetailMetadataOptions {
    pub labels: Vec<IssueListLabel>,
    pub assignees: Vec<IssueListUser>,
    pub milestones: Vec<IssueListMilestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestDetailView {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub repository: PullRequestDetailRepository,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub body_html: String,
    pub state: PullRequestState,
    pub is_draft: bool,
    pub author: IssueListUser,
    pub author_role: String,
    pub head_ref: String,
    pub base_ref: String,
    pub labels: Vec<IssueListLabel>,
    pub milestone: Option<IssueListMilestone>,
    pub assignees: Vec<IssueListUser>,
    pub requested_reviewers: Vec<IssueListUser>,
    pub latest_reviews: Vec<PullRequestReviewStatus>,
    pub linked_issues: Vec<LinkedIssueHint>,
    pub participants: Vec<IssueListUser>,
    pub review: PullRequestReviewSummary,
    pub checks: PullRequestChecksSummary,
    pub task_progress: PullRequestTaskProgress,
    pub stats: PullRequestDetailStats,
    pub subscription: PullRequestSubscriptionState,
    pub mergeability: PullRequestMergeability,
    pub metadata_options: PullRequestDetailMetadataOptions,
    pub href: String,
    pub commits_href: String,
    pub checks_href: String,
    pub files_href: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub viewer_permission: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestTimelineComment {
    pub id: Uuid,
    pub body: String,
    pub body_html: String,
    pub is_minimized: bool,
    pub reactions: Vec<ReactionSummary>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestTimelineItem {
    pub id: Uuid,
    pub event_type: String,
    pub actor: Option<IssueListUser>,
    pub comment: Option<PullRequestTimelineComment>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestCompareStatus {
    SameRef,
    NoDiff,
    Ahead,
    Diverged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompareRef {
    pub repository: PullRequestListRepository,
    pub name: String,
    pub short_name: String,
    pub kind: String,
    pub oid: String,
    pub commit_id: Uuid,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompareCommit {
    pub id: Uuid,
    pub oid: String,
    pub short_oid: String,
    pub message: String,
    pub author_login: Option<String>,
    pub committed_at: DateTime<Utc>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompareFileStatus {
    Added,
    Modified,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompareFile {
    pub path: String,
    pub status: CompareFileStatus,
    pub additions: i64,
    pub deletions: i64,
    pub byte_size: i64,
    pub blob_oid: Option<String>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestCompareView {
    pub repository: PullRequestListRepository,
    pub viewer_permission: Option<String>,
    pub base: CompareRef,
    pub head: CompareRef,
    pub status: PullRequestCompareStatus,
    pub ahead_by: i64,
    pub behind_by: i64,
    pub total_commits: i64,
    pub total_files: i64,
    pub commits: Vec<CompareCommit>,
    pub files: Vec<CompareFile>,
    pub additions: i64,
    pub deletions: i64,
    pub default_branch_href: String,
    pub pull_list_href: String,
    pub compare_href: String,
    pub swap_href: String,
    pub create_options: PullRequestCreateOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestTemplateOption {
    pub slug: String,
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestCreateOptions {
    pub can_create: bool,
    pub templates: Vec<PullRequestTemplateOption>,
    pub labels: Vec<IssueListLabel>,
    pub users: Vec<IssueListUser>,
    pub milestones: Vec<IssueListMilestone>,
    pub fork_repositories: Vec<PullRequestCompareRepositoryOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestCompareRepositoryOption {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub visibility: RepositoryVisibility,
    pub default_branch: String,
    pub href: String,
    pub compare_href: String,
    pub is_base: bool,
    pub is_selected_head: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestDiffReviewSettings {
    pub view: String,
    pub whitespace: String,
    pub commit: Option<String>,
    pub filter: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestDiffReviewQuery {
    pub view: String,
    pub whitespace: String,
    pub commit: Option<String>,
    pub filter: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

impl Default for PullRequestDiffReviewQuery {
    fn default() -> Self {
        Self {
            view: "unified".to_owned(),
            whitespace: "show".to_owned(),
            commit: None,
            filter: None,
            page: 1,
            page_size: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestDiffFileTreeItem {
    pub id: Uuid,
    pub path: String,
    pub status: String,
    pub additions: i64,
    pub deletions: i64,
    pub viewed: bool,
    pub version_key: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PullRequestDiffLineKind {
    Context,
    Added,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestDiffLine {
    pub kind: PullRequestDiffLineKind,
    pub old_line: Option<i64>,
    pub new_line: Option<i64>,
    pub content: String,
    pub position: i64,
    pub comment_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestDiffHunk {
    pub id: Uuid,
    pub header: String,
    pub old_start: i64,
    pub old_lines: i64,
    pub new_start: i64,
    pub new_lines: i64,
    pub lines: Vec<PullRequestDiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestDiffReviewComment {
    pub id: Uuid,
    pub author: IssueListUser,
    pub body: String,
    pub body_html: String,
    pub path: String,
    pub side: String,
    pub old_line: Option<i64>,
    pub new_line: Option<i64>,
    pub position: Option<i64>,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreatePullRequestReviewDraftComment {
    pub pull_request_id: Uuid,
    pub actor_user_id: Uuid,
    pub file_id: Uuid,
    pub body: String,
    pub side: String,
    pub old_line: Option<i64>,
    pub new_line: Option<i64>,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePullRequestReviewDraftComment {
    pub pull_request_id: Uuid,
    pub actor_user_id: Uuid,
    pub draft_comment_id: Uuid,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SubmitPullRequestReview {
    pub pull_request_id: Uuid,
    pub actor_user_id: Uuid,
    pub body: Option<String>,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestSubmittedReview {
    pub id: Uuid,
    pub reviewer: IssueListUser,
    pub state: String,
    pub body: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub published_comment_count: i64,
    pub pending_review: PullRequestDiffPendingReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestDiffFile {
    pub id: Uuid,
    pub path: String,
    pub status: String,
    pub additions: i64,
    pub deletions: i64,
    pub byte_size: i64,
    pub blob_oid: Option<String>,
    pub language: Option<String>,
    pub viewed: bool,
    pub viewed_at: Option<DateTime<Utc>>,
    pub version_key: String,
    pub href: String,
    pub hunks: Vec<PullRequestDiffHunk>,
    pub comments: Vec<PullRequestDiffReviewComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestDiffPendingReview {
    pub draft_id: Option<Uuid>,
    pub comment_count: i64,
    pub summary_body: Option<String>,
    pub review_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestViewedFileState {
    pub file_id: Uuid,
    pub path: String,
    pub viewed: bool,
    pub viewed_at: Option<DateTime<Utc>>,
    pub version_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestDiffReviewView {
    pub pull_request: PullRequestDetailView,
    pub settings: PullRequestDiffReviewSettings,
    pub total_files: i64,
    pub page: i64,
    pub page_size: i64,
    pub has_more: bool,
    pub file_tree: Vec<PullRequestDiffFileTreeItem>,
    pub files: Vec<PullRequestDiffFile>,
    pub commits: Vec<CompareCommit>,
    pub pending_review: PullRequestDiffPendingReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PullRequestListQuery {
    pub query: Option<String>,
    pub state: PullRequestState,
    pub viewer_user_id: Option<Uuid>,
    pub author: Option<String>,
    pub labels: Vec<String>,
    pub milestone: Option<String>,
    pub no_milestone: bool,
    pub assignee: Option<String>,
    pub no_assignee: bool,
    pub project: Option<String>,
    pub review: Option<String>,
    pub checks: Option<String>,
    pub sort: String,
}

impl Default for PullRequestListQuery {
    fn default() -> Self {
        Self {
            query: Some("is:pr is:open".to_owned()),
            state: PullRequestState::Open,
            viewer_user_id: None,
            author: None,
            labels: Vec::new(),
            milestone: None,
            no_milestone: false,
            assignee: None,
            no_assignee: false,
            project: None,
            review: None,
            checks: None,
            sort: "updated-desc".to_owned(),
        }
    }
}

pub async fn compare_pull_request_refs_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
    base_ref: &str,
    head_ref: &str,
    commit_limit: i64,
    file_limit: i64,
) -> Result<PullRequestCompareView, CollaborationError> {
    compare_pull_request_refs_for_viewer_with_head(
        pool,
        ComparePullRequestRefsInput {
            repository_id,
            actor_user_id,
            base_ref,
            head_ref,
            head_repository_id: None,
            commit_limit,
            file_limit,
        },
    )
    .await
}

pub async fn compare_pull_request_refs_for_viewer_with_head(
    pool: &PgPool,
    input: ComparePullRequestRefsInput<'_>,
) -> Result<PullRequestCompareView, CollaborationError> {
    let repository = get_repository(pool, input.repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    let viewer_permission = match input.actor_user_id {
        Some(user_id) => {
            repository_viewer_permission(pool, &repository, user_id, RepositoryRole::Read).await?
        }
        None if repository.visibility == RepositoryVisibility::Public => Some("read".to_owned()),
        None => return Err(CollaborationError::RepositoryAccessDenied),
    };
    let head_repository = if let Some(head_repository_id) = input.head_repository_id {
        get_repository(pool, head_repository_id)
            .await
            .map_err(|error| match error {
                super::repositories::RepositoryError::Sqlx(error) => {
                    CollaborationError::Sqlx(error)
                }
                _ => CollaborationError::RepositoryNotFound,
            })?
            .ok_or(CollaborationError::RepositoryNotFound)?
    } else {
        repository.clone()
    };
    validate_compare_head_repository(pool, &repository, &head_repository, input.actor_user_id)
        .await?;

    let base = resolve_compare_ref(pool, &repository, input.base_ref).await?;
    let head = resolve_compare_ref(pool, &head_repository, input.head_ref).await?;
    let commit_limit = input.commit_limit.clamp(1, 250);
    let file_limit = input.file_limit.clamp(1, 500);

    let base_ancestors = commit_ancestor_oids(pool, repository.id, &base.oid).await?;
    let head_ancestors = commit_ancestor_oids(pool, head_repository.id, &head.oid).await?;
    let ahead_oids = head_ancestors
        .iter()
        .filter(|oid| !base_ancestors.contains(*oid))
        .cloned()
        .collect::<HashSet<_>>();
    let behind_oids = base_ancestors
        .iter()
        .filter(|oid| !head_ancestors.contains(*oid))
        .cloned()
        .collect::<HashSet<_>>();
    let mut commits = compare_commits(pool, &head_repository, &ahead_oids, commit_limit).await?;
    commits.sort_by(|left, right| {
        left.committed_at
            .cmp(&right.committed_at)
            .then(left.oid.cmp(&right.oid))
    });
    let files = compare_files(
        pool,
        &repository,
        &head_repository,
        &base,
        &head,
        file_limit,
    )
    .await?;
    let additions = files.iter().map(|file| file.additions).sum();
    let deletions = files.iter().map(|file| file.deletions).sum();
    let same_target = base.commit_id == head.commit_id || base.oid == head.oid;
    let status = if same_target {
        PullRequestCompareStatus::SameRef
    } else if ahead_oids.is_empty() && files.is_empty() {
        PullRequestCompareStatus::NoDiff
    } else if behind_oids.is_empty() {
        PullRequestCompareStatus::Ahead
    } else {
        PullRequestCompareStatus::Diverged
    };
    let compare_href = compare_href_for_repositories(
        &repository,
        &head_repository,
        &base.short_name,
        &head.short_name,
    );
    let swap_href = compare_href_for_repositories(
        &repository,
        &head_repository,
        &head.short_name,
        &base.short_name,
    );
    let can_write_base = viewer_permission
        .as_deref()
        .is_some_and(|role| matches!(role, "write" | "admin" | "owner"));
    let can_create_from_head = if let Some(actor_user_id) = input.actor_user_id {
        can_write_base
            || can_open_pull_from_head_repository(
                pool,
                &repository,
                &head_repository,
                actor_user_id,
            )
            .await?
    } else {
        false
    };
    let create_options = if can_create_from_head {
        pull_request_create_options(
            pool,
            &repository,
            input.actor_user_id,
            &head_repository,
            &base.short_name,
            &head.short_name,
            can_write_base,
        )
        .await?
    } else if input.actor_user_id.is_some() {
        PullRequestCreateOptions {
            fork_repositories: pull_request_fork_options(
                pool,
                &repository,
                input.actor_user_id,
                &head_repository,
                input.base_ref,
                input.head_ref,
            )
            .await?,
            ..PullRequestCreateOptions::default()
        }
    } else {
        PullRequestCreateOptions::default()
    };

    Ok(PullRequestCompareView {
        repository: PullRequestListRepository {
            id: repository.id,
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            visibility: repository.visibility,
            default_branch: repository.default_branch.clone(),
        },
        viewer_permission,
        base,
        head,
        status,
        ahead_by: ahead_oids.len() as i64,
        behind_by: behind_oids.len() as i64,
        total_commits: ahead_oids.len() as i64,
        total_files: files.len() as i64,
        commits,
        files,
        additions,
        deletions,
        default_branch_href: format!(
            "/{}/{}/compare/{}...{}",
            repository.owner_login,
            repository.name,
            encode_path_component(&repository.default_branch),
            encode_path_component(&repository.default_branch)
        ),
        pull_list_href: format!("/{}/{}/pulls", repository.owner_login, repository.name),
        compare_href,
        swap_href,
        create_options,
    })
}

pub async fn create_pull_request(
    pool: &PgPool,
    mut input: CreatePullRequest,
) -> Result<PullRequestDetail, CollaborationError> {
    let repository = get_repository(pool, input.repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    let head_repository =
        resolve_create_head_repository(pool, &repository, input.head_repository_id).await?;
    let can_write_base =
        can_write_repository_id(pool, input.repository_id, input.actor_user_id).await?;
    if !can_write_base {
        if !can_open_pull_from_head_repository(
            pool,
            &repository,
            &head_repository,
            input.actor_user_id,
        )
        .await?
        {
            return Err(CollaborationError::RepositoryAccessDenied);
        }
        if !input.label_ids.is_empty()
            || !input.assignee_user_ids.is_empty()
            || !input.reviewer_user_ids.is_empty()
            || input.milestone_id.is_some()
        {
            return Err(CollaborationError::InvalidIssueField {
                field_key: "metadata".to_owned(),
                message: "labels, assignees, reviewers, and milestones require write access to the base repository".to_owned(),
            });
        }
    }
    let compare = match compare_pull_request_refs_for_viewer(
        pool,
        input.repository_id,
        Some(input.actor_user_id),
        &input.base_ref,
        &input.head_ref,
        250,
        500,
    )
    .await
    {
        Ok(compare) => Some(compare),
        Err(CollaborationError::InvalidIssueFilter(message))
            if message.contains("was not found") =>
        {
            None
        }
        Err(error) => return Err(error),
    };
    let compare = if input.head_repository_id.is_some() {
        match compare_pull_request_refs_for_viewer_with_head(
            pool,
            ComparePullRequestRefsInput {
                repository_id: input.repository_id,
                actor_user_id: Some(input.actor_user_id),
                base_ref: &input.base_ref,
                head_ref: &input.head_ref,
                head_repository_id: input.head_repository_id,
                commit_limit: 250,
                file_limit: 500,
            },
        )
        .await
        {
            Ok(compare) => Some(compare),
            Err(CollaborationError::InvalidIssueFilter(message))
                if message.contains("was not found") =>
            {
                None
            }
            Err(error) => return Err(error),
        }
    } else {
        compare
    };
    if compare.as_ref().is_some_and(|view| {
        matches!(
            view.status,
            PullRequestCompareStatus::SameRef | PullRequestCompareStatus::NoDiff
        )
    }) {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "headRef".to_owned(),
            message: "base and compare refs do not contain changes".to_owned(),
        });
    }
    reject_duplicate_open_pull_request(pool, &input).await?;
    validate_pull_request_create_metadata(pool, &repository, &input).await?;
    apply_pull_request_template(pool, &mut input).await?;

    let number = next_issue_number(pool, input.repository_id).await?;
    let label_ids = input.label_ids.clone();
    let assignee_user_ids = input.assignee_user_ids.clone();
    let reviewer_user_ids = input.reviewer_user_ids.clone();
    let issue = insert_issue_with_number(
        pool,
        CreateIssue {
            repository_id: input.repository_id,
            actor_user_id: input.actor_user_id,
            title: input.title.clone(),
            body: input.body.clone(),
            template_id: None,
            template_slug: None,
            field_values: std::collections::HashMap::new(),
            milestone_id: input.milestone_id,
            label_ids: label_ids.clone(),
            assignee_user_ids: assignee_user_ids.clone(),
            attachments: Vec::new(),
        },
        number,
    )
    .await?;

    let row = sqlx::query(
        r#"
        INSERT INTO pull_requests (
            repository_id,
            issue_id,
            number,
            title,
            body,
            author_user_id,
            head_ref,
            base_ref,
            head_repository_id,
            base_repository_id,
            is_draft
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, COALESCE($9, $1), $1, $10)
        RETURNING id, repository_id, issue_id, number, title, body, state, author_user_id,
                  head_ref, base_ref, head_repository_id, base_repository_id, merge_commit_id,
                  merged_by_user_id, merged_at, closed_at, created_at, updated_at, is_draft
        "#,
    )
    .bind(input.repository_id)
    .bind(issue.id)
    .bind(number)
    .bind(&input.title)
    .bind(&input.body)
    .bind(input.actor_user_id)
    .bind(&input.head_ref)
    .bind(&input.base_ref)
    .bind(input.head_repository_id)
    .bind(input.is_draft)
    .fetch_one(pool)
    .await?;
    let pull_request = pull_request_from_row(row)?;
    if let Some(compare) = &compare {
        persist_pull_request_snapshot(pool, &pull_request, compare).await?;
    }
    insert_review_requests(pool, &pull_request, input.actor_user_id, &reviewer_user_ids).await?;
    insert_closing_issue_references(pool, &pull_request, input.actor_user_id).await?;
    append_timeline_event(
        pool,
        input.repository_id,
        None,
        Some(pull_request.id),
        Some(input.actor_user_id),
        "opened",
        json!({
            "number": pull_request.number,
            "headRef": pull_request.head_ref,
            "baseRef": pull_request.base_ref,
            "draft": pull_request.is_draft,
            "labels": label_ids.len(),
            "assignees": assignee_user_ids.len(),
            "reviewers": reviewer_user_ids.len(),
            "files": compare.as_ref().map(|view| view.files.len()).unwrap_or(0),
            "commits": compare.as_ref().map(|view| view.commits.len()).unwrap_or(0)
        }),
    )
    .await?;
    notify_pull_request_participants(
        pool,
        &pull_request,
        input.actor_user_id,
        &assignee_user_ids,
        &reviewer_user_ids,
    )
    .await?;
    insert_pull_request_audit_event(pool, &pull_request, input.actor_user_id).await?;
    index_pull_request_search_document(pool, &pull_request, repository.created_by_user_id).await?;
    auto_add_project_items_for_repository_event(
        pool,
        ProjectAutomationInput {
            actor_user_id: input.actor_user_id,
            repository_id: pull_request.repository_id,
            issue_id: None,
            pull_request_id: Some(pull_request.id),
            event: ProjectAutomationEvent::ItemAdded,
        },
    )
    .await
    .map_err(|error| match error {
        super::projects::ProjectsError::Sqlx(error) => CollaborationError::Sqlx(error),
        _ => CollaborationError::PullRequestNotFound,
    })?;
    trigger_workflows_for_pull_request(
        pool,
        TriggerWorkflowsForPullRequest {
            repository_id: repository.id,
            actor_user_id: input.actor_user_id,
            pull_request_id: pull_request.id,
            action: "opened".to_owned(),
        },
    )
    .await
    .map_err(|error| match error {
        super::actions::AutomationError::Sqlx(error) => CollaborationError::Sqlx(error),
        super::actions::AutomationError::RepositoryNotFound => {
            CollaborationError::RepositoryNotFound
        }
        super::actions::AutomationError::RepositoryAccessDenied => {
            CollaborationError::RepositoryAccessDenied
        }
        other => CollaborationError::InvalidIssueField {
            field_key: "actions".to_owned(),
            message: other.to_string(),
        },
    })?;

    let href = pull_request_href(&repository, pull_request.number);
    Ok(PullRequestDetail {
        pull_request,
        issue,
        href,
    })
}

pub async fn list_pull_requests(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
    state: Option<PullRequestState>,
    page: i64,
    page_size: i64,
) -> Result<ListEnvelope<PullRequest>, CollaborationError> {
    require_repository_read(pool, repository_id, actor_user_id).await?;
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;
    let state_filter = state.as_ref().map(PullRequestState::as_str);

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM pull_requests
        WHERE repository_id = $1
          AND ($2::text IS NULL OR state = $2)
        "#,
    )
    .bind(repository_id)
    .bind(state_filter)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT id, repository_id, issue_id, number, title, body, state, author_user_id,
               head_ref, base_ref, head_repository_id, base_repository_id, merge_commit_id,
               merged_by_user_id, merged_at, closed_at, created_at, updated_at, is_draft
        FROM pull_requests
        WHERE repository_id = $1
          AND ($2::text IS NULL OR state = $2)
        ORDER BY updated_at DESC, number DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(repository_id)
    .bind(state_filter)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(pull_request_from_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListEnvelope {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn repository_pull_request_list_view_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
    filters: PullRequestListQuery,
    page: i64,
    page_size: i64,
) -> Result<PullRequestListView, CollaborationError> {
    let repository = get_repository(pool, repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    let viewer_permission = match actor_user_id {
        Some(user_id) => {
            repository_viewer_permission(pool, &repository, user_id, RepositoryRole::Read).await?
        }
        None if repository.visibility == RepositoryVisibility::Public => Some("read".to_owned()),
        None => return Err(CollaborationError::RepositoryAccessDenied),
    };
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;
    let state_filter = filters.state.as_str();
    let text_filter = filters
        .query
        .as_deref()
        .map(search_text_from_pull_query)
        .filter(|value| !value.is_empty());

    let open_count = count_pull_request_list_items(
        pool,
        repository_id,
        PullRequestState::Open.as_str(),
        text_filter.as_deref(),
        &filters,
        actor_user_id,
    )
    .await?;
    let closed_count = count_pull_request_list_items(
        pool,
        repository_id,
        PullRequestState::Closed.as_str(),
        text_filter.as_deref(),
        &filters,
        actor_user_id,
    )
    .await?;
    let merged_count = count_pull_request_list_items(
        pool,
        repository_id,
        PullRequestState::Merged.as_str(),
        text_filter.as_deref(),
        &filters,
        actor_user_id,
    )
    .await?;
    let total = match filters.state {
        PullRequestState::Open => open_count,
        PullRequestState::Closed => closed_count,
        PullRequestState::Merged => merged_count,
    };

    let rows = sqlx::query(
        r#"
        SELECT pull_requests.id, pull_requests.repository_id, pull_requests.issue_id,
               pull_requests.number, pull_requests.title, pull_requests.body,
               pull_requests.state, pull_requests.author_user_id, pull_requests.head_ref,
               pull_requests.base_ref, pull_requests.head_repository_id,
               pull_requests.base_repository_id, pull_requests.merge_commit_id,
               pull_requests.merged_by_user_id, pull_requests.merged_at,
               pull_requests.closed_at, pull_requests.created_at, pull_requests.updated_at
        FROM pull_requests
        JOIN issues ON issues.id = pull_requests.issue_id
        WHERE pull_requests.repository_id = $1
          AND pull_requests.state = $2
          AND (
              $3::text IS NULL
              OR pull_requests.title ILIKE '%' || $3 || '%'
              OR COALESCE(pull_requests.body, '') ILIKE '%' || $3 || '%'
              OR pull_requests.head_ref ILIKE '%' || $3 || '%'
              OR pull_requests.base_ref ILIKE '%' || $3 || '%'
          )
          AND (
              $4::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM users
                  WHERE users.id = pull_requests.author_user_id
                    AND (
                        lower(users.email) = lower($4)
                        OR lower(users.username) = lower($4)
                    )
              )
          )
          AND (
              cardinality($5::text[]) = 0
              OR NOT EXISTS (
                  SELECT 1
                  FROM unnest($5::text[]) wanted_label(name)
                  WHERE NOT EXISTS (
                      SELECT 1
                      FROM issue_labels
                      JOIN labels ON labels.id = issue_labels.label_id
                      WHERE issue_labels.issue_id = issues.id
                        AND lower(labels.name) = lower(wanted_label.name)
                  )
              )
          )
          AND (
              $6::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM milestones
                  WHERE milestones.id = issues.milestone_id
                    AND lower(milestones.title) = lower($6)
              )
          )
          AND (
              $7::bool = false
              OR issues.milestone_id IS NULL
          )
          AND (
              $8::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM issue_assignees
                  JOIN users ON users.id = issue_assignees.user_id
                  WHERE issue_assignees.issue_id = issues.id
                    AND (
                        lower(users.email) = lower($8)
                        OR lower(users.username) = lower($8)
                    )
              )
          )
          AND (
              $9::bool = false
              OR NOT EXISTS (
                  SELECT 1 FROM issue_assignees WHERE issue_assignees.issue_id = issues.id
              )
          )
          AND (
              $10::text IS NULL
              OR ($10 = 'none' AND NOT EXISTS (
                  SELECT 1
                  FROM pull_request_reviews
                  WHERE pull_request_reviews.pull_request_id = pull_requests.id
              ) AND NOT EXISTS (
                  SELECT 1
                  FROM pull_request_review_requests
                  WHERE pull_request_review_requests.pull_request_id = pull_requests.id
              ))
              OR ($10 IN ('approved', 'changes_requested', 'commented') AND EXISTS (
                  SELECT 1
                  FROM pull_request_reviews
                  WHERE pull_request_reviews.pull_request_id = pull_requests.id
                    AND pull_request_reviews.state = $10
              ))
              OR ($10 = 'required' AND EXISTS (
                  SELECT 1
                  FROM pull_request_review_requests
                  WHERE pull_request_review_requests.pull_request_id = pull_requests.id
              ))
              OR ($10 = 'reviewed_by_me' AND $11::uuid IS NOT NULL AND EXISTS (
                  SELECT 1
                  FROM pull_request_reviews
                  WHERE pull_request_reviews.pull_request_id = pull_requests.id
                    AND pull_request_reviews.reviewer_user_id = $11
              ))
              OR ($10 = 'not_reviewed_by_me' AND $11::uuid IS NOT NULL AND NOT EXISTS (
                  SELECT 1
                  FROM pull_request_reviews
                  WHERE pull_request_reviews.pull_request_id = pull_requests.id
                    AND pull_request_reviews.reviewer_user_id = $11
              ))
              OR ($10 = 'review_requested' AND $11::uuid IS NOT NULL AND EXISTS (
                  SELECT 1
                  FROM pull_request_review_requests
                  WHERE pull_request_review_requests.pull_request_id = pull_requests.id
                    AND pull_request_review_requests.requested_user_id = $11
              ))
              OR ($10 = 'team_review_requested' AND $11::uuid IS NOT NULL AND (
                  EXISTS (
                      SELECT 1
                      FROM pull_request_review_requests
                      WHERE pull_request_review_requests.pull_request_id = pull_requests.id
                        AND pull_request_review_requests.requested_user_id = $11
                  )
                  OR EXISTS (
                      SELECT 1
                      FROM pull_request_review_requests
                      JOIN team_memberships requested_memberships
                        ON requested_memberships.user_id = pull_request_review_requests.requested_user_id
                      JOIN team_memberships viewer_memberships
                        ON viewer_memberships.team_id = requested_memberships.team_id
                       AND viewer_memberships.user_id = $11
                      WHERE pull_request_review_requests.pull_request_id = pull_requests.id
                  )
              ))
          )
          AND (
              $12::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM pull_request_checks_summary
                  WHERE pull_request_checks_summary.pull_request_id = pull_requests.id
                    AND (
                        pull_request_checks_summary.status = $12
                        OR pull_request_checks_summary.conclusion = $12
                    )
              )
          )
        ORDER BY
          CASE WHEN $13 = 'best-match' THEN (
              (CASE WHEN pull_requests.title ILIKE $3 || '%' THEN 12 ELSE 0 END)
              + (CASE WHEN pull_requests.title ILIKE '%' || $3 || '%' THEN 8 ELSE 0 END)
              + (CASE WHEN COALESCE(pull_requests.body, '') ILIKE '%' || $3 || '%' THEN 3 ELSE 0 END)
              + (CASE WHEN pull_requests.head_ref ILIKE '%' || $3 || '%' THEN 2 ELSE 0 END)
              + (CASE WHEN pull_requests.base_ref ILIKE '%' || $3 || '%' THEN 1 ELSE 0 END)
          ) END DESC,
          CASE WHEN $13 = 'created-asc' THEN pull_requests.created_at END ASC,
          CASE WHEN $13 = 'created-desc' THEN pull_requests.created_at END DESC,
          CASE WHEN $13 = 'updated-asc' THEN pull_requests.updated_at END ASC,
          CASE WHEN $13 = 'updated-desc' THEN pull_requests.updated_at END DESC,
          CASE WHEN $13 = 'comments-desc' THEN (
              SELECT count(*) FROM comments WHERE comments.pull_request_id = pull_requests.id
          ) END DESC,
          CASE WHEN $13 = 'comments-asc' THEN (
              SELECT count(*) FROM comments WHERE comments.pull_request_id = pull_requests.id
          ) END ASC,
          CASE WHEN $13 = 'reactions-desc' THEN (
              SELECT count(*) FROM reactions WHERE reactions.pull_request_id = pull_requests.id
          ) END DESC,
          CASE WHEN $13 = 'reactions-thumbs_up-desc' THEN (
              SELECT count(*) FROM reactions WHERE reactions.pull_request_id = pull_requests.id AND reactions.content = 'thumbs_up'
          ) END DESC,
          CASE WHEN $13 = 'reactions-thumbs_down-desc' THEN (
              SELECT count(*) FROM reactions WHERE reactions.pull_request_id = pull_requests.id AND reactions.content = 'thumbs_down'
          ) END DESC,
          CASE WHEN $13 = 'reactions-laugh-desc' THEN (
              SELECT count(*) FROM reactions WHERE reactions.pull_request_id = pull_requests.id AND reactions.content = 'laugh'
          ) END DESC,
          CASE WHEN $13 = 'reactions-hooray-desc' THEN (
              SELECT count(*) FROM reactions WHERE reactions.pull_request_id = pull_requests.id AND reactions.content = 'hooray'
          ) END DESC,
          CASE WHEN $13 = 'reactions-confused-desc' THEN (
              SELECT count(*) FROM reactions WHERE reactions.pull_request_id = pull_requests.id AND reactions.content = 'confused'
          ) END DESC,
          CASE WHEN $13 = 'reactions-heart-desc' THEN (
              SELECT count(*) FROM reactions WHERE reactions.pull_request_id = pull_requests.id AND reactions.content = 'heart'
          ) END DESC,
          CASE WHEN $13 = 'reactions-rocket-desc' THEN (
              SELECT count(*) FROM reactions WHERE reactions.pull_request_id = pull_requests.id AND reactions.content = 'rocket'
          ) END DESC,
          CASE WHEN $13 = 'reactions-eyes-desc' THEN (
              SELECT count(*) FROM reactions WHERE reactions.pull_request_id = pull_requests.id AND reactions.content = 'eyes'
          ) END DESC,
          pull_requests.updated_at DESC,
          pull_requests.number DESC
        LIMIT $14 OFFSET $15
        "#,
    )
    .bind(repository_id)
    .bind(state_filter)
    .bind(text_filter.as_deref())
    .bind(filters.author.as_deref())
    .bind(&filters.labels)
    .bind(filters.milestone.as_deref())
    .bind(filters.no_milestone)
    .bind(filters.assignee.as_deref())
    .bind(filters.no_assignee)
    .bind(filters.review.as_deref())
    .bind(filters.viewer_user_id)
    .bind(filters.checks.as_deref())
    .bind(&filters.sort)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let pull_requests = rows
        .into_iter()
        .map(pull_request_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    let items = pull_request_list_items(pool, &repository, pull_requests).await?;
    let preferences = match actor_user_id {
        Some(user_id) => repository_pull_preferences(pool, repository_id, user_id).await?,
        None => PullRequestListPreferences {
            dismissed_contributor_banner: false,
            dismissed_contributor_banner_at: None,
        },
    };

    Ok(PullRequestListView {
        items,
        total,
        page,
        page_size,
        open_count,
        closed_count,
        merged_count,
        counts: PullRequestListCounts {
            open: open_count,
            closed: closed_count,
            merged: merged_count,
        },
        filters: PullRequestListFilters {
            query: filters.query.unwrap_or_else(|| "is:pr is:open".to_owned()),
            state: filters.state,
            author: filters.author,
            labels: filters.labels,
            milestone: filters.milestone,
            no_milestone: filters.no_milestone,
            assignee: filters.assignee,
            no_assignee: filters.no_assignee,
            project: filters.project,
            review: filters.review,
            checks: filters.checks,
            sort: filters.sort,
        },
        filter_options: PullRequestFilterOptions {
            labels: pull_list_label_options(pool, repository_id).await?,
            users: pull_list_user_options(pool, repository_id).await?,
            milestones: pull_list_milestone_options(pool, repository_id).await?,
            projects: pull_list_project_options().await?,
            review_states: vec![
                "none".to_owned(),
                "required".to_owned(),
                "approved".to_owned(),
                "changes_requested".to_owned(),
                "reviewed_by_me".to_owned(),
                "not_reviewed_by_me".to_owned(),
                "review_requested".to_owned(),
                "team_review_requested".to_owned(),
            ],
            check_states: vec![
                "success".to_owned(),
                "failure".to_owned(),
                "pending".to_owned(),
                "running".to_owned(),
            ],
            sort_options: pull_sort_options(),
        },
        viewer_permission,
        repository: PullRequestListRepository {
            id: repository.id,
            owner_login: repository.owner_login,
            name: repository.name,
            visibility: repository.visibility,
            default_branch: repository.default_branch,
        },
        preferences,
    })
}

pub async fn global_pull_request_list_for_viewer(
    pool: &PgPool,
    actor_user_id: Uuid,
    filters: GlobalPullRequestListQuery,
    page: i64,
    page_size: i64,
) -> Result<GlobalPullRequestListView, CollaborationError> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;
    let text_filter = filters
        .query
        .as_deref()
        .map(search_text_from_pull_query)
        .filter(|value| !value.is_empty());
    let state_filter = filters.state.as_ref().map(PullRequestState::as_str);
    let scope_filter = filters.scope.as_str();

    let counts = GlobalPullRequestCounts {
        created: count_global_pull_request_list_items(
            pool,
            actor_user_id,
            "created",
            state_filter,
            text_filter.as_deref(),
            &filters,
        )
        .await?,
        assigned: count_global_pull_request_list_items(
            pool,
            actor_user_id,
            "assigned",
            state_filter,
            text_filter.as_deref(),
            &filters,
        )
        .await?,
        mentioned: count_global_pull_request_list_items(
            pool,
            actor_user_id,
            "mentioned",
            state_filter,
            text_filter.as_deref(),
            &filters,
        )
        .await?,
        review_requests: count_global_pull_request_list_items(
            pool,
            actor_user_id,
            "review_requests",
            state_filter,
            text_filter.as_deref(),
            &filters,
        )
        .await?,
    };
    let total = match filters.scope {
        GlobalPullRequestScope::Created => counts.created,
        GlobalPullRequestScope::Assigned => counts.assigned,
        GlobalPullRequestScope::Mentioned => counts.mentioned,
        GlobalPullRequestScope::ReviewRequests => counts.review_requests,
    };

    let rows = sqlx::query(
        r#"
        SELECT pull_requests.id, pull_requests.repository_id, pull_requests.issue_id,
               pull_requests.number, pull_requests.title, pull_requests.body,
               pull_requests.state, pull_requests.is_draft, pull_requests.author_user_id,
               pull_requests.head_ref, pull_requests.base_ref,
               pull_requests.head_repository_id, pull_requests.base_repository_id,
               pull_requests.merge_commit_id, pull_requests.merged_by_user_id,
               pull_requests.merged_at, pull_requests.closed_at,
               pull_requests.created_at, pull_requests.updated_at
        FROM pull_requests
        JOIN issues ON issues.id = pull_requests.issue_id
        JOIN repositories ON repositories.id = pull_requests.repository_id
        LEFT JOIN users owner_users ON owner_users.id = repositories.owner_user_id
        LEFT JOIN organizations owner_orgs ON owner_orgs.id = repositories.owner_organization_id
        WHERE (
              repositories.visibility = 'public'
              OR EXISTS (
                  SELECT 1
                  FROM repository_permissions
                  WHERE repository_permissions.repository_id = repositories.id
                    AND repository_permissions.user_id = $1
                    AND repository_permissions.role IN ('owner', 'admin', 'maintain', 'write', 'triage', 'read')
              )
          )
          AND (
              ($2 = 'created' AND pull_requests.author_user_id = $1)
              OR ($2 = 'assigned' AND EXISTS (
                  SELECT 1
                  FROM issue_assignees
                  WHERE issue_assignees.issue_id = pull_requests.issue_id
                    AND issue_assignees.user_id = $1
              ))
              OR ($2 = 'mentioned' AND EXISTS (
                  SELECT 1
                  FROM notifications
                  WHERE notifications.subject_type = 'pull_request'
                    AND notifications.subject_id = pull_requests.id
                    AND notifications.user_id = $1
                    AND notifications.reason IN ('mention', 'team_mention')
              ))
              OR ($2 = 'review_requests' AND EXISTS (
                  SELECT 1
                  FROM pull_request_review_requests
                  WHERE pull_request_review_requests.pull_request_id = pull_requests.id
                    AND pull_request_review_requests.requested_user_id = $1
              ))
          )
          AND ($3::text IS NULL OR pull_requests.state = $3)
          AND (
              $4::text IS NULL
              OR pull_requests.title ILIKE '%' || $4 || '%'
              OR COALESCE(pull_requests.body, '') ILIKE '%' || $4 || '%'
              OR pull_requests.head_ref ILIKE '%' || $4 || '%'
              OR pull_requests.base_ref ILIKE '%' || $4 || '%'
              OR repositories.name ILIKE '%' || $4 || '%'
              OR COALESCE(owner_users.username, owner_users.email, owner_orgs.slug) ILIKE '%' || $4 || '%'
          )
          AND (
              $5::text IS NULL
              OR lower(format('%s/%s', COALESCE(owner_users.username, owner_users.email, owner_orgs.slug), repositories.name)) = lower($5)
              OR lower(repositories.name) = lower($5)
          )
          AND (
              cardinality($6::text[]) = 0
              OR NOT EXISTS (
                  SELECT 1
                  FROM unnest($6::text[]) wanted_label(name)
                  WHERE NOT EXISTS (
                      SELECT 1
                      FROM issue_labels
                      JOIN labels ON labels.id = issue_labels.label_id
                      WHERE issue_labels.issue_id = issues.id
                        AND lower(labels.name) = lower(wanted_label.name)
                  )
              )
          )
          AND (
              $7::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM milestones
                  WHERE milestones.id = issues.milestone_id
                    AND lower(milestones.title) = lower($7)
              )
          )
        ORDER BY
          CASE WHEN $8 = 'best-match' THEN (
              (CASE WHEN pull_requests.title ILIKE $4 || '%' THEN 12 ELSE 0 END)
              + (CASE WHEN pull_requests.title ILIKE '%' || $4 || '%' THEN 8 ELSE 0 END)
              + (CASE WHEN COALESCE(pull_requests.body, '') ILIKE '%' || $4 || '%' THEN 3 ELSE 0 END)
              + (CASE WHEN repositories.name ILIKE '%' || $4 || '%' THEN 2 ELSE 0 END)
          ) END DESC,
          CASE WHEN $8 = 'created-asc' THEN pull_requests.created_at END ASC,
          CASE WHEN $8 = 'created-desc' THEN pull_requests.created_at END DESC,
          CASE WHEN $8 = 'updated-asc' THEN pull_requests.updated_at END ASC,
          CASE WHEN $8 = 'updated-desc' THEN pull_requests.updated_at END DESC,
          CASE WHEN $8 = 'comments-desc' THEN (
              SELECT count(*) FROM comments WHERE comments.pull_request_id = pull_requests.id
          ) END DESC,
          CASE WHEN $8 = 'comments-asc' THEN (
              SELECT count(*) FROM comments WHERE comments.pull_request_id = pull_requests.id
          ) END ASC,
          pull_requests.updated_at DESC,
          pull_requests.number DESC
        LIMIT $9 OFFSET $10
        "#,
    )
    .bind(actor_user_id)
    .bind(scope_filter)
    .bind(state_filter)
    .bind(text_filter.as_deref())
    .bind(filters.repository.as_deref())
    .bind(&filters.labels)
    .bind(filters.milestone.as_deref())
    .bind(&filters.sort)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let pull_requests = rows
        .into_iter()
        .map(pull_request_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    let items = global_pull_request_list_items(pool, pull_requests).await?;

    Ok(GlobalPullRequestListView {
        items,
        total,
        page,
        page_size,
        counts,
        filters: GlobalPullRequestFilters {
            scope: filters.scope,
            query: filters.query.unwrap_or_else(|| "is:pr is:open".to_owned()),
            state: filters.state,
            repository: filters.repository,
            labels: filters.labels,
            milestone: filters.milestone,
            sort: filters.sort,
        },
        filter_options: GlobalPullRequestFilterOptions {
            repositories: global_pull_repository_options(pool, actor_user_id, scope_filter).await?,
            labels: global_pull_label_options(pool, actor_user_id, scope_filter).await?,
            milestones: global_pull_milestone_options(pool, actor_user_id, scope_filter).await?,
            sort_options: pull_sort_options(),
        },
    })
}

pub async fn get_pull_request(
    pool: &PgPool,
    repository_id: Uuid,
    number: i64,
    actor_user_id: Uuid,
) -> Result<PullRequestDetail, CollaborationError> {
    require_repository_read(pool, repository_id, actor_user_id).await?;
    let row = sqlx::query(
        r#"
        SELECT id, repository_id, issue_id, number, title, body, state, author_user_id,
               head_ref, base_ref, head_repository_id, base_repository_id, merge_commit_id,
               merged_by_user_id, merged_at, closed_at, created_at, updated_at
        FROM pull_requests
        WHERE repository_id = $1 AND number = $2
        "#,
    )
    .bind(repository_id)
    .bind(number)
    .fetch_optional(pool)
    .await?
    .ok_or(CollaborationError::PullRequestNotFound)?;
    let pull_request = pull_request_from_row(row)?;
    let issue_row = sqlx::query(
        r#"
        SELECT id, repository_id, number, title, body, state, author_user_id, milestone_id,
               locked, closed_by_user_id, closed_at, created_at, updated_at
        FROM issues
        WHERE id = $1
        "#,
    )
    .bind(pull_request.issue_id)
    .fetch_one(pool)
    .await?;
    let issue = issue_from_row(issue_row)?;

    Ok(PullRequestDetail {
        href: pull_request_href_by_id(pool, pull_request.repository_id, pull_request.number)
            .await?,
        pull_request,
        issue,
    })
}

pub async fn pull_request_detail_view_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    number: i64,
    actor_user_id: Option<Uuid>,
) -> Result<PullRequestDetailView, CollaborationError> {
    let repository = get_repository(pool, repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    let viewer_permission = match actor_user_id {
        Some(user_id) => {
            repository_viewer_permission(pool, &repository, user_id, RepositoryRole::Read).await?
        }
        None if repository.visibility == RepositoryVisibility::Public => Some("read".to_owned()),
        None => return Err(CollaborationError::RepositoryAccessDenied),
    };

    let row = sqlx::query(
        r#"
        SELECT id, repository_id, issue_id, number, title, body, state, author_user_id,
               head_ref, base_ref, head_repository_id, base_repository_id, merge_commit_id,
               merged_by_user_id, merged_at, closed_at, created_at, updated_at, is_draft
        FROM pull_requests
        WHERE repository_id = $1 AND number = $2
        "#,
    )
    .bind(repository_id)
    .bind(number)
    .fetch_optional(pool)
    .await?
    .ok_or(CollaborationError::PullRequestNotFound)?;
    let pull_request = pull_request_from_row(row)?;
    let pull_ids = vec![pull_request.id];
    let issue_ids = vec![pull_request.issue_id];
    let authors = pull_list_authors(pool, &pull_ids).await?;
    let labels = pull_list_labels(pool, &issue_ids).await?;
    let milestones = pull_list_milestones(pool, &issue_ids).await?;
    let assignees = pull_list_assignees(pool, &issue_ids).await?;
    let linked_issues = linked_issue_hints(pool, &issue_ids, &repository).await?;
    let reviews = pull_review_summaries(pool, &pull_ids).await?;
    let checks = pull_check_summaries(pool, &pull_ids).await?;
    let tasks = pull_task_progress(pool, &pull_ids).await?;
    let roles = pull_author_roles(pool, repository_id, &pull_ids).await?;
    let comments = pull_comment_counts(pool, &pull_ids).await?;
    let stats = pull_request_detail_stats(
        pool,
        pull_request.id,
        *comments.get(&pull_request.id).unwrap_or(&0),
    )
    .await?;
    let participants = pull_request_detail_participants(pool, pull_request.id).await?;
    let latest_reviews = pull_request_latest_reviews(pool, pull_request.id).await?;
    let rendered = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            markdown: pull_request.body.clone().unwrap_or_default(),
            repository_id: Some(repository_id),
            owner: Some(repository.owner_login.clone()),
            repo: Some(repository.name.clone()),
            ref_name: Some(pull_request.base_ref.clone()),
            enable_task_toggles: Some(false),
        },
    )
    .await
    .map_err(|error| match error {
        super::markdown::MarkdownError::Sqlx(error) => CollaborationError::Sqlx(error),
        super::markdown::MarkdownError::TooLarge | super::markdown::MarkdownError::TaskNotFound => {
            CollaborationError::InvalidIssueField {
                field_key: "body".to_owned(),
                message: "pull request body could not be rendered".to_owned(),
            }
        }
    })?;
    let href = format!(
        "/{}/{}/pull/{}",
        repository.owner_login, repository.name, pull_request.number
    );
    let subscription = pull_request_subscription_state(pool, &pull_request, actor_user_id).await?;
    let mergeability = pull_request_mergeability(pool, &pull_request, actor_user_id).await?;

    Ok(PullRequestDetailView {
        id: pull_request.id,
        issue_id: pull_request.issue_id,
        repository: PullRequestDetailRepository {
            id: repository.id,
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            visibility: repository.visibility,
            default_branch: repository.default_branch.clone(),
        },
        number: pull_request.number,
        title: pull_request.title,
        body: pull_request.body,
        body_html: rendered.html,
        state: pull_request.state,
        is_draft: pull_request.is_draft,
        author: authors
            .get(&pull_request.id)
            .cloned()
            .unwrap_or_else(|| fallback_user(pull_request.author_user_id)),
        author_role: roles
            .get(&pull_request.id)
            .cloned()
            .unwrap_or_else(|| "contributor".to_owned()),
        head_ref: pull_request.head_ref,
        base_ref: pull_request.base_ref,
        labels: labels
            .get(&pull_request.issue_id)
            .cloned()
            .unwrap_or_default(),
        milestone: milestones.get(&pull_request.issue_id).cloned(),
        assignees: assignees
            .get(&pull_request.issue_id)
            .cloned()
            .unwrap_or_default(),
        requested_reviewers: reviews
            .get(&pull_request.id)
            .map(|review| review.requested_reviewers.clone())
            .unwrap_or_default(),
        latest_reviews,
        linked_issues: linked_issues
            .get(&pull_request.issue_id)
            .cloned()
            .unwrap_or_default(),
        participants,
        review: reviews
            .get(&pull_request.id)
            .cloned()
            .unwrap_or_else(default_review_summary),
        checks: checks
            .get(&pull_request.id)
            .cloned()
            .unwrap_or_else(default_checks_summary),
        task_progress: tasks
            .get(&pull_request.id)
            .cloned()
            .unwrap_or(PullRequestTaskProgress {
                completed: 0,
                total: 0,
            }),
        stats,
        subscription,
        mergeability,
        metadata_options: PullRequestDetailMetadataOptions {
            labels: pull_list_label_options(pool, repository_id).await?,
            assignees: pull_list_user_options(pool, repository_id).await?,
            milestones: pull_list_milestone_options(pool, repository_id).await?,
        },
        commits_href: format!("{href}/commits"),
        checks_href: format!("{href}/checks"),
        files_href: format!("{href}/files"),
        href,
        created_at: pull_request.created_at,
        updated_at: pull_request.updated_at,
        closed_at: pull_request.closed_at,
        merged_at: pull_request.merged_at,
        viewer_permission,
    })
}

pub async fn pull_request_diff_review_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    number: i64,
    actor_user_id: Option<Uuid>,
    query: PullRequestDiffReviewQuery,
) -> Result<PullRequestDiffReviewView, CollaborationError> {
    let view = normalize_diff_view(&query.view)?;
    let whitespace = normalize_diff_whitespace(&query.whitespace)?;
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, 100);
    let filter = query
        .filter
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(160).collect::<String>());
    let commit = query
        .commit
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(120).collect::<String>());

    let pull_request =
        pull_request_detail_view_for_viewer(pool, repository_id, number, actor_user_id).await?;
    let file_filter = filter
        .as_deref()
        .map(|value| format!("%{}%", value.to_lowercase()));
    let offset = (page - 1) * page_size;
    let total_files = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM pull_request_files
        WHERE pull_request_id = $1
          AND ($2::text IS NULL OR lower(path) LIKE $2)
        "#,
    )
    .bind(pull_request.id)
    .bind(file_filter.as_deref())
    .fetch_one(pool)
    .await?;
    let file_rows = sqlx::query(
        r#"
        SELECT files.id, files.path, files.status, files.additions, files.deletions,
               files.byte_size, files.blob_oid,
               viewed.viewed, viewed.viewed_at, viewed.version_key AS viewed_version_key
        FROM pull_request_files files
        LEFT JOIN pull_request_viewed_files viewed
          ON viewed.pull_request_file_id = files.id
         AND viewed.user_id = $5
        WHERE files.pull_request_id = $1
          AND ($2::text IS NULL OR lower(files.path) LIKE $2)
        ORDER BY lower(files.path)
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(pull_request.id)
    .bind(file_filter.as_deref())
    .bind(page_size)
    .bind(offset)
    .bind(actor_user_id)
    .fetch_all(pool)
    .await?;
    let file_ids = file_rows
        .iter()
        .map(|row| row.get::<Uuid, _>("id"))
        .collect::<Vec<_>>();
    let hunk_rows = if file_ids.is_empty() {
        Vec::new()
    } else {
        sqlx::query(
            r#"
            SELECT id, pull_request_file_id, old_start, old_lines, new_start, new_lines, header
            FROM pull_request_file_hunks
            WHERE pull_request_file_id = ANY($1)
            ORDER BY pull_request_file_id, display_order, old_start, new_start
            "#,
        )
        .bind(&file_ids)
        .fetch_all(pool)
        .await?
    };
    let hunk_ids = hunk_rows
        .iter()
        .map(|row| row.get::<Uuid, _>("id"))
        .collect::<Vec<_>>();
    let line_rows = if hunk_ids.is_empty() {
        Vec::new()
    } else {
        sqlx::query(
            r#"
            SELECT lines.hunk_id, lines.kind, lines.old_line, lines.new_line, lines.content,
                   lines.position,
                   COALESCE((
                       SELECT count(*)
                       FROM pull_request_review_comments comments
                       WHERE comments.pull_request_file_id = hunks.pull_request_file_id
                         AND (
                            comments.state = 'published'
                            OR (comments.state = 'pending' AND comments.author_user_id = $2)
                         )
                         AND (
                            (lines.new_line IS NOT NULL AND comments.new_line = lines.new_line)
                            OR (lines.old_line IS NOT NULL AND comments.old_line = lines.old_line)
                         )
                   ), 0)::bigint AS comment_count
            FROM pull_request_hunk_lines lines
            JOIN pull_request_file_hunks hunks ON hunks.id = lines.hunk_id
            WHERE lines.hunk_id = ANY($1)
            ORDER BY lines.hunk_id, lines.position
            "#,
        )
        .bind(&hunk_ids)
        .bind(actor_user_id)
        .fetch_all(pool)
        .await?
    };
    let comment_rows = if file_ids.is_empty() {
        Vec::new()
    } else {
        sqlx::query(
            r#"
            SELECT comments.id, comments.pull_request_file_id, comments.author_user_id,
                   comments.body, comments.body_html, comments.path, comments.side,
                   comments.old_line, comments.new_line, comments.position, comments.state,
                   comments.created_at, comments.updated_at,
                   COALESCE(users.username, users.email) AS login,
                   users.display_name, users.avatar_url
            FROM pull_request_review_comments comments
            JOIN users ON users.id = comments.author_user_id
            WHERE comments.pull_request_id = $1
              AND comments.pull_request_file_id = ANY($2)
              AND (
                comments.state = 'published'
                OR (comments.state = 'pending' AND comments.author_user_id = $3)
              )
            ORDER BY comments.path, comments.created_at
            "#,
        )
        .bind(pull_request.id)
        .bind(&file_ids)
        .bind(actor_user_id)
        .fetch_all(pool)
        .await?
    };

    let mut lines_by_hunk: HashMap<Uuid, Vec<PullRequestDiffLine>> = HashMap::new();
    for row in line_rows {
        let kind = match row.get::<String, _>("kind").as_str() {
            "added" => PullRequestDiffLineKind::Added,
            "removed" => PullRequestDiffLineKind::Removed,
            _ => PullRequestDiffLineKind::Context,
        };
        lines_by_hunk
            .entry(row.get("hunk_id"))
            .or_default()
            .push(PullRequestDiffLine {
                kind,
                old_line: row.get("old_line"),
                new_line: row.get("new_line"),
                content: row.get("content"),
                position: row.get("position"),
                comment_count: row.get("comment_count"),
            });
    }

    let mut hunks_by_file: HashMap<Uuid, Vec<PullRequestDiffHunk>> = HashMap::new();
    for row in hunk_rows {
        let hunk_id = row.get("id");
        hunks_by_file
            .entry(row.get("pull_request_file_id"))
            .or_default()
            .push(PullRequestDiffHunk {
                id: hunk_id,
                header: row.get("header"),
                old_start: row.get("old_start"),
                old_lines: row.get("old_lines"),
                new_start: row.get("new_start"),
                new_lines: row.get("new_lines"),
                lines: lines_by_hunk.remove(&hunk_id).unwrap_or_default(),
            });
    }

    let mut comments_by_file: HashMap<Uuid, Vec<PullRequestDiffReviewComment>> = HashMap::new();
    for row in comment_rows {
        comments_by_file
            .entry(row.get("pull_request_file_id"))
            .or_default()
            .push(PullRequestDiffReviewComment {
                id: row.get("id"),
                author: IssueListUser {
                    id: row.get("author_user_id"),
                    login: row.get("login"),
                    display_name: row.get("display_name"),
                    avatar_url: row.get("avatar_url"),
                },
                body: row.get("body"),
                body_html: row.get("body_html"),
                path: row.get("path"),
                side: row.get("side"),
                old_line: row.get("old_line"),
                new_line: row.get("new_line"),
                position: row.get("position"),
                state: row.get("state"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
    }

    let mut file_tree = Vec::new();
    let mut files = Vec::new();
    for row in file_rows {
        let file_id = row.get("id");
        let path: String = row.get("path");
        let additions = row.get("additions");
        let deletions = row.get("deletions");
        let blob_oid: Option<String> = row.get("blob_oid");
        let version_key = pull_request_file_version_key(blob_oid.as_deref(), additions, deletions);
        let viewed = actor_user_id.is_some()
            && row
                .get::<Option<String>, _>("viewed_version_key")
                .as_deref()
                .is_some_and(|stored| stored == version_key)
            && row.get::<Option<bool>, _>("viewed").unwrap_or(false);
        let href = format!(
            "{}#diff-{}",
            pull_request.files_href,
            diff_anchor_for_path(&path)
        );
        let tree_item = PullRequestDiffFileTreeItem {
            id: file_id,
            path: path.clone(),
            status: row.get("status"),
            additions,
            deletions,
            viewed,
            version_key: version_key.clone(),
            href: href.clone(),
        };
        file_tree.push(tree_item.clone());
        files.push(PullRequestDiffFile {
            id: file_id,
            path: path.clone(),
            status: tree_item.status,
            additions,
            deletions,
            byte_size: row.get("byte_size"),
            blob_oid,
            language: language_for_path(&path),
            viewed,
            viewed_at: row.get("viewed_at"),
            version_key,
            href,
            hunks: hunks_by_file.remove(&file_id).unwrap_or_default(),
            comments: comments_by_file.remove(&file_id).unwrap_or_default(),
        });
    }

    let commits = pull_request_diff_commits(pool, &pull_request).await?;
    let pending_review = pull_request_pending_review(pool, pull_request.id, actor_user_id).await?;

    Ok(PullRequestDiffReviewView {
        pull_request,
        settings: PullRequestDiffReviewSettings {
            view,
            whitespace,
            commit,
            filter,
            page,
            page_size,
        },
        total_files,
        page,
        page_size,
        has_more: offset + page_size < total_files,
        file_tree,
        files,
        commits,
        pending_review,
    })
}

pub async fn pull_request_plain_diff_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    number: i64,
    actor_user_id: Option<Uuid>,
) -> Result<String, CollaborationError> {
    let review = pull_request_diff_review_for_viewer(
        pool,
        repository_id,
        number,
        actor_user_id,
        PullRequestDiffReviewQuery {
            view: "unified".to_owned(),
            whitespace: "show".to_owned(),
            commit: None,
            filter: None,
            page: 1,
            page_size: 100,
        },
    )
    .await?;
    Ok(render_pull_request_diff(&review))
}

pub async fn pull_request_patch_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    number: i64,
    actor_user_id: Option<Uuid>,
) -> Result<String, CollaborationError> {
    let review = pull_request_diff_review_for_viewer(
        pool,
        repository_id,
        number,
        actor_user_id,
        PullRequestDiffReviewQuery {
            view: "unified".to_owned(),
            whitespace: "show".to_owned(),
            commit: None,
            filter: None,
            page: 1,
            page_size: 100,
        },
    )
    .await?;
    Ok(render_pull_request_patch(&review))
}

pub async fn update_pull_request_viewed_file(
    pool: &PgPool,
    pull_request_id: Uuid,
    actor_user_id: Uuid,
    file_id: Uuid,
    version_key: String,
    viewed: bool,
) -> Result<PullRequestViewedFileState, CollaborationError> {
    let version_key = version_key.trim().to_owned();
    if version_key.is_empty() {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "versionKey".to_owned(),
            message: "file version key is required".to_owned(),
        });
    }

    let row = sqlx::query(
        r#"
        SELECT files.id, files.path, files.additions, files.deletions, files.blob_oid,
               pulls.repository_id
        FROM pull_request_files files
        JOIN pull_requests pulls ON pulls.id = files.pull_request_id
        WHERE files.id = $1
          AND files.pull_request_id = $2
        "#,
    )
    .bind(file_id)
    .bind(pull_request_id)
    .fetch_optional(pool)
    .await?
    .ok_or(CollaborationError::InvalidIssueField {
        field_key: "fileId".to_owned(),
        message: "changed file was not found for this pull request".to_owned(),
    })?;

    let repository_id = row.get("repository_id");
    require_repository_read(pool, repository_id, actor_user_id).await?;

    let expected_version = pull_request_file_version_key(
        row.get::<Option<String>, _>("blob_oid").as_deref(),
        row.get("additions"),
        row.get("deletions"),
    );
    if version_key != expected_version {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "versionKey".to_owned(),
            message: "file changed since this viewed state was loaded".to_owned(),
        });
    }

    let updated = sqlx::query(
        r#"
        INSERT INTO pull_request_viewed_files (
            pull_request_file_id,
            user_id,
            version_key,
            viewed,
            viewed_at
        )
        VALUES ($1, $2, $3, $4, now())
        ON CONFLICT (pull_request_file_id, user_id) DO UPDATE
        SET version_key = EXCLUDED.version_key,
            viewed = EXCLUDED.viewed,
            viewed_at = now()
        RETURNING viewed, viewed_at, version_key
        "#,
    )
    .bind(file_id)
    .bind(actor_user_id)
    .bind(&version_key)
    .bind(viewed)
    .fetch_one(pool)
    .await?;

    Ok(PullRequestViewedFileState {
        file_id,
        path: row.get("path"),
        viewed: updated.get("viewed"),
        viewed_at: updated.get("viewed_at"),
        version_key: updated.get("version_key"),
    })
}

pub async fn create_pull_request_review_draft_comment(
    pool: &PgPool,
    input: CreatePullRequestReviewDraftComment,
) -> Result<PullRequestDiffReviewComment, CollaborationError> {
    let body = input.body.trim().to_owned();
    if body.is_empty() {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "body".to_owned(),
            message: "review comment body is required".to_owned(),
        });
    }
    let side = normalize_review_comment_side(&input.side)?;
    let file = validate_pull_request_diff_line(
        pool,
        input.pull_request_id,
        input.file_id,
        &side,
        input.old_line,
        input.new_line,
        input.position,
    )
    .await?;
    require_repository_read(pool, file.repository_id, input.actor_user_id).await?;
    let rendered =
        render_pull_request_review_comment_markdown(pool, file.repository_id, &body).await?;

    sqlx::query(
        r#"
        INSERT INTO pull_request_review_drafts (pull_request_id, author_user_id)
        VALUES ($1, $2)
        ON CONFLICT (pull_request_id, author_user_id) DO UPDATE
        SET updated_at = now()
        "#,
    )
    .bind(input.pull_request_id)
    .bind(input.actor_user_id)
    .execute(pool)
    .await?;

    let row = sqlx::query(
        r#"
        INSERT INTO pull_request_review_comments (
            pull_request_id, pull_request_file_id, author_user_id, body, body_html,
            path, side, old_line, new_line, position, state
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'pending')
        RETURNING id, pull_request_file_id, author_user_id, body, body_html, path, side,
                  old_line, new_line, position, state, created_at, updated_at
        "#,
    )
    .bind(input.pull_request_id)
    .bind(input.file_id)
    .bind(input.actor_user_id)
    .bind(&body)
    .bind(&rendered)
    .bind(&file.path)
    .bind(&side)
    .bind(input.old_line)
    .bind(input.new_line)
    .bind(input.position)
    .fetch_one(pool)
    .await?;

    pull_request_review_comment_from_row(pool, row).await
}

pub async fn update_pull_request_review_draft_comment(
    pool: &PgPool,
    input: UpdatePullRequestReviewDraftComment,
) -> Result<PullRequestDiffReviewComment, CollaborationError> {
    let body = input.body.trim().to_owned();
    if body.is_empty() {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "body".to_owned(),
            message: "review comment body is required".to_owned(),
        });
    }
    let current = sqlx::query(
        r#"
        SELECT comments.id, pulls.repository_id
        FROM pull_request_review_comments comments
        JOIN pull_requests pulls ON pulls.id = comments.pull_request_id
        WHERE comments.id = $1
          AND comments.pull_request_id = $2
          AND comments.author_user_id = $3
          AND comments.state = 'pending'
        "#,
    )
    .bind(input.draft_comment_id)
    .bind(input.pull_request_id)
    .bind(input.actor_user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(CollaborationError::InvalidIssueField {
        field_key: "draftId".to_owned(),
        message: "pending review comment was not found".to_owned(),
    })?;
    let repository_id = current.get("repository_id");
    require_repository_read(pool, repository_id, input.actor_user_id).await?;
    let rendered = render_pull_request_review_comment_markdown(pool, repository_id, &body).await?;
    let row = sqlx::query(
        r#"
        UPDATE pull_request_review_comments
        SET body = $4, body_html = $5, updated_at = now()
        WHERE id = $1
          AND pull_request_id = $2
          AND author_user_id = $3
          AND state = 'pending'
        RETURNING id, pull_request_file_id, author_user_id, body, body_html, path, side,
                  old_line, new_line, position, state, created_at, updated_at
        "#,
    )
    .bind(input.draft_comment_id)
    .bind(input.pull_request_id)
    .bind(input.actor_user_id)
    .bind(&body)
    .bind(&rendered)
    .fetch_one(pool)
    .await?;

    pull_request_review_comment_from_row(pool, row).await
}

pub async fn delete_pull_request_review_draft_comment(
    pool: &PgPool,
    pull_request_id: Uuid,
    actor_user_id: Uuid,
    draft_comment_id: Uuid,
) -> Result<PullRequestDiffPendingReview, CollaborationError> {
    let deleted = sqlx::query_scalar::<_, Option<Uuid>>(
        r#"
        DELETE FROM pull_request_review_comments
        WHERE id = $1
          AND pull_request_id = $2
          AND author_user_id = $3
          AND state = 'pending'
        RETURNING id
        "#,
    )
    .bind(draft_comment_id)
    .bind(pull_request_id)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?;
    if deleted.flatten().is_none() {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "draftId".to_owned(),
            message: "pending review comment was not found".to_owned(),
        });
    }
    let remaining = pull_request_pending_review(pool, pull_request_id, Some(actor_user_id)).await?;
    if remaining.comment_count == 0 && remaining.summary_body.is_none() {
        sqlx::query(
            r#"
            DELETE FROM pull_request_review_drafts
            WHERE pull_request_id = $1 AND author_user_id = $2
            "#,
        )
        .bind(pull_request_id)
        .bind(actor_user_id)
        .execute(pool)
        .await?;
        return pull_request_pending_review(pool, pull_request_id, Some(actor_user_id)).await;
    }
    Ok(remaining)
}

pub async fn submit_pull_request_review(
    pool: &PgPool,
    input: SubmitPullRequestReview,
) -> Result<PullRequestSubmittedReview, CollaborationError> {
    let state = normalize_submitted_review_state(&input.state)?;
    let body = input
        .body
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let pull_request = pull_request_by_id(pool, input.pull_request_id).await?;
    require_repository_read(pool, pull_request.repository_id, input.actor_user_id).await?;
    if pull_request.author_user_id == input.actor_user_id
        && matches!(state.as_str(), "approved" | "changes_requested")
    {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "state".to_owned(),
            message: "authors can only leave comment reviews on their own pull requests".to_owned(),
        });
    }

    let pending_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM pull_request_review_comments
        WHERE pull_request_id = $1
          AND author_user_id = $2
          AND state = 'pending'
        "#,
    )
    .bind(input.pull_request_id)
    .bind(input.actor_user_id)
    .fetch_one(pool)
    .await?;
    if pending_count == 0 && body.is_none() {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "body".to_owned(),
            message: "write a review summary or add pending comments before submitting".to_owned(),
        });
    }

    let mut tx = pool.begin().await?;
    let review_row = sqlx::query(
        r#"
        INSERT INTO pull_request_reviews (pull_request_id, reviewer_user_id, state, body)
        VALUES ($1, $2, $3, $4)
        RETURNING id, state, body, submitted_at
        "#,
    )
    .bind(input.pull_request_id)
    .bind(input.actor_user_id)
    .bind(&state)
    .bind(&body)
    .fetch_one(&mut *tx)
    .await?;
    let review_id: Uuid = review_row.get("id");
    let published_comment_count = sqlx::query_scalar::<_, i64>(
        r#"
        WITH published AS (
            UPDATE pull_request_review_comments
            SET state = 'published', review_id = $3, updated_at = now()
            WHERE pull_request_id = $1
              AND author_user_id = $2
              AND state = 'pending'
            RETURNING id
        )
        SELECT count(*)::bigint FROM published
        "#,
    )
    .bind(input.pull_request_id)
    .bind(input.actor_user_id)
    .bind(review_id)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "DELETE FROM pull_request_review_drafts WHERE pull_request_id = $1 AND author_user_id = $2",
    )
    .bind(input.pull_request_id)
    .bind(input.actor_user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO timeline_events (
            repository_id, issue_id, pull_request_id, actor_user_id, event_type, metadata
        )
        VALUES ($1, NULL, $2, $3, 'reviewed', $4)
        "#,
    )
    .bind(pull_request.repository_id)
    .bind(input.pull_request_id)
    .bind(input.actor_user_id)
    .bind(json!({
        "reviewId": review_id,
        "state": state,
        "commentCount": published_comment_count
    }))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    notify_pull_request_review_submitted(pool, &pull_request, input.actor_user_id, &state).await?;
    insert_pull_request_action_audit_event(
        pool,
        &pull_request,
        input.actor_user_id,
        "pull_request.review_submitted",
        json!({
            "reviewId": review_id,
            "state": state,
            "commentCount": published_comment_count
        }),
    )
    .await?;

    Ok(PullRequestSubmittedReview {
        id: review_id,
        reviewer: user_for_review_comment(pool, input.actor_user_id).await?,
        state: review_row.get("state"),
        body: review_row.get("body"),
        submitted_at: review_row.get("submitted_at"),
        published_comment_count,
        pending_review: pull_request_pending_review(
            pool,
            input.pull_request_id,
            Some(input.actor_user_id),
        )
        .await?,
    })
}

pub async fn abandon_pull_request_review_draft(
    pool: &PgPool,
    pull_request_id: Uuid,
    actor_user_id: Uuid,
) -> Result<PullRequestDiffPendingReview, CollaborationError> {
    let pull_request = pull_request_by_id(pool, pull_request_id).await?;
    require_repository_read(pool, pull_request.repository_id, actor_user_id).await?;
    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        DELETE FROM pull_request_review_comments
        WHERE pull_request_id = $1
          AND author_user_id = $2
          AND state = 'pending'
        "#,
    )
    .bind(pull_request_id)
    .bind(actor_user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "DELETE FROM pull_request_review_drafts WHERE pull_request_id = $1 AND author_user_id = $2",
    )
    .bind(pull_request_id)
    .bind(actor_user_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    pull_request_pending_review(pool, pull_request_id, Some(actor_user_id)).await
}

pub async fn update_pull_request_state(
    pool: &PgPool,
    pull_request_id: Uuid,
    input: UpdatePullRequestState,
) -> Result<PullRequest, CollaborationError> {
    let current = pull_request_by_id(pool, pull_request_id).await?;
    require_repository_write(pool, current.repository_id, input.actor_user_id).await?;
    if current.state == PullRequestState::Merged {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "state".to_owned(),
            message: "merged pull requests cannot be reopened or closed".to_owned(),
        });
    }
    if input.state == PullRequestState::Merged {
        let mergeability =
            pull_request_mergeability(pool, &current, Some(input.actor_user_id)).await?;
        if let Some(method) = &input.method {
            if !mergeability.methods.iter().any(|allowed| allowed == method) {
                return Err(CollaborationError::InvalidIssueField {
                    field_key: "method".to_owned(),
                    message: format!(
                        "{} is disabled by this repository's merge settings",
                        method.as_str()
                    ),
                });
            }
        }
        if !mergeability.can_merge {
            return Err(CollaborationError::InvalidIssueField {
                field_key: "merge".to_owned(),
                message: mergeability.summary,
            });
        }
    }
    let issue_state = if input.state == PullRequestState::Open {
        IssueState::Open
    } else {
        IssueState::Closed
    };

    let row = sqlx::query(
        r#"
        UPDATE pull_requests
        SET state = $2,
            merge_commit_id = CASE WHEN $2 = 'merged' THEN $4 ELSE NULL END,
            merged_by_user_id = CASE WHEN $2 = 'merged' THEN $3 ELSE NULL END,
            merged_at = CASE WHEN $2 = 'merged' THEN now() ELSE NULL END,
            closed_at = CASE WHEN $2 IN ('closed', 'merged') THEN now() ELSE NULL END
        WHERE id = $1
        RETURNING id, repository_id, issue_id, number, title, body, state, author_user_id,
                  head_ref, base_ref, head_repository_id, base_repository_id, merge_commit_id,
                  merged_by_user_id, merged_at, closed_at, created_at, updated_at
        "#,
    )
    .bind(pull_request_id)
    .bind(input.state.as_str())
    .bind(input.actor_user_id)
    .bind(input.merge_commit_id)
    .fetch_one(pool)
    .await?;
    let pull_request = pull_request_from_row(row)?;

    sqlx::query(
        r#"
        UPDATE issues
        SET state = $2,
            closed_by_user_id = CASE WHEN $2 = 'closed' THEN $3 ELSE NULL END,
            closed_at = CASE WHEN $2 = 'closed' THEN now() ELSE NULL END
        WHERE id = $1
        "#,
    )
    .bind(pull_request.issue_id)
    .bind(issue_state.as_str())
    .bind(input.actor_user_id)
    .execute(pool)
    .await?;

    let event_type = match input.state {
        PullRequestState::Open => "reopened",
        PullRequestState::Closed => "closed",
        PullRequestState::Merged => "merged",
    };
    append_timeline_event(
        pool,
        current.repository_id,
        None,
        Some(pull_request.id),
        Some(input.actor_user_id),
        event_type,
        json!({ "number": pull_request.number, "state": input.state.as_str() }),
    )
    .await?;
    insert_pull_request_action_audit_event(
        pool,
        &pull_request,
        input.actor_user_id,
        match input.state {
            PullRequestState::Open => "pull_request.reopened",
            PullRequestState::Closed => "pull_request.closed",
            PullRequestState::Merged => "pull_request.merged",
        },
        json!({ "state": input.state.as_str() }),
    )
    .await?;
    run_project_item_automation(
        pool,
        ProjectAutomationInput {
            actor_user_id: input.actor_user_id,
            repository_id: pull_request.repository_id,
            issue_id: None,
            pull_request_id: Some(pull_request.id),
            event: match input.state {
                PullRequestState::Open => ProjectAutomationEvent::IssueReopened,
                PullRequestState::Closed => ProjectAutomationEvent::PullRequestClosed,
                PullRequestState::Merged => ProjectAutomationEvent::PullRequestMerged,
            },
        },
    )
    .await
    .map_err(|error| match error {
        super::projects::ProjectsError::Sqlx(error) => CollaborationError::Sqlx(error),
        _ => CollaborationError::PullRequestNotFound,
    })?;
    index_pull_request_search_document(pool, &pull_request, input.actor_user_id).await?;
    Ok(pull_request)
}

pub async fn merge_pull_request(
    pool: &PgPool,
    pull_request_id: Uuid,
    input: MergePullRequestInput,
) -> Result<PullRequest, MergePullRequestError> {
    let current = pull_request_by_id(pool, pull_request_id).await?;
    require_repository_write(pool, current.repository_id, input.actor_user_id).await?;
    let mergeability = pull_request_mergeability(pool, &current, Some(input.actor_user_id)).await?;
    if !mergeability
        .methods
        .iter()
        .any(|allowed| allowed == &input.method)
    {
        return Err(MergePullRequestError::Blocked {
            summary: format!(
                "{} is disabled by this repository's merge settings",
                input.method.as_str()
            ),
            blockers: vec![merge_blocker(
                "merge_method_disabled",
                &format!(
                    "{} is disabled by this repository's merge settings.",
                    input.method.as_str()
                ),
            )],
        });
    }
    if !mergeability.can_merge {
        return Err(MergePullRequestError::Blocked {
            summary: mergeability.summary,
            blockers: mergeability.blockers,
        });
    }

    let mut tx = pool.begin().await?;
    let locked_row = sqlx::query(
        r#"
        SELECT id, repository_id, issue_id, number, title, body, state, author_user_id,
               head_ref, base_ref, head_repository_id, base_repository_id, merge_commit_id,
               merged_by_user_id, merged_at, closed_at, created_at, updated_at, is_draft
        FROM pull_requests
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(pull_request_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(CollaborationError::PullRequestNotFound)?;
    let locked = pull_request_from_row(locked_row)?;
    if locked.state != PullRequestState::Open {
        return Err(MergePullRequestError::Blocked {
            summary: "This pull request is no longer open.".to_owned(),
            blockers: vec![merge_blocker(
                if locked.state == PullRequestState::Merged {
                    "already_merged"
                } else {
                    "pull_request_closed"
                },
                if locked.state == PullRequestState::Merged {
                    "This pull request has already been merged."
                } else {
                    "Closed pull requests must be reopened before they can merge."
                },
            )],
        });
    }

    let base_ref_name = format!("refs/heads/{}", locked.base_ref);
    let head_ref_name = format!("refs/heads/{}", locked.head_ref);
    let base_ref = sqlx::query(
        r#"
        SELECT id, target_commit_id
        FROM repository_git_refs
        WHERE repository_id = $1 AND name = $2 AND kind = 'branch'
        FOR UPDATE
        "#,
    )
    .bind(locked.base_repository_id.unwrap_or(locked.repository_id))
    .bind(&base_ref_name)
    .fetch_optional(&mut *tx)
    .await?;
    let head_ref = sqlx::query(
        r#"
        SELECT id, target_commit_id
        FROM repository_git_refs
        WHERE repository_id = $1 AND name = $2 AND kind = 'branch'
        "#,
    )
    .bind(locked.head_repository_id.unwrap_or(locked.repository_id))
    .bind(&head_ref_name)
    .fetch_optional(&mut *tx)
    .await?;

    let base_ref_id = base_ref.as_ref().map(|row| row.get::<Uuid, _>("id"));
    let base_commit_id = base_ref
        .as_ref()
        .and_then(|row| row.get::<Option<Uuid>, _>("target_commit_id"));
    let head_commit_id = head_ref
        .as_ref()
        .and_then(|row| row.get::<Option<Uuid>, _>("target_commit_id"));
    let parent_oids = merge_parent_oids(
        &mut tx,
        locked.repository_id,
        base_commit_id,
        head_commit_id,
    )
    .await?;
    let commit_title = input
        .commit_title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| default_merge_commit_title(&input.method, &locked));
    let commit_body = input
        .commit_body
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let message = match commit_body {
        Some(body) => format!("{commit_title}\n\n{body}"),
        None => commit_title,
    };
    let oid = format!("og-{}-{}", input.method.as_str(), Uuid::new_v4().simple());
    let merge_commit_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO commits (
            repository_id, oid, author_user_id, committer_user_id, message, parent_oids
        )
        VALUES ($1, $2, $3, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(locked.repository_id)
    .bind(&oid)
    .bind(input.actor_user_id)
    .bind(&message)
    .bind(&parent_oids)
    .fetch_one(&mut *tx)
    .await?;

    if let Some(base_ref_id) = base_ref_id {
        let updated = sqlx::query(
            r#"
            UPDATE repository_git_refs
            SET target_commit_id = $2
            WHERE id = $1 AND target_commit_id IS NOT DISTINCT FROM $3
            "#,
        )
        .bind(base_ref_id)
        .bind(merge_commit_id)
        .bind(base_commit_id)
        .execute(&mut *tx)
        .await?;
        if updated.rows_affected() != 1 {
            return Err(MergePullRequestError::Blocked {
                summary: "The base branch changed before the merge could complete.".to_owned(),
                blockers: vec![merge_blocker(
                    "stale_base_ref",
                    "The base branch changed before the merge could complete.",
                )],
            });
        }
    } else {
        sqlx::query(
            r#"
            INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
            VALUES ($1, $2, 'branch', $3)
            "#,
        )
        .bind(locked.base_repository_id.unwrap_or(locked.repository_id))
        .bind(&base_ref_name)
        .bind(merge_commit_id)
        .execute(&mut *tx)
        .await?;
    }

    let merged_row = sqlx::query(
        r#"
        UPDATE pull_requests
        SET state = 'merged',
            merge_commit_id = $2,
            merged_by_user_id = $3,
            merged_at = now(),
            closed_at = now()
        WHERE id = $1 AND state = 'open'
        RETURNING id, repository_id, issue_id, number, title, body, state, author_user_id,
                  head_ref, base_ref, head_repository_id, base_repository_id, merge_commit_id,
                  merged_by_user_id, merged_at, closed_at, created_at, updated_at, is_draft
        "#,
    )
    .bind(locked.id)
    .bind(merge_commit_id)
    .bind(input.actor_user_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| MergePullRequestError::Blocked {
        summary: "This pull request is no longer open.".to_owned(),
        blockers: vec![merge_blocker(
            "stale_pull_request_state",
            "This pull request is no longer open.",
        )],
    })?;
    let merged = pull_request_from_row(merged_row)?;

    sqlx::query(
        r#"
        UPDATE issues
        SET state = 'closed',
            closed_by_user_id = $2,
            closed_at = now()
        WHERE id = $1
        "#,
    )
    .bind(merged.issue_id)
    .bind(input.actor_user_id)
    .execute(&mut *tx)
    .await?;

    close_linked_issues_for_merge(&mut tx, &merged, input.actor_user_id).await?;

    if input.delete_branch
        && locked.head_repository_id.unwrap_or(locked.repository_id) == locked.repository_id
        && locked.head_ref != locked.base_ref
    {
        sqlx::query(
            "DELETE FROM repository_git_refs WHERE repository_id = $1 AND name = $2 AND kind = 'branch'",
        )
        .bind(locked.repository_id)
        .bind(&head_ref_name)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO timeline_events (
            repository_id, issue_id, pull_request_id, actor_user_id, event_type, metadata
        )
        VALUES ($1, NULL, $2, $3, 'merged', $4)
        "#,
    )
    .bind(merged.repository_id)
    .bind(merged.id)
    .bind(input.actor_user_id)
    .bind(json!({
        "number": merged.number,
        "state": "merged",
        "method": input.method.as_str(),
        "mergeCommitId": merge_commit_id,
        "deleteBranch": input.delete_branch
    }))
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'pull_request.merged', 'pull_request', $2, $3)
        "#,
    )
    .bind(input.actor_user_id)
    .bind(merged.id.to_string())
    .bind(json!({
        "repositoryId": merged.repository_id,
        "number": merged.number,
        "method": input.method.as_str(),
        "mergeCommitId": merge_commit_id,
        "deleteBranch": input.delete_branch,
    }))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    notify_pull_request_merged(pool, &merged, input.actor_user_id).await?;
    run_project_item_automation(
        pool,
        ProjectAutomationInput {
            actor_user_id: input.actor_user_id,
            repository_id: merged.repository_id,
            issue_id: None,
            pull_request_id: Some(merged.id),
            event: ProjectAutomationEvent::PullRequestMerged,
        },
    )
    .await
    .map_err(|error| match error {
        super::projects::ProjectsError::Sqlx(error) => {
            MergePullRequestError::Collaboration(CollaborationError::Sqlx(error))
        }
        _ => MergePullRequestError::Collaboration(CollaborationError::PullRequestNotFound),
    })?;
    index_pull_request_search_document(pool, &merged, input.actor_user_id).await?;
    Ok(merged)
}

pub async fn update_pull_request_draft_state(
    pool: &PgPool,
    pull_request_id: Uuid,
    input: UpdatePullRequestDraftState,
) -> Result<PullRequest, CollaborationError> {
    let current = pull_request_by_id(pool, pull_request_id).await?;
    require_repository_write(pool, current.repository_id, input.actor_user_id).await?;
    let row = sqlx::query(
        r#"
        UPDATE pull_requests
        SET is_draft = $2
        WHERE id = $1
        RETURNING id, repository_id, issue_id, number, title, body, state, author_user_id,
                  head_ref, base_ref, head_repository_id, base_repository_id, merge_commit_id,
                  merged_by_user_id, merged_at, closed_at, created_at, updated_at, is_draft
        "#,
    )
    .bind(pull_request_id)
    .bind(input.is_draft)
    .fetch_one(pool)
    .await?;
    let pull_request = pull_request_from_row(row)?;
    append_timeline_event(
        pool,
        pull_request.repository_id,
        None,
        Some(pull_request.id),
        Some(input.actor_user_id),
        if input.is_draft {
            "converted_to_draft"
        } else {
            "ready_for_review"
        },
        json!({ "number": pull_request.number, "draft": pull_request.is_draft }),
    )
    .await?;
    insert_pull_request_action_audit_event(
        pool,
        &pull_request,
        input.actor_user_id,
        if input.is_draft {
            "pull_request.converted_to_draft"
        } else {
            "pull_request.ready_for_review"
        },
        json!({ "draft": pull_request.is_draft }),
    )
    .await?;
    index_pull_request_search_document(pool, &pull_request, input.actor_user_id).await?;
    Ok(pull_request)
}

pub async fn update_pull_request_metadata(
    pool: &PgPool,
    pull_request_id: Uuid,
    input: UpdatePullRequestMetadata,
) -> Result<(), CollaborationError> {
    let pull_request = pull_request_by_id(pool, pull_request_id).await?;
    let repository = get_repository(pool, pull_request.repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    require_repository_write(pool, repository.id, input.actor_user_id).await?;

    for label_id in &input.label_ids {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM labels WHERE id = $1 AND repository_id = $2)",
        )
        .bind(label_id)
        .bind(repository.id)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(CollaborationError::InvalidIssueField {
                field_key: "labelIds".to_owned(),
                message: "label is not available for this repository".to_owned(),
            });
        }
    }

    if let Some(milestone_id) = input.milestone_id {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM milestones WHERE id = $1 AND repository_id = $2)",
        )
        .bind(milestone_id)
        .bind(repository.id)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(CollaborationError::InvalidIssueField {
                field_key: "milestoneId".to_owned(),
                message: "milestone is not available for this repository".to_owned(),
            });
        }
    }

    for assignee_user_id in &input.assignee_user_ids {
        if repository_viewer_permission(pool, &repository, *assignee_user_id, RepositoryRole::Read)
            .await?
            .is_none()
        {
            return Err(CollaborationError::InvalidIssueField {
                field_key: "assigneeUserIds".to_owned(),
                message: "assignee is not available for this repository".to_owned(),
            });
        }
    }

    sqlx::query("UPDATE issues SET milestone_id = $2 WHERE id = $1")
        .bind(pull_request.issue_id)
        .bind(input.milestone_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM issue_labels WHERE issue_id = $1")
        .bind(pull_request.issue_id)
        .execute(pool)
        .await?;
    for label_id in &input.label_ids {
        sqlx::query(
            "INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(pull_request.issue_id)
        .bind(label_id)
        .execute(pool)
        .await?;
    }
    sqlx::query("DELETE FROM issue_assignees WHERE issue_id = $1")
        .bind(pull_request.issue_id)
        .execute(pool)
        .await?;
    for assignee_user_id in &input.assignee_user_ids {
        sqlx::query(
            r#"
            INSERT INTO issue_assignees (issue_id, user_id, assigned_by_user_id)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(pull_request.issue_id)
        .bind(assignee_user_id)
        .bind(input.actor_user_id)
        .execute(pool)
        .await?;
    }

    append_timeline_event(
        pool,
        pull_request.repository_id,
        None,
        Some(pull_request.id),
        Some(input.actor_user_id),
        "metadata_changed",
        json!({
            "labelIds": input.label_ids,
            "assigneeUserIds": input.assignee_user_ids,
            "milestoneId": input.milestone_id,
        }),
    )
    .await?;
    notify_pull_request_participants(
        pool,
        &pull_request,
        input.actor_user_id,
        &input.assignee_user_ids,
        &[],
    )
    .await?;
    insert_pull_request_action_audit_event(
        pool,
        &pull_request,
        input.actor_user_id,
        "pull_request.metadata_updated",
        json!({
            "labelIds": input.label_ids,
            "assigneeUserIds": input.assignee_user_ids,
            "milestoneId": input.milestone_id,
        }),
    )
    .await?;
    index_pull_request_search_document(pool, &pull_request, input.actor_user_id).await?;
    Ok(())
}

pub async fn update_pull_request_review_requests(
    pool: &PgPool,
    pull_request_id: Uuid,
    input: UpdatePullRequestReviewRequests,
) -> Result<(), CollaborationError> {
    let pull_request = pull_request_by_id(pool, pull_request_id).await?;
    let repository = get_repository(pool, pull_request.repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    require_repository_write(pool, repository.id, input.actor_user_id).await?;

    let mut reviewer_ids = input.reviewer_user_ids;
    reviewer_ids.sort();
    reviewer_ids.dedup();
    reviewer_ids.retain(|id| *id != input.actor_user_id);
    for reviewer_user_id in &reviewer_ids {
        if repository_viewer_permission(pool, &repository, *reviewer_user_id, RepositoryRole::Read)
            .await?
            .is_none()
        {
            return Err(CollaborationError::InvalidIssueField {
                field_key: "reviewerUserIds".to_owned(),
                message: "reviewer is not available for this repository".to_owned(),
            });
        }
    }

    let previous_rows = sqlx::query(
        "SELECT requested_user_id FROM pull_request_review_requests WHERE pull_request_id = $1",
    )
    .bind(pull_request.id)
    .fetch_all(pool)
    .await?;
    let previous: HashSet<Uuid> = previous_rows
        .into_iter()
        .map(|row| row.get("requested_user_id"))
        .collect();
    let requested: HashSet<Uuid> = reviewer_ids.iter().copied().collect();

    sqlx::query("DELETE FROM pull_request_review_requests WHERE pull_request_id = $1 AND NOT (requested_user_id = ANY($2))")
        .bind(pull_request.id)
        .bind(&reviewer_ids)
        .execute(pool)
        .await?;
    insert_review_requests(pool, &pull_request, input.actor_user_id, &reviewer_ids).await?;

    let added: Vec<Uuid> = requested.difference(&previous).copied().collect();
    let removed: Vec<Uuid> = previous.difference(&requested).copied().collect();
    if !added.is_empty() || !removed.is_empty() {
        append_timeline_event(
            pool,
            pull_request.repository_id,
            None,
            Some(pull_request.id),
            Some(input.actor_user_id),
            "review_requested",
            json!({ "addedReviewerUserIds": added, "removedReviewerUserIds": removed }),
        )
        .await?;
        notify_pull_request_participants(pool, &pull_request, input.actor_user_id, &[], &added)
            .await?;
        insert_pull_request_action_audit_event(
            pool,
            &pull_request,
            input.actor_user_id,
            "pull_request.review_requests_updated",
            json!({ "addedReviewerUserIds": added, "removedReviewerUserIds": removed }),
        )
        .await?;
    }
    Ok(())
}

pub async fn update_pull_request_subscription(
    pool: &PgPool,
    pull_request_id: Uuid,
    input: UpdatePullRequestSubscription,
) -> Result<PullRequestSubscriptionState, CollaborationError> {
    let pull_request = pull_request_by_id(pool, pull_request_id).await?;
    require_role(
        pool,
        pull_request.repository_id,
        input.actor_user_id,
        RepositoryRole::Read,
    )
    .await?;
    let custom_events = super::issues::normalize_thread_subscription_events(&input.custom_events)?;
    sqlx::query(
        r#"
        INSERT INTO pull_request_subscriptions (
            pull_request_id, user_id, subscribed, reason, custom_events
        )
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (pull_request_id, user_id)
        DO UPDATE SET
            subscribed = EXCLUDED.subscribed,
            reason = EXCLUDED.reason,
            custom_events = EXCLUDED.custom_events
        "#,
    )
    .bind(pull_request.id)
    .bind(input.actor_user_id)
    .bind(input.subscribed)
    .bind(if input.subscribed {
        "subscribed"
    } else {
        "ignored"
    })
    .bind(&custom_events)
    .execute(pool)
    .await?;
    pull_request_subscription_state(pool, &pull_request, Some(input.actor_user_id)).await
}

pub async fn add_pull_request_comment(
    pool: &PgPool,
    pull_request_id: Uuid,
    input: CreateComment,
) -> Result<super::issues::Comment, CollaborationError> {
    let repository_id =
        sqlx::query_scalar::<_, Uuid>("SELECT repository_id FROM pull_requests WHERE id = $1")
            .bind(pull_request_id)
            .fetch_optional(pool)
            .await?
            .ok_or(CollaborationError::PullRequestNotFound)?;
    require_repository_write(pool, repository_id, input.actor_user_id).await?;

    let row = sqlx::query(
        r#"
        INSERT INTO comments (repository_id, pull_request_id, author_user_id, body)
        VALUES ($1, $2, $3, $4)
        RETURNING id, repository_id, issue_id, pull_request_id, author_user_id, body,
                  is_minimized, created_at, updated_at
        "#,
    )
    .bind(repository_id)
    .bind(pull_request_id)
    .bind(input.actor_user_id)
    .bind(&input.body)
    .fetch_one(pool)
    .await?;
    let comment = super::issues::comment_from_row(row);
    append_timeline_event(
        pool,
        repository_id,
        None,
        Some(pull_request_id),
        Some(input.actor_user_id),
        "commented",
        json!({ "commentId": comment.id }),
    )
    .await?;
    Ok(comment)
}

pub async fn pull_request_comment_timeline_item(
    pool: &PgPool,
    comment_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<PullRequestTimelineItem, CollaborationError> {
    let row = sqlx::query(
        r#"
        SELECT timeline_events.id, timeline_events.event_type, timeline_events.metadata,
               timeline_events.created_at,
               comments.id AS comment_id, comments.body, comments.is_minimized,
               comments.created_at AS comment_created_at, comments.updated_at AS comment_updated_at,
               users.id AS actor_id, COALESCE(users.username, users.email) AS actor_login,
               users.display_name AS actor_display_name, users.avatar_url AS actor_avatar_url,
               repositories.id AS repository_id,
               COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug) AS owner_login,
               repositories.name,
               pull_requests.base_ref
        FROM comments
        JOIN pull_requests ON pull_requests.id = comments.pull_request_id
        JOIN repositories ON repositories.id = pull_requests.repository_id
        LEFT JOIN users AS owner_user ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        JOIN timeline_events
          ON timeline_events.pull_request_id = comments.pull_request_id
         AND timeline_events.event_type = 'commented'
         AND timeline_events.metadata->>'commentId' = comments.id::text
        JOIN users ON users.id = comments.author_user_id
        WHERE comments.id = $1
        ORDER BY timeline_events.created_at DESC, timeline_events.id DESC
        LIMIT 1
        "#,
    )
    .bind(comment_id)
    .fetch_one(pool)
    .await?;

    pull_timeline_item_from_row(pool, row, viewer_user_id).await
}

pub async fn pull_request_timeline(
    pool: &PgPool,
    pull_request_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<Vec<TimelineEvent>, CollaborationError> {
    let repository_id =
        sqlx::query_scalar::<_, Uuid>("SELECT repository_id FROM pull_requests WHERE id = $1")
            .bind(pull_request_id)
            .fetch_optional(pool)
            .await?
            .ok_or(CollaborationError::PullRequestNotFound)?;
    require_repository_read_for_viewer(pool, repository_id, actor_user_id).await?;
    let rows = sqlx::query(
        r#"
        SELECT id, repository_id, issue_id, pull_request_id, actor_user_id, event_type, metadata, created_at
        FROM timeline_events
        WHERE pull_request_id = $1
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(pull_request_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(super::issues::timeline_event_from_row)
        .collect())
}

pub async fn pull_request_timeline_view(
    pool: &PgPool,
    pull_request_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<Vec<PullRequestTimelineItem>, CollaborationError> {
    let repository_id =
        sqlx::query_scalar::<_, Uuid>("SELECT repository_id FROM pull_requests WHERE id = $1")
            .bind(pull_request_id)
            .fetch_optional(pool)
            .await?
            .ok_or(CollaborationError::PullRequestNotFound)?;
    require_repository_read_for_viewer(pool, repository_id, actor_user_id).await?;
    let rows = sqlx::query(
        r#"
        SELECT timeline_events.id, timeline_events.event_type, timeline_events.metadata,
               timeline_events.created_at,
               comments.id AS comment_id, comments.body, comments.is_minimized,
               comments.created_at AS comment_created_at, comments.updated_at AS comment_updated_at,
               users.id AS actor_id, COALESCE(users.username, users.email) AS actor_login,
               users.display_name AS actor_display_name, users.avatar_url AS actor_avatar_url,
               repositories.id AS repository_id,
               COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug) AS owner_login,
               repositories.name,
               pull_requests.base_ref
        FROM timeline_events
        JOIN pull_requests ON pull_requests.id = timeline_events.pull_request_id
        JOIN repositories ON repositories.id = pull_requests.repository_id
        LEFT JOIN users AS owner_user ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        LEFT JOIN comments
          ON timeline_events.event_type = 'commented'
         AND timeline_events.metadata->>'commentId' = comments.id::text
        LEFT JOIN users
          ON users.id = COALESCE(comments.author_user_id, timeline_events.actor_user_id)
        WHERE timeline_events.pull_request_id = $1
        ORDER BY timeline_events.created_at ASC, timeline_events.id ASC
        "#,
    )
    .bind(pull_request_id)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(pull_timeline_item_from_row(pool, row, actor_user_id).await?);
    }
    Ok(items)
}

async fn pull_timeline_item_from_row(
    pool: &PgPool,
    row: sqlx::postgres::PgRow,
    viewer_user_id: Option<Uuid>,
) -> Result<PullRequestTimelineItem, CollaborationError> {
    let actor_id: Option<Uuid> = row.get("actor_id");
    let comment_id: Option<Uuid> = row.get("comment_id");
    let body: Option<String> = row.get("body");
    let comment = match (comment_id, body) {
        (Some(comment_id), Some(body)) => {
            let repository_id: Uuid = row.get("repository_id");
            let owner: String = row.get("owner_login");
            let repo: String = row.get("name");
            let ref_name: String = row.get("base_ref");
            let rendered = render_markdown(
                Some(pool),
                RenderMarkdownInput {
                    markdown: body.clone(),
                    repository_id: Some(repository_id),
                    owner: Some(owner),
                    repo: Some(repo),
                    ref_name: Some(ref_name),
                    enable_task_toggles: Some(false),
                },
            )
            .await
            .map_err(|error| match error {
                super::markdown::MarkdownError::Sqlx(error) => CollaborationError::Sqlx(error),
                super::markdown::MarkdownError::TooLarge
                | super::markdown::MarkdownError::TaskNotFound => {
                    CollaborationError::InvalidIssueField {
                        field_key: "comment".to_owned(),
                        message: "pull request comment could not be rendered".to_owned(),
                    }
                }
            })?;
            let reactions =
                reaction_summaries(pool, None, Some(comment_id), viewer_user_id).await?;
            Some(PullRequestTimelineComment {
                id: comment_id,
                body,
                body_html: rendered.html,
                is_minimized: row.get("is_minimized"),
                reactions,
                created_at: row.get("comment_created_at"),
                updated_at: row.get("comment_updated_at"),
            })
        }
        _ => None,
    };

    Ok(PullRequestTimelineItem {
        id: row.get("id"),
        event_type: row.get("event_type"),
        actor: actor_id.map(|id| IssueListUser {
            id,
            login: row.get("actor_login"),
            display_name: row.get("actor_display_name"),
            avatar_url: row.get("actor_avatar_url"),
        }),
        comment,
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
    })
}

pub async fn repository_for_actor_by_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    actor_user_id: Uuid,
    required_role: RepositoryRole,
) -> Result<Uuid, CollaborationError> {
    Ok(
        repository_for_actor(pool, owner_login, repo_name, actor_user_id, required_role)
            .await?
            .id,
    )
}

async fn require_repository_read(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
) -> Result<(), CollaborationError> {
    require_role(pool, repository_id, user_id, RepositoryRole::Read).await
}

async fn require_repository_read_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Option<Uuid>,
) -> Result<(), CollaborationError> {
    match user_id {
        Some(user_id) => require_repository_read(pool, repository_id, user_id).await,
        None => {
            let repository = get_repository(pool, repository_id)
                .await
                .map_err(|error| match error {
                    super::repositories::RepositoryError::Sqlx(error) => {
                        CollaborationError::Sqlx(error)
                    }
                    _ => CollaborationError::RepositoryNotFound,
                })?
                .ok_or(CollaborationError::RepositoryNotFound)?;
            if repository.visibility == RepositoryVisibility::Public {
                Ok(())
            } else {
                Err(CollaborationError::RepositoryAccessDenied)
            }
        }
    }
}

async fn require_repository_write(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
) -> Result<(), CollaborationError> {
    require_role(pool, repository_id, user_id, RepositoryRole::Write).await
}

async fn can_write_repository_id(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
) -> Result<bool, CollaborationError> {
    let repository = get_repository(pool, repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    if repository.owner_user_id == Some(user_id) {
        return Ok(true);
    }
    let permission = repository_permission_for_user(pool, repository_id, user_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryAccessDenied,
        })?;
    Ok(permission.is_some_and(|permission| permission.role.can_write()))
}

async fn resolve_create_head_repository(
    pool: &PgPool,
    base_repository: &Repository,
    head_repository_id: Option<Uuid>,
) -> Result<Repository, CollaborationError> {
    let Some(head_repository_id) = head_repository_id else {
        return Ok(base_repository.clone());
    };
    get_repository(pool, head_repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)
}

async fn can_open_pull_from_head_repository(
    pool: &PgPool,
    base_repository: &Repository,
    head_repository: &Repository,
    actor_user_id: Uuid,
) -> Result<bool, CollaborationError> {
    if base_repository.id == head_repository.id {
        return can_write_repository_id(pool, base_repository.id, actor_user_id).await;
    }
    let is_fork = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM repository_forks
            WHERE source_repository_id = $1 AND fork_repository_id = $2
        )
        "#,
    )
    .bind(base_repository.id)
    .bind(head_repository.id)
    .fetch_one(pool)
    .await?;
    if !is_fork {
        return Ok(false);
    }
    let base_readable = base_repository.visibility == RepositoryVisibility::Public
        || repository_viewer_permission(pool, base_repository, actor_user_id, RepositoryRole::Read)
            .await
            .is_ok();
    if !base_readable {
        return Ok(false);
    }
    can_write_repository_id(pool, head_repository.id, actor_user_id).await
}

async fn validate_compare_head_repository(
    pool: &PgPool,
    base_repository: &Repository,
    head_repository: &Repository,
    actor_user_id: Option<Uuid>,
) -> Result<(), CollaborationError> {
    if base_repository.id == head_repository.id {
        return Ok(());
    }
    let is_fork = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM repository_forks
            WHERE source_repository_id = $1 AND fork_repository_id = $2
        )
        "#,
    )
    .bind(base_repository.id)
    .bind(head_repository.id)
    .fetch_one(pool)
    .await?;
    if !is_fork {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "headRepositoryId".to_owned(),
            message: "head repository must be a readable fork of the base repository".to_owned(),
        });
    }
    match actor_user_id {
        Some(user_id) => {
            repository_viewer_permission(pool, head_repository, user_id, RepositoryRole::Read)
                .await?;
            Ok(())
        }
        None if head_repository.visibility == RepositoryVisibility::Public => Ok(()),
        None => Err(CollaborationError::RepositoryAccessDenied),
    }
}

async fn require_role(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
    required_role: RepositoryRole,
) -> Result<(), CollaborationError> {
    let permission = repository_permission_for_user(pool, repository_id, user_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryAccessDenied,
        })?
        .ok_or(CollaborationError::RepositoryAccessDenied)?;
    let allowed = match required_role {
        RepositoryRole::Read => permission.role.can_read(),
        RepositoryRole::Triage => permission.role >= RepositoryRole::Triage,
        RepositoryRole::Write => permission.role.can_write(),
        RepositoryRole::Maintain => permission.role >= RepositoryRole::Maintain,
        RepositoryRole::Admin => permission.role.can_admin(),
        RepositoryRole::Owner => permission.role == RepositoryRole::Owner,
    };
    if allowed {
        Ok(())
    } else {
        Err(CollaborationError::RepositoryAccessDenied)
    }
}

async fn repository_viewer_permission(
    pool: &PgPool,
    repository: &Repository,
    user_id: Uuid,
    required_role: RepositoryRole,
) -> Result<Option<String>, CollaborationError> {
    let permission = repository_permission_for_user(pool, repository.id, user_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryAccessDenied,
        })?;
    let Some(permission) = permission else {
        if required_role == RepositoryRole::Read
            && repository.visibility == RepositoryVisibility::Public
        {
            return Ok(Some("read".to_owned()));
        }
        return Err(CollaborationError::RepositoryAccessDenied);
    };

    let allowed = match required_role {
        RepositoryRole::Read => permission.role.can_read(),
        RepositoryRole::Triage => permission.role >= RepositoryRole::Triage,
        RepositoryRole::Write => permission.role.can_write(),
        RepositoryRole::Maintain => permission.role >= RepositoryRole::Maintain,
        RepositoryRole::Admin => permission.role.can_admin(),
        RepositoryRole::Owner => permission.role == RepositoryRole::Owner,
    };

    if allowed {
        Ok(Some(permission.role.as_str().to_owned()))
    } else {
        Err(CollaborationError::RepositoryAccessDenied)
    }
}

fn normalize_diff_view(value: &str) -> Result<String, CollaborationError> {
    let value = value.trim().to_lowercase();
    match value.as_str() {
        "" | "unified" => Ok("unified".to_owned()),
        "split" => Ok("split".to_owned()),
        _ => Err(CollaborationError::InvalidIssueFilter(
            "diff view must be unified or split".to_owned(),
        )),
    }
}

fn normalize_diff_whitespace(value: &str) -> Result<String, CollaborationError> {
    let value = value.trim().to_lowercase();
    match value.as_str() {
        "" | "show" => Ok("show".to_owned()),
        "hide" | "ignore" | "ignore-all" => Ok("hide".to_owned()),
        _ => Err(CollaborationError::InvalidIssueFilter(
            "whitespace must be show or hide".to_owned(),
        )),
    }
}

fn pull_request_file_version_key(blob_oid: Option<&str>, additions: i64, deletions: i64) -> String {
    format!(
        "{}:{}:{}",
        blob_oid.unwrap_or("no-blob"),
        additions.max(0),
        deletions.max(0)
    )
}

fn render_pull_request_diff(review: &PullRequestDiffReviewView) -> String {
    let pr = &review.pull_request;
    let mut out = String::new();
    out.push_str(&format!(
        "diff --opengithub a/{} b/{}\n",
        pr.base_ref, pr.head_ref
    ));
    out.push_str(&format!(
        "# Pull request #{}: {}\n",
        pr.number,
        pr.title.replace('\n', " ")
    ));
    out.push_str(&format!(
        "# {} additions, {} deletions across {} files\n",
        pr.stats.additions, pr.stats.deletions, review.total_files
    ));
    if review.has_more {
        out.push_str("# Output truncated to the first 100 files.\n");
    }
    out.push('\n');

    for file in &review.files {
        out.push_str(&format!("diff --git a/{0} b/{0}\n", file.path));
        out.push_str(&format!("--- {}\n", old_diff_path(file)));
        out.push_str(&format!("+++ {}\n", new_diff_path(file)));
        for hunk in &file.hunks {
            out.push_str(&hunk.header);
            out.push('\n');
            for line in &hunk.lines {
                out.push(diff_line_prefix(&line.kind));
                out.push_str(&line.content);
                out.push('\n');
            }
        }
        out.push('\n');
    }

    out
}

fn render_pull_request_patch(review: &PullRequestDiffReviewView) -> String {
    let pr = &review.pull_request;
    let mut out = String::new();
    for commit in &review.commits {
        out.push_str(&format!("From {} Mon Sep 17 00:00:00 2001\n", commit.oid));
        out.push_str(&format!(
            "From: {}\n",
            commit.author_login.as_deref().unwrap_or("unknown")
        ));
        out.push_str(&format!("Date: {}\n", commit.committed_at.to_rfc3339()));
        out.push_str(&format!(
            "Subject: [PATCH] {}\n\n",
            first_commit_line(&commit.message)
        ));
        let body = commit
            .message
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n");
        if !body.trim().is_empty() {
            out.push_str(body.trim());
            out.push_str("\n\n");
        }
    }
    if review.commits.is_empty() {
        out.push_str(&format!(
            "Subject: [PATCH] Pull request #{}: {}\n\n",
            pr.number,
            pr.title.replace('\n', " ")
        ));
    }
    out.push_str(&format!(
        "---\n {} files changed, {} insertions(+), {} deletions(-)\n\n",
        review.total_files, pr.stats.additions, pr.stats.deletions
    ));
    out.push_str(&render_pull_request_diff(review));
    out.push_str("-- \nopengithub\n");
    out
}

fn first_commit_line(message: &str) -> String {
    message
        .lines()
        .next()
        .unwrap_or("Pull request update")
        .trim()
        .chars()
        .take(160)
        .collect()
}

fn old_diff_path(file: &PullRequestDiffFile) -> String {
    if file.status == "added" {
        "/dev/null".to_owned()
    } else {
        format!("a/{}", file.path)
    }
}

fn new_diff_path(file: &PullRequestDiffFile) -> String {
    if file.status == "removed" {
        "/dev/null".to_owned()
    } else {
        format!("b/{}", file.path)
    }
}

fn diff_line_prefix(kind: &PullRequestDiffLineKind) -> char {
    match kind {
        PullRequestDiffLineKind::Added => '+',
        PullRequestDiffLineKind::Removed => '-',
        PullRequestDiffLineKind::Context => ' ',
    }
}

fn diff_anchor_for_path(path: &str) -> String {
    path.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}

fn language_for_path(path: &str) -> Option<String> {
    let extension = path.rsplit_once('.').map(|(_, ext)| ext.to_lowercase())?;
    let language = match extension.as_str() {
        "rs" => "Rust",
        "ts" | "tsx" => "TypeScript",
        "js" | "jsx" => "JavaScript",
        "json" => "JSON",
        "md" | "mdx" => "Markdown",
        "css" => "CSS",
        "html" => "HTML",
        "sql" => "SQL",
        "yml" | "yaml" => "YAML",
        "toml" => "TOML",
        _ => return None,
    };
    Some(language.to_owned())
}

async fn pull_request_diff_commits(
    pool: &PgPool,
    pull_request: &PullRequestDetailView,
) -> Result<Vec<CompareCommit>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT commits.id, commits.oid, commits.message,
               COALESCE(users.username, users.email) AS author_login,
               commits.committed_at
        FROM pull_request_commits snapshots
        JOIN commits ON commits.id = snapshots.commit_id
        LEFT JOIN users ON users.id = commits.author_user_id
        WHERE snapshots.pull_request_id = $1
        ORDER BY snapshots.position, commits.committed_at, commits.oid
        LIMIT 250
        "#,
    )
    .bind(pull_request.id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let oid: String = row.get("oid");
            CompareCommit {
                id: row.get("id"),
                short_oid: oid.chars().take(7).collect(),
                oid,
                message: row.get("message"),
                author_login: row.get("author_login"),
                committed_at: row.get("committed_at"),
                href: format!(
                    "/{}/{}/commit/{}",
                    pull_request.repository.owner_login,
                    pull_request.repository.name,
                    row.get::<String, _>("oid")
                ),
            }
        })
        .collect())
}

async fn pull_request_pending_review(
    pool: &PgPool,
    pull_request_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<PullRequestDiffPendingReview, CollaborationError> {
    let Some(actor_user_id) = actor_user_id else {
        return Ok(PullRequestDiffPendingReview {
            draft_id: None,
            comment_count: 0,
            summary_body: None,
            review_state: "commented".to_owned(),
        });
    };
    let draft = sqlx::query(
        r#"
        SELECT id, summary_body, review_state
        FROM pull_request_review_drafts
        WHERE pull_request_id = $1 AND author_user_id = $2
        "#,
    )
    .bind(pull_request_id)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?;
    let comment_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM pull_request_review_comments
        WHERE pull_request_id = $1
          AND author_user_id = $2
          AND state = 'pending'
        "#,
    )
    .bind(pull_request_id)
    .bind(actor_user_id)
    .fetch_one(pool)
    .await?;
    Ok(match draft {
        Some(row) => PullRequestDiffPendingReview {
            draft_id: row.get("id"),
            comment_count,
            summary_body: row.get("summary_body"),
            review_state: row.get("review_state"),
        },
        None => PullRequestDiffPendingReview {
            draft_id: None,
            comment_count,
            summary_body: None,
            review_state: "commented".to_owned(),
        },
    })
}

struct ValidatedDiffFileLine {
    repository_id: Uuid,
    path: String,
}

async fn validate_pull_request_diff_line(
    pool: &PgPool,
    pull_request_id: Uuid,
    file_id: Uuid,
    side: &str,
    old_line: Option<i64>,
    new_line: Option<i64>,
    position: i64,
) -> Result<ValidatedDiffFileLine, CollaborationError> {
    if position < 0 {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "position".to_owned(),
            message: "diff position must be zero or greater".to_owned(),
        });
    }
    if side == "left" && old_line.is_none() {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "oldLine".to_owned(),
            message: "left-side review comments require an old line".to_owned(),
        });
    }
    if side == "right" && new_line.is_none() {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "newLine".to_owned(),
            message: "right-side review comments require a new line".to_owned(),
        });
    }
    let row = sqlx::query(
        r#"
        SELECT files.path, pulls.repository_id
        FROM pull_request_files files
        JOIN pull_requests pulls ON pulls.id = files.pull_request_id
        WHERE files.id = $1
          AND files.pull_request_id = $2
          AND EXISTS (
            SELECT 1
            FROM pull_request_file_hunks hunks
            JOIN pull_request_hunk_lines lines ON lines.hunk_id = hunks.id
            WHERE hunks.pull_request_file_id = files.id
              AND lines.position = $3
              AND ($4::bigint IS NULL OR lines.old_line = $4)
              AND ($5::bigint IS NULL OR lines.new_line = $5)
          )
        "#,
    )
    .bind(file_id)
    .bind(pull_request_id)
    .bind(position)
    .bind(old_line)
    .bind(new_line)
    .fetch_optional(pool)
    .await?
    .ok_or(CollaborationError::InvalidIssueField {
        field_key: "position".to_owned(),
        message: "review comment position is not part of this pull request diff".to_owned(),
    })?;

    Ok(ValidatedDiffFileLine {
        repository_id: row.get("repository_id"),
        path: row.get("path"),
    })
}

fn normalize_review_comment_side(value: &str) -> Result<String, CollaborationError> {
    match value.trim().to_lowercase().as_str() {
        "left" => Ok("left".to_owned()),
        "right" | "" => Ok("right".to_owned()),
        _ => Err(CollaborationError::InvalidIssueField {
            field_key: "side".to_owned(),
            message: "review comment side must be left or right".to_owned(),
        }),
    }
}

fn normalize_submitted_review_state(value: &str) -> Result<String, CollaborationError> {
    match value.trim().to_lowercase().as_str() {
        "comment" | "commented" => Ok("commented".to_owned()),
        "approve" | "approved" => Ok("approved".to_owned()),
        "request_changes" | "request-changes" | "changes_requested" => {
            Ok("changes_requested".to_owned())
        }
        _ => Err(CollaborationError::InvalidIssueField {
            field_key: "state".to_owned(),
            message: "review state must be commented, approved, or changes_requested".to_owned(),
        }),
    }
}

async fn render_pull_request_review_comment_markdown(
    pool: &PgPool,
    repository_id: Uuid,
    body: &str,
) -> Result<String, CollaborationError> {
    let rendered = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            markdown: body.to_owned(),
            repository_id: Some(repository_id),
            owner: None,
            repo: None,
            ref_name: None,
            enable_task_toggles: Some(false),
        },
    )
    .await
    .map_err(|error| match error {
        super::markdown::MarkdownError::Sqlx(error) => CollaborationError::Sqlx(error),
        super::markdown::MarkdownError::TooLarge | super::markdown::MarkdownError::TaskNotFound => {
            CollaborationError::InvalidIssueField {
                field_key: "body".to_owned(),
                message: "review comment body could not be rendered".to_owned(),
            }
        }
    })?;
    Ok(rendered.html)
}

async fn pull_request_review_comment_from_row(
    pool: &PgPool,
    row: sqlx::postgres::PgRow,
) -> Result<PullRequestDiffReviewComment, CollaborationError> {
    let author_user_id = row.get("author_user_id");
    Ok(PullRequestDiffReviewComment {
        id: row.get("id"),
        author: user_for_review_comment(pool, author_user_id).await?,
        body: row.get("body"),
        body_html: row.get("body_html"),
        path: row.get("path"),
        side: row.get("side"),
        old_line: row.get("old_line"),
        new_line: row.get("new_line"),
        position: row.get("position"),
        state: row.get("state"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

async fn user_for_review_comment(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<IssueListUser, CollaborationError> {
    let row = sqlx::query(
        r#"
        SELECT id, COALESCE(username, email) AS login, display_name, avatar_url
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(IssueListUser {
        id: row.get("id"),
        login: row.get("login"),
        display_name: row.get("display_name"),
        avatar_url: row.get("avatar_url"),
    })
}

fn pull_request_from_row(row: sqlx::postgres::PgRow) -> Result<PullRequest, CollaborationError> {
    let state: String = row.get("state");
    Ok(PullRequest {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        issue_id: row.get("issue_id"),
        number: row.get("number"),
        title: row.get("title"),
        body: row.get("body"),
        state: PullRequestState::try_from(state.as_str())?,
        is_draft: row.try_get("is_draft").unwrap_or(false),
        author_user_id: row.get("author_user_id"),
        head_ref: row.get("head_ref"),
        base_ref: row.get("base_ref"),
        head_repository_id: row.get("head_repository_id"),
        base_repository_id: row.get("base_repository_id"),
        merge_commit_id: row.get("merge_commit_id"),
        merged_by_user_id: row.get("merged_by_user_id"),
        merged_at: row.get("merged_at"),
        closed_at: row.get("closed_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

async fn pull_request_by_id(
    pool: &PgPool,
    pull_request_id: Uuid,
) -> Result<PullRequest, CollaborationError> {
    let row = sqlx::query(
        r#"
        SELECT id, repository_id, issue_id, number, title, body, state, author_user_id,
               head_ref, base_ref, head_repository_id, base_repository_id, merge_commit_id,
               merged_by_user_id, merged_at, closed_at, created_at, updated_at, is_draft
        FROM pull_requests
        WHERE id = $1
        "#,
    )
    .bind(pull_request_id)
    .fetch_optional(pool)
    .await?
    .ok_or(CollaborationError::PullRequestNotFound)?;
    pull_request_from_row(row)
}

fn repository_from_row(row: sqlx::postgres::PgRow) -> Result<Repository, CollaborationError> {
    let visibility: String = row.get("visibility");
    Ok(Repository {
        id: row.get("id"),
        owner_user_id: row.get("owner_user_id"),
        owner_organization_id: row.get("owner_organization_id"),
        owner_login: row.get("owner_login"),
        name: row.get("name"),
        description: row.get("description"),
        visibility: RepositoryVisibility::try_from(visibility.as_str())
            .map_err(|_| CollaborationError::RepositoryNotFound)?,
        default_branch: row.get("default_branch"),
        is_archived: row.get("is_archived"),
        created_by_user_id: row.get("created_by_user_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn pull_request_list_repository(repository: &Repository) -> PullRequestListRepository {
    PullRequestListRepository {
        id: repository.id,
        owner_login: repository.owner_login.clone(),
        name: repository.name.clone(),
        visibility: repository.visibility.clone(),
        default_branch: repository.default_branch.clone(),
    }
}

fn compare_href_for_repositories(
    base_repository: &Repository,
    head_repository: &Repository,
    base_ref: &str,
    head_ref: &str,
) -> String {
    let base = format!(
        "/{}/{}/compare/{}...{}",
        base_repository.owner_login,
        base_repository.name,
        encode_path_component(base_ref),
        encode_path_component(head_ref)
    );
    if head_repository.id == base_repository.id {
        base
    } else {
        format!(
            "{}?headOwner={}&headRepo={}",
            base,
            encode_path_component(&head_repository.owner_login),
            encode_path_component(&head_repository.name)
        )
    }
}

async fn resolve_compare_ref(
    pool: &PgPool,
    repository: &Repository,
    ref_name: &str,
) -> Result<CompareRef, CollaborationError> {
    let normalized = normalize_compare_ref(ref_name)?;
    let branch_name = format!("refs/heads/{normalized}");
    let tag_name = format!("refs/tags/{normalized}");
    let row = sqlx::query(
        r#"
        SELECT repository_git_refs.name,
               repository_git_refs.kind,
               commits.id AS commit_id,
               commits.oid
        FROM repository_git_refs
        JOIN commits ON commits.id = repository_git_refs.target_commit_id
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
    .bind(&normalized)
    .bind(&branch_name)
    .bind(&tag_name)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        CollaborationError::InvalidIssueFilter(format!("comparison ref `{ref_name}` was not found"))
    })?;
    let name: String = row.get("name");
    let kind: String = row.get("kind");
    let short_name = short_ref_name(&name);

    Ok(CompareRef {
        repository: pull_request_list_repository(repository),
        name,
        short_name: short_name.clone(),
        kind,
        oid: row.get("oid"),
        commit_id: row.get("commit_id"),
        href: format!(
            "/{}/{}/tree/{}",
            repository.owner_login,
            repository.name,
            encode_path_component(&short_name)
        ),
    })
}

async fn commit_ancestor_oids(
    pool: &PgPool,
    repository_id: Uuid,
    start_oid: &str,
) -> Result<HashSet<String>, CollaborationError> {
    let mut seen = HashSet::new();
    let mut queue = VecDeque::from([start_oid.to_owned()]);

    while let Some(oid) = queue.pop_front() {
        if !seen.insert(oid.clone()) {
            continue;
        }
        let parent_oids = sqlx::query_scalar::<_, Vec<String>>(
            "SELECT parent_oids FROM commits WHERE repository_id = $1 AND oid = $2",
        )
        .bind(repository_id)
        .bind(&oid)
        .fetch_optional(pool)
        .await?
        .unwrap_or_default();
        for parent_oid in parent_oids {
            if !seen.contains(&parent_oid) {
                queue.push_back(parent_oid);
            }
        }
    }

    Ok(seen)
}

async fn compare_commits(
    pool: &PgPool,
    repository: &Repository,
    ahead_oids: &HashSet<String>,
    limit: i64,
) -> Result<Vec<CompareCommit>, CollaborationError> {
    if ahead_oids.is_empty() {
        return Ok(Vec::new());
    }
    let oids = ahead_oids.iter().cloned().collect::<Vec<_>>();
    let rows = sqlx::query(
        r#"
        SELECT commits.id,
               commits.oid,
               commits.message,
               commits.committed_at,
               COALESCE(NULLIF(users.username, ''), users.email) AS author_login
        FROM commits
        LEFT JOIN users ON users.id = commits.author_user_id
        WHERE commits.repository_id = $1
          AND commits.oid = ANY($2)
        ORDER BY commits.committed_at ASC, commits.created_at ASC
        LIMIT $3
        "#,
    )
    .bind(repository.id)
    .bind(&oids)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let oid: String = row.get("oid");
            CompareCommit {
                id: row.get("id"),
                short_oid: oid.chars().take(7).collect(),
                href: format!(
                    "/{}/{}/commits/{}",
                    repository.owner_login, repository.name, oid
                ),
                oid,
                message: row.get("message"),
                author_login: row.get("author_login"),
                committed_at: row.get("committed_at"),
            }
        })
        .collect())
}

async fn compare_files(
    pool: &PgPool,
    base_repository: &Repository,
    head_repository: &Repository,
    base: &CompareRef,
    head: &CompareRef,
    limit: i64,
) -> Result<Vec<CompareFile>, CollaborationError> {
    let base_files = files_at_commit(pool, base_repository.id, base.commit_id).await?;
    let head_files = files_at_commit(pool, head_repository.id, head.commit_id).await?;
    let paths = base_files
        .keys()
        .chain(head_files.keys())
        .cloned()
        .collect::<HashSet<_>>();
    let mut files = paths
        .into_iter()
        .filter_map(|path| {
            let base_file = base_files.get(&path);
            let head_file = head_files.get(&path);
            match (base_file, head_file) {
                (None, Some(head_file)) => Some(compare_file_from_parts(
                    head_repository,
                    FileCompareParts {
                        ref_name: &head.short_name,
                        path,
                        status: CompareFileStatus::Added,
                        old_content: "",
                        new_content: &head_file.content,
                        byte_size: head_file.byte_size,
                        blob_oid: Some(head_file.oid.clone()),
                    },
                )),
                (Some(base_file), None) => Some(compare_file_from_parts(
                    base_repository,
                    FileCompareParts {
                        ref_name: &head.short_name,
                        path,
                        status: CompareFileStatus::Removed,
                        old_content: &base_file.content,
                        new_content: "",
                        byte_size: base_file.byte_size,
                        blob_oid: Some(base_file.oid.clone()),
                    },
                )),
                (Some(base_file), Some(head_file)) if base_file.oid != head_file.oid => {
                    Some(compare_file_from_parts(
                        head_repository,
                        FileCompareParts {
                            ref_name: &head.short_name,
                            path,
                            status: CompareFileStatus::Modified,
                            old_content: &base_file.content,
                            new_content: &head_file.content,
                            byte_size: head_file.byte_size,
                            blob_oid: Some(head_file.oid.clone()),
                        },
                    ))
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.path.cmp(&right.path));
    files.truncate(limit as usize);
    Ok(files)
}

#[derive(Debug, Clone)]
struct FileSnapshot {
    oid: String,
    content: String,
    byte_size: i64,
}

struct FileCompareParts<'a> {
    ref_name: &'a str,
    path: String,
    status: CompareFileStatus,
    old_content: &'a str,
    new_content: &'a str,
    byte_size: i64,
    blob_oid: Option<String>,
}

async fn files_at_commit(
    pool: &PgPool,
    repository_id: Uuid,
    commit_id: Uuid,
) -> Result<HashMap<String, FileSnapshot>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT path, content, oid, byte_size
        FROM repository_files
        WHERE repository_id = $1 AND commit_id = $2
        "#,
    )
    .bind(repository_id)
    .bind(commit_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get("path"),
                FileSnapshot {
                    oid: row.get("oid"),
                    content: row.get("content"),
                    byte_size: row.get("byte_size"),
                },
            )
        })
        .collect())
}

fn compare_file_from_parts(repository: &Repository, parts: FileCompareParts<'_>) -> CompareFile {
    let old_lines = parts.old_content.lines().collect::<HashSet<_>>();
    let new_lines = parts.new_content.lines().collect::<HashSet<_>>();
    let additions = new_lines.difference(&old_lines).count() as i64;
    let deletions = old_lines.difference(&new_lines).count() as i64;

    CompareFile {
        href: format!(
            "/{}/{}/blob/{}/{}",
            repository.owner_login,
            repository.name,
            encode_path_component(parts.ref_name),
            parts
                .path
                .split('/')
                .map(encode_path_component)
                .collect::<Vec<_>>()
                .join("/")
        ),
        path: parts.path,
        status: parts.status,
        additions,
        deletions,
        byte_size: parts.byte_size,
        blob_oid: parts.blob_oid,
    }
}

async fn reject_duplicate_open_pull_request(
    pool: &PgPool,
    input: &CreatePullRequest,
) -> Result<(), CollaborationError> {
    let duplicate = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT number
        FROM pull_requests
        WHERE repository_id = $1
          AND state = 'open'
          AND base_ref = $2
          AND head_ref = $3
          AND COALESCE(head_repository_id, repository_id) = COALESCE($4, $1)
        LIMIT 1
        "#,
    )
    .bind(input.repository_id)
    .bind(&input.base_ref)
    .bind(&input.head_ref)
    .bind(input.head_repository_id)
    .fetch_optional(pool)
    .await?;

    if let Some(number) = duplicate {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "headRef".to_owned(),
            message: format!("an open pull request already exists for these refs: #{number}"),
        });
    }
    Ok(())
}

async fn validate_pull_request_create_metadata(
    pool: &PgPool,
    repository: &Repository,
    input: &CreatePullRequest,
) -> Result<(), CollaborationError> {
    for label_id in &input.label_ids {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM labels WHERE id = $1 AND repository_id = $2)",
        )
        .bind(label_id)
        .bind(repository.id)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(CollaborationError::InvalidIssueField {
                field_key: "labelIds".to_owned(),
                message: "label is not available for this repository".to_owned(),
            });
        }
    }

    if let Some(milestone_id) = input.milestone_id {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM milestones WHERE id = $1 AND repository_id = $2)",
        )
        .bind(milestone_id)
        .bind(repository.id)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(CollaborationError::InvalidIssueField {
                field_key: "milestoneId".to_owned(),
                message: "milestone is not available for this repository".to_owned(),
            });
        }
    }

    for user_id in input
        .assignee_user_ids
        .iter()
        .chain(input.reviewer_user_ids.iter())
    {
        if let Err(error) =
            repository_viewer_permission(pool, repository, *user_id, RepositoryRole::Read).await
        {
            return Err(match error {
                CollaborationError::RepositoryAccessDenied => {
                    CollaborationError::InvalidIssueField {
                        field_key: "userIds".to_owned(),
                        message: "requested user is not available for this repository".to_owned(),
                    }
                }
                other => other,
            });
        }
    }
    Ok(())
}

async fn apply_pull_request_template(
    pool: &PgPool,
    input: &mut CreatePullRequest,
) -> Result<(), CollaborationError> {
    let Some(slug) = input
        .template_slug
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    let body = sqlx::query_scalar::<_, String>(
        r#"
        SELECT body
        FROM pull_request_templates
        WHERE repository_id = $1 AND lower(slug) = lower($2)
        "#,
    )
    .bind(input.repository_id)
    .bind(slug)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| CollaborationError::InvalidIssueField {
        field_key: "templateSlug".to_owned(),
        message: "pull request template was not found".to_owned(),
    })?;
    if input
        .body
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        input.body = Some(body);
    }
    Ok(())
}

async fn persist_pull_request_snapshot(
    pool: &PgPool,
    pull_request: &PullRequest,
    compare: &PullRequestCompareView,
) -> Result<(), CollaborationError> {
    for (position, commit) in compare.commits.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO pull_request_commits (pull_request_id, commit_id, position)
            VALUES ($1, $2, $3)
            ON CONFLICT (pull_request_id, commit_id) DO NOTHING
            "#,
        )
        .bind(pull_request.id)
        .bind(commit.id)
        .bind(position as i64)
        .execute(pool)
        .await?;
    }

    for file in &compare.files {
        sqlx::query(
            r#"
            INSERT INTO pull_request_files (
                pull_request_id, path, status, additions, deletions, blob_oid, byte_size
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (pull_request_id, lower(path)) DO UPDATE SET
                status = EXCLUDED.status,
                additions = EXCLUDED.additions,
                deletions = EXCLUDED.deletions,
                blob_oid = EXCLUDED.blob_oid,
                byte_size = EXCLUDED.byte_size
            "#,
        )
        .bind(pull_request.id)
        .bind(&file.path)
        .bind(match file.status {
            CompareFileStatus::Added => "added",
            CompareFileStatus::Modified => "modified",
            CompareFileStatus::Removed => "removed",
        })
        .bind(file.additions)
        .bind(file.deletions)
        .bind(file.blob_oid.as_deref())
        .bind(file.byte_size)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn insert_review_requests(
    pool: &PgPool,
    pull_request: &PullRequest,
    actor_user_id: Uuid,
    reviewer_user_ids: &[Uuid],
) -> Result<(), CollaborationError> {
    for reviewer_user_id in reviewer_user_ids {
        if *reviewer_user_id == actor_user_id {
            continue;
        }
        sqlx::query(
            r#"
            INSERT INTO pull_request_review_requests (
                pull_request_id, requested_user_id, requested_by_user_id
            )
            VALUES ($1, $2, $3)
            ON CONFLICT (pull_request_id, requested_user_id) DO NOTHING
            "#,
        )
        .bind(pull_request.id)
        .bind(reviewer_user_id)
        .bind(actor_user_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn insert_closing_issue_references(
    pool: &PgPool,
    pull_request: &PullRequest,
    actor_user_id: Uuid,
) -> Result<(), CollaborationError> {
    let Some(body) = pull_request.body.as_deref() else {
        return Ok(());
    };
    let re = regex::Regex::new(r"(?i)\b(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?)\s+#(\d+)")
        .expect("closing keyword regex should compile");
    let numbers = re
        .captures_iter(body)
        .filter_map(|capture| capture.get(1))
        .filter_map(|number| number.as_str().parse::<i64>().ok())
        .collect::<HashSet<_>>();
    for number in numbers {
        if number == pull_request.number {
            continue;
        }
        let target_issue_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM issues WHERE repository_id = $1 AND number = $2",
        )
        .bind(pull_request.repository_id)
        .bind(number)
        .fetch_optional(pool)
        .await?;
        if let Some(target_issue_id) = target_issue_id {
            sqlx::query(
                r#"
                INSERT INTO issue_cross_references (
                    source_issue_id, target_issue_id, created_by_user_id
                )
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(pull_request.issue_id)
            .bind(target_issue_id)
            .bind(actor_user_id)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

async fn notify_pull_request_participants(
    pool: &PgPool,
    pull_request: &PullRequest,
    actor_user_id: Uuid,
    assignee_user_ids: &[Uuid],
    reviewer_user_ids: &[Uuid],
) -> Result<(), CollaborationError> {
    let mut recipients = HashSet::new();
    recipients.extend(assignee_user_ids.iter().copied());
    recipients.extend(reviewer_user_ids.iter().copied());
    for user_id in recipients {
        if user_id == pull_request.author_user_id {
            continue;
        }
        if !should_deliver_notification(
            pool,
            NotificationDeliveryCheck {
                user_id,
                repository_id: pull_request.repository_id,
                subject_type: "pull_request".to_owned(),
                subject_id: Some(pull_request.id),
                reason: "review_requested".to_owned(),
                repository_event: Some(RepositoryWatchEvent::PullRequests),
                actor_user_id: Some(actor_user_id),
                participating: false,
                direct: true,
            },
        )
        .await
        .map_err(|error| match error {
            super::notifications::NotificationError::Sqlx(error) => CollaborationError::Sqlx(error),
            super::notifications::NotificationError::NotFound => {
                CollaborationError::PullRequestNotFound
            }
            super::notifications::NotificationError::Validation(_) => {
                CollaborationError::PullRequestNotFound
            }
        })? {
            continue;
        }
        create_notification(
            pool,
            CreateNotification {
                user_id,
                repository_id: Some(pull_request.repository_id),
                subject_type: "pull_request".to_owned(),
                subject_id: Some(pull_request.id),
                title: format!(
                    "Pull request #{} needs your attention: {}",
                    pull_request.number, pull_request.title
                ),
                reason: "review_requested".to_owned(),
            },
        )
        .await
        .map_err(|error| match error {
            super::notifications::NotificationError::Sqlx(error) => CollaborationError::Sqlx(error),
            super::notifications::NotificationError::NotFound => {
                CollaborationError::PullRequestNotFound
            }
            super::notifications::NotificationError::Validation(_) => {
                CollaborationError::PullRequestNotFound
            }
        })?;
    }
    Ok(())
}

async fn notify_pull_request_review_submitted(
    pool: &PgPool,
    pull_request: &PullRequest,
    actor_user_id: Uuid,
    state: &str,
) -> Result<(), CollaborationError> {
    let mut recipients = HashSet::from([pull_request.author_user_id]);
    let requested_rows = sqlx::query(
        "SELECT requested_user_id FROM pull_request_review_requests WHERE pull_request_id = $1",
    )
    .bind(pull_request.id)
    .fetch_all(pool)
    .await?;
    for row in requested_rows {
        recipients.insert(row.get("requested_user_id"));
    }
    recipients.remove(&actor_user_id);

    for user_id in recipients {
        if !should_deliver_notification(
            pool,
            NotificationDeliveryCheck {
                user_id,
                repository_id: pull_request.repository_id,
                subject_type: "pull_request".to_owned(),
                subject_id: Some(pull_request.id),
                reason: "review_submitted".to_owned(),
                repository_event: Some(RepositoryWatchEvent::PullRequests),
                actor_user_id: Some(actor_user_id),
                participating: true,
                direct: false,
            },
        )
        .await
        .map_err(|error| match error {
            super::notifications::NotificationError::Sqlx(error) => CollaborationError::Sqlx(error),
            super::notifications::NotificationError::NotFound => {
                CollaborationError::PullRequestNotFound
            }
            super::notifications::NotificationError::Validation(_) => {
                CollaborationError::PullRequestNotFound
            }
        })? {
            continue;
        }
        create_notification(
            pool,
            CreateNotification {
                user_id,
                repository_id: Some(pull_request.repository_id),
                subject_type: "pull_request".to_owned(),
                subject_id: Some(pull_request.id),
                title: format!(
                    "Pull request #{} review submitted: {}",
                    pull_request.number,
                    state.replace('_', " ")
                ),
                reason: "review_submitted".to_owned(),
            },
        )
        .await
        .map_err(|error| match error {
            super::notifications::NotificationError::Sqlx(error) => CollaborationError::Sqlx(error),
            super::notifications::NotificationError::NotFound => {
                CollaborationError::PullRequestNotFound
            }
            super::notifications::NotificationError::Validation(_) => {
                CollaborationError::PullRequestNotFound
            }
        })?;
    }
    Ok(())
}

async fn insert_pull_request_audit_event(
    pool: &PgPool,
    pull_request: &PullRequest,
    actor_user_id: Uuid,
) -> Result<(), CollaborationError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'pull_request.created', 'pull_request', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(pull_request.id.to_string())
    .bind(json!({
        "repositoryId": pull_request.repository_id,
        "number": pull_request.number,
        "draft": pull_request.is_draft,
        "headRef": pull_request.head_ref,
        "baseRef": pull_request.base_ref
    }))
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_pull_request_action_audit_event(
    pool: &PgPool,
    pull_request: &PullRequest,
    actor_user_id: Uuid,
    event_type: &str,
    metadata: serde_json::Value,
) -> Result<(), CollaborationError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, $2, 'pull_request', $3, $4)
        "#,
    )
    .bind(actor_user_id)
    .bind(event_type)
    .bind(pull_request.id.to_string())
    .bind(json!({
        "repositoryId": pull_request.repository_id,
        "number": pull_request.number,
        "data": metadata,
    }))
    .execute(pool)
    .await?;
    Ok(())
}

async fn pull_request_href_by_id(
    pool: &PgPool,
    repository_id: Uuid,
    number: i64,
) -> Result<String, CollaborationError> {
    let repository = get_repository(pool, repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    Ok(pull_request_href(&repository, number))
}

fn pull_request_href(repository: &Repository, number: i64) -> String {
    format!(
        "/{}/{}/pull/{}",
        repository.owner_login, repository.name, number
    )
}

fn normalize_compare_ref(ref_name: &str) -> Result<String, CollaborationError> {
    let trimmed = ref_name.trim().trim_start_matches('/');
    if trimmed.is_empty() || trimmed.contains("..") {
        return Err(CollaborationError::InvalidIssueFilter(
            "comparison ref must be a branch or tag name".to_owned(),
        ));
    }
    Ok(trimmed.to_owned())
}

fn short_ref_name(name: &str) -> String {
    name.strip_prefix("refs/heads/")
        .or_else(|| name.strip_prefix("refs/tags/"))
        .unwrap_or(name)
        .to_owned()
}

fn encode_path_component(value: impl AsRef<str>) -> String {
    url::form_urlencoded::byte_serialize(value.as_ref().as_bytes()).collect()
}

async fn count_pull_request_list_items(
    pool: &PgPool,
    repository_id: Uuid,
    state: &str,
    text_filter: Option<&str>,
    filters: &PullRequestListQuery,
    actor_user_id: Option<Uuid>,
) -> Result<i64, CollaborationError> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM pull_requests
        JOIN issues ON issues.id = pull_requests.issue_id
        WHERE pull_requests.repository_id = $1
          AND pull_requests.state = $2
          AND (
              $3::text IS NULL
              OR pull_requests.title ILIKE '%' || $3 || '%'
              OR COALESCE(pull_requests.body, '') ILIKE '%' || $3 || '%'
              OR pull_requests.head_ref ILIKE '%' || $3 || '%'
              OR pull_requests.base_ref ILIKE '%' || $3 || '%'
          )
          AND (
              $4::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM users
                  WHERE users.id = pull_requests.author_user_id
                    AND (
                        lower(users.email) = lower($4)
                        OR lower(users.username) = lower($4)
                    )
              )
          )
          AND (
              cardinality($5::text[]) = 0
              OR NOT EXISTS (
                  SELECT 1
                  FROM unnest($5::text[]) wanted_label(name)
                  WHERE NOT EXISTS (
                      SELECT 1
                      FROM issue_labels
                      JOIN labels ON labels.id = issue_labels.label_id
                      WHERE issue_labels.issue_id = issues.id
                        AND lower(labels.name) = lower(wanted_label.name)
                  )
              )
          )
          AND (
              $6::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM milestones
                  WHERE milestones.id = issues.milestone_id
                    AND lower(milestones.title) = lower($6)
              )
          )
          AND (
              $7::bool = false
              OR issues.milestone_id IS NULL
          )
          AND (
              $8::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM issue_assignees
                  JOIN users ON users.id = issue_assignees.user_id
                  WHERE issue_assignees.issue_id = issues.id
                    AND (
                        lower(users.email) = lower($8)
                        OR lower(users.username) = lower($8)
                    )
              )
          )
          AND (
              $9::bool = false
              OR NOT EXISTS (
                  SELECT 1 FROM issue_assignees WHERE issue_assignees.issue_id = issues.id
              )
          )
          AND (
              $10::text IS NULL
              OR ($10 = 'none' AND NOT EXISTS (
                  SELECT 1
                  FROM pull_request_reviews
                  WHERE pull_request_reviews.pull_request_id = pull_requests.id
              ) AND NOT EXISTS (
                  SELECT 1
                  FROM pull_request_review_requests
                  WHERE pull_request_review_requests.pull_request_id = pull_requests.id
              ))
              OR ($10 IN ('approved', 'changes_requested', 'commented') AND EXISTS (
                  SELECT 1
                  FROM pull_request_reviews
                  WHERE pull_request_reviews.pull_request_id = pull_requests.id
                    AND pull_request_reviews.state = $10
              ))
              OR ($10 = 'required' AND EXISTS (
                  SELECT 1
                  FROM pull_request_review_requests
                  WHERE pull_request_review_requests.pull_request_id = pull_requests.id
              ))
              OR ($10 = 'reviewed_by_me' AND $11::uuid IS NOT NULL AND EXISTS (
                  SELECT 1
                  FROM pull_request_reviews
                  WHERE pull_request_reviews.pull_request_id = pull_requests.id
                    AND pull_request_reviews.reviewer_user_id = $11
              ))
              OR ($10 = 'not_reviewed_by_me' AND $11::uuid IS NOT NULL AND NOT EXISTS (
                  SELECT 1
                  FROM pull_request_reviews
                  WHERE pull_request_reviews.pull_request_id = pull_requests.id
                    AND pull_request_reviews.reviewer_user_id = $11
              ))
              OR ($10 = 'review_requested' AND $11::uuid IS NOT NULL AND EXISTS (
                  SELECT 1
                  FROM pull_request_review_requests
                  WHERE pull_request_review_requests.pull_request_id = pull_requests.id
                    AND pull_request_review_requests.requested_user_id = $11
              ))
              OR ($10 = 'team_review_requested' AND $11::uuid IS NOT NULL AND (
                  EXISTS (
                      SELECT 1
                      FROM pull_request_review_requests
                      WHERE pull_request_review_requests.pull_request_id = pull_requests.id
                        AND pull_request_review_requests.requested_user_id = $11
                  )
                  OR EXISTS (
                      SELECT 1
                      FROM pull_request_review_requests
                      JOIN team_memberships requested_memberships
                        ON requested_memberships.user_id = pull_request_review_requests.requested_user_id
                      JOIN team_memberships viewer_memberships
                        ON viewer_memberships.team_id = requested_memberships.team_id
                       AND viewer_memberships.user_id = $11
                      WHERE pull_request_review_requests.pull_request_id = pull_requests.id
                  )
              ))
          )
          AND (
              $12::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM pull_request_checks_summary
                  WHERE pull_request_checks_summary.pull_request_id = pull_requests.id
                    AND (
                        pull_request_checks_summary.status = $12
                        OR pull_request_checks_summary.conclusion = $12
                    )
              )
          )
        "#,
    )
    .bind(repository_id)
    .bind(state)
    .bind(text_filter)
    .bind(filters.author.as_deref())
    .bind(&filters.labels)
    .bind(filters.milestone.as_deref())
    .bind(filters.no_milestone)
    .bind(filters.assignee.as_deref())
    .bind(filters.no_assignee)
    .bind(filters.review.as_deref())
    .bind(actor_user_id)
    .bind(filters.checks.as_deref())
    .fetch_one(pool)
    .await
    .map_err(CollaborationError::from)
}

async fn pull_request_list_items(
    pool: &PgPool,
    repository: &Repository,
    pull_requests: Vec<PullRequest>,
) -> Result<Vec<PullRequestListItem>, CollaborationError> {
    let pull_ids = pull_requests
        .iter()
        .map(|pull_request| pull_request.id)
        .collect::<Vec<_>>();
    let issue_ids = pull_requests
        .iter()
        .map(|pull_request| pull_request.issue_id)
        .collect::<Vec<_>>();
    let authors = pull_list_authors(pool, &pull_ids).await?;
    let labels = pull_list_labels(pool, &issue_ids).await?;
    let milestones = pull_list_milestones(pool, &issue_ids).await?;
    let comment_counts = pull_comment_counts(pool, &pull_ids).await?;
    let linked_issues = linked_issue_hints(pool, &issue_ids, repository).await?;
    let reviews = pull_review_summaries(pool, &pull_ids).await?;
    let checks = pull_check_summaries(pool, &pull_ids).await?;
    let tasks = pull_task_progress(pool, &pull_ids).await?;
    let roles = pull_author_roles(pool, repository.id, &pull_ids).await?;

    Ok(pull_requests
        .into_iter()
        .map(|pull_request| {
            let href = format!(
                "/{}/{}/pull/{}",
                repository.owner_login, repository.name, pull_request.number
            );
            PullRequestListItem {
                id: pull_request.id,
                repository_id: pull_request.repository_id,
                repository_owner: repository.owner_login.clone(),
                repository_name: repository.name.clone(),
                number: pull_request.number,
                title: pull_request.title,
                body: pull_request.body,
                state: pull_request.state,
                is_draft: pull_request.is_draft,
                author: authors
                    .get(&pull_request.id)
                    .cloned()
                    .unwrap_or_else(|| fallback_user(pull_request.author_user_id)),
                author_role: roles
                    .get(&pull_request.id)
                    .cloned()
                    .unwrap_or_else(|| "contributor".to_owned()),
                labels: labels
                    .get(&pull_request.issue_id)
                    .cloned()
                    .unwrap_or_default(),
                milestone: milestones.get(&pull_request.issue_id).cloned(),
                comment_count: *comment_counts.get(&pull_request.id).unwrap_or(&0),
                linked_issues: linked_issues
                    .get(&pull_request.issue_id)
                    .cloned()
                    .unwrap_or_default(),
                review: reviews
                    .get(&pull_request.id)
                    .cloned()
                    .unwrap_or_else(default_review_summary),
                checks: checks
                    .get(&pull_request.id)
                    .cloned()
                    .unwrap_or_else(default_checks_summary),
                task_progress: tasks.get(&pull_request.id).cloned().unwrap_or(
                    PullRequestTaskProgress {
                        completed: 0,
                        total: 0,
                    },
                ),
                head_ref: pull_request.head_ref,
                base_ref: pull_request.base_ref,
                checks_href: format!("{href}/checks"),
                reviews_href: format!("{href}#reviews"),
                comments_href: format!("{href}#comments"),
                linked_issues_href: format!("{href}#linked-issues"),
                href,
                created_at: pull_request.created_at,
                updated_at: pull_request.updated_at,
                closed_at: pull_request.closed_at,
                merged_at: pull_request.merged_at,
            }
        })
        .collect())
}

async fn pull_list_authors(
    pool: &PgPool,
    pull_request_ids: &[Uuid],
) -> Result<HashMap<Uuid, IssueListUser>, CollaborationError> {
    if pull_request_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT pull_requests.id AS pull_request_id, users.id,
               COALESCE(users.username, users.email) AS login,
               users.display_name, users.avatar_url
        FROM pull_requests
        JOIN users ON users.id = pull_requests.author_user_id
        WHERE pull_requests.id = ANY($1)
        "#,
    )
    .bind(pull_request_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get("pull_request_id"),
                IssueListUser {
                    id: row.get("id"),
                    login: row.get("login"),
                    display_name: row.get("display_name"),
                    avatar_url: row.get("avatar_url"),
                },
            )
        })
        .collect())
}

async fn pull_list_labels(
    pool: &PgPool,
    issue_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<IssueListLabel>>, CollaborationError> {
    let mut by_issue: HashMap<Uuid, Vec<IssueListLabel>> = HashMap::new();
    if issue_ids.is_empty() {
        return Ok(by_issue);
    }
    let rows = sqlx::query(
        r#"
        SELECT issue_labels.issue_id, labels.id, labels.name, labels.color, labels.description
        FROM issue_labels
        JOIN labels ON labels.id = issue_labels.label_id
        WHERE issue_labels.issue_id = ANY($1)
        ORDER BY lower(labels.name)
        "#,
    )
    .bind(issue_ids)
    .fetch_all(pool)
    .await?;
    for row in rows {
        by_issue
            .entry(row.get("issue_id"))
            .or_default()
            .push(IssueListLabel {
                id: row.get("id"),
                name: row.get("name"),
                color: row.get("color"),
                description: row.get("description"),
            });
    }
    Ok(by_issue)
}

async fn pull_list_assignees(
    pool: &PgPool,
    issue_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<IssueListUser>>, CollaborationError> {
    let mut by_issue: HashMap<Uuid, Vec<IssueListUser>> = HashMap::new();
    if issue_ids.is_empty() {
        return Ok(by_issue);
    }
    let rows = sqlx::query(
        r#"
        SELECT issue_assignees.issue_id, users.id, COALESCE(users.username, users.email) AS login,
               users.display_name, users.avatar_url
        FROM issue_assignees
        JOIN users ON users.id = issue_assignees.user_id
        WHERE issue_assignees.issue_id = ANY($1)
        ORDER BY lower(COALESCE(users.username, users.email))
        "#,
    )
    .bind(issue_ids)
    .fetch_all(pool)
    .await?;
    for row in rows {
        by_issue
            .entry(row.get("issue_id"))
            .or_default()
            .push(IssueListUser {
                id: row.get("id"),
                login: row.get("login"),
                display_name: row.get("display_name"),
                avatar_url: row.get("avatar_url"),
            });
    }
    Ok(by_issue)
}

async fn pull_list_label_options(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<IssueListLabel>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, color, description
        FROM labels
        WHERE repository_id = $1
        ORDER BY lower(name)
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| IssueListLabel {
            id: row.get("id"),
            name: row.get("name"),
            color: row.get("color"),
            description: row.get("description"),
        })
        .collect())
}

async fn pull_list_user_options(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<IssueListUser>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT users.id, COALESCE(users.username, users.email) AS login,
               users.display_name, users.avatar_url
        FROM users
        WHERE users.id IN (
            SELECT repositories.created_by_user_id
            FROM repositories
            WHERE repositories.id = $1
            UNION
            SELECT repository_permissions.user_id
            FROM repository_permissions
            WHERE repository_permissions.repository_id = $1
            UNION
            SELECT pull_requests.author_user_id
            FROM pull_requests
            WHERE pull_requests.repository_id = $1
            UNION
            SELECT issue_assignees.user_id
            FROM issue_assignees
            JOIN issues ON issues.id = issue_assignees.issue_id
            JOIN pull_requests ON pull_requests.issue_id = issues.id
            WHERE pull_requests.repository_id = $1
        )
        ORDER BY lower(COALESCE(users.username, users.email))
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| IssueListUser {
            id: row.get("id"),
            login: row.get("login"),
            display_name: row.get("display_name"),
            avatar_url: row.get("avatar_url"),
        })
        .collect())
}

async fn pull_list_milestones(
    pool: &PgPool,
    issue_ids: &[Uuid],
) -> Result<HashMap<Uuid, IssueListMilestone>, CollaborationError> {
    if issue_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT issues.id AS issue_id, milestones.id, milestones.title, milestones.state
        FROM issues
        JOIN milestones ON milestones.id = issues.milestone_id
        WHERE issues.id = ANY($1)
        "#,
    )
    .bind(issue_ids)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let state: String = row.get("state");
            Ok((
                row.get("issue_id"),
                IssueListMilestone {
                    id: row.get("id"),
                    title: row.get("title"),
                    state: IssueState::try_from(state.as_str())?,
                },
            ))
        })
        .collect()
}

async fn pull_list_milestone_options(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<IssueListMilestone>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, state
        FROM milestones
        WHERE repository_id = $1
        ORDER BY state ASC, lower(title)
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let state: String = row.get("state");
            Ok(IssueListMilestone {
                id: row.get("id"),
                title: row.get("title"),
                state: IssueState::try_from(state.as_str())?,
            })
        })
        .collect()
}

async fn pull_request_template_options(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<PullRequestTemplateOption>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT slug, name, body
        FROM pull_request_templates
        WHERE repository_id = $1
        ORDER BY display_order ASC, lower(name)
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| PullRequestTemplateOption {
            slug: row.get("slug"),
            name: row.get("name"),
            body: row.get("body"),
        })
        .collect())
}

async fn pull_request_label_options(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<IssueListLabel>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, color, description
        FROM labels
        WHERE repository_id = $1
        ORDER BY is_default DESC, lower(name)
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| IssueListLabel {
            id: row.get("id"),
            name: row.get("name"),
            color: row.get("color"),
            description: row.get("description"),
        })
        .collect())
}

async fn pull_request_create_options(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Option<Uuid>,
    selected_head_repository: &Repository,
    base_ref: &str,
    head_ref: &str,
    include_base_metadata: bool,
) -> Result<PullRequestCreateOptions, CollaborationError> {
    Ok(PullRequestCreateOptions {
        can_create: true,
        templates: pull_request_template_options(pool, repository.id).await?,
        labels: if include_base_metadata {
            pull_request_label_options(pool, repository.id).await?
        } else {
            Vec::new()
        },
        users: if include_base_metadata {
            pull_list_user_options(pool, repository.id).await?
        } else {
            Vec::new()
        },
        milestones: if include_base_metadata {
            pull_list_milestone_options(pool, repository.id).await?
        } else {
            Vec::new()
        },
        fork_repositories: pull_request_fork_options(
            pool,
            repository,
            actor_user_id,
            selected_head_repository,
            base_ref,
            head_ref,
        )
        .await?,
    })
}

async fn global_pull_request_list_items(
    pool: &PgPool,
    pull_requests: Vec<PullRequest>,
) -> Result<Vec<PullRequestListItem>, CollaborationError> {
    let mut by_repository: HashMap<Uuid, Vec<PullRequest>> = HashMap::new();
    let mut order = Vec::new();
    for pull_request in pull_requests {
        if !by_repository.contains_key(&pull_request.repository_id) {
            order.push(pull_request.repository_id);
        }
        by_repository
            .entry(pull_request.repository_id)
            .or_default()
            .push(pull_request);
    }

    let mut grouped_items = HashMap::new();
    for repository_id in &order {
        let repository = get_repository(pool, *repository_id)
            .await
            .map_err(|error| match error {
                super::repositories::RepositoryError::Sqlx(error) => {
                    CollaborationError::Sqlx(error)
                }
                _ => CollaborationError::RepositoryNotFound,
            })?
            .ok_or(CollaborationError::RepositoryNotFound)?;
        let pulls = by_repository.remove(repository_id).unwrap_or_default();
        grouped_items.insert(
            *repository_id,
            pull_request_list_items(pool, &repository, pulls).await?,
        );
    }

    let mut items = Vec::new();
    for repository_id in order {
        items.extend(grouped_items.remove(&repository_id).unwrap_or_default());
    }
    Ok(items)
}

async fn count_global_pull_request_list_items(
    pool: &PgPool,
    actor_user_id: Uuid,
    scope: &str,
    state_filter: Option<&str>,
    text_filter: Option<&str>,
    filters: &GlobalPullRequestListQuery,
) -> Result<i64, CollaborationError> {
    Ok(sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM pull_requests
        JOIN issues ON issues.id = pull_requests.issue_id
        JOIN repositories ON repositories.id = pull_requests.repository_id
        LEFT JOIN users owner_users ON owner_users.id = repositories.owner_user_id
        LEFT JOIN organizations owner_orgs ON owner_orgs.id = repositories.owner_organization_id
        WHERE (
              repositories.visibility = 'public'
              OR EXISTS (
                  SELECT 1
                  FROM repository_permissions
                  WHERE repository_permissions.repository_id = repositories.id
                    AND repository_permissions.user_id = $1
                    AND repository_permissions.role IN ('owner', 'admin', 'maintain', 'write', 'triage', 'read')
              )
          )
          AND (
              ($2 = 'created' AND pull_requests.author_user_id = $1)
              OR ($2 = 'assigned' AND EXISTS (
                  SELECT 1
                  FROM issue_assignees
                  WHERE issue_assignees.issue_id = pull_requests.issue_id
                    AND issue_assignees.user_id = $1
              ))
              OR ($2 = 'mentioned' AND EXISTS (
                  SELECT 1
                  FROM notifications
                  WHERE notifications.subject_type = 'pull_request'
                    AND notifications.subject_id = pull_requests.id
                    AND notifications.user_id = $1
                    AND notifications.reason IN ('mention', 'team_mention')
              ))
              OR ($2 = 'review_requests' AND EXISTS (
                  SELECT 1
                  FROM pull_request_review_requests
                  WHERE pull_request_review_requests.pull_request_id = pull_requests.id
                    AND pull_request_review_requests.requested_user_id = $1
              ))
          )
          AND ($3::text IS NULL OR pull_requests.state = $3)
          AND (
              $4::text IS NULL
              OR pull_requests.title ILIKE '%' || $4 || '%'
              OR COALESCE(pull_requests.body, '') ILIKE '%' || $4 || '%'
              OR pull_requests.head_ref ILIKE '%' || $4 || '%'
              OR pull_requests.base_ref ILIKE '%' || $4 || '%'
              OR repositories.name ILIKE '%' || $4 || '%'
              OR COALESCE(owner_users.username, owner_users.email, owner_orgs.slug) ILIKE '%' || $4 || '%'
          )
          AND (
              $5::text IS NULL
              OR lower(format('%s/%s', COALESCE(owner_users.username, owner_users.email, owner_orgs.slug), repositories.name)) = lower($5)
              OR lower(repositories.name) = lower($5)
          )
          AND (
              cardinality($6::text[]) = 0
              OR NOT EXISTS (
                  SELECT 1
                  FROM unnest($6::text[]) wanted_label(name)
                  WHERE NOT EXISTS (
                      SELECT 1
                      FROM issue_labels
                      JOIN labels ON labels.id = issue_labels.label_id
                      WHERE issue_labels.issue_id = issues.id
                        AND lower(labels.name) = lower(wanted_label.name)
                  )
              )
          )
          AND (
              $7::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM milestones
                  WHERE milestones.id = issues.milestone_id
                    AND lower(milestones.title) = lower($7)
              )
          )
        "#,
    )
    .bind(actor_user_id)
    .bind(scope)
    .bind(state_filter)
    .bind(text_filter)
    .bind(filters.repository.as_deref())
    .bind(&filters.labels)
    .bind(filters.milestone.as_deref())
    .fetch_one(pool)
    .await?)
}

async fn global_pull_repository_options(
    pool: &PgPool,
    actor_user_id: Uuid,
    scope: &str,
) -> Result<Vec<GlobalPullRequestRepositoryOption>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT repositories.id,
               COALESCE(owner_users.username, owner_users.email, owner_orgs.slug) AS owner_login,
               repositories.name,
               count(*) AS count
        FROM pull_requests
        JOIN issues ON issues.id = pull_requests.issue_id
        JOIN repositories ON repositories.id = pull_requests.repository_id
        LEFT JOIN users owner_users ON owner_users.id = repositories.owner_user_id
        LEFT JOIN organizations owner_orgs ON owner_orgs.id = repositories.owner_organization_id
        WHERE (
              repositories.visibility = 'public'
              OR EXISTS (
                  SELECT 1 FROM repository_permissions
                  WHERE repository_permissions.repository_id = repositories.id
                    AND repository_permissions.user_id = $1
                    AND repository_permissions.role IN ('owner', 'admin', 'maintain', 'write', 'triage', 'read')
              )
          )
          AND (
              ($2 = 'created' AND pull_requests.author_user_id = $1)
              OR ($2 = 'assigned' AND EXISTS (
                  SELECT 1 FROM issue_assignees
                  WHERE issue_assignees.issue_id = pull_requests.issue_id
                    AND issue_assignees.user_id = $1
              ))
              OR ($2 = 'mentioned' AND EXISTS (
                  SELECT 1 FROM notifications
                  WHERE notifications.subject_type = 'pull_request'
                    AND notifications.subject_id = pull_requests.id
                    AND notifications.user_id = $1
                    AND notifications.reason IN ('mention', 'team_mention')
              ))
              OR ($2 = 'review_requests' AND EXISTS (
                  SELECT 1 FROM pull_request_review_requests
                  WHERE pull_request_review_requests.pull_request_id = pull_requests.id
                    AND pull_request_review_requests.requested_user_id = $1
              ))
          )
        GROUP BY repositories.id, owner_login, repositories.name
        ORDER BY count(*) DESC, owner_login ASC, repositories.name ASC
        LIMIT 50
        "#,
    )
    .bind(actor_user_id)
    .bind(scope)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let owner_login: String = row.get("owner_login");
            let name: String = row.get("name");
            GlobalPullRequestRepositoryOption {
                id: row.get("id"),
                full_name: format!("{owner_login}/{name}"),
                owner_login,
                name,
                count: row.get("count"),
            }
        })
        .collect())
}

async fn global_pull_label_options(
    pool: &PgPool,
    actor_user_id: Uuid,
    scope: &str,
) -> Result<Vec<IssueListLabel>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT labels.id, labels.name, labels.color, labels.description, count(*) AS uses
        FROM pull_requests
        JOIN issues ON issues.id = pull_requests.issue_id
        JOIN repositories ON repositories.id = pull_requests.repository_id
        JOIN issue_labels ON issue_labels.issue_id = issues.id
        JOIN labels ON labels.id = issue_labels.label_id
        WHERE (
              repositories.visibility = 'public'
              OR EXISTS (
                  SELECT 1 FROM repository_permissions
                  WHERE repository_permissions.repository_id = repositories.id
                    AND repository_permissions.user_id = $1
                    AND repository_permissions.role IN ('owner', 'admin', 'maintain', 'write', 'triage', 'read')
              )
          )
          AND (
              ($2 = 'created' AND pull_requests.author_user_id = $1)
              OR ($2 = 'assigned' AND EXISTS (
                  SELECT 1 FROM issue_assignees WHERE issue_assignees.issue_id = pull_requests.issue_id AND issue_assignees.user_id = $1
              ))
              OR ($2 = 'mentioned' AND EXISTS (
                  SELECT 1 FROM notifications WHERE notifications.subject_type = 'pull_request' AND notifications.subject_id = pull_requests.id AND notifications.user_id = $1 AND notifications.reason IN ('mention', 'team_mention')
              ))
              OR ($2 = 'review_requests' AND EXISTS (
                  SELECT 1 FROM pull_request_review_requests WHERE pull_request_review_requests.pull_request_id = pull_requests.id AND pull_request_review_requests.requested_user_id = $1
              ))
          )
        GROUP BY labels.id, labels.name, labels.color, labels.description
        ORDER BY uses DESC, lower(labels.name) ASC
        LIMIT 50
        "#,
    )
    .bind(actor_user_id)
    .bind(scope)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| IssueListLabel {
            id: row.get("id"),
            name: row.get("name"),
            color: row.get("color"),
            description: row.get("description"),
        })
        .collect())
}

async fn global_pull_milestone_options(
    pool: &PgPool,
    actor_user_id: Uuid,
    scope: &str,
) -> Result<Vec<IssueListMilestone>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT milestones.id, milestones.title, milestones.state, milestones.due_on,
               count(*) AS pull_count
        FROM pull_requests
        JOIN issues ON issues.id = pull_requests.issue_id
        JOIN repositories ON repositories.id = pull_requests.repository_id
        JOIN milestones ON milestones.id = issues.milestone_id
        WHERE (
              repositories.visibility = 'public'
              OR EXISTS (
                  SELECT 1 FROM repository_permissions
                  WHERE repository_permissions.repository_id = repositories.id
                    AND repository_permissions.user_id = $1
                    AND repository_permissions.role IN ('owner', 'admin', 'maintain', 'write', 'triage', 'read')
              )
          )
          AND (
              ($2 = 'created' AND pull_requests.author_user_id = $1)
              OR ($2 = 'assigned' AND EXISTS (
                  SELECT 1 FROM issue_assignees WHERE issue_assignees.issue_id = pull_requests.issue_id AND issue_assignees.user_id = $1
              ))
              OR ($2 = 'mentioned' AND EXISTS (
                  SELECT 1 FROM notifications WHERE notifications.subject_type = 'pull_request' AND notifications.subject_id = pull_requests.id AND notifications.user_id = $1 AND notifications.reason IN ('mention', 'team_mention')
              ))
              OR ($2 = 'review_requests' AND EXISTS (
                  SELECT 1 FROM pull_request_review_requests WHERE pull_request_review_requests.pull_request_id = pull_requests.id AND pull_request_review_requests.requested_user_id = $1
              ))
          )
        GROUP BY milestones.id, milestones.title, milestones.state, milestones.due_on
        ORDER BY pull_count DESC, lower(milestones.title) ASC
        LIMIT 50
        "#,
    )
    .bind(actor_user_id)
    .bind(scope)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let state: String = row.get("state");
            Ok(IssueListMilestone {
                id: row.get("id"),
                title: row.get("title"),
                state: IssueState::try_from(state.as_str())?,
            })
        })
        .collect()
}

async fn pull_request_fork_options(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Option<Uuid>,
    selected_head_repository: &Repository,
    base_ref: &str,
    head_ref: &str,
) -> Result<Vec<PullRequestCompareRepositoryOption>, CollaborationError> {
    let mut options = vec![compare_repository_option(
        repository,
        repository,
        base_ref,
        head_ref,
        true,
        selected_head_repository.id == repository.id,
    )];
    let rows = sqlx::query(
        r#"
        SELECT forks.id,
               forks.owner_user_id,
               forks.owner_organization_id,
               COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug) AS owner_login,
               forks.name,
               forks.description,
               forks.visibility,
               forks.default_branch,
               forks.is_archived,
               forks.created_by_user_id,
               forks.created_at,
               forks.updated_at
        FROM repository_forks
        JOIN repositories forks ON forks.id = repository_forks.fork_repository_id
        LEFT JOIN users owner_user ON owner_user.id = forks.owner_user_id
        LEFT JOIN organizations ON organizations.id = forks.owner_organization_id
        WHERE repository_forks.source_repository_id = $1
        ORDER BY repository_forks.created_at DESC
        LIMIT 25
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;
    for row in rows {
        let fork = repository_from_row(row)?;
        let readable = match actor_user_id {
            Some(user_id) => {
                repository_viewer_permission(pool, &fork, user_id, RepositoryRole::Read)
                    .await
                    .is_ok()
            }
            None => fork.visibility == RepositoryVisibility::Public,
        };
        if readable {
            options.push(compare_repository_option(
                repository,
                &fork,
                base_ref,
                head_ref,
                false,
                selected_head_repository.id == fork.id,
            ));
        }
    }
    Ok(options)
}

fn compare_repository_option(
    base_repository: &Repository,
    option_repository: &Repository,
    base_ref: &str,
    head_ref: &str,
    is_base: bool,
    is_selected_head: bool,
) -> PullRequestCompareRepositoryOption {
    PullRequestCompareRepositoryOption {
        id: option_repository.id,
        owner_login: option_repository.owner_login.clone(),
        name: option_repository.name.clone(),
        visibility: option_repository.visibility.clone(),
        default_branch: option_repository.default_branch.clone(),
        href: format!(
            "/{}/{}",
            option_repository.owner_login, option_repository.name
        ),
        compare_href: compare_href_for_repositories(
            base_repository,
            option_repository,
            base_ref,
            head_ref,
        ),
        is_base,
        is_selected_head,
    }
}

async fn pull_list_project_options() -> Result<Vec<IssueListMetadataOption>, CollaborationError> {
    Ok(vec![IssueListMetadataOption {
        id: "projects-unavailable".to_owned(),
        name: "No repository projects".to_owned(),
        description: Some("Project links are not modeled for pull requests yet.".to_owned()),
        count: 0,
        disabled_reason: Some("Project filters will activate when project links exist.".to_owned()),
    }])
}

async fn pull_comment_counts(
    pool: &PgPool,
    pull_request_ids: &[Uuid],
) -> Result<HashMap<Uuid, i64>, CollaborationError> {
    if pull_request_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT pull_request_id, count(*) AS count
        FROM comments
        WHERE pull_request_id = ANY($1)
        GROUP BY pull_request_id
        "#,
    )
    .bind(pull_request_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.get("pull_request_id"), row.get("count")))
        .collect())
}

async fn pull_request_detail_stats(
    pool: &PgPool,
    pull_request_id: Uuid,
    comments: i64,
) -> Result<PullRequestDetailStats, CollaborationError> {
    let row = sqlx::query(
        r#"
        SELECT
            (SELECT count(*) FROM pull_request_commits WHERE pull_request_id = $1) AS commits,
            (SELECT count(*) FROM pull_request_files WHERE pull_request_id = $1) AS files,
            COALESCE((SELECT sum(additions) FROM pull_request_files WHERE pull_request_id = $1), 0)::bigint AS additions,
            COALESCE((SELECT sum(deletions) FROM pull_request_files WHERE pull_request_id = $1), 0)::bigint AS deletions
        "#,
    )
    .bind(pull_request_id)
    .fetch_one(pool)
    .await?;

    Ok(PullRequestDetailStats {
        commits: row.get("commits"),
        files: row.get("files"),
        additions: row.get("additions"),
        deletions: row.get("deletions"),
        comments,
    })
}

async fn pull_request_subscription_state(
    pool: &PgPool,
    pull_request: &PullRequest,
    actor_user_id: Option<Uuid>,
) -> Result<PullRequestSubscriptionState, CollaborationError> {
    let Some(actor_user_id) = actor_user_id else {
        return Ok(PullRequestSubscriptionState {
            subscribed: false,
            reason: "anonymous".to_owned(),
            custom_events: Vec::new(),
            can_customize: false,
        });
    };
    let explicit = sqlx::query(
        r#"
        SELECT subscribed, reason, custom_events
        FROM pull_request_subscriptions
        WHERE pull_request_id = $1 AND user_id = $2
        "#,
    )
    .bind(pull_request.id)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?;
    if let Some(row) = explicit {
        return Ok(PullRequestSubscriptionState {
            subscribed: row.get("subscribed"),
            reason: row.get("reason"),
            custom_events: row.get("custom_events"),
            can_customize: true,
        });
    }
    let participating = pull_request.author_user_id == actor_user_id
        || sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM issue_assignees
                WHERE issue_id = $1 AND user_id = $2
            )
            OR EXISTS (
                SELECT 1 FROM pull_request_review_requests
                WHERE pull_request_id = $3 AND requested_user_id = $2
            )
            OR EXISTS (
                SELECT 1 FROM pull_request_reviews
                WHERE pull_request_id = $3 AND reviewer_user_id = $2
            )
            OR EXISTS (
                SELECT 1 FROM comments
                WHERE pull_request_id = $3 AND author_user_id = $2
            )
            "#,
        )
        .bind(pull_request.issue_id)
        .bind(actor_user_id)
        .bind(pull_request.id)
        .fetch_one(pool)
        .await?;
    Ok(PullRequestSubscriptionState {
        subscribed: participating,
        reason: if participating {
            "participating".to_owned()
        } else {
            "not_subscribed".to_owned()
        },
        custom_events: Vec::new(),
        can_customize: true,
    })
}

async fn pull_request_mergeability(
    pool: &PgPool,
    pull_request: &PullRequest,
    actor_user_id: Option<Uuid>,
) -> Result<PullRequestMergeability, CollaborationError> {
    let can_write = match actor_user_id {
        Some(user_id) => can_write_repository_id(pool, pull_request.repository_id, user_id).await?,
        None => false,
    };
    let stats = pull_request_detail_stats(pool, pull_request.id, 0).await?;
    sync_check_runs_for_pull_request(pool, pull_request).await?;
    let checks = pull_check_summaries(pool, &[pull_request.id])
        .await?
        .remove(&pull_request.id)
        .unwrap_or_else(default_checks_summary);
    let review = pull_review_summaries(pool, &[pull_request.id])
        .await?
        .remove(&pull_request.id)
        .unwrap_or_else(default_review_summary);
    let merge_settings = repository_merge_settings(pool, pull_request.repository_id).await?;
    let branch_protection = evaluate_branch_policy(
        pool,
        pull_request.repository_id,
        &pull_request.base_ref,
        actor_user_id,
        BranchPolicyOperation::Merge,
    )
    .await?;
    let approving_review_count = pull_request_approval_count(pool, pull_request.id).await?;
    let mut blockers = Vec::new();

    if !can_write {
        blockers.push(merge_blocker(
            "missing_write_permission",
            "You need write access to change this pull request.",
        ));
    }
    match pull_request.state {
        PullRequestState::Closed => blockers.push(merge_blocker(
            "pull_request_closed",
            "Closed pull requests must be reopened before they can merge.",
        )),
        PullRequestState::Merged => blockers.push(merge_blocker(
            "already_merged",
            "This pull request has already been merged.",
        )),
        PullRequestState::Open => {}
    }
    if pull_request.is_draft {
        blockers.push(merge_blocker(
            "draft",
            "Draft pull requests must be marked ready for review before merging.",
        ));
    }
    if stats.files == 0 {
        blockers.push(merge_blocker(
            "no_diff",
            "There are no changed files or commits to merge.",
        ));
    }
    if !branch_protection.required_status_checks.is_empty() && checks.total_count == 0 {
        blockers.push(merge_blocker(
            "required_checks_missing",
            &format!(
                "Required status checks have not reported yet: {}.",
                branch_protection.required_status_checks.join(", ")
            ),
        ));
    } else if checks.failed_count > 0
        || checks
            .conclusion
            .as_deref()
            .is_some_and(|conclusion| !matches!(conclusion, "success" | "skipped"))
    {
        blockers.push(merge_blocker(
            "required_checks_failed",
            "Required status checks have failed.",
        ));
    } else if checks.total_count > 0 && checks.completed_count < checks.total_count {
        blockers.push(merge_blocker(
            "required_checks_pending",
            "Required status checks are still running.",
        ));
    }
    if review.state == "changes_requested" {
        blockers.push(merge_blocker(
            "changes_requested",
            "A reviewer requested changes.",
        ));
    } else if branch_protection.required_approving_review_count > approving_review_count {
        blockers.push(merge_blocker(
            "required_approvals",
            &format!(
                "{} approving review{} required by branch protection.",
                branch_protection.required_approving_review_count,
                if branch_protection.required_approving_review_count == 1 {
                    " is"
                } else {
                    "s are"
                }
            ),
        ));
    } else if review.required && review.reviewer_count == 0 {
        blockers.push(merge_blocker(
            "review_required",
            "At least one requested review is still required.",
        ));
    }
    for reason in &branch_protection.blocking_reasons {
        if reason == "linear history is required"
            && merge_settings
                .methods
                .iter()
                .any(|method| matches!(method, MergeMethod::Squash | MergeMethod::Rebase))
        {
            continue;
        }
        blockers.push(merge_blocker("branch_policy_blocked", reason));
    }

    let terminal = matches!(
        pull_request.state,
        PullRequestState::Closed | PullRequestState::Merged
    );
    let can_merge = can_write && !terminal && blockers.is_empty();
    let state = if pull_request.state == PullRequestState::Merged {
        "merged"
    } else if pull_request.state == PullRequestState::Closed {
        "closed"
    } else if can_merge {
        "ready"
    } else {
        "blocked"
    };
    let summary = if can_merge {
        let checks_summary = if checks.total_count > 0 {
            format!(
                "{} of {} checks complete",
                checks.completed_count, checks.total_count
            )
        } else {
            "checks not configured".to_owned()
        };
        format!(
            "Ready to merge: {} review state, {}, {} changed files.",
            review.state.replace('_', " "),
            checks_summary,
            stats.files
        )
    } else if blockers.is_empty() {
        "This pull request is not mergeable yet.".to_owned()
    } else {
        blockers
            .iter()
            .map(|blocker| blocker.message.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    };

    Ok(PullRequestMergeability {
        state: state.to_owned(),
        can_merge,
        can_close: can_write && pull_request.state == PullRequestState::Open,
        can_reopen: can_write && pull_request.state == PullRequestState::Closed,
        can_mark_ready: can_write
            && pull_request.state == PullRequestState::Open
            && pull_request.is_draft,
        default_method: merge_settings.default_method,
        methods: merge_settings.methods,
        branch_protection,
        blockers,
        summary,
    })
}

#[derive(Debug, Clone)]
struct RepositoryMergeSettings {
    default_method: MergeMethod,
    methods: Vec<MergeMethod>,
}

async fn repository_merge_settings(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<RepositoryMergeSettings, CollaborationError> {
    let row = sqlx::query(
        r#"
        SELECT allow_squash, allow_merge_commit, allow_rebase, default_method
        FROM repository_merge_settings
        WHERE repository_id = $1
        "#,
    )
    .bind(repository_id)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(RepositoryMergeSettings {
            default_method: MergeMethod::Squash,
            methods: vec![
                MergeMethod::Squash,
                MergeMethod::MergeCommit,
                MergeMethod::Rebase,
            ],
        });
    };

    let mut methods = Vec::new();
    if row.get("allow_squash") {
        methods.push(MergeMethod::Squash);
    }
    if row.get("allow_merge_commit") {
        methods.push(MergeMethod::MergeCommit);
    }
    if row.get("allow_rebase") {
        methods.push(MergeMethod::Rebase);
    }
    if methods.is_empty() {
        methods.push(MergeMethod::Squash);
    }
    let configured_default = merge_method_from_str(row.get::<String, _>("default_method").as_str());
    let default_method = methods
        .iter()
        .find(|method| **method == configured_default)
        .cloned()
        .unwrap_or_else(|| methods[0].clone());
    Ok(RepositoryMergeSettings {
        default_method,
        methods,
    })
}

async fn pull_request_approval_count(
    pool: &PgPool,
    pull_request_id: Uuid,
) -> Result<i64, CollaborationError> {
    Ok(sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(DISTINCT reviewer_user_id)
        FROM pull_request_reviews
        WHERE pull_request_id = $1 AND state = 'approved'
        "#,
    )
    .bind(pull_request_id)
    .fetch_one(pool)
    .await?)
}

fn merge_method_from_str(value: &str) -> MergeMethod {
    match value {
        "merge_commit" => MergeMethod::MergeCommit,
        "rebase" => MergeMethod::Rebase,
        _ => MergeMethod::Squash,
    }
}

fn merge_blocker(code: &str, message: &str) -> PullRequestMergeBlocker {
    PullRequestMergeBlocker {
        code: code.to_owned(),
        message: message.to_owned(),
        severity: "blocking".to_owned(),
    }
}

fn default_merge_commit_title(method: &MergeMethod, pull_request: &PullRequest) -> String {
    match method {
        MergeMethod::Squash => format!("{} (#{})", pull_request.title, pull_request.number),
        MergeMethod::MergeCommit => format!(
            "Merge pull request #{} from {}",
            pull_request.number, pull_request.head_ref
        ),
        MergeMethod::Rebase => format!(
            "Rebase pull request #{} onto {}",
            pull_request.number, pull_request.base_ref
        ),
    }
}

async fn merge_parent_oids(
    tx: &mut Transaction<'_, Postgres>,
    repository_id: Uuid,
    base_commit_id: Option<Uuid>,
    head_commit_id: Option<Uuid>,
) -> Result<Vec<String>, CollaborationError> {
    let mut parent_ids = Vec::new();
    if let Some(base_commit_id) = base_commit_id {
        parent_ids.push(base_commit_id);
    }
    if let Some(head_commit_id) = head_commit_id {
        parent_ids.push(head_commit_id);
    }
    if parent_ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query(
        r#"
        SELECT oid
        FROM commits
        WHERE repository_id = $1 AND id = ANY($2)
        ORDER BY array_position($2, id)
        "#,
    )
    .bind(repository_id)
    .bind(&parent_ids)
    .fetch_all(&mut **tx)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| row.get::<String, _>("oid"))
        .collect())
}

async fn close_linked_issues_for_merge(
    tx: &mut Transaction<'_, Postgres>,
    pull_request: &PullRequest,
    actor_user_id: Uuid,
) -> Result<(), CollaborationError> {
    for number in closing_issue_numbers(pull_request) {
        let issue_row = sqlx::query(
            r#"
            UPDATE issues
            SET state = 'closed',
                closed_by_user_id = $3,
                closed_at = now()
            WHERE repository_id = $1
              AND number = $2
              AND id <> $4
              AND state = 'open'
            RETURNING id
            "#,
        )
        .bind(pull_request.repository_id)
        .bind(number)
        .bind(actor_user_id)
        .bind(pull_request.issue_id)
        .fetch_optional(&mut **tx)
        .await?;
        if let Some(row) = issue_row {
            let issue_id = row.get::<Uuid, _>("id");
            sqlx::query(
                r#"
                INSERT INTO timeline_events (
                    repository_id, issue_id, pull_request_id, actor_user_id, event_type, metadata
                )
                VALUES ($1, $2, NULL, $3, 'closed', $4)
                "#,
            )
            .bind(pull_request.repository_id)
            .bind(issue_id)
            .bind(actor_user_id)
            .bind(json!({
                "reason": "pull_request_merged",
                "pullRequestNumber": pull_request.number,
            }))
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

fn closing_issue_numbers(pull_request: &PullRequest) -> HashSet<i64> {
    let text = [
        pull_request.title.as_str(),
        pull_request.body.as_deref().unwrap_or(""),
    ]
    .join("\n");
    let re = regex::Regex::new(r"(?i)\b(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?)\s+#(\d+)")
        .expect("closing keyword regex should compile");
    re.captures_iter(&text)
        .filter_map(|capture| capture.get(1))
        .filter_map(|number| number.as_str().parse::<i64>().ok())
        .filter(|number| *number != pull_request.number)
        .collect()
}

async fn notify_pull_request_merged(
    pool: &PgPool,
    pull_request: &PullRequest,
    actor_user_id: Uuid,
) -> Result<(), CollaborationError> {
    let mut recipients = HashSet::from([pull_request.author_user_id]);
    let requested_rows = sqlx::query(
        "SELECT requested_user_id FROM pull_request_review_requests WHERE pull_request_id = $1",
    )
    .bind(pull_request.id)
    .fetch_all(pool)
    .await?;
    for row in requested_rows {
        recipients.insert(row.get("requested_user_id"));
    }
    recipients.remove(&actor_user_id);

    for user_id in recipients {
        if !should_deliver_notification(
            pool,
            NotificationDeliveryCheck {
                user_id,
                repository_id: pull_request.repository_id,
                subject_type: "pull_request".to_owned(),
                subject_id: Some(pull_request.id),
                reason: "pull_request_merged".to_owned(),
                repository_event: Some(RepositoryWatchEvent::PullRequests),
                actor_user_id: Some(actor_user_id),
                participating: true,
                direct: false,
            },
        )
        .await
        .map_err(|error| match error {
            super::notifications::NotificationError::Sqlx(error) => CollaborationError::Sqlx(error),
            super::notifications::NotificationError::NotFound => {
                CollaborationError::PullRequestNotFound
            }
            super::notifications::NotificationError::Validation(_) => {
                CollaborationError::PullRequestNotFound
            }
        })? {
            continue;
        }
        create_notification(
            pool,
            CreateNotification {
                user_id,
                repository_id: Some(pull_request.repository_id),
                subject_type: "pull_request".to_owned(),
                subject_id: Some(pull_request.id),
                title: format!("Pull request #{} was merged", pull_request.number),
                reason: "pull_request_merged".to_owned(),
            },
        )
        .await
        .map_err(|error| match error {
            super::notifications::NotificationError::Sqlx(error) => CollaborationError::Sqlx(error),
            super::notifications::NotificationError::NotFound => {
                CollaborationError::PullRequestNotFound
            }
            super::notifications::NotificationError::Validation(_) => {
                CollaborationError::PullRequestNotFound
            }
        })?;
    }
    Ok(())
}

async fn pull_request_detail_participants(
    pool: &PgPool,
    pull_request_id: Uuid,
) -> Result<Vec<IssueListUser>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT ON (users.id)
               users.id, COALESCE(users.username, users.email) AS login,
               users.display_name, users.avatar_url
        FROM users
        WHERE users.id IN (
            SELECT author_user_id FROM pull_requests WHERE id = $1
            UNION
            SELECT requested_user_id FROM pull_request_review_requests WHERE pull_request_id = $1
            UNION
            SELECT reviewer_user_id FROM pull_request_reviews WHERE pull_request_id = $1
            UNION
            SELECT author_user_id FROM comments WHERE pull_request_id = $1
            UNION
            SELECT actor_user_id FROM timeline_events
            WHERE pull_request_id = $1 AND actor_user_id IS NOT NULL
        )
        ORDER BY users.id, lower(COALESCE(users.username, users.email))
        "#,
    )
    .bind(pull_request_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| IssueListUser {
            id: row.get("id"),
            login: row.get("login"),
            display_name: row.get("display_name"),
            avatar_url: row.get("avatar_url"),
        })
        .collect())
}

async fn pull_request_latest_reviews(
    pool: &PgPool,
    pull_request_id: Uuid,
) -> Result<Vec<PullRequestReviewStatus>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT ON (reviews.reviewer_user_id)
               users.id, COALESCE(users.username, users.email) AS login,
               users.display_name, users.avatar_url, reviews.state, reviews.submitted_at
        FROM pull_request_reviews reviews
        JOIN users ON users.id = reviews.reviewer_user_id
        WHERE reviews.pull_request_id = $1
        ORDER BY reviews.reviewer_user_id, reviews.submitted_at DESC
        "#,
    )
    .bind(pull_request_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| PullRequestReviewStatus {
            reviewer: IssueListUser {
                id: row.get("id"),
                login: row.get("login"),
                display_name: row.get("display_name"),
                avatar_url: row.get("avatar_url"),
            },
            state: row.get("state"),
            submitted_at: row.get("submitted_at"),
        })
        .collect())
}

async fn linked_issue_hints(
    pool: &PgPool,
    issue_ids: &[Uuid],
    repository: &Repository,
) -> Result<HashMap<Uuid, Vec<LinkedIssueHint>>, CollaborationError> {
    let mut by_issue: HashMap<Uuid, Vec<LinkedIssueHint>> = HashMap::new();
    if issue_ids.is_empty() {
        return Ok(by_issue);
    }
    let rows = sqlx::query(
        r#"
        SELECT issue_cross_references.source_issue_id, issues.number, issues.state, issues.title
        FROM issue_cross_references
        JOIN issues ON issues.id = issue_cross_references.target_issue_id
        WHERE issue_cross_references.source_issue_id = ANY($1)
        ORDER BY issues.updated_at DESC, issues.number DESC
        "#,
    )
    .bind(issue_ids)
    .fetch_all(pool)
    .await?;
    for row in rows {
        let issue_id = row.get("source_issue_id");
        let number = row.get("number");
        by_issue.entry(issue_id).or_default().push(LinkedIssueHint {
            number,
            state: row.get("state"),
            title: row.get("title"),
            href: format!(
                "/{}/{}/issues/{}",
                repository.owner_login, repository.name, number
            ),
        });
    }
    Ok(by_issue)
}

async fn pull_review_summaries(
    pool: &PgPool,
    pull_request_ids: &[Uuid],
) -> Result<HashMap<Uuid, PullRequestReviewSummary>, CollaborationError> {
    if pull_request_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let review_rows = sqlx::query(
        r#"
        SELECT pull_request_id,
               count(*) AS reviewer_count,
               bool_or(state = 'changes_requested') AS has_changes_requested,
               bool_or(state = 'approved') AS has_approved,
               bool_or(state = 'commented') AS has_commented
        FROM pull_request_reviews
        WHERE pull_request_id = ANY($1)
        GROUP BY pull_request_id
        "#,
    )
    .bind(pull_request_ids)
    .fetch_all(pool)
    .await?;
    let request_rows = sqlx::query(
        r#"
        SELECT review_requests.pull_request_id, users.id,
               COALESCE(users.username, users.email) AS login,
               users.display_name, users.avatar_url
        FROM pull_request_review_requests review_requests
        JOIN users ON users.id = review_requests.requested_user_id
        WHERE review_requests.pull_request_id = ANY($1)
        ORDER BY lower(COALESCE(users.username, users.email))
        "#,
    )
    .bind(pull_request_ids)
    .fetch_all(pool)
    .await?;

    let mut summaries = HashMap::new();
    for row in review_rows {
        let pull_request_id = row.get("pull_request_id");
        let has_changes_requested: bool = row.get("has_changes_requested");
        let has_approved: bool = row.get("has_approved");
        let has_commented: bool = row.get("has_commented");
        let state = if has_changes_requested {
            "changes_requested"
        } else if has_approved {
            "approved"
        } else if has_commented {
            "commented"
        } else {
            "pending"
        };
        summaries.insert(
            pull_request_id,
            PullRequestReviewSummary {
                state: state.to_owned(),
                required: false,
                requested_reviewers: Vec::new(),
                reviewer_count: row.get("reviewer_count"),
            },
        );
    }

    for row in request_rows {
        let pull_request_id = row.get("pull_request_id");
        let summary = summaries
            .entry(pull_request_id)
            .or_insert_with(default_review_summary);
        summary.required = true;
        if summary.state == "none" {
            summary.state = "required".to_owned();
        }
        summary.requested_reviewers.push(IssueListUser {
            id: row.get("id"),
            login: row.get("login"),
            display_name: row.get("display_name"),
            avatar_url: row.get("avatar_url"),
        });
    }

    Ok(summaries)
}

async fn pull_check_summaries(
    pool: &PgPool,
    pull_request_ids: &[Uuid],
) -> Result<HashMap<Uuid, PullRequestChecksSummary>, CollaborationError> {
    if pull_request_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT pull_request_id, status, conclusion, total_count, completed_count, failed_count
        FROM pull_request_checks_summary
        WHERE pull_request_id = ANY($1)
        "#,
    )
    .bind(pull_request_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get("pull_request_id"),
                PullRequestChecksSummary {
                    status: row.get("status"),
                    conclusion: row.get("conclusion"),
                    total_count: row.get("total_count"),
                    completed_count: row.get("completed_count"),
                    failed_count: row.get("failed_count"),
                },
            )
        })
        .collect())
}

pub async fn pull_request_checks_view(
    pool: &PgPool,
    repository_id: Uuid,
    number: i64,
    actor_user_id: Option<Uuid>,
) -> Result<PullRequestChecksView, CollaborationError> {
    let pull_request = pull_request_by_repository_number(pool, repository_id, number).await?;
    sync_check_runs_for_pull_request(pool, &pull_request).await?;
    let detail =
        pull_request_detail_view_for_viewer(pool, repository_id, number, actor_user_id).await?;
    let required_status_checks = detail
        .mergeability
        .branch_protection
        .required_status_checks
        .clone();
    let check_runs = pull_request_check_runs(
        pool,
        repository_id,
        &detail.repository.owner_login,
        &detail.repository.name,
        &pull_request,
        &required_status_checks,
        actor_user_id.is_some(),
    )
    .await?;

    Ok(PullRequestChecksView {
        repository: detail.repository,
        pull_request: PullRequestChecksPullRequest {
            id: detail.id,
            number: detail.number,
            title: detail.title,
            state: detail.state,
            head_ref: detail.head_ref,
            base_ref: detail.base_ref,
            head_sha: pull_request_head_sha(pool, &pull_request).await?,
            href: detail.href,
        },
        summary: detail.checks,
        required_status_checks,
        check_runs,
        can_rerun: actor_user_id.is_some(),
    })
}

pub async fn sync_check_runs_for_pull_request(
    pool: &PgPool,
    pull_request: &PullRequest,
) -> Result<(), CollaborationError> {
    let Some(head_sha) = pull_request_head_sha(pool, pull_request).await? else {
        return Ok(());
    };
    sync_check_runs_for_head_sha(pool, pull_request.repository_id, &head_sha).await?;
    let summary =
        check_run_summary_for_head_sha(pool, pull_request.repository_id, &head_sha).await?;
    if summary.total_count == 0 {
        return Ok(());
    }
    sqlx::query(
        r#"
        INSERT INTO pull_request_checks_summary (
            pull_request_id, status, conclusion, total_count, completed_count, failed_count
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (pull_request_id) DO UPDATE
        SET status = EXCLUDED.status,
            conclusion = EXCLUDED.conclusion,
            total_count = EXCLUDED.total_count,
            completed_count = EXCLUDED.completed_count,
            failed_count = EXCLUDED.failed_count,
            updated_at = now()
        "#,
    )
    .bind(pull_request.id)
    .bind(&summary.status)
    .bind(&summary.conclusion)
    .bind(summary.total_count)
    .bind(summary.completed_count)
    .bind(summary.failed_count)
    .execute(pool)
    .await?;
    Ok(())
}

async fn sync_check_runs_for_head_sha(
    pool: &PgPool,
    repository_id: Uuid,
    head_sha: &str,
) -> Result<(), CollaborationError> {
    sqlx::query(
        r#"
        INSERT INTO check_runs (
            repository_id, workflow_run_id, workflow_job_id, head_sha, name, status, conclusion,
            started_at, completed_at, output_title, output_summary, annotations_count
        )
        SELECT workflow_runs.repository_id,
               workflow_runs.id,
               workflow_jobs.id,
               workflow_runs.head_sha,
               workflow_jobs.name,
               CASE
                WHEN workflow_jobs.status = 'completed' THEN 'completed'
                WHEN workflow_jobs.status IN ('in_progress', 'cancelled') THEN 'in_progress'
                ELSE 'queued'
               END,
               CASE
                WHEN workflow_jobs.status = 'cancelled' THEN 'cancelled'
                ELSE workflow_jobs.conclusion
               END,
               workflow_jobs.started_at,
               workflow_jobs.completed_at,
               workflow_jobs.name,
               CASE
                WHEN workflow_jobs.conclusion = 'failure' THEN 'Job failed. Review annotations and logs.'
                WHEN workflow_jobs.conclusion = 'success' THEN 'Job completed successfully.'
                ELSE NULL
               END,
               COALESCE(annotation_counts.count, 0)
        FROM workflow_jobs
        JOIN workflow_runs ON workflow_runs.id = workflow_jobs.run_id
        LEFT JOIN LATERAL (
            SELECT count(*)::bigint AS count
            FROM workflow_annotations
            WHERE workflow_annotations.job_id = workflow_jobs.id
        ) annotation_counts ON true
        WHERE workflow_runs.repository_id = $1
          AND workflow_runs.head_sha = $2
        ON CONFLICT (repository_id, workflow_job_id) WHERE workflow_job_id IS NOT NULL
        DO UPDATE SET status = EXCLUDED.status,
            conclusion = EXCLUDED.conclusion,
            started_at = EXCLUDED.started_at,
            completed_at = EXCLUDED.completed_at,
            output_title = EXCLUDED.output_title,
            output_summary = EXCLUDED.output_summary,
            annotations_count = EXCLUDED.annotations_count
        "#,
    )
    .bind(repository_id)
    .bind(head_sha)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO check_annotations (
            check_run_id, workflow_annotation_id, path, start_line, end_line, level, message, created_at
        )
        SELECT check_runs.id,
               workflow_annotations.id,
               workflow_annotations.path,
               workflow_annotations.start_line,
               workflow_annotations.end_line,
               workflow_annotations.annotation_level,
               workflow_annotations.message,
               workflow_annotations.created_at
        FROM check_runs
        JOIN workflow_annotations ON workflow_annotations.job_id = check_runs.workflow_job_id
        WHERE check_runs.repository_id = $1
          AND check_runs.head_sha = $2
        ON CONFLICT (check_run_id, workflow_annotation_id) WHERE workflow_annotation_id IS NOT NULL
        DO UPDATE SET path = EXCLUDED.path,
            start_line = EXCLUDED.start_line,
            end_line = EXCLUDED.end_line,
            level = EXCLUDED.level,
            message = EXCLUDED.message
        "#,
    )
    .bind(repository_id)
    .bind(head_sha)
    .execute(pool)
    .await?;
    Ok(())
}

async fn pull_request_check_runs(
    pool: &PgPool,
    repository_id: Uuid,
    owner: &str,
    repo: &str,
    pull_request: &PullRequest,
    required: &[String],
    can_rerun: bool,
) -> Result<Vec<PullRequestCheckRun>, CollaborationError> {
    let Some(head_sha) = pull_request_head_sha(pool, pull_request).await? else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query(
        r#"
        SELECT id, workflow_run_id, workflow_job_id, name, status, conclusion, started_at,
               completed_at, output_title, output_summary, annotations_count
        FROM check_runs
        WHERE repository_id = $1 AND head_sha = $2
        ORDER BY CASE status WHEN 'in_progress' THEN 0 WHEN 'queued' THEN 1 ELSE 2 END,
                 lower(name)
        "#,
    )
    .bind(repository_id)
    .bind(&head_sha)
    .fetch_all(pool)
    .await?;
    let check_run_ids = rows.iter().map(|row| row.get("id")).collect::<Vec<Uuid>>();
    let mut annotations_by_check = check_annotations_by_check_run(pool, &check_run_ids).await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            let run_id: Option<Uuid> = row.get("workflow_run_id");
            let job_id: Option<Uuid> = row.get("workflow_job_id");
            PullRequestCheckRun {
                id,
                name: row.get("name"),
                status: row.get("status"),
                conclusion: row.get("conclusion"),
                required: required
                    .iter()
                    .any(|check| check == row.get::<String, _>("name").as_str()),
                started_at: row.get("started_at"),
                completed_at: row.get("completed_at"),
                output_title: row.get("output_title"),
                output_summary: row.get("output_summary"),
                annotations_count: row.get("annotations_count"),
                details_href: match (run_id, job_id) {
                    (Some(_), Some(job_id)) => Some(format!(
                        "/{owner}/{repo}/actions/runs/{}/jobs/{job_id}",
                        run_id.unwrap()
                    )),
                    _ => None,
                },
                rerun_href: if can_rerun {
                    Some(format!(
                        "/{owner}/{repo}/pull/{}/checks/{id}/rerun",
                        pull_request.number
                    ))
                } else {
                    None
                },
                annotations: annotations_by_check.remove(&id).unwrap_or_default(),
            }
        })
        .collect())
}

async fn check_annotations_by_check_run(
    pool: &PgPool,
    check_run_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<PullRequestCheckAnnotation>>, CollaborationError> {
    if check_run_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT check_run_id, id, path, start_line, end_line, level, message, created_at
        FROM check_annotations
        WHERE check_run_id = ANY($1)
        ORDER BY created_at, path, start_line
        "#,
    )
    .bind(check_run_ids)
    .fetch_all(pool)
    .await?;
    let mut grouped: HashMap<Uuid, Vec<PullRequestCheckAnnotation>> = HashMap::new();
    for row in rows {
        grouped
            .entry(row.get("check_run_id"))
            .or_default()
            .push(PullRequestCheckAnnotation {
                id: row.get("id"),
                path: row.get("path"),
                start_line: row.get("start_line"),
                end_line: row.get("end_line"),
                level: row.get("level"),
                message: row.get("message"),
                created_at: row.get("created_at"),
            });
    }
    Ok(grouped)
}

async fn check_run_summary_for_head_sha(
    pool: &PgPool,
    repository_id: Uuid,
    head_sha: &str,
) -> Result<PullRequestChecksSummary, CollaborationError> {
    let row = sqlx::query(
        r#"
        SELECT CASE
                WHEN count(*) FILTER (WHERE status IN ('queued', 'in_progress')) > 0 THEN 'running'
                WHEN count(*) = 0 THEN 'pending'
                ELSE 'completed'
               END AS status,
               CASE
                WHEN count(*) = 0 THEN NULL
                WHEN count(*) FILTER (WHERE conclusion = 'failure') > 0 THEN 'failure'
                WHEN count(*) FILTER (WHERE conclusion = 'cancelled') > 0 THEN 'cancelled'
                WHEN count(*) FILTER (WHERE conclusion IN ('success', 'skipped', 'neutral')) = count(*) THEN 'success'
                ELSE NULL
               END AS conclusion,
               count(*)::bigint AS total_count,
               count(*) FILTER (WHERE status = 'completed')::bigint AS completed_count,
               count(*) FILTER (WHERE conclusion = 'failure')::bigint AS failed_count
        FROM check_runs
        WHERE repository_id = $1 AND head_sha = $2
        "#,
    )
    .bind(repository_id)
    .bind(head_sha)
    .fetch_one(pool)
    .await?;
    Ok(PullRequestChecksSummary {
        status: row.get("status"),
        conclusion: row.get("conclusion"),
        total_count: row.get("total_count"),
        completed_count: row.get("completed_count"),
        failed_count: row.get("failed_count"),
    })
}

async fn pull_request_head_sha(
    pool: &PgPool,
    pull_request: &PullRequest,
) -> Result<Option<String>, CollaborationError> {
    let row = sqlx::query(
        r#"
        SELECT COALESCE(head_commit.oid, ref_commit.oid) AS head_sha
        FROM pull_requests
        LEFT JOIN LATERAL (
            SELECT commits.oid
            FROM pull_request_commits
            JOIN commits ON commits.id = pull_request_commits.commit_id
            WHERE pull_request_commits.pull_request_id = pull_requests.id
            ORDER BY commits.committed_at DESC
            LIMIT 1
        ) head_commit ON true
        LEFT JOIN repository_git_refs ON repository_git_refs.repository_id = pull_requests.repository_id
             AND repository_git_refs.name = ('refs/heads/' || pull_requests.head_ref)
        LEFT JOIN commits ref_commit ON ref_commit.id = repository_git_refs.target_commit_id
        WHERE pull_requests.id = $1
        "#,
    )
    .bind(pull_request.id)
    .fetch_one(pool)
    .await?;
    Ok(row.get("head_sha"))
}

async fn pull_request_by_repository_number(
    pool: &PgPool,
    repository_id: Uuid,
    number: i64,
) -> Result<PullRequest, CollaborationError> {
    let row = sqlx::query(
        r#"
        SELECT id, repository_id, issue_id, number, title, body, state, author_user_id,
               head_ref, base_ref, head_repository_id, base_repository_id, merge_commit_id,
               merged_by_user_id, merged_at, closed_at, created_at, updated_at, is_draft
        FROM pull_requests
        WHERE repository_id = $1 AND number = $2
        "#,
    )
    .bind(repository_id)
    .bind(number)
    .fetch_optional(pool)
    .await?
    .ok_or(CollaborationError::PullRequestNotFound)?;
    pull_request_from_row(row)
}

async fn pull_task_progress(
    pool: &PgPool,
    pull_request_ids: &[Uuid],
) -> Result<HashMap<Uuid, PullRequestTaskProgress>, CollaborationError> {
    if pull_request_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT pull_request_id, completed_count, total_count
        FROM pull_request_task_progress
        WHERE pull_request_id = ANY($1)
        "#,
    )
    .bind(pull_request_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get("pull_request_id"),
                PullRequestTaskProgress {
                    completed: row.get("completed_count"),
                    total: row.get("total_count"),
                },
            )
        })
        .collect())
}

async fn pull_author_roles(
    pool: &PgPool,
    repository_id: Uuid,
    pull_request_ids: &[Uuid],
) -> Result<HashMap<Uuid, String>, CollaborationError> {
    if pull_request_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT pull_requests.id AS pull_request_id,
               COALESCE(repository_permissions.role, 'contributor') AS role
        FROM pull_requests
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = $1
         AND repository_permissions.user_id = pull_requests.author_user_id
        WHERE pull_requests.id = ANY($2)
        "#,
    )
    .bind(repository_id)
    .bind(pull_request_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.get("pull_request_id"), row.get("role")))
        .collect())
}

async fn repository_pull_preferences(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
) -> Result<PullRequestListPreferences, CollaborationError> {
    let row = sqlx::query(
        r#"
        SELECT dismissed_contributor_banner_at
        FROM repository_pull_preferences
        WHERE repository_id = $1 AND user_id = $2
        "#,
    )
    .bind(repository_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(match row {
        Some(row) => {
            let dismissed_at: Option<DateTime<Utc>> = row.get("dismissed_contributor_banner_at");
            PullRequestListPreferences {
                dismissed_contributor_banner: dismissed_at.is_some(),
                dismissed_contributor_banner_at: dismissed_at,
            }
        }
        None => PullRequestListPreferences {
            dismissed_contributor_banner: false,
            dismissed_contributor_banner_at: None,
        },
    })
}

pub async fn save_repository_pull_preferences(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
    dismissed_contributor_banner: bool,
) -> Result<PullRequestListPreferences, CollaborationError> {
    require_repository_read(pool, repository_id, user_id).await?;
    let dismissed_at = dismissed_contributor_banner.then(Utc::now);
    sqlx::query(
        r#"
        INSERT INTO repository_pull_preferences (
            repository_id,
            user_id,
            dismissed_contributor_banner_at
        )
        VALUES ($1, $2, $3)
        ON CONFLICT (repository_id, user_id)
        DO UPDATE SET dismissed_contributor_banner_at = EXCLUDED.dismissed_contributor_banner_at
        "#,
    )
    .bind(repository_id)
    .bind(user_id)
    .bind(dismissed_at)
    .execute(pool)
    .await?;

    repository_pull_preferences(pool, repository_id, user_id).await
}

fn default_review_summary() -> PullRequestReviewSummary {
    PullRequestReviewSummary {
        state: "none".to_owned(),
        required: false,
        requested_reviewers: Vec::new(),
        reviewer_count: 0,
    }
}

fn default_checks_summary() -> PullRequestChecksSummary {
    PullRequestChecksSummary {
        status: "pending".to_owned(),
        conclusion: None,
        total_count: 0,
        completed_count: 0,
        failed_count: 0,
    }
}

fn fallback_user(user_id: Uuid) -> IssueListUser {
    IssueListUser {
        id: user_id,
        login: "unknown".to_owned(),
        display_name: None,
        avatar_url: None,
    }
}

fn search_text_from_pull_query(query: &str) -> String {
    pull_query_terms(query)
        .into_iter()
        .filter(|term| {
            !matches!(
                term.as_str(),
                "is:pr" | "is:pull-request" | "is:open" | "is:closed" | "is:merged"
            ) && !term.starts_with("state:")
                && !term.starts_with("author:")
                && !term.starts_with("label:")
                && !term.starts_with("milestone:")
                && term != "no:milestone"
                && !term.starts_with("assignee:")
                && term != "no:assignee"
                && !term.starts_with("project:")
                && !term.starts_with("review:")
                && !term.starts_with("review-requested:")
                && !term.starts_with("reviewed-by:")
                && !term.starts_with("checks:")
                && !term.starts_with("sort:")
                && !term.starts_with("order:")
        })
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_owned()
}

fn pull_query_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut rest = query.trim();
    while !rest.is_empty() {
        let token_end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let token = &rest[..token_end];
        if let Some(quote_index) = token.find(":\"") {
            let prefix_length = quote_index + 2;
            let quoted_rest = &rest[prefix_length..];
            if let Some(end_quote) = quoted_rest.find('"') {
                terms.push(format!(
                    "{}{}",
                    &token[..prefix_length],
                    &quoted_rest[..=end_quote]
                ));
                rest = quoted_rest[end_quote + 1..].trim_start();
            } else {
                terms.push(rest.to_owned());
                rest = "";
            }
        } else {
            terms.push(token.to_owned());
            rest = rest[token_end..].trim_start();
        }
    }
    terms
}

pub fn pull_sort_options() -> Vec<String> {
    [
        "best-match",
        "updated-desc",
        "updated-asc",
        "created-desc",
        "created-asc",
        "comments-desc",
        "comments-asc",
        "reactions-desc",
        "reactions-thumbs_up-desc",
        "reactions-thumbs_down-desc",
        "reactions-laugh-desc",
        "reactions-hooray-desc",
        "reactions-confused-desc",
        "reactions-heart-desc",
        "reactions-rocket-desc",
        "reactions-eyes-desc",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect()
}

async fn index_pull_request_search_document(
    pool: &PgPool,
    pull_request: &PullRequest,
    actor_user_id: Uuid,
) -> Result<(), CollaborationError> {
    let repository = get_repository(pool, pull_request.repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    let author_login = user_login(pool, pull_request.author_user_id).await?;
    let body = [
        pull_request.body.as_deref().unwrap_or(""),
        pull_request.head_ref.as_str(),
        pull_request.base_ref.as_str(),
    ]
    .into_iter()
    .filter(|part| !part.trim().is_empty())
    .collect::<Vec<_>>()
    .join("\n");

    upsert_search_document(
        pool,
        actor_user_id,
        UpsertSearchDocument {
            repository_id: Some(repository.id),
            owner_user_id: repository.owner_user_id,
            owner_organization_id: repository.owner_organization_id,
            kind: SearchDocumentKind::PullRequest,
            resource_id: format!("{}:{}", repository.id, pull_request.number),
            title: pull_request.title.clone(),
            body: Some(body),
            path: None,
            language: None,
            branch: Some(pull_request.head_ref.clone()),
            visibility: repository.visibility,
            metadata: json!({
                "number": pull_request.number,
                "state": pull_request.state.as_str(),
                "headRef": pull_request.head_ref,
                "baseRef": pull_request.base_ref,
                "labels": [],
                "authorLogin": author_login,
                "createdAt": pull_request.created_at,
                "updatedAt": pull_request.updated_at,
                "href": format!("/{}/{}/pull/{}", repository.owner_login, repository.name, pull_request.number),
            }),
        },
    )
    .await
    .map_err(search_error_to_collaboration)?;

    Ok(())
}
