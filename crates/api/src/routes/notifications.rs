use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope},
    auth::extractor::AuthenticatedUser,
    domain::notifications::{
        bulk_triage_notifications, create_notification_custom_filter,
        delete_notification_custom_filter, notification_delivery_settings,
        notification_filter_settings, notification_inbox_view, triage_notification,
        update_notification_custom_filter, update_notification_delivery_settings,
        NotificationError, NotificationInboxQuery, NotificationInboxView, NotificationTriageAction,
        UpdateNotificationDeliverySettings, UpsertNotificationCustomFilter,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/notifications", get(list))
        .route("/api/notifications/bulk", post(bulk))
        .route(
            "/api/notifications/custom-filters",
            get(filter_settings).post(create_filter),
        )
        .route(
            "/api/notifications/delivery-preferences",
            get(delivery_settings).patch(update_delivery_settings),
        )
        .route(
            "/api/notifications/custom-filters/:id",
            patch(update_filter).delete(delete_filter),
        )
        .route("/api/notifications/:id/read", patch(mark_read))
        .route("/api/notifications/:id/unread", patch(mark_unread))
        .route("/api/notifications/:id/save", patch(save))
        .route("/api/notifications/:id/unsave", patch(unsave))
        .route("/api/notifications/:id/done", patch(done))
        .route("/api/notifications/:id/inbox", patch(move_to_inbox))
        .route("/api/notifications/:id/subscribe", patch(subscribe))
        .route("/api/notifications/:id/unsubscribe", patch(unsubscribe))
}

async fn delivery_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let response = notification_delivery_settings(pool, actor.0.id)
        .await
        .map_err(map_notification_error)?;
    Ok(Json(
        serde_json::to_value(response).expect("delivery settings should serialize"),
    ))
}

async fn update_delivery_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UpdateNotificationDeliverySettings>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let response = update_notification_delivery_settings(pool, actor.0.id, body)
        .await
        .map_err(map_notification_error)?;
    Ok(Json(
        serde_json::to_value(response).expect("delivery settings should serialize"),
    ))
}

async fn filter_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let response = notification_filter_settings(pool, actor.0.id)
        .await
        .map_err(map_notification_error)?;
    Ok(Json(
        serde_json::to_value(response).expect("filter settings should serialize"),
    ))
}

async fn create_filter(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UpsertNotificationCustomFilter>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let response = create_notification_custom_filter(pool, actor.0.id, body)
        .await
        .map_err(map_notification_error)?;
    Ok(Json(
        serde_json::to_value(response).expect("filter settings should serialize"),
    ))
}

async fn update_filter(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<UpsertNotificationCustomFilter>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let response = update_notification_custom_filter(pool, actor.0.id, id, body)
        .await
        .map_err(map_notification_error)?;
    Ok(Json(
        serde_json::to_value(response).expect("filter settings should serialize"),
    ))
}

async fn delete_filter(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let response = delete_notification_custom_filter(pool, actor.0.id, id)
        .await
        .map_err(map_notification_error)?;
    Ok(Json(
        serde_json::to_value(response).expect("filter settings should serialize"),
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BulkTriageBody {
    notification_ids: Vec<Uuid>,
    action: NotificationTriageActionBody,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NotificationTriageActionBody {
    Read,
    Unread,
    Save,
    Unsave,
    Done,
    Inbox,
    Subscribe,
    Unsubscribe,
}

impl From<NotificationTriageActionBody> for NotificationTriageAction {
    fn from(action: NotificationTriageActionBody) -> Self {
        match action {
            NotificationTriageActionBody::Read => Self::Read,
            NotificationTriageActionBody::Unread => Self::Unread,
            NotificationTriageActionBody::Save => Self::Save,
            NotificationTriageActionBody::Unsave => Self::Unsave,
            NotificationTriageActionBody::Done => Self::Done,
            NotificationTriageActionBody::Inbox => Self::Inbox,
            NotificationTriageActionBody::Subscribe => Self::Subscribe,
            NotificationTriageActionBody::Unsubscribe => Self::Unsubscribe,
        }
    }
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
    triage(state, headers, id, NotificationTriageAction::Read).await
}

async fn mark_unread(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    triage(state, headers, id, NotificationTriageAction::Unread).await
}

async fn save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    triage(state, headers, id, NotificationTriageAction::Save).await
}

async fn unsave(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    triage(state, headers, id, NotificationTriageAction::Unsave).await
}

async fn done(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    triage(state, headers, id, NotificationTriageAction::Done).await
}

async fn move_to_inbox(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    triage(state, headers, id, NotificationTriageAction::Inbox).await
}

async fn subscribe(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    triage(state, headers, id, NotificationTriageAction::Subscribe).await
}

async fn unsubscribe(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    triage(state, headers, id, NotificationTriageAction::Unsubscribe).await
}

async fn bulk(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<BulkTriageBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    validate_bulk_notification_ids(&body.notification_ids)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let response = bulk_triage_notifications(
        pool,
        actor.0.id,
        body.notification_ids,
        NotificationTriageAction::from(body.action),
    )
    .await
    .map_err(map_notification_error)?;
    Ok(Json(
        serde_json::to_value(response).expect("bulk triage response should serialize"),
    ))
}

fn validate_bulk_notification_ids(
    notification_ids: &[Uuid],
) -> Result<(), (StatusCode, Json<ErrorEnvelope>)> {
    if notification_ids.is_empty() {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "notificationIds must contain at least one notification.",
        ));
    }
    if notification_ids.len() > 100 {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "notificationIds cannot contain more than 100 notifications.",
        ));
    }
    let unique_count = notification_ids
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    if unique_count != notification_ids.len() {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "notificationIds cannot contain duplicates.",
        ));
    }
    Ok(())
}

async fn triage(
    state: AppState,
    headers: HeaderMap,
    id: Uuid,
    action: NotificationTriageAction,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let response = triage_notification(pool, id, actor.0.id, action)
        .await
        .map_err(map_notification_error)?;
    Ok(Json(
        serde_json::to_value(response).expect("triage response should serialize"),
    ))
}

fn map_notification_error(error: NotificationError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        NotificationError::NotFound => error_response(
            StatusCode::NOT_FOUND,
            "notification_not_found",
            "Notification was not found.",
        ),
        NotificationError::Validation(message) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            &message,
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
