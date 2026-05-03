use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, patch},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    api_types::{
        database_unavailable, error_response, error_response_with_details, normalize_pagination,
        ErrorEnvelope, RestJson,
    },
    auth::extractor::AuthenticatedUser,
    domain::{
        identity::User,
        issues::{
            add_issue_comment, create_issue, get_issue, issue_comment_timeline_item,
            issue_timeline_view, list_issue_templates_for_viewer,
            repository_issue_detail_view_for_viewer, repository_issue_list_view_for_viewer,
            save_repository_issue_preferences, toggle_issue_reaction, update_issue_metadata,
            update_issue_state, update_issue_subscription, CollaborationError, CreateComment,
            CreateIssue, CreateIssueAttachment, IssueListQuery, IssueState, ReactionContent,
            UpdateIssueMetadata, UpdateIssueState, UpdateIssueSubscription,
        },
        permissions::RepositoryRole,
        pulls::repository_for_actor_by_name,
        repositories::{get_repository_by_owner_name, RepositoryError},
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/repos/:owner/:repo/issues", get(list).post(create))
        .route("/api/repos/:owner/:repo/issues/templates", get(templates))
        .route(
            "/api/repos/:owner/:repo/issues/preferences",
            patch(update_preferences),
        )
        .route(
            "/api/repos/:owner/:repo/issues/:number",
            get(read).patch(update_state),
        )
        .route(
            "/api/repos/:owner/:repo/issues/:number/comments",
            get(timeline).post(comment),
        )
        .route(
            "/api/repos/:owner/:repo/issues/:number/timeline",
            get(timeline),
        )
        .route(
            "/api/repos/:owner/:repo/issues/:number/reactions",
            post_reaction_route(),
        )
        .route(
            "/api/repos/:owner/:repo/issues/:number/subscription",
            patch(update_subscription),
        )
        .route(
            "/api/repos/:owner/:repo/issues/:number/metadata",
            patch(update_metadata),
        )
}

