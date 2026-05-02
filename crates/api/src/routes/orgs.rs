use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope},
    auth::extractor::AuthenticatedUser,
    domain::{orgs::organization_overview_for_viewer, repositories::RepositoryError},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/orgs/:org", get(read))
}

async fn read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let overview = organization_overview_for_viewer(pool, &org, actor.map(|user| user.id))
        .await
        .map_err(map_organization_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "organization_not_found",
                "Organization was not found",
            )
        })?;

    Ok(Json(serde_json::json!(overview)))
}

fn map_organization_error(error: RepositoryError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        RepositoryError::PermissionDenied => error_response(
            StatusCode::FORBIDDEN,
            "forbidden",
            "You do not have access to this organization",
        ),
        RepositoryError::Sqlx(sqlx::Error::PoolTimedOut)
        | RepositoryError::Sqlx(sqlx::Error::PoolClosed) => database_unavailable(),
        other => {
            tracing::warn!(error = %other, "organization overview operation failed");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "organization_overview_failed",
                "Unable to load organization overview",
            )
        }
    }
}
