use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    api_types::{
        database_unavailable, error_response, normalize_pagination, ErrorEnvelope, ListEnvelope,
        Pagination, DEFAULT_PAGE, DEFAULT_PAGE_SIZE,
    },
    auth::extractor::AuthenticatedUser,
    domain::search::{
        create_saved_search, delete_saved_search, record_recent_search, search_code_results,
        search_collaboration_results, search_documents, search_index_status, search_suggestions,
        CodeSearchQuery, CodeSearchResponse, CollaborationSearchKind, CollaborationSearchQuery,
        CollaborationSearchResponse, CreateSavedSearchInput, SearchDocumentKind, SearchError,
        SearchQuery, SearchResult, SearchSuggestionQuery,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/search", get(search))
        .route("/api/search/code", get(search_code_rest))
        .route("/api/search/repositories", get(search_repositories_rest))
        .route("/api/search/issues", get(search_issues_rest))
        .route("/api/search/users", get(search_users_rest))
        .route("/api/search/commits", get(search_commits_rest))
        .route("/api/search/suggestions", get(suggestions))
        .route("/api/search/saved-searches", post(create_saved))
        .route("/api/search/saved-searches/:id", delete(delete_saved))
        .route("/api/search/recent", post(create_recent))
        .route("/api/admin/search", get(admin_search_status))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchRequest {
    q: Option<String>,
    kind: Option<String>,
    #[serde(rename = "type")]
    result_type: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
    #[serde(alias = "per_page")]
    per_page: Option<i64>,
    sort: Option<String>,
    order: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SuggestionsRequest {
    q: Option<String>,
    scope: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSavedSearchRequest {
    name: Option<String>,
    query: Option<String>,
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateRecentSearchRequest {
    query: Option<String>,
    scope: Option<String>,
    result_type: Option<String>,
}

async fn suggestions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(request): Query<SuggestionsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let dashboard = search_suggestions(
        pool,
        SearchSuggestionQuery {
            actor_user_id: actor.0.id,
            query: request.q.unwrap_or_default(),
            scope: request.scope,
            limit: request.limit.unwrap_or(8),
        },
    )
    .await
    .map_err(map_search_error)?;

    Ok(Json(json!(dashboard)))
}

async fn admin_search_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let _actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let status = search_index_status(pool).await.map_err(map_search_error)?;

    Ok(Json(json!(status)))
}

async fn search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(request): Query<SearchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let pagination = normalize_web_search_pagination(request.page, request.page_size);
    let selected_type = request
        .result_type
        .as_deref()
        .or(request.kind.as_deref())
        .unwrap_or("repositories");
    if matches!(selected_type, "discussions" | "discussion") {
        if request
            .q
            .as_deref()
            .unwrap_or_default()
            .trim()
            .chars()
            .count()
            < 2
        {
            return Err(map_search_error(SearchError::QueryTooShort));
        }
        return Ok(Json(json!(ListEnvelope::<serde_json::Value> {
            items: Vec::new(),
            total: 0,
            page: pagination.page,
            page_size: pagination.page_size,
        })));
    }
    let kind = request
        .result_type
        .as_deref()
        .or(request.kind.as_deref())
        .map(search_kind_from_param)
        .transpose()
        .map_err(map_search_error)?;
    let search_query = request.q.unwrap_or_default();
    if kind == Some(SearchDocumentKind::Code) {
        let results = search_code_results(
            pool,
            CodeSearchQuery {
                actor_user_id: actor.0.id,
                query: search_query.clone(),
                page: pagination.page,
                page_size: pagination.page_size,
            },
        )
        .await
        .map_err(map_search_error)?;
        let _ = record_recent_search(pool, actor.0.id, &search_query, "code", Some("code")).await;

        return Ok(Json(json!(results)));
    }
    if let Some(result_type) = collaboration_kind_from_param(selected_type) {
        let results = search_collaboration_results(
            pool,
            CollaborationSearchQuery {
                actor_user_id: actor.0.id,
                query: search_query.clone(),
                result_type,
                page: pagination.page,
                page_size: pagination.page_size,
                sort: request.sort,
            },
        )
        .await
        .map_err(map_search_error)?;
        let _ = record_recent_search(
            pool,
            actor.0.id,
            &search_query,
            selected_type,
            Some(selected_type),
        )
        .await;

        return Ok(Json(json!(results)));
    }

    let results = search_documents(
        pool,
        SearchQuery {
            actor_user_id: actor.0.id,
            query: search_query.clone(),
            kind,
            page: pagination.page,
            page_size: pagination.page_size,
        },
    )
    .await
    .map_err(map_search_error)?;
    let scope = selected_type.to_owned();
    let _ =
        record_recent_search(pool, actor.0.id, &search_query, &scope, Some(selected_type)).await;

    Ok(Json(json!(results)))
}

async fn search_code_rest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(request): Query<SearchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let pagination = normalize_rest_pagination(&request);
    let query = request.q.unwrap_or_default();
    let results = search_code_results(
        pool,
        CodeSearchQuery {
            actor_user_id: actor.0.id,
            query: query.clone(),
            page: pagination.page,
            page_size: pagination.page_size,
        },
    )
    .await
    .map_err(map_search_error)?;
    let _ = record_recent_search(pool, actor.0.id, &query, "code", Some("code")).await;

    Ok(Json(github_code_response(results)))
}

