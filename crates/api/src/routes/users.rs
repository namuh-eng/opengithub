use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, patch, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope, RestJson},
    auth::extractor::AuthenticatedUser,
    domain::{
        identity::User,
        personal_settings::{
            personal_profile_settings, update_personal_avatar, update_personal_profile_settings,
            PersonalProfileSettings, PersonalSettingsError, UpdateAvatarInput,
            UpdatePersonalProfileSettings,
        },
        profiles::{
            block_user, follow_user, profile_repositories, public_user_profile, report_user,
            starred_repositories, unfollow_user, ProfileActionState, ProfileError, ProfileReport,
            ProfileRepositoryList, ProfileRepositoryListQuery, PublicUserProfile,
        },
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/user", get(current_api_user))
        .route(
            "/api/user/settings/profile",
            get(personal_profile_settings_route).patch(update_personal_profile_settings_route),
        )
        .route(
            "/api/user/settings/profile/avatar",
            patch(update_personal_avatar_route),
        )
        .route("/api/users/:username/profile", get(public_profile))
        .route(
            "/api/users/:username/repositories",
            get(public_repositories),
        )
        .route("/api/users/:username/stars", get(public_stars))
        .route("/api/users/:username/follow", put(follow).delete(unfollow))
        .route("/api/users/:username/block", put(block))
        .route("/api/users/:username/reports", post(report))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiUser {
    pub id: Uuid,
    pub login: String,
    pub name: Option<String>,
    pub email: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    #[serde(rename = "htmlUrl")]
    pub html_url: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

impl ApiUser {
    pub fn from_user(user: User, app_url: &url::Url) -> Self {
        let login = user
            .username
            .clone()
            .unwrap_or_else(|| fallback_login_from_email(&user.email));
        let html_url = app_url
            .join(&format!("/{login}"))
            .map(|url| url.to_string())
            .unwrap_or_else(|_| format!("/{login}"));

        Self {
            id: user.id,
            login,
            name: user.display_name,
            email: user.email,
            avatar_url: user.avatar_url,
            html_url,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

pub async fn current_api_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiUser>, (StatusCode, Json<ErrorEnvelope>)> {
    let user = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    Ok(Json(ApiUser::from_user(user, &state.config.app_url)))
}

pub async fn public_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Query(query): Query<PublicProfileQuery>,
) -> Result<Json<PublicUserProfile>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let profile = public_user_profile(
        pool,
        &username,
        actor.map(|user| user.id),
        &state.config.app_url,
        query.year,
    )
    .await
    .map_err(map_profile_error)?;

    Ok(Json(profile))
}

pub async fn public_repositories(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Query(query): Query<PublicRepositoriesQuery>,
) -> Result<Json<ProfileRepositoryList>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repositories = profile_repositories(
        pool,
        &username,
        actor.map(|user| user.id),
        ProfileRepositoryListQuery {
            query: query.q.as_deref(),
            repository_type: query.repository_type.as_deref(),
            language: query.language.as_deref(),
            sort: query.sort.as_deref(),
            page: query.page,
            page_size: query.page_size,
        },
    )
    .await
    .map_err(map_profile_error)?;

    Ok(Json(repositories))
}

pub async fn public_stars(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Query(query): Query<PublicStarsQuery>,
) -> Result<Json<ProfileRepositoryList>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repositories = starred_repositories(
        pool,
        &username,
        actor.map(|user| user.id),
        ProfileRepositoryListQuery {
            query: query.q.as_deref(),
            repository_type: None,
            language: query.language.as_deref(),
            sort: query.sort.as_deref(),
            page: query.page,
            page_size: query.page_size,
        },
    )
    .await
    .map_err(map_profile_error)?;

    Ok(Json(repositories))
}

async fn personal_profile_settings_route(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PersonalProfileSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings = personal_profile_settings(pool, actor.id)
        .await
        .map_err(map_personal_settings_error)?;
    Ok(Json(settings))
}

async fn update_personal_profile_settings_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    RestJson(request): RestJson<UpdatePersonalProfileSettings>,
) -> Result<Json<PersonalProfileSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings = update_personal_profile_settings(pool, actor.id, request)
        .await
        .map_err(map_personal_settings_error)?;
    Ok(Json(settings))
}

async fn update_personal_avatar_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    RestJson(request): RestJson<UpdateAvatarInput>,
) -> Result<Json<PersonalProfileSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings = update_personal_avatar(pool, actor.id, request)
        .await
        .map_err(map_personal_settings_error)?;
    Ok(Json(settings))
}

#[derive(Debug, Clone, Deserialize)]
pub struct PublicProfileQuery {
    pub year: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicRepositoriesQuery {
    pub q: Option<String>,
    #[serde(rename = "type")]
    pub repository_type: Option<String>,
    pub language: Option<String>,
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicStarsQuery {
    pub q: Option<String>,
    pub language: Option<String>,
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockUserRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportUserRequest {
    pub reason: String,
    pub details: Option<String>,
}

async fn follow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(username): Path<String>,
) -> Result<Json<ProfileActionState>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let state = follow_user(pool, actor.id, &username)
        .await
        .map_err(map_profile_error)?;
    Ok(Json(state))
}

async fn unfollow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(username): Path<String>,
) -> Result<Json<ProfileActionState>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let state = unfollow_user(pool, actor.id, &username)
        .await
        .map_err(map_profile_error)?;
    Ok(Json(state))
}

async fn block(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(username): Path<String>,
    RestJson(request): RestJson<BlockUserRequest>,
) -> Result<Json<ProfileActionState>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let state = block_user(pool, actor.id, &username, request.reason.as_deref())
        .await
        .map_err(map_profile_error)?;
    Ok(Json(state))
}

async fn report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(username): Path<String>,
    RestJson(request): RestJson<ReportUserRequest>,
) -> Result<Json<ProfileReport>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let report = report_user(
        pool,
        actor.id,
        &username,
        &request.reason,
        request.details.as_deref(),
    )
    .await
    .map_err(map_profile_error)?;
    Ok(Json(report))
}

fn fallback_login_from_email(email: &str) -> String {
    let local_part = email.split('@').next().unwrap_or("user");
    let normalized: String = local_part
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = normalized.trim_matches('-');
    if trimmed.is_empty() {
        "user".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn map_profile_error(error: ProfileError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        ProfileError::NotFound => error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "user profile was not found",
        ),
        ProfileError::SelfAction => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "profile action cannot target your own account",
        ),
        ProfileError::PrivateProfile => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "profile action is not available for private profiles",
        ),
        ProfileError::BlankReportReason => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "report reason is required",
        ),
        ProfileError::InvalidRepositoryFilter(message) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            message,
        ),
        ProfileError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "profile could not be loaded",
        ),
    }
}

fn map_personal_settings_error(error: PersonalSettingsError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        PersonalSettingsError::Validation(message) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            message,
        ),
        PersonalSettingsError::EmailNotFound => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "public email must belong to the signed-in user",
        ),
        PersonalSettingsError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "profile settings could not be saved",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::fallback_login_from_email;

    #[test]
    fn fallback_login_is_url_safe_and_never_empty() {
        assert_eq!(
            fallback_login_from_email("Octo.Cat@example.test"),
            "octo-cat"
        );
        assert_eq!(fallback_login_from_email("@example.test"), "user");
    }
}
