use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    api_types::{
        database_unavailable as shared_database_unavailable, normalize_pagination, ErrorEnvelope,
        RestJson,
    },
    auth::extractor::AuthenticatedUser,
    domain::{
        identity::User,
        issues::CreateComment,
        permissions::RepositoryRole,
        pulls::{
            abandon_pull_request_review_draft, add_pull_request_comment,
            compare_pull_request_refs_for_viewer_with_head, create_pull_request,
            create_pull_request_review_draft_comment, delete_pull_request_review_draft_comment,
            get_pull_request, pull_request_comment_timeline_item,
            pull_request_detail_view_for_viewer, pull_request_diff_review_for_viewer,
            pull_request_timeline_view, pull_sort_options, repository_for_actor_by_name,
            repository_pull_request_list_view_for_viewer, save_repository_pull_preferences,
            submit_pull_request_review, update_pull_request_draft_state,
            update_pull_request_metadata, update_pull_request_review_draft_comment,
            update_pull_request_review_requests, update_pull_request_state,
            update_pull_request_subscription, update_pull_request_viewed_file,
            ComparePullRequestRefsInput, CreatePullRequest, CreatePullRequestReviewDraftComment,
            MergeMethod, PullRequestDiffReviewQuery, PullRequestListQuery, PullRequestState,
            SubmitPullRequestReview, UpdatePullRequestDraftState, UpdatePullRequestMetadata,
            UpdatePullRequestReviewDraftComment, UpdatePullRequestReviewRequests,
            UpdatePullRequestState, UpdatePullRequestSubscription,
        },
        repositories::{get_repository_by_owner_name, RepositoryError},
    },
    routes::issues::map_collaboration_error,
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/repos/:owner/:repo/pulls", get(list).post(create))
        .route("/api/repos/:owner/:repo/compare/*range", get(compare))
        .route(
            "/api/repos/:owner/:repo/pulls/preferences",
            patch(update_preferences),
        )
        .route(
            "/api/repos/:owner/:repo/pulls/:number",
            get(read).patch(update_state),
        )
        .route(
            "/api/repos/:owner/:repo/pulls/:number/comments",
            get(timeline).post(comment),
        )
        .route(
            "/api/repos/:owner/:repo/pulls/:number/timeline",
            get(timeline),
        )
        .route("/api/repos/:owner/:repo/pulls/:number/files", get(files))
        .route(
            "/api/repos/:owner/:repo/pulls/:number/files/viewed",
            patch(update_viewed_file),
        )
        .route(
            "/api/repos/:owner/:repo/pulls/:number/review-comments/drafts",
            post(create_review_draft_comment),
        )
        .route(
            "/api/repos/:owner/:repo/pulls/:number/review-comments/drafts/:draft_id",
            patch(update_review_draft_comment).delete(delete_review_draft_comment),
        )
        .route(
            "/api/repos/:owner/:repo/pulls/:number/reviews",
            post(submit_review),
        )
        .route(
            "/api/repos/:owner/:repo/pulls/:number/reviews/draft",
            delete(abandon_review_draft),
        )
        .route(
            "/api/repos/:owner/:repo/pulls/:number/review-requests",
            patch(update_review_requests),
        )
        .route(
            "/api/repos/:owner/:repo/pulls/:number/draft",
            patch(update_draft),
        )
        .route(
            "/api/repos/:owner/:repo/pulls/:number/metadata",
            patch(update_metadata),
        )
        .route(
            "/api/repos/:owner/:repo/pulls/:number/subscription",
            patch(update_subscription),
        )
        .route("/api/repos/:owner/:repo/pulls/:number/merge", post(merge))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQuery {
    state: Option<PullRequestState>,
    q: Option<String>,
    author: Option<String>,
    labels: Option<String>,
    milestone: Option<String>,
    #[serde(alias = "no_milestone", alias = "noMilestone")]
    no_milestone: Option<bool>,
    assignee: Option<String>,
    #[serde(alias = "no_assignee", alias = "noAssignee")]
    no_assignee: Option<bool>,
    project: Option<String>,
    review: Option<String>,
    checks: Option<String>,
    sort: Option<String>,
    order: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompareQuery {
    commits: Option<i64>,
    files: Option<i64>,
    #[serde(alias = "head_owner", alias = "headOwner")]
    head_owner: Option<String>,
    #[serde(alias = "head_repo", alias = "headRepo")]
    head_repo: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FilesQuery {
    view: Option<String>,
    whitespace: Option<String>,
    commit: Option<String>,
    filter: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreatePullRequestRequest {
    title: String,
    body: Option<String>,
    head_ref: String,
    base_ref: String,
    head_repository_id: Option<Uuid>,
    #[serde(alias = "head_owner", alias = "headOwner")]
    head_owner: Option<String>,
    #[serde(alias = "head_repo", alias = "headRepo")]
    head_repo: Option<String>,
    is_draft: Option<bool>,
    label_ids: Option<Vec<Uuid>>,
    milestone_id: Option<Uuid>,
    assignee_user_ids: Option<Vec<Uuid>>,
    reviewer_user_ids: Option<Vec<Uuid>>,
    template_slug: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePullRequestStateRequest {
    state: PullRequestState,
    merge_commit_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MergePullRequestRequest {
    method: Option<MergeMethod>,
    merge_commit_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateCommentRequest {
    body: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePullPreferencesRequest {
    dismissed_contributor_banner: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePullReviewRequestsRequest {
    reviewer_user_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePullDraftRequest {
    is_draft: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePullMetadataRequest {
    label_ids: Option<Vec<Uuid>>,
    assignee_user_ids: Option<Vec<Uuid>>,
    milestone_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePullSubscriptionRequest {
    subscribed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePullViewedFileRequest {
    file_id: Uuid,
    version_key: String,
    viewed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateReviewDraftCommentRequest {
    file_id: Uuid,
    body: String,
    side: String,
    old_line: Option<i64>,
    new_line: Option<i64>,
    position: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateReviewDraftCommentRequest {
    body: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubmitReviewRequest {
    body: Option<String>,
    state: String,
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
        .ok_or_else(|| {
            map_collaboration_error(crate::domain::issues::CollaborationError::RepositoryNotFound)
        })?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let envelope = repository_pull_request_list_view_for_viewer(
        pool,
        repository.id,
        actor.as_ref().map(|user| user.id),
        pull_list_query(&query, actor.as_ref()).map_err(map_collaboration_error)?,
        pagination.page,
        pagination.page_size,
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(envelope)))
}

async fn compare(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, range)): Path<(String, String, String)>,
    Query(query): Query<CompareQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repository = get_repository_by_owner_name(pool, &owner, &repo)
        .await
        .map_err(repository_lookup_error)?
        .ok_or_else(|| {
            map_collaboration_error(crate::domain::issues::CollaborationError::RepositoryNotFound)
        })?;
    let (base, head) = parse_compare_range(&range).map_err(map_collaboration_error)?;
    let head_repository_id =
        resolve_optional_head_repository_id(pool, &query.head_owner, &query.head_repo).await?;
    let view = compare_pull_request_refs_for_viewer_with_head(
        pool,
        ComparePullRequestRefsInput {
            repository_id: repository.id,
            actor_user_id: actor.as_ref().map(|user| user.id),
            base_ref: &base,
            head_ref: &head,
            head_repository_id,
            commit_limit: query.commits.unwrap_or(100),
            file_limit: query.files.unwrap_or(300),
        },
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(view)))
}

async fn files(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    Query(query): Query<FilesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repository = get_repository_by_owner_name(pool, &owner, &repo)
        .await
        .map_err(repository_lookup_error)?
        .ok_or_else(|| {
            map_collaboration_error(crate::domain::issues::CollaborationError::RepositoryNotFound)
        })?;
    let view = pull_request_diff_review_for_viewer(
        pool,
        repository.id,
        number,
        actor.as_ref().map(|user| user.id),
        PullRequestDiffReviewQuery {
            view: query.view.unwrap_or_else(|| "unified".to_owned()),
            whitespace: query.whitespace.unwrap_or_else(|| "show".to_owned()),
            commit: query.commit,
            filter: query.filter,
            page: query.page.unwrap_or(1),
            page_size: query.page_size.unwrap_or(50),
        },
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(view)))
}

async fn update_viewed_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    RestJson(request): RestJson<UpdatePullViewedFileRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_collaboration_error)?;
    let detail = get_pull_request(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    let viewed = update_pull_request_viewed_file(
        pool,
        detail.pull_request.id,
        actor.0.id,
        request.file_id,
        request.version_key,
        request.viewed,
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(viewed)))
}

async fn create_review_draft_comment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    RestJson(request): RestJson<CreateReviewDraftCommentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_collaboration_error)?;
    let detail = get_pull_request(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    let draft = create_pull_request_review_draft_comment(
        pool,
        CreatePullRequestReviewDraftComment {
            pull_request_id: detail.pull_request.id,
            actor_user_id: actor.0.id,
            file_id: request.file_id,
            body: request.body,
            side: request.side,
            old_line: request.old_line,
            new_line: request.new_line,
            position: request.position,
        },
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(draft)))
}

async fn update_review_draft_comment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number, draft_id)): Path<(String, String, i64, Uuid)>,
    RestJson(request): RestJson<UpdateReviewDraftCommentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_collaboration_error)?;
    let detail = get_pull_request(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    let draft = update_pull_request_review_draft_comment(
        pool,
        UpdatePullRequestReviewDraftComment {
            pull_request_id: detail.pull_request.id,
            actor_user_id: actor.0.id,
            draft_comment_id: draft_id,
            body: request.body,
        },
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(draft)))
}

async fn delete_review_draft_comment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number, draft_id)): Path<(String, String, i64, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_collaboration_error)?;
    let detail = get_pull_request(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    let pending = delete_pull_request_review_draft_comment(
        pool,
        detail.pull_request.id,
        actor.0.id,
        draft_id,
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(pending)))
}

