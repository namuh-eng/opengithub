use axum::{
    http::{header, HeaderMap},
    Json,
};

use crate::{
    api_types::{database_unavailable, unauthorized, ErrorEnvelope},
    auth::session,
    domain::{
        identity::{AuthUser, User},
        tokens::{verify_personal_access_token, PersonalAccessTokenError},
    },
    AppState,
};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub User);

impl AuthenticatedUser {
    pub async fn from_headers(
        state: &AppState,
        headers: &HeaderMap,
    ) -> Result<Self, (axum::http::StatusCode, Json<ErrorEnvelope>)> {
        if let Some(user) = bearer_user_from_headers(state, headers).await? {
            return Ok(Self(user));
        }
        let user =
            session::require_current_user_from_headers(state.db.as_ref(), &state.config, headers)
                .await
                .map_err(map_verification_error)?;
        Ok(Self(user))
    }

    pub fn into_auth_user(self) -> AuthUser {
        AuthUser::from(self.0)
    }

    pub async fn optional_from_headers(
        state: &AppState,
        headers: &HeaderMap,
    ) -> Result<Option<User>, (axum::http::StatusCode, Json<ErrorEnvelope>)> {
        let Some(pool) = state.db.as_ref() else {
            return Ok(None);
        };
        if let Some(user) = bearer_user_from_headers(state, headers).await? {
            return Ok(Some(user));
        }
        match session::current_user_from_headers(pool, &state.config, headers).await {
            Ok(user) => Ok(user),
            Err(
                session::SessionError::MissingConfig
                | session::SessionError::InvalidCookie
                | session::SessionError::Signing,
            ) => Ok(None),
            Err(session::SessionError::Database(error)) => Err(map_verification_error(
                session::SessionVerificationError::Database(error),
            )),
        }
    }
}

async fn bearer_user_from_headers(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<User>, (axum::http::StatusCode, Json<ErrorEnvelope>)> {
    let Some(token) = bearer_token(headers) else {
        return Ok(None);
    };
    let Some(pool) = state.db.as_ref() else {
        return Err(unauthorized());
    };
    let verified = match verify_personal_access_token(pool, &token).await {
        Ok(verified) => verified,
        Err(PersonalAccessTokenError::Sqlx(error)) => {
            return Err(map_verification_error(
                session::SessionVerificationError::Database(error),
            ));
        }
        Err(_) => return Err(unauthorized()),
    };
    if !verified
        .scopes
        .iter()
        .any(|scope| matches!(scope.as_str(), "api" | "api:read" | "api:write" | "repo"))
    {
        return Err(unauthorized());
    }
    let user = crate::domain::identity::get_user(pool, verified.user_id)
        .await
        .map_err(|error| {
            map_verification_error(session::SessionVerificationError::Database(error))
        })?
        .ok_or_else(unauthorized)?;
    Ok(Some(user))
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?.trim();
    value
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
}

pub fn map_verification_error(
    error: session::SessionVerificationError,
) -> (axum::http::StatusCode, Json<ErrorEnvelope>) {
    match error {
        session::SessionVerificationError::MissingDatabase
        | session::SessionVerificationError::Database(_) => database_unavailable(),
        session::SessionVerificationError::MissingCookie
        | session::SessionVerificationError::InvalidCookie
        | session::SessionVerificationError::MissingConfig
        | session::SessionVerificationError::NoActiveSession => unauthorized(),
    }
}
