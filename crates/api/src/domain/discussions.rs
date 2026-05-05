use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::{
    notifications::{create_notification, CreateNotification},
    permissions::RepositoryRole,
    repositories::{
        get_repository_by_owner_name, replace_repository_snapshot, repository_permission_for_user,
        CreateCommit, Repository, RepositoryError, RepositorySnapshot, RepositorySnapshotFile,
        RepositoryVisibility,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiscussionStateFilter {
    Open,
    Closed,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDiscussionsQuery<'a> {
    pub q: Option<&'a str>,
    pub label: Option<&'a str>,
    pub state: Option<&'a str>,
    pub answered: Option<&'a str>,
    pub locked: Option<&'a str>,
    pub pinned: Option<&'a str>,
    pub sort: Option<&'a str>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDiscussionsView {
    pub repository: DiscussionRepositorySummary,
    pub viewer: DiscussionViewer,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
    pub filters: DiscussionFilterState,
    pub categories: Vec<DiscussionCategorySummary>,
    pub labels: Vec<DiscussionLabelSummary>,
    pub pinned: Vec<PinnedDiscussionCard>,
    pub helpful_contributors: Vec<HelpfulContributorSummary>,
    pub community_links: Vec<CommunityLinkSummary>,
    pub items: Vec<DiscussionRow>,
    pub open_count: i64,
    pub closed_count: i64,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub has_next_page: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionRepositorySummary {
    pub id: Uuid,
    pub owner: String,
    pub name: String,
    pub visibility: String,
    pub is_archived: bool,
    pub href: String,
    pub discussions_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionViewer {
    pub authenticated: bool,
    pub permission: Option<String>,
    pub can_read: bool,
    pub can_vote: bool,
    pub can_create: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionFilterState {
    pub query: String,
    pub label: Option<String>,
    pub state: String,
    pub answered: Option<bool>,
    pub locked: Option<bool>,
    pub pinned: Option<bool>,
    pub sort: String,
    pub category: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCategorySummary {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub emoji: String,
    pub description: Option<String>,
    pub count: i64,
    pub open_count: i64,
    pub href: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionLabelSummary {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionAuthorSummary {
    pub id: Option<Uuid>,
    pub login: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionRow {
    pub id: Uuid,
    pub number: i64,
    pub title: String,
    pub state: String,
    pub answered: bool,
    pub locked: bool,
    pub pinned: bool,
    pub category: DiscussionCategorySummary,
    pub labels: Vec<DiscussionLabelSummary>,
    pub author: DiscussionAuthorSummary,
    pub comments_count: i64,
    pub votes_count: i64,
    pub viewer_voted: bool,
    pub href: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PinnedDiscussionCard {
    pub discussion: DiscussionRow,
    pub position: i32,
    pub pinned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HelpfulContributorSummary {
    pub user: DiscussionAuthorSummary,
    pub comments_count: i64,
    pub helpful_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityLinkSummary {
    pub id: Uuid,
    pub label: String,
    pub href: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionVoteResponse {
    pub discussion_id: Uuid,
    pub discussion_number: i64,
    pub viewer_voted: bool,
    pub votes_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDiscussionDetailQuery<'a> {
    pub sort: Option<&'a str>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDiscussionDetailView {
    pub repository: DiscussionRepositorySummary,
    pub viewer: DiscussionDetailViewer,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
    pub discussion: DiscussionDetailSummary,
    pub author: DiscussionAuthorSummary,
    pub category: DiscussionCategorySummary,
    pub labels: Vec<DiscussionLabelSummary>,
    pub body: DiscussionBodyView,
    pub form_answers: Vec<DiscussionFormAnswerView>,
    pub poll: Option<DiscussionPollView>,
    pub answer: Option<DiscussionAnswerSummary>,
    pub reactions: Vec<DiscussionReactionSummary>,
    pub subscription: DiscussionSubscriptionState,
    pub sidebar: DiscussionSidebarView,
    pub timeline: Vec<DiscussionTimelineItem>,
    pub sort: String,
    pub page: i64,
    pub page_size: i64,
    pub total_comments: i64,
    pub has_next_page: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionDetailViewer {
    pub authenticated: bool,
    pub permission: Option<String>,
    pub can_read: bool,
    pub can_comment: bool,
    pub can_react: bool,
    pub can_subscribe: bool,
    pub can_mark_answer: bool,
    pub can_moderate: bool,
    pub viewer_voted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionDetailSummary {
    pub id: Uuid,
    pub number: i64,
    pub title: String,
    pub state: String,
    pub answered: bool,
    pub locked: bool,
    pub comments_count: i64,
    pub votes_count: i64,
    pub href: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionBodyView {
    pub markdown: String,
    pub html: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionFormAnswerView {
    pub field_id: String,
    pub field_label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionPollView {
    pub id: Uuid,
    pub question: String,
    pub allows_multiple: bool,
    pub options: Vec<DiscussionPollOptionView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionPollOptionView {
    pub id: Uuid,
    pub position: i32,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionAnswerSummary {
    pub comment_id: Uuid,
    pub marked_by: Option<DiscussionAuthorSummary>,
    pub marked_at: DateTime<Utc>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionReactionSummary {
    pub content: String,
    pub count: i64,
    pub viewer_reacted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionSubscriptionState {
    pub state: String,
    pub reason: Option<String>,
    pub subscribed: bool,
    pub can_change: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionSidebarView {
    pub category: DiscussionCategorySummary,
    pub labels: Vec<DiscussionLabelSummary>,
    pub category_options: Vec<DiscussionCategoryChoice>,
    pub label_options: Vec<DiscussionLabelSummary>,
    pub participants: Vec<DiscussionAuthorSummary>,
    pub events: Vec<DiscussionEventView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionEventView {
    pub id: Uuid,
    pub event_type: String,
    pub actor: Option<DiscussionAuthorSummary>,
    pub payload: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum DiscussionTimelineItem {
    Comment(DiscussionCommentView),
    Event(DiscussionEventView),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCommentView {
    pub id: Uuid,
    pub author: DiscussionAuthorSummary,
    pub body: DiscussionBodyView,
    pub reactions: Vec<DiscussionReactionSummary>,
    pub replies: Vec<DiscussionReplyView>,
    pub answer: bool,
    pub href: String,
    pub edited: bool,
    pub deleted: bool,
    pub deleted_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionReplyView {
    pub id: Uuid,
    pub author: DiscussionAuthorSummary,
    pub body: DiscussionBodyView,
    pub reactions: Vec<DiscussionReactionSummary>,
    pub href: String,
    pub edited: bool,
    pub deleted: bool,
    pub deleted_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCreationView {
    pub repository: DiscussionRepositorySummary,
    pub viewer: DiscussionViewer,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
    pub categories: Vec<DiscussionCategoryChoice>,
    pub selected_category: Option<DiscussionCategoryChoice>,
    pub form: DiscussionFormDefinition,
    pub similar_search: DiscussionSimilarSearch,
    pub community_links: Vec<CommunityLinkSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCategoryChoice {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub emoji: String,
    pub description: Option<String>,
    pub accepts_answers: bool,
    pub is_poll: bool,
    pub count: i64,
    pub open_count: i64,
    pub href: String,
    pub form_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionFormDefinition {
    pub category_slug: Option<String>,
    pub template_path: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub body: String,
    pub fields: Vec<DiscussionFormField>,
    pub valid: bool,
    pub fallback: bool,
    pub parse_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionFormField {
    pub id: String,
    pub field_type: String,
    pub label: String,
    pub description: Option<String>,
    pub placeholder: Option<String>,
    pub required: bool,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionSimilarSearch {
    pub required: bool,
    pub query: String,
    pub href: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDiscussionRequest {
    pub category_slug: String,
    pub title: String,
    pub body: Option<String>,
    pub similar_search_acknowledged: bool,
    #[serde(default)]
    pub form_answers: Vec<DiscussionFormAnswerInput>,
    pub poll: Option<DiscussionPollInput>,
    #[serde(default)]
    pub attachment_drafts: Vec<DiscussionAttachmentDraft>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionFormAnswerInput {
    pub field_id: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionPollInput {
    pub question: String,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub allows_multiple: bool,
}

#[derive(Debug, Clone)]
struct NormalizedDiscussionPoll {
    question: String,
    options: Vec<String>,
    allows_multiple: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionAttachmentDraft {
    pub id: Option<Uuid>,
    pub file_name: String,
    pub content_type: String,
    pub byte_size: i64,
    pub storage_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDiscussionResponse {
    pub discussion_id: Uuid,
    pub discussion_number: i64,
    pub href: String,
    pub title: String,
    pub category: DiscussionCategoryChoice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDiscussionCommentRequest {
    pub body: String,
    #[serde(default)]
    pub attachment_drafts: Vec<DiscussionAttachmentDraft>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionReactionRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionSubscriptionRequest {
    pub subscribed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionAnswerRequest {
    pub comment_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionStateRequest {
    pub state: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionMetadataRequest {
    pub category_slug: Option<String>,
    pub label_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscussionCategoryFormat {
    Announcement,
    OpenEnded,
    Poll,
    QuestionAndAnswer,
}

impl DiscussionCategoryFormat {
    fn as_str(self) -> &'static str {
        match self {
            Self::Announcement => "announcement",
            Self::OpenEnded => "open_ended",
            Self::Poll => "poll",
            Self::QuestionAndAnswer => "question_and_answer",
        }
    }

    fn accepts_answers(self) -> bool {
        matches!(self, Self::QuestionAndAnswer)
    }
}

impl TryFrom<&str> for DiscussionCategoryFormat {
    type Error = RepositoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "announcement" => Ok(Self::Announcement),
            "open_ended" => Ok(Self::OpenEnded),
            "poll" => Ok(Self::Poll),
            "question_and_answer" | "q_and_a" | "q-a" => Ok(Self::QuestionAndAnswer),
            other => Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported discussion category format `{other}`"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCategoryAdminViewer {
    pub authenticated: bool,
    pub permission: Option<String>,
    pub can_read: bool,
    pub can_manage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCategorySectionItem {
    pub id: Uuid,
    pub name: String,
    pub position: i32,
    pub category_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCategoryAdminItem {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub emoji: String,
    pub description: Option<String>,
    pub format: DiscussionCategoryFormat,
    pub accepts_answers: bool,
    pub is_poll: bool,
    pub is_default: bool,
    pub section_id: Option<Uuid>,
    pub section_name: Option<String>,
    pub template_path: Option<String>,
    pub count: i64,
    pub open_count: i64,
    pub position: i32,
    pub href: String,
    pub edit_href: String,
    pub template_href: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCategorySettingsView {
    pub repository: DiscussionRepositorySummary,
    pub viewer: DiscussionCategoryAdminViewer,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
    pub category_limit: i64,
    pub remaining_categories: i64,
    pub sections: Vec<DiscussionCategorySectionItem>,
    pub categories: Vec<DiscussionCategoryAdminItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDiscussionCategoryRequest {
    pub name: String,
    pub emoji: Option<String>,
    pub description: Option<String>,
    pub format: Option<DiscussionCategoryFormat>,
    pub section_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDiscussionCategoryRequest {
    pub name: Option<String>,
    pub emoji: Option<String>,
    pub description: Option<String>,
    pub format: Option<DiscussionCategoryFormat>,
    pub section_id: Option<Option<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDiscussionCategorySectionRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDiscussionCategorySectionRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCategoryOrderItem {
    pub id: Uuid,
    pub section_id: Option<Uuid>,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionSectionOrderItem {
    pub id: Uuid,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCategoryOrderRequest {
    pub items: Vec<DiscussionCategoryOrderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionSectionOrderRequest {
    pub items: Vec<DiscussionSectionOrderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDiscussionCategoryRequest {
    pub move_to_category_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCategoryTemplateView {
    pub repository: DiscussionRepositorySummary,
    pub viewer: DiscussionCategoryAdminViewer,
    pub category: DiscussionCategoryAdminItem,
    pub path: String,
    pub content: String,
    pub content_sha: String,
    pub branch: String,
    pub form: DiscussionFormDefinition,
    pub commit_href: Option<String>,
    pub blob_href: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCategoryTemplatePreviewRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCategoryTemplateCommitRequest {
    pub content: String,
    pub commit_message: String,
    pub branch: Option<String>,
    pub propose_change: Option<bool>,
    pub expected_content_sha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscussionCategoryTemplateCommitResponse {
    pub template: DiscussionCategoryTemplateView,
    pub proposed: bool,
    pub commit_oid: String,
    pub commit_href: String,
}

pub struct DiscussionReactionMutation<'a> {
    pub content: &'a str,
    pub reacted: bool,
}

#[derive(Debug)]
struct NormalizedDiscussionFilters {
    query: String,
    label: Option<String>,
    state: DiscussionStateFilter,
    answered: Option<bool>,
    locked: Option<bool>,
    pinned: Option<bool>,
    sort: String,
    page: i64,
    page_size: i64,
}

#[derive(Debug, Clone)]
struct NormalizedDiscussionDetailQuery {
    sort: String,
    page: i64,
    page_size: i64,
}

pub async fn repository_discussions_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    category_slug: Option<&str>,
    query: RepositoryDiscussionsQuery<'_>,
) -> Result<Option<RepositoryDiscussionsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };

    let permission = repository_permission_for_user(pool, repository.id, actor_user_id).await?;
    let can_read = repository.visibility == RepositoryVisibility::Public
        || repository.owner_user_id == Some(actor_user_id)
        || permission.as_ref().is_some_and(|p| p.role.can_read());
    if !can_read {
        return Err(RepositoryError::PermissionDenied);
    }

    let filters = normalize_discussion_filters(query)?;
    let selected_category = match category_slug {
        Some(slug) => Some(normalize_slug(slug)?),
        None => None,
    };

    let policy_enabled = repository_discussions_policy_enabled(pool, repository.id).await?;
    let category_exists = if let Some(slug) = selected_category.as_deref() {
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM discussion_categories WHERE repository_id = $1 AND slug = $2)",
        )
        .bind(repository.id)
        .bind(slug)
        .fetch_one(pool)
        .await?
    } else {
        true
    };
    if !category_exists {
        return Ok(None);
    }

    let categories =
        load_discussion_categories(pool, &repository, selected_category.as_deref()).await?;
    let labels = load_discussion_labels(pool, repository.id).await?;
    let items = if policy_enabled {
        load_discussion_rows(
            pool,
            &repository,
            actor_user_id,
            selected_category.as_deref(),
            &filters,
        )
        .await?
    } else {
        Vec::new()
    };
    let total =
        count_discussions(pool, repository.id, selected_category.as_deref(), &filters).await?;
    let (open_count, closed_count) =
        discussion_state_counts(pool, repository.id, selected_category.as_deref()).await?;
    let pinned = if policy_enabled {
        load_pinned_discussions(
            pool,
            &repository,
            actor_user_id,
            selected_category.as_deref(),
        )
        .await?
    } else {
        Vec::new()
    };
    let helpful_contributors = load_helpful_contributors(pool, repository.id).await?;
    let community_links = load_community_links(pool, repository.id).await?;
    let viewer_permission = permission.map(|p| p.role.as_str().to_owned()).or_else(|| {
        (repository.owner_user_id == Some(actor_user_id))
            .then(|| RepositoryRole::Admin.as_str().to_owned())
    });
    let can_write = matches!(
        viewer_permission.as_deref(),
        Some("write" | "maintain" | "admin" | "owner")
    );

    Ok(Some(RepositoryDiscussionsView {
        repository: DiscussionRepositorySummary {
            id: repository.id,
            owner: repository.owner_login.clone(),
            name: repository.name.clone(),
            visibility: repository.visibility.as_str().to_owned(),
            is_archived: repository.is_archived,
            href: format!("/{}/{}", repository.owner_login, repository.name),
            discussions_href: format!(
                "/{}/{}/discussions",
                repository.owner_login, repository.name
            ),
        },
        viewer: DiscussionViewer {
            authenticated: true,
            permission: viewer_permission,
            can_read,
            can_vote: policy_enabled && !repository.is_archived,
            can_create: policy_enabled && !repository.is_archived && can_write,
        },
        enabled: policy_enabled,
        disabled_reason: (!policy_enabled)
            .then(|| "Repository discussions are disabled by organization policy.".to_owned()),
        filters: DiscussionFilterState {
            query: filters.query.clone(),
            label: filters.label.clone(),
            state: match filters.state {
                DiscussionStateFilter::Open => "open",
                DiscussionStateFilter::Closed => "closed",
                DiscussionStateFilter::All => "all",
            }
            .to_owned(),
            answered: filters.answered,
            locked: filters.locked,
            pinned: filters.pinned,
            sort: filters.sort.clone(),
            category: selected_category,
            page: filters.page,
            page_size: filters.page_size,
        },
        categories,
        labels,
        pinned,
        helpful_contributors,
        community_links,
        items,
        open_count,
        closed_count,
        total,
        page: filters.page,
        page_size: filters.page_size,
        has_next_page: filters.page * filters.page_size < total,
    }))
}

pub async fn repository_discussion_category_settings_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
) -> Result<Option<DiscussionCategorySettingsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    let (permission, can_read, can_write) =
        discussion_permissions(pool, &repository, actor_user_id).await?;
    if !can_read {
        return Err(RepositoryError::PermissionDenied);
    }
    let can_manage = can_write || repository.owner_user_id == Some(actor_user_id);
    if !can_manage {
        return Err(RepositoryError::PermissionDenied);
    }
    let enabled = repository_discussions_policy_enabled(pool, repository.id).await?;
    let sections = load_discussion_category_sections(pool, repository.id).await?;
    let categories = load_discussion_category_admin_items(pool, &repository).await?;
    let category_limit = 25;
    let remaining_categories = (category_limit - categories.len() as i64).max(0);

    Ok(Some(DiscussionCategorySettingsView {
        repository: discussion_repository_summary(&repository),
        viewer: DiscussionCategoryAdminViewer {
            authenticated: true,
            permission,
            can_read,
            can_manage,
        },
        enabled,
        disabled_reason: (!enabled)
            .then(|| "Repository discussions are disabled by organization policy.".to_owned()),
        category_limit,
        remaining_categories,
        sections,
        categories,
    }))
}

pub async fn repository_discussion_category_template_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    category_id: Uuid,
) -> Result<Option<DiscussionCategoryTemplateView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    ensure_category_admin_allowed(pool, &repository, actor_user_id).await?;
    let Some(category) =
        load_discussion_category_admin_item(pool, &repository, category_id).await?
    else {
        return Ok(None);
    };
    Ok(Some(
        discussion_category_template_view(pool, &repository, actor_user_id, category, None).await?,
    ))
}

pub async fn preview_repository_discussion_category_template_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    category_id: Uuid,
    request: DiscussionCategoryTemplatePreviewRequest,
) -> Result<Option<DiscussionFormDefinition>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    ensure_category_admin_allowed(pool, &repository, actor_user_id).await?;
    let Some(category) =
        load_discussion_category_admin_item(pool, &repository, category_id).await?
    else {
        return Ok(None);
    };
    if category.is_poll {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "poll discussion categories cannot use YAML templates".to_owned(),
        ));
    }
    let content = normalize_discussion_template_content(&request.content)?;
    let path = discussion_template_path(&category.slug);
    Ok(Some(parse_discussion_template(
        &content,
        &category.slug,
        &path,
    )))
}

pub async fn commit_repository_discussion_category_template_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    category_id: Uuid,
    request: DiscussionCategoryTemplateCommitRequest,
) -> Result<Option<DiscussionCategoryTemplateCommitResponse>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    ensure_category_admin_allowed(pool, &repository, actor_user_id).await?;
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    let Some(category) =
        load_discussion_category_admin_item(pool, &repository, category_id).await?
    else {
        return Ok(None);
    };
    if category.is_poll {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "poll discussion categories cannot use YAML templates".to_owned(),
        ));
    }

    let content = normalize_discussion_template_content(&request.content)?;
    let path = discussion_template_path(&category.slug);
    let parsed = parse_discussion_template(&content, &category.slug, &path);
    if !parsed.valid {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            parsed
                .parse_error
                .clone()
                .unwrap_or_else(|| "discussion template YAML is invalid".to_owned()),
        ));
    }

    let existing = current_discussion_template_file(pool, repository.id, &path).await?;
    if let Some(expected) = request
        .expected_content_sha
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let current_sha = existing
            .as_ref()
            .map(|file| content_sha(&file.content))
            .unwrap_or_default();
        if expected != current_sha {
            return Err(RepositoryError::SecurityPolicyConflict);
        }
    }

    let target_branch = normalize_template_branch(
        &repository,
        request.branch.as_deref(),
        request.propose_change,
    )?;
    let commit_message = normalize_template_commit_message(&request.commit_message)?;
    let commit = write_discussion_template_snapshot(
        pool,
        &repository,
        actor_user_id,
        &target_branch,
        &path,
        &content,
        &commit_message,
    )
    .await?;

    cache_discussion_template_form(pool, repository.id, category.id, &path, &content, &parsed)
        .await?;
    sqlx::query(
        r#"
        UPDATE discussion_categories
        SET template_path = $3, updated_at = now()
        WHERE repository_id = $1 AND id = $2
        "#,
    )
    .bind(repository.id)
    .bind(category.id)
    .bind(&path)
    .execute(pool)
    .await?;
    record_discussion_settings_audit(
        pool,
        actor_user_id,
        repository.id,
        "repository.discussion_category.template_update",
        "discussion_category",
        category.id,
        json!({
            "path": path,
            "branch": target_branch,
            "commitOid": commit.oid,
            "proposed": target_branch != repository.default_branch,
        }),
    )
    .await?;

    let category = load_discussion_category_admin_item(pool, &repository, category_id)
        .await?
        .ok_or(RepositoryError::NotFound)?;
    let view = discussion_category_template_view(
        pool,
        &repository,
        actor_user_id,
        category,
        Some(content),
    )
    .await?;
    let commit_href = format!(
        "/{}/{}/commits/{}",
        repository.owner_login, repository.name, commit.oid
    );
    Ok(Some(DiscussionCategoryTemplateCommitResponse {
        template: view,
        proposed: target_branch != repository.default_branch,
        commit_oid: commit.oid,
        commit_href,
    }))
}

pub async fn create_repository_discussion_category_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    request: CreateDiscussionCategoryRequest,
) -> Result<Option<DiscussionCategorySettingsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    ensure_category_admin_allowed(pool, &repository, actor_user_id).await?;
    if !repository_discussions_policy_enabled(pool, repository.id).await? {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "repository discussions are disabled by organization policy".to_owned(),
        ));
    }
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussion_categories WHERE repository_id = $1",
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    if count >= 25 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "repositories can have at most 25 discussion categories".to_owned(),
        ));
    }

    let name = normalize_category_name(&request.name)?;
    let emoji = normalize_category_emoji(request.emoji.as_deref())?;
    let description = normalize_category_description(request.description.as_deref())?;
    let format = request
        .format
        .unwrap_or(DiscussionCategoryFormat::QuestionAndAnswer);
    ensure_category_section_exists(pool, repository.id, request.section_id).await?;
    ensure_category_uniqueness(pool, repository.id, None, &name, &emoji).await?;
    let slug = unique_category_slug(pool, repository.id, &name).await?;
    let position: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM discussion_categories WHERE repository_id = $1",
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;

    let category_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_categories (
            repository_id, section_id, slug, name, emoji, description, position,
            format, accepts_answers
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(request.section_id)
    .bind(&slug)
    .bind(&name)
    .bind(&emoji)
    .bind(&description)
    .bind(position)
    .bind(format.as_str())
    .bind(format.accepts_answers())
    .fetch_one(pool)
    .await?;
    record_category_admin_audit(
        pool,
        actor_user_id,
        repository.id,
        "repository.discussion_category.create",
        category_id,
        json!({
            "slug": slug,
            "name": name,
            "format": format.as_str(),
            "sectionId": request.section_id,
        }),
    )
    .await?;
    repository_discussion_category_settings_for_actor_by_owner_name(
        pool,
        actor_user_id,
        owner,
        repo,
    )
    .await
}

pub async fn update_repository_discussion_category_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    category_id: Uuid,
    request: UpdateDiscussionCategoryRequest,
) -> Result<Option<DiscussionCategorySettingsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    ensure_category_admin_allowed(pool, &repository, actor_user_id).await?;
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    let Some(current) = load_category_admin_row(pool, repository.id, category_id).await? else {
        return Err(RepositoryError::NotFound);
    };
    let current_name: String = current.try_get("name")?;
    let current_emoji: String = current.try_get("emoji")?;
    let current_description: Option<String> = current.try_get("description")?;
    let current_format =
        DiscussionCategoryFormat::try_from(current.try_get::<String, _>("format")?.as_str())?;
    let current_section_id: Option<Uuid> = current.try_get("section_id")?;
    let template_path: Option<String> = current.try_get("template_path")?;

    let next_name = match request.name.as_deref() {
        Some(value) => normalize_category_name(value)?,
        None => current_name,
    };
    let next_emoji = match request.emoji.as_deref() {
        Some(value) => normalize_category_emoji(Some(value))?,
        None => current_emoji,
    };
    let next_description = match request.description.as_deref() {
        Some(value) => normalize_category_description(Some(value))?,
        None => current_description,
    };
    let next_format = request.format.unwrap_or(current_format);
    let next_section_id = request.section_id.unwrap_or(current_section_id);
    ensure_category_section_exists(pool, repository.id, next_section_id).await?;
    ensure_category_uniqueness(
        pool,
        repository.id,
        Some(category_id),
        &next_name,
        &next_emoji,
    )
    .await?;
    if next_format == DiscussionCategoryFormat::Poll && template_path.is_some() {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "poll categories cannot use discussion category forms".to_owned(),
        ));
    }

    sqlx::query(
        r#"
        UPDATE discussion_categories
        SET name = $1,
            emoji = $2,
            description = $3,
            format = $4,
            accepts_answers = $5,
            section_id = $6,
            updated_at = now()
        WHERE id = $7 AND repository_id = $8
        "#,
    )
    .bind(&next_name)
    .bind(&next_emoji)
    .bind(&next_description)
    .bind(next_format.as_str())
    .bind(next_format.accepts_answers())
    .bind(next_section_id)
    .bind(category_id)
    .bind(repository.id)
    .execute(pool)
    .await?;
    record_category_admin_audit(
        pool,
        actor_user_id,
        repository.id,
        "repository.discussion_category.update",
        category_id,
        json!({
            "name": next_name,
            "emoji": next_emoji,
            "format": next_format.as_str(),
            "sectionId": next_section_id,
        }),
    )
    .await?;
    repository_discussion_category_settings_for_actor_by_owner_name(
        pool,
        actor_user_id,
        owner,
        repo,
    )
    .await
}

pub async fn create_repository_discussion_category_section_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    request: CreateDiscussionCategorySectionRequest,
) -> Result<Option<DiscussionCategorySettingsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    ensure_category_admin_allowed(pool, &repository, actor_user_id).await?;
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    let name = normalize_category_section_name(&request.name)?;
    ensure_section_name_unique(pool, repository.id, None, &name).await?;
    let position: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM discussion_category_sections WHERE repository_id = $1",
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let section_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO discussion_category_sections (repository_id, name, position)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(&name)
    .bind(position)
    .fetch_one(pool)
    .await?;
    record_discussion_settings_audit(
        pool,
        actor_user_id,
        repository.id,
        "repository.discussion_category_section.create",
        "repository_discussion_category_section",
        section_id,
        json!({ "name": name, "position": position }),
    )
    .await?;
    repository_discussion_category_settings_for_actor_by_owner_name(
        pool,
        actor_user_id,
        owner,
        repo,
    )
    .await
}

pub async fn update_repository_discussion_category_section_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    section_id: Uuid,
    request: UpdateDiscussionCategorySectionRequest,
) -> Result<Option<DiscussionCategorySettingsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    ensure_category_admin_allowed(pool, &repository, actor_user_id).await?;
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    ensure_category_section_exists(pool, repository.id, Some(section_id)).await?;
    let name = normalize_category_section_name(&request.name)?;
    ensure_section_name_unique(pool, repository.id, Some(section_id), &name).await?;
    sqlx::query(
        "UPDATE discussion_category_sections SET name = $1, updated_at = now() WHERE repository_id = $2 AND id = $3",
    )
    .bind(&name)
    .bind(repository.id)
    .bind(section_id)
    .execute(pool)
    .await?;
    record_discussion_settings_audit(
        pool,
        actor_user_id,
        repository.id,
        "repository.discussion_category_section.update",
        "repository_discussion_category_section",
        section_id,
        json!({ "name": name }),
    )
    .await?;
    repository_discussion_category_settings_for_actor_by_owner_name(
        pool,
        actor_user_id,
        owner,
        repo,
    )
    .await
}

pub async fn delete_repository_discussion_category_section_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    section_id: Uuid,
) -> Result<Option<DiscussionCategorySettingsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    ensure_category_admin_allowed(pool, &repository, actor_user_id).await?;
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    ensure_category_section_exists(pool, repository.id, Some(section_id)).await?;
    sqlx::query(
        "UPDATE discussion_categories SET section_id = NULL, updated_at = now() WHERE repository_id = $1 AND section_id = $2",
    )
    .bind(repository.id)
    .bind(section_id)
    .execute(pool)
    .await?;
    sqlx::query("DELETE FROM discussion_category_sections WHERE repository_id = $1 AND id = $2")
        .bind(repository.id)
        .bind(section_id)
        .execute(pool)
        .await?;
    record_discussion_settings_audit(
        pool,
        actor_user_id,
        repository.id,
        "repository.discussion_category_section.delete",
        "repository_discussion_category_section",
        section_id,
        json!({ "movedCategoriesTo": null }),
    )
    .await?;
    repository_discussion_category_settings_for_actor_by_owner_name(
        pool,
        actor_user_id,
        owner,
        repo,
    )
    .await
}

pub async fn reorder_repository_discussion_categories_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    request: DiscussionCategoryOrderRequest,
) -> Result<Option<DiscussionCategorySettingsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    ensure_category_admin_allowed(pool, &repository, actor_user_id).await?;
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    if request.items.is_empty() {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "category order must include at least one category".to_owned(),
        ));
    }
    for item in &request.items {
        ensure_category_section_exists(pool, repository.id, item.section_id).await?;
        let updated = sqlx::query(
            "UPDATE discussion_categories SET section_id = $1, position = $2, updated_at = now() WHERE repository_id = $3 AND id = $4",
        )
        .bind(item.section_id)
        .bind(item.position.max(1))
        .bind(repository.id)
        .bind(item.id)
        .execute(pool)
        .await?;
        if updated.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
    }
    record_discussion_settings_audit(
        pool,
        actor_user_id,
        repository.id,
        "repository.discussion_category.reorder",
        "repository",
        repository.id,
        json!({ "count": request.items.len() }),
    )
    .await?;
    repository_discussion_category_settings_for_actor_by_owner_name(
        pool,
        actor_user_id,
        owner,
        repo,
    )
    .await
}

pub async fn reorder_repository_discussion_category_sections_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    request: DiscussionSectionOrderRequest,
) -> Result<Option<DiscussionCategorySettingsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    ensure_category_admin_allowed(pool, &repository, actor_user_id).await?;
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    if request.items.is_empty() {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "section order must include at least one section".to_owned(),
        ));
    }
    for item in &request.items {
        let updated = sqlx::query(
            "UPDATE discussion_category_sections SET position = $1, updated_at = now() WHERE repository_id = $2 AND id = $3",
        )
        .bind(item.position.max(1))
        .bind(repository.id)
        .bind(item.id)
        .execute(pool)
        .await?;
        if updated.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
    }
    record_discussion_settings_audit(
        pool,
        actor_user_id,
        repository.id,
        "repository.discussion_category_section.reorder",
        "repository",
        repository.id,
        json!({ "count": request.items.len() }),
    )
    .await?;
    repository_discussion_category_settings_for_actor_by_owner_name(
        pool,
        actor_user_id,
        owner,
        repo,
    )
    .await
}

pub async fn delete_repository_discussion_category_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    category_id: Uuid,
    request: DeleteDiscussionCategoryRequest,
) -> Result<Option<DiscussionCategorySettingsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    ensure_category_admin_allowed(pool, &repository, actor_user_id).await?;
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    let category_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussion_categories WHERE repository_id = $1",
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    if category_count <= 1 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "the last discussion category cannot be deleted".to_owned(),
        ));
    }
    let discussion_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussions WHERE repository_id = $1 AND category_id = $2",
    )
    .bind(repository.id)
    .bind(category_id)
    .fetch_one(pool)
    .await?;
    let move_to_category_id = request.move_to_category_id;
    if discussion_count > 0 {
        let Some(destination_id) = move_to_category_id else {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "choose a destination category before deleting a category with discussions"
                    .to_owned(),
            ));
        };
        if destination_id == category_id {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "destination category must be different from the deleted category".to_owned(),
            ));
        }
        load_category_admin_row(pool, repository.id, destination_id)
            .await?
            .ok_or(RepositoryError::NotFound)?;
        sqlx::query(
            "UPDATE discussions SET category_id = $1, updated_at = now() WHERE repository_id = $2 AND category_id = $3",
        )
        .bind(destination_id)
        .bind(repository.id)
        .bind(category_id)
        .execute(pool)
        .await?;
    }
    let deleted =
        sqlx::query("DELETE FROM discussion_categories WHERE repository_id = $1 AND id = $2")
            .bind(repository.id)
            .bind(category_id)
            .execute(pool)
            .await?;
    if deleted.rows_affected() == 0 {
        return Err(RepositoryError::NotFound);
    }
    record_discussion_settings_audit(
        pool,
        actor_user_id,
        repository.id,
        "repository.discussion_category.delete",
        "repository_discussion_category",
        category_id,
        json!({
            "movedDiscussions": discussion_count,
            "moveToCategoryId": move_to_category_id,
        }),
    )
    .await?;
    repository_discussion_category_settings_for_actor_by_owner_name(
        pool,
        actor_user_id,
        owner,
        repo,
    )
    .await
}