async fn submit_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    RestJson(request): RestJson<SubmitReviewRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_collaboration_error)?;
    let detail = get_pull_request(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    let review = submit_pull_request_review(
        pool,
        SubmitPullRequestReview {
            pull_request_id: detail.pull_request.id,
            actor_user_id: actor.0.id,
            body: request.body,
            state: request.state,
        },
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(review)))
}

async fn abandon_review_draft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_collaboration_error)?;
    let detail = get_pull_request(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    let pending = abandon_pull_request_review_draft(pool, detail.pull_request.id, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;

    Ok(Json(json!(pending)))
}

async fn update_preferences(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<UpdatePullPreferencesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_collaboration_error)?;
    let preferences = save_repository_pull_preferences(
        pool,
        repository_id,
        actor.0.id,
        request.dismissed_contributor_banner,
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(preferences)))
}

fn parse_compare_range(
    range: &str,
) -> Result<(String, String), crate::domain::issues::CollaborationError> {
    let range = range.trim().trim_start_matches('/');
    let Some((base, head)) = range.split_once("...") else {
        return Err(
            crate::domain::issues::CollaborationError::InvalidIssueFilter(
                "compare range must use base...head".to_owned(),
            ),
        );
    };
    let base = urlencoding_decode(base)?;
    let head = urlencoding_decode(head)?;
    if base.trim().is_empty() || head.trim().is_empty() {
        return Err(
            crate::domain::issues::CollaborationError::InvalidIssueFilter(
                "compare range must include both base and head refs".to_owned(),
            ),
        );
    }
    Ok((base, head))
}

