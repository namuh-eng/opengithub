use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use crate::api_types::ListEnvelope;

use super::{
    issues::{
        append_timeline_event, insert_issue_with_number, issue_from_row, next_issue_number,
        reaction_summaries, repository_for_actor, search_error_to_collaboration, user_login,
        CollaborationError, CreateComment, CreateIssue, Issue, IssueListLabel,
        IssueListMetadataOption, IssueListMilestone, IssueListUser, IssueState, ReactionSummary,
        TimelineEvent,
    },
    markdown::{render_markdown, RenderMarkdownInput},
    notifications::{create_notification, CreateNotification},
    permissions::RepositoryRole,
    repositories::{
        get_repository, repository_permission_for_user, Repository, RepositoryVisibility,
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
    notify_pull_request_participants(pool, &pull_request, &assignee_user_ids, &reviewer_user_ids)
        .await?;
    insert_pull_request_audit_event(pool, &pull_request, input.actor_user_id).await?;
    index_pull_request_search_document(pool, &pull_request, repository.created_by_user_id).await?;

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
                    AND lower(COALESCE(users.username, users.email)) = lower($4)
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
                    AND lower(COALESCE(users.username, users.email)) = lower($8)
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
    index_pull_request_search_document(pool, &pull_request, input.actor_user_id).await?;
    Ok(pull_request)
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
    notify_pull_request_participants(pool, &pull_request, &input.assignee_user_ids, &[]).await?;
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
        notify_pull_request_participants(pool, &pull_request, &[], &added).await?;
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
    sqlx::query(
        r#"
        INSERT INTO pull_request_subscriptions (pull_request_id, user_id, subscribed, reason)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (pull_request_id, user_id)
        DO UPDATE SET subscribed = EXCLUDED.subscribed, reason = EXCLUDED.reason
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
        RepositoryRole::Write => permission.role.can_write(),
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
        RepositoryRole::Write => permission.role.can_write(),
        RepositoryRole::Admin => permission.role.can_admin(),
        RepositoryRole::Owner => permission.role == RepositoryRole::Owner,
    };

    if allowed {
        Ok(Some(permission.role.as_str().to_owned()))
    } else {
        Err(CollaborationError::RepositoryAccessDenied)
    }
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
                    AND lower(COALESCE(users.username, users.email)) = lower($4)
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
                    AND lower(COALESCE(users.username, users.email)) = lower($8)
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
        });
    };
    let explicit = sqlx::query(
        r#"
        SELECT subscribed, reason
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
    let checks = pull_check_summaries(pool, &[pull_request.id])
        .await?
        .remove(&pull_request.id)
        .unwrap_or_else(default_checks_summary);
    let review = pull_review_summaries(pool, &[pull_request.id])
        .await?
        .remove(&pull_request.id)
        .unwrap_or_else(default_review_summary);
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
    if checks.failed_count > 0
        || checks
            .conclusion
            .as_deref()
            .is_some_and(|conclusion| !matches!(conclusion, "success" | "skipped"))
    {
        blockers.push(merge_blocker(
            "checks_failed",
            "Required checks have failed.",
        ));
    } else if checks.total_count > 0 && checks.completed_count < checks.total_count {
        blockers.push(merge_blocker(
            "checks_pending",
            "Required checks are still running.",
        ));
    }
    if review.state == "changes_requested" {
        blockers.push(merge_blocker(
            "changes_requested",
            "A reviewer requested changes.",
        ));
    } else if review.required && review.reviewer_count == 0 {
        blockers.push(merge_blocker(
            "review_required",
            "At least one requested review is still required.",
        ));
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
        default_method: MergeMethod::Squash,
        methods: vec![
            MergeMethod::Squash,
            MergeMethod::MergeCommit,
            MergeMethod::Rebase,
        ],
        blockers,
        summary,
    })
}

fn merge_blocker(code: &str, message: &str) -> PullRequestMergeBlocker {
    PullRequestMergeBlocker {
        code: code.to_owned(),
        message: message.to_owned(),
        severity: "blocking".to_owned(),
    }
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