pub async fn set_repository_discussion_vote_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    discussion_number: i64,
    voted: bool,
) -> Result<Option<DiscussionVoteResponse>, RepositoryError> {
    if discussion_number < 1 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "discussion number must be positive".to_owned(),
        ));
    }
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    let permission = repository_permission_for_user(pool, repository.id, actor_user_id).await?;
    let can_read = repository.visibility == RepositoryVisibility::Public
        || repository.owner_user_id == Some(actor_user_id)
        || permission.as_ref().is_some_and(|p| p.role.can_read());
    if !can_read {
        return Err(RepositoryError::PermissionDenied);
    }
    if repository.is_archived {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "archived repositories do not accept discussion votes".to_owned(),
        ));
    }
    if !repository_discussions_policy_enabled(pool, repository.id).await? {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "repository discussions are disabled by organization policy".to_owned(),
        ));
    }

    let Some(row) = sqlx::query(
        r#"
        SELECT id, number, title, author_user_id
        FROM discussions
        WHERE repository_id = $1 AND number = $2
        "#,
    )
    .bind(repository.id)
    .bind(discussion_number)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };
    let discussion_id: Uuid = row.try_get("id")?;
    let title: String = row.try_get("title")?;
    let author_user_id: Option<Uuid> = row.try_get("author_user_id")?;

    let changed = if voted {
        sqlx::query(
            "INSERT INTO discussion_votes (discussion_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(discussion_id)
        .bind(actor_user_id)
        .execute(pool)
        .await?
        .rows_affected()
            > 0
    } else {
        sqlx::query("DELETE FROM discussion_votes WHERE discussion_id = $1 AND user_id = $2")
            .bind(discussion_id)
            .bind(actor_user_id)
            .execute(pool)
            .await?
            .rows_affected()
            > 0
    };

    let votes_count: i64 = sqlx::query_scalar(
        r#"
        UPDATE discussions
        SET votes_count = (
                SELECT COUNT(*)::bigint
                FROM discussion_votes
                WHERE discussion_votes.discussion_id = discussions.id
            ),
            last_activity_at = CASE WHEN $3 THEN now() ELSE last_activity_at END,
            updated_at = CASE WHEN $3 THEN now() ELSE updated_at END
        WHERE id = $1 AND number = $2
        RETURNING votes_count
        "#,
    )
    .bind(discussion_id)
    .bind(discussion_number)
    .bind(changed)
    .fetch_one(pool)
    .await?;

    if changed {
        let event_type = if voted { "voted" } else { "unvoted" };
        sqlx::query(
            r#"
            INSERT INTO discussion_activity_events (discussion_id, actor_user_id, event_type, payload)
            VALUES ($1, $2, $3, jsonb_build_object('votesCount', $4))
            "#,
        )
        .bind(discussion_id)
        .bind(actor_user_id)
        .bind(event_type)
        .bind(votes_count)
        .execute(pool)
        .await?;

        if voted {
            if let Some(author_user_id) = author_user_id.filter(|id| *id != actor_user_id) {
                create_notification(
                    pool,
                    CreateNotification {
                        user_id: author_user_id,
                        repository_id: Some(repository.id),
                        subject_type: "discussion".to_owned(),
                        subject_id: Some(discussion_id),
                        title: format!(
                            "Discussion #{} received an upvote: {}",
                            discussion_number, title
                        ),
                        reason: "discussion_vote".to_owned(),
                    },
                )
                .await
                .map_err(|error| match error {
                    super::notifications::NotificationError::Sqlx(error) => {
                        RepositoryError::Sqlx(error)
                    }
                    super::notifications::NotificationError::NotFound
                    | super::notifications::NotificationError::Validation(_) => {
                        RepositoryError::NotFound
                    }
                })?;
            }
        }
    }

    Ok(Some(DiscussionVoteResponse {
        discussion_id,
        discussion_number,
        viewer_voted: voted,
        votes_count,
    }))
}