fn urlencoding_decode(value: &str) -> Result<String, crate::domain::issues::CollaborationError> {
    url::form_urlencoded::parse(value.as_bytes())
        .next()
        .map(|(key, value)| {
            if value.is_empty() {
                key.into_owned()
            } else {
                format!("{}={}", key, value)
            }
        })
        .ok_or_else(|| {
            crate::domain::issues::CollaborationError::InvalidIssueFilter(
                "compare range contains an invalid ref".to_owned(),
            )
        })
}

async fn resolve_optional_head_repository_id(
    pool: &sqlx::PgPool,
    head_owner: &Option<String>,
    head_repo: &Option<String>,
) -> Result<Option<Uuid>, (StatusCode, Json<ErrorEnvelope>)> {
    match (
        head_owner
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
        head_repo
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    ) {
        (Some(owner), Some(repo)) => Ok(Some(
            get_repository_by_owner_name(pool, owner, repo)
                .await
                .map_err(repository_lookup_error)?
                .ok_or_else(|| {
                    map_collaboration_error(
                        crate::domain::issues::CollaborationError::RepositoryNotFound,
                    )
                })?
                .id,
        )),
        (None, None) => Ok(None),
        _ => Err(map_collaboration_error(
            crate::domain::issues::CollaborationError::InvalidIssueField {
                field_key: "headRepository".to_owned(),
                message: "headOwner and headRepo must be provided together".to_owned(),
            },
        )),
    }
}

