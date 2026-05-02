use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope},
    auth::extractor::AuthenticatedUser,
    domain::organizations::{
        organization_repositories, public_organization_profile, OrganizationProfileError,
        OrganizationRepositoryList, OrganizationRepositoryListQuery, PublicOrganizationProfile,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/orgs/:org/profile", get(public_profile))
        .route("/api/orgs/:org/repositories", get(public_repositories))
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

async fn public_repositories(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org): Path<String>,
    Query(query): Query<OrganizationRepositoriesQuery>,
) -> Result<Json<OrganizationRepositoryList>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repositories = organization_repositories(
        pool,
        &org,
        actor.map(|user| user.id),
        OrganizationRepositoryListQuery {
            query: query.q.as_deref(),
            repository_type: query.repository_type.as_deref(),
            language: query.language.as_deref(),
            sort: query.sort.as_deref(),
            density: query.density.as_deref(),
            page: query.page,
            page_size: query.page_size,
        },
    )
    .await
    .map_err(map_organization_profile_error)?;

    Ok(Json(repositories))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrganizationRepositoriesQuery {
    q: Option<String>,
    #[serde(rename = "type")]
    repository_type: Option<String>,
    language: Option<String>,
    sort: Option<String>,
    density: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
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
        OrganizationProfileError::InvalidRepositoryFilter(message) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            message,
        ),
        OrganizationProfileError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "organization profile could not be loaded",
        ),
    }
}