fn post_reaction_route() -> axum::routing::MethodRouter<AppState> {
    axum::routing::post(reaction)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQuery {
    state: Option<IssueState>,
    q: Option<String>,
    author: Option<String>,
    #[serde(alias = "excluded_author", alias = "excludedAuthor")]
    excluded_author: Option<String>,
    labels: Option<String>,
    #[serde(alias = "excluded_labels")]
    excluded_labels: Option<String>,
    #[serde(alias = "no_labels", alias = "noLabels")]
    no_labels: Option<bool>,
    milestone: Option<String>,
    #[serde(alias = "no_milestone", alias = "noMilestone")]
    no_milestone: Option<bool>,
    assignee: Option<String>,
    #[serde(alias = "no_assignee", alias = "noAssignee")]
    no_assignee: Option<bool>,
    project: Option<String>,
    #[serde(alias = "type")]
    issue_type: Option<String>,
    sort: Option<String>,
    order: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateIssueRequest {
    title: String,
    body: Option<String>,
    template_id: Option<Uuid>,
    template_slug: Option<String>,
    field_values: Option<std::collections::HashMap<String, String>>,
    milestone_id: Option<Uuid>,
    label_ids: Option<Vec<Uuid>>,
    assignee_user_ids: Option<Vec<Uuid>>,
    attachments: Option<Vec<CreateIssueAttachmentRequest>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateIssueAttachmentRequest {
    file_name: String,
    byte_size: i64,
    content_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateIssueStateRequest {
    state: IssueState,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateCommentRequest {
    body: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactionRequest {
    content: ReactionContent,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateIssueSubscriptionRequest {
    subscribed: bool,
    #[serde(default)]
    custom_events: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateIssueMetadataRequest {
    label_ids: Option<Vec<Uuid>>,
    assignee_user_ids: Option<Vec<Uuid>>,
    milestone_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateIssuePreferencesRequest {
    dismissed_contributor_banner: bool,
}

async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repository = get_repository_by_owner_name(pool, &owner, &repo)
        .await
        .map_err(repository_lookup_error)?
        .ok_or_else(|| map_collaboration_error(CollaborationError::RepositoryNotFound))?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let envelope = repository_issue_list_view_for_viewer(
        pool,
        repository.id,
        actor.as_ref().map(|user| user.id),
        issue_list_query(&query, actor.as_ref()).map_err(map_collaboration_error)?,
        pagination.page,
        pagination.page_size,
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(envelope)))
}

const ISSUE_SORTS: &[&str] = &[
    "updated-desc",
    "updated-asc",
    "created-desc",
    "created-asc",
    "comments-desc",
    "comments-asc",
    "best-match",
];

fn issue_list_query(
    query: &ListQuery,
    actor: Option<&User>,
) -> Result<IssueListQuery, CollaborationError> {
    let mut filters = IssueListQuery::default();
    let q = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("is:issue state:open");

    validate_issue_query(q)?;
    filters.query = Some(q.chars().take(240).collect());
    filters.state = query.state.clone().unwrap_or_else(|| state_from_query(q));
    filters.author = query
        .author
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| normalize_user_filter(value, actor))
        .or_else(|| {
            qualifier_from_query(q, "author:")
                .as_deref()
                .map(|value| normalize_user_filter(value, actor))
        });
    filters.excluded_author = query
        .excluded_author
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| normalize_user_filter(value, actor))
        .or_else(|| {
            qualifier_from_query(q, "-author:")
                .as_deref()
                .map(|value| normalize_user_filter(value, actor))
        });
    filters.labels = labels_from_query(q, query.labels.as_deref());
    filters.excluded_labels = excluded_labels_from_query(q, query.excluded_labels.as_deref());
    filters.no_labels = query.no_labels.unwrap_or(false) || no_labels_from_query(q);
    filters.milestone = query
        .milestone
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| qualifier_from_query(q, "milestone:"));
    filters.no_milestone =
        query.no_milestone.unwrap_or(false) || no_filter_from_query(q, "milestone");
    filters.assignee = query
        .assignee
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| normalize_user_filter(value, actor))
        .or_else(|| {
            qualifier_from_query(q, "assignee:")
                .as_deref()
                .map(|value| normalize_user_filter(value, actor))
        });
    filters.no_assignee = query.no_assignee.unwrap_or(false) || no_filter_from_query(q, "assignee");
    filters.project = query
        .project
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| qualifier_from_query(q, "project:"));
    filters.issue_type = query
        .issue_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| qualifier_from_query(q, "type:"));
    let raw_sort = query
        .sort
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| qualifier_from_query(q, "sort:"))
        .unwrap_or_else(|| "updated-desc".to_owned());
    let raw_order = query
        .order
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| qualifier_from_query(q, "order:"));
    filters.sort = normalize_issue_sort(&raw_sort, raw_order.as_deref())?;
    Ok(filters)
}

fn normalize_issue_sort(sort: &str, order: Option<&str>) -> Result<String, CollaborationError> {
    let order = order.unwrap_or("desc").to_lowercase();
    if !matches!(order.as_str(), "asc" | "desc") {
        return Err(CollaborationError::InvalidIssueFilter(
            "order must be asc or desc".to_owned(),
        ));
    }

    let normalized = match sort.to_lowercase().as_str() {
        "updated" | "recently-updated" => format!("updated-{order}"),
        "created" | "newest" => format!("created-{order}"),
        "comments" | "commented" | "most-commented" => format!("comments-{order}"),
        "least-commented" => "comments-asc".to_owned(),
        "oldest" => "created-asc".to_owned(),
        "least-recently-updated" => "updated-asc".to_owned(),
        "best" | "best-match" => "best-match".to_owned(),
        value => value.to_owned(),
    };

    if !ISSUE_SORTS.contains(&normalized.as_str()) {
        return Err(CollaborationError::InvalidIssueFilter(
            "sort must be one of updated-desc, updated-asc, created-desc, created-asc, comments-desc, comments-asc, best-match".to_owned(),
        ));
    }

    Ok(normalized)
}

