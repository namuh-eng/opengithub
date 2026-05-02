use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope, RestJson},
    auth::extractor::AuthenticatedUser,
    domain::profiles::{
        profile_by_login, report_user, set_block_state, set_follow_state, ProfileError,
        ReportInput,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/users/:login/profile", get(read_profile))
        .route(
            "/api/users/:login/follow",
            put(follow_profile).delete(unfollow_profile),
        )
        .route(
            "/api/users/:login/block",
            put(block_profile).delete(unblock_profile),
        )
        .route("/api/users/:login/report", post(report_profile))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileQuery {
    year: Option<i32>,
}

async fn read_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(login): Path<String>,
    Query(query): Query<ProfileQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let viewer = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let profile = profile_by_login(pool, viewer.map(|user| user.id), &login, query.year)
        .await
        .map_err(map_profile_error)?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "not_found", "profile was not found"))?;
    Ok(Json(json!(profile)))
}

async fn follow_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(login): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_follow(State(state), headers, login, true).await
}

async fn unfollow_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(login): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_follow(State(state), headers, login, false).await
}

async fn set_follow(
    State(state): State<AppState>,
    headers: HeaderMap,
    login: String,
    following: bool,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let state = set_follow_state(pool, actor.0.id, &login, following)
        .await
        .map_err(map_profile_error)?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "not_found", "profile was not found"))?;
    Ok(Json(json!(state)))
}

async fn block_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(login): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_block(State(state), headers, login, true).await
}

async fn unblock_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(login): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_block(State(state), headers, login, false).await
}

async fn set_block(
    State(state): State<AppState>,
    headers: HeaderMap,
    login: String,
    blocked: bool,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let state = set_block_state(pool, actor.0.id, &login, blocked)
        .await
        .map_err(map_profile_error)?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "not_found", "profile was not found"))?;
    Ok(Json(json!(state)))
}

async fn report_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(login): Path<String>,
    RestJson(input): RestJson<ReportInput>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let receipt = report_user(pool, actor.0.id, &login, input)
        .await
        .map_err(map_profile_error)?
        .ok_or_else(|| error_response(StatusCode::NOT_FOUND, "not_found", "profile was not found"))?;
    Ok((StatusCode::CREATED, Json(json!(receipt))))
}

fn map_profile_error(error: ProfileError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        ProfileError::NotFound => error_response(StatusCode::NOT_FOUND, "not_found", "profile was not found"),
        ProfileError::SelfRelationship => error_response(
            StatusCode::CONFLICT,
            "self_relationship_not_allowed",
            "Profile relationship controls cannot target yourself",
        ),
        ProfileError::InvalidReportReason => error_response(
            StatusCode::BAD_REQUEST,
            "invalid_report_reason",
            "Report reason is required and must be concise",
        ),
        ProfileError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "profile_failed",
            "Profile request failed",
        ),
    }
}