pub async fn repository_discussion_detail_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    discussion_number: i64,
    query: RepositoryDiscussionDetailQuery<'_>,
) -> Result<Option<RepositoryDiscussionDetailView>, RepositoryError> {
    if discussion_number < 1 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "discussion number must be positive".to_owned(),
        ));
    }
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    let (viewer_permission, can_read, _) =
        discussion_permissions(pool, &repository, actor_user_id).await?;
    if !can_read {
        return Err(RepositoryError::PermissionDenied);
    }

    let normalized = normalize_discussion_detail_query(query)?;
    let policy_enabled = repository_discussions_policy_enabled(pool, repository.id).await?;
    let Some(row) = sqlx::query(
        r#"
        SELECT discussions.id, discussions.number, discussions.title, discussions.body,
               discussions.state, discussions.answered, discussions.locked,
               discussions.comments_count, discussions.votes_count, discussions.created_at,
               discussions.updated_at, discussions.last_activity_at, discussions.answer_comment_id,
               discussion_categories.id AS category_id, discussion_categories.slug AS category_slug,
               discussion_categories.name AS category_name, discussion_categories.emoji AS category_emoji,
               discussion_categories.description AS category_description,
               discussion_categories.accepts_answers AS category_accepts_answers,
               author.id AS author_id,
               COALESCE(NULLIF(author.username, ''), author.email, 'ghost') AS author_login,
               author.display_name AS author_display_name,
               author.avatar_url AS author_avatar_url,
               EXISTS (
                 SELECT 1 FROM discussion_votes
                 WHERE discussion_votes.discussion_id = discussions.id
                   AND discussion_votes.user_id = $3
               ) AS viewer_voted
        FROM discussions
        JOIN discussion_categories ON discussion_categories.id = discussions.category_id
        LEFT JOIN users author ON author.id = discussions.author_user_id
        WHERE discussions.repository_id = $1 AND discussions.number = $2
        "#,
    )
    .bind(repository.id)
    .bind(discussion_number)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let discussion_id: Uuid = row.try_get("id")?;
    let answer_comment_id: Option<Uuid> = row.try_get("answer_comment_id")?;
    let category = DiscussionCategorySummary {
        id: row.try_get("category_id")?,
        slug: row.try_get("category_slug")?,
        name: row.try_get("category_name")?,
        emoji: row.try_get("category_emoji")?,
        description: row.try_get("category_description")?,
        count: 0,
        open_count: 0,
        href: format!(
            "/{}/{}/discussions/categories/{}",
            repository.owner_login,
            repository.name,
            row.try_get::<String, _>("category_slug")?
        ),
        active: true,
    };
    let category_accepts_answers: bool = row.try_get("category_accepts_answers")?;
    let labels = load_discussion_labels_for_discussion(pool, discussion_id).await?;
    let comments = load_discussion_detail_comments(
        pool,
        &repository,
        discussion_id,
        answer_comment_id,
        actor_user_id,
        &normalized,
    )
    .await?;
    let events = load_discussion_detail_events(pool, discussion_id).await?;
    let participants = load_discussion_participants(pool, discussion_id).await?;
    let category_options = load_discussion_category_choices(pool, &repository).await?;
    let label_options = load_discussion_labels(pool, repository.id).await?;
    let total_comments = count_top_level_discussion_comments(pool, discussion_id).await?;
    let subscription = load_discussion_subscription(pool, discussion_id, actor_user_id).await?;
    let form_answers = load_discussion_form_answer_views(pool, discussion_id).await?;
    let poll = load_discussion_poll_view(pool, discussion_id).await?;
    let reactions = load_discussion_reactions(pool, discussion_id, None, actor_user_id).await?;
    let answer = load_discussion_answer_summary(
        pool,
        &repository,
        discussion_id,
        answer_comment_id,
        actor_user_id,
    )
    .await?;
    let author = DiscussionAuthorSummary {
        id: row.try_get("author_id")?,
        login: row.try_get("author_login")?,
        display_name: row.try_get("author_display_name")?,
        avatar_url: row.try_get("author_avatar_url")?,
    };
    let can_moderate = matches!(
        viewer_permission.as_deref(),
        Some("triage" | "write" | "maintain" | "admin" | "owner")
    );
    let locked: bool = row.try_get("locked")?;
    let can_comment = policy_enabled && !repository.is_archived && !locked;
    let body_markdown: String = row.try_get("body")?;

    Ok(Some(RepositoryDiscussionDetailView {
        repository: discussion_repository_summary(&repository),
        viewer: DiscussionDetailViewer {
            authenticated: true,
            permission: viewer_permission,
            can_read,
            can_comment,
            can_react: policy_enabled && !repository.is_archived,
            can_subscribe: policy_enabled,
            can_mark_answer: policy_enabled
                && category_accepts_answers
                && !repository.is_archived
                && can_moderate,
            can_moderate,
            viewer_voted: row.try_get("viewer_voted")?,
        },
        enabled: policy_enabled,
        disabled_reason: (!policy_enabled)
            .then(|| "Repository discussions are disabled by organization policy.".to_owned()),
        discussion: DiscussionDetailSummary {
            id: discussion_id,
            number: row.try_get("number")?,
            title: row.try_get("title")?,
            state: row.try_get("state")?,
            answered: row.try_get("answered")?,
            locked,
            comments_count: row.try_get("comments_count")?,
            votes_count: row.try_get("votes_count")?,
            href: format!(
                "/{}/{}/discussions/{}",
                repository.owner_login, repository.name, discussion_number
            ),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            last_activity_at: row.try_get("last_activity_at")?,
        },
        author,
        category: category.clone(),
        labels: labels.clone(),
        body: discussion_body_view(body_markdown),
        form_answers,
        poll,
        answer,
        reactions,
        subscription,
        sidebar: DiscussionSidebarView {
            category,
            labels,
            category_options,
            label_options,
            participants,
            events: events.clone(),
        },
        timeline: merge_discussion_timeline(comments, events),
        sort: normalized.sort,
        page: normalized.page,
        page_size: normalized.page_size,
        total_comments,
        has_next_page: normalized.page * normalized.page_size < total_comments,
    }))
}

pub async fn repository_discussion_creation_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    category_slug: Option<&str>,
    title_query: Option<&str>,
) -> Result<Option<DiscussionCreationView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    let (viewer_permission, can_read, can_write) =
        discussion_permissions(pool, &repository, actor_user_id).await?;
    if !can_read {
        return Err(RepositoryError::PermissionDenied);
    }

    let policy_enabled = repository_discussions_policy_enabled(pool, repository.id).await?;
    let selected_slug = match category_slug {
        Some(slug) => Some(normalize_slug(slug)?),
        None => None,
    };
    let categories = load_discussion_category_choices(pool, &repository).await?;
    let selected_category = match selected_slug.as_deref() {
        Some(slug) => Some(
            categories
                .iter()
                .find(|category| category.slug == slug)
                .cloned()
                .ok_or_else(|| RepositoryError::NotFound)?,
        ),
        None => categories.first().cloned(),
    };
    let form = match selected_category.as_ref() {
        Some(category) => load_discussion_form_definition(pool, repository.id, category).await?,
        None => generic_discussion_form(None),
    };
    let query = normalize_short_text(title_query, "title", 160)?.unwrap_or_default();
    let similar_query = if query.is_empty() {
        "is:open".to_owned()
    } else {
        format!("is:open {query}")
    };

    Ok(Some(DiscussionCreationView {
        repository: discussion_repository_summary(&repository),
        viewer: DiscussionViewer {
            authenticated: true,
            permission: viewer_permission,
            can_read,
            can_vote: policy_enabled && !repository.is_archived,
            can_create: policy_enabled && !repository.is_archived && can_write,
        },
        enabled: policy_enabled,
        disabled_reason: if repository.is_archived {
            Some("Archived repositories do not accept new discussions.".to_owned())
        } else {
            (!policy_enabled)
                .then(|| "Repository discussions are disabled by organization policy.".to_owned())
        },
        categories,
        selected_category,
        form,
        similar_search: DiscussionSimilarSearch {
            required: true,
            href: format!(
                "/{}/{}/discussions?q={}",
                repository.owner_login,
                repository.name,
                url::form_urlencoded::byte_serialize(similar_query.as_bytes()).collect::<String>()
            ),
            query: similar_query,
        },
        community_links: load_community_links(pool, repository.id).await?,
    }))
}

pub async fn create_repository_discussion_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    request: CreateDiscussionRequest,
) -> Result<Option<CreateDiscussionResponse>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    let (_, can_read, can_write) = discussion_permissions(pool, &repository, actor_user_id).await?;
    if !can_read {
        return Err(RepositoryError::PermissionDenied);
    }
    if !can_write {
        return Err(RepositoryError::PermissionDenied);
    }
    if repository.is_archived {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "archived repositories do not accept new discussions".to_owned(),
        ));
    }
    if !repository_discussions_policy_enabled(pool, repository.id).await? {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "repository discussions are disabled by organization policy".to_owned(),
        ));
    }
    if !request.similar_search_acknowledged {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "confirm that you searched for similar discussions before starting a discussion"
                .to_owned(),
        ));
    }

    let category_slug = normalize_slug(&request.category_slug)?;
    let category = load_discussion_category_choices(pool, &repository)
        .await?
        .into_iter()
        .find(|category| category.slug == category_slug)
        .ok_or_else(|| RepositoryError::NotFound)?;
    let title = normalize_required_text(&request.title, "title", 240)?;
    let form = load_discussion_form_definition(pool, repository.id, &category).await?;
    let poll = normalize_discussion_poll(&category, &form, request.poll.as_ref())?;
    if poll.is_some() && !request.form_answers.is_empty() {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "poll discussions cannot include category form answers".to_owned(),
        ));
    }
    let body = if poll.is_some() {
        request
            .body
            .as_deref()
            .map(|value| normalize_required_text(value, "body", 64 * 1024))
            .transpose()?
            .unwrap_or_else(|| "Poll discussion".to_owned())
    } else {
        normalize_optional_body(request.body.as_deref(), 64 * 1024)?
    };
    validate_form_answers(&form, &request.form_answers)?;
    validate_attachment_drafts(&request.attachment_drafts)?;

    let next_number: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(number), 0) + 1 FROM discussions WHERE repository_id = $1",
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let discussion_id = Uuid::new_v4();
    let comment_id = Uuid::new_v4();

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
    .bind(repository.id)
    .bind(category.id)
    .bind(next_number)
    .bind(&title)
    .bind(&body)
    .bind(actor_user_id)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO discussion_comments (id, discussion_id, author_user_id, body) VALUES ($1, $2, $3, $4)",
    )
    .bind(comment_id)
    .bind(discussion_id)
    .bind(actor_user_id)
    .bind(&body)
    .execute(pool)
    .await?;

    for answer in normalized_form_answers(&form, &request.form_answers)? {
        sqlx::query(
            r#"
            INSERT INTO discussion_form_answers (discussion_id, field_id, field_label, value)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(discussion_id)
        .bind(answer.0)
        .bind(answer.1)
        .bind(answer.2)
        .execute(pool)
        .await?;
    }

    if let Some(poll) = poll {
        let poll_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO discussion_polls (id, discussion_id, question, allows_multiple)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(poll_id)
        .bind(discussion_id)
        .bind(&poll.question)
        .bind(poll.allows_multiple)
        .execute(pool)
        .await?;
        for (position, option) in poll.options.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO discussion_poll_options (poll_id, position, label)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(poll_id)
            .bind(position as i32)
            .bind(option)
            .execute(pool)
            .await?;
        }
    }

    for attachment in request.attachment_drafts {
        sqlx::query(
            r#"
            INSERT INTO discussion_attachments (
                id, discussion_id, comment_id, uploaded_by_user_id, file_name,
                content_type, byte_size, storage_key, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'attached')
            "#,
        )
        .bind(attachment.id.unwrap_or_else(Uuid::new_v4))
        .bind(discussion_id)
        .bind(comment_id)
        .bind(actor_user_id)
        .bind(attachment.file_name)
        .bind(attachment.content_type)
        .bind(attachment.byte_size)
        .bind(attachment.storage_key)
        .execute(pool)
        .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO discussion_subscriptions (discussion_id, user_id, state, reason)
        VALUES ($1, $2, 'subscribed', 'participating')
        ON CONFLICT (discussion_id, user_id)
        DO UPDATE SET state = 'subscribed', reason = 'participating', updated_at = now()
        "#,
    )
    .bind(discussion_id)
    .bind(actor_user_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO discussion_activity_events (discussion_id, actor_user_id, event_type, payload)
        VALUES ($1, $2, 'created', jsonb_build_object('category', $3, 'similarSearchAcknowledged', true))
        "#,
    )
    .bind(discussion_id)
    .bind(actor_user_id)
    .bind(&category.slug)
    .execute(pool)
    .await?;

    notify_repository_maintainers_of_discussion(
        pool,
        &repository,
        actor_user_id,
        discussion_id,
        next_number,
        &title,
    )
    .await?;

    Ok(Some(CreateDiscussionResponse {
        discussion_id,
        discussion_number: next_number,
        href: format!(
            "/{}/{}/discussions/{}",
            repository.owner_login, repository.name, next_number
        ),
        title,
        category,
    }))
}

