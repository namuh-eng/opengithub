use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
    Json, Router,
};

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope, RestJson},
    auth::{extractor::AuthenticatedUser, session},
    domain::tokens::{
        create_personal_access_token, create_sudo_grant, personal_access_token_list,
        personal_access_token_new_context, revoke_personal_access_token,
        CreatePersonalAccessTokenRequest, CreatePersonalAccessTokenResponse,
        CreateSudoGrantRequest, PersonalAccessTokenError, PersonalAccessTokenList,
        PersonalAccessTokenNewContext, RevokePersonalAccessTokenResponse, SudoGrantResponse,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/settings/tokens", get(list_tokens).post(create_token))
        .route("/api/settings/tokens/:token_id", delete(revoke_token))
        .route("/api/settings/tokens/new", get(new_token_context))
        .route("/api/settings/sudo", post(create_sudo))
}

async fn list_tokens(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PersonalAccessTokenList>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let session_id = session::session_id_from_headers(&state.config, &headers)
        .map_err(|_| unauthorized_for_token_settings())?;
    let response = personal_access_token_list(pool, actor.id, session_id.as_deref())
        .await
        .map_err(map_token_error)?;
    Ok(Json(response))
}

async fn new_token_context(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PersonalAccessTokenNewContext>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let session_id = session::session_id_from_headers(&state.config, &headers)
        .map_err(|_| unauthorized_for_token_settings())?;
    let response = personal_access_token_new_context(pool, actor.id, session_id.as_deref())
        .await
        .map_err(map_token_error)?;
    Ok(Json(response))
}

async fn create_sudo(
    State(state): State<AppState>,
    headers: HeaderMap,
    RestJson(request): RestJson<CreateSudoGrantRequest>,
) -> Result<Json<SudoGrantResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let session_id = session::session_id_from_headers(&state.config, &headers)
        .map_err(|_| unauthorized_for_token_settings())?
        .ok_or_else(unauthorized_for_token_settings)?;
    let response = create_sudo_grant(pool, actor.id, &session_id, request)
        .await
        .map_err(map_token_error)?;
    Ok(Json(response))
}

async fn create_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    RestJson(request): RestJson<CreatePersonalAccessTokenRequest>,
) -> Result<Json<CreatePersonalAccessTokenResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let session_id = session::session_id_from_headers(&state.config, &headers)
        .map_err(|_| unauthorized_for_token_settings())?
        .ok_or_else(unauthorized_for_token_settings)?;
    let response = create_personal_access_token(pool, actor.id, &session_id, request)
        .await
        .map_err(map_token_error)?;
    Ok(Json(response))
}

async fn revoke_token(
    State(state): State<AppState>,
    Path(token_id): Path<uuid::Uuid>,
    headers: HeaderMap,
) -> Result<Json<RevokePersonalAccessTokenResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let session_id = session::session_id_from_headers(&state.config, &headers)
        .map_err(|_| unauthorized_for_token_settings())?
        .ok_or_else(unauthorized_for_token_settings)?;
    let response = revoke_personal_access_token(pool, actor.id, &session_id, token_id)
        .await
        .map_err(map_token_error)?;
    Ok(Json(response))
}

fn map_token_error(error: PersonalAccessTokenError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        PersonalAccessTokenError::InvalidSudoConfirmation => error_response(
            StatusCode::FORBIDDEN,
            "sudo_confirmation_failed",
            "Sudo confirmation did not match the current account",
        ),
        PersonalAccessTokenError::SudoRequired => error_response(
            StatusCode::FORBIDDEN,
            "sudo_required",
            "Sudo mode is required before creating or revoking tokens",
        ),
        PersonalAccessTokenError::Validation(message) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            &message,
        ),
        PersonalAccessTokenError::Forbidden => error_response(
            StatusCode::FORBIDDEN,
            "resource_owner_forbidden",
            "The selected resource owner is not available",
        ),
        PersonalAccessTokenError::Invalid => error_response(
            StatusCode::UNAUTHORIZED,
            "invalid_token",
            "Personal access token is invalid",
        ),
        PersonalAccessTokenError::Sqlx(error) => {
            tracing::warn!(%error, "token settings operation failed");
            database_unavailable()
        }
    }
}

fn unauthorized_for_token_settings() -> (StatusCode, Json<ErrorEnvelope>) {
    error_response(
        StatusCode::UNAUTHORIZED,
        "not_authenticated",
        "No active session is available",
    )
}
