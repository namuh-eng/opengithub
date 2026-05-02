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
    },
    auth::extractor::AuthenticatedUser,
    domain::search::{
        create_saved_search, delete_saved_search, record_recent_search, search_code_results,
        search_collaboration_results, search_documents, search_suggestions, CodeSearchQuery,
        CollaborationSearchQuery, CreateSavedSearchInput, SearchDocumentKind, SearchError,
        SearchQuery, SearchSuggestionQuery,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/search", get(search))
        .route("/api/search/suggestions", get(suggestions))
        .route("/api/search/saved-searches", post(create_saved))
        .route("/api/search/saved-searches/:id", delete(delete_saved))
        .route("/api/search/recent", post(create_recent))
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
    sort: Option<String>,
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

async fn search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(request): Query<SearchRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let pagination = normalize_pagination(request.page, request.page_size);
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
    let scope = selected_type.to_owned();
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
    if matches!(
        kind,
        Some(SearchDocumentKind::Issue | SearchDocumentKind::PullRequest)
    ) {
        let collaboration_kind = kind.clone().expect("kind checked above");
        let results = search_collaboration_results(
            pool,
            CollaborationSearchQuery {
                actor_user_id: actor.0.id,
                query: search_query.clone(),
                kind: collaboration_kind,
                page: pagination.page,
                page_size: pagination.page_size,
                sort: request.sort.clone(),
            },
        )
        .await
        .map_err(map_search_error)?;
        let _ = record_recent_search(pool, actor.0.id, &search_query, &scope, Some(selected_type))
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
    let _ =
        record_recent_search(pool, actor.0.id, &search_query, &scope, Some(selected_type)).await;

    Ok(Json(json!(results)))
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
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
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

fn map_search_error(error: SearchError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        SearchError::QueryTooShort | SearchError::InvalidKind(_) | SearchError::Validation(_) => {
            error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_failed",
                error.to_string(),
            )
        }
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
