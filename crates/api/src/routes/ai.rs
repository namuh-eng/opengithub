use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope, RestJson},
    auth::extractor::AuthenticatedUser,
    domain::ai::{
        ai_changelog, pull_request_ai_summary, repository_ai_summary, AiChangelogRequest, AiError,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/ai/repos/:owner/:repo/summary",
            get(repo_summary).post(regenerate_repo_summary),
        )
        .route(
            "/api/ai/repos/:owner/:repo/pulls/:number/summary",
            get(pr_summary).post(regenerate_pr_summary),
        )
        .route(
            "/api/ai/repos/:owner/:repo/releases/changelog",
            post(changelog),
        )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AiQuery {
    regenerate: Option<bool>,
}

async fn repo_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<AiQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let summary = repository_ai_summary(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
        query.regenerate.unwrap_or(false),
    )
    .await
    .map_err(map_ai_error)?;
    Ok(Json(json!(summary)))
}

async fn regenerate_repo_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let summary = repository_ai_summary(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
        true,
    )
    .await
    .map_err(map_ai_error)?;
    Ok(Json(json!(summary)))
}

async fn pr_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
    Query(query): Query<AiQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let summary = pull_request_ai_summary(
        pool,
        &owner,
        &repo,
        number,
        actor.as_ref().map(|user| user.id),
        query.regenerate.unwrap_or(false),
    )
    .await
    .map_err(map_ai_error)?;
    Ok(Json(json!(summary)))
}

async fn regenerate_pr_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, number)): Path<(String, String, i64)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let summary = pull_request_ai_summary(
        pool,
        &owner,
        &repo,
        number,
        actor.as_ref().map(|user| user.id),
        true,
    )
    .await
    .map_err(map_ai_error)?;
    Ok(Json(json!(summary)))
}

async fn changelog(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<AiQuery>,
    RestJson(request): RestJson<AiChangelogRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let changelog = ai_changelog(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
        request,
        query.regenerate.unwrap_or(false),
    )
    .await
    .map_err(map_ai_error)?;
    Ok(Json(json!(changelog)))
}

fn map_ai_error(error: AiError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        AiError::RepositoryNotFound | AiError::PullRequestNotFound | AiError::ReleaseNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        AiError::PermissionDenied => error_response(
            StatusCode::FORBIDDEN,
            "forbidden",
            "Repository is not readable",
        ),
        AiError::Disabled => {
            error_response(StatusCode::FORBIDDEN, "ai_disabled", error.to_string())
        }
        AiError::ProviderNotConfigured => error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "ai_provider_not_configured",
            "OPENAI_API_KEY is not configured",
        ),
        AiError::ProviderFailed => error_response(
            StatusCode::BAD_GATEWAY,
            "ai_provider_failed",
            "AI provider request failed",
        ),
        AiError::Repository(_) | AiError::Sqlx(_) => {
            tracing::warn!(%error, "AI surface failed");
            error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "database_unavailable",
                "AI output could not be loaded",
            )
        }
    }
}