fn pull_list_query(
    query: &ListQuery,
    actor: Option<&User>,
) -> Result<PullRequestListQuery, crate::domain::issues::CollaborationError> {
    let q = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("is:pr is:open");
    validate_pull_query(q)?;

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

    let normalized_sort = normalize_pull_sort(&raw_sort, raw_order.as_deref())?;
    if normalized_sort == "best-match" && search_text_from_pull_query(q).is_empty() {
        return Err(
            crate::domain::issues::CollaborationError::InvalidIssueFilter(
                "best match sort requires a search term".to_owned(),
            ),
        );
    }

    Ok(PullRequestListQuery {
        query: Some(q.chars().take(240).collect()),
        state: query
            .state
            .clone()
            .unwrap_or_else(|| pull_state_from_query(q)),
        viewer_user_id: actor.map(|user| user.id),
        author: query
            .author
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| normalize_user_filter(value, actor))
            .or_else(|| {
                qualifier_from_query(q, "author:")
                    .as_deref()
                    .map(|value| normalize_user_filter(value, actor))
            }),
        labels: labels_from_query(q, query.labels.as_deref()),
        milestone: query
            .milestone
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| qualifier_from_query(q, "milestone:")),
        no_milestone: query.no_milestone.unwrap_or(false) || no_filter_from_query(q, "milestone"),
        assignee: query
            .assignee
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| normalize_user_filter(value, actor))
            .or_else(|| {
                qualifier_from_query(q, "assignee:")
                    .as_deref()
                    .map(|value| normalize_user_filter(value, actor))
            }),
        no_assignee: query.no_assignee.unwrap_or(false) || no_filter_from_query(q, "assignee"),
        project: project_filter_from_query(query, q)?,
        review: review_filter_from_query(query, q, actor)?,
        checks: query
            .checks
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| qualifier_from_query(q, "checks:"))
            .map(validate_checks_filter)
            .transpose()?,
        sort: normalized_sort,
    })
}

fn project_filter_from_query(
    query: &ListQuery,
    q: &str,
) -> Result<Option<String>, crate::domain::issues::CollaborationError> {
    let project = query
        .project
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| qualifier_from_query(q, "project:"));
    if project.is_some() {
        return Err(
            crate::domain::issues::CollaborationError::InvalidIssueFilter(
                "project filters are not available until repository project links are modeled"
                    .to_owned(),
            ),
        );
    }
    Ok(None)
}

