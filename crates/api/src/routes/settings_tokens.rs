use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope, RestJson},
    auth::{extractor::AuthenticatedUser, session},
    domain::tokens::{
        create_sudo_grant, personal_access_token_list, personal_access_token_new_context,
        CreateSudoGrantRequest, PersonalAccessTokenError, PersonalAccessTokenList,
        PersonalAccessTokenNewContext, SudoGrantResponse,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/settings/tokens", get(list_tokens))
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

fn map_token_error(error: PersonalAccessTokenError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        PersonalAccessTokenError::InvalidSudoConfirmation => error_response(
            StatusCode::FORBIDDEN,
            "sudo_confirmation_failed",
            "Sudo confirmation did not match the current account",
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
