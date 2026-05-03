use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, QueryBuilder, Row};
use uuid::Uuid;

use super::repositories::{
    can_read_repository, get_repository, get_repository_by_owner_name, RepositoryWatchEvent,
    RepositoryWatchLevel,
};
use crate::api_types::ListEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub repository_id: Option<Uuid>,
    pub subject_type: String,
    pub subject_id: Option<Uuid>,
    pub title: String,
    pub reason: String,
    pub unread: bool,
    pub saved: bool,
    pub done_at: Option<DateTime<Utc>>,
    pub last_read_at: Option<DateTime<Utc>>,
    pub saved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotification {
    pub user_id: Uuid,
    pub repository_id: Option<Uuid>,
    pub subject_type: String,
    pub subject_id: Option<Uuid>,
    pub title: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct NotificationDeliveryCheck {
    pub user_id: Uuid,
    pub repository_id: Uuid,
    pub subject_type: String,
    pub subject_id: Option<Uuid>,
    pub reason: String,
    pub repository_event: Option<RepositoryWatchEvent>,
    pub actor_user_id: Option<Uuid>,
    pub participating: bool,
    pub direct: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("notification was not found")]
    NotFound,
    #[error("{0}")]
    Validation(String),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationTriageAction {
    Read,
    Unread,
    Save,
    Unsave,
    Done,
    Inbox,
    Subscribe,
    Unsubscribe,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationTriageResponse {
    pub id: Uuid,
    pub unread: bool,
    pub saved: bool,
    pub done: bool,
    pub subscribed: bool,
    pub last_read_at: Option<DateTime<Utc>>,
    pub saved_at: Option<DateTime<Utc>>,
    pub unread_count: i64,
    pub folder_counts: NotificationFolderCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationFolderCounts {
    pub inbox: i64,
    pub saved: i64,
    pub done: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationBulkTriageResponse {
    pub action: String,
    pub updated: Vec<NotificationTriageResponse>,
    pub failed: Vec<NotificationBulkFailure>,
    pub unread_count: i64,
    pub folder_counts: NotificationFolderCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationBulkFailure {
    pub id: Uuid,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationCustomFilter {
    pub id: Uuid,
    pub name: String,
    pub query_string: String,
    pub position: i32,
    pub href: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationDefaultFilter {
    pub id: String,
    pub name: String,
    pub query_string: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationFilterSettings {
    pub default_filters: Vec<NotificationDefaultFilter>,
    pub custom_filters: Vec<NotificationCustomFilter>,
    pub limit: i64,
    pub remaining: i64,
    pub allowed_qualifiers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertNotificationCustomFilter {
    pub name: String,
    pub query_string: String,
}

pub async fn create_notification(
    pool: &PgPool,
    input: CreateNotification,
) -> Result<Notification, NotificationError> {
    let thread_id = ensure_notification_thread(
        pool,
        input.repository_id,
        &input.subject_type,
        input.subject_id,
        None,
    )
    .await?;
    reactivate_notification_subscription_if_needed(pool, thread_id, input.user_id, &input.reason)
        .await?;

    let row = sqlx::query(
        r#"
        INSERT INTO notifications (
            user_id, repository_id, thread_id, subject_type, subject_id, title, reason
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, user_id, repository_id, subject_type, subject_id, title, reason,
                  unread, saved, done_at, last_read_at, saved_at, created_at, updated_at
        "#,
    )
    .bind(input.user_id)
    .bind(input.repository_id)
    .bind(thread_id)
    .bind(&input.subject_type)
    .bind(input.subject_id)
    .bind(&input.title)
    .bind(&input.reason)
    .fetch_one(pool)
    .await?;

    Ok(notification_from_row(row))
}

pub async fn should_deliver_notification(
    pool: &PgPool,
    input: NotificationDeliveryCheck,
) -> Result<bool, NotificationError> {
    if input.actor_user_id == Some(input.user_id) {
        return Ok(false);
    }

    let Some(repository) = get_repository(pool, input.repository_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => NotificationError::Sqlx(error),
            other => NotificationError::Validation(other.to_string()),
        })?
    else {
        return Ok(false);
    };
    if !can_read_repository(pool, &repository, input.user_id)
        .await
        .map_err(|error| match error {
            super::repositories::RepositoryError::Sqlx(error) => NotificationError::Sqlx(error),
            other => NotificationError::Validation(other.to_string()),
        })?
    {
        return Ok(false);
    }

    if input.direct || direct_reactivation_reason(&input.reason) {
        return Ok(true);
    }

    if let Some(thread_decision) = thread_subscription_delivers(pool, &input).await? {
        return Ok(thread_decision);
    }

    if input.participating {
        return Ok(true);
    }

    let Some(repository_event) = input.repository_event else {
        return Ok(false);
    };
    repository_watch_delivers_event(pool, input.user_id, input.repository_id, repository_event)
        .await
}

pub async fn list_notifications(
    pool: &PgPool,
    user_id: Uuid,
    unread_only: bool,
    page: i64,
    page_size: i64,
) -> Result<ListEnvelope<Notification>, NotificationError> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;
    let unread_filter = if unread_only { Some(true) } else { None };

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM notifications
        WHERE user_id = $1 AND ($2::boolean IS NULL OR unread = $2)
        "#,
    )
    .bind(user_id)
    .bind(unread_filter)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT id, user_id, repository_id, subject_type, subject_id, title, reason,
               unread, saved, done_at, last_read_at, saved_at, created_at, updated_at
        FROM notifications
        WHERE user_id = $1 AND ($2::boolean IS NULL OR unread = $2)
        ORDER BY updated_at DESC, created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(user_id)
    .bind(unread_filter)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(ListEnvelope {
        items: rows.into_iter().map(notification_from_row).collect(),
        total,
        page,
        page_size,
    })
}

pub async fn unread_notification_count(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<i64, NotificationError> {
    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM notifications
        WHERE user_id = $1
          AND unread = true
          AND done_at IS NULL
          AND NOT EXISTS (
              SELECT 1 FROM notification_subscriptions
              WHERE notification_subscriptions.thread_id = notifications.thread_id
                AND notification_subscriptions.user_id = notifications.user_id
                AND notification_subscriptions.state = 'unsubscribed'
          )
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(total)
}

pub async fn notification_folder_counts(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<NotificationFolderCounts, NotificationError> {
    let row = sqlx::query(
        r#"
        SELECT count(*) FILTER (
                   WHERE done_at IS NULL
                     AND NOT EXISTS (
                         SELECT 1 FROM notification_subscriptions
                         WHERE notification_subscriptions.thread_id = notifications.thread_id
                           AND notification_subscriptions.user_id = notifications.user_id
                           AND notification_subscriptions.state = 'unsubscribed'
                     )
               ) AS inbox,
               count(*) FILTER (WHERE saved = true) AS saved,
               count(*) FILTER (WHERE done_at IS NOT NULL) AS done
        FROM notifications
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(NotificationFolderCounts {
        inbox: row.get("inbox"),
        saved: row.get("saved"),
        done: row.get("done"),
    })
}

pub async fn mark_notification_read(
    pool: &PgPool,
    notification_id: Uuid,
    user_id: Uuid,
) -> Result<Notification, NotificationError> {
    let row = sqlx::query(
        r#"
        UPDATE notifications
        SET unread = false, last_read_at = now()
        WHERE id = $1 AND user_id = $2
        RETURNING id, user_id, repository_id, subject_type, subject_id, title, reason,
                  unread, saved, done_at, last_read_at, saved_at, created_at, updated_at
        "#,
    )
    .bind(notification_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(NotificationError::NotFound)?;

    Ok(notification_from_row(row))
}

pub async fn triage_notification(
    pool: &PgPool,
    notification_id: Uuid,
    user_id: Uuid,
    action: NotificationTriageAction,
) -> Result<NotificationTriageResponse, NotificationError> {
    let row = match action {
        NotificationTriageAction::Read => {
            sqlx::query(
                r#"
                UPDATE notifications
                SET unread = false, last_read_at = now()
                WHERE id = $1 AND user_id = $2
                RETURNING id, unread, saved, done_at, last_read_at, saved_at
                "#,
            )
            .bind(notification_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?
        }
        NotificationTriageAction::Unread => {
            sqlx::query(
                r#"
                UPDATE notifications
                SET unread = true, last_read_at = NULL
                WHERE id = $1 AND user_id = $2
                RETURNING id, unread, saved, done_at, last_read_at, saved_at
                "#,
            )
            .bind(notification_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?
        }
        NotificationTriageAction::Save => {
            sqlx::query(
                r#"
                UPDATE notifications
                SET saved = true, saved_at = COALESCE(saved_at, now())
                WHERE id = $1 AND user_id = $2
                RETURNING id, unread, saved, done_at, last_read_at, saved_at
                "#,
            )
            .bind(notification_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?
        }
        NotificationTriageAction::Unsave => {
            sqlx::query(
                r#"
                UPDATE notifications
                SET saved = false, saved_at = NULL
                WHERE id = $1 AND user_id = $2
                RETURNING id, unread, saved, done_at, last_read_at, saved_at
                "#,
            )
            .bind(notification_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?
        }
        NotificationTriageAction::Done => {
            sqlx::query(
                r#"
                UPDATE notifications
                SET done_at = COALESCE(done_at, now()), updated_at = now()
                WHERE id = $1 AND user_id = $2
                RETURNING id, unread, saved, done_at, last_read_at, saved_at
                "#,
            )
            .bind(notification_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?
        }
        NotificationTriageAction::Inbox => {
            sqlx::query(
                r#"
                UPDATE notifications
                SET done_at = NULL, updated_at = now()
                WHERE id = $1 AND user_id = $2
                RETURNING id, unread, saved, done_at, last_read_at, saved_at
                "#,
            )
            .bind(notification_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?
        }
        NotificationTriageAction::Subscribe => {
            set_notification_subscription(pool, notification_id, user_id, "subscribed").await?;
            select_notification_triage_row(pool, notification_id, user_id).await?
        }
        NotificationTriageAction::Unsubscribe => {
            set_notification_subscription(pool, notification_id, user_id, "unsubscribed").await?;
            select_notification_triage_row(pool, notification_id, user_id).await?
        }
    }
    .ok_or(NotificationError::NotFound)?;

    let subscribed = notification_subscribed(pool, notification_id, user_id).await?;
    Ok(NotificationTriageResponse {
        id: row.get("id"),
        unread: row.get("unread"),
        saved: row.get("saved"),
        done: row.get::<Option<DateTime<Utc>>, _>("done_at").is_some(),
        subscribed,
        last_read_at: row.get("last_read_at"),
        saved_at: row.get("saved_at"),
        unread_count: unread_notification_count(pool, user_id).await?,
        folder_counts: notification_folder_counts(pool, user_id).await?,
    })
}

pub async fn bulk_triage_notifications(
    pool: &PgPool,
    user_id: Uuid,
    notification_ids: Vec<Uuid>,
    action: NotificationTriageAction,
) -> Result<NotificationBulkTriageResponse, NotificationError> {
    let mut updated = Vec::with_capacity(notification_ids.len());
    let mut failed = Vec::new();
    for notification_id in notification_ids {
        match triage_notification(pool, notification_id, user_id, action).await {
            Ok(response) => updated.push(response),
            Err(NotificationError::NotFound) => failed.push(NotificationBulkFailure {
                id: notification_id,
                code: "notification_not_found".to_owned(),
                message: "Notification was not found.".to_owned(),
            }),
            Err(error) => return Err(error),
        }
    }

    Ok(NotificationBulkTriageResponse {
        action: notification_action_name(action).to_owned(),
        updated,
        failed,
        unread_count: unread_notification_count(pool, user_id).await?,
        folder_counts: notification_folder_counts(pool, user_id).await?,
    })
}

pub async fn notification_filter_settings(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<NotificationFilterSettings, NotificationError> {
    let custom_filters = list_notification_custom_filters(pool, user_id).await?;
    let remaining = (15 - custom_filters.len() as i64).max(0);
    Ok(NotificationFilterSettings {
        default_filters: notification_default_filters()
            .into_iter()
            .map(|(id, name, query_string)| NotificationDefaultFilter {
                id: id.to_owned(),
                name: name.to_owned(),
                query_string: query_string.to_owned(),
                href: notification_href("inbox", "all", "newest", "date", query_string, None),
            })
            .collect(),
        custom_filters,
        limit: 15,
        remaining,
        allowed_qualifiers: ["repo", "org", "author", "is", "reason"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
    })
}

pub async fn list_notification_custom_filters(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<NotificationCustomFilter>, NotificationError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, query_string, position, created_at, updated_at
        FROM notification_custom_filters
        WHERE user_id = $1
        ORDER BY position ASC, created_at ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(custom_filter_from_row).collect())
}

pub async fn create_notification_custom_filter(
    pool: &PgPool,
    user_id: Uuid,
    input: UpsertNotificationCustomFilter,
) -> Result<NotificationFilterSettings, NotificationError> {
    let normalized = validate_custom_filter_input(pool, user_id, input).await?;
    let count = custom_filter_count(pool, user_id).await?;
    if count >= 15 {
        return Err(NotificationError::Validation(
            "You can create up to 15 custom notification filters.".to_owned(),
        ));
    }
    sqlx::query(
        r#"
        INSERT INTO notification_custom_filters (user_id, name, query_string, position)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(user_id)
    .bind(&normalized.name)
    .bind(&normalized.query_string)
    .bind((count + 1) as i32)
    .execute(pool)
    .await
    .map_err(map_custom_filter_write_error)?;

    notification_filter_settings(pool, user_id).await
}

pub async fn update_notification_custom_filter(
    pool: &PgPool,
    user_id: Uuid,
    filter_id: Uuid,
    input: UpsertNotificationCustomFilter,
) -> Result<NotificationFilterSettings, NotificationError> {
    let normalized = validate_custom_filter_input(pool, user_id, input).await?;
    let result = sqlx::query(
        r#"
        UPDATE notification_custom_filters
        SET name = $3, query_string = $4
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(filter_id)
    .bind(user_id)
    .bind(&normalized.name)
    .bind(&normalized.query_string)
    .execute(pool)
    .await
    .map_err(map_custom_filter_write_error)?;
    if result.rows_affected() == 0 {
        return Err(NotificationError::NotFound);
    }

    notification_filter_settings(pool, user_id).await
}

pub async fn delete_notification_custom_filter(
    pool: &PgPool,
    user_id: Uuid,
    filter_id: Uuid,
) -> Result<NotificationFilterSettings, NotificationError> {
    let deleted_position = sqlx::query_scalar::<_, i32>(
        r#"
        DELETE FROM notification_custom_filters
        WHERE id = $1 AND user_id = $2
        RETURNING position
        "#,
    )
    .bind(filter_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(NotificationError::NotFound)?;

    sqlx::query(
        r#"
        UPDATE notification_custom_filters
        SET position = position - 1
        WHERE user_id = $1 AND position > $2
        "#,
    )
    .bind(user_id)
    .bind(deleted_position)
    .execute(pool)
    .await?;

    notification_filter_settings(pool, user_id).await
}

fn notification_action_name(action: NotificationTriageAction) -> &'static str {
    match action {
        NotificationTriageAction::Read => "read",
        NotificationTriageAction::Unread => "unread",
        NotificationTriageAction::Save => "save",
        NotificationTriageAction::Unsave => "unsave",
        NotificationTriageAction::Done => "done",
        NotificationTriageAction::Inbox => "inbox",
        NotificationTriageAction::Subscribe => "subscribe",
        NotificationTriageAction::Unsubscribe => "unsubscribe",
    }
}

#[derive(Debug, Clone)]
struct NormalizedCustomFilter {
    name: String,
    query_string: String,
}

async fn validate_custom_filter_input(
    pool: &PgPool,
    user_id: Uuid,
    input: UpsertNotificationCustomFilter,
) -> Result<NormalizedCustomFilter, NotificationError> {
    let name = input.name.trim().to_owned();
    if name.is_empty() {
        return Err(NotificationError::Validation(
            "Filter name is required.".to_owned(),
        ));
    }
    if name.chars().count() > 60 {
        return Err(NotificationError::Validation(
            "Filter name must be 60 characters or fewer.".to_owned(),
        ));
    }

    let query_string = input
        .query_string
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    validate_custom_filter_query(pool, user_id, &query_string).await?;
    Ok(NormalizedCustomFilter { name, query_string })
}

async fn validate_custom_filter_query(
    pool: &PgPool,
    user_id: Uuid,
    query: &str,
) -> Result<(), NotificationError> {
    if query.is_empty() {
        return Err(NotificationError::Validation(
            "Filter query is required.".to_owned(),
        ));
    }
    if query.chars().count() > 180 {
        return Err(NotificationError::Validation(
            "Filter query must be 180 characters or fewer.".to_owned(),
        ));
    }

    for token in query.split_whitespace() {
        if token.eq_ignore_ascii_case("NOT") || token.starts_with('-') {
            return Err(NotificationError::Validation(
                "Custom notification filters do not support NOT or exclusion searches.".to_owned(),
            ));
        }
        let Some((qualifier, value)) = token.split_once(':') else {
            return Err(NotificationError::Validation(
                "Custom notification filters must use supported qualifiers instead of full-text searches.".to_owned(),
            ));
        };
        let qualifier = qualifier.to_ascii_lowercase();
        let value = value.trim_matches('"').trim();
        if value.is_empty() {
            return Err(NotificationError::Validation(format!(
                "{qualifier}: requires a value."
            )));
        }
        match qualifier.as_str() {
            "repo" => validate_repo_qualifier(pool, user_id, value).await?,
            "org" => validate_org_qualifier(pool, user_id, value).await?,
            "author" => {}
            "is" => match value {
                "unread" | "read" | "saved" | "done" => {}
                _ => {
                    return Err(NotificationError::Validation(
                        "is: only supports unread, read, saved, or done.".to_owned(),
                    ));
                }
            },
            "reason" => {}
            _ => {
                return Err(NotificationError::Validation(format!(
                    "{qualifier}: is not supported. Use repo:, org:, author:, is:, or reason:."
                )));
            }
        }
    }
    Ok(())
}

async fn validate_repo_qualifier(
    pool: &PgPool,
    user_id: Uuid,
    value: &str,
) -> Result<(), NotificationError> {
    let Some((owner, repo)) = value.split_once('/') else {
        return Err(NotificationError::Validation(
            "repo: must use owner/name.".to_owned(),
        ));
    };
    let repository = get_repository_by_owner_name(pool, owner, repo)
        .await
        .map_err(|error| NotificationError::Validation(error.to_string()))?
        .ok_or_else(|| {
            NotificationError::Validation("repo: is not available for this account.".to_owned())
        })?;
    if can_read_repository(pool, &repository, user_id)
        .await
        .map_err(|error| NotificationError::Validation(error.to_string()))?
    {
        Ok(())
    } else {
        Err(NotificationError::Validation(
            "repo: is not available for this account.".to_owned(),
        ))
    }
}

async fn validate_org_qualifier(
    pool: &PgPool,
    user_id: Uuid,
    value: &str,
) -> Result<(), NotificationError> {
    let allowed = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM organizations
            JOIN organization_memberships
              ON organization_memberships.organization_id = organizations.id
             AND organization_memberships.user_id = $2
            WHERE lower(organizations.slug) = lower($1)
        )
        "#,
    )
    .bind(value)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    if allowed {
        Ok(())
    } else {
        Err(NotificationError::Validation(
            "org: is not available for this account.".to_owned(),
        ))
    }
}

async fn custom_filter_count(pool: &PgPool, user_id: Uuid) -> Result<i64, NotificationError> {
    Ok(sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM notification_custom_filters WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}

fn map_custom_filter_write_error(error: sqlx::Error) -> NotificationError {
    if let sqlx::Error::Database(database_error) = &error {
        if database_error.constraint() == Some("notification_custom_filters_user_name_unique") {
            return NotificationError::Validation(
                "A custom notification filter with that name already exists.".to_owned(),
            );
        }
    }
    NotificationError::Sqlx(error)
}

async fn select_notification_triage_row(
    pool: &PgPool,
    notification_id: Uuid,
    user_id: Uuid,
) -> Result<Option<sqlx::postgres::PgRow>, NotificationError> {
    Ok(sqlx::query(
        r#"
        SELECT id, unread, saved, done_at, last_read_at, saved_at
        FROM notifications
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(notification_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?)
}

async fn ensure_notification_thread(
    pool: &PgPool,
    repository_id: Option<Uuid>,
    subject_type: &str,
    subject_id: Option<Uuid>,
    fallback_subject_key: Option<Uuid>,
) -> Result<Uuid, NotificationError> {
    let repository_key = repository_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "global".to_owned());
    let subject_key = subject_id
        .or(fallback_subject_key)
        .map(|id| id.to_string())
        .unwrap_or_else(|| format!("{repository_key}:{subject_type}"));
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO notification_threads (repository_id, repository_key, subject_type, subject_id, subject_key)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (repository_key, subject_type, subject_key)
        DO UPDATE SET updated_at = notification_threads.updated_at
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(repository_key)
    .bind(subject_type)
    .bind(subject_id)
    .bind(subject_key)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

async fn set_notification_subscription(
    pool: &PgPool,
    notification_id: Uuid,
    user_id: Uuid,
    state: &str,
) -> Result<(), NotificationError> {
    let row = sqlx::query(
        r#"
        SELECT repository_id, thread_id, subject_type, subject_id
        FROM notifications
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(notification_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(NotificationError::NotFound)?;

    let thread_id = if let Some(thread_id) = row.get::<Option<Uuid>, _>("thread_id") {
        thread_id
    } else {
        let subject_type: String = row.get("subject_type");
        let thread_id = ensure_notification_thread(
            pool,
            row.get("repository_id"),
            &subject_type,
            row.get("subject_id"),
            Some(notification_id),
        )
        .await?;
        sqlx::query("UPDATE notifications SET thread_id = $1 WHERE id = $2")
            .bind(thread_id)
            .bind(notification_id)
            .execute(pool)
            .await?;
        thread_id
    };

    sqlx::query(
        r#"
        INSERT INTO notification_subscriptions (thread_id, user_id, state, reason)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (thread_id, user_id)
        DO UPDATE SET state = EXCLUDED.state, reason = EXCLUDED.reason
        "#,
    )
    .bind(thread_id)
    .bind(user_id)
    .bind(state)
    .bind(if state == "unsubscribed" {
        "manual_unsubscribe"
    } else {
        "manual_subscribe"
    })
    .execute(pool)
    .await?;
    Ok(())
}

async fn notification_subscribed(
    pool: &PgPool,
    notification_id: Uuid,
    user_id: Uuid,
) -> Result<bool, NotificationError> {
    let subscribed = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT COALESCE(
            (
                SELECT notification_subscriptions.state <> 'unsubscribed'
                FROM notification_subscriptions
                WHERE notification_subscriptions.thread_id = notifications.thread_id
                  AND notification_subscriptions.user_id = notifications.user_id
            ),
            EXISTS (
                SELECT 1 FROM repository_watches
                WHERE repository_watches.user_id = notifications.user_id
                  AND repository_watches.repository_id = notifications.repository_id
            ),
            false
        )
        FROM notifications
        WHERE notifications.id = $1 AND notifications.user_id = $2
        "#,
    )
    .bind(notification_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(NotificationError::NotFound)?;
    Ok(subscribed)
}

async fn reactivate_notification_subscription_if_needed(
    pool: &PgPool,
    thread_id: Uuid,
    user_id: Uuid,
    reason: &str,
) -> Result<(), NotificationError> {
    let state = match reason {
        "mention" | "team_mention" | "review_requested" => Some("subscribed"),
        "participating" | "comment" | "review_submitted" | "merged" | "closed" | "reopened" => {
            Some("participating")
        }
        _ => None,
    };
    let Some(state) = state else {
        return Ok(());
    };
    sqlx::query(
        r#"
        INSERT INTO notification_subscriptions (thread_id, user_id, state, reason)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (thread_id, user_id)
        DO UPDATE SET state = EXCLUDED.state, reason = EXCLUDED.reason
        "#,
    )
    .bind(thread_id)
    .bind(user_id)
    .bind(state)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

fn direct_reactivation_reason(reason: &str) -> bool {
    matches!(
        reason,
        "mention" | "team_mention" | "review_requested" | "assigned"
    )
}

async fn repository_watch_delivers_event(
    pool: &PgPool,
    user_id: Uuid,
    repository_id: Uuid,
    event: RepositoryWatchEvent,
) -> Result<bool, NotificationError> {
    let row = sqlx::query(
        r#"
        SELECT level, custom_events
        FROM repository_watches
        WHERE user_id = $1 AND repository_id = $2
        "#,
    )
    .bind(user_id)
    .bind(repository_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(false);
    };
    let level = RepositoryWatchLevel::try_from(row.get::<String, _>("level").as_str())
        .map_err(|error| NotificationError::Validation(error.to_string()))?;
    Ok(match level {
        RepositoryWatchLevel::Participating => false,
        RepositoryWatchLevel::All => true,
        RepositoryWatchLevel::Ignore => false,
        RepositoryWatchLevel::Custom => {
            let custom_events = repository_watch_events_from_json(row.get("custom_events"))?;
            custom_events.contains(&event)
        }
    })
}

async fn thread_subscription_delivers(
    pool: &PgPool,
    input: &NotificationDeliveryCheck,
) -> Result<Option<bool>, NotificationError> {
    let thread_id = ensure_notification_thread(
        pool,
        Some(input.repository_id),
        &input.subject_type,
        input.subject_id,
        None,
    )
    .await?;
    let generic_state = sqlx::query_scalar::<_, String>(
        r#"
        SELECT state
        FROM notification_subscriptions
        WHERE thread_id = $1 AND user_id = $2
        "#,
    )
    .bind(thread_id)
    .bind(input.user_id)
    .fetch_optional(pool)
    .await?;
    match generic_state.as_deref() {
        Some("unsubscribed") => return Ok(Some(false)),
        Some("subscribed" | "participating") => return Ok(Some(true)),
        _ => {}
    }

    match input.subject_type.as_str() {
        "issue" => {
            let Some(issue_id) = input.subject_id else {
                return Ok(None);
            };
            let row = sqlx::query(
                r#"
                SELECT subscribed, custom_events
                FROM issue_subscriptions
                WHERE issue_id = $1 AND user_id = $2
                "#,
            )
            .bind(issue_id)
            .bind(input.user_id)
            .fetch_optional(pool)
            .await?;
            Ok(row.map(|row| {
                thread_subscription_decision(
                    row.get("subscribed"),
                    row.get::<Vec<String>, _>("custom_events"),
                    &input.reason,
                )
            }))
        }
        "pull_request" => {
            let Some(pull_request_id) = input.subject_id else {
                return Ok(None);
            };
            let row = sqlx::query(
                r#"
                SELECT subscribed, custom_events
                FROM pull_request_subscriptions
                WHERE pull_request_id = $1 AND user_id = $2
                "#,
            )
            .bind(pull_request_id)
            .bind(input.user_id)
            .fetch_optional(pool)
            .await?;
            Ok(row.map(|row| {
                thread_subscription_decision(
                    row.get("subscribed"),
                    row.get::<Vec<String>, _>("custom_events"),
                    &input.reason,
                )
            }))
        }
        _ => Ok(None),
    }
}

fn thread_subscription_decision(
    subscribed: bool,
    custom_events: Vec<String>,
    reason: &str,
) -> bool {
    if !subscribed {
        return false;
    }
    let Some(event) = thread_event_for_reason(reason) else {
        return true;
    };
    custom_events.is_empty() || custom_events.iter().any(|custom| custom == event)
}

fn thread_event_for_reason(reason: &str) -> Option<&'static str> {
    match reason {
        "closed" => Some("closed"),
        "reopened" => Some("reopened"),
        "merged" | "pull_request_merged" => Some("merged"),
        _ => None,
    }
}

fn repository_watch_events_from_json(
    value: serde_json::Value,
) -> Result<Vec<RepositoryWatchEvent>, NotificationError> {
    value
        .as_array()
        .ok_or_else(|| NotificationError::Validation("custom_events must be an array".to_owned()))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| {
                    NotificationError::Validation("custom_events must contain strings".to_owned())
                })
                .and_then(|event| {
                    RepositoryWatchEvent::try_from(event)
                        .map_err(|error| NotificationError::Validation(error.to_string()))
                })
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationInboxView {
    pub query: NotificationInboxQueryView,
    pub folders: Vec<NotificationFacet>,
    pub filters: Vec<NotificationFacet>,
    pub repositories: Vec<NotificationFacet>,
    pub sort_options: Vec<NotificationChoice>,
    pub group_options: Vec<NotificationChoice>,
    pub groups: Vec<NotificationGroup>,
    pub total: i64,
    pub unread_count: i64,
    pub page: i64,
    pub page_size: i64,
    pub empty_title: String,
    pub empty_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationInboxQueryView {
    pub q: String,
    pub folder: String,
    pub tab: String,
    pub sort: String,
    pub group: String,
    pub repo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationFacet {
    pub id: String,
    pub label: String,
    pub query: String,
    pub href: String,
    pub count: i64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationChoice {
    pub id: String,
    pub label: String,
    pub href: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationGroup {
    pub id: String,
    pub label: String,
    pub count: i64,
    pub rows: Vec<NotificationInboxRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationInboxRow {
    pub id: Uuid,
    pub repository_id: Option<Uuid>,
    pub repository_name: String,
    pub repository_href: Option<String>,
    pub subject_type: String,
    pub subject_number: Option<i64>,
    pub title: String,
    pub reason: String,
    pub reason_label: String,
    pub href: String,
    pub open_href: String,
    pub unread: bool,
    pub saved: bool,
    pub done: bool,
    pub subscribed: bool,
    pub updated_at: DateTime<Utc>,
    pub relative_time: String,
}

#[derive(Debug, Clone, Default)]
pub struct NotificationInboxQuery {
    pub q: Option<String>,
    pub folder: Option<String>,
    pub tab: Option<String>,
    pub sort: Option<String>,
    pub group: Option<String>,
    pub repo: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone)]
struct InboxRowData {
    id: Uuid,
    repository_id: Option<Uuid>,
    owner_login: Option<String>,
    repo_name: Option<String>,
    subject_type: String,
    title: String,
    reason: String,
    unread: bool,
    saved: bool,
    done_at: Option<DateTime<Utc>>,
    updated_at: DateTime<Utc>,
    issue_number: Option<i64>,
    pull_number: Option<i64>,
    run_number: Option<i64>,
    release_tag: Option<String>,
    subscribed: bool,
}

#[derive(Debug, Clone)]
struct ParsedNotificationQuery {
    text_terms: Vec<String>,
    unread: Option<bool>,
    saved: Option<bool>,
    done: Option<bool>,
    reason: Option<String>,
    repo: Option<String>,
    subject_type: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct NotificationLinkContext<'a> {
    folder: &'a str,
    tab: &'a str,
    sort: &'a str,
    group: &'a str,
    q: &'a str,
    repo: Option<&'a str>,
}

pub async fn notification_inbox_view(
    pool: &PgPool,
    user_id: Uuid,
    query: NotificationInboxQuery,
) -> Result<NotificationInboxView, NotificationError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).clamp(1, 100);
    let offset = (page - 1) * page_size;
    let folder = normalize_folder(query.folder.as_deref());
    let tab = normalize_tab(query.tab.as_deref());
    let sort = normalize_sort(query.sort.as_deref());
    let group = normalize_group(query.group.as_deref());
    let raw_q = query.q.unwrap_or_default().trim().to_owned();
    let mut parsed = parse_notification_query(&raw_q);
    if tab == "unread" {
        parsed.unread = Some(true);
    }
    match folder.as_str() {
        "saved" => parsed.saved = Some(true),
        "done" => parsed.done = Some(true),
        _ => {
            if parsed.done.is_none() {
                parsed.done = Some(false);
            }
        }
    }
    if let Some(repo) = query
        .repo
        .as_deref()
        .map(str::trim)
        .filter(|repo| !repo.is_empty())
    {
        parsed.repo = Some(repo.to_owned());
    }

    let total = count_matching_notifications(pool, user_id, &parsed).await?;
    let unread_count = unread_notification_count(pool, user_id).await?;
    let rows =
        fetch_matching_notifications(pool, user_id, &parsed, &sort, page_size, offset).await?;
    let inbox_rows: Vec<NotificationInboxRow> = rows.into_iter().map(inbox_row_view).collect();
    let groups = group_notifications(&inbox_rows, &group);
    let repositories = repository_facets(
        pool,
        user_id,
        NotificationLinkContext {
            folder: &folder,
            tab: &tab,
            sort: &sort,
            group: &group,
            q: &raw_q,
            repo: parsed.repo.as_deref(),
        },
    )
    .await?;
    let folder_facets = folder_facets(pool, user_id, &folder, &tab, &sort, &group, &raw_q).await?;
    let filters =
        default_filter_facets(pool, user_id, &folder, &tab, &sort, &group, &raw_q).await?;

    let empty_title = if folder == "saved" {
        "No saved notifications".to_owned()
    } else if folder == "done" {
        "No done notifications".to_owned()
    } else if tab == "unread" {
        "No unread notifications".to_owned()
    } else {
        "No matching notifications".to_owned()
    };

    Ok(NotificationInboxView {
        query: NotificationInboxQueryView {
            q: raw_q.clone(),
            folder: folder.clone(),
            tab: tab.clone(),
            sort: sort.clone(),
            group: group.clone(),
            repo: parsed.repo,
        },
        folders: folder_facets,
        filters,
        repositories,
        sort_options: choice_options(
            "sort",
            &[("newest", "Newest"), ("oldest", "Oldest")],
            &sort,
            NotificationLinkContext {
                folder: &folder,
                tab: &tab,
                sort: &sort,
                group: &group,
                q: &raw_q,
                repo: None,
            },
        ),
        group_options: choice_options(
            "group",
            &[("date", "Date"), ("repository", "Repository")],
            &group,
            NotificationLinkContext {
                folder: &folder,
                tab: &tab,
                sort: &sort,
                group: &group,
                q: &raw_q,
                repo: None,
            },
        ),
        groups,
        total,
        unread_count,
        page,
        page_size,
        empty_title,
        empty_message: "Adjust the query, folder, repository, or unread tab to broaden the inbox."
            .to_owned(),
    })
}

async fn count_matching_notifications(
    pool: &PgPool,
    user_id: Uuid,
    parsed: &ParsedNotificationQuery,
) -> Result<i64, NotificationError> {
    let mut builder = QueryBuilder::new(base_count_sql());
    push_filters(&mut builder, user_id, parsed);
    Ok(builder.build_query_scalar().fetch_one(pool).await?)
}

async fn fetch_matching_notifications(
    pool: &PgPool,
    user_id: Uuid,
    parsed: &ParsedNotificationQuery,
    sort: &str,
    page_size: i64,
    offset: i64,
) -> Result<Vec<InboxRowData>, NotificationError> {
    let mut builder = QueryBuilder::new(base_select_sql());
    push_filters(&mut builder, user_id, parsed);
    if sort == "oldest" {
        builder.push(" ORDER BY notifications.updated_at ASC, notifications.created_at ASC ");
    } else {
        builder.push(" ORDER BY notifications.updated_at DESC, notifications.created_at DESC ");
    }
    builder
        .push(" LIMIT ")
        .push_bind(page_size)
        .push(" OFFSET ")
        .push_bind(offset);

    let rows = builder.build().fetch_all(pool).await?;
    Ok(rows.into_iter().map(inbox_row_data_from_row).collect())
}

fn base_count_sql() -> &'static str {
    r#"
    SELECT count(*)
    FROM notifications
    LEFT JOIN repositories ON repositories.id = notifications.repository_id
    LEFT JOIN users owner_user ON owner_user.id = repositories.owner_user_id
    LEFT JOIN organizations owner_org ON owner_org.id = repositories.owner_organization_id
    LEFT JOIN issues ON notifications.subject_type = 'issue' AND issues.id = notifications.subject_id
    LEFT JOIN pull_requests ON notifications.subject_type = 'pull_request' AND pull_requests.id = notifications.subject_id
    LEFT JOIN workflow_runs ON notifications.subject_type = 'workflow_run' AND workflow_runs.id = notifications.subject_id
    LEFT JOIN releases ON notifications.subject_type = 'release' AND releases.id = notifications.subject_id
    "#
}

fn base_select_sql() -> &'static str {
    r#"
    SELECT notifications.id, notifications.repository_id,
           COALESCE(NULLIF(owner_user.username, ''), owner_user.email, owner_org.slug) AS owner_login,
           repositories.name AS repo_name,
           notifications.subject_type, notifications.title,
           notifications.reason, notifications.unread, notifications.saved, notifications.done_at,
           notifications.updated_at,
           issues.number AS issue_number, pull_requests.number AS pull_number,
           workflow_runs.run_number AS run_number, releases.tag_name AS release_tag,
           COALESCE(
               notification_subscriptions.state <> 'unsubscribed',
               EXISTS (
                   SELECT 1 FROM repository_watches
                   WHERE repository_watches.user_id = notifications.user_id
                     AND repository_watches.repository_id = notifications.repository_id
               ),
               false
           ) AS subscribed
    FROM notifications
    LEFT JOIN repositories ON repositories.id = notifications.repository_id
    LEFT JOIN users owner_user ON owner_user.id = repositories.owner_user_id
    LEFT JOIN organizations owner_org ON owner_org.id = repositories.owner_organization_id
    LEFT JOIN notification_subscriptions
           ON notification_subscriptions.thread_id = notifications.thread_id
          AND notification_subscriptions.user_id = notifications.user_id
    LEFT JOIN issues ON notifications.subject_type = 'issue' AND issues.id = notifications.subject_id
    LEFT JOIN pull_requests ON notifications.subject_type = 'pull_request' AND pull_requests.id = notifications.subject_id
    LEFT JOIN workflow_runs ON notifications.subject_type = 'workflow_run' AND workflow_runs.id = notifications.subject_id
    LEFT JOIN releases ON notifications.subject_type = 'release' AND releases.id = notifications.subject_id
    "#
}

fn push_filters<'a>(
    builder: &mut QueryBuilder<'a, sqlx::Postgres>,
    user_id: Uuid,
    parsed: &'a ParsedNotificationQuery,
) {
    builder
        .push(" WHERE notifications.user_id = ")
        .push_bind(user_id);
    if let Some(unread) = parsed.unread {
        builder
            .push(" AND notifications.unread = ")
            .push_bind(unread);
    }
    if let Some(saved) = parsed.saved {
        builder.push(" AND notifications.saved = ").push_bind(saved);
    }
    if let Some(done) = parsed.done {
        builder.push(if done {
            " AND notifications.done_at IS NOT NULL"
        } else {
            " AND notifications.done_at IS NULL"
        });
        if !done {
            builder.push(
                " AND NOT EXISTS (
                    SELECT 1 FROM notification_subscriptions hidden_subscriptions
                    WHERE hidden_subscriptions.thread_id = notifications.thread_id
                      AND hidden_subscriptions.user_id = notifications.user_id
                      AND hidden_subscriptions.state = 'unsubscribed'
                )",
            );
        }
    }
    if let Some(reason) = &parsed.reason {
        builder
            .push(" AND lower(notifications.reason) = lower(")
            .push_bind(reason)
            .push(")");
    }
    if let Some(subject_type) = &parsed.subject_type {
        builder
            .push(" AND notifications.subject_type = ")
            .push_bind(subject_type);
    }
    if let Some(repo) = &parsed.repo {
        let pattern = format!("%{}%", repo.to_lowercase());
        builder.push(" AND lower(COALESCE(NULLIF(owner_user.username, ''), owner_user.email, owner_org.slug, '') || '/' || COALESCE(repositories.name, '')) LIKE ").push_bind(pattern);
    }
    for term in &parsed.text_terms {
        let pattern = format!("%{}%", term.to_lowercase());
        builder
            .push(" AND (lower(notifications.title) LIKE ")
            .push_bind(pattern.clone())
            .push(" OR lower(notifications.reason) LIKE ")
            .push_bind(pattern)
            .push(")");
    }
}

async fn repository_facets(
    pool: &PgPool,
    user_id: Uuid,
    context: NotificationLinkContext<'_>,
) -> Result<Vec<NotificationFacet>, NotificationError> {
    let rows = sqlx::query(
        r#"
        SELECT COALESCE(NULLIF(owner_user.username, ''), owner_user.email, owner_org.slug) AS owner_login,
               repositories.name AS repo_name,
               count(*) AS count
        FROM notifications
        JOIN repositories ON repositories.id = notifications.repository_id
        LEFT JOIN users owner_user ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = repositories.owner_organization_id
        WHERE notifications.user_id = $1
        GROUP BY owner_login, repositories.name
        ORDER BY count DESC, owner_login ASC, repositories.name ASC
        LIMIT 12
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let owner: String = row.get("owner_login");
            let repo_name: String = row.get("repo_name");
            let full = format!("{owner}/{repo_name}");
            let active = context
                .repo
                .is_some_and(|repo| repo.eq_ignore_ascii_case(&full));
            NotificationFacet {
                id: full.clone(),
                label: full.clone(),
                query: context.q.to_owned(),
                href: notification_href(
                    context.folder,
                    context.tab,
                    context.sort,
                    context.group,
                    context.q,
                    Some(&full),
                ),
                count: row.get("count"),
                active,
            }
        })
        .collect())
}

async fn folder_facets(
    pool: &PgPool,
    user_id: Uuid,
    active_folder: &str,
    tab: &str,
    sort: &str,
    group: &str,
    q: &str,
) -> Result<Vec<NotificationFacet>, NotificationError> {
    let counts = notification_folder_counts(pool, user_id).await?;
    let specs = [
        ("inbox", "Inbox", counts.inbox),
        ("saved", "Saved", counts.saved),
        ("done", "Done", counts.done),
    ];
    Ok(specs
        .into_iter()
        .map(|(id, label, count)| NotificationFacet {
            id: id.to_owned(),
            label: label.to_owned(),
            query: q.to_owned(),
            href: notification_href(id, tab, sort, group, q, None),
            count,
            active: id == active_folder,
        })
        .collect())
}

async fn default_filter_facets(
    pool: &PgPool,
    user_id: Uuid,
    folder: &str,
    tab: &str,
    sort: &str,
    group: &str,
    q: &str,
) -> Result<Vec<NotificationFacet>, NotificationError> {
    let custom_filters = list_notification_custom_filters(pool, user_id).await?;
    let mut facets =
        Vec::with_capacity(notification_default_filters().len() + custom_filters.len());
    for (id, label, query) in notification_default_filters() {
        let parsed = parse_notification_query(query);
        let count = count_matching_notifications(pool, user_id, &parsed).await?;
        facets.push(NotificationFacet {
            id: id.to_owned(),
            label: label.to_owned(),
            query: query.to_owned(),
            href: notification_href(folder, tab, sort, group, query, None),
            count,
            active: q == query,
        });
    }
    for custom_filter in custom_filters {
        let parsed = parse_notification_query(&custom_filter.query_string);
        let count = count_matching_notifications(pool, user_id, &parsed).await?;
        facets.push(NotificationFacet {
            id: format!("custom-{}", custom_filter.id),
            label: custom_filter.name,
            query: custom_filter.query_string.clone(),
            href: notification_href(folder, tab, sort, group, &custom_filter.query_string, None),
            count,
            active: q == custom_filter.query_string,
        });
    }
    Ok(facets)
}

fn notification_default_filters() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("assigned", "Assigned", "reason:assigned"),
        ("participating", "Participating", "reason:participating"),
        ("mentioned", "Mentioned", "reason:mention"),
        ("team-mentioned", "Team mentioned", "reason:team_mention"),
        (
            "review-requested",
            "Review requested",
            "reason:review_requested",
        ),
    ]
}

fn choice_options(
    param: &str,
    specs: &[(&str, &str)],
    active: &str,
    context: NotificationLinkContext<'_>,
) -> Vec<NotificationChoice> {
    specs
        .iter()
        .map(|(id, label)| {
            let (sort, group) = if param == "sort" {
                (*id, context.group)
            } else {
                (context.sort, *id)
            };
            NotificationChoice {
                id: (*id).to_owned(),
                label: (*label).to_owned(),
                href: notification_href(
                    context.folder,
                    context.tab,
                    sort,
                    group,
                    context.q,
                    context.repo,
                ),
                active: *id == active,
            }
        })
        .collect()
}

fn notification_href(
    folder: &str,
    tab: &str,
    sort: &str,
    group: &str,
    q: &str,
    repo: Option<&str>,
) -> String {
    let mut params = Vec::new();
    if folder != "inbox" {
        params.push(("folder", folder.to_owned()));
    }
    if tab != "all" {
        params.push(("tab", tab.to_owned()));
    }
    if !q.trim().is_empty() {
        params.push(("q", q.trim().to_owned()));
    }
    if sort != "newest" {
        params.push(("sort", sort.to_owned()));
    }
    if group != "date" {
        params.push(("group", group.to_owned()));
    }
    if let Some(repo) = repo {
        params.push(("repo", repo.to_owned()));
    }
    if params.is_empty() {
        return "/notifications".to_owned();
    }
    let query = params
        .into_iter()
        .map(|(key, value)| format!("{}={}", key, url_encode(&value)))
        .collect::<Vec<_>>()
        .join("&");
    format!("/notifications?{query}")
}

fn inbox_row_view(row: InboxRowData) -> NotificationInboxRow {
    let repository_name = match (&row.owner_login, &row.repo_name) {
        (Some(owner), Some(repo)) => format!("{owner}/{repo}"),
        _ => "OpenGitHub".to_owned(),
    };
    let repository_href = match (&row.owner_login, &row.repo_name) {
        (Some(owner), Some(repo)) => Some(format!("/{owner}/{repo}")),
        _ => None,
    };
    let subject_number = row.pull_number.or(row.issue_number).or(row.run_number);
    let href = target_href(&row, repository_href.as_deref());
    let open_href = format!("/notifications/{}/open?next={}", row.id, url_encode(&href));
    NotificationInboxRow {
        id: row.id,
        repository_id: row.repository_id,
        repository_name,
        repository_href,
        subject_type: row.subject_type,
        subject_number,
        title: row.title,
        reason_label: reason_label(&row.reason),
        reason: row.reason,
        href,
        open_href,
        unread: row.unread,
        saved: row.saved,
        done: row.done_at.is_some(),
        subscribed: row.subscribed,
        updated_at: row.updated_at,
        relative_time: relative_time(row.updated_at),
    }
}

fn target_href(row: &InboxRowData, repository_href: Option<&str>) -> String {
    let Some(repo_href) = repository_href else {
        return "/notifications".to_owned();
    };
    match row.subject_type.as_str() {
        "pull_request" => row
            .pull_number
            .map(|n| format!("{repo_href}/pull/{n}"))
            .unwrap_or_else(|| repo_href.to_owned()),
        "issue" => row
            .issue_number
            .map(|n| format!("{repo_href}/issues/{n}"))
            .unwrap_or_else(|| repo_href.to_owned()),
        "workflow_run" => row
            .run_number
            .map(|n| format!("{repo_href}/actions/runs/{n}"))
            .unwrap_or_else(|| format!("{repo_href}/actions")),
        "release" => row
            .release_tag
            .as_ref()
            .map(|tag| format!("{repo_href}/releases/tag/{tag}"))
            .unwrap_or_else(|| format!("{repo_href}/releases")),
        _ => repo_href.to_owned(),
    }
}

fn group_notifications(rows: &[NotificationInboxRow], group: &str) -> Vec<NotificationGroup> {
    let mut groups: Vec<NotificationGroup> = Vec::new();
    for row in rows {
        let (id, label) = if group == "repository" {
            (row.repository_name.clone(), row.repository_name.clone())
        } else {
            let age = Utc::now().signed_duration_since(row.updated_at);
            let label = if age.num_days() == 0 {
                "Today"
            } else if age.num_days() == 1 {
                "Yesterday"
            } else {
                "Earlier"
            };
            (label.to_lowercase(), label.to_owned())
        };
        if let Some(existing) = groups.iter_mut().find(|g| g.id == id) {
            existing.count += 1;
            existing.rows.push(row.clone());
        } else {
            groups.push(NotificationGroup {
                id,
                label,
                count: 1,
                rows: vec![row.clone()],
            });
        }
    }
    groups
}

fn inbox_row_data_from_row(row: sqlx::postgres::PgRow) -> InboxRowData {
    InboxRowData {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        owner_login: row.get("owner_login"),
        repo_name: row.get("repo_name"),
        subject_type: row.get("subject_type"),
        title: row.get("title"),
        reason: row.get("reason"),
        unread: row.get("unread"),
        saved: row.get("saved"),
        done_at: row.get("done_at"),
        updated_at: row.get("updated_at"),
        issue_number: row.get("issue_number"),
        pull_number: row.get("pull_number"),
        run_number: row.try_get("run_number").ok(),
        release_tag: row.try_get("release_tag").ok(),
        subscribed: row.get("subscribed"),
    }
}

fn parse_notification_query(query: &str) -> ParsedNotificationQuery {
    let mut parsed = ParsedNotificationQuery {
        text_terms: Vec::new(),
        unread: None,
        saved: None,
        done: None,
        reason: None,
        repo: None,
        subject_type: None,
    };
    for token in query.split_whitespace() {
        if let Some(value) = token.strip_prefix("is:") {
            match value {
                "unread" => parsed.unread = Some(true),
                "read" => parsed.unread = Some(false),
                "saved" => parsed.saved = Some(true),
                "done" => parsed.done = Some(true),
                _ => parsed.text_terms.push(token.to_owned()),
            }
        } else if let Some(value) = token.strip_prefix("reason:") {
            parsed.reason = Some(value.trim_matches('"').to_owned());
        } else if let Some(value) = token.strip_prefix("repo:") {
            parsed.repo = Some(value.trim_matches('"').to_owned());
        } else if let Some(value) = token.strip_prefix("type:") {
            parsed.subject_type = Some(match value {
                "pr" | "pull" | "pull_request" => "pull_request".to_owned(),
                "run" | "workflow" | "workflow_run" => "workflow_run".to_owned(),
                other => other.to_owned(),
            });
        } else {
            parsed.text_terms.push(token.to_owned());
        }
    }
    parsed
}

fn normalize_folder(value: Option<&str>) -> String {
    match value {
        Some("saved") => "saved",
        Some("done") => "done",
        _ => "inbox",
    }
    .to_owned()
}
fn normalize_tab(value: Option<&str>) -> String {
    match value {
        Some("unread") => "unread",
        _ => "all",
    }
    .to_owned()
}
fn normalize_sort(value: Option<&str>) -> String {
    match value {
        Some("oldest") => "oldest",
        _ => "newest",
    }
    .to_owned()
}
fn normalize_group(value: Option<&str>) -> String {
    match value {
        Some("repository") => "repository",
        _ => "date",
    }
    .to_owned()
}
fn reason_label(reason: &str) -> String {
    reason
        .replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn url_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn relative_time(updated_at: DateTime<Utc>) -> String {
    let age = Utc::now().signed_duration_since(updated_at);
    if age.num_minutes() < 1 {
        "just now".to_owned()
    } else if age.num_hours() < 1 {
        format!("{}m ago", age.num_minutes())
    } else if age.num_days() < 1 {
        format!("{}h ago", age.num_hours())
    } else {
        format!("{}d ago", age.num_days())
    }
}

fn notification_from_row(row: sqlx::postgres::PgRow) -> Notification {
    Notification {
        id: row.get("id"),
        user_id: row.get("user_id"),
        repository_id: row.get("repository_id"),
        subject_type: row.get("subject_type"),
        subject_id: row.get("subject_id"),
        title: row.get("title"),
        reason: row.get("reason"),
        unread: row.get("unread"),
        saved: row.get("saved"),
        done_at: row.get("done_at"),
        last_read_at: row.get("last_read_at"),
        saved_at: row.get("saved_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn custom_filter_from_row(row: sqlx::postgres::PgRow) -> NotificationCustomFilter {
    let id: Uuid = row.get("id");
    let query_string: String = row.get("query_string");
    NotificationCustomFilter {
        id,
        name: row.get("name"),
        href: notification_href("inbox", "all", "newest", "date", &query_string, None),
        query_string,
        position: row.get("position"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