pub async fn create_repository_discussion_comment_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    discussion_number: i64,
    request: CreateDiscussionCommentRequest,
) -> Result<Option<RepositoryDiscussionDetailView>, RepositoryError> {
    create_discussion_comment_or_reply(
        pool,
        actor_user_id,
        owner,
        repo,
        discussion_number,
        None,
        request,
    )
    .await
}

pub async fn create_repository_discussion_reply_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    discussion_number: i64,
    comment_id: Uuid,
    request: CreateDiscussionCommentRequest,
) -> Result<Option<RepositoryDiscussionDetailView>, RepositoryError> {
    create_discussion_comment_or_reply(
        pool,
        actor_user_id,
        owner,
        repo,
        discussion_number,
        Some(comment_id),
        request,
    )
    .await
}

pub async fn toggle_repository_discussion_reaction_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    discussion_number: i64,
    comment_id: Option<Uuid>,
    mutation: DiscussionReactionMutation<'_>,
) -> Result<Option<Vec<DiscussionReactionSummary>>, RepositoryError> {
    let content = normalize_discussion_reaction(mutation.content)?;
    let Some((repository, discussion_id, _title, _author_user_id)) =
        load_discussion_mutation_context(pool, actor_user_id, owner, repo, discussion_number)
            .await?
    else {
        return Ok(None);
    };
    if repository.is_archived {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "archived repositories do not accept discussion reactions".to_owned(),
        ));
    }
    if !repository_discussions_policy_enabled(pool, repository.id).await? {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "repository discussions are disabled by organization policy".to_owned(),
        ));
    }
    if let Some(comment_id) = comment_id {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM discussion_comments WHERE discussion_id = $1 AND id = $2)",
        )
        .bind(discussion_id)
        .bind(comment_id)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(RepositoryError::NotFound);
        }
    }

    let changed = if mutation.reacted {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM discussion_reactions
                WHERE discussion_id = $1
                  AND (($2::uuid IS NULL AND comment_id IS NULL) OR comment_id = $2)
                  AND user_id = $3
                  AND content = $4
            )
            "#,
        )
        .bind(discussion_id)
        .bind(comment_id)
        .bind(actor_user_id)
        .bind(&content)
        .fetch_one(pool)
        .await?;
        if exists {
            false
        } else {
            sqlx::query(
                r#"
                INSERT INTO discussion_reactions (discussion_id, comment_id, user_id, content)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(discussion_id)
            .bind(comment_id)
            .bind(actor_user_id)
            .bind(&content)
            .execute(pool)
            .await?
            .rows_affected()
                > 0
        }
    } else {
        sqlx::query(
            r#"
            DELETE FROM discussion_reactions
            WHERE discussion_id = $1
              AND (($2::uuid IS NULL AND comment_id IS NULL) OR comment_id = $2)
              AND user_id = $3
              AND content = $4
            "#,
        )
        .bind(discussion_id)
        .bind(comment_id)
        .bind(actor_user_id)
        .bind(&content)
        .execute(pool)
        .await?
        .rows_affected()
            > 0
    };

    if changed {
        sqlx::query(
            r#"
            INSERT INTO discussion_activity_events (discussion_id, actor_user_id, event_type, payload)
            VALUES ($1, $2, $3, jsonb_build_object('content', $4, 'commentId', $5))
            "#,
        )
        .bind(discussion_id)
        .bind(actor_user_id)
        .bind(if mutation.reacted {
            "reaction_added"
        } else {
            "reaction_removed"
        })
        .bind(&content)
        .bind(comment_id)
        .execute(pool)
        .await?;
    }

    Ok(Some(
        load_discussion_reactions(pool, discussion_id, comment_id, actor_user_id).await?,
    ))
}

pub async fn set_repository_discussion_subscription_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    discussion_number: i64,
    request: DiscussionSubscriptionRequest,
) -> Result<Option<DiscussionSubscriptionState>, RepositoryError> {
    let Some((repository, discussion_id, _title, _author_user_id)) =
        load_discussion_mutation_context(pool, actor_user_id, owner, repo, discussion_number)
            .await?
    else {
        return Ok(None);
    };
    if !repository_discussions_policy_enabled(pool, repository.id).await? {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "repository discussions are disabled by organization policy".to_owned(),
        ));
    }
    let (state, reason) = if request.subscribed {
        ("subscribed", "manual")
    } else {
        ("unsubscribed", "manual")
    };
    sqlx::query(
        r#"
        INSERT INTO discussion_subscriptions (discussion_id, user_id, state, reason)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (discussion_id, user_id)
        DO UPDATE SET state = EXCLUDED.state, reason = EXCLUDED.reason, updated_at = now()
        "#,
    )
    .bind(discussion_id)
    .bind(actor_user_id)
    .bind(state)
    .bind(reason)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO discussion_activity_events (discussion_id, actor_user_id, event_type, payload)
        VALUES ($1, $2, $3, '{}'::jsonb)
        "#,
    )
    .bind(discussion_id)
    .bind(actor_user_id)
    .bind(if request.subscribed {
        "subscribed"
    } else {
        "unsubscribed"
    })
    .execute(pool)
    .await?;

    load_discussion_subscription(pool, discussion_id, actor_user_id)
        .await
        .map(Some)
}

pub async fn set_repository_discussion_answer_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    discussion_number: i64,
    request: DiscussionAnswerRequest,
    marked: bool,
) -> Result<Option<RepositoryDiscussionDetailView>, RepositoryError> {
    let Some((repository, discussion_id, title, author_user_id)) =
        load_discussion_mutation_context(pool, actor_user_id, owner, repo, discussion_number)
            .await?
    else {
        return Ok(None);
    };
    ensure_discussion_moderation_allowed(pool, &repository, actor_user_id, discussion_id).await?;
    let accepts_answers: bool = sqlx::query_scalar(
        r#"
        SELECT discussion_categories.accepts_answers
        FROM discussions
        JOIN discussion_categories ON discussion_categories.id = discussions.category_id
        WHERE discussions.id = $1
        "#,
    )
    .bind(discussion_id)
    .fetch_one(pool)
    .await?;
    if !accepts_answers {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "this discussion category does not accept answers".to_owned(),
        ));
    }
    let comment_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM discussion_comments
            WHERE discussion_id = $1 AND id = $2 AND parent_comment_id IS NULL AND deleted_at IS NULL
        )
        "#,
    )
    .bind(discussion_id)
    .bind(request.comment_id)
    .fetch_one(pool)
    .await?;
    if !comment_exists {
        return Err(RepositoryError::NotFound);
    }

    if marked {
        sqlx::query(
            r#"
            INSERT INTO discussion_answers (discussion_id, comment_id, marked_by_user_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (discussion_id)
            DO UPDATE SET comment_id = EXCLUDED.comment_id,
                          marked_by_user_id = EXCLUDED.marked_by_user_id,
                          marked_at = now()
            "#,
        )
        .bind(discussion_id)
        .bind(request.comment_id)
        .bind(actor_user_id)
        .execute(pool)
        .await?;
        sqlx::query(
            "UPDATE discussions SET answer_comment_id = $1, answered = true, updated_at = now(), last_activity_at = now() WHERE id = $2",
        )
        .bind(request.comment_id)
        .bind(discussion_id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query("DELETE FROM discussion_answers WHERE discussion_id = $1")
            .bind(discussion_id)
            .execute(pool)
            .await?;
        sqlx::query(
            "UPDATE discussions SET answer_comment_id = NULL, answered = false, updated_at = now(), last_activity_at = now() WHERE id = $1",
        )
        .bind(discussion_id)
        .execute(pool)
        .await?;
    }

    let event_type = if marked {
        "answer_marked"
    } else {
        "answer_unmarked"
    };
    sqlx::query(
        r#"
        INSERT INTO discussion_activity_events (discussion_id, actor_user_id, event_type, payload)
        VALUES ($1, $2, $3, jsonb_build_object('commentId', $4::text))
        "#,
    )
    .bind(discussion_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(request.comment_id)
    .execute(pool)
    .await?;
    notify_discussion_author(
        pool,
        repository.id,
        discussion_id,
        author_user_id,
        actor_user_id,
        format!(
            "Discussion #{} answer state changed: {}",
            discussion_number, title
        ),
        "discussion_answer",
    )
    .await?;

    repository_discussion_detail_for_actor_by_owner_name(
        pool,
        actor_user_id,
        owner,
        repo,
        discussion_number,
        RepositoryDiscussionDetailQuery {
            sort: None,
            page: None,
            page_size: None,
        },
    )
    .await
}

pub async fn update_repository_discussion_state_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    discussion_number: i64,
    request: DiscussionStateRequest,
) -> Result<Option<RepositoryDiscussionDetailView>, RepositoryError> {
    let Some((repository, discussion_id, title, author_user_id)) =
        load_discussion_mutation_context(pool, actor_user_id, owner, repo, discussion_number)
            .await?
    else {
        return Ok(None);
    };
    ensure_discussion_moderation_allowed(pool, &repository, actor_user_id, discussion_id).await?;
    let next_state = match request.state.trim() {
        "open" => "open",
        "closed" => "closed",
        other => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported discussion state `{other}`"
            )))
        }
    };
    let reason = normalize_discussion_state_reason(request.reason.as_deref(), next_state)?;
    let current_state: String = sqlx::query_scalar("SELECT state FROM discussions WHERE id = $1")
        .bind(discussion_id)
        .fetch_one(pool)
        .await?;
    if current_state != next_state {
        sqlx::query(
            "UPDATE discussions SET state = $1, updated_at = now(), last_activity_at = now() WHERE id = $2",
        )
        .bind(next_state)
        .bind(discussion_id)
        .execute(pool)
        .await?;
        sqlx::query(
            r#"
            INSERT INTO discussion_activity_events (discussion_id, actor_user_id, event_type, payload)
            VALUES ($1, $2, $3, jsonb_build_object('reason', $4))
            "#,
        )
        .bind(discussion_id)
        .bind(actor_user_id)
        .bind(if next_state == "closed" {
            "closed"
        } else {
            "reopened"
        })
        .bind(reason)
        .execute(pool)
        .await?;
        notify_discussion_author(
            pool,
            repository.id,
            discussion_id,
            author_user_id,
            actor_user_id,
            format!(
                "Discussion #{} was {}: {}",
                discussion_number, next_state, title
            ),
            "discussion_state",
        )
        .await?;
    }

    repository_discussion_detail_for_actor_by_owner_name(
        pool,
        actor_user_id,
        owner,
        repo,
        discussion_number,
        RepositoryDiscussionDetailQuery {
            sort: None,
            page: None,
            page_size: None,
        },
    )
    .await
}

pub async fn update_repository_discussion_metadata_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    discussion_number: i64,
    request: DiscussionMetadataRequest,
) -> Result<Option<RepositoryDiscussionDetailView>, RepositoryError> {
    let Some((repository, discussion_id, _title, _author_user_id)) =
        load_discussion_mutation_context(pool, actor_user_id, owner, repo, discussion_number)
            .await?
    else {
        return Ok(None);
    };
    ensure_discussion_moderation_allowed(pool, &repository, actor_user_id, discussion_id).await?;

    if let Some(category_slug) = request.category_slug.as_deref() {
        let normalized_slug = normalize_slug(category_slug)?;
        let Some(category) = load_discussion_category_choices(pool, &repository)
            .await?
            .into_iter()
            .find(|category| category.slug == normalized_slug)
        else {
            return Err(RepositoryError::NotFound);
        };
        let has_poll = load_discussion_poll_view(pool, discussion_id)
            .await?
            .is_some();
        let has_form_answers: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM discussion_form_answers WHERE discussion_id = $1)",
        )
        .bind(discussion_id)
        .fetch_one(pool)
        .await?;
        if has_poll && !category.is_poll {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "poll discussions must stay in a poll category".to_owned(),
            ));
        }
        if has_form_answers && category.is_poll {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "form discussions cannot move into a poll category".to_owned(),
            ));
        }
        sqlx::query("UPDATE discussions SET category_id = $1, updated_at = now(), last_activity_at = now() WHERE id = $2")
            .bind(category.id)
            .bind(discussion_id)
            .execute(pool)
            .await?;
        sqlx::query(
            r#"
            INSERT INTO discussion_activity_events (discussion_id, actor_user_id, event_type, payload)
            VALUES ($1, $2, 'category_changed', jsonb_build_object('category', $3))
            "#,
        )
        .bind(discussion_id)
        .bind(actor_user_id)
        .bind(category.slug)
        .execute(pool)
        .await?;
    }

    if let Some(label_ids) = request.label_ids {
        if label_ids.len() > 25 {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "at most 25 labels can be assigned to a discussion".to_owned(),
            ));
        }
        for label_id in &label_ids {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM labels WHERE id = $1 AND repository_id = $2)",
            )
            .bind(label_id)
            .bind(repository.id)
            .fetch_one(pool)
            .await?;
            if !exists {
                return Err(RepositoryError::NotFound);
            }
        }
        sqlx::query("DELETE FROM discussion_labels WHERE discussion_id = $1")
            .bind(discussion_id)
            .execute(pool)
            .await?;
        for label_id in label_ids {
            sqlx::query(
                "INSERT INTO discussion_labels (discussion_id, label_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(discussion_id)
            .bind(label_id)
            .execute(pool)
            .await?;
        }
        sqlx::query(
            r#"
            INSERT INTO discussion_activity_events (discussion_id, actor_user_id, event_type, payload)
            VALUES ($1, $2, 'labels_changed', '{}'::jsonb)
            "#,
        )
        .bind(discussion_id)
        .bind(actor_user_id)
        .execute(pool)
        .await?;
    }

    repository_discussion_detail_for_actor_by_owner_name(
        pool,
        actor_user_id,
        owner,
        repo,
        discussion_number,
        RepositoryDiscussionDetailQuery {
            sort: None,
            page: None,
            page_size: None,
        },
    )
    .await
}

fn normalize_discussion_filters(
    query: RepositoryDiscussionsQuery<'_>,
) -> Result<NormalizedDiscussionFilters, RepositoryError> {
    let state = match query.state.map(str::trim).filter(|value| !value.is_empty()) {
        Some("open") | None => DiscussionStateFilter::Open,
        Some("closed") => DiscussionStateFilter::Closed,
        Some("all") => DiscussionStateFilter::All,
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported discussion state `{other}`"
            )))
        }
    };
    let sort = match query.sort.map(str::trim).filter(|value| !value.is_empty()) {
        Some(sort @ ("latest" | "newest" | "top" | "most_commented")) => sort.to_owned(),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported discussion sort `{other}`"
            )))
        }
        None => "latest".to_owned(),
    };
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(30);
    if page < 1 || !(1..=100).contains(&page_size) {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "page must be positive and page_size must be between 1 and 100".to_owned(),
        ));
    }
    Ok(NormalizedDiscussionFilters {
        query: normalize_short_text(query.q, "q", 160)?.unwrap_or_else(|| "is:open".to_owned()),
        label: normalize_short_text(query.label, "label", 80)?,
        state,
        answered: normalize_bool(query.answered, "answered")?,
        locked: normalize_bool(query.locked, "locked")?,
        pinned: normalize_bool(query.pinned, "pinned")?,
        sort,
        page,
        page_size,
    })
}

fn normalize_discussion_detail_query(
    query: RepositoryDiscussionDetailQuery<'_>,
) -> Result<NormalizedDiscussionDetailQuery, RepositoryError> {
    let sort = match query.sort.map(str::trim).filter(|value| !value.is_empty()) {
        Some(sort @ ("oldest" | "newest" | "top")) => sort.to_owned(),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported discussion comment sort `{other}`"
            )))
        }
        None => "oldest".to_owned(),
    };
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(30);
    if page < 1 || !(1..=100).contains(&page_size) {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "page must be positive and page_size must be between 1 and 100".to_owned(),
        ));
    }
    Ok(NormalizedDiscussionDetailQuery {
        sort,
        page,
        page_size,
    })
}

async fn load_discussion_mutation_context(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    discussion_number: i64,
) -> Result<Option<(super::repositories::Repository, Uuid, String, Option<Uuid>)>, RepositoryError>
{
    if discussion_number < 1 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "discussion number must be positive".to_owned(),
        ));
    }
    let Some(repository) = get_repository_by_owner_name(pool, owner, repo).await? else {
        return Ok(None);
    };
    let (_, can_read, _) = discussion_permissions(pool, &repository, actor_user_id).await?;
    if !can_read {
        return Err(RepositoryError::PermissionDenied);
    }
    let Some(row) = sqlx::query(
        "SELECT id, title, author_user_id FROM discussions WHERE repository_id = $1 AND number = $2",
    )
    .bind(repository.id)
    .bind(discussion_number)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };
    Ok(Some((
        repository,
        row.try_get("id")?,
        row.try_get("title")?,
        row.try_get("author_user_id")?,
    )))
}

