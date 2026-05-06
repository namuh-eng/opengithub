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
            account_security_log, account_security_log_export, account_security_settings,
            create_account_security_sudo_grant, require_account_security_sudo,
            revoke_account_session, sign_out_everywhere, unlink_sign_in_method,
            update_current_session_metadata, AccountSecurityError, AccountSecurityLog,
            AccountSecurityLogQuery, AccountSecuritySettings, AccountSessions,
            RevokeAccountSessionResponse, SignOutEverywhereResponse, UnlinkSignInMethodResponse,
        },
        tokens::CreateSudoGrantRequest,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/settings/security", get(settings))
        .route("/api/settings/security-log", get(security_log))
        .route(
            "/api/settings/security-log/export",
            get(export_security_log),
        )
        .route("/api/settings/security/sessions", get(sessions))
        .route(
            "/api/settings/security/sessions/sign-out-everywhere",
            post(sign_out_elsewhere),
        )
        .route(
            "/api/settings/security/sessions/:session_id",
            delete(revoke_session),
        )
        .route("/api/settings/security/sudo", post(create_sudo))
        .route(
            "/api/settings/security/sign-in-methods/:account_id",
            delete(unlink_method),
        )
        .route("/api/settings/security/google/link", get(link_google))
}

#[derive(Debug, Deserialize)]
struct SecurityLogQuery {
    action: Option<String>,
    page: Option<i64>,
    #[serde(rename = "pageSize")]
    page_size: Option<i64>,
}

async fn security_log(
    State(state): State<AppState>,
    Query(query): Query<SecurityLogQuery>,
    headers: HeaderMap,
) -> Result<Json<AccountSecurityLog>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let response = account_security_log(
        pool,
        actor.id,
        AccountSecurityLogQuery::normalized(query.action, query.page, query.page_size),
    )
    .await
    .map_err(map_account_security_error)?;
    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
struct SecurityLogExportQuery {
    action: Option<String>,
    format: Option<String>,
}

async fn export_security_log(
    State(state): State<AppState>,
    Query(query): Query<SecurityLogExportQuery>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let format = query
        .format
        .as_deref()
        .unwrap_or("csv")
        .trim()
        .to_ascii_lowercase();
    let (body, content_type, filename) =
        account_security_log_export(pool, actor.id, query.action, &format)
            .await
            .map_err(map_account_security_error)?;
    let mut response = Response::new(axum::body::Body::from(body));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&content_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")).map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "security_log_export_failed",
                "Security log export could not be prepared",
            )
        })?,
    );
    Ok(response)
}

async fn sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AccountSessions>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let session_id = current_session_id(&state, &headers)?;
    update_current_session_metadata(
        pool,
        actor.id,
        &session_id,
        header_str(&headers, header::USER_AGENT.as_str()),
        client_ip(&headers).as_deref(),
    )
    .await
    .map_err(map_account_security_error)?;
    let response = crate::domain::account_security::account_sessions(pool, actor.id, &session_id)
        .await
        .map_err(map_account_security_error)?;
    Ok(Json(response))
}

async fn revoke_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<RevokeAccountSessionResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let current_session_id = current_session_id(&state, &headers)?;
    let response = revoke_account_session(pool, actor.id, &current_session_id, &session_id)
        .await
        .map_err(map_account_security_error)?;
    Ok(Json(response))
}

async fn sign_out_elsewhere(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<SignOutEverywhereResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let current_session_id = current_session_id(&state, &headers)?;
    let response = sign_out_everywhere(pool, actor.id, &current_session_id)
        .await
        .map_err(map_account_security_error)?;
    Ok(Json(response))
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
        AccountSecurityError::SessionNotFound => error_response(
            StatusCode::NOT_FOUND,
            "session_not_found",
            "The selected session is not available",
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

fn current_session_id(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<String, (StatusCode, Json<ErrorEnvelope>)> {
    session::session_id_from_headers(&state.config, headers)
        .map_err(|_| unauthorized_for_security_settings())?
        .ok_or_else(unauthorized_for_security_settings)
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

fn client_ip(headers: &HeaderMap) -> Option<String> {
    header_str(headers, "x-forwarded-for")
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| header_str(headers, "x-real-ip"))
        .map(str::to_owned)
}