fn validate_issue_query(query: &str) -> Result<(), CollaborationError> {
    for term in query.split_whitespace() {
        if let Some(value) = term.strip_prefix("state:") {
            if !matches!(value, "open" | "closed") {
                return Err(CollaborationError::InvalidIssueFilter(
                    "state filter must be open or closed".to_owned(),
                ));
            }
        }
        if let Some(value) = term.strip_prefix("is:") {
            if !matches!(value, "issue" | "open" | "closed") {
                return Err(CollaborationError::InvalidIssueFilter(
                    "is filter must be issue, open, or closed".to_owned(),
                ));
            }
        }
        if matches!(term, "no:labels" | "no:label") {
            continue;
        }
        for prefix in [
            "label:",
            "-label:",
            "author:",
            "-author:",
            "assignee:",
            "milestone:",
            "project:",
            "type:",
        ] {
            if let Some(value) = term.strip_prefix(prefix) {
                let normalized = value.trim().trim_matches('"');
                if normalized.is_empty() {
                    return Err(CollaborationError::InvalidIssueFilter(format!(
                        "{} filters require a value",
                        prefix.trim_end_matches(':')
                    )));
                }
            }
        }
        if let Some(value) = term.strip_prefix("no:") {
            if !matches!(value, "label" | "labels" | "assignee" | "milestone") {
                return Err(CollaborationError::InvalidIssueFilter(
                    "no filter must be label, assignee, or milestone".to_owned(),
                ));
            }
        }
    }
    Ok(())
}

fn state_from_query(query: &str) -> IssueState {
    if query
        .split_whitespace()
        .any(|term| matches!(term, "state:closed" | "is:closed"))
    {
        IssueState::Closed
    } else {
        IssueState::Open
    }
}

fn labels_from_query(query: &str, explicit_labels: Option<&str>) -> Vec<String> {
    let mut labels = explicit_labels
        .into_iter()
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    labels.extend(
        qualifier_values_from_query(query, "label:")
            .into_iter()
            .filter(|value| !value.is_empty()),
    );
    labels.sort_by_key(|value| value.to_lowercase());
    labels.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    labels
}

fn excluded_labels_from_query(query: &str, explicit_labels: Option<&str>) -> Vec<String> {
    let mut labels = explicit_labels
        .into_iter()
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    labels.extend(
        qualifier_values_from_query(query, "-label:")
            .into_iter()
            .filter(|value| !value.is_empty()),
    );
    labels.sort_by_key(|value| value.to_lowercase());
    labels.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    labels
}

fn no_labels_from_query(query: &str) -> bool {
    no_filter_from_query(query, "label") || no_filter_from_query(query, "labels")
}

fn no_filter_from_query(query: &str, value: &str) -> bool {
    query
        .split_whitespace()
        .any(|term| term.strip_prefix("no:").is_some_and(|term| term == value))
}

fn qualifier_from_query(query: &str, prefix: &str) -> Option<String> {
    qualifier_values_from_query(query, prefix)
        .into_iter()
        .find(|value| !value.is_empty())
}

fn qualifier_values_from_query(query: &str, prefix: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut rest = query;
    while let Some(index) = rest.find(prefix) {
        if prefix == "label:" && index > 0 && rest.as_bytes()[index - 1] == b'-' {
            rest = &rest[index + prefix.len()..];
            continue;
        }
        let after_prefix = &rest[index + prefix.len()..];
        let trimmed = after_prefix.trim_start();
        if let Some(quoted) = trimmed.strip_prefix('"') {
            if let Some(end_quote) = quoted.find('"') {
                values.push(quoted[..end_quote].trim().to_owned());
                rest = &quoted[end_quote + 1..];
            } else {
                values.push(quoted.trim().to_owned());
                break;
            }
        } else {
            let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
            values.push(trimmed[..end].trim().to_owned());
            rest = &trimmed[end..];
        }
    }
    values
}