async fn ensure_discussion_moderation_allowed(
    pool: &PgPool,
    repository: &super::repositories::Repository,
    actor_user_id: Uuid,
    discussion_id: Uuid,
) -> Result<(), RepositoryError> {
    if repository.is_archived {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "archived repositories do not accept discussion moderation changes".to_owned(),
        ));
    }
    if !repository_discussions_policy_enabled(pool, repository.id).await? {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "repository discussions are disabled by organization policy".to_owned(),
        ));
    }
    let (viewer_permission, _can_read, _can_write) =
        discussion_permissions(pool, repository, actor_user_id).await?;
    if !matches!(
        viewer_permission.as_deref(),
        Some("triage" | "write" | "maintain" | "admin" | "owner")
    ) {
        return Err(RepositoryError::PermissionDenied);
    }
    let locked: bool = sqlx::query_scalar("SELECT locked FROM discussions WHERE id = $1")
        .bind(discussion_id)
        .fetch_one(pool)
        .await?;
    if locked {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "locked discussions do not accept moderation changes".to_owned(),
        ));
    }
    Ok(())
}

fn normalize_discussion_state_reason(
    reason: Option<&str>,
    next_state: &str,
) -> Result<Option<String>, RepositoryError> {
    if next_state == "open" {
        return Ok(None);
    }
    let reason = reason.map(str::trim).filter(|value| !value.is_empty());
    match reason.unwrap_or("resolved") {
        value @ ("resolved" | "duplicate" | "outdated" | "off-topic") => Ok(Some(value.to_owned())),
        other => Err(RepositoryError::InvalidDependencyGraphQuery(format!(
            "unsupported discussion close reason `{other}`"
        ))),
    }
}

async fn notify_discussion_author(
    pool: &PgPool,
    repository_id: Uuid,
    discussion_id: Uuid,
    author_user_id: Option<Uuid>,
    actor_user_id: Uuid,
    title: String,
    reason: &str,
) -> Result<(), RepositoryError> {
    let Some(author_user_id) = author_user_id.filter(|id| *id != actor_user_id) else {
        return Ok(());
    };
    create_notification(
        pool,
        CreateNotification {
            user_id: author_user_id,
            repository_id: Some(repository_id),
            subject_type: "discussion".to_owned(),
            subject_id: Some(discussion_id),
            title,
            reason: reason.to_owned(),
        },
    )
    .await
    .map_err(|error| match error {
        super::notifications::NotificationError::Sqlx(error) => RepositoryError::Sqlx(error),
        super::notifications::NotificationError::NotFound
        | super::notifications::NotificationError::Validation(_) => RepositoryError::NotFound,
    })?;
    Ok(())
}

async fn create_discussion_comment_or_reply(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: &str,
    repo: &str,
    discussion_number: i64,
    parent_comment_id: Option<Uuid>,
    request: CreateDiscussionCommentRequest,
) -> Result<Option<RepositoryDiscussionDetailView>, RepositoryError> {
    let Some((repository, discussion_id, title, author_user_id)) =
        load_discussion_mutation_context(pool, actor_user_id, owner, repo, discussion_number)
            .await?
    else {
        return Ok(None);
    };
    if repository.is_archived {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "archived repositories do not accept discussion comments".to_owned(),
        ));
    }
    if !repository_discussions_policy_enabled(pool, repository.id).await? {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "repository discussions are disabled by organization policy".to_owned(),
        ));
    }
    let locked: bool = sqlx::query_scalar("SELECT locked FROM discussions WHERE id = $1")
        .bind(discussion_id)
        .fetch_one(pool)
        .await?;
    if locked {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "locked discussions do not accept new comments".to_owned(),
        ));
    }
    if let Some(parent_comment_id) = parent_comment_id {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM discussion_comments
                WHERE discussion_id = $1 AND id = $2 AND parent_comment_id IS NULL
            )
            "#,
        )
        .bind(discussion_id)
        .bind(parent_comment_id)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(RepositoryError::NotFound);
        }
    }

    let body = normalize_required_text(&request.body, "body", 64 * 1024)?;
    validate_attachment_drafts(&request.attachment_drafts)?;
    let comment_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO discussion_comments (id, discussion_id, parent_comment_id, author_user_id, body)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(comment_id)
    .bind(discussion_id)
    .bind(parent_comment_id)
    .bind(actor_user_id)
    .bind(&body)
    .execute(pool)
    .await?;

    for attachment in request.attachment_drafts {
        sqlx::query(
            r#"
            INSERT INTO discussion_attachments (
                id, discussion_id, comment_id, uploaded_by_user_id, file_name,
                content_type, byte_size, storage_key, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'attached')
            "#,
        )
        .bind(attachment.id.unwrap_or_else(Uuid::new_v4))
        .bind(discussion_id)
        .bind(comment_id)
        .bind(actor_user_id)
        .bind(attachment.file_name)
        .bind(attachment.content_type)
        .bind(attachment.byte_size)
        .bind(attachment.storage_key)
        .execute(pool)
        .await?;
    }

    sqlx::query(
        r#"
        UPDATE discussions
        SET comments_count = (
                SELECT COUNT(*)::bigint
                FROM discussion_comments
                WHERE discussion_id = $1
            ),
            updated_at = now(),
            last_activity_at = now()
        WHERE id = $1
        "#,
    )
    .bind(discussion_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO discussion_subscriptions (discussion_id, user_id, state, reason)
        VALUES ($1, $2, 'subscribed', 'participating')
        ON CONFLICT (discussion_id, user_id)
        DO UPDATE SET state = 'subscribed', reason = 'participating', updated_at = now()
        "#,
    )
    .bind(discussion_id)
    .bind(actor_user_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO discussion_activity_events (discussion_id, actor_user_id, event_type, payload)
        VALUES ($1, $2, $3, jsonb_build_object('commentId', $4, 'parentCommentId', $5))
        "#,
    )
    .bind(discussion_id)
    .bind(actor_user_id)
    .bind(if parent_comment_id.is_some() {
        "replied"
    } else {
        "commented"
    })
    .bind(comment_id)
    .bind(parent_comment_id)
    .execute(pool)
    .await?;

    if let Some(author_user_id) = author_user_id.filter(|id| *id != actor_user_id) {
        create_notification(
            pool,
            CreateNotification {
                user_id: author_user_id,
                repository_id: Some(repository.id),
                subject_type: "discussion".to_owned(),
                subject_id: Some(discussion_id),
                title: format!(
                    "New comment on discussion #{}: {}",
                    discussion_number, title
                ),
                reason: "discussion_comment".to_owned(),
            },
        )
        .await
        .map_err(|error| match error {
            super::notifications::NotificationError::Sqlx(error) => RepositoryError::Sqlx(error),
            super::notifications::NotificationError::NotFound
            | super::notifications::NotificationError::Validation(_) => RepositoryError::NotFound,
        })?;
    }

    repository_discussion_detail_for_actor_by_owner_name(
        pool,
        actor_user_id,
        owner,
        repo,
        discussion_number,
        RepositoryDiscussionDetailQuery {
            sort: Some("oldest"),
            page: Some(1),
            page_size: Some(100),
        },
    )
    .await
}

fn normalize_discussion_reaction(value: &str) -> Result<String, RepositoryError> {
    match value.trim() {
        "+1" | "thumbs_up" => Ok("+1".to_owned()),
        "-1" | "thumbs_down" => Ok("-1".to_owned()),
        content @ ("laugh" | "hooray" | "confused" | "heart" | "rocket" | "eyes") => {
            Ok(content.to_owned())
        }
        other => Err(RepositoryError::InvalidDependencyGraphQuery(format!(
            "unsupported discussion reaction `{other}`"
        ))),
    }
}

fn normalize_short_text(
    value: Option<&str>,
    field: &str,
    max_len: usize,
) -> Result<Option<String>, RepositoryError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if value.len() > max_len {
        return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
            "{field} must be at most {max_len} characters"
        )));
    }
    Ok(Some(value.to_owned()))
}

fn discussion_body_view(markdown: String) -> DiscussionBodyView {
    DiscussionBodyView {
        html: ammonia::clean(&markdown),
        markdown,
    }
}

fn normalize_required_text(
    value: &str,
    field: &str,
    max_len: usize,
) -> Result<String, RepositoryError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
            "{field} must not be empty"
        )));
    }
    if value.len() > max_len {
        return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
            "{field} must be at most {max_len} characters"
        )));
    }
    Ok(value.to_owned())
}

fn normalize_optional_body(value: Option<&str>, max_len: usize) -> Result<String, RepositoryError> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "body must not be empty".to_owned(),
        ));
    }
    if value.len() > max_len {
        return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
            "body must be at most {max_len} characters"
        )));
    }
    Ok(value.to_owned())
}

fn validate_form_answers(
    form: &DiscussionFormDefinition,
    answers: &[DiscussionFormAnswerInput],
) -> Result<(), RepositoryError> {
    if form.fallback || form.fields.is_empty() {
        return Ok(());
    }
    let answer_map = answers
        .iter()
        .map(|answer| (answer.field_id.trim(), answer.value.trim()))
        .collect::<std::collections::HashMap<_, _>>();
    for field in &form.fields {
        if field.required
            && answer_map
                .get(field.id.as_str())
                .is_none_or(|value| value.is_empty())
        {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "{} is required",
                field.label
            )));
        }
    }
    for answer in answers {
        if answer.field_id.len() > 80 || answer.value.len() > 20_000 {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "form answers are too large".to_owned(),
            ));
        }
    }
    Ok(())
}

fn normalize_discussion_poll(
    category: &DiscussionCategoryChoice,
    form: &DiscussionFormDefinition,
    poll: Option<&DiscussionPollInput>,
) -> Result<Option<NormalizedDiscussionPoll>, RepositoryError> {
    let Some(poll) = poll else {
        if category.is_poll {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "poll question and options are required for this category".to_owned(),
            ));
        }
        return Ok(None);
    };

    if !category.is_poll {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "polls can only be created in poll discussion categories".to_owned(),
        ));
    }
    if !form.fields.is_empty() && !form.fallback {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "poll categories cannot be submitted with category form fields".to_owned(),
        ));
    }

    let question = normalize_required_text(&poll.question, "poll question", 240)?;
    let mut options = Vec::new();
    for option in &poll.options {
        let option = option.trim();
        if option.is_empty() {
            continue;
        }
        if option.len() > 160 {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "poll options must be at most 160 characters".to_owned(),
            ));
        }
        if options
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(option))
        {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "poll options must be unique".to_owned(),
            ));
        }
        options.push(option.to_owned());
    }
    if !(2..=10).contains(&options.len()) {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "polls require between 2 and 10 options".to_owned(),
        ));
    }

    Ok(Some(NormalizedDiscussionPoll {
        question,
        options,
        allows_multiple: poll.allows_multiple,
    }))
}

fn normalized_form_answers(
    form: &DiscussionFormDefinition,
    answers: &[DiscussionFormAnswerInput],
) -> Result<Vec<(String, String, String)>, RepositoryError> {
    let mut out = Vec::new();
    for answer in answers {
        let field_id = slugify(&answer.field_id);
        let value = answer.value.trim();
        if field_id.is_empty() || value.is_empty() {
            continue;
        }
        let Some(field) = form.fields.iter().find(|field| field.id == field_id) else {
            if form.fields.is_empty() || form.fallback {
                continue;
            }
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported discussion form field `{field_id}`"
            )));
        };
        out.push((field_id, field.label.clone(), value.to_owned()));
    }
    Ok(out)
}

fn validate_attachment_drafts(
    attachments: &[DiscussionAttachmentDraft],
) -> Result<(), RepositoryError> {
    if attachments.len() > 10 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "a discussion can attach at most 10 files".to_owned(),
        ));
    }
    for attachment in attachments {
        normalize_required_text(&attachment.file_name, "attachment file name", 180)?;
        normalize_required_text(&attachment.storage_key, "attachment storage key", 300)?;
        normalize_required_text(&attachment.content_type, "attachment content type", 120)?;
        if attachment.byte_size < 0 || attachment.byte_size > 25 * 1024 * 1024 {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "attachment size must be between 0 and 25 MiB".to_owned(),
            ));
        }
    }
    Ok(())
}

async fn notify_repository_maintainers_of_discussion(
    pool: &PgPool,
    repository: &super::repositories::Repository,
    actor_user_id: Uuid,
    discussion_id: Uuid,
    discussion_number: i64,
    title: &str,
) -> Result<(), RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT user_id
        FROM repository_permissions
        WHERE repository_id = $1
          AND user_id <> $2
          AND role IN ('owner', 'admin', 'write')
        LIMIT 20
        "#,
    )
    .bind(repository.id)
    .bind(actor_user_id)
    .fetch_all(pool)
    .await?;
    for row in rows {
        let user_id: Uuid = row.try_get("user_id")?;
        create_notification(
            pool,
            CreateNotification {
                user_id,
                repository_id: Some(repository.id),
                subject_type: "discussion".to_owned(),
                subject_id: Some(discussion_id),
                title: format!("Discussion #{} started: {}", discussion_number, title),
                reason: "discussion_created".to_owned(),
            },
        )
        .await
        .map_err(|error| match error {
            super::notifications::NotificationError::Sqlx(error) => RepositoryError::Sqlx(error),
            super::notifications::NotificationError::NotFound
            | super::notifications::NotificationError::Validation(_) => RepositoryError::NotFound,
        })?;
    }
    Ok(())
}

fn sanitize_template_text(value: impl AsRef<str>) -> String {
    let html = ammonia::clean(value.as_ref());
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.trim().chars().take(2000).collect()
}

fn slugify(value: &str) -> String {
    let mut slug = String::with_capacity(value.len());
    let mut last_dash = false;
    for ch in value.trim().to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    slug.trim_matches('-').chars().take(80).collect()
}

fn normalize_slug(value: &str) -> Result<String, RepositoryError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 80
        || !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "discussion category slug is invalid".to_owned(),
        ));
    }
    Ok(value.to_owned())
}

fn normalize_bool(value: Option<&str>, field: &str) -> Result<Option<bool>, RepositoryError> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some("true" | "1" | "yes") => Ok(Some(true)),
        Some("false" | "0" | "no") => Ok(Some(false)),
        Some(other) => Err(RepositoryError::InvalidDependencyGraphQuery(format!(
            "{field} must be a boolean, got `{other}`"
        ))),
        None => Ok(None),
    }
}

#[derive(Debug, Clone)]
struct CurrentBranchCommit {
    id: Uuid,
    oid: String,
}

async fn load_discussion_category_admin_item(
    pool: &PgPool,
    repository: &Repository,
    category_id: Uuid,
) -> Result<Option<DiscussionCategoryAdminItem>, RepositoryError> {
    Ok(load_discussion_category_admin_items(pool, repository)
        .await?
        .into_iter()
        .find(|category| category.id == category_id))
}

async fn discussion_category_template_view(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    category: DiscussionCategoryAdminItem,
    override_content: Option<String>,
) -> Result<DiscussionCategoryTemplateView, RepositoryError> {
    let (permission, can_read, can_write) =
        discussion_permissions(pool, repository, actor_user_id).await?;
    let path = category
        .template_path
        .clone()
        .unwrap_or_else(|| discussion_template_path(&category.slug));
    let content = match override_content {
        Some(content) => content,
        None => current_discussion_template_file(pool, repository.id, &path)
            .await?
            .map(|file| file.content)
            .unwrap_or_else(|| default_discussion_template_content(&category)),
    };
    let form = if category.is_poll {
        DiscussionFormDefinition {
            category_slug: Some(category.slug.clone()),
            template_path: None,
            title: "Start a poll".to_owned(),
            description: category.description.clone(),
            body: String::new(),
            fields: Vec::new(),
            valid: false,
            fallback: true,
            parse_error: Some("Poll categories cannot use YAML templates.".to_owned()),
        }
    } else {
        parse_discussion_template(&content, &category.slug, &path)
    };
    let blob_href = Some(format!(
        "/{}/{}/blob/{}/{}",
        repository.owner_login, repository.name, repository.default_branch, path
    ));
    Ok(DiscussionCategoryTemplateView {
        repository: discussion_repository_summary(repository),
        viewer: DiscussionCategoryAdminViewer {
            authenticated: true,
            permission,
            can_read,
            can_manage: can_write || repository.owner_user_id == Some(actor_user_id),
        },
        category,
        content_sha: content_sha(&content),
        content,
        path,
        branch: repository.default_branch.clone(),
        form,
        commit_href: None,
        blob_href,
    })
}

fn discussion_template_path(category_slug: &str) -> String {
    format!(".github/DISCUSSION_TEMPLATE/{}.yml", slugify(category_slug))
}

