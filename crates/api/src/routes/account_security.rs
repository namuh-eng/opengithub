use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::Response,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope, RestJson},
    auth::{self, extractor::AuthenticatedUser, session},
    domain::{
        account_security::{
            account_security_settings, create_account_security_sudo_grant,
            require_account_security_sudo, unlink_sign_in_method, AccountSecurityError,
            AccountSecuritySettings, UnlinkSignInMethodResponse,
        },
        tokens::CreateSudoGrantRequest,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/settings/security", get(settings))
        .route("/api/settings/security/sudo", post(create_sudo))
        .route(
            "/api/settings/security/sign-in-methods/:account_id",
            delete(unlink_method),
        )
        .route("/api/settings/security/google/link", get(link_google))
}

#[derive(Debug, Deserialize)]
struct LinkGoogleQuery {
    next: Option<String>,
}

async fn settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AccountSecuritySettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let session_id = session::session_id_from_headers(&state.config, &headers)
        .map_err(|_| unauthorized_for_security_settings())?;
    let response = account_security_settings(pool, actor.id, session_id.as_deref())
        .await
        .map_err(map_account_security_error)?;
    Ok(Json(response))
}

async fn create_sudo(
    State(state): State<AppState>,
    headers: HeaderMap,
    RestJson(request): RestJson<CreateSudoGrantRequest>,
) -> Result<Json<AccountSecuritySettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let session_id = session::session_id_from_headers(&state.config, &headers)
        .map_err(|_| unauthorized_for_security_settings())?
        .ok_or_else(unauthorized_for_security_settings)?;
    let response = create_account_security_sudo_grant(pool, actor.id, &session_id, request)
        .await
        .map_err(map_account_security_error)?;
    Ok(Json(response))
}

async fn unlink_method(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<UnlinkSignInMethodResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let session_id = session::session_id_from_headers(&state.config, &headers)
        .map_err(|_| unauthorized_for_security_settings())?
        .ok_or_else(unauthorized_for_security_settings)?;
    let response = unlink_sign_in_method(pool, actor.id, &session_id, account_id)
        .await
        .map_err(map_account_security_error)?;
    Ok(Json(response))
}

async fn link_google(
    State(state): State<AppState>,
    Query(query): Query<LinkGoogleQuery>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let session_id = session::session_id_from_headers(&state.config, &headers)
        .map_err(|_| unauthorized_for_security_settings())?
        .ok_or_else(unauthorized_for_security_settings)?;
    require_account_security_sudo(pool, actor.id, &session_id)
        .await
        .map_err(map_account_security_error)?;

    let next = query.next.as_deref().unwrap_or("/settings/security");
    let start = auth::google_authorization_url(&state.config, Some(next)).map_err(|_| {
        error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "auth_not_configured",
            "Google OAuth is not configured",
        )
    })?;
    let mut response = Response::new(axum::body::Body::empty());
    *response.status_mut() = StatusCode::FOUND;
    response.headers_mut().insert(
        header::LOCATION,
        HeaderValue::from_str(start.authorization_url.as_str()).map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "auth_start_failed",
                "OAuth link flow could not be started",
            )
        })?,
    );
    Ok(response)
}

fn map_account_security_error(error: AccountSecurityError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        AccountSecurityError::InvalidSudoConfirmation => error_response(
            StatusCode::FORBIDDEN,
            "sudo_confirmation_failed",
            "Sudo confirmation did not match the current account",
        ),
        AccountSecurityError::SudoRequired => error_response(
            StatusCode::FORBIDDEN,
            "sudo_required",
            "Sudo mode is required before changing sign-in methods",
        ),
        AccountSecurityError::LastIdentity => error_response(
            StatusCode::CONFLICT,
            "last_identity",
            "The last sign-in method cannot be removed",
        ),
        AccountSecurityError::Forbidden => error_response(
            StatusCode::FORBIDDEN,
            "sign_in_method_forbidden",
            "The selected sign-in method is not available",
        ),
        AccountSecurityError::Sqlx(error) => {
            tracing::warn!(%error, "account security operation failed");
            database_unavailable()
        }
    }
}

fn unauthorized_for_security_settings() -> (StatusCode, Json<ErrorEnvelope>) {
    error_response(
        StatusCode::UNAUTHORIZED,
        "not_authenticated",
        "No active session is available",
    )
}