fn normalize_user_filter(value: &str, actor: Option<&User>) -> String {
    let normalized = value.trim().trim_start_matches('@');
    if normalized.eq_ignore_ascii_case("me") {
        actor
            .map(|user| {
                user.username
                    .as_deref()
                    .unwrap_or(user.email.as_str())
                    .to_owned()
            })
            .unwrap_or_else(|| "@me".to_owned())
    } else {
        normalized.to_owned()
    }
}

async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<CreateIssueRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_collaboration_error)?;
    if request.title.trim().is_empty() {
        return Err(error_response_with_details(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "issue title is required",
            json!({
                "field": "title",
                "reason": "issue title is required",
            }),
        ));
    }
    let issue = create_issue(
        pool,
        CreateIssue {
            repository_id,
            actor_user_id: actor.0.id,
            title: request.title,
            body: request.body,
            template_id: request.template_id,
            template_slug: request.template_slug,
            field_values: request.field_values.unwrap_or_default(),
            milestone_id: request.milestone_id,
            label_ids: request.label_ids.unwrap_or_default(),
            assignee_user_ids: request.assignee_user_ids.unwrap_or_default(),
            attachments: request
                .attachments
                .unwrap_or_default()
                .into_iter()
                .map(|attachment| CreateIssueAttachment {
                    file_name: attachment.file_name,
                    byte_size: attachment.byte_size,
                    content_type: attachment.content_type,
                })
                .collect(),
        },
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok((StatusCode::CREATED, Json(json!(issue))))
}

async fn templates(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repository = get_repository_by_owner_name(pool, &owner, &repo)
        .await
        .map_err(repository_lookup_error)?
        .ok_or_else(|| map_collaboration_error(CollaborationError::RepositoryNotFound))?;
    let templates =
        list_issue_templates_for_viewer(pool, repository.id, actor.as_ref().map(|user| user.id))
            .await
            .map_err(map_collaboration_error)?;

    Ok(Json(json!({ "items": templates })))
}

async fn update_preferences(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<UpdateIssuePreferencesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_collaboration_error)?;
    let preferences = save_repository_issue_preferences(
        pool,
        repository_id,
        actor.0.id,
        request.dismissed_contributor_banner,
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(preferences)))
}

async fn read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repository = get_repository_by_owner_name(pool, &owner, &repo)
        .await
        .map_err(repository_lookup_error)?
        .ok_or_else(|| map_collaboration_error(CollaborationError::RepositoryNotFound))?;
    let issue = repository_issue_detail_view_for_viewer(
        pool,
        repository.id,
        number,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(issue)))
}

async fn update_state(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    RestJson(request): RestJson<UpdateIssueStateRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_collaboration_error)?;
    let issue = get_issue(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    let updated = update_issue_state(
        pool,
        issue.id,
        UpdateIssueState {
            actor_user_id: actor.0.id,
            state: request.state,
        },
    )
    .await
    .map_err(map_collaboration_error)?;
    let detail = repository_issue_detail_view_for_viewer(
        pool,
        repository_id,
        updated.number,
        Some(actor.0.id),
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(detail)))
}

async fn comment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    RestJson(request): RestJson<CreateCommentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_collaboration_error)?;
    let issue = get_issue(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    if request.body.trim().is_empty() {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "comment body is required",
        ));
    }
    let comment = add_issue_comment(
        pool,
        issue.id,
        CreateComment {
            actor_user_id: actor.0.id,
            body: request.body,
        },
    )
    .await
    .map_err(map_collaboration_error)?;
    let item = issue_comment_timeline_item(pool, comment.id)
        .await
        .map_err(map_collaboration_error)?;

    Ok((StatusCode::CREATED, Json(json!(item))))
}