fn default_discussion_template_content(category: &DiscussionCategoryAdminItem) -> String {
    format!(
        "name: \"{}\"\ndescription: \"{}\"\nbody:\n  - type: textarea\n    id: details\n    attributes:\n      label: Details\n      description: Share the context maintainers need.\n      placeholder: What should other contributors know?\n    validations:\n      required: true\n",
        sanitize_template_text(&category.name),
        sanitize_template_text(
            category
                .description
                .as_deref()
                .unwrap_or("Start a focused repository discussion.")
        )
    )
}

fn normalize_discussion_template_content(content: &str) -> Result<String, RepositoryError> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "discussion template YAML cannot be empty".to_owned(),
        ));
    }
    if trimmed.len() > 32_000 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "discussion template YAML must be 32 KB or smaller".to_owned(),
        ));
    }
    Ok(format!("{trimmed}\n"))
}

fn normalize_template_commit_message(message: &str) -> Result<String, RepositoryError> {
    let message = sanitize_template_text(message);
    if message.trim().is_empty() {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "commit message is required".to_owned(),
        ));
    }
    if message.len() > 240 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "commit message must be 240 characters or fewer".to_owned(),
        ));
    }
    Ok(message)
}

fn normalize_template_branch(
    repository: &Repository,
    branch: Option<&str>,
    propose_change: Option<bool>,
) -> Result<String, RepositoryError> {
    let requested = branch.map(str::trim).filter(|value| !value.is_empty());
    let branch = if propose_change.unwrap_or(false) {
        requested.unwrap_or("discussion-template-update")
    } else {
        requested.unwrap_or(&repository.default_branch)
    };
    if branch.len() > 120
        || branch.contains("..")
        || branch.starts_with('/')
        || branch.ends_with('/')
        || !branch
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '/' | '.'))
    {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "branch name is invalid".to_owned(),
        ));
    }
    Ok(branch.to_owned())
}

fn content_sha(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn deterministic_content_oid(kind: &str, content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update([0]);
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn repository_discussions_policy_enabled(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<bool, RepositoryError> {
    Ok(sqlx::query_scalar::<_, bool>(
        r#"
        SELECT COALESCE(organization_policy_settings.repository_discussions_enabled, true)
        FROM repositories
        LEFT JOIN organization_policy_settings
          ON organization_policy_settings.organization_id = repositories.owner_organization_id
        WHERE repositories.id = $1
        "#,
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?)
}

async fn discussion_permissions(
    pool: &PgPool,
    repository: &super::repositories::Repository,
    actor_user_id: Uuid,
) -> Result<(Option<String>, bool, bool), RepositoryError> {
    let permission = repository_permission_for_user(pool, repository.id, actor_user_id).await?;
    let can_read = repository.visibility == RepositoryVisibility::Public
        || repository.owner_user_id == Some(actor_user_id)
        || permission.as_ref().is_some_and(|p| p.role.can_read());
    let viewer_permission = permission.map(|p| p.role.as_str().to_owned()).or_else(|| {
        (repository.owner_user_id == Some(actor_user_id))
            .then(|| RepositoryRole::Admin.as_str().to_owned())
    });
    let can_write = matches!(
        viewer_permission.as_deref(),
        Some("write" | "maintain" | "admin" | "owner")
    );
    Ok((viewer_permission, can_read, can_write))
}

fn discussion_repository_summary(
    repository: &super::repositories::Repository,
) -> DiscussionRepositorySummary {
    DiscussionRepositorySummary {
        id: repository.id,
        owner: repository.owner_login.clone(),
        name: repository.name.clone(),
        visibility: repository.visibility.as_str().to_owned(),
        is_archived: repository.is_archived,
        href: format!("/{}/{}", repository.owner_login, repository.name),
        discussions_href: format!(
            "/{}/{}/discussions",
            repository.owner_login, repository.name
        ),
    }
}

async fn load_discussion_category_choices(
    pool: &PgPool,
    repository: &super::repositories::Repository,
) -> Result<Vec<DiscussionCategoryChoice>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT discussion_categories.id, discussion_categories.slug, discussion_categories.name,
               discussion_categories.emoji, discussion_categories.description,
               discussion_categories.accepts_answers,
               COUNT(discussions.id)::bigint AS count,
               COUNT(discussions.id) FILTER (WHERE discussions.state = 'open')::bigint AS open_count
        FROM discussion_categories
        LEFT JOIN discussions ON discussions.category_id = discussion_categories.id
        WHERE discussion_categories.repository_id = $1
        GROUP BY discussion_categories.id
        ORDER BY discussion_categories.position ASC, discussion_categories.name ASC
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let slug: String = row.try_get("slug")?;
            Ok(DiscussionCategoryChoice {
                id: row.try_get("id")?,
                form_href: format!(
                    "/{}/{}/discussions/new?category={}",
                    repository.owner_login, repository.name, slug
                ),
                href: format!(
                    "/{}/{}/discussions/categories/{}",
                    repository.owner_login, repository.name, slug
                ),
                is_poll: slug == "polls" || slug == "poll",
                slug,
                name: row.try_get("name")?,
                emoji: row.try_get("emoji")?,
                description: row.try_get("description")?,
                accepts_answers: row.try_get("accepts_answers")?,
                count: row.try_get("count")?,
                open_count: row.try_get("open_count")?,
            })
        })
        .collect()
}

async fn load_discussion_category_sections(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<DiscussionCategorySectionItem>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT discussion_category_sections.id,
               discussion_category_sections.name,
               discussion_category_sections.position,
               COUNT(discussion_categories.id)::bigint AS category_count
        FROM discussion_category_sections
        LEFT JOIN discussion_categories
          ON discussion_categories.section_id = discussion_category_sections.id
        WHERE discussion_category_sections.repository_id = $1
        GROUP BY discussion_category_sections.id
        ORDER BY discussion_category_sections.position ASC, discussion_category_sections.name ASC
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(DiscussionCategorySectionItem {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                position: row.try_get("position")?,
                category_count: row.try_get("category_count")?,
            })
        })
        .collect()
}

async fn load_discussion_category_admin_items(
    pool: &PgPool,
    repository: &super::repositories::Repository,
) -> Result<Vec<DiscussionCategoryAdminItem>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT discussion_categories.id,
               discussion_categories.slug,
               discussion_categories.name,
               discussion_categories.emoji,
               discussion_categories.description,
               discussion_categories.format,
               discussion_categories.accepts_answers,
               discussion_categories.is_default,
               discussion_categories.section_id,
               discussion_category_sections.name AS section_name,
               discussion_categories.template_path,
               discussion_categories.position,
               discussion_categories.created_at,
               discussion_categories.updated_at,
               COUNT(discussions.id)::bigint AS count,
               COUNT(discussions.id) FILTER (WHERE discussions.state = 'open')::bigint AS open_count
        FROM discussion_categories
        LEFT JOIN discussion_category_sections
          ON discussion_category_sections.id = discussion_categories.section_id
        LEFT JOIN discussions ON discussions.category_id = discussion_categories.id
        WHERE discussion_categories.repository_id = $1
        GROUP BY discussion_categories.id, discussion_category_sections.name
        ORDER BY COALESCE(discussion_category_sections.position, -1) ASC,
                 discussion_categories.position ASC,
                 discussion_categories.name ASC
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let slug: String = row.try_get("slug")?;
            let id: Uuid = row.try_get("id")?;
            let format =
                DiscussionCategoryFormat::try_from(row.try_get::<String, _>("format")?.as_str())?;
            Ok(DiscussionCategoryAdminItem {
                id,
                href: format!(
                    "/{}/{}/discussions/categories/{}",
                    repository.owner_login, repository.name, slug
                ),
                edit_href: format!(
                    "/{}/{}/discussions/categories/edit",
                    repository.owner_login, repository.name
                ),
                template_href: format!(
                    "/{}/{}/discussions/categories/{}/template",
                    repository.owner_login, repository.name, id
                ),
                is_poll: format == DiscussionCategoryFormat::Poll,
                slug,
                name: row.try_get("name")?,
                emoji: row.try_get("emoji")?,
                description: row.try_get("description")?,
                accepts_answers: row.try_get("accepts_answers")?,
                format,
                is_default: row.try_get("is_default")?,
                section_id: row.try_get("section_id")?,
                section_name: row.try_get("section_name")?,
                template_path: row.try_get("template_path")?,
                count: row.try_get("count")?,
                open_count: row.try_get("open_count")?,
                position: row.try_get("position")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })
        })
        .collect()
}

async fn ensure_category_admin_allowed(
    pool: &PgPool,
    repository: &super::repositories::Repository,
    actor_user_id: Uuid,
) -> Result<(), RepositoryError> {
    let (_permission, can_read, can_write) =
        discussion_permissions(pool, repository, actor_user_id).await?;
    if !can_read || !(can_write || repository.owner_user_id == Some(actor_user_id)) {
        return Err(RepositoryError::PermissionDenied);
    }
    Ok(())
}

async fn load_category_admin_row(
    pool: &PgPool,
    repository_id: Uuid,
    category_id: Uuid,
) -> Result<Option<sqlx::postgres::PgRow>, RepositoryError> {
    Ok(sqlx::query(
        r#"
        SELECT id, name, emoji, description, format, section_id, template_path
        FROM discussion_categories
        WHERE repository_id = $1 AND id = $2
        "#,
    )
    .bind(repository_id)
    .bind(category_id)
    .fetch_optional(pool)
    .await?)
}

async fn ensure_category_section_exists(
    pool: &PgPool,
    repository_id: Uuid,
    section_id: Option<Uuid>,
) -> Result<(), RepositoryError> {
    if let Some(section_id) = section_id {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM discussion_category_sections WHERE repository_id = $1 AND id = $2)",
        )
        .bind(repository_id)
        .bind(section_id)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(RepositoryError::NotFound);
        }
    }
    Ok(())
}

async fn ensure_category_uniqueness(
    pool: &PgPool,
    repository_id: Uuid,
    except_category_id: Option<Uuid>,
    name: &str,
    emoji: &str,
) -> Result<(), RepositoryError> {
    let normalized = name.to_ascii_lowercase();
    let conflict: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM discussion_categories
            WHERE repository_id = $1
              AND ($2::uuid IS NULL OR id <> $2)
              AND (lower(name) = $3 OR (lower(name) = $3 AND emoji = $4))
        )
        "#,
    )
    .bind(repository_id)
    .bind(except_category_id)
    .bind(normalized)
    .bind(emoji)
    .fetch_one(pool)
    .await?;
    if conflict {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "discussion category names must be unique within the repository".to_owned(),
        ));
    }
    Ok(())
}

async fn unique_category_slug(
    pool: &PgPool,
    repository_id: Uuid,
    name: &str,
) -> Result<String, RepositoryError> {
    let base = slugify(name);
    if base.is_empty() {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "discussion category name must produce a URL slug".to_owned(),
        ));
    }
    for suffix in 0..25 {
        let candidate = if suffix == 0 {
            base.clone()
        } else {
            format!("{base}-{suffix}")
        };
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM discussion_categories WHERE repository_id = $1 AND slug = $2)",
        )
        .bind(repository_id)
        .bind(&candidate)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Ok(candidate);
        }
    }
    Err(RepositoryError::InvalidDependencyGraphQuery(
        "discussion category slug could not be made unique".to_owned(),
    ))
}

fn normalize_category_name(value: &str) -> Result<String, RepositoryError> {
    normalize_short_text(Some(value), "name", 80)?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            RepositoryError::InvalidDependencyGraphQuery(
                "discussion category name is required".to_owned(),
            )
        })
}

fn normalize_category_emoji(value: Option<&str>) -> Result<String, RepositoryError> {
    let emoji = normalize_short_text(value, "emoji", 16)?.unwrap_or_else(|| "💬".to_owned());
    if emoji.trim().is_empty() {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "discussion category emoji is required".to_owned(),
        ));
    }
    Ok(emoji)
}

fn normalize_category_description(value: Option<&str>) -> Result<Option<String>, RepositoryError> {
    normalize_short_text(value, "description", 280)
}

fn normalize_category_section_name(value: &str) -> Result<String, RepositoryError> {
    normalize_short_text(Some(value), "section name", 80)?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            RepositoryError::InvalidDependencyGraphQuery(
                "discussion category section name is required".to_owned(),
            )
        })
}

async fn ensure_section_name_unique(
    pool: &PgPool,
    repository_id: Uuid,
    except_section_id: Option<Uuid>,
    name: &str,
) -> Result<(), RepositoryError> {
    let conflict: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM discussion_category_sections
            WHERE repository_id = $1
              AND ($2::uuid IS NULL OR id <> $2)
              AND lower(name) = lower($3)
        )
        "#,
    )
    .bind(repository_id)
    .bind(except_section_id)
    .bind(name)
    .fetch_one(pool)
    .await?;
    if conflict {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "discussion category section names must be unique within the repository".to_owned(),
        ));
    }
    Ok(())
}

async fn record_category_admin_audit(
    pool: &PgPool,
    actor_user_id: Uuid,
    repository_id: Uuid,
    event_type: &str,
    category_id: Uuid,
    metadata: Value,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, $2, 'repository_discussion_category', $3, $4::jsonb)
        "#,
    )
    .bind(actor_user_id)
    .bind(event_type)
    .bind(category_id.to_string())
    .bind(metadata.to_string())
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO discussion_activity_events (discussion_id, actor_user_id, event_type, payload)
        SELECT id, $2, $3, jsonb_build_object('categoryId', $4::text)
        FROM discussions
        WHERE repository_id = $1
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(repository_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(category_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn record_discussion_settings_audit(
    pool: &PgPool,
    actor_user_id: Uuid,
    _repository_id: Uuid,
    event_type: &str,
    target_type: &str,
    target_id: Uuid,
    metadata: Value,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, $2, $3, $4, $5::jsonb)
        "#,
    )
    .bind(actor_user_id)
    .bind(event_type)
    .bind(target_type)
    .bind(target_id.to_string())
    .bind(metadata.to_string())
    .execute(pool)
    .await?;
    Ok(())
}

async fn load_discussion_form_definition(
    pool: &PgPool,
    repository_id: Uuid,
    category: &DiscussionCategoryChoice,
) -> Result<DiscussionFormDefinition, RepositoryError> {
    if category.is_poll {
        return Ok(DiscussionFormDefinition {
            category_slug: Some(category.slug.clone()),
            template_path: None,
            title: "Start a poll".to_owned(),
            description: category.description.clone(),
            body: String::new(),
            fields: Vec::new(),
            valid: true,
            fallback: false,
            parse_error: None,
        });
    }

    if let Some(row) = sqlx::query(
        r#"
        SELECT template_path, title, description, body, fields::text, valid, parse_error
        FROM discussion_category_forms
        WHERE repository_id = $1 AND category_id = $2
        "#,
    )
    .bind(repository_id)
    .bind(category.id)
    .fetch_optional(pool)
    .await?
    {
        let fields_json: String = row.try_get("fields")?;
        let fields: Vec<DiscussionFormField> =
            serde_json::from_str(&fields_json).unwrap_or_default();
        let valid: bool = row.try_get("valid")?;
        return Ok(DiscussionFormDefinition {
            category_slug: Some(category.slug.clone()),
            template_path: row.try_get("template_path")?,
            title: row
                .try_get::<Option<String>, _>("title")?
                .unwrap_or_else(|| format!("Start a {} discussion", category.name)),
            description: row.try_get("description")?,
            body: row.try_get("body")?,
            fields: if valid { fields } else { Vec::new() },
            valid,
            fallback: !valid,
            parse_error: row.try_get("parse_error")?,
        });
    }

    if let Some((path, content)) =
        load_discussion_template_from_git(pool, repository_id, &category.slug).await?
    {
        let mut parsed = parse_discussion_template(&content, &category.slug, &path);
        let fields_value = serde_json::to_value(&parsed.fields).unwrap_or(Value::Array(Vec::new()));
        sqlx::query(
            r#"
            INSERT INTO discussion_category_forms (
                repository_id, category_id, template_path, title, description, body,
                fields, valid, parse_error, content_sha
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, $8, $9, encode(sha256($10::bytea), 'hex'))
            ON CONFLICT (repository_id, category_id)
            DO UPDATE SET
                template_path = EXCLUDED.template_path,
                title = EXCLUDED.title,
                description = EXCLUDED.description,
                body = EXCLUDED.body,
                fields = EXCLUDED.fields,
                valid = EXCLUDED.valid,
                parse_error = EXCLUDED.parse_error,
                content_sha = EXCLUDED.content_sha,
                parsed_at = now(),
                updated_at = now()
            "#,
        )
        .bind(repository_id)
        .bind(category.id)
        .bind(&path)
        .bind(Some(parsed.title.clone()))
        .bind(parsed.description.clone())
        .bind(&parsed.body)
        .bind(fields_value.to_string())
        .bind(parsed.valid)
        .bind(parsed.parse_error.clone())
        .bind(content.as_bytes())
        .execute(pool)
        .await?;
        parsed.category_slug = Some(category.slug.clone());
        return Ok(parsed);
    }

    Ok(generic_discussion_form(Some(category.slug.clone())))
}

