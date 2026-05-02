use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::get,
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
        search_documents, search_suggestions, SearchDocumentKind, SearchError, SearchQuery,
        SearchSuggestionQuery,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/search", get(search))
        .route("/api/search/suggestions", get(suggestions))
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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SuggestionsRequest {
    q: Option<String>,
    scope: Option<String>,
    limit: Option<i64>,
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
    let results = search_documents(
        pool,
        SearchQuery {
            actor_user_id: actor.0.id,
            query: request.q.unwrap_or_default(),
            kind,
            page: pagination.page,
            page_size: pagination.page_size,
        },
    )
    .await
    .map_err(map_search_error)?;

    Ok(Json(json!(results)))
}

fn search_kind_from_param(value: &str) -> Result<SearchDocumentKind, SearchError> {
    match value {
        "repositories" | "repository" => Ok(SearchDocumentKind::Repository),
        "code" => Ok(SearchDocumentKind::Code),
        "commits" | "commit" => Ok(SearchDocumentKind::Commit),
        "issues" | "issue" => Ok(SearchDocumentKind::Issue),
        "pull_requests" | "pull_request" | "pulls" | "pull" => Ok(SearchDocumentKind::PullRequest),
        "users" | "user" => Ok(SearchDocumentKind::User),
        "organizations" | "organization" | "orgs" | "org" => Ok(SearchDocumentKind::Organization),
        "packages" | "package" => Ok(SearchDocumentKind::Package),
        other => Err(SearchError::InvalidKind(other.to_owned())),
    }
}

fn map_search_error(error: SearchError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        SearchError::QueryTooShort | SearchError::InvalidKind(_) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
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