async fn timeline(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository = get_repository_by_owner_name(pool, &owner, &repo)
        .await
        .map_err(repository_lookup_error)?
        .ok_or_else(|| map_collaboration_error(CollaborationError::RepositoryNotFound))?;
    let issue = repository_issue_detail_view_for_viewer(
        pool,
        repository.id,
        number,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_collaboration_error)?;
    let issue_id = issue.id;
    let events = issue_timeline_view(pool, issue_id, actor.as_ref().map(|user| user.id))
        .await
        .map_err(map_collaboration_error)?;

    Ok(Json(json!(events)))
}

async fn reaction(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    RestJson(request): RestJson<ReactionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_collaboration_error)?;
    let issue = get_issue(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    let summaries = toggle_issue_reaction(pool, issue.id, actor.0.id, request.content)
        .await
        .map_err(map_collaboration_error)?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "user_id": actor.0.id,
            "summaries": summaries,
        })),
    ))
}

async fn update_subscription(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    RestJson(request): RestJson<UpdateIssueSubscriptionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_collaboration_error)?;
    let issue = get_issue(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    let subscription = update_issue_subscription(
        pool,
        issue.id,
        UpdateIssueSubscription {
            actor_user_id: actor.0.id,
            subscribed: request.subscribed,
            custom_events: request.custom_events,
        },
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(subscription)))
}

async fn update_metadata(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    RestJson(request): RestJson<UpdateIssueMetadataRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_collaboration_error)?;
    let issue = get_issue(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    update_issue_metadata(
        pool,
        issue.id,
        UpdateIssueMetadata {
            actor_user_id: actor.0.id,
            label_ids: request.label_ids.unwrap_or_default(),
            assignee_user_ids: request.assignee_user_ids.unwrap_or_default(),
            milestone_id: request.milestone_id,
        },
    )
    .await
    .map_err(map_collaboration_error)?;

    let detail =
        repository_issue_detail_view_for_viewer(pool, repository_id, number, Some(actor.0.id))
            .await
            .map_err(map_collaboration_error)?;
    Ok(Json(json!(detail)))
}

pub(crate) fn map_collaboration_error(
    error: CollaborationError,
) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        CollaborationError::RepositoryAccessDenied => error_response(
            StatusCode::FORBIDDEN,
            "forbidden",
            "user does not have repository access".to_owned(),
        ),
        CollaborationError::RepositoryNotFound
        | CollaborationError::IssueNotFound
        | CollaborationError::PullRequestNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        CollaborationError::InvalidState(_)
        | CollaborationError::InvalidReaction(_)
        | CollaborationError::InvalidIssueFilter(_)
        | CollaborationError::InvalidIssueAttachment(_) => {
            let message = error.to_string();
            error_response_with_details(
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_failed",
                message.clone(),
                json!({
                    "field": "q",
                    "reason": message,
                }),
            )
        }
        CollaborationError::InvalidIssueField { field_key, message } => {
            error_response_with_details(
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_failed",
                message.clone(),
                json!({
                    "field": format!("fieldValues.{field_key}"),
                    "fieldKey": field_key,
                    "reason": message,
                }),
            )
        }
        CollaborationError::Sqlx(sqlx::Error::Database(database_error))
            if database_error.is_unique_violation() =>
        {
            error_response(
                StatusCode::CONFLICT,
                "conflict",
                database_error.message().to_owned(),
            )
        }
        CollaborationError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "collaboration operation failed".to_owned(),
        ),
    }
}

fn repository_lookup_error(error: RepositoryError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        RepositoryError::Sqlx(error) => map_collaboration_error(CollaborationError::Sqlx(error)),
        _ => map_collaboration_error(CollaborationError::RepositoryNotFound),
    }
}