async fn load_discussion_template_from_git(
    pool: &PgPool,
    repository_id: Uuid,
    category_slug: &str,
) -> Result<Option<(String, String)>, RepositoryError> {
    let candidates = [
        format!(".github/DISCUSSION_TEMPLATE/{category_slug}.yml"),
        format!(".github/DISCUSSION_TEMPLATE/{category_slug}.yaml"),
    ];
    let row = sqlx::query(
        r#"
        SELECT repository_files.path, repository_files.content
        FROM repository_files
        JOIN repository_git_refs ON repository_git_refs.target_commit_id = repository_files.commit_id
        JOIN repositories ON repositories.id = repository_files.repository_id
        WHERE repository_files.repository_id = $1
          AND repository_git_refs.name = 'refs/heads/' || repositories.default_branch
          AND lower(repository_files.path) = ANY($2)
        ORDER BY repository_files.path ASC
        LIMIT 1
        "#,
    )
    .bind(repository_id)
    .bind(candidates.map(|candidate| candidate.to_lowercase()).to_vec())
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| (row.get("path"), row.get("content"))))
}

async fn current_discussion_template_file(
    pool: &PgPool,
    repository_id: Uuid,
    path: &str,
) -> Result<Option<RepositorySnapshotFile>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT repository_files.path, repository_files.content,
               repository_files.oid, repository_files.byte_size
        FROM repository_files
        JOIN repository_git_refs ON repository_git_refs.target_commit_id = repository_files.commit_id
        JOIN repositories ON repositories.id = repository_files.repository_id
        WHERE repository_files.repository_id = $1
          AND repository_git_refs.name = 'refs/heads/' || repositories.default_branch
          AND lower(repository_files.path) = lower($2)
        LIMIT 1
        "#,
    )
    .bind(repository_id)
    .bind(path)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| RepositorySnapshotFile {
        path: row.get("path"),
        content: row.get("content"),
        oid: row.get("oid"),
        byte_size: row.get("byte_size"),
    }))
}

async fn current_branch_commit(
    pool: &PgPool,
    repository_id: Uuid,
    branch: &str,
) -> Result<Option<CurrentBranchCommit>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT commits.id, commits.oid
        FROM repository_git_refs
        JOIN commits ON commits.id = repository_git_refs.target_commit_id
        WHERE repository_git_refs.repository_id = $1
          AND repository_git_refs.name = $2
          AND repository_git_refs.kind = 'branch'
        "#,
    )
    .bind(repository_id)
    .bind(format!("refs/heads/{branch}"))
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

async fn write_discussion_template_snapshot(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    branch: &str,
    path: &str,
    content: &str,
    commit_message: &str,
) -> Result<super::repositories::Commit, RepositoryError> {
    let current_ref = current_branch_commit(pool, repository.id, branch).await?;
    let mut files =
        current_branch_files(pool, repository.id, current_ref.as_ref().map(|r| r.id)).await?;
    let blob_oid = deterministic_content_oid("blob", content);
    if let Some(file) = files
        .iter_mut()
        .find(|file| file.path.eq_ignore_ascii_case(path))
    {
        file.path = path.to_owned();
        file.content = content.to_owned();
        file.oid = blob_oid.clone();
        file.byte_size = content.len() as i64;
    } else {
        files.push(RepositorySnapshotFile {
            path: path.to_owned(),
            content: content.to_owned(),
            oid: blob_oid,
            byte_size: content.len() as i64,
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
            repository.id,
            branch,
            tree_oid,
            commit_message,
            content_sha(content)
        ),
    );
    replace_repository_snapshot(
        pool,
        repository.id,
        RepositorySnapshot {
            commit: CreateCommit {
                oid: commit_oid,
                author_user_id: Some(actor_user_id),
                committer_user_id: Some(actor_user_id),
                message: commit_message.to_owned(),
                tree_oid: Some(tree_oid),
                parent_oids,
                committed_at: Utc::now(),
            },
            branch_name: branch.to_owned(),
            files,
        },
    )
    .await
}

async fn cache_discussion_template_form(
    pool: &PgPool,
    repository_id: Uuid,
    category_id: Uuid,
    path: &str,
    content: &str,
    parsed: &DiscussionFormDefinition,
) -> Result<(), RepositoryError> {
    let fields_value = serde_json::to_value(&parsed.fields).unwrap_or(Value::Array(Vec::new()));
    sqlx::query(
        r#"
        INSERT INTO discussion_category_forms (
            repository_id, category_id, template_path, title, description, body,
            fields, valid, parse_error, content_sha
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7::jsonb, $8, $9, encode(sha256($10::bytea), 'hex'))
        ON CONFLICT (repository_id, category_id)
        DO UPDATE SET
            template_path = EXCLUDED.template_path,
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            body = EXCLUDED.body,
            fields = EXCLUDED.fields,
            valid = EXCLUDED.valid,
            parse_error = EXCLUDED.parse_error,
            content_sha = EXCLUDED.content_sha,
            parsed_at = now(),
            updated_at = now()
        "#,
    )
    .bind(repository_id)
    .bind(category_id)
    .bind(path)
    .bind(Some(parsed.title.clone()))
    .bind(parsed.description.clone())
    .bind(&parsed.body)
    .bind(fields_value.to_string())
    .bind(parsed.valid)
    .bind(parsed.parse_error.clone())
    .bind(content.as_bytes())
    .execute(pool)
    .await?;
    Ok(())
}

fn parse_discussion_template(
    content: &str,
    category_slug: &str,
    path: &str,
) -> DiscussionFormDefinition {
    match serde_yaml::from_str::<Value>(content) {
        Ok(value) => {
            let title = yaml_string(&value, "name")
                .or_else(|| yaml_string(&value, "title"))
                .unwrap_or_else(|| "Start a discussion".to_owned());
            let description = yaml_string(&value, "description").map(sanitize_template_text);
            let body = yaml_string(&value, "body").unwrap_or_default();
            let fields = yaml_fields(&value);
            DiscussionFormDefinition {
                category_slug: Some(category_slug.to_owned()),
                template_path: Some(path.to_owned()),
                title: sanitize_template_text(title),
                description,
                body: sanitize_template_text(body),
                fields,
                valid: true,
                fallback: false,
                parse_error: None,
            }
        }
        Err(error) => DiscussionFormDefinition {
            category_slug: Some(category_slug.to_owned()),
            template_path: Some(path.to_owned()),
            title: "Start a discussion".to_owned(),
            description: Some(
                "This category template could not be loaded, so the generic composer will be used."
                    .to_owned(),
            ),
            body: String::new(),
            fields: Vec::new(),
            valid: false,
            fallback: true,
            parse_error: Some(sanitize_template_text(error.to_string())),
        },
    }
}

fn yaml_fields(value: &Value) -> Vec<DiscussionFormField> {
    value
        .get("body")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(20)
        .filter_map(|item| {
            let field_type = item.get("type").and_then(Value::as_str)?.to_owned();
            let attributes = item.get("attributes").unwrap_or(item);
            let label = attributes.get("label").and_then(Value::as_str)?.trim();
            if label.is_empty() {
                return None;
            }
            let supported = matches!(
                field_type.as_str(),
                "input" | "textarea" | "dropdown" | "checkboxes"
            );
            if !supported {
                return None;
            }
            let id = item
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| slugify(label));
            let required = item
                .get("validations")
                .and_then(|v| v.get("required"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let options = attributes
                .get("options")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .take(20)
                .map(sanitize_template_text)
                .collect();
            Some(DiscussionFormField {
                id: slugify(&id),
                field_type,
                label: sanitize_template_text(label),
                description: attributes
                    .get("description")
                    .and_then(Value::as_str)
                    .map(sanitize_template_text),
                placeholder: attributes
                    .get("placeholder")
                    .and_then(Value::as_str)
                    .map(sanitize_template_text),
                required,
                options,
            })
        })
        .collect()
}

fn yaml_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn generic_discussion_form(category_slug: Option<String>) -> DiscussionFormDefinition {
    DiscussionFormDefinition {
        category_slug,
        template_path: None,
        title: "Start a discussion".to_owned(),
        description: None,
        body: String::new(),
        fields: Vec::new(),
        valid: true,
        fallback: false,
        parse_error: None,
    }
}

async fn load_discussion_categories(
    pool: &PgPool,
    repository: &super::repositories::Repository,
    selected_slug: Option<&str>,
) -> Result<Vec<DiscussionCategorySummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT discussion_categories.id, discussion_categories.slug, discussion_categories.name,
               discussion_categories.emoji, discussion_categories.description,
               COUNT(discussions.id)::bigint AS count,
               COUNT(discussions.id) FILTER (WHERE discussions.state = 'open')::bigint AS open_count
        FROM discussion_categories
        LEFT JOIN discussions ON discussions.category_id = discussion_categories.id
        WHERE discussion_categories.repository_id = $1
        GROUP BY discussion_categories.id
        ORDER BY discussion_categories.position ASC, discussion_categories.name ASC
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let slug: String = row.try_get("slug")?;
            Ok(DiscussionCategorySummary {
                id: row.try_get("id")?,
                href: format!(
                    "/{}/{}/discussions/categories/{}",
                    repository.owner_login, repository.name, slug
                ),
                active: selected_slug == Some(slug.as_str()),
                slug,
                name: row.try_get("name")?,
                emoji: row.try_get("emoji")?,
                description: row.try_get("description")?,
                count: row.try_get("count")?,
                open_count: row.try_get("open_count")?,
            })
        })
        .collect()
}

async fn load_discussion_labels(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<DiscussionLabelSummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT labels.id, labels.name, labels.color, labels.description,
               COUNT(discussion_labels.discussion_id)::bigint AS count
        FROM labels
        LEFT JOIN discussion_labels ON discussion_labels.label_id = labels.id
        WHERE labels.repository_id = $1
        GROUP BY labels.id
        ORDER BY labels.name ASC
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(DiscussionLabelSummary {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                color: row.try_get("color")?,
                description: row.try_get("description")?,
                count: row.try_get("count")?,
            })
        })
        .collect()
}

async fn load_discussion_labels_for_discussion(
    pool: &PgPool,
    discussion_id: Uuid,
) -> Result<Vec<DiscussionLabelSummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT labels.id, labels.name, labels.color, labels.description
        FROM discussion_labels
        JOIN labels ON labels.id = discussion_labels.label_id
        WHERE discussion_labels.discussion_id = $1
        ORDER BY labels.name ASC
        "#,
    )
    .bind(discussion_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(DiscussionLabelSummary {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                color: row.try_get("color")?,
                description: row.try_get("description")?,
                count: 0,
            })
        })
        .collect()
}

async fn load_discussion_detail_comments(
    pool: &PgPool,
    repository: &super::repositories::Repository,
    discussion_id: Uuid,
    answer_comment_id: Option<Uuid>,
    actor_user_id: Uuid,
    query: &NormalizedDiscussionDetailQuery,
) -> Result<Vec<DiscussionCommentView>, RepositoryError> {
    let order = match query.sort.as_str() {
        "newest" => "discussion_comments.created_at DESC",
        "top" => "reaction_count DESC, discussion_comments.created_at ASC",
        _ => "discussion_comments.created_at ASC",
    };
    let rows = sqlx::query(&format!(
        r#"
        SELECT discussion_comments.id, discussion_comments.body, discussion_comments.created_at,
               discussion_comments.updated_at, discussion_comments.edited_at,
               discussion_comments.deleted_at, discussion_comments.deleted_reason,
               author.id AS author_id,
               COALESCE(NULLIF(author.username, ''), author.email, 'ghost') AS author_login,
               author.display_name AS author_display_name,
               author.avatar_url AS author_avatar_url,
               COUNT(discussion_reactions.id)::bigint AS reaction_count
        FROM discussion_comments
        LEFT JOIN users author ON author.id = discussion_comments.author_user_id
        LEFT JOIN discussion_reactions ON discussion_reactions.comment_id = discussion_comments.id
        WHERE discussion_comments.discussion_id = $1
          AND discussion_comments.parent_comment_id IS NULL
        GROUP BY discussion_comments.id, author.id
        ORDER BY {order}
        OFFSET $2 LIMIT $3
        "#
    ))
    .bind(discussion_id)
    .bind((query.page - 1) * query.page_size)
    .bind(query.page_size)
    .fetch_all(pool)
    .await?;

    let mut comments = Vec::new();
    for row in rows {
        let comment_id: Uuid = row.try_get("id")?;
        let replies =
            load_discussion_replies(pool, repository, discussion_id, comment_id, actor_user_id)
                .await?;
        comments.push(DiscussionCommentView {
            id: comment_id,
            author: author_from_row(&row)?,
            body: discussion_body_view(row.try_get("body")?),
            reactions: load_discussion_reactions(
                pool,
                discussion_id,
                Some(comment_id),
                actor_user_id,
            )
            .await?,
            replies,
            answer: answer_comment_id == Some(comment_id),
            href: format!(
                "/{}/{}/discussions/{}#discussioncomment-{}",
                repository.owner_login,
                repository.name,
                discussion_number_for_comment(pool, discussion_id).await?,
                comment_id
            ),
            edited: row
                .try_get::<Option<DateTime<Utc>>, _>("edited_at")?
                .is_some(),
            deleted: row
                .try_get::<Option<DateTime<Utc>>, _>("deleted_at")?
                .is_some(),
            deleted_reason: row.try_get("deleted_reason")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        });
    }
    Ok(comments)
}

async fn discussion_number_for_comment(
    pool: &PgPool,
    discussion_id: Uuid,
) -> Result<i64, RepositoryError> {
    Ok(
        sqlx::query_scalar("SELECT number FROM discussions WHERE id = $1")
            .bind(discussion_id)
            .fetch_one(pool)
            .await?,
    )
}

async fn load_discussion_replies(
    pool: &PgPool,
    repository: &super::repositories::Repository,
    discussion_id: Uuid,
    parent_comment_id: Uuid,
    actor_user_id: Uuid,
) -> Result<Vec<DiscussionReplyView>, RepositoryError> {
    let number = discussion_number_for_comment(pool, discussion_id).await?;
    let rows = sqlx::query(
        r#"
        SELECT discussion_comments.id, discussion_comments.body, discussion_comments.created_at,
               discussion_comments.updated_at, discussion_comments.edited_at,
               discussion_comments.deleted_at, discussion_comments.deleted_reason,
               author.id AS author_id,
               COALESCE(NULLIF(author.username, ''), author.email, 'ghost') AS author_login,
               author.display_name AS author_display_name,
               author.avatar_url AS author_avatar_url
        FROM discussion_comments
        LEFT JOIN users author ON author.id = discussion_comments.author_user_id
        WHERE discussion_comments.discussion_id = $1
          AND discussion_comments.parent_comment_id = $2
        ORDER BY discussion_comments.created_at ASC
        LIMIT 100
        "#,
    )
    .bind(discussion_id)
    .bind(parent_comment_id)
    .fetch_all(pool)
    .await?;
    let mut replies = Vec::new();
    for row in rows {
        let reply_id: Uuid = row.try_get("id")?;
        replies.push(DiscussionReplyView {
            id: reply_id,
            author: author_from_row(&row)?,
            body: discussion_body_view(row.try_get("body")?),
            reactions: load_discussion_reactions(
                pool,
                discussion_id,
                Some(reply_id),
                actor_user_id,
            )
            .await?,
            href: format!(
                "/{}/{}/discussions/{}#discussioncomment-{}",
                repository.owner_login, repository.name, number, reply_id
            ),
            edited: row
                .try_get::<Option<DateTime<Utc>>, _>("edited_at")?
                .is_some(),
            deleted: row
                .try_get::<Option<DateTime<Utc>>, _>("deleted_at")?
                .is_some(),
            deleted_reason: row.try_get("deleted_reason")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        });
    }
    Ok(replies)
}

async fn load_discussion_reactions(
    pool: &PgPool,
    discussion_id: Uuid,
    comment_id: Option<Uuid>,
    actor_user_id: Uuid,
) -> Result<Vec<DiscussionReactionSummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT content, COUNT(*)::bigint AS count,
               BOOL_OR(user_id = $3) AS viewer_reacted
        FROM discussion_reactions
        WHERE discussion_id = $1
          AND (
            ($2::uuid IS NULL AND comment_id IS NULL)
            OR comment_id = $2
          )
        GROUP BY content
        ORDER BY content ASC
        "#,
    )
    .bind(discussion_id)
    .bind(comment_id)
    .bind(actor_user_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(DiscussionReactionSummary {
                content: row.try_get("content")?,
                count: row.try_get("count")?,
                viewer_reacted: row.try_get("viewer_reacted")?,
            })
        })
        .collect()
}

