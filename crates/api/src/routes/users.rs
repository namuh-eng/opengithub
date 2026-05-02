use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope},
    auth::extractor::AuthenticatedUser,
    domain::{
        identity::User,
        profiles::{public_user_profile, ProfileError, PublicUserProfile},
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/user", get(current_api_user))
        .route("/api/users/:username/profile", get(public_profile))
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
) -> Result<Json<PublicUserProfile>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let profile = public_user_profile(
        pool,
        &username,
        actor.map(|user| user.id),
        &state.config.app_url,
    )
    .await
    .map_err(map_profile_error)?;

    Ok(Json(profile))
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
        ProfileError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "profile could not be loaded",
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
