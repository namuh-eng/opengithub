use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api_types::ListEnvelope;

use super::{
    markdown::{render_markdown, RenderMarkdownInput},
    notifications::{
        create_notification, should_deliver_notification, CreateNotification,
        NotificationDeliveryCheck,
    },
    permissions::RepositoryRole,
    projects::{run_project_item_automation, ProjectAutomationEvent, ProjectAutomationInput},
    repositories::{
        get_repository, get_repository_by_owner_name, repository_permission_for_user, Repository,
        RepositoryVisibility, RepositoryWatchEvent,
    },
    search::{upsert_search_document, SearchDocumentKind, SearchError, UpsertSearchDocument},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueState {
    #[default]
    Open,
    Closed,
}

impl IssueState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
        }
    }
}

impl TryFrom<&str> for IssueState {
    type Error = CollaborationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "open" => Ok(Self::Open),
            "closed" => Ok(Self::Closed),
            other => Err(CollaborationError::InvalidState(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Label {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Milestone {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub due_on: Option<DateTime<Utc>>,
    pub state: IssueState,
    pub created_by_user_id: Uuid,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Issue {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: IssueState,
    pub author_user_id: Uuid,
    pub milestone_id: Option<Uuid>,
    pub locked: bool,
    pub closed_by_user_id: Option<Uuid>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueListLabel {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueListUser {
    pub id: Uuid,
    pub login: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueListMilestone {
    pub id: Uuid,
    pub title: String,
    pub state: IssueState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LinkedPullRequestHint {
    pub number: i64,
    pub state: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueListItem {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub repository_owner: String,
    pub repository_name: String,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: IssueState,
    pub author: IssueListUser,
    pub labels: Vec<IssueListLabel>,
    pub milestone: Option<IssueListMilestone>,
    pub assignees: Vec<IssueListUser>,
    pub comment_count: i64,
    pub linked_pull_request: Option<LinkedPullRequestHint>,
    pub href: String,
    pub locked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueAttachmentMetadata {
    pub id: Uuid,
    pub file_name: String,
    pub byte_size: i64,
    pub content_type: Option<String>,
    pub storage_status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueSubscriptionState {
    pub subscribed: bool,
    pub reason: String,
    pub custom_events: Vec<String>,
    pub can_customize: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReactionSummary {
    pub content: ReactionContent,
    pub count: i64,
    pub viewer_reacted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueDetailView {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub repository_owner: String,
    pub repository_name: String,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub body_html: String,
    pub state: IssueState,
    pub author: IssueListUser,
    pub labels: Vec<IssueListLabel>,
    pub milestone: Option<IssueListMilestone>,
    pub assignees: Vec<IssueListUser>,
    pub participants: Vec<IssueListUser>,
    pub attachments: Vec<IssueAttachmentMetadata>,
    pub comment_count: i64,
    pub linked_pull_request: Option<LinkedPullRequestHint>,
    pub href: String,
    pub locked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub viewer_permission: Option<String>,
    pub repository: IssueListRepository,
    pub subscription: IssueSubscriptionState,
    pub reactions: Vec<ReactionSummary>,
    pub metadata_options: IssueDetailMetadataOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueDetailMetadataOptions {
    pub labels: Vec<IssueListLabel>,
    pub assignees: Vec<IssueListUser>,
    pub milestones: Vec<IssueListMilestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueTimelineItem {
    pub id: Uuid,
    pub event_type: String,
    pub actor: Option<IssueListUser>,
    pub comment: Option<IssueTimelineComment>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueTimelineComment {
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
pub struct IssueListCounts {
    pub open: i64,
    pub closed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueListFilters {
    pub query: String,
    pub state: IssueState,
    pub author: Option<String>,
    pub excluded_author: Option<String>,
    pub labels: Vec<String>,
    pub excluded_labels: Vec<String>,
    pub no_labels: bool,
    pub milestone: Option<String>,
    pub no_milestone: bool,
    pub assignee: Option<String>,
    pub no_assignee: bool,
    pub project: Option<String>,
    pub issue_type: Option<String>,
    pub sort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueListFilterOptions {
    pub labels: Vec<IssueListLabel>,
    pub users: Vec<IssueListUser>,
    pub milestones: Vec<IssueListMilestone>,
    pub projects: Vec<IssueListMetadataOption>,
    pub issue_types: Vec<IssueListMetadataOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueListMetadataOption {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub count: i64,
    pub disabled_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueListView {
    pub items: Vec<IssueListItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub open_count: i64,
    pub closed_count: i64,
    pub counts: IssueListCounts,
    pub filters: IssueListFilters,
    pub filter_options: IssueListFilterOptions,
    pub viewer_permission: Option<String>,
    pub repository: IssueListRepository,
    pub preferences: IssueListPreferences,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GlobalIssueScope {
    #[default]
    Created,
    Assigned,
    Mentioned,
}

impl GlobalIssueScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Assigned => "assigned",
            Self::Mentioned => "mentioned",
        }
    }
}

impl TryFrom<&str> for GlobalIssueScope {
    type Error = CollaborationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "created" => Ok(Self::Created),
            "assigned" => Ok(Self::Assigned),
            "mentioned" => Ok(Self::Mentioned),
            other => Err(CollaborationError::InvalidIssueFilter(format!(
                "scope must be created, assigned, or mentioned; got {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalIssueListQuery {
    pub scope: GlobalIssueScope,
    pub query: Option<String>,
    pub state: Option<IssueState>,
    pub repository: Option<String>,
    pub labels: Vec<String>,
    pub milestone: Option<String>,
    pub project: Option<String>,
    pub sort: String,
}

impl Default for GlobalIssueListQuery {
    fn default() -> Self {
        Self {
            scope: GlobalIssueScope::Created,
            query: Some("is:issue state:open".to_owned()),
            state: Some(IssueState::Open),
            repository: None,
            labels: Vec::new(),
            milestone: None,
            project: None,
            sort: "updated-desc".to_owned(),
        }
    }
}

pub fn issue_sort_options() -> Vec<&'static str> {
    vec![
        "updated-desc",
        "updated-asc",
        "created-desc",
        "created-asc",
        "comments-desc",
        "comments-asc",
        "best-match",
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalIssueCounts {
    pub created: i64,
    pub assigned: i64,
    pub mentioned: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalIssueFilters {
    pub scope: GlobalIssueScope,
    pub query: String,
    pub state: Option<IssueState>,
    pub repository: Option<String>,
    pub labels: Vec<String>,
    pub milestone: Option<String>,
    pub project: Option<String>,
    pub sort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalIssueRepositoryOption {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub full_name: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalIssueFilterOptions {
    pub repositories: Vec<GlobalIssueRepositoryOption>,
    pub labels: Vec<IssueListLabel>,
    pub milestones: Vec<IssueListMilestone>,
    pub projects: Vec<IssueListMetadataOption>,
    pub sort_options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalIssueListView {
    pub items: Vec<IssueListItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub counts: GlobalIssueCounts,
    pub filters: GlobalIssueFilters,
    pub filter_options: GlobalIssueFilterOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueListRepository {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub visibility: RepositoryVisibility,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueListPreferences {
    pub dismissed_contributor_banner: bool,
    pub dismissed_contributor_banner_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueTemplate {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub title_prefill: Option<String>,
    pub body: String,
    pub issue_type: Option<String>,
    pub form_fields: Vec<IssueFormField>,
    pub default_label_ids: Vec<Uuid>,
    pub default_assignee_user_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueFormField {
    pub id: Uuid,
    pub template_id: Uuid,
    pub field_key: String,
    pub label: String,
    pub field_type: String,
    pub description: Option<String>,
    pub placeholder: Option<String>,
    pub value: Option<String>,
    pub required: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueListQuery {
    pub query: Option<String>,
    pub state: IssueState,
    pub author: Option<String>,
    pub excluded_author: Option<String>,
    pub labels: Vec<String>,
    pub excluded_labels: Vec<String>,
    pub no_labels: bool,
    pub milestone: Option<String>,
    pub no_milestone: bool,
    pub assignee: Option<String>,
    pub no_assignee: bool,
    pub project: Option<String>,
    pub issue_type: Option<String>,
    pub sort: String,
}

impl Default for IssueListQuery {
    fn default() -> Self {
        Self {
            query: Some("is:issue state:open".to_owned()),
            state: IssueState::Open,
            author: None,
            excluded_author: None,
            labels: Vec::new(),
            excluded_labels: Vec::new(),
            no_labels: false,
            milestone: None,
            no_milestone: false,
            assignee: None,
            no_assignee: false,
            project: None,
            issue_type: None,
            sort: "updated-desc".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Comment {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub issue_id: Option<Uuid>,
    pub pull_request_id: Option<Uuid>,
    pub author_user_id: Uuid,
    pub body: String,
    pub is_minimized: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimelineEvent {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub issue_id: Option<Uuid>,
    pub pull_request_id: Option<Uuid>,
    pub actor_user_id: Option<Uuid>,
    pub event_type: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reaction {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub issue_id: Option<Uuid>,
    pub pull_request_id: Option<Uuid>,
    pub comment_id: Option<Uuid>,
    pub user_id: Uuid,
    pub content: ReactionContent,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssue {
    pub repository_id: Uuid,
    pub actor_user_id: Uuid,
    pub title: String,
    pub body: Option<String>,
    pub template_id: Option<Uuid>,
    pub template_slug: Option<String>,
    pub field_values: HashMap<String, String>,
    pub milestone_id: Option<Uuid>,
    pub label_ids: Vec<Uuid>,
    pub assignee_user_ids: Vec<Uuid>,
    pub attachments: Vec<CreateIssueAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueAttachment {
    pub file_name: String,
    pub byte_size: i64,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIssueState {
    pub actor_user_id: Uuid,
    pub state: IssueState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIssueSubscription {
    pub actor_user_id: Uuid,
    pub subscribed: bool,
    pub custom_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIssueMetadata {
    pub actor_user_id: Uuid,
    pub label_ids: Vec<Uuid>,
    pub assignee_user_ids: Vec<Uuid>,
    pub milestone_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueDiscussionConversionCategory {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub emoji: String,
    pub description: Option<String>,
    pub disabled_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IssueDiscussionConversionView {
    pub issue_id: Uuid,
    pub issue_number: i64,
    pub already_converted: bool,
    pub converted_discussion_number: Option<i64>,
    pub converted_discussion_href: Option<String>,
    pub categories: Vec<IssueDiscussionConversionCategory>,
    pub comment_count: i64,
    pub can_convert: bool,
    pub disabled_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertIssueToDiscussion {
    pub actor_user_id: Uuid,
    pub category_slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConvertIssueToDiscussionResponse {
    pub issue_id: Uuid,
    pub issue_number: i64,
    pub discussion_id: Uuid,
    pub discussion_number: i64,
    pub href: String,
    pub title: String,
    pub category_slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateComment {
    pub actor_user_id: Uuid,
    pub body: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReactionContent {
    ThumbsUp,
    ThumbsDown,
    Laugh,
    Hooray,
    Confused,
    Heart,
    Rocket,
    Eyes,
}

impl ReactionContent {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ThumbsUp => "thumbs_up",
            Self::ThumbsDown => "thumbs_down",
            Self::Laugh => "laugh",
            Self::Hooray => "hooray",
            Self::Confused => "confused",
            Self::Heart => "heart",
            Self::Rocket => "rocket",
            Self::Eyes => "eyes",
        }
    }
}

impl TryFrom<&str> for ReactionContent {
    type Error = CollaborationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "thumbs_up" => Ok(Self::ThumbsUp),
            "thumbs_down" => Ok(Self::ThumbsDown),
            "laugh" => Ok(Self::Laugh),
            "hooray" => Ok(Self::Hooray),
            "confused" => Ok(Self::Confused),
            "heart" => Ok(Self::Heart),
            "rocket" => Ok(Self::Rocket),
            "eyes" => Ok(Self::Eyes),
            other => Err(CollaborationError::InvalidReaction(other.to_owned())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CollaborationError {
    #[error("repository was not found")]
    RepositoryNotFound,
    #[error("user does not have repository access")]
    RepositoryAccessDenied,
    #[error("issue was not found")]
    IssueNotFound,
    #[error("pull request was not found")]
    PullRequestNotFound,
    #[error("invalid state `{0}`")]
    InvalidState(String),
    #[error("invalid reaction `{0}`")]
    InvalidReaction(String),
    #[error("invalid issue filter: {0}")]
    InvalidIssueFilter(String),
    #[error("invalid issue attachment: {0}")]
    InvalidIssueAttachment(String),
    #[error("{message}")]
    InvalidIssueField { field_key: String, message: String },
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub async fn ensure_default_labels(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<Label>, CollaborationError> {
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

    list_labels(pool, repository_id).await
}

pub async fn list_labels(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<Label>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT id, repository_id, name, color, description, is_default, created_at, updated_at
        FROM labels
        WHERE repository_id = $1
        ORDER BY lower(name)
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(label_from_row).collect())
}

pub async fn list_issue_templates_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<Vec<IssueTemplate>, CollaborationError> {
    let repository = get_repository(pool, repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;

    match actor_user_id {
        Some(user_id) => {
            repository_viewer_permission(pool, &repository, user_id, RepositoryRole::Read).await?;
        }
        None if repository.visibility == RepositoryVisibility::Public => {}
        None => return Err(CollaborationError::RepositoryAccessDenied),
    }

    let rows = sqlx::query(
        r#"
        SELECT id, repository_id, slug, name, description, title_prefill, body, issue_type,
               created_at, updated_at
        FROM issue_templates
        WHERE repository_id = $1
        ORDER BY display_order ASC, lower(name) ASC
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    let mut templates = rows
        .into_iter()
        .map(issue_template_from_row)
        .collect::<Vec<_>>();
    hydrate_issue_template_defaults(pool, &mut templates).await?;
    hydrate_issue_template_fields(pool, &mut templates).await?;
    Ok(templates)
}

pub async fn create_issue(pool: &PgPool, input: CreateIssue) -> Result<Issue, CollaborationError> {
    let actor_user_id = input.actor_user_id;
    require_repository_role(
        pool,
        input.repository_id,
        actor_user_id,
        RepositoryRole::Write,
    )
    .await?;
    let input = prepare_issue_create_body(pool, input).await?;
    validate_issue_create_metadata(pool, &input).await?;
    let number = next_issue_number(pool, input.repository_id).await?;
    let assignee_user_ids = input.assignee_user_ids.clone();
    let attachments = input.attachments.clone();
    let issue = insert_issue_with_number(pool, input, number).await?;
    insert_issue_body_version(pool, &issue).await?;
    insert_issue_attachments(pool, &issue, &attachments).await?;
    append_timeline_event(
        pool,
        issue.repository_id,
        Some(issue.id),
        None,
        Some(issue.author_user_id),
        "opened",
        json!({
            "number": issue.number,
            "attachments": attachments.len(),
        }),
    )
    .await?;
    notify_issue_assignees(pool, &issue, actor_user_id, &assignee_user_ids).await?;
    index_issue_search_document(pool, &issue, actor_user_id).await?;
    Ok(issue)
}

async fn prepare_issue_create_body(
    pool: &PgPool,
    mut input: CreateIssue,
) -> Result<CreateIssue, CollaborationError> {
    if input.template_id.is_none()
        && input
            .template_slug
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
    {
        return Ok(input);
    }

    let template = issue_template_for_create(
        pool,
        input.repository_id,
        input.template_id,
        input.template_slug.as_deref(),
    )
    .await?;
    validate_required_issue_fields(&template, &input.field_values)?;
    merge_uuid_defaults(&mut input.label_ids, &template.default_label_ids);
    merge_uuid_defaults(
        &mut input.assignee_user_ids,
        &template.default_assignee_user_ids,
    );
    input.body = Some(compose_issue_body_from_fields(
        input.body.as_deref(),
        &template.form_fields,
        &input.field_values,
    ));
    Ok(input)
}

fn merge_uuid_defaults(target: &mut Vec<Uuid>, defaults: &[Uuid]) {
    for id in defaults {
        if !target.contains(id) {
            target.push(*id);
        }
    }
}

async fn validate_issue_create_metadata(
    pool: &PgPool,
    input: &CreateIssue,
) -> Result<(), CollaborationError> {
    for label_id in &input.label_ids {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM labels WHERE id = $1 AND repository_id = $2)",
        )
        .bind(label_id)
        .bind(input.repository_id)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(CollaborationError::InvalidIssueField {
                field_key: "labelIds".to_owned(),
                message: "label is not available for this repository".to_owned(),
            });
        }
    }

    let repository = get_repository(pool, input.repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    for assignee_user_id in &input.assignee_user_ids {
        if let Err(error) =
            repository_viewer_permission(pool, &repository, *assignee_user_id, RepositoryRole::Read)
                .await
        {
            return Err(match error {
                CollaborationError::RepositoryAccessDenied => {
                    CollaborationError::InvalidIssueField {
                        field_key: "assigneeUserIds".to_owned(),
                        message: "assignee is not available for this repository".to_owned(),
                    }
                }
                other => other,
            });
        }
    }

    for attachment in &input.attachments {
        if attachment.file_name.trim().is_empty() {
            return Err(CollaborationError::InvalidIssueAttachment(
                "attachment filename is required".to_owned(),
            ));
        }
        if attachment.byte_size < 0 {
            return Err(CollaborationError::InvalidIssueAttachment(
                "attachment size must be non-negative".to_owned(),
            ));
        }
    }

    Ok(())
}

pub async fn list_issues(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
    state: Option<IssueState>,
    page: i64,
    page_size: i64,
) -> Result<ListEnvelope<Issue>, CollaborationError> {
    require_repository_role(pool, repository_id, actor_user_id, RepositoryRole::Read).await?;
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;
    let state_filter = state.as_ref().map(IssueState::as_str);

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM issues
        WHERE repository_id = $1
          AND ($2::text IS NULL OR state = $2)
          AND NOT EXISTS (
              SELECT 1 FROM pull_requests WHERE pull_requests.issue_id = issues.id
          )
        "#,
    )
    .bind(repository_id)
    .bind(state_filter)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT id, repository_id, number, title, body, state, author_user_id, milestone_id,
               locked, closed_by_user_id, closed_at, created_at, updated_at
        FROM issues
        WHERE repository_id = $1
          AND ($2::text IS NULL OR state = $2)
          AND NOT EXISTS (
              SELECT 1 FROM pull_requests WHERE pull_requests.issue_id = issues.id
          )
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
        .map(issue_from_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListEnvelope {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn repository_issue_list_view(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Uuid,
    filters: IssueListQuery,
    page: i64,
    page_size: i64,
) -> Result<IssueListView, CollaborationError> {
    repository_issue_list_view_for_viewer(
        pool,
        repository_id,
        Some(actor_user_id),
        filters,
        page,
        page_size,
    )
    .await
}

pub async fn repository_issue_list_view_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    actor_user_id: Option<Uuid>,
    filters: IssueListQuery,
    page: i64,
    page_size: i64,
) -> Result<IssueListView, CollaborationError> {
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
        .map(search_text_from_issue_query)
        .filter(|value| !value.is_empty());
    let label_filters = filters.labels.clone();
    let excluded_label_filters = filters.excluded_labels.clone();
    let author_filter = filters.author.clone();
    let excluded_author_filter = filters.excluded_author.clone();
    let milestone_filter = filters.milestone.clone();
    let assignee_filter = filters.assignee.clone();
    let project_filter = filters.project.clone();
    let issue_type_filter = filters.issue_type.clone();
    let count_filters = IssueListCountFilters {
        text_filter: text_filter.as_deref(),
        author_filter: author_filter.as_deref(),
        excluded_author_filter: excluded_author_filter.as_deref(),
        label_filters: &label_filters,
        excluded_label_filters: &excluded_label_filters,
        no_labels: filters.no_labels,
        milestone_filter: milestone_filter.as_deref(),
        no_milestone: filters.no_milestone,
        assignee_filter: assignee_filter.as_deref(),
        no_assignee: filters.no_assignee,
        project_filter: project_filter.as_deref(),
        issue_type_filter: issue_type_filter.as_deref(),
    };

    let open_count = count_issue_list_items(
        pool,
        repository_id,
        IssueState::Open.as_str(),
        &count_filters,
    )
    .await?;
    let closed_count = count_issue_list_items(
        pool,
        repository_id,
        IssueState::Closed.as_str(),
        &count_filters,
    )
    .await?;
    let total = if filters.state == IssueState::Open {
        open_count
    } else {
        closed_count
    };

    let rows = sqlx::query(
        r#"
        SELECT issues.id, issues.repository_id, issues.number, issues.title, issues.body,
               issues.state, issues.author_user_id, issues.milestone_id, issues.locked,
               issues.closed_by_user_id, issues.closed_at, issues.created_at, issues.updated_at
        FROM issues
        WHERE issues.repository_id = $1
          AND issues.state = $2
          AND NOT EXISTS (
              SELECT 1 FROM pull_requests WHERE pull_requests.issue_id = issues.id
          )
          AND (
              $3::text IS NULL
              OR issues.title ILIKE '%' || $3 || '%'
              OR COALESCE(issues.body, '') ILIKE '%' || $3 || '%'
          )
          AND (
              $4::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM users
                  WHERE users.id = issues.author_user_id
                    AND (
                        lower(COALESCE(users.username, users.email)) = lower($4)
                        OR lower(users.email) = lower($4)
                    )
              )
          )
          AND (
              $5::text IS NULL
              OR NOT EXISTS (
                  SELECT 1
                  FROM users
                  WHERE users.id = issues.author_user_id
                    AND (
                        lower(COALESCE(users.username, users.email)) = lower($5)
                        OR lower(users.email) = lower($5)
                    )
              )
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
              cardinality($7::text[]) = 0
              OR NOT EXISTS (
                  SELECT 1
                  FROM issue_labels
                  JOIN labels ON labels.id = issue_labels.label_id
                  JOIN unnest($7::text[]) blocked_label(name)
                    ON lower(labels.name) = lower(blocked_label.name)
                  WHERE issue_labels.issue_id = issues.id
              )
          )
          AND (
              $8::boolean = false
              OR NOT EXISTS (
                  SELECT 1 FROM issue_labels WHERE issue_labels.issue_id = issues.id
              )
          )
          AND (
              $9::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM milestones
                  WHERE milestones.id = issues.milestone_id
                    AND lower(milestones.title) = lower($9)
              )
          )
          AND (
              $10::boolean = false
              OR issues.milestone_id IS NULL
          )
          AND (
              $11::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM issue_assignees
                  JOIN users ON users.id = issue_assignees.user_id
                  WHERE issue_assignees.issue_id = issues.id
                    AND (
                        lower(COALESCE(users.username, users.email)) = lower($11)
                        OR lower(users.email) = lower($11)
                    )
              )
          )
          AND (
              $12::boolean = false
              OR NOT EXISTS (
                  SELECT 1 FROM issue_assignees WHERE issue_assignees.issue_id = issues.id
              )
          )
          AND $13::text IS NULL
          AND (
              $14::text IS NULL
              OR lower($14) IN ('issue', 'issues')
          )
        ORDER BY
          CASE WHEN $15 = 'best-match' AND $3::text IS NOT NULL AND issues.title ILIKE '%' || $3 || '%' THEN 0 END ASC,
          CASE WHEN $15 = 'best-match' AND $3::text IS NOT NULL AND COALESCE(issues.body, '') ILIKE '%' || $3 || '%' THEN 1 END ASC,
          CASE WHEN $15 = 'best-match' THEN issues.updated_at END DESC,
          CASE WHEN $15 = 'comments-desc' THEN (
              SELECT count(*) FROM comments WHERE comments.issue_id = issues.id
          ) END DESC,
          CASE WHEN $15 = 'comments-asc' THEN (
              SELECT count(*) FROM comments WHERE comments.issue_id = issues.id
          ) END ASC,
          CASE WHEN $15 = 'created-asc' THEN issues.created_at END ASC,
          CASE WHEN $15 = 'created-desc' THEN issues.created_at END DESC,
          CASE WHEN $15 = 'updated-desc' THEN issues.updated_at END DESC,
          CASE WHEN $15 = 'updated-asc' THEN issues.updated_at END ASC,
          issues.updated_at DESC,
          issues.number DESC
        LIMIT $16 OFFSET $17
        "#,
    )
    .bind(repository_id)
    .bind(state_filter)
    .bind(text_filter.as_deref())
    .bind(author_filter.as_deref())
    .bind(excluded_author_filter.as_deref())
    .bind(&label_filters)
    .bind(&excluded_label_filters)
    .bind(filters.no_labels)
    .bind(milestone_filter.as_deref())
    .bind(filters.no_milestone)
    .bind(assignee_filter.as_deref())
    .bind(filters.no_assignee)
    .bind(project_filter.as_deref())
    .bind(issue_type_filter.as_deref())
    .bind(&filters.sort)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let issues = rows
        .into_iter()
        .map(issue_from_row)
        .collect::<Result<Vec<_>, _>>()?;
    let items = issue_list_items_for_issues(pool, &repository, issues).await?;
    let label_options = issue_list_label_options(pool, repository_id).await?;
    let user_options = issue_list_user_options(pool, repository_id).await?;
    let milestone_options = issue_list_milestone_options(pool, repository_id).await?;
    let preferences = match actor_user_id {
        Some(user_id) => get_repository_issue_preferences(pool, repository_id, user_id).await?,
        None => IssueListPreferences {
            dismissed_contributor_banner: false,
            dismissed_contributor_banner_at: None,
        },
    };

    Ok(IssueListView {
        items,
        total,
        page,
        page_size,
        open_count,
        closed_count,
        counts: IssueListCounts {
            open: open_count,
            closed: closed_count,
        },
        filters: IssueListFilters {
            query: filters
                .query
                .unwrap_or_else(|| "is:issue state:open".to_owned()),
            state: filters.state,
            author: filters.author,
            excluded_author: filters.excluded_author,
            labels: filters.labels,
            excluded_labels: filters.excluded_labels,
            no_labels: filters.no_labels,
            milestone: filters.milestone,
            no_milestone: filters.no_milestone,
            assignee: filters.assignee,
            no_assignee: filters.no_assignee,
            project: filters.project,
            issue_type: filters.issue_type,
            sort: filters.sort,
        },
        filter_options: IssueListFilterOptions {
            labels: label_options,
            users: user_options,
            milestones: milestone_options,
            projects: Vec::new(),
            issue_types: Vec::new(),
        },
        viewer_permission,
        repository: IssueListRepository {
            id: repository.id,
            owner_login: repository.owner_login,
            name: repository.name,
            visibility: repository.visibility,
        },
        preferences,
    })
}

pub async fn get_repository_issue_preferences(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
) -> Result<IssueListPreferences, CollaborationError> {
    require_repository_role(pool, repository_id, user_id, RepositoryRole::Read).await?;
    repository_issue_preferences_row(pool, repository_id, user_id).await
}

pub async fn save_repository_issue_preferences(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
    dismissed_contributor_banner: bool,
) -> Result<IssueListPreferences, CollaborationError> {
    require_repository_role(pool, repository_id, user_id, RepositoryRole::Read).await?;
    let dismissed_at = dismissed_contributor_banner.then(Utc::now);
    sqlx::query(
        r#"
        INSERT INTO repository_issue_preferences (
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

    repository_issue_preferences_row(pool, repository_id, user_id).await
}

pub async fn global_issue_list_for_viewer(
    pool: &PgPool,
    actor_user_id: Uuid,
    filters: GlobalIssueListQuery,
    page: i64,
    page_size: i64,
) -> Result<GlobalIssueListView, CollaborationError> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;
    let scope = filters.scope.as_str();
    let state_filter = filters.state.as_ref().map(IssueState::as_str);
    let text_filter = filters
        .query
        .as_deref()
        .map(search_text_from_issue_query)
        .filter(|value| !value.is_empty());

    let total = count_global_issue_list_items(
        pool,
        actor_user_id,
        scope,
        state_filter,
        text_filter.as_deref(),
        &filters,
    )
    .await?;
    let counts = GlobalIssueCounts {
        created: count_global_issue_list_items(
            pool,
            actor_user_id,
            "created",
            state_filter,
            text_filter.as_deref(),
            &filters,
        )
        .await?,
        assigned: count_global_issue_list_items(
            pool,
            actor_user_id,
            "assigned",
            state_filter,
            text_filter.as_deref(),
            &filters,
        )
        .await?,
        mentioned: count_global_issue_list_items(
            pool,
            actor_user_id,
            "mentioned",
            state_filter,
            text_filter.as_deref(),
            &filters,
        )
        .await?,
    };

    let rows = sqlx::query(
        r#"
        SELECT issues.id, issues.repository_id, issues.number, issues.title, issues.body,
               issues.state, issues.author_user_id, issues.milestone_id, issues.locked,
               issues.closed_by_user_id, issues.closed_at, issues.created_at, issues.updated_at
        FROM issues
        JOIN repositories ON repositories.id = issues.repository_id
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
          AND NOT EXISTS (
              SELECT 1 FROM pull_requests WHERE pull_requests.issue_id = issues.id
          )
          AND (
              ($2 = 'created' AND issues.author_user_id = $1)
              OR ($2 = 'assigned' AND EXISTS (
                  SELECT 1 FROM issue_assignees
                  WHERE issue_assignees.issue_id = issues.id
                    AND issue_assignees.user_id = $1
              ))
              OR ($2 = 'mentioned' AND EXISTS (
                  SELECT 1 FROM notifications
                  WHERE notifications.subject_type = 'issue'
                    AND notifications.subject_id = issues.id
                    AND notifications.user_id = $1
                    AND notifications.reason IN ('mention', 'team_mention')
              ))
          )
          AND ($3::text IS NULL OR issues.state = $3)
          AND (
              $4::text IS NULL
              OR issues.title ILIKE '%' || $4 || '%'
              OR COALESCE(issues.body, '') ILIKE '%' || $4 || '%'
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
                  SELECT 1 FROM milestones
                  WHERE milestones.id = issues.milestone_id
                    AND lower(milestones.title) = lower($7)
              )
          )
          AND (
              $8::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM project_items
                  JOIN projects ON projects.id = project_items.project_id
                  WHERE project_items.issue_id = issues.id
                    AND project_items.archived_at IS NULL
                    AND lower(projects.title) = lower($8)
              )
          )
        ORDER BY
          CASE WHEN $9 = 'best-match' AND $4::text IS NOT NULL AND issues.title ILIKE '%' || $4 || '%' THEN 0 END ASC,
          CASE WHEN $9 = 'best-match' AND $4::text IS NOT NULL AND COALESCE(issues.body, '') ILIKE '%' || $4 || '%' THEN 1 END ASC,
          CASE WHEN $9 = 'best-match' THEN issues.updated_at END DESC,
          CASE WHEN $9 = 'comments-desc' THEN (SELECT count(*) FROM comments WHERE comments.issue_id = issues.id) END DESC,
          CASE WHEN $9 = 'comments-asc' THEN (SELECT count(*) FROM comments WHERE comments.issue_id = issues.id) END ASC,
          CASE WHEN $9 = 'created-asc' THEN issues.created_at END ASC,
          CASE WHEN $9 = 'created-desc' THEN issues.created_at END DESC,
          CASE WHEN $9 = 'updated-desc' THEN issues.updated_at END DESC,
          CASE WHEN $9 = 'updated-asc' THEN issues.updated_at END ASC,
          issues.updated_at DESC,
          issues.number DESC
        LIMIT $10 OFFSET $11
        "#,
    )
    .bind(actor_user_id)
    .bind(scope)
    .bind(state_filter)
    .bind(text_filter.as_deref())
    .bind(filters.repository.as_deref())
    .bind(&filters.labels)
    .bind(filters.milestone.as_deref())
    .bind(filters.project.as_deref())
    .bind(&filters.sort)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let issues = rows
        .into_iter()
        .map(issue_from_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(GlobalIssueListView {
        items: global_issue_list_items(pool, issues).await?,
        total,
        page,
        page_size,
        counts,
        filters: GlobalIssueFilters {
            scope: filters.scope,
            query: filters
                .query
                .unwrap_or_else(|| "is:issue state:open".to_owned()),
            state: filters.state,
            repository: filters.repository,
            labels: filters.labels,
            milestone: filters.milestone,
            project: filters.project,
            sort: filters.sort,
        },
        filter_options: GlobalIssueFilterOptions {
            repositories: global_issue_repository_options(pool, actor_user_id, scope).await?,
            labels: global_issue_label_options(pool, actor_user_id, scope).await?,
            milestones: global_issue_milestone_options(pool, actor_user_id, scope).await?,
            projects: global_issue_project_options(pool, actor_user_id, scope).await?,
            sort_options: issue_sort_options()
                .into_iter()
                .map(ToOwned::to_owned)
                .collect(),
        },
    })
}

pub async fn get_issue(
    pool: &PgPool,
    repository_id: Uuid,
    number: i64,
    actor_user_id: Uuid,
) -> Result<Issue, CollaborationError> {
    require_repository_role(pool, repository_id, actor_user_id, RepositoryRole::Read).await?;
    let row = sqlx::query(
        r#"
        SELECT id, repository_id, number, title, body, state, author_user_id, milestone_id,
               locked, closed_by_user_id, closed_at, created_at, updated_at
        FROM issues
        WHERE repository_id = $1 AND number = $2
          AND NOT EXISTS (
              SELECT 1 FROM pull_requests WHERE pull_requests.issue_id = issues.id
          )
        "#,
    )
    .bind(repository_id)
    .bind(number)
    .fetch_optional(pool)
    .await?
    .ok_or(CollaborationError::IssueNotFound)?;

    issue_from_row(row)
}

pub async fn repository_issue_detail_view_for_viewer(
    pool: &PgPool,
    repository_id: Uuid,
    number: i64,
    actor_user_id: Option<Uuid>,
) -> Result<IssueDetailView, CollaborationError> {
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
        SELECT id, repository_id, number, title, body, state, author_user_id, milestone_id,
               locked, closed_by_user_id, closed_at, created_at, updated_at
        FROM issues
        WHERE repository_id = $1 AND number = $2
          AND NOT EXISTS (
              SELECT 1 FROM pull_requests WHERE pull_requests.issue_id = issues.id
          )
        "#,
    )
    .bind(repository_id)
    .bind(number)
    .fetch_optional(pool)
    .await?
    .ok_or(CollaborationError::IssueNotFound)?;
    let issue = issue_from_row(row)?;
    let issue_ids = vec![issue.id];
    let authors = issue_list_users(pool, &issue_ids, "author").await?;
    let labels = issue_list_labels(pool, &issue_ids).await?;
    let milestones = issue_list_milestones(pool, &issue_ids).await?;
    let assignees = issue_list_assignees(pool, &issue_ids).await?;
    let comment_counts = issue_comment_counts(pool, &issue_ids).await?;
    let linked_pull_requests = linked_pull_request_hints(pool, &issue_ids, &repository).await?;
    let participants = issue_detail_participants(pool, issue.id).await?;
    let attachments = issue_detail_attachments(pool, issue.id).await?;
    let subscription = issue_subscription_state(pool, issue.id, actor_user_id).await?;
    let reactions = reaction_summaries(pool, Some(issue.id), None, actor_user_id).await?;
    let metadata_options = IssueDetailMetadataOptions {
        labels: issue_list_label_options(pool, repository_id).await?,
        assignees: issue_list_user_options(pool, repository_id).await?,
        milestones: issue_list_milestone_options(pool, repository_id).await?,
    };
    let rendered = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            markdown: issue.body.clone().unwrap_or_default(),
            repository_id: Some(repository_id),
            owner: Some(repository.owner_login.clone()),
            repo: Some(repository.name.clone()),
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
                message: "issue body could not be rendered".to_owned(),
            }
        }
    })?;

    Ok(IssueDetailView {
        id: issue.id,
        repository_id: issue.repository_id,
        repository_owner: repository.owner_login.clone(),
        repository_name: repository.name.clone(),
        number: issue.number,
        title: issue.title,
        body: issue.body,
        body_html: rendered.html,
        state: issue.state,
        author: authors
            .get(&issue.id)
            .cloned()
            .unwrap_or_else(|| fallback_issue_user(issue.author_user_id)),
        labels: labels.get(&issue.id).cloned().unwrap_or_default(),
        milestone: milestones.get(&issue.id).cloned(),
        assignees: assignees.get(&issue.id).cloned().unwrap_or_default(),
        participants,
        attachments,
        comment_count: *comment_counts.get(&issue.id).unwrap_or(&0),
        linked_pull_request: linked_pull_requests.get(&issue.id).cloned(),
        href: format!(
            "/{}/{}/issues/{}",
            repository.owner_login, repository.name, issue.number
        ),
        locked: issue.locked,
        created_at: issue.created_at,
        updated_at: issue.updated_at,
        closed_at: issue.closed_at,
        viewer_permission,
        repository: IssueListRepository {
            id: repository.id,
            owner_login: repository.owner_login,
            name: repository.name,
            visibility: repository.visibility,
        },
        subscription,
        reactions,
        metadata_options,
    })
}

pub async fn update_issue_metadata(
    pool: &PgPool,
    issue_id: Uuid,
    input: UpdateIssueMetadata,
) -> Result<Issue, CollaborationError> {
    let issue = issue_by_id(pool, issue_id).await?;
    let repository = get_repository(pool, issue.repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    require_repository_role(
        pool,
        issue.repository_id,
        input.actor_user_id,
        RepositoryRole::Write,
    )
    .await?;

    for label_id in &input.label_ids {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM labels WHERE id = $1 AND repository_id = $2)",
        )
        .bind(label_id)
        .bind(issue.repository_id)
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
        .bind(issue.repository_id)
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
        .bind(issue_id)
        .bind(input.milestone_id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM issue_labels WHERE issue_id = $1")
        .bind(issue_id)
        .execute(pool)
        .await?;
    for label_id in &input.label_ids {
        sqlx::query(
            r#"
            INSERT INTO issue_labels (issue_id, label_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(issue_id)
        .bind(label_id)
        .execute(pool)
        .await?;
    }

    sqlx::query("DELETE FROM issue_assignees WHERE issue_id = $1")
        .bind(issue_id)
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
        .bind(issue_id)
        .bind(assignee_user_id)
        .bind(input.actor_user_id)
        .execute(pool)
        .await?;
    }

    append_timeline_event(
        pool,
        issue.repository_id,
        Some(issue.id),
        None,
        Some(input.actor_user_id),
        "metadata_changed",
        json!({
            "labelIds": input.label_ids,
            "assigneeUserIds": input.assignee_user_ids,
            "milestoneId": input.milestone_id,
        }),
    )
    .await?;
    notify_issue_assignees(pool, &issue, input.actor_user_id, &input.assignee_user_ids).await?;
    index_issue_search_document(pool, &issue, input.actor_user_id).await?;

    issue_by_id(pool, issue_id).await
}

pub async fn update_issue_state(
    pool: &PgPool,
    issue_id: Uuid,
    input: UpdateIssueState,
) -> Result<Issue, CollaborationError> {
    let repository_id = issue_repository_id(pool, issue_id).await?;
    require_repository_role(
        pool,
        repository_id,
        input.actor_user_id,
        RepositoryRole::Write,
    )
    .await?;

    let row = sqlx::query(
        r#"
        UPDATE issues
        SET state = $2,
            closed_by_user_id = CASE WHEN $2 = 'closed' THEN $3 ELSE NULL END,
            closed_at = CASE WHEN $2 = 'closed' THEN now() ELSE NULL END
        WHERE id = $1
        RETURNING id, repository_id, number, title, body, state, author_user_id, milestone_id,
                  locked, closed_by_user_id, closed_at, created_at, updated_at
        "#,
    )
    .bind(issue_id)
    .bind(input.state.as_str())
    .bind(input.actor_user_id)
    .fetch_one(pool)
    .await?;
    let issue = issue_from_row(row)?;
    let event_type = match issue.state {
        IssueState::Open => "reopened",
        IssueState::Closed => "closed",
    };
    append_timeline_event(
        pool,
        issue.repository_id,
        Some(issue.id),
        None,
        Some(input.actor_user_id),
        event_type,
        json!({ "number": issue.number }),
    )
    .await?;
    notify_issue_participants(
        pool,
        &issue,
        input.actor_user_id,
        event_type,
        format!("Issue #{} was {}", issue.number, event_type),
    )
    .await?;
    run_project_item_automation(
        pool,
        ProjectAutomationInput {
            actor_user_id: input.actor_user_id,
            repository_id: issue.repository_id,
            issue_id: Some(issue.id),
            pull_request_id: None,
            event: match issue.state {
                IssueState::Open => ProjectAutomationEvent::IssueReopened,
                IssueState::Closed => ProjectAutomationEvent::IssueClosed,
            },
        },
    )
    .await
    .map_err(|error| match error {
        super::projects::ProjectsError::Sqlx(error) => CollaborationError::Sqlx(error),
        _ => CollaborationError::IssueNotFound,
    })?;
    index_issue_search_document(pool, &issue, input.actor_user_id).await?;
    Ok(issue)
}

pub async fn update_issue_subscription(
    pool: &PgPool,
    issue_id: Uuid,
    input: UpdateIssueSubscription,
) -> Result<IssueSubscriptionState, CollaborationError> {
    let repository_id = issue_repository_id(pool, issue_id).await?;
    require_repository_role(
        pool,
        repository_id,
        input.actor_user_id,
        RepositoryRole::Read,
    )
    .await?;

    let custom_events = normalize_thread_subscription_events(&input.custom_events)?;
    sqlx::query(
        r#"
        INSERT INTO issue_subscriptions (issue_id, user_id, subscribed, reason, custom_events)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (issue_id, user_id)
        DO UPDATE SET
            subscribed = EXCLUDED.subscribed,
            reason = EXCLUDED.reason,
            custom_events = EXCLUDED.custom_events
        "#,
    )
    .bind(issue_id)
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

    issue_subscription_state(pool, issue_id, Some(input.actor_user_id)).await
}

pub async fn add_issue_comment(
    pool: &PgPool,
    issue_id: Uuid,
    input: CreateComment,
) -> Result<Comment, CollaborationError> {
    let repository_id = issue_repository_id(pool, issue_id).await?;
    require_repository_role(
        pool,
        repository_id,
        input.actor_user_id,
        RepositoryRole::Write,
    )
    .await?;

    let row = sqlx::query(
        r#"
        INSERT INTO comments (repository_id, issue_id, author_user_id, body)
        VALUES ($1, $2, $3, $4)
        RETURNING id, repository_id, issue_id, pull_request_id, author_user_id, body,
                  is_minimized, created_at, updated_at
        "#,
    )
    .bind(repository_id)
    .bind(issue_id)
    .bind(input.actor_user_id)
    .bind(&input.body)
    .fetch_one(pool)
    .await?;
    let comment = comment_from_row(row);
    append_timeline_event(
        pool,
        repository_id,
        Some(issue_id),
        None,
        Some(input.actor_user_id),
        "commented",
        json!({ "commentId": comment.id }),
    )
    .await?;
    let issue = issue_by_id(pool, issue_id).await?;
    notify_issue_participants(
        pool,
        &issue,
        input.actor_user_id,
        "comment",
        format!("New comment on issue #{}: {}", issue.number, issue.title),
    )
    .await?;
    Ok(comment)
}

pub async fn issue_discussion_conversion_view(
    pool: &PgPool,
    repository_id: Uuid,
    issue_number: i64,
    actor_user_id: Uuid,
) -> Result<IssueDiscussionConversionView, CollaborationError> {
    require_repository_role(pool, repository_id, actor_user_id, RepositoryRole::Triage).await?;
    let issue = get_issue(pool, repository_id, issue_number, actor_user_id).await?;
    let converted = issue_converted_discussion(pool, issue.id).await?;
    let categories = issue_conversion_categories(pool, repository_id).await?;
    let disabled_reason = if converted.is_some() {
        Some("This issue has already been converted to a discussion.".to_owned())
    } else if categories
        .iter()
        .all(|category| category.disabled_reason.is_some())
    {
        Some("No eligible discussion categories are available.".to_owned())
    } else {
        None
    };
    Ok(IssueDiscussionConversionView {
        issue_id: issue.id,
        issue_number: issue.number,
        already_converted: converted.is_some(),
        converted_discussion_number: converted.as_ref().map(|item| item.0),
        converted_discussion_href: converted.map(|item| item.1),
        categories,
        comment_count: issue_comment_count(pool, issue.id).await?,
        can_convert: disabled_reason.is_none(),
        disabled_reason,
    })
}

pub async fn convert_issue_to_discussion(
    pool: &PgPool,
    repository_id: Uuid,
    issue_number: i64,
    input: ConvertIssueToDiscussion,
) -> Result<ConvertIssueToDiscussionResponse, CollaborationError> {
    require_repository_role(
        pool,
        repository_id,
        input.actor_user_id,
        RepositoryRole::Triage,
    )
    .await?;
    let issue = get_issue(pool, repository_id, issue_number, input.actor_user_id).await?;
    if let Some((discussion_number, href)) = issue_converted_discussion(pool, issue.id).await? {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "issue".to_owned(),
            message: format!("issue was already converted to {href} (#{discussion_number})"),
        });
    }
    let category_slug = input
        .category_slug
        .trim()
        .trim_start_matches('/')
        .to_lowercase();
    if category_slug.is_empty() {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "categorySlug".to_owned(),
            message: "choose a discussion category".to_owned(),
        });
    }
    let category = sqlx::query(
        r#"
        SELECT id, slug, name, emoji, description, format
        FROM discussion_categories
        WHERE repository_id = $1 AND lower(slug) = lower($2)
        "#,
    )
    .bind(repository_id)
    .bind(&category_slug)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| CollaborationError::InvalidIssueField {
        field_key: "categorySlug".to_owned(),
        message: "discussion category is not available".to_owned(),
    })?;
    let category_id: Uuid = category.get("id");
    let category_format: String = category
        .try_get("format")
        .unwrap_or_else(|_| "discussion".to_owned());
    if matches!(category_format.as_str(), "poll") {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "categorySlug".to_owned(),
            message: "poll categories cannot receive converted issues".to_owned(),
        });
    }

    let repository = get_repository(pool, repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    if repository.is_archived {
        return Err(CollaborationError::InvalidIssueField {
            field_key: "repository".to_owned(),
            message: "archived repositories cannot convert issues".to_owned(),
        });
    }

    let next_number: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(number), 0) + 1 FROM discussions WHERE repository_id = $1",
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    let discussion_id = Uuid::new_v4();
    let body = issue.body.clone().unwrap_or_default();
    sqlx::query(
        r#"
        INSERT INTO discussions (
            id, repository_id, category_id, number, title, body, author_user_id,
            comments_count, last_activity_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, 1, now())
        "#,
    )
    .bind(discussion_id)
    .bind(repository_id)
    .bind(category_id)
    .bind(next_number)
    .bind(&issue.title)
    .bind(&body)
    .bind(issue.author_user_id)
    .execute(pool)
    .await?;

    let root_comment_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO discussion_comments (id, discussion_id, author_user_id, body)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(root_comment_id)
    .bind(discussion_id)
    .bind(issue.author_user_id)
    .bind(&body)
    .execute(pool)
    .await?;

    let comment_rows = sqlx::query(
        r#"
        SELECT id, author_user_id, body
        FROM comments
        WHERE issue_id = $1 AND is_minimized = false
        ORDER BY created_at ASC
        "#,
    )
    .bind(issue.id)
    .fetch_all(pool)
    .await?;
    for row in comment_rows {
        sqlx::query(
            r#"
            INSERT INTO discussion_comments (
                id, discussion_id, author_user_id, body, converted_issue_comment_id
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(discussion_id)
        .bind(row.get::<Uuid, _>("author_user_id"))
        .bind(row.get::<String, _>("body"))
        .bind(row.get::<Uuid, _>("id"))
        .execute(pool)
        .await?;
    }
    let copied_comments = issue_comment_count(pool, issue.id).await?;
    sqlx::query(
        r#"
        UPDATE discussions
        SET comments_count = $2, updated_at = now(), last_activity_at = now()
        WHERE id = $1
        "#,
    )
    .bind(discussion_id)
    .bind(copied_comments + 1)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE issues
        SET state = 'closed',
            closed_by_user_id = $3,
            closed_at = COALESCE(closed_at, now()),
            converted_discussion_id = $2,
            converted_to_discussion_at = now(),
            converted_to_discussion_by_user_id = $3,
            updated_at = now()
        WHERE id = $1
        "#,
    )
    .bind(issue.id)
    .bind(discussion_id)
    .bind(input.actor_user_id)
    .execute(pool)
    .await?;

    let href = format!(
        "/{}/{}/discussions/{}",
        repository.owner_login, repository.name, next_number
    );
    append_timeline_event(
        pool,
        repository_id,
        Some(issue.id),
        None,
        Some(input.actor_user_id),
        "converted_to_discussion",
        json!({
            "discussionId": discussion_id,
            "discussionNumber": next_number,
            "discussionHref": href,
            "categorySlug": category_slug,
            "copiedComments": copied_comments,
        }),
    )
    .await?;
    sqlx::query(
        r#"
        INSERT INTO discussion_activity_events (discussion_id, actor_user_id, event_type, payload)
        VALUES ($1, $2, 'converted_from_issue', $3::jsonb)
        "#,
    )
    .bind(discussion_id)
    .bind(input.actor_user_id)
    .bind(
        json!({
            "issueId": issue.id,
            "issueNumber": issue.number,
            "copiedComments": copied_comments,
        })
        .to_string(),
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'repository.issue.convert_to_discussion', 'issue', $2, $3::jsonb)
        "#,
    )
    .bind(input.actor_user_id)
    .bind(issue.id.to_string())
    .bind(
        json!({
            "repositoryId": repository_id,
            "discussionId": discussion_id,
            "discussionNumber": next_number,
            "categorySlug": category_slug,
        })
        .to_string(),
    )
    .execute(pool)
    .await?;

    if issue.author_user_id != input.actor_user_id {
        create_notification(
            pool,
            CreateNotification {
                user_id: issue.author_user_id,
                repository_id: Some(repository_id),
                subject_type: "discussion".to_owned(),
                subject_id: Some(discussion_id),
                title: format!(
                    "Issue #{} was converted to discussion #{}: {}",
                    issue.number, next_number, issue.title
                ),
                reason: "issue_converted_to_discussion".to_owned(),
            },
        )
        .await
        .map_err(|error| match error {
            super::notifications::NotificationError::Sqlx(error) => CollaborationError::Sqlx(error),
            super::notifications::NotificationError::NotFound => CollaborationError::IssueNotFound,
            super::notifications::NotificationError::Validation(_) => {
                CollaborationError::IssueNotFound
            }
        })?;
    }

    Ok(ConvertIssueToDiscussionResponse {
        issue_id: issue.id,
        issue_number: issue.number,
        discussion_id,
        discussion_number: next_number,
        href,
        title: issue.title,
        category_slug,
    })
}

pub async fn issue_comment_timeline_item(
    pool: &PgPool,
    comment_id: Uuid,
) -> Result<IssueTimelineItem, CollaborationError> {
    let row = sqlx::query(
        r#"
        SELECT timeline_events.id, timeline_events.event_type, timeline_events.metadata,
               timeline_events.created_at,
               comments.id AS comment_id, comments.body, comments.is_minimized,
               comments.created_at AS comment_created_at, comments.updated_at AS comment_updated_at,
               users.id AS actor_id, COALESCE(users.username, users.email) AS actor_login,
               users.display_name AS actor_display_name, users.avatar_url AS actor_avatar_url
        FROM comments
        JOIN timeline_events
          ON timeline_events.issue_id = comments.issue_id
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

    timeline_item_from_row(pool, row, None).await
}

pub async fn add_issue_reaction(
    pool: &PgPool,
    issue_id: Uuid,
    user_id: Uuid,
    content: ReactionContent,
) -> Result<Reaction, CollaborationError> {
    let repository_id = issue_repository_id(pool, issue_id).await?;
    require_repository_role(pool, repository_id, user_id, RepositoryRole::Read).await?;
    let row = sqlx::query(
        r#"
        INSERT INTO reactions (repository_id, issue_id, user_id, content)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (issue_id, user_id, content)
        WHERE issue_id IS NOT NULL
        DO UPDATE SET content = EXCLUDED.content
        RETURNING id, repository_id, issue_id, pull_request_id, comment_id, user_id, content, created_at
        "#,
    )
    .bind(repository_id)
    .bind(issue_id)
    .bind(user_id)
    .bind(content.as_str())
    .fetch_one(pool)
    .await?;

    reaction_from_row(row)
}

pub async fn toggle_issue_reaction(
    pool: &PgPool,
    issue_id: Uuid,
    user_id: Uuid,
    content: ReactionContent,
) -> Result<Vec<ReactionSummary>, CollaborationError> {
    let repository_id = issue_repository_id(pool, issue_id).await?;
    require_repository_role(pool, repository_id, user_id, RepositoryRole::Read).await?;

    let deleted = sqlx::query(
        r#"
        DELETE FROM reactions
        WHERE issue_id = $1 AND user_id = $2 AND content = $3
        "#,
    )
    .bind(issue_id)
    .bind(user_id)
    .bind(content.as_str())
    .execute(pool)
    .await?
    .rows_affected();

    if deleted == 0 {
        sqlx::query(
            r#"
            INSERT INTO reactions (repository_id, issue_id, user_id, content)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (issue_id, user_id, content)
            WHERE issue_id IS NOT NULL
            DO NOTHING
            "#,
        )
        .bind(repository_id)
        .bind(issue_id)
        .bind(user_id)
        .bind(content.as_str())
        .execute(pool)
        .await?;
    }

    reaction_summaries(pool, Some(issue_id), None, Some(user_id)).await
}

pub async fn issue_timeline(
    pool: &PgPool,
    issue_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<Vec<TimelineEvent>, CollaborationError> {
    require_issue_read_access(pool, issue_id, actor_user_id).await?;
    let rows = sqlx::query(
        r#"
        SELECT id, repository_id, issue_id, pull_request_id, actor_user_id, event_type, metadata, created_at
        FROM timeline_events
        WHERE issue_id = $1
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(issue_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(timeline_event_from_row).collect())
}

pub async fn issue_timeline_view(
    pool: &PgPool,
    issue_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<Vec<IssueTimelineItem>, CollaborationError> {
    require_issue_read_access(pool, issue_id, actor_user_id).await?;
    let rows = sqlx::query(
        r#"
        SELECT timeline_events.id, timeline_events.event_type, timeline_events.metadata,
               timeline_events.created_at,
               comments.id AS comment_id, comments.body, comments.is_minimized,
               comments.created_at AS comment_created_at, comments.updated_at AS comment_updated_at,
               users.id AS actor_id, COALESCE(users.username, users.email) AS actor_login,
               users.display_name AS actor_display_name, users.avatar_url AS actor_avatar_url
        FROM timeline_events
        LEFT JOIN comments
          ON timeline_events.event_type = 'commented'
         AND timeline_events.metadata->>'commentId' = comments.id::text
        LEFT JOIN users
          ON users.id = COALESCE(comments.author_user_id, timeline_events.actor_user_id)
        WHERE timeline_events.issue_id = $1
        ORDER BY timeline_events.created_at ASC, timeline_events.id ASC
        "#,
    )
    .bind(issue_id)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(timeline_item_from_row(pool, row, actor_user_id).await?);
    }
    Ok(items)
}

async fn require_issue_read_access(
    pool: &PgPool,
    issue_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<Uuid, CollaborationError> {
    let repository_id = issue_repository_id(pool, issue_id).await?;
    match actor_user_id {
        Some(user_id) => {
            require_repository_role(pool, repository_id, user_id, RepositoryRole::Read).await?;
        }
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
            if repository.visibility != RepositoryVisibility::Public {
                return Err(CollaborationError::RepositoryAccessDenied);
            }
        }
    }
    Ok(repository_id)
}

async fn timeline_item_from_row(
    pool: &PgPool,
    row: sqlx::postgres::PgRow,
    viewer_user_id: Option<Uuid>,
) -> Result<IssueTimelineItem, CollaborationError> {
    let actor_id: Option<Uuid> = row.get("actor_id");
    let comment_id: Option<Uuid> = row.get("comment_id");
    let body: Option<String> = row.get("body");
    let comment = match (comment_id, body) {
        (Some(comment_id), Some(body)) => {
            let rendered = render_markdown(
                Some(pool),
                RenderMarkdownInput {
                    markdown: body.clone(),
                    repository_id: None,
                    owner: None,
                    repo: None,
                    ref_name: None,
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
                        message: "comment body could not be rendered".to_owned(),
                    }
                }
            })?;
            let reactions =
                reaction_summaries(pool, None, Some(comment_id), viewer_user_id).await?;
            Some(IssueTimelineComment {
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

    Ok(IssueTimelineItem {
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

async fn issue_subscription_state(
    pool: &PgPool,
    issue_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<IssueSubscriptionState, CollaborationError> {
    let Some(user_id) = actor_user_id else {
        return Ok(IssueSubscriptionState {
            subscribed: false,
            reason: "anonymous".to_owned(),
            custom_events: Vec::new(),
            can_customize: false,
        });
    };

    let row = sqlx::query(
        r#"
        SELECT subscribed, reason, custom_events
        FROM issue_subscriptions
        WHERE issue_id = $1 AND user_id = $2
        "#,
    )
    .bind(issue_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(IssueSubscriptionState {
            subscribed: false,
            reason: "not_subscribed".to_owned(),
            custom_events: Vec::new(),
            can_customize: true,
        });
    };

    Ok(IssueSubscriptionState {
        subscribed: row.get("subscribed"),
        reason: row.get("reason"),
        custom_events: row.get("custom_events"),
        can_customize: true,
    })
}

pub(crate) fn normalize_thread_subscription_events(
    events: &[String],
) -> Result<Vec<String>, CollaborationError> {
    let mut normalized = Vec::new();
    for event in events {
        let event = event.trim().to_ascii_lowercase();
        let event = event.replace('-', "_");
        if event.is_empty() || normalized.iter().any(|existing| existing == &event) {
            continue;
        }
        if !matches!(event.as_str(), "closed" | "reopened" | "merged") {
            return Err(CollaborationError::InvalidIssueField {
                field_key: "customEvents".to_owned(),
                message: format!("unsupported notification event `{event}`"),
            });
        }
        normalized.push(event);
    }
    Ok(normalized)
}

pub(crate) async fn reaction_summaries(
    pool: &PgPool,
    issue_id: Option<Uuid>,
    comment_id: Option<Uuid>,
    actor_user_id: Option<Uuid>,
) -> Result<Vec<ReactionSummary>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT content,
               count(*)::bigint AS count,
               bool_or(user_id = $3) AS viewer_reacted
        FROM reactions
        WHERE (($1::uuid IS NOT NULL AND issue_id = $1)
            OR ($2::uuid IS NOT NULL AND comment_id = $2))
        GROUP BY content
        ORDER BY content
        "#,
    )
    .bind(issue_id)
    .bind(comment_id)
    .bind(actor_user_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let content: String = row.get("content");
            Ok(ReactionSummary {
                content: ReactionContent::try_from(content.as_str())?,
                count: row.get("count"),
                viewer_reacted: row
                    .get::<Option<bool>, _>("viewer_reacted")
                    .unwrap_or(false),
            })
        })
        .collect()
}

async fn notify_issue_participants(
    pool: &PgPool,
    issue: &Issue,
    actor_user_id: Uuid,
    reason: &str,
    title: String,
) -> Result<(), CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT user_id
        FROM (
            SELECT author_user_id AS user_id FROM issues WHERE id = $1
            UNION
            SELECT author_user_id AS user_id FROM comments WHERE issue_id = $1
            UNION
            SELECT user_id FROM issue_subscriptions
            WHERE issue_id = $1 AND subscribed = true
        ) participants
        WHERE user_id <> $2
        "#,
    )
    .bind(issue.id)
    .bind(actor_user_id)
    .fetch_all(pool)
    .await?;

    for row in rows {
        let user_id: Uuid = row.get("user_id");
        if !should_deliver_notification(
            pool,
            NotificationDeliveryCheck {
                user_id,
                repository_id: issue.repository_id,
                subject_type: "issue".to_owned(),
                subject_id: Some(issue.id),
                reason: reason.to_owned(),
                repository_event: Some(RepositoryWatchEvent::Issues),
                actor_user_id: Some(actor_user_id),
                participating: true,
                direct: false,
            },
        )
        .await
        .map_err(|error| match error {
            super::notifications::NotificationError::Sqlx(error) => CollaborationError::Sqlx(error),
            super::notifications::NotificationError::NotFound => CollaborationError::IssueNotFound,
            super::notifications::NotificationError::Validation(_) => {
                CollaborationError::IssueNotFound
            }
        })? {
            continue;
        }
        create_notification(
            pool,
            CreateNotification {
                user_id,
                repository_id: Some(issue.repository_id),
                subject_type: "issue".to_owned(),
                subject_id: Some(issue.id),
                title: title.clone(),
                reason: reason.to_owned(),
            },
        )
        .await
        .map_err(|error| match error {
            super::notifications::NotificationError::Sqlx(error) => CollaborationError::Sqlx(error),
            super::notifications::NotificationError::NotFound => CollaborationError::IssueNotFound,
            super::notifications::NotificationError::Validation(_) => {
                CollaborationError::IssueNotFound
            }
        })?;
    }

    Ok(())
}

pub async fn repository_for_actor(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    actor_user_id: Uuid,
    required_role: RepositoryRole,
) -> Result<Repository, CollaborationError> {
    let repository = get_repository_by_owner_name(pool, owner_login, repo_name)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    require_repository_role(pool, repository.id, actor_user_id, required_role).await?;
    Ok(repository)
}

pub(crate) async fn insert_issue_with_number(
    pool: &PgPool,
    input: CreateIssue,
    number: i64,
) -> Result<Issue, CollaborationError> {
    get_repository(pool, input.repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    ensure_default_labels(pool, input.repository_id).await?;

    let row = sqlx::query(
        r#"
        INSERT INTO issues (repository_id, number, title, body, author_user_id, milestone_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, repository_id, number, title, body, state, author_user_id, milestone_id,
                  locked, closed_by_user_id, closed_at, created_at, updated_at
        "#,
    )
    .bind(input.repository_id)
    .bind(number)
    .bind(&input.title)
    .bind(&input.body)
    .bind(input.actor_user_id)
    .bind(input.milestone_id)
    .fetch_one(pool)
    .await?;
    let issue = issue_from_row(row)?;

    for label_id in input.label_ids {
        sqlx::query(
            r#"
            INSERT INTO issue_labels (issue_id, label_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(issue.id)
        .bind(label_id)
        .execute(pool)
        .await?;
    }

    for assignee_user_id in input.assignee_user_ids {
        sqlx::query(
            r#"
            INSERT INTO issue_assignees (issue_id, user_id, assigned_by_user_id)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(issue.id)
        .bind(assignee_user_id)
        .bind(issue.author_user_id)
        .execute(pool)
        .await?;
    }

    Ok(issue)
}

async fn insert_issue_body_version(pool: &PgPool, issue: &Issue) -> Result<(), CollaborationError> {
    sqlx::query(
        r#"
        INSERT INTO issue_body_versions (issue_id, editor_user_id, body, version)
        VALUES ($1, $2, $3, 1)
        "#,
    )
    .bind(issue.id)
    .bind(issue.author_user_id)
    .bind(issue.body.as_deref().unwrap_or(""))
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_issue_attachments(
    pool: &PgPool,
    issue: &Issue,
    attachments: &[CreateIssueAttachment],
) -> Result<(), CollaborationError> {
    for attachment in attachments {
        sqlx::query(
            r#"
            INSERT INTO issue_attachments (
                issue_id, uploader_user_id, file_name, byte_size, content_type, storage_status
            )
            VALUES ($1, $2, $3, $4, $5, 'metadata_only')
            "#,
        )
        .bind(issue.id)
        .bind(issue.author_user_id)
        .bind(attachment.file_name.trim())
        .bind(attachment.byte_size)
        .bind(attachment.content_type.as_deref())
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn notify_issue_assignees(
    pool: &PgPool,
    issue: &Issue,
    actor_user_id: Uuid,
    assignee_user_ids: &[Uuid],
) -> Result<(), CollaborationError> {
    for assignee_user_id in assignee_user_ids {
        if *assignee_user_id == issue.author_user_id {
            continue;
        }
        if !should_deliver_notification(
            pool,
            NotificationDeliveryCheck {
                user_id: *assignee_user_id,
                repository_id: issue.repository_id,
                subject_type: "issue".to_owned(),
                subject_id: Some(issue.id),
                reason: "assigned".to_owned(),
                repository_event: Some(RepositoryWatchEvent::Issues),
                actor_user_id: Some(actor_user_id),
                participating: false,
                direct: true,
            },
        )
        .await
        .map_err(|error| match error {
            super::notifications::NotificationError::Sqlx(error) => CollaborationError::Sqlx(error),
            super::notifications::NotificationError::NotFound => CollaborationError::IssueNotFound,
            super::notifications::NotificationError::Validation(_) => {
                CollaborationError::IssueNotFound
            }
        })? {
            continue;
        }
        create_notification(
            pool,
            CreateNotification {
                user_id: *assignee_user_id,
                repository_id: Some(issue.repository_id),
                subject_type: "issue".to_owned(),
                subject_id: Some(issue.id),
                title: format!("Assigned to issue #{}: {}", issue.number, issue.title),
                reason: "assigned".to_owned(),
            },
        )
        .await
        .map_err(|error| match error {
            super::notifications::NotificationError::Sqlx(error) => CollaborationError::Sqlx(error),
            super::notifications::NotificationError::NotFound => CollaborationError::IssueNotFound,
            super::notifications::NotificationError::Validation(_) => {
                CollaborationError::IssueNotFound
            }
        })?;
    }
    Ok(())
}

pub(crate) async fn next_issue_number(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<i64, CollaborationError> {
    let next = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(max(number), 0) + 1
        FROM issues
        WHERE repository_id = $1
        "#,
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    Ok(next)
}

pub(crate) async fn append_timeline_event(
    pool: &PgPool,
    repository_id: Uuid,
    issue_id: Option<Uuid>,
    pull_request_id: Option<Uuid>,
    actor_user_id: Option<Uuid>,
    event_type: &str,
    metadata: serde_json::Value,
) -> Result<TimelineEvent, CollaborationError> {
    let row = sqlx::query(
        r#"
        INSERT INTO timeline_events (
            repository_id,
            issue_id,
            pull_request_id,
            actor_user_id,
            event_type,
            metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, repository_id, issue_id, pull_request_id, actor_user_id, event_type, metadata, created_at
        "#,
    )
    .bind(repository_id)
    .bind(issue_id)
    .bind(pull_request_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(metadata)
    .fetch_one(pool)
    .await?;

    Ok(timeline_event_from_row(row))
}

async fn issue_repository_id(pool: &PgPool, issue_id: Uuid) -> Result<Uuid, CollaborationError> {
    sqlx::query_scalar::<_, Uuid>("SELECT repository_id FROM issues WHERE id = $1")
        .bind(issue_id)
        .fetch_optional(pool)
        .await?
        .ok_or(CollaborationError::IssueNotFound)
}

async fn issue_converted_discussion(
    pool: &PgPool,
    issue_id: Uuid,
) -> Result<Option<(i64, String)>, CollaborationError> {
    let row = sqlx::query(
        r#"
        SELECT discussions.number,
               COALESCE(users.username, organizations.slug) AS owner_login,
               repositories.name
        FROM issues
        JOIN discussions ON discussions.id = issues.converted_discussion_id
        JOIN repositories ON repositories.id = discussions.repository_id
        LEFT JOIN users ON users.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE issues.id = $1
        "#,
    )
    .bind(issue_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| {
        let number: i64 = row.get("number");
        let owner: String = row.get("owner_login");
        let name: String = row.get("name");
        (number, format!("/{owner}/{name}/discussions/{number}"))
    }))
}

async fn issue_conversion_categories(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<IssueDiscussionConversionCategory>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT id, slug, name, emoji, description, format
        FROM discussion_categories
        WHERE repository_id = $1
        ORDER BY position ASC, lower(name) ASC
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let format: String = row
                .try_get("format")
                .unwrap_or_else(|_| "discussion".to_owned());
            IssueDiscussionConversionCategory {
                id: row.get("id"),
                slug: row.get("slug"),
                name: row.get("name"),
                emoji: row.get("emoji"),
                description: row.get("description"),
                disabled_reason: if format == "poll" {
                    Some("Poll categories cannot receive converted issues.".to_owned())
                } else {
                    None
                },
            }
        })
        .collect())
}

async fn issue_comment_count(pool: &PgPool, issue_id: Uuid) -> Result<i64, CollaborationError> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM comments WHERE issue_id = $1 AND is_minimized = false",
    )
    .bind(issue_id)
    .fetch_one(pool)
    .await
    .map_err(CollaborationError::Sqlx)
}

async fn issue_by_id(pool: &PgPool, issue_id: Uuid) -> Result<Issue, CollaborationError> {
    let row = sqlx::query(
        r#"
        SELECT id, repository_id, number, title, body, state, author_user_id, milestone_id,
               locked, closed_by_user_id, closed_at, created_at, updated_at
        FROM issues
        WHERE id = $1
          AND NOT EXISTS (
              SELECT 1 FROM pull_requests WHERE pull_requests.issue_id = issues.id
          )
        "#,
    )
    .bind(issue_id)
    .fetch_optional(pool)
    .await?
    .ok_or(CollaborationError::IssueNotFound)?;

    issue_from_row(row)
}

async fn require_repository_role(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
    required_role: RepositoryRole,
) -> Result<(), CollaborationError> {
    let repository = get_repository(pool, repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    repository_viewer_permission(pool, &repository, user_id, required_role).await?;
    Ok(())
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

async fn repository_issue_preferences_row(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
) -> Result<IssueListPreferences, CollaborationError> {
    let dismissed_contributor_banner_at = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
        r#"
        SELECT dismissed_contributor_banner_at
        FROM repository_issue_preferences
        WHERE repository_id = $1 AND user_id = $2
        "#,
    )
    .bind(repository_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .flatten();

    Ok(IssueListPreferences {
        dismissed_contributor_banner: dismissed_contributor_banner_at.is_some(),
        dismissed_contributor_banner_at,
    })
}

struct IssueListCountFilters<'a> {
    text_filter: Option<&'a str>,
    author_filter: Option<&'a str>,
    excluded_author_filter: Option<&'a str>,
    label_filters: &'a [String],
    excluded_label_filters: &'a [String],
    no_labels: bool,
    milestone_filter: Option<&'a str>,
    no_milestone: bool,
    assignee_filter: Option<&'a str>,
    no_assignee: bool,
    project_filter: Option<&'a str>,
    issue_type_filter: Option<&'a str>,
}

async fn count_issue_list_items(
    pool: &PgPool,
    repository_id: Uuid,
    state: &str,
    filters: &IssueListCountFilters<'_>,
) -> Result<i64, CollaborationError> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM issues
        WHERE issues.repository_id = $1
          AND issues.state = $2
          AND NOT EXISTS (
              SELECT 1 FROM pull_requests WHERE pull_requests.issue_id = issues.id
          )
          AND (
              $3::text IS NULL
              OR issues.title ILIKE '%' || $3 || '%'
              OR COALESCE(issues.body, '') ILIKE '%' || $3 || '%'
          )
          AND (
              $4::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM users
                  WHERE users.id = issues.author_user_id
                    AND (
                        lower(COALESCE(users.username, users.email)) = lower($4)
                        OR lower(users.email) = lower($4)
                    )
              )
          )
          AND (
              $5::text IS NULL
              OR NOT EXISTS (
                  SELECT 1
                  FROM users
                  WHERE users.id = issues.author_user_id
                    AND (
                        lower(COALESCE(users.username, users.email)) = lower($5)
                        OR lower(users.email) = lower($5)
                    )
              )
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
              cardinality($7::text[]) = 0
              OR NOT EXISTS (
                  SELECT 1
                  FROM issue_labels
                  JOIN labels ON labels.id = issue_labels.label_id
                  JOIN unnest($7::text[]) blocked_label(name)
                    ON lower(labels.name) = lower(blocked_label.name)
                  WHERE issue_labels.issue_id = issues.id
              )
          )
          AND (
              $8::boolean = false
              OR NOT EXISTS (
                  SELECT 1 FROM issue_labels WHERE issue_labels.issue_id = issues.id
              )
          )
          AND (
              $9::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM milestones
                  WHERE milestones.id = issues.milestone_id
                    AND lower(milestones.title) = lower($9)
              )
          )
          AND (
              $10::boolean = false
              OR issues.milestone_id IS NULL
          )
          AND (
              $11::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM issue_assignees
                  JOIN users ON users.id = issue_assignees.user_id
                  WHERE issue_assignees.issue_id = issues.id
                    AND (
                        lower(COALESCE(users.username, users.email)) = lower($11)
                        OR lower(users.email) = lower($11)
                    )
              )
          )
          AND (
              $12::boolean = false
              OR NOT EXISTS (
                  SELECT 1 FROM issue_assignees WHERE issue_assignees.issue_id = issues.id
              )
          )
          AND $13::text IS NULL
          AND (
              $14::text IS NULL
              OR lower($14) IN ('issue', 'issues')
          )
        "#,
    )
    .bind(repository_id)
    .bind(state)
    .bind(filters.text_filter)
    .bind(filters.author_filter)
    .bind(filters.excluded_author_filter)
    .bind(filters.label_filters)
    .bind(filters.excluded_label_filters)
    .bind(filters.no_labels)
    .bind(filters.milestone_filter)
    .bind(filters.no_milestone)
    .bind(filters.assignee_filter)
    .bind(filters.no_assignee)
    .bind(filters.project_filter)
    .bind(filters.issue_type_filter)
    .fetch_one(pool)
    .await
    .map_err(CollaborationError::from)
}

async fn global_issue_list_items(
    pool: &PgPool,
    issues: Vec<Issue>,
) -> Result<Vec<IssueListItem>, CollaborationError> {
    let mut by_repository: HashMap<Uuid, Vec<Issue>> = HashMap::new();
    let mut order = Vec::new();
    for issue in issues {
        if !by_repository.contains_key(&issue.repository_id) {
            order.push(issue.repository_id);
        }
        by_repository
            .entry(issue.repository_id)
            .or_default()
            .push(issue);
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
        let issues = by_repository.remove(repository_id).unwrap_or_default();
        grouped_items.insert(
            *repository_id,
            issue_list_items_for_issues(pool, &repository, issues).await?,
        );
    }

    let mut items = Vec::new();
    for repository_id in order {
        items.extend(grouped_items.remove(&repository_id).unwrap_or_default());
    }
    Ok(items)
}

async fn count_global_issue_list_items(
    pool: &PgPool,
    actor_user_id: Uuid,
    scope: &str,
    state_filter: Option<&str>,
    text_filter: Option<&str>,
    filters: &GlobalIssueListQuery,
) -> Result<i64, CollaborationError> {
    Ok(sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM issues
        JOIN repositories ON repositories.id = issues.repository_id
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
          AND NOT EXISTS (
              SELECT 1 FROM pull_requests WHERE pull_requests.issue_id = issues.id
          )
          AND (
              ($2 = 'created' AND issues.author_user_id = $1)
              OR ($2 = 'assigned' AND EXISTS (
                  SELECT 1 FROM issue_assignees
                  WHERE issue_assignees.issue_id = issues.id
                    AND issue_assignees.user_id = $1
              ))
              OR ($2 = 'mentioned' AND EXISTS (
                  SELECT 1 FROM notifications
                  WHERE notifications.subject_type = 'issue'
                    AND notifications.subject_id = issues.id
                    AND notifications.user_id = $1
                    AND notifications.reason IN ('mention', 'team_mention')
              ))
          )
          AND ($3::text IS NULL OR issues.state = $3)
          AND (
              $4::text IS NULL
              OR issues.title ILIKE '%' || $4 || '%'
              OR COALESCE(issues.body, '') ILIKE '%' || $4 || '%'
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
                  SELECT 1 FROM milestones
                  WHERE milestones.id = issues.milestone_id
                    AND lower(milestones.title) = lower($7)
              )
          )
          AND (
              $8::text IS NULL
              OR EXISTS (
                  SELECT 1
                  FROM project_items
                  JOIN projects ON projects.id = project_items.project_id
                  WHERE project_items.issue_id = issues.id
                    AND project_items.archived_at IS NULL
                    AND lower(projects.title) = lower($8)
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
    .bind(filters.project.as_deref())
    .fetch_one(pool)
    .await?)
}

async fn global_issue_repository_options(
    pool: &PgPool,
    actor_user_id: Uuid,
    scope: &str,
) -> Result<Vec<GlobalIssueRepositoryOption>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT repositories.id,
               COALESCE(owner_users.username, owner_users.email, owner_orgs.slug) AS owner_login,
               repositories.name,
               count(*) AS count
        FROM issues
        JOIN repositories ON repositories.id = issues.repository_id
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
          AND NOT EXISTS (SELECT 1 FROM pull_requests WHERE pull_requests.issue_id = issues.id)
          AND (
              ($2 = 'created' AND issues.author_user_id = $1)
              OR ($2 = 'assigned' AND EXISTS (
                  SELECT 1 FROM issue_assignees WHERE issue_assignees.issue_id = issues.id AND issue_assignees.user_id = $1
              ))
              OR ($2 = 'mentioned' AND EXISTS (
                  SELECT 1 FROM notifications WHERE notifications.subject_type = 'issue' AND notifications.subject_id = issues.id AND notifications.user_id = $1 AND notifications.reason IN ('mention', 'team_mention')
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
            GlobalIssueRepositoryOption {
                id: row.get("id"),
                full_name: format!("{owner_login}/{name}"),
                owner_login,
                name,
                count: row.get("count"),
            }
        })
        .collect())
}

async fn global_issue_label_options(
    pool: &PgPool,
    actor_user_id: Uuid,
    scope: &str,
) -> Result<Vec<IssueListLabel>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT labels.id, labels.name, labels.color, labels.description, count(*) AS uses
        FROM issues
        JOIN repositories ON repositories.id = issues.repository_id
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
          AND NOT EXISTS (SELECT 1 FROM pull_requests WHERE pull_requests.issue_id = issues.id)
          AND (
              ($2 = 'created' AND issues.author_user_id = $1)
              OR ($2 = 'assigned' AND EXISTS (
                  SELECT 1 FROM issue_assignees WHERE issue_assignees.issue_id = issues.id AND issue_assignees.user_id = $1
              ))
              OR ($2 = 'mentioned' AND EXISTS (
                  SELECT 1 FROM notifications WHERE notifications.subject_type = 'issue' AND notifications.subject_id = issues.id AND notifications.user_id = $1 AND notifications.reason IN ('mention', 'team_mention')
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

async fn global_issue_milestone_options(
    pool: &PgPool,
    actor_user_id: Uuid,
    scope: &str,
) -> Result<Vec<IssueListMilestone>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT milestones.id, milestones.title, milestones.state, count(*) AS issue_count
        FROM issues
        JOIN repositories ON repositories.id = issues.repository_id
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
          AND NOT EXISTS (SELECT 1 FROM pull_requests WHERE pull_requests.issue_id = issues.id)
          AND (
              ($2 = 'created' AND issues.author_user_id = $1)
              OR ($2 = 'assigned' AND EXISTS (
                  SELECT 1 FROM issue_assignees WHERE issue_assignees.issue_id = issues.id AND issue_assignees.user_id = $1
              ))
              OR ($2 = 'mentioned' AND EXISTS (
                  SELECT 1 FROM notifications WHERE notifications.subject_type = 'issue' AND notifications.subject_id = issues.id AND notifications.user_id = $1 AND notifications.reason IN ('mention', 'team_mention')
              ))
          )
        GROUP BY milestones.id, milestones.title, milestones.state
        ORDER BY issue_count DESC, lower(milestones.title) ASC
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

async fn global_issue_project_options(
    pool: &PgPool,
    actor_user_id: Uuid,
    scope: &str,
) -> Result<Vec<IssueListMetadataOption>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT projects.id, projects.title AS name, projects.short_description AS description,
               count(*) AS issue_count
        FROM issues
        JOIN repositories ON repositories.id = issues.repository_id
        JOIN project_items ON project_items.issue_id = issues.id
        JOIN projects ON projects.id = project_items.project_id
        WHERE project_items.archived_at IS NULL
          AND (
              repositories.visibility = 'public'
              OR EXISTS (
                  SELECT 1 FROM repository_permissions
                  WHERE repository_permissions.repository_id = repositories.id
                    AND repository_permissions.user_id = $1
                    AND repository_permissions.role IN ('owner', 'admin', 'maintain', 'write', 'triage', 'read')
              )
          )
          AND NOT EXISTS (SELECT 1 FROM pull_requests WHERE pull_requests.issue_id = issues.id)
          AND (
              ($2 = 'created' AND issues.author_user_id = $1)
              OR ($2 = 'assigned' AND EXISTS (
                  SELECT 1 FROM issue_assignees WHERE issue_assignees.issue_id = issues.id AND issue_assignees.user_id = $1
              ))
              OR ($2 = 'mentioned' AND EXISTS (
                  SELECT 1 FROM notifications WHERE notifications.subject_type = 'issue' AND notifications.subject_id = issues.id AND notifications.user_id = $1 AND notifications.reason IN ('mention', 'team_mention')
              ))
          )
        GROUP BY projects.id, projects.title, projects.short_description
        ORDER BY issue_count DESC, lower(projects.title) ASC
        LIMIT 50
        "#,
    )
    .bind(actor_user_id)
    .bind(scope)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| IssueListMetadataOption {
            id: row.get::<Uuid, _>("id").to_string(),
            name: row.get("name"),
            description: row.get("description"),
            count: row.get("issue_count"),
            disabled_reason: None,
        })
        .collect())
}

async fn issue_list_items_for_issues(
    pool: &PgPool,
    repository: &Repository,
    issues: Vec<Issue>,
) -> Result<Vec<IssueListItem>, CollaborationError> {
    let issue_ids = issues.iter().map(|issue| issue.id).collect::<Vec<_>>();
    let authors = issue_list_users(pool, &issue_ids, "author").await?;
    let labels = issue_list_labels(pool, &issue_ids).await?;
    let milestones = issue_list_milestones(pool, &issue_ids).await?;
    let assignees = issue_list_assignees(pool, &issue_ids).await?;
    let comment_counts = issue_comment_counts(pool, &issue_ids).await?;
    let linked_pull_requests = linked_pull_request_hints(pool, &issue_ids, repository).await?;

    Ok(issues
        .into_iter()
        .map(|issue| IssueListItem {
            id: issue.id,
            repository_id: issue.repository_id,
            repository_owner: repository.owner_login.clone(),
            repository_name: repository.name.clone(),
            number: issue.number,
            title: issue.title,
            body: issue.body,
            state: issue.state,
            author: authors
                .get(&issue.id)
                .cloned()
                .unwrap_or_else(|| fallback_issue_user(issue.author_user_id)),
            labels: labels.get(&issue.id).cloned().unwrap_or_default(),
            milestone: milestones.get(&issue.id).cloned(),
            assignees: assignees.get(&issue.id).cloned().unwrap_or_default(),
            comment_count: *comment_counts.get(&issue.id).unwrap_or(&0),
            linked_pull_request: linked_pull_requests.get(&issue.id).cloned(),
            href: format!(
                "/{}/{}/issues/{}",
                repository.owner_login, repository.name, issue.number
            ),
            locked: issue.locked,
            created_at: issue.created_at,
            updated_at: issue.updated_at,
            closed_at: issue.closed_at,
        })
        .collect())
}

async fn issue_list_users(
    pool: &PgPool,
    issue_ids: &[Uuid],
    role: &str,
) -> Result<HashMap<Uuid, IssueListUser>, CollaborationError> {
    if issue_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT issues.id AS issue_id, users.id, COALESCE(users.username, users.email) AS login,
               users.display_name, users.avatar_url
        FROM issues
        JOIN users ON users.id = issues.author_user_id
        WHERE issues.id = ANY($1)
          AND $2 = 'author'
        "#,
    )
    .bind(issue_ids)
    .bind(role)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            (
                row.get("issue_id"),
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

async fn issue_list_labels(
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

async fn issue_list_label_options(
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

async fn issue_list_user_options(
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
            SELECT issues.author_user_id
            FROM issues
            WHERE issues.repository_id = $1
            UNION
            SELECT issue_assignees.user_id
            FROM issue_assignees
            JOIN issues ON issues.id = issue_assignees.issue_id
            WHERE issues.repository_id = $1
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

async fn issue_list_milestone_options(
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

async fn issue_list_milestones(
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

async fn issue_list_assignees(
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

async fn issue_comment_counts(
    pool: &PgPool,
    issue_ids: &[Uuid],
) -> Result<HashMap<Uuid, i64>, CollaborationError> {
    if issue_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT issue_id, count(*) AS count
        FROM comments
        WHERE issue_id = ANY($1)
        GROUP BY issue_id
        "#,
    )
    .bind(issue_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.get("issue_id"), row.get("count")))
        .collect())
}

async fn linked_pull_request_hints(
    pool: &PgPool,
    issue_ids: &[Uuid],
    repository: &Repository,
) -> Result<HashMap<Uuid, LinkedPullRequestHint>, CollaborationError> {
    if issue_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT issues.id AS issue_id, pull_requests.number, pull_requests.state
        FROM issue_cross_references
        JOIN pull_requests ON pull_requests.issue_id = issue_cross_references.source_issue_id
        JOIN issues ON issues.id = issue_cross_references.target_issue_id
        WHERE issue_cross_references.target_issue_id = ANY($1)
        ORDER BY pull_requests.updated_at DESC, pull_requests.number DESC
        "#,
    )
    .bind(issue_ids)
    .fetch_all(pool)
    .await?;
    let mut hints = HashMap::new();
    for row in rows {
        let issue_id = row.get("issue_id");
        hints.entry(issue_id).or_insert_with(|| {
            let number = row.get("number");
            LinkedPullRequestHint {
                number,
                state: row.get("state"),
                href: format!(
                    "/{}/{}/pull/{}",
                    repository.owner_login, repository.name, number
                ),
            }
        });
    }
    Ok(hints)
}

async fn issue_detail_participants(
    pool: &PgPool,
    issue_id: Uuid,
) -> Result<Vec<IssueListUser>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT ON (users.id)
               users.id, COALESCE(users.username, users.email) AS login,
               users.display_name, users.avatar_url
        FROM users
        WHERE users.id IN (
            SELECT author_user_id FROM issues WHERE id = $1
            UNION
            SELECT user_id FROM issue_assignees WHERE issue_id = $1
            UNION
            SELECT author_user_id FROM comments WHERE issue_id = $1
            UNION
            SELECT actor_user_id FROM timeline_events
            WHERE issue_id = $1 AND actor_user_id IS NOT NULL
        )
        ORDER BY users.id, lower(COALESCE(users.username, users.email))
        "#,
    )
    .bind(issue_id)
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

async fn issue_detail_attachments(
    pool: &PgPool,
    issue_id: Uuid,
) -> Result<Vec<IssueAttachmentMetadata>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT id, file_name, byte_size, content_type, storage_status, created_at
        FROM issue_attachments
        WHERE issue_id = $1
        ORDER BY created_at ASC, file_name ASC
        "#,
    )
    .bind(issue_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| IssueAttachmentMetadata {
            id: row.get("id"),
            file_name: row.get("file_name"),
            byte_size: row.get("byte_size"),
            content_type: row.get("content_type"),
            storage_status: row.get("storage_status"),
            created_at: row.get("created_at"),
        })
        .collect())
}

fn search_text_from_issue_query(query: &str) -> String {
    issue_query_terms(query)
        .into_iter()
        .filter(|term| {
            !matches!(
                term.as_str(),
                "is:issue" | "is:open" | "is:closed" | "state:open" | "state:closed"
            ) && !term.starts_with("label:")
                && !term.starts_with("-label:")
                && term != "no:label"
                && term != "no:labels"
                && term != "no:assignee"
                && term != "no:milestone"
                && !term.starts_with("author:")
                && !term.starts_with("-author:")
                && !term.starts_with("milestone:")
                && !term.starts_with("assignee:")
                && !term.starts_with("project:")
                && !term.starts_with("type:")
                && !term.starts_with("sort:")
                && !term.starts_with("order:")
        })
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_owned()
}

fn issue_query_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut rest = query.trim();
    while !rest.is_empty() {
        let next_space = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let term = &rest[..next_space];
        if let Some(quote_index) = term.find(":\"") {
            let prefix_len = quote_index + 2;
            let quoted_rest = &rest[prefix_len..];
            if let Some(end_quote) = quoted_rest.find('"') {
                terms.push(format!(
                    "{}{}",
                    &term[..prefix_len],
                    &quoted_rest[..end_quote + 1]
                ));
                rest = quoted_rest[end_quote + 1..].trim_start();
            } else {
                terms.push(rest.to_owned());
                break;
            }
        } else {
            terms.push(term.to_owned());
            rest = rest[next_space..].trim_start();
        }
    }
    terms
}

fn fallback_issue_user(user_id: Uuid) -> IssueListUser {
    IssueListUser {
        id: user_id,
        login: "unknown".to_owned(),
        display_name: None,
        avatar_url: None,
    }
}

fn label_from_row(row: sqlx::postgres::PgRow) -> Label {
    Label {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        name: row.get("name"),
        color: row.get("color"),
        description: row.get("description"),
        is_default: row.get("is_default"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn issue_template_from_row(row: sqlx::postgres::PgRow) -> IssueTemplate {
    IssueTemplate {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        slug: row.get("slug"),
        name: row.get("name"),
        description: row.get("description"),
        title_prefill: row.get("title_prefill"),
        body: row.get("body"),
        issue_type: row.get("issue_type"),
        form_fields: Vec::new(),
        default_label_ids: Vec::new(),
        default_assignee_user_ids: Vec::new(),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn issue_form_field_from_row(row: sqlx::postgres::PgRow) -> IssueFormField {
    IssueFormField {
        id: row.get("id"),
        template_id: row.get("template_id"),
        field_key: row.get("field_key"),
        label: row.get("label"),
        field_type: row.get("field_type"),
        description: row.get("description"),
        placeholder: row.get("placeholder"),
        value: row.get("value"),
        required: row.get("required"),
        display_order: row.get("display_order"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

async fn issue_template_for_create(
    pool: &PgPool,
    repository_id: Uuid,
    template_id: Option<Uuid>,
    template_slug: Option<&str>,
) -> Result<IssueTemplate, CollaborationError> {
    let template_slug = template_slug
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let row = sqlx::query(
        r#"
        SELECT id, repository_id, slug, name, description, title_prefill, body, issue_type,
               created_at, updated_at
        FROM issue_templates
        WHERE repository_id = $1
          AND (
              ($2::uuid IS NOT NULL AND id = $2)
              OR ($2::uuid IS NULL AND $3::text IS NOT NULL AND lower(slug) = lower($3))
          )
        "#,
    )
    .bind(repository_id)
    .bind(template_id)
    .bind(template_slug)
    .fetch_optional(pool)
    .await?;
    let mut template =
        row.map(issue_template_from_row)
            .ok_or(CollaborationError::InvalidIssueField {
                field_key: "template".to_owned(),
                message: "issue template was not found".to_owned(),
            })?;
    hydrate_issue_template_defaults(pool, std::slice::from_mut(&mut template)).await?;
    hydrate_issue_template_fields(pool, std::slice::from_mut(&mut template)).await?;
    Ok(template)
}

fn validate_required_issue_fields(
    template: &IssueTemplate,
    field_values: &HashMap<String, String>,
) -> Result<(), CollaborationError> {
    for field in &template.form_fields {
        if field.required {
            let value = field_values
                .get(&field.field_key)
                .or(field.value.as_ref())
                .map(String::as_str)
                .unwrap_or("");
            if value.trim().is_empty() {
                return Err(CollaborationError::InvalidIssueField {
                    field_key: field.field_key.clone(),
                    message: format!("{} is required", field.label),
                });
            }
        }
    }
    Ok(())
}

fn compose_issue_body_from_fields(
    base_body: Option<&str>,
    fields: &[IssueFormField],
    field_values: &HashMap<String, String>,
) -> String {
    let mut sections = Vec::new();
    if let Some(body) = base_body.map(str::trim).filter(|value| !value.is_empty()) {
        sections.push(body.to_owned());
    }

    for field in fields {
        let value = field_values
            .get(&field.field_key)
            .or(field.value.as_ref())
            .map(String::as_str)
            .unwrap_or("")
            .trim();
        if value.is_empty() {
            continue;
        }
        sections.push(format!("### {}\n\n{}", field.label, value));
    }

    sections.join("\n\n")
}

async fn hydrate_issue_template_defaults(
    pool: &PgPool,
    templates: &mut [IssueTemplate],
) -> Result<(), CollaborationError> {
    if templates.is_empty() {
        return Ok(());
    }

    let template_ids = templates
        .iter()
        .map(|template| template.id)
        .collect::<Vec<_>>();
    let index = templates
        .iter()
        .enumerate()
        .map(|(position, template)| (template.id, position))
        .collect::<HashMap<_, _>>();

    let label_rows = sqlx::query(
        r#"
        SELECT template_id, label_id
        FROM issue_template_default_labels
        WHERE template_id = ANY($1)
        ORDER BY created_at ASC
        "#,
    )
    .bind(&template_ids)
    .fetch_all(pool)
    .await?;
    for row in label_rows {
        let template_id: Uuid = row.get("template_id");
        let label_id: Uuid = row.get("label_id");
        if let Some(position) = index.get(&template_id).copied() {
            templates[position].default_label_ids.push(label_id);
        }
    }

    let assignee_rows = sqlx::query(
        r#"
        SELECT template_id, user_id
        FROM issue_template_default_assignees
        WHERE template_id = ANY($1)
        ORDER BY created_at ASC
        "#,
    )
    .bind(&template_ids)
    .fetch_all(pool)
    .await?;
    for row in assignee_rows {
        let template_id: Uuid = row.get("template_id");
        let user_id: Uuid = row.get("user_id");
        if let Some(position) = index.get(&template_id).copied() {
            templates[position].default_assignee_user_ids.push(user_id);
        }
    }

    Ok(())
}

async fn hydrate_issue_template_fields(
    pool: &PgPool,
    templates: &mut [IssueTemplate],
) -> Result<(), CollaborationError> {
    if templates.is_empty() {
        return Ok(());
    }

    let template_ids = templates
        .iter()
        .map(|template| template.id)
        .collect::<Vec<_>>();
    let index = templates
        .iter()
        .enumerate()
        .map(|(position, template)| (template.id, position))
        .collect::<HashMap<_, _>>();
    let rows = sqlx::query(
        r#"
        SELECT id, template_id, field_key, label, field_type, description, placeholder, value,
               required, display_order, created_at, updated_at
        FROM issue_form_fields
        WHERE template_id = ANY($1)
        ORDER BY display_order ASC, lower(label) ASC
        "#,
    )
    .bind(&template_ids)
    .fetch_all(pool)
    .await?;
    for row in rows {
        let field = issue_form_field_from_row(row);
        if let Some(position) = index.get(&field.template_id).copied() {
            templates[position].form_fields.push(field);
        }
    }
    Ok(())
}

pub(crate) async fn index_issue_search_document(
    pool: &PgPool,
    issue: &Issue,
    actor_user_id: Uuid,
) -> Result<(), CollaborationError> {
    let repository = get_repository(pool, issue.repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => CollaborationError::Sqlx(error),
            _ => CollaborationError::RepositoryNotFound,
        })?
        .ok_or(CollaborationError::RepositoryNotFound)?;
    let labels = labels_for_issue(pool, issue.id).await?;
    let author_login = user_login(pool, issue.author_user_id).await?;
    let label_names = labels
        .iter()
        .map(|label| label.name.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let body = [issue.body.as_deref().unwrap_or(""), label_names.as_str()]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let label_metadata = labels
        .iter()
        .map(|label| json!({ "name": label.name, "color": label.color }))
        .collect::<Vec<_>>();

    upsert_search_document(
        pool,
        actor_user_id,
        UpsertSearchDocument {
            repository_id: Some(repository.id),
            owner_user_id: repository.owner_user_id,
            owner_organization_id: repository.owner_organization_id,
            kind: SearchDocumentKind::Issue,
            resource_id: format!("{}:{}", repository.id, issue.number),
            title: issue.title.clone(),
            body: Some(body),
            path: None,
            language: None,
            branch: None,
            visibility: repository.visibility,
            metadata: json!({
                "number": issue.number,
                "state": issue.state.as_str(),
                "labels": label_metadata,
                "authorLogin": author_login,
                "createdAt": issue.created_at,
                "updatedAt": issue.updated_at,
                "href": format!("/{}/{}/issues/{}", repository.owner_login, repository.name, issue.number),
            }),
        },
    )
    .await
    .map_err(search_error_to_collaboration)?;

    Ok(())
}

pub(crate) async fn user_login(pool: &PgPool, user_id: Uuid) -> Result<String, CollaborationError> {
    sqlx::query_scalar::<_, String>(
        "SELECT COALESCE(NULLIF(username, ''), email) FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(CollaborationError::Sqlx)
}

async fn labels_for_issue(pool: &PgPool, issue_id: Uuid) -> Result<Vec<Label>, CollaborationError> {
    let rows = sqlx::query(
        r#"
        SELECT labels.id, labels.repository_id, labels.name, labels.color, labels.description,
               labels.is_default, labels.created_at, labels.updated_at
        FROM labels
        JOIN issue_labels ON issue_labels.label_id = labels.id
        WHERE issue_labels.issue_id = $1
        ORDER BY lower(labels.name)
        "#,
    )
    .bind(issue_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(label_from_row).collect())
}

pub(crate) fn search_error_to_collaboration(error: SearchError) -> CollaborationError {
    match error {
        SearchError::RepositoryAccessDenied => CollaborationError::RepositoryAccessDenied,
        SearchError::Repository(super::repositories::RepositoryError::Sqlx(error))
        | SearchError::Sqlx(error) => CollaborationError::Sqlx(error),
        SearchError::Repository(_) => CollaborationError::RepositoryNotFound,
        SearchError::QueryTooShort
        | SearchError::InvalidKind(_)
        | SearchError::InvalidIndexStatus(_)
        | SearchError::Validation(_)
        | SearchError::DuplicateSavedSearchName
        | SearchError::SavedSearchNotFound => CollaborationError::RepositoryAccessDenied,
    }
}

pub(crate) fn issue_from_row(row: sqlx::postgres::PgRow) -> Result<Issue, CollaborationError> {
    let state: String = row.get("state");
    Ok(Issue {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        number: row.get("number"),
        title: row.get("title"),
        body: row.get("body"),
        state: IssueState::try_from(state.as_str())?,
        author_user_id: row.get("author_user_id"),
        milestone_id: row.get("milestone_id"),
        locked: row.get("locked"),
        closed_by_user_id: row.get("closed_by_user_id"),
        closed_at: row.get("closed_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub(crate) fn comment_from_row(row: sqlx::postgres::PgRow) -> Comment {
    Comment {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        issue_id: row.get("issue_id"),
        pull_request_id: row.get("pull_request_id"),
        author_user_id: row.get("author_user_id"),
        body: row.get("body"),
        is_minimized: row.get("is_minimized"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

pub(crate) fn timeline_event_from_row(row: sqlx::postgres::PgRow) -> TimelineEvent {
    TimelineEvent {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        issue_id: row.get("issue_id"),
        pull_request_id: row.get("pull_request_id"),
        actor_user_id: row.get("actor_user_id"),
        event_type: row.get("event_type"),
        metadata: row.get("metadata"),
        created_at: row.get("created_at"),
    }
}

fn reaction_from_row(row: sqlx::postgres::PgRow) -> Result<Reaction, CollaborationError> {
    let content: String = row.get("content");
    Ok(Reaction {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        issue_id: row.get("issue_id"),
        pull_request_id: row.get("pull_request_id"),
        comment_id: row.get("comment_id"),
        user_id: row.get("user_id"),
        content: ReactionContent::try_from(content.as_str())?,
        created_at: row.get("created_at"),
    })
}