fn review_filter_from_query(
    query: &ListQuery,
    q: &str,
    actor: Option<&User>,
) -> Result<Option<String>, crate::domain::issues::CollaborationError> {
    query
        .review
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| qualifier_from_query(q, "review:"))
        .or_else(|| {
            qualifier_from_query(q, "review-requested:")
                .map(|value| format!("review-requested:{value}"))
        })
        .or_else(|| {
            qualifier_from_query(q, "reviewed-by:").map(|value| format!("reviewed-by:{value}"))
        })
        .map(|value| normalize_review_filter(&value, actor))
        .transpose()
}

fn validate_pull_query(query: &str) -> Result<(), crate::domain::issues::CollaborationError> {
    for term in pull_query_terms(query) {
        if let Some(value) = term.strip_prefix("state:") {
            if !matches!(value, "open" | "closed" | "merged") {
                return Err(
                    crate::domain::issues::CollaborationError::InvalidIssueFilter(
                        "state filter must be open, closed, or merged".to_owned(),
                    ),
                );
            }
        }
        if let Some(value) = term.strip_prefix("is:") {
            if !matches!(value, "pr" | "pull-request" | "open" | "closed" | "merged") {
                return Err(
                    crate::domain::issues::CollaborationError::InvalidIssueFilter(
                        "is filter must be pr, open, closed, or merged".to_owned(),
                    ),
                );
            }
        }
        for prefix in [
            "author:",
            "label:",
            "milestone:",
            "project:",
            "assignee:",
            "review:",
            "review-requested:",
            "reviewed-by:",
            "checks:",
            "sort:",
            "order:",
        ] {
            if let Some(value) = term.strip_prefix(prefix) {
                if value.trim().trim_matches('"').is_empty() {
                    return Err(
                        crate::domain::issues::CollaborationError::InvalidIssueFilter(format!(
                            "{} filters require a value",
                            prefix.trim_end_matches(':')
                        )),
                    );
                }
            }
        }
        if let Some(value) = term.strip_prefix("no:") {
            if !matches!(value, "assignee" | "milestone") {
                return Err(
                    crate::domain::issues::CollaborationError::InvalidIssueFilter(
                        "no filter must be assignee or milestone".to_owned(),
                    ),
                );
            }
        }
    }
    Ok(())
}

