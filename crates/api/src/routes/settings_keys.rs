use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, patch, post},
    Json, Router,
};

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope, RestJson},
    auth::{extractor::AuthenticatedUser, session},
    domain::signing_keys::{
        create_gpg_key, create_ssh_key, key_settings, revoke_gpg_key, revoke_ssh_key,
        update_vigilant_mode, CreateGpgKeyRequest, CreateGpgKeyResponse, CreateSshKeyRequest,
        CreateSshKeyResponse, KeySettings, RevokeGpgKeyResponse, RevokeSshKeyResponse,
        SigningKeyError, UpdateVigilantModeRequest, UpdateVigilantModeResponse,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/settings/keys", get(list_keys))
        .route("/api/settings/keys/ssh", post(create_ssh))
        .route("/api/settings/keys/ssh/:key_id", delete(revoke_ssh))
        .route("/api/settings/keys/gpg", post(create_gpg))
        .route("/api/settings/keys/gpg/:key_id", delete(revoke_gpg))
        .route("/api/settings/keys/vigilant-mode", patch(update_vigilant))
}

async fn list_keys(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<KeySettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let session_id = session::session_id_from_headers(&state.config, &headers)
        .map_err(|_| unauthorized_for_key_settings())?;
    let response = key_settings(pool, actor.id, session_id.as_deref())
        .await
        .map_err(map_key_error)?;
    Ok(Json(response))
}

async fn create_ssh(
    State(state): State<AppState>,
    headers: HeaderMap,
    RestJson(request): RestJson<CreateSshKeyRequest>,
) -> Result<Json<CreateSshKeyResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let response = create_ssh_key(pool, actor.id, request)
        .await
        .map_err(map_key_error)?;
    Ok(Json(response))
}

async fn revoke_ssh(
    State(state): State<AppState>,
    Path(key_id): Path<uuid::Uuid>,
    headers: HeaderMap,
) -> Result<Json<RevokeSshKeyResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let session_id = session::session_id_from_headers(&state.config, &headers)
        .map_err(|_| unauthorized_for_key_settings())?
        .ok_or_else(unauthorized_for_key_settings)?;
    let response = revoke_ssh_key(pool, actor.id, &session_id, key_id)
        .await
        .map_err(map_key_error)?;
    Ok(Json(response))
}

async fn create_gpg(
    State(state): State<AppState>,
    headers: HeaderMap,
    RestJson(request): RestJson<CreateGpgKeyRequest>,
) -> Result<Json<CreateGpgKeyResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let response = create_gpg_key(pool, actor.id, request)
        .await
        .map_err(map_key_error)?;
    Ok(Json(response))
}

async fn revoke_gpg(
    State(state): State<AppState>,
    Path(key_id): Path<uuid::Uuid>,
    headers: HeaderMap,
) -> Result<Json<RevokeGpgKeyResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let session_id = session::session_id_from_headers(&state.config, &headers)
        .map_err(|_| unauthorized_for_key_settings())?
        .ok_or_else(unauthorized_for_key_settings)?;
    let response = revoke_gpg_key(pool, actor.id, &session_id, key_id)
        .await
        .map_err(map_key_error)?;
    Ok(Json(response))
}

async fn update_vigilant(
    State(state): State<AppState>,
    headers: HeaderMap,
    RestJson(request): RestJson<UpdateVigilantModeRequest>,
) -> Result<Json<UpdateVigilantModeResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let response = update_vigilant_mode(pool, actor.id, request)
        .await
        .map_err(map_key_error)?;
    Ok(Json(response))
}

fn map_key_error(error: SigningKeyError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        SigningKeyError::SudoRequired => error_response(
            StatusCode::FORBIDDEN,
            "sudo_required",
            "Sudo mode is required before revoking signing keys",
        ),
        SigningKeyError::Validation(message) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            &message,
        ),
        SigningKeyError::NotFound => error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "Signing key was not found",
        ),
        SigningKeyError::Sqlx(error) => {
            tracing::warn!(%error, "signing key settings operation failed");
            database_unavailable()
        }
    }
}

fn unauthorized_for_key_settings() -> (StatusCode, Json<ErrorEnvelope>) {
    error_response(
        StatusCode::UNAUTHORIZED,
        "not_authenticated",
        "No active session is available",
    )
}
