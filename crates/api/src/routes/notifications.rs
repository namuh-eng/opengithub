use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, patch},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope},
    auth::extractor::AuthenticatedUser,
    domain::notifications::{
        mark_notification_read, notification_inbox_view, NotificationError, NotificationInboxQuery,
        NotificationInboxView,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/notifications", get(list))
        .route("/api/notifications/:id/read", patch(mark_read))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQuery {
    q: Option<String>,
    folder: Option<String>,
    tab: Option<String>,
    sort: Option<String>,
    group: Option<String>,
    repo: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<NotificationInboxView>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = notification_inbox_view(
        pool,
        actor.0.id,
        NotificationInboxQuery {
            q: query.q,
            folder: query.folder,
            tab: query.tab,
            sort: query.sort,
            group: query.group,
            repo: query.repo,
            page: query.page,
            page_size: query.page_size,
        },
    )
    .await
    .map_err(map_notification_error)?;
    Ok(Json(view))
}

async fn mark_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let notification = mark_notification_read(pool, id, actor.0.id)
        .await
        .map_err(map_notification_error)?;
    Ok(Json(serde_json::json!({
        "id": notification.id,
        "unread": notification.unread,
        "lastReadAt": notification.last_read_at,
    })))
}

fn map_notification_error(error: NotificationError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        NotificationError::NotFound => error_response(
            StatusCode::NOT_FOUND,
            "notification_not_found",
            "Notification was not found.",
        ),
        NotificationError::Sqlx(sqlx::Error::PoolTimedOut)
        | NotificationError::Sqlx(sqlx::Error::PoolClosed) => database_unavailable(),
        other => {
            tracing::warn!(error = %other, "notification inbox operation failed");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "notifications_failed",
                "Notifications could not be loaded.",
            )
        }
    }
}