async fn load_discussion_detail_events(
    pool: &PgPool,
    discussion_id: Uuid,
) -> Result<Vec<DiscussionEventView>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT discussion_activity_events.id, discussion_activity_events.event_type,
               discussion_activity_events.payload, discussion_activity_events.created_at,
               actor.id AS author_id,
               COALESCE(NULLIF(actor.username, ''), actor.email, 'ghost') AS author_login,
               actor.display_name AS author_display_name,
               actor.avatar_url AS author_avatar_url
        FROM discussion_activity_events
        LEFT JOIN users actor ON actor.id = discussion_activity_events.actor_user_id
        WHERE discussion_activity_events.discussion_id = $1
        ORDER BY discussion_activity_events.created_at ASC
        LIMIT 100
        "#,
    )
    .bind(discussion_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(DiscussionEventView {
                id: row.try_get("id")?,
                event_type: row.try_get("event_type")?,
                actor: Some(author_from_row(&row)?),
                payload: row.try_get("payload")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}

async fn load_discussion_participants(
    pool: &PgPool,
    discussion_id: Uuid,
) -> Result<Vec<DiscussionAuthorSummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT users.id, COALESCE(NULLIF(users.username, ''), users.email, 'ghost') AS author_login,
               users.display_name AS author_display_name, users.avatar_url AS author_avatar_url
        FROM users
        JOIN discussion_comments ON discussion_comments.author_user_id = users.id
        WHERE discussion_comments.discussion_id = $1
        ORDER BY author_login ASC
        LIMIT 20
        "#,
    )
    .bind(discussion_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(|row| author_from_row(&row)).collect()
}

async fn count_top_level_discussion_comments(
    pool: &PgPool,
    discussion_id: Uuid,
) -> Result<i64, RepositoryError> {
    Ok(sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM discussion_comments WHERE discussion_id = $1 AND parent_comment_id IS NULL",
    )
    .bind(discussion_id)
    .fetch_one(pool)
    .await?)
}

async fn load_discussion_subscription(
    pool: &PgPool,
    discussion_id: Uuid,
    actor_user_id: Uuid,
) -> Result<DiscussionSubscriptionState, RepositoryError> {
    let row = sqlx::query(
        "SELECT state, reason FROM discussion_subscriptions WHERE discussion_id = $1 AND user_id = $2",
    )
    .bind(discussion_id)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?;
    let (state, reason) = match row {
        Some(row) => (
            row.try_get::<String, _>("state")?,
            row.try_get::<Option<String>, _>("reason")?,
        ),
        None => ("unsubscribed".to_owned(), None),
    };
    Ok(DiscussionSubscriptionState {
        subscribed: state == "subscribed",
        state,
        reason,
        can_change: true,
    })
}

async fn load_discussion_form_answer_views(
    pool: &PgPool,
    discussion_id: Uuid,
) -> Result<Vec<DiscussionFormAnswerView>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT field_id, field_label, value
        FROM discussion_form_answers
        WHERE discussion_id = $1
        ORDER BY created_at ASC, field_label ASC
        "#,
    )
    .bind(discussion_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(DiscussionFormAnswerView {
                field_id: row.try_get("field_id")?,
                field_label: row.try_get("field_label")?,
                value: row.try_get("value")?,
            })
        })
        .collect()
}

async fn load_discussion_poll_view(
    pool: &PgPool,
    discussion_id: Uuid,
) -> Result<Option<DiscussionPollView>, RepositoryError> {
    let Some(row) = sqlx::query(
        "SELECT id, question, allows_multiple FROM discussion_polls WHERE discussion_id = $1",
    )
    .bind(discussion_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };
    let poll_id: Uuid = row.try_get("id")?;
    let option_rows = sqlx::query(
        "SELECT id, position, label FROM discussion_poll_options WHERE poll_id = $1 ORDER BY position ASC",
    )
    .bind(poll_id)
    .fetch_all(pool)
    .await?;
    let options = option_rows
        .into_iter()
        .map(|row| {
            Ok(DiscussionPollOptionView {
                id: row.try_get("id")?,
                position: row.try_get("position")?,
                label: row.try_get("label")?,
            })
        })
        .collect::<Result<Vec<_>, RepositoryError>>()?;
    Ok(Some(DiscussionPollView {
        id: poll_id,
        question: row.try_get("question")?,
        allows_multiple: row.try_get("allows_multiple")?,
        options,
    }))
}

async fn load_discussion_answer_summary(
    pool: &PgPool,
    repository: &super::repositories::Repository,
    discussion_id: Uuid,
    answer_comment_id: Option<Uuid>,
    _actor_user_id: Uuid,
) -> Result<Option<DiscussionAnswerSummary>, RepositoryError> {
    let Some(comment_id) = answer_comment_id else {
        return Ok(None);
    };
    let number = discussion_number_for_comment(pool, discussion_id).await?;
    let row = sqlx::query(
        r#"
        SELECT COALESCE(discussion_answers.marked_at, discussion_comments.updated_at) AS marked_at,
               marker.id AS author_id,
               COALESCE(NULLIF(marker.username, ''), marker.email, 'ghost') AS author_login,
               marker.display_name AS author_display_name,
               marker.avatar_url AS author_avatar_url
        FROM discussion_comments
        LEFT JOIN discussion_answers ON discussion_answers.comment_id = discussion_comments.id
        LEFT JOIN users marker ON marker.id = discussion_answers.marked_by_user_id
        WHERE discussion_comments.discussion_id = $1 AND discussion_comments.id = $2
        "#,
    )
    .bind(discussion_id)
    .bind(comment_id)
    .fetch_optional(pool)
    .await?;
    row.map(|row| {
        Ok(DiscussionAnswerSummary {
            comment_id,
            marked_by: if row.try_get::<Option<Uuid>, _>("author_id")?.is_some() {
                Some(author_from_row(&row)?)
            } else {
                None
            },
            marked_at: row.try_get("marked_at")?,
            href: format!(
                "/{}/{}/discussions/{}#discussioncomment-{}",
                repository.owner_login, repository.name, number, comment_id
            ),
        })
    })
    .transpose()
}

fn merge_discussion_timeline(
    comments: Vec<DiscussionCommentView>,
    events: Vec<DiscussionEventView>,
) -> Vec<DiscussionTimelineItem> {
    let mut items = Vec::with_capacity(comments.len() + events.len());
    for comment in comments {
        items.push(DiscussionTimelineItem::Comment(comment));
    }
    for event in events {
        items.push(DiscussionTimelineItem::Event(event));
    }
    items
}

fn author_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<DiscussionAuthorSummary, RepositoryError> {
    Ok(DiscussionAuthorSummary {
        id: row.try_get("author_id")?,
        login: row.try_get("author_login")?,
        display_name: row.try_get("author_display_name")?,
        avatar_url: row.try_get("author_avatar_url")?,
    })
}

async fn load_discussion_rows(
    pool: &PgPool,
    repository: &super::repositories::Repository,
    actor_user_id: Uuid,
    category_slug: Option<&str>,
    filters: &NormalizedDiscussionFilters,
) -> Result<Vec<DiscussionRow>, RepositoryError> {
    let sql = filtered_discussion_sql(filters, false);
    let rows = sqlx::query(&sql)
        .bind(repository.id)
        .bind(actor_user_id)
        .bind(category_slug)
        .bind(filters.label.as_deref())
        .bind(filters.query.as_str())
        .bind((filters.page - 1) * filters.page_size)
        .bind(filters.page_size)
        .fetch_all(pool)
        .await?;
    rows.into_iter()
        .map(|row| discussion_row_from_row(row, repository))
        .collect()
}

fn filtered_discussion_sql(filters: &NormalizedDiscussionFilters, count_only: bool) -> String {
    let state_clause = match filters.state {
        DiscussionStateFilter::Open => "AND discussions.state = 'open'",
        DiscussionStateFilter::Closed => "AND discussions.state = 'closed'",
        DiscussionStateFilter::All => "",
    };
    let answered_clause = match filters.answered {
        Some(true) => "AND discussions.answered = true",
        Some(false) => "AND discussions.answered = false",
        None => "",
    };
    let locked_clause = match filters.locked {
        Some(true) => "AND discussions.locked = true",
        Some(false) => "AND discussions.locked = false",
        None => "",
    };
    let pinned_clause = match filters.pinned {
        Some(true) => "AND discussion_pins.discussion_id IS NOT NULL",
        Some(false) => "AND discussion_pins.discussion_id IS NULL",
        None => "",
    };
    let order = match filters.sort.as_str() {
        "newest" => "discussions.created_at DESC",
        "top" => "discussions.votes_count DESC, discussions.last_activity_at DESC",
        "most_commented" => "discussions.comments_count DESC, discussions.last_activity_at DESC",
        _ => "discussions.last_activity_at DESC",
    };
    let select = if count_only {
        "COUNT(DISTINCT discussions.id)::bigint AS total".to_owned()
    } else {
        format!("{DISCUSSION_ROW_SELECT} ORDER BY {order} OFFSET $6 LIMIT $7")
    };
    format!(
        r#"
        SELECT {select}
        FROM discussions
        JOIN discussion_categories ON discussion_categories.id = discussions.category_id
        LEFT JOIN discussion_pins ON discussion_pins.discussion_id = discussions.id
        LEFT JOIN users author ON author.id = discussions.author_user_id
        WHERE discussions.repository_id = $1
          AND ($3::text IS NULL OR discussion_categories.slug = $3)
          AND ($4::text IS NULL OR EXISTS (
              SELECT 1 FROM discussion_labels
              JOIN labels ON labels.id = discussion_labels.label_id
              WHERE discussion_labels.discussion_id = discussions.id
                AND lower(labels.name) = lower($4)
          ))
          AND (
              $5::text = 'is:open'
              OR discussions.title ILIKE '%' || $5 || '%'
              OR discussions.body ILIKE '%' || $5 || '%'
          )
          {state_clause}
          {answered_clause}
          {locked_clause}
          {pinned_clause}
        "#
    )
}

const DISCUSSION_ROW_SELECT: &str = r#"
        discussions.id, discussions.number, discussions.title, discussions.state,
        discussions.answered, discussions.locked, discussions.comments_count,
        discussions.votes_count, discussions.created_at, discussions.updated_at,
        discussions.last_activity_at,
        discussion_categories.id AS category_id, discussion_categories.slug AS category_slug,
        discussion_categories.name AS category_name, discussion_categories.emoji AS category_emoji,
        discussion_categories.description AS category_description,
        discussion_pins.discussion_id IS NOT NULL AS pinned,
        EXISTS (SELECT 1 FROM discussion_votes WHERE discussion_votes.discussion_id = discussions.id AND discussion_votes.user_id = $2) AS viewer_voted,
        author.id AS author_id,
        COALESCE(NULLIF(author.username, ''), author.email, 'ghost') AS author_login,
        author.display_name AS author_display_name,
        author.avatar_url AS author_avatar_url,
        COALESCE((
          SELECT jsonb_agg(jsonb_build_object(
            'id', labels.id,
            'name', labels.name,
            'color', labels.color,
            'description', labels.description,
            'count', 0
          ) ORDER BY labels.name)
          FROM discussion_labels
          JOIN labels ON labels.id = discussion_labels.label_id
          WHERE discussion_labels.discussion_id = discussions.id
        ), '[]'::jsonb)::text AS labels_json
"#;

fn discussion_row_from_row(
    row: sqlx::postgres::PgRow,
    repository: &super::repositories::Repository,
) -> Result<DiscussionRow, RepositoryError> {
    let number: i64 = row.try_get("number")?;
    let labels_json: String = row.try_get("labels_json")?;
    let labels: Vec<DiscussionLabelSummary> =
        serde_json::from_str(&labels_json).map_err(|error| {
            RepositoryError::InvalidDependencyGraphQuery(format!(
                "discussion labels were malformed: {error}"
            ))
        })?;
    Ok(DiscussionRow {
        id: row.try_get("id")?,
        number,
        title: row.try_get("title")?,
        state: row.try_get("state")?,
        answered: row.try_get("answered")?,
        locked: row.try_get("locked")?,
        pinned: row.try_get("pinned")?,
        category: DiscussionCategorySummary {
            id: row.try_get("category_id")?,
            slug: row.try_get("category_slug")?,
            name: row.try_get("category_name")?,
            emoji: row.try_get("category_emoji")?,
            description: row.try_get("category_description")?,
            count: 0,
            open_count: 0,
            href: format!(
                "/{}/{}/discussions/categories/{}",
                repository.owner_login,
                repository.name,
                row.try_get::<String, _>("category_slug")?
            ),
            active: false,
        },
        labels,
        author: DiscussionAuthorSummary {
            id: row.try_get("author_id")?,
            login: row.try_get("author_login")?,
            display_name: row.try_get("author_display_name")?,
            avatar_url: row.try_get("author_avatar_url")?,
        },
        comments_count: row.try_get("comments_count")?,
        votes_count: row.try_get("votes_count")?,
        viewer_voted: row.try_get("viewer_voted")?,
        href: format!(
            "/{}/{}/discussions/{}",
            repository.owner_login, repository.name, number
        ),
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        last_activity_at: row.try_get("last_activity_at")?,
    })
}

async fn count_discussions(
    pool: &PgPool,
    repository_id: Uuid,
    category_slug: Option<&str>,
    filters: &NormalizedDiscussionFilters,
) -> Result<i64, RepositoryError> {
    let sql = filtered_discussion_sql(filters, true);
    let row = sqlx::query(&sql)
        .bind(repository_id)
        .bind(Uuid::nil())
        .bind(category_slug)
        .bind(filters.label.as_deref())
        .bind(filters.query.as_str())
        .fetch_one(pool)
        .await?;
    Ok(row.try_get("total")?)
}

async fn discussion_state_counts(
    pool: &PgPool,
    repository_id: Uuid,
    category_slug: Option<&str>,
) -> Result<(i64, i64), RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT COUNT(*) FILTER (WHERE discussions.state = 'open')::bigint AS open_count,
               COUNT(*) FILTER (WHERE discussions.state = 'closed')::bigint AS closed_count
        FROM discussions
        JOIN discussion_categories ON discussion_categories.id = discussions.category_id
        WHERE discussions.repository_id = $1
          AND ($2::text IS NULL OR discussion_categories.slug = $2)
        "#,
    )
    .bind(repository_id)
    .bind(category_slug)
    .fetch_one(pool)
    .await?;
    Ok((row.try_get("open_count")?, row.try_get("closed_count")?))
}

async fn load_pinned_discussions(
    pool: &PgPool,
    repository: &super::repositories::Repository,
    actor_user_id: Uuid,
    category_slug: Option<&str>,
) -> Result<Vec<PinnedDiscussionCard>, RepositoryError> {
    let rows = sqlx::query(&format!(
        r#"
        SELECT {DISCUSSION_ROW_SELECT}, discussion_pins.position, discussion_pins.created_at AS pinned_at
        FROM discussion_pins
        JOIN discussions ON discussions.id = discussion_pins.discussion_id
        JOIN discussion_categories ON discussion_categories.id = discussions.category_id
        LEFT JOIN users author ON author.id = discussions.author_user_id
        WHERE discussions.repository_id = $1
          AND ($3::text IS NULL OR discussion_categories.slug = $3)
        ORDER BY discussion_pins.position ASC, discussion_pins.created_at DESC
        LIMIT 6
        "#
    ))
    .bind(repository.id)
    .bind(actor_user_id)
    .bind(category_slug)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let position = row.try_get("position")?;
            let pinned_at = row.try_get("pinned_at")?;
            Ok(PinnedDiscussionCard {
                discussion: discussion_row_from_row(row, repository)?,
                position,
                pinned_at,
            })
        })
        .collect()
}

async fn load_helpful_contributors(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<HelpfulContributorSummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT users.id, COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.display_name, users.avatar_url,
               COUNT(discussion_comments.id)::bigint AS comments_count,
               COUNT(discussions.id) FILTER (WHERE discussions.answer_comment_id = discussion_comments.id)::bigint AS helpful_count
        FROM discussion_comments
        JOIN discussions ON discussions.id = discussion_comments.discussion_id
        JOIN users ON users.id = discussion_comments.author_user_id
        WHERE discussions.repository_id = $1
          AND discussion_comments.created_at >= now() - interval '30 days'
        GROUP BY users.id
        ORDER BY helpful_count DESC, comments_count DESC, login ASC
        LIMIT 5
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(HelpfulContributorSummary {
                user: DiscussionAuthorSummary {
                    id: row.try_get("id")?,
                    login: row.try_get("login")?,
                    display_name: row.try_get("display_name")?,
                    avatar_url: row.try_get("avatar_url")?,
                },
                comments_count: row.try_get("comments_count")?,
                helpful_count: row.try_get("helpful_count")?,
            })
        })
        .collect()
}

async fn load_community_links(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<CommunityLinkSummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT id, label, href, kind
        FROM repository_community_links
        WHERE repository_id = $1
        ORDER BY position ASC, label ASC
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(CommunityLinkSummary {
                id: row.try_get("id")?,
                label: row.try_get("label")?,
                href: row.try_get("href")?,
                kind: row.try_get("kind")?,
            })
        })
        .collect()
}
