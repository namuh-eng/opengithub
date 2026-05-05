use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::{
    notifications::{create_notification, CreateNotification},
    permissions::RepositoryRole,
    repositories::{
        get_repository_by_owner_name, repository_permission_for_user, RepositoryError,
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
        JOIN discussion_labels ON discussion_labels.label_id = labels.id
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