async fn search_repositories_rest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(request): Query<SearchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    generic_rest_search(
        state,
        headers,
        request,
        SearchDocumentKind::Repository,
        "repositories",
    )
    .await
}

async fn search_users_rest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(request): Query<SearchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    generic_rest_search(state, headers, request, SearchDocumentKind::User, "users").await
}

async fn search_commits_rest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(request): Query<SearchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    generic_rest_search(
        state,
        headers,
        request,
        SearchDocumentKind::Commit,
        "commits",
    )
    .await
}

async fn search_issues_rest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(request): Query<SearchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let pagination = normalize_rest_pagination(&request);
    let query = request.q.unwrap_or_default();
    let results = search_collaboration_results(
        pool,
        CollaborationSearchQuery {
            actor_user_id: actor.0.id,
            query: query.clone(),
            result_type: CollaborationSearchKind::Issues,
            page: pagination.page,
            page_size: pagination.page_size,
            sort: rest_collaboration_sort(request.sort.as_deref(), request.order.as_deref()),
        },
    )
    .await
    .map_err(map_search_error)?;
    let _ = record_recent_search(pool, actor.0.id, &query, "issues", Some("issues")).await;

    Ok(Json(github_collaboration_response(results)))
}

async fn generic_rest_search(
    state: AppState,
    headers: HeaderMap,
    request: SearchRequest,
    kind: SearchDocumentKind,
    scope: &'static str,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let pagination = normalize_rest_pagination(&request);
    let original_query = request.q.unwrap_or_default();
    let query = generic_rest_query(&original_query, scope);
    let _sort = request.sort.as_deref();
    let _order = request.order.as_deref();
    let results = search_documents(
        pool,
        SearchQuery {
            actor_user_id: actor.0.id,
            query: query.clone(),
            kind: Some(kind),
            page: pagination.page,
            page_size: pagination.page_size,
        },
    )
    .await
    .map_err(map_search_error)?;
    let _ = record_recent_search(pool, actor.0.id, &original_query, scope, Some(scope)).await;

    Ok(Json(github_generic_response(results)))
}

async fn create_saved(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateSavedSearchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let saved = create_saved_search(
        pool,
        CreateSavedSearchInput {
            actor_user_id: actor.0.id,
            name: request.name.unwrap_or_default(),
            query: request.query.unwrap_or_default(),
            scope: request.scope,
        },
    )
    .await
    .map_err(map_search_error)?;

    Ok(Json(json!(saved)))
}

async fn delete_saved(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let id = id.parse::<uuid::Uuid>().map_err(|_| {
        map_search_error(SearchError::Validation(
            "saved search id must be a valid UUID".to_owned(),
        ))
    })?;
    delete_saved_search(pool, actor.0.id, id)
        .await
        .map_err(map_search_error)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn create_recent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateRecentSearchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let recent = record_recent_search(
        pool,
        actor.0.id,
        request.query.as_deref().unwrap_or_default(),
        request.scope.as_deref().unwrap_or("repositories"),
        request.result_type.as_deref(),
    )
    .await
    .map_err(map_search_error)?;

    Ok(Json(json!(recent)))
}

