use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope},
    auth::extractor::AuthenticatedUser,
    domain::organizations::{
        public_organization_profile, OrganizationProfileError, PublicOrganizationProfile,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/orgs/:org/profile", get(public_profile))
}

async fn public_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org): Path<String>,
) -> Result<Json<PublicOrganizationProfile>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let profile = public_organization_profile(pool, &org, actor.map(|user| user.id))
        .await
        .map_err(map_organization_profile_error)?;

    Ok(Json(profile))
}

fn map_organization_profile_error(
    error: OrganizationProfileError,
) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        OrganizationProfileError::NotFound => error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "organization profile was not found",
        ),
        OrganizationProfileError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "organization profile could not be loaded",
        ),
    }
}