fn pull_state_from_query(query: &str) -> PullRequestState {
    if pull_query_terms(query)
        .into_iter()
        .any(|term| matches!(term.as_str(), "state:merged" | "is:merged"))
    {
        PullRequestState::Merged
    } else if pull_query_terms(query)
        .into_iter()
        .any(|term| matches!(term.as_str(), "state:closed" | "is:closed"))
    {
        PullRequestState::Closed
    } else {
        PullRequestState::Open
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
    labels.extend(qualifier_values_from_query(query, "label:"));
    labels.sort_by_key(|value| value.to_lowercase());
    labels.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    labels
}

fn no_filter_from_query(query: &str, value: &str) -> bool {
    pull_query_terms(query)
        .into_iter()
        .any(|term| term.strip_prefix("no:").is_some_and(|term| term == value))
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

fn normalize_pull_sort(
    sort: &str,
    order: Option<&str>,
) -> Result<String, crate::domain::issues::CollaborationError> {
    let order = order.unwrap_or("desc").to_lowercase();
    if !matches!(order.as_str(), "asc" | "desc") {
        return Err(
            crate::domain::issues::CollaborationError::InvalidIssueFilter(
                "order must be asc or desc".to_owned(),
            ),
        );
    }
    let normalized = match sort.to_lowercase().as_str() {
        "best-match" | "best_match" => "best-match".to_owned(),
        "updated" | "recently-updated" => format!("updated-{order}"),
        "created" | "newest" => format!("created-{order}"),
        "comments" | "commented" | "most-commented" => format!("comments-{order}"),
        "least-commented" => "comments-asc".to_owned(),
        "oldest" => "created-asc".to_owned(),
        "least-recently-updated" => "updated-asc".to_owned(),
        "reactions" | "most-reactions" => "reactions-desc".to_owned(),
        "thumbs-up" | "thumbs_up" | "+1" => "reactions-thumbs_up-desc".to_owned(),
        "thumbs-down" | "thumbs_down" | "-1" => "reactions-thumbs_down-desc".to_owned(),
        "laugh" | "smile" => "reactions-laugh-desc".to_owned(),
        "hooray" | "tada" => "reactions-hooray-desc".to_owned(),
        "confused" | "thinking_face" => "reactions-confused-desc".to_owned(),
        "heart" => "reactions-heart-desc".to_owned(),
        "rocket" => "reactions-rocket-desc".to_owned(),
        "eyes" => "reactions-eyes-desc".to_owned(),
        value => value.to_owned(),
    };
    if !pull_sort_options().contains(&normalized) {
        return Err(
            crate::domain::issues::CollaborationError::InvalidIssueFilter(format!(
                "sort must be one of {}",
                pull_sort_options().join(", ")
            )),
        );
    }
    Ok(normalized)
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

fn validate_review_filter(
    value: String,
) -> Result<String, crate::domain::issues::CollaborationError> {
    if matches!(
        value.as_str(),
        "none"
            | "required"
            | "approved"
            | "changes_requested"
            | "commented"
            | "reviewed_by_me"
            | "not_reviewed_by_me"
            | "review_requested"
            | "team_review_requested"
    ) {
        Ok(value)
    } else {
        Err(
            crate::domain::issues::CollaborationError::InvalidIssueFilter(
                "review must be none, required, approved, changes_requested, reviewed_by_me, not_reviewed_by_me, review_requested, or team_review_requested".to_owned(),
            ),
        )
    }
}

fn normalize_review_filter(
    value: &str,
    actor: Option<&User>,
) -> Result<String, crate::domain::issues::CollaborationError> {
    let normalized = value.trim().trim_matches('"').to_lowercase();
    let canonical = match normalized.as_str() {
        "none" | "no_reviews" | "no-reviews" => "none",
        "required" | "review_required" | "review-required" => "required",
        "approved" => "approved",
        "changes_requested" | "changes-requested" => "changes_requested",
        "commented" => "commented",
        "reviewed_by_me" | "reviewed-by-me" | "reviewed-by:@me" => "reviewed_by_me",
        "not_reviewed_by_me" | "not-reviewed-by-me" => "not_reviewed_by_me",
        "review_requested" | "review-requested" | "review-requested:@me" => "review_requested",
        "team_review_requested"
        | "team-review-requested"
        | "team-review-requested:@me"
        | "review-requested:team" => "team_review_requested",
        other => return validate_review_filter(other.to_owned()),
    };
    if matches!(
        canonical,
        "reviewed_by_me" | "not_reviewed_by_me" | "review_requested" | "team_review_requested"
    ) && actor.is_none()
    {
        return Err(
            crate::domain::issues::CollaborationError::InvalidIssueFilter(
                "viewer-relative review filters require a signed-in session".to_owned(),
            ),
        );
    }
    Ok(canonical.to_owned())
}

fn validate_checks_filter(
    value: String,
) -> Result<String, crate::domain::issues::CollaborationError> {
    if matches!(
        value.as_str(),
        "success" | "failure" | "pending" | "running"
    ) {
        Ok(value)
    } else {
        Err(
            crate::domain::issues::CollaborationError::InvalidIssueFilter(
                "checks must be success, failure, pending, or running".to_owned(),
            ),
        )
    }
}

fn qualifier_from_query(query: &str, prefix: &str) -> Option<String> {
    qualifier_values_from_query(query, prefix)
        .into_iter()
        .next()
}

fn qualifier_values_from_query(query: &str, prefix: &str) -> Vec<String> {
    pull_query_terms(query)
        .into_iter()
        .filter_map(|term| {
            term.strip_prefix(prefix)
                .map(|value| value.trim().trim_matches('"').to_owned())
        })
        .filter(|value| !value.is_empty())
        .collect()
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

fn repository_lookup_error(error: RepositoryError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        RepositoryError::Sqlx(error) => {
            map_collaboration_error(crate::domain::issues::CollaborationError::Sqlx(error))
        }
        _ => map_collaboration_error(crate::domain::issues::CollaborationError::RepositoryNotFound),
    }
}

async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<CreatePullRequestRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id = get_repository_by_owner_name(pool, &owner, &repo)
        .await
        .map_err(repository_lookup_error)?
        .ok_or_else(|| {
            map_collaboration_error(crate::domain::issues::CollaborationError::RepositoryNotFound)
        })?
        .id;
    if request.title.trim().is_empty()
        || request.head_ref.trim().is_empty()
        || request.base_ref.trim().is_empty()
    {
        return Err(crate::api_types::error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "pull request title, headRef, and baseRef are required",
        ));
    }
    let detail = create_pull_request(
        pool,
        CreatePullRequest {
            repository_id,
            actor_user_id: actor.0.id,
            title: request.title,
            body: request.body,
            head_ref: request.head_ref,
            base_ref: request.base_ref,
            head_repository_id: match request.head_repository_id {
                Some(id) => Some(id),
                None => {
                    resolve_optional_head_repository_id(
                        pool,
                        &request.head_owner,
                        &request.head_repo,
                    )
                    .await?
                }
            },
            is_draft: request.is_draft.unwrap_or(false),
            label_ids: request.label_ids.unwrap_or_default(),
            milestone_id: request.milestone_id,
            assignee_user_ids: request.assignee_user_ids.unwrap_or_default(),
            reviewer_user_ids: request.reviewer_user_ids.unwrap_or_default(),
            template_slug: request.template_slug,
        },
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok((StatusCode::CREATED, Json(json!(detail))))
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
        .ok_or_else(|| {
            map_collaboration_error(crate::domain::issues::CollaborationError::RepositoryNotFound)
        })?;
    let detail = pull_request_detail_view_for_viewer(
        pool,
        repository.id,
        number,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_collaboration_error)?;

    Ok(Json(json!(detail)))
}

async fn update_state(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    RestJson(request): RestJson<UpdatePullRequestStateRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_collaboration_error)?;
    let detail = get_pull_request(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    let updated = update_pull_request_state(
        pool,
        detail.pull_request.id,
        UpdatePullRequestState {
            actor_user_id: actor.0.id,
            state: request.state,
            merge_commit_id: request.merge_commit_id,
            method: None,
        },
    )
    .await
    .map_err(map_collaboration_error)?;

    let refreshed = pull_request_detail_view_for_viewer(
        pool,
        updated.repository_id,
        updated.number,
        Some(actor.0.id),
    )
    .await
    .map_err(map_collaboration_error)?;
    Ok(Json(json!(refreshed)))
}

async fn merge(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    RestJson(request): RestJson<MergePullRequestRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_collaboration_error)?;
    let detail = get_pull_request(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    let _method = request.method.unwrap_or_default();
    let updated = update_pull_request_state(
        pool,
        detail.pull_request.id,
        UpdatePullRequestState {
            actor_user_id: actor.0.id,
            state: PullRequestState::Merged,
            merge_commit_id: request.merge_commit_id,
            method: Some(_method),
        },
    )
    .await
    .map_err(map_collaboration_error)?;
    let refreshed = pull_request_detail_view_for_viewer(
        pool,
        updated.repository_id,
        updated.number,
        Some(actor.0.id),
    )
    .await
    .map_err(map_collaboration_error)?;
    Ok(Json(json!(refreshed)))
}

async fn update_review_requests(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    RestJson(request): RestJson<UpdatePullReviewRequestsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_collaboration_error)?;
    let detail = get_pull_request(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    update_pull_request_review_requests(
        pool,
        detail.pull_request.id,
        UpdatePullRequestReviewRequests {
            actor_user_id: actor.0.id,
            reviewer_user_ids: request.reviewer_user_ids.unwrap_or_default(),
        },
    )
    .await
    .map_err(map_collaboration_error)?;
    let updated =
        pull_request_detail_view_for_viewer(pool, repository_id, number, Some(actor.0.id))
            .await
            .map_err(map_collaboration_error)?;
    Ok(Json(json!(updated)))
}

async fn update_draft(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    RestJson(request): RestJson<UpdatePullDraftRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_collaboration_error)?;
    let detail = get_pull_request(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    update_pull_request_draft_state(
        pool,
        detail.pull_request.id,
        UpdatePullRequestDraftState {
            actor_user_id: actor.0.id,
            is_draft: request.is_draft,
        },
    )
    .await
    .map_err(map_collaboration_error)?;
    let updated =
        pull_request_detail_view_for_viewer(pool, repository_id, number, Some(actor.0.id))
            .await
            .map_err(map_collaboration_error)?;
    Ok(Json(json!(updated)))
}

async fn update_metadata(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    RestJson(request): RestJson<UpdatePullMetadataRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_collaboration_error)?;
    let detail = get_pull_request(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    update_pull_request_metadata(
        pool,
        detail.pull_request.id,
        UpdatePullRequestMetadata {
            actor_user_id: actor.0.id,
            label_ids: request.label_ids.unwrap_or_default(),
            assignee_user_ids: request.assignee_user_ids.unwrap_or_default(),
            milestone_id: request.milestone_id,
        },
    )
    .await
    .map_err(map_collaboration_error)?;
    let updated =
        pull_request_detail_view_for_viewer(pool, repository_id, number, Some(actor.0.id))
            .await
            .map_err(map_collaboration_error)?;
    Ok(Json(json!(updated)))
}

async fn update_subscription(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    RestJson(request): RestJson<UpdatePullSubscriptionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_collaboration_error)?;
    let detail = get_pull_request(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    let subscription = update_pull_request_subscription(
        pool,
        detail.pull_request.id,
        UpdatePullRequestSubscription {
            actor_user_id: actor.0.id,
            subscribed: request.subscribed,
        },
    )
    .await
    .map_err(map_collaboration_error)?;
    Ok(Json(json!(subscription)))
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
    let detail = get_pull_request(pool, repository_id, number, actor.0.id)
        .await
        .map_err(map_collaboration_error)?;
    if request.body.trim().is_empty() {
        return Err(crate::api_types::error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "comment body is required",
        ));
    }
    let comment = add_pull_request_comment(
        pool,
        detail.pull_request.id,
        CreateComment {
            actor_user_id: actor.0.id,
            body: request.body,
        },
    )
    .await
    .map_err(map_collaboration_error)?;
    let item = pull_request_comment_timeline_item(pool, comment.id, Some(actor.0.id))
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
        .ok_or_else(|| {
            map_collaboration_error(crate::domain::issues::CollaborationError::RepositoryNotFound)
        })?;
    let pull_request_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM pull_requests WHERE repository_id = $1 AND number = $2",
    )
    .bind(repository.id)
    .bind(number)
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        map_collaboration_error(crate::domain::issues::CollaborationError::Sqlx(error))
    })?
    .ok_or_else(|| {
        map_collaboration_error(crate::domain::issues::CollaborationError::PullRequestNotFound)
    })?;
    let events =
        pull_request_timeline_view(pool, pull_request_id, actor.as_ref().map(|user| user.id))
            .await
            .map_err(map_collaboration_error)?;

    Ok(Json(json!(events)))
}

fn database_unavailable() -> (StatusCode, Json<ErrorEnvelope>) {
    shared_database_unavailable()
}