fn search_kind_from_param(value: &str) -> Result<SearchDocumentKind, SearchError> {
    match value {
        "repositories" | "repository" => Ok(SearchDocumentKind::Repository),
        "code" => Ok(SearchDocumentKind::Code),
        "commits" | "commit" => Ok(SearchDocumentKind::Commit),
        "issues" | "issue" => Ok(SearchDocumentKind::Issue),
        "pull_requests" | "pull_request" | "pullrequests" | "pulls" | "pull" => {
            Ok(SearchDocumentKind::PullRequest)
        }
        "users" | "user" => Ok(SearchDocumentKind::User),
        "organizations" | "organization" | "orgs" | "org" => Ok(SearchDocumentKind::Organization),
        "packages" | "package" => Ok(SearchDocumentKind::Package),
        other => Err(SearchError::InvalidKind(other.to_owned())),
    }
}

fn normalize_web_search_pagination(page: Option<i64>, page_size: Option<i64>) -> Pagination {
    Pagination {
        page: page.unwrap_or(DEFAULT_PAGE).max(1),
        page_size: page_size.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, 50),
    }
}

fn normalize_rest_pagination(request: &SearchRequest) -> Pagination {
    normalize_pagination(request.page, request.per_page.or(request.page_size))
}

fn rest_collaboration_sort(sort: Option<&str>, order: Option<&str>) -> Option<String> {
    let sort = sort?;
    let direction = order.unwrap_or("desc");
    let normalized = match (sort, direction) {
        ("comments", "asc") => "comments-asc",
        ("comments", _) => "comments-desc",
        ("created", "asc") => "created-asc",
        ("created", _) => "created-desc",
        ("updated", "asc") => "updated-asc",
        ("updated", _) => "updated-desc",
        ("interactions", "asc") => "least_commented",
        ("interactions", _) => "most_commented",
        _ => sort,
    };
    Some(normalized.to_owned())
}

fn generic_rest_query(query: &str, scope: &str) -> String {
    let mut terms = Vec::new();
    for token in query.split_whitespace() {
        if let Some((qualifier, value)) = token.split_once(':') {
            let qualifier = qualifier.to_ascii_lowercase();
            let value = value.trim().trim_matches('"');
            if is_rest_qualifier(&qualifier) {
                match (scope, qualifier.as_str()) {
                    ("users", "user") | ("users", "org") => terms.push(value.to_owned()),
                    ("repositories", "repo") | ("commits", "repo") => {
                        terms.extend(value.split('/').filter_map(non_empty_string));
                    }
                    ("repositories", "user") | ("repositories", "org") => {
                        terms.push(value.to_owned());
                    }
                    _ => {}
                }
                continue;
            }
        }
        terms.push(token.to_owned());
    }

    let normalized = terms.join(" ");
    if normalized.trim().is_empty() {
        query.to_owned()
    } else {
        normalized
    }
}

fn is_rest_qualifier(value: &str) -> bool {
    matches!(
        value,
        "repo" | "path" | "user" | "org" | "language" | "state" | "is"
    )
}

fn non_empty_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn github_code_response(results: CodeSearchResponse) -> serde_json::Value {
    let total = rest_total(results.total);
    json!({
        "total_count": total,
        "incomplete_results": results.total > 1000,
        "items": results.items.iter().map(github_search_item).collect::<Vec<_>>(),
        "page": results.page,
        "per_page": results.page_size,
        "facets": results.facets,
        "active_chips": results.active_chips,
        "query_duration_ms": results.query_duration_ms,
        "diagnostics": results.diagnostics,
    })
}

fn github_collaboration_response(results: CollaborationSearchResponse) -> serde_json::Value {
    let total = rest_total(results.total);
    json!({
        "total_count": total,
        "incomplete_results": results.total > 1000,
        "items": results.items.into_iter().map(|item| {
            json!({
                "id": item.id,
                "type": item.result_type,
                "url": item.href,
                "html_url": item.href,
                "repository_url": item.repository.href,
                "repository": item.repository,
                "number": item.number,
                "title": item.title,
                "state": item.state,
                "labels": item.labels,
                "user": item.author,
                "assignees": item.assignees,
                "milestone": item.milestone,
                "comments": item.comment_count,
                "score": item.rank,
                "text_matches": item.snippets,
            })
        }).collect::<Vec<_>>(),
        "page": results.page,
        "per_page": results.page_size,
        "facets": results.facets,
        "active_chips": results.active_chips,
        "sort": results.sort,
        "query_duration_ms": results.query_duration_ms,
        "diagnostics": results.diagnostics,
    })
}

