use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::{
    permissions::RepositoryRole,
    repositories::{repository_permission_for_user, RepositoryError},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Webhook {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub url: String,
    pub secret_hash: Option<String>,
    pub events: Vec<String>,
    pub active: bool,
    pub created_by_user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event: String,
    pub payload: Value,
    pub status: DeliveryStatus,
    pub attempt_count: i32,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    Queued,
    Delivered,
    Failed,
}

impl DeliveryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Delivered => "delivered",
            Self::Failed => "failed",
        }
    }
}

impl TryFrom<&str> for DeliveryStatus {
    type Error = WebhookError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "queued" => Ok(Self::Queued),
            "delivered" => Ok(Self::Delivered),
            "failed" => Ok(Self::Failed),
            other => Err(WebhookError::InvalidDeliveryStatus(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhook {
    pub repository_id: Uuid,
    pub actor_user_id: Uuid,
    pub url: String,
    pub secret_hash: Option<String>,
    pub events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookDelivery {
    pub webhook_id: Uuid,
    pub event: String,
    pub payload: Value,
}

#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("user does not have repository admin access")]
    RepositoryAccessDenied,
    #[error("webhook was not found")]
    WebhookNotFound,
    #[error("invalid delivery status `{0}`")]
    InvalidDeliveryStatus(String),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub async fn create_webhook(pool: &PgPool, input: CreateWebhook) -> Result<Webhook, WebhookError> {
    require_repository_role(
        pool,
        input.repository_id,
        input.actor_user_id,
        RepositoryRole::Admin,
    )
    .await?;
    let row = sqlx::query(
        r#"
        INSERT INTO webhooks (repository_id, url, secret_hash, events, created_by_user_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, repository_id, url, secret_hash, events, active, created_by_user_id, created_at, updated_at
        "#,
    )
    .bind(input.repository_id)
    .bind(&input.url)
    .bind(&input.secret_hash)
    .bind(&input.events)
    .bind(input.actor_user_id)
    .fetch_one(pool)
    .await?;

    Ok(webhook_from_row(row))
}

pub async fn create_webhook_delivery(
    pool: &PgPool,
    input: CreateWebhookDelivery,
) -> Result<WebhookDelivery, WebhookError> {
    let row = sqlx::query(
        r#"
        INSERT INTO webhook_deliveries (webhook_id, event, payload)
        VALUES ($1, $2, $3)
        RETURNING id, webhook_id, event, payload, status, attempt_count, next_attempt_at,
                  response_status, response_body, delivered_at, created_at, updated_at
        "#,
    )
    .bind(input.webhook_id)
    .bind(&input.event)
    .bind(input.payload)
    .fetch_one(pool)
    .await?;

    delivery_from_row(row)
}

pub async fn record_webhook_delivery_attempt(
    pool: &PgPool,
    delivery_id: Uuid,
    status: DeliveryStatus,
    response_status: Option<i32>,
    response_body: Option<String>,
    retry_after_seconds: Option<i64>,
) -> Result<WebhookDelivery, WebhookError> {
    let row = sqlx::query(
        r#"
        UPDATE webhook_deliveries
        SET status = $2,
            response_status = $3,
            response_body = $4,
            attempt_count = attempt_count + 1,
            next_attempt_at = CASE
                WHEN $5::bigint IS NULL THEN NULL
                ELSE now() + ($5::bigint * interval '1 second')
            END,
            delivered_at = CASE WHEN $2 = 'delivered' THEN now() ELSE NULL END
        WHERE id = $1
        RETURNING id, webhook_id, event, payload, status, attempt_count, next_attempt_at,
                  response_status, response_body, delivered_at, created_at, updated_at
        "#,
    )
    .bind(delivery_id)
    .bind(status.as_str())
    .bind(response_status)
    .bind(&response_body)
    .bind(retry_after_seconds)
    .fetch_optional(pool)
    .await?
    .ok_or(WebhookError::WebhookNotFound)?;

    delivery_from_row(row)
}

async fn require_repository_role(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
    required_role: RepositoryRole,
) -> Result<(), WebhookError> {
    let permission = repository_permission_for_user(pool, repository_id, user_id)
        .await
        .map_err(map_repository_error)?
        .ok_or(WebhookError::RepositoryAccessDenied)?;

    let allowed = match required_role {
        RepositoryRole::Read => permission.role.can_read(),
        RepositoryRole::Triage => permission.role >= RepositoryRole::Triage,
        RepositoryRole::Write => permission.role.can_write(),
        RepositoryRole::Maintain => permission.role >= RepositoryRole::Maintain,
        RepositoryRole::Admin => permission.role.can_admin(),
        RepositoryRole::Owner => permission.role == RepositoryRole::Owner,
    };

    if allowed {
        Ok(())
    } else {
        Err(WebhookError::RepositoryAccessDenied)
    }
}

fn map_repository_error(error: RepositoryError) -> WebhookError {
    match error {
        RepositoryError::Sqlx(error) => WebhookError::Sqlx(error),
        _ => WebhookError::RepositoryAccessDenied,
    }
}

fn webhook_from_row(row: sqlx::postgres::PgRow) -> Webhook {
    Webhook {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        url: row.get("url"),
        secret_hash: row.get("secret_hash"),
        events: row.get("events"),
        active: row.get("active"),
        created_by_user_id: row.get("created_by_user_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn delivery_from_row(row: sqlx::postgres::PgRow) -> Result<WebhookDelivery, WebhookError> {
    let status: String = row.get("status");
    Ok(WebhookDelivery {
        id: row.get("id"),
        webhook_id: row.get("webhook_id"),
        event: row.get("event"),
        payload: row.get("payload"),
        status: DeliveryStatus::try_from(status.as_str())?,
        attempt_count: row.get("attempt_count"),
        next_attempt_at: row.get("next_attempt_at"),
        response_status: row.get("response_status"),
        response_body: row.get("response_body"),
        delivered_at: row.get("delivered_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