fn github_generic_response(results: ListEnvelope<SearchResult>) -> serde_json::Value {
    let total = rest_total(results.total);
    json!({
        "total_count": total,
        "incomplete_results": results.total > 1000,
        "items": results.items.iter().map(github_search_item).collect::<Vec<_>>(),
        "page": results.page,
        "per_page": results.page_size,
    })
}

fn github_search_item(result: &SearchResult) -> serde_json::Value {
    match result.result_type.as_str() {
        "repositories" => json!({
            "id": result.document.id,
            "name": result.repository_name.as_deref().unwrap_or(&result.title),
            "full_name": full_name(result.owner_login.as_deref(), result.repository_name.as_deref()),
            "owner": github_owner(result.owner_login.as_deref(), "User"),
            "private": result.visibility.as_str() == "private",
            "html_url": result.href,
            "url": result.href,
            "description": result.summary,
            "score": result.rank,
        }),
        "users" => json!({
            "id": result.document.id,
            "login": result.owner_login.as_deref().unwrap_or(&result.document.resource_id),
            "type": "User",
            "avatar_url": result.avatar_url,
            "html_url": result.href,
            "url": result.href,
            "score": result.rank,
        }),
        "commits" => {
            let commit = result.commit.as_ref();
            json!({
                "sha": commit.map(|commit| commit.oid.as_str()).unwrap_or(&result.document.resource_id),
                "html_url": result.href,
                "url": result.href,
                "repository": github_repository_summary(result),
                "commit": {
                    "message": result.document.body,
                    "author": {
                        "name": commit.and_then(|commit| commit.author_login.as_deref()),
                        "date": commit.and_then(|commit| commit.committed_at),
                    }
                },
                "score": result.rank,
            })
        }
        "code" => json!({
            "name": result.document.path.as_deref().unwrap_or(&result.title),
            "path": result.document.path,
            "sha": result.document.resource_id,
            "html_url": result.href,
            "url": result.href,
            "repository": github_repository_summary(result),
            "score": result.rank,
            "text_matches": result.snippets,
        }),
        _ => json!({
            "id": result.document.id,
            "type": result.result_type,
            "title": result.title,
            "html_url": result.href,
            "url": result.href,
            "score": result.rank,
        }),
    }
}

fn github_repository_summary(result: &SearchResult) -> serde_json::Value {
    json!({
        "id": result.document.repository_id,
        "name": result.repository_name,
        "full_name": full_name(result.owner_login.as_deref(), result.repository_name.as_deref()),
        "owner": github_owner(result.owner_login.as_deref(), "User"),
        "private": result.visibility.as_str() == "private",
        "html_url": result.owner_login.as_ref().zip(result.repository_name.as_ref()).map(|(owner, repo)| format!("/{owner}/{repo}")),
    })
}

fn github_owner(login: Option<&str>, owner_type: &str) -> serde_json::Value {
    json!({
        "login": login,
        "type": owner_type,
        "html_url": login.map(|login| format!("/{login}")),
    })
}

fn full_name(owner_login: Option<&str>, repository_name: Option<&str>) -> Option<String> {
    owner_login
        .zip(repository_name)
        .map(|(owner, repo)| format!("{owner}/{repo}"))
}

fn rest_total(total: i64) -> i64 {
    total.clamp(0, 1000)
}

fn collaboration_kind_from_param(value: &str) -> Option<CollaborationSearchKind> {
    match value {
        "issues" | "issue" => Some(CollaborationSearchKind::Issues),
        "pull_requests" | "pull_request" | "pullrequests" | "pulls" | "pull" => {
            Some(CollaborationSearchKind::PullRequests)
        }
        _ => None,
    }
}

fn map_search_error(error: SearchError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        SearchError::QueryTooShort
        | SearchError::InvalidKind(_)
        | SearchError::InvalidIndexStatus(_)
        | SearchError::Validation(_) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        SearchError::DuplicateSavedSearchName => error_response(
            StatusCode::CONFLICT,
            "duplicate_saved_search",
            error.to_string(),
        ),
        SearchError::SavedSearchNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        SearchError::RepositoryAccessDenied => error_response(
            StatusCode::FORBIDDEN,
            "forbidden",
            "user does not have repository access",
        ),
        SearchError::Repository(_) | SearchError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "search operation failed",
        ),
    }
}
