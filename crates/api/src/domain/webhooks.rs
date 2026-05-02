use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use sqlx::{PgPool, Row};
use std::time::Instant;
use uuid::Uuid;

use super::{
    permissions::RepositoryRole,
    repositories::{get_repository_by_owner_name, repository_permission_for_user, RepositoryError},
};

type HmacSha256 = Hmac<Sha256>;

pub const SUPPORTED_EVENTS: &[&str] = &[
    "push",
    "pull_request",
    "pull_request_review",
    "issues",
    "issue_comment",
    "release",
    "workflow_run",
    "check_run",
    "ping",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookScopeType {
    Repository,
    Organization,
}

impl WebhookScopeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Repository => "repository",
            Self::Organization => "organization",
        }
    }
}

impl TryFrom<&str> for WebhookScopeType {
    type Error = WebhookError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "repository" => Ok(Self::Repository),
            "organization" => Ok(Self::Organization),
            other => Err(WebhookError::InvalidScope(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Webhook {
    pub id: Uuid,
    pub scope_type: WebhookScopeType,
    pub scope_id: Uuid,
    pub repository_id: Option<Uuid>,
    pub url: String,
    pub content_type: String,
    #[serde(skip_serializing)]
    pub secret_ciphertext: Option<String>,
    pub has_secret: bool,
    pub events: Vec<String>,
    pub active: bool,
    pub ssl_verify: bool,
    pub created_by_user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event: String,
    pub request_headers: Value,
    pub request_body: String,
    pub response_status: Option<i32>,
    pub response_headers: Value,
    pub response_body: Option<String>,
    pub duration_ms: Option<i32>,
    pub redelivery_of: Option<Uuid>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub status: DeliveryStatus,
    pub attempt_count: i32,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WebhookWithDeliveries {
    #[serde(flatten)]
    pub webhook: Webhook,
    pub deliveries: Vec<WebhookDelivery>,
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
pub struct CreateScopedWebhook {
    pub scope_type: WebhookScopeType,
    pub scope_id: Uuid,
    pub actor_user_id: Uuid,
    pub url: String,
    pub content_type: String,
    pub secret: Option<String>,
    pub events: Vec<String>,
    pub active: bool,
    pub ssl_verify: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWebhook {
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookDelivery {
    pub webhook_id: Uuid,
    pub event: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchWebhookEvent {
    pub scope_type: WebhookScopeType,
    pub scope_id: Uuid,
    pub event: String,
    pub payload: Value,
}

#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("user does not have webhook admin access")]
    AccessDenied,
    #[error("webhook was not found")]
    WebhookNotFound,
    #[error("invalid webhook URL")]
    InvalidUrl,
    #[error("unsupported webhook event `{0}`")]
    UnsupportedEvent(String),
    #[error("invalid webhook scope `{0}`")]
    InvalidScope(String),
    #[error("invalid delivery status `{0}`")]
    InvalidDeliveryStatus(String),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub fn hmac_sha256_signature(secret: &str, body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac accepts any key length");
    mac.update(body);
    let bytes = mac.finalize().into_bytes();
    format!("sha256={}", bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>())
}

pub fn validate_events(events: &[String]) -> Result<Vec<String>, WebhookError> {
    let mut normalized = Vec::new();
    for event in events {
        let event = event.trim().to_ascii_lowercase();
        if !SUPPORTED_EVENTS.contains(&event.as_str()) {
            return Err(WebhookError::UnsupportedEvent(event));
        }
        if !normalized.contains(&event) {
            normalized.push(event);
        }
    }
    if normalized.is_empty() {
        normalized.push("push".to_owned());
    }
    Ok(normalized)
}

pub async fn create_webhook(pool: &PgPool, input: CreateWebhook) -> Result<Webhook, WebhookError> {
    create_scoped_webhook(pool, CreateScopedWebhook {
        scope_type: WebhookScopeType::Repository,
        scope_id: input.repository_id,
        actor_user_id: input.actor_user_id,
        url: input.url,
        content_type: "json".to_owned(),
        secret: input.secret_hash,
        events: input.events,
        active: true,
        ssl_verify: true,
    }).await
}

pub async fn create_scoped_webhook(pool: &PgPool, input: CreateScopedWebhook) -> Result<Webhook, WebhookError> {
    require_scope_admin(pool, &input.scope_type, input.scope_id, input.actor_user_id).await?;
    if !(input.url.starts_with("https://") || input.url.starts_with("http://")) {
        return Err(WebhookError::InvalidUrl);
    }
    let events = validate_events(&input.events)?;
    let repository_id = if input.scope_type == WebhookScopeType::Repository { Some(input.scope_id) } else { None };
    let row = sqlx::query(
        r#"
        INSERT INTO webhooks (repository_id, scope_type, scope_id, url, content_type, secret_ciphertext, secret_hash, events, active, ssl_verify, created_by_user_id)
        VALUES ($1, $2, $3, $4, $5, $6, $6, $7, $8, $9, $10)
        RETURNING id, repository_id, scope_type, scope_id, url, content_type, secret_ciphertext, events, active, ssl_verify, created_by_user_id, created_at, updated_at
        "#,
    )
    .bind(repository_id)
    .bind(input.scope_type.as_str())
    .bind(input.scope_id)
    .bind(&input.url)
    .bind(&input.content_type)
    .bind(&input.secret)
    .bind(&events)
    .bind(input.active)
    .bind(input.ssl_verify)
    .bind(input.actor_user_id)
    .fetch_one(pool)
    .await?;

    webhook_from_row(row)
}

pub async fn list_webhooks(pool: &PgPool, scope_type: WebhookScopeType, scope_id: Uuid, actor_user_id: Uuid) -> Result<Vec<WebhookWithDeliveries>, WebhookError> {
    require_scope_admin(pool, &scope_type, scope_id, actor_user_id).await?;
    let rows = sqlx::query(
        r#"
        SELECT id, repository_id, scope_type, scope_id, url, content_type, secret_ciphertext, events, active, ssl_verify, created_by_user_id, created_at, updated_at
        FROM webhooks
        WHERE scope_type = $1 AND scope_id = $2
        ORDER BY created_at DESC
        "#,
    )
    .bind(scope_type.as_str())
    .bind(scope_id)
    .fetch_all(pool)
    .await?;

    let mut out = Vec::new();
    for row in rows {
        let webhook = webhook_from_row(row)?;
        let deliveries = list_recent_deliveries(pool, webhook.id, 6).await?;
        out.push(WebhookWithDeliveries { webhook, deliveries });
    }
    Ok(out)
}

pub async fn update_webhook(pool: &PgPool, webhook_id: Uuid, actor_user_id: Uuid, input: UpdateWebhook) -> Result<Webhook, WebhookError> {
    let current = get_webhook(pool, webhook_id).await?.ok_or(WebhookError::WebhookNotFound)?;
    require_scope_admin(pool, &current.scope_type, current.scope_id, actor_user_id).await?;
    let row = sqlx::query(
        r#"
        UPDATE webhooks
        SET active = COALESCE($2, active)
        WHERE id = $1
        RETURNING id, repository_id, scope_type, scope_id, url, content_type, secret_ciphertext, events, active, ssl_verify, created_by_user_id, created_at, updated_at
        "#,
    )
    .bind(webhook_id)
    .bind(input.active)
    .fetch_one(pool)
    .await?;
    webhook_from_row(row)
}

pub async fn create_webhook_delivery(pool: &PgPool, input: CreateWebhookDelivery) -> Result<WebhookDelivery, WebhookError> {
    let body = serde_json::to_string(&input.payload).unwrap_or_else(|_| "{}".to_owned());
    insert_delivery(pool, input.webhook_id, &input.event, body, None).await
}

pub async fn dispatch_event(pool: &PgPool, input: DispatchWebhookEvent) -> Result<Vec<WebhookDelivery>, WebhookError> {
    let event = validate_events(&[input.event])?.remove(0);
    let body = serde_json::to_string(&input.payload).unwrap_or_else(|_| "{}".to_owned());
    let hooks = sqlx::query(
        r#"
        SELECT id, repository_id, scope_type, scope_id, url, content_type, secret_ciphertext, events, active, ssl_verify, created_by_user_id, created_at, updated_at
        FROM webhooks
        WHERE scope_type = $1 AND scope_id = $2 AND active = true AND ($3 = ANY(events) OR 'everything' = ANY(events))
        "#,
    )
    .bind(input.scope_type.as_str())
    .bind(input.scope_id)
    .bind(&event)
    .fetch_all(pool)
    .await?;

    let mut deliveries = Vec::new();
    for row in hooks {
        let hook = webhook_from_row(row)?;
        let mut delivery = insert_delivery(pool, hook.id, &event, body.clone(), None).await?;
        delivery = deliver_once(pool, &hook, delivery).await?;
        deliveries.push(delivery);
    }
    Ok(deliveries)
}

pub async fn redeliver(pool: &PgPool, webhook_id: Uuid, actor_user_id: Uuid, delivery_id: Option<Uuid>) -> Result<WebhookDelivery, WebhookError> {
    let hook = get_webhook(pool, webhook_id).await?.ok_or(WebhookError::WebhookNotFound)?;
    require_scope_admin(pool, &hook.scope_type, hook.scope_id, actor_user_id).await?;
    let original = if let Some(delivery_id) = delivery_id {
        get_delivery(pool, delivery_id).await?.ok_or(WebhookError::WebhookNotFound)?
    } else {
        list_recent_deliveries(pool, webhook_id, 1).await?.into_iter().next().ok_or(WebhookError::WebhookNotFound)?
    };
    let delivery = insert_delivery(pool, webhook_id, &original.event, original.request_body, Some(original.id)).await?;
    deliver_once(pool, &hook, delivery).await
}

pub async fn process_due_deliveries(pool: &PgPool, limit: i64) -> Result<usize, WebhookError> {
    let rows = sqlx::query(
        r#"
        SELECT d.id AS delivery_id, w.id, w.repository_id, w.scope_type, w.scope_id, w.url, w.content_type, w.secret_ciphertext, w.events, w.active, w.ssl_verify, w.created_by_user_id, w.created_at, w.updated_at
        FROM webhook_deliveries d
        JOIN webhooks w ON w.id = d.webhook_id
        WHERE d.status = 'queued' AND w.active = true AND (d.next_attempt_at IS NULL OR d.next_attempt_at <= now()) AND d.attempt_count < 3
        ORDER BY d.created_at ASC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    let mut count = 0;
    for row in rows {
        let delivery_id: Uuid = row.get("delivery_id");
        if let Some(delivery) = get_delivery(pool, delivery_id).await? {
            let hook = webhook_from_row(row)?;
            let _ = deliver_once(pool, &hook, delivery).await?;
            count += 1;
        }
    }
    Ok(count)
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
            next_attempt_at = CASE WHEN $5::bigint IS NULL THEN NULL ELSE now() + ($5::bigint * interval '1 second') END,
            delivered_at = CASE WHEN $2 = 'delivered' THEN now() ELSE delivered_at END
        WHERE id = $1
        RETURNING id, webhook_id, event, request_headers, request_body, response_status, response_headers, response_body, duration_ms, redelivery_of, delivered_at, status, attempt_count, next_attempt_at, created_at, updated_at
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

struct DeliveryOutcome {
    status: DeliveryStatus,
    response_status: Option<i32>,
    response_headers: Value,
    response_body: Option<String>,
    duration_ms: i32,
    retry_after_seconds: Option<i64>,
}

async fn deliver_once(pool: &PgPool, hook: &Webhook, delivery: WebhookDelivery) -> Result<WebhookDelivery, WebhookError> {
    let delivery_uuid = Uuid::new_v4();
    let body = delivery.request_body.clone();
    let signature = hook.secret_ciphertext.as_ref().map(|secret| hmac_sha256_signature(secret, body.as_bytes())).unwrap_or_else(|| "sha256=".to_owned());
    let headers = json!({
        "content-type": if hook.content_type == "form" { "application/x-www-form-urlencoded" } else { "application/json" },
        "x-github-event": delivery.event,
        "x-github-delivery": delivery_uuid,
        "x-hub-signature-256": signature,
    });
    sqlx::query("UPDATE webhook_deliveries SET request_headers = $2 WHERE id = $1")
        .bind(delivery.id)
        .bind(&headers)
        .execute(pool)
        .await?;

    let client = Client::builder().danger_accept_invalid_certs(!hook.ssl_verify).build().map_err(|_| WebhookError::InvalidUrl)?;
    let start = Instant::now();
    let response = client
        .post(&hook.url)
        .header("content-type", headers["content-type"].as_str().unwrap_or("application/json"))
        .header("x-github-event", &delivery.event)
        .header("x-github-delivery", delivery_uuid.to_string())
        .header("x-hub-signature-256", signature)
        .body(body)
        .send()
        .await;
    let duration_ms = start.elapsed().as_millis().min(i32::MAX as u128) as i32;

    match response {
        Ok(response) => {
            let status_code = response.status().as_u16() as i32;
            let response_headers = Value::Object(response.headers().iter().map(|(key, value)| (key.as_str().to_owned(), Value::String(value.to_str().unwrap_or("").to_owned()))).collect());
            let text = response.text().await.unwrap_or_default();
            let delivered = (200..300).contains(&status_code);
            finalize_delivery(pool, delivery.id, DeliveryOutcome { status: if delivered { DeliveryStatus::Delivered } else if delivery.attempt_count + 1 >= 3 { DeliveryStatus::Failed } else { DeliveryStatus::Queued }, response_status: Some(status_code), response_headers, response_body: Some(text), duration_ms, retry_after_seconds: if delivered || delivery.attempt_count + 1 >= 3 { None } else { Some(2_i64.pow((delivery.attempt_count + 1) as u32) * 30) } }).await
        }
        Err(error) => finalize_delivery(pool, delivery.id, DeliveryOutcome { status: if delivery.attempt_count + 1 >= 3 { DeliveryStatus::Failed } else { DeliveryStatus::Queued }, response_status: None, response_headers: json!({}), response_body: Some(error.to_string()), duration_ms, retry_after_seconds: if delivery.attempt_count + 1 >= 3 { None } else { Some(2_i64.pow((delivery.attempt_count + 1) as u32) * 30) } }).await,
    }
}

async fn insert_delivery(pool: &PgPool, webhook_id: Uuid, event: &str, request_body: String, redelivery_of: Option<Uuid>) -> Result<WebhookDelivery, WebhookError> {
    let payload: Value = serde_json::from_str(&request_body).unwrap_or_else(|_| json!({}));
    let row = sqlx::query(
        r#"
        INSERT INTO webhook_deliveries (webhook_id, event, payload, request_body, redelivery_of)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, webhook_id, event, request_headers, request_body, response_status, response_headers, response_body, duration_ms, redelivery_of, delivered_at, status, attempt_count, next_attempt_at, created_at, updated_at
        "#,
    )
    .bind(webhook_id)
    .bind(event)
    .bind(payload)
    .bind(request_body)
    .bind(redelivery_of)
    .fetch_one(pool)
    .await?;
    delivery_from_row(row)
}

async fn finalize_delivery(pool: &PgPool, delivery_id: Uuid, outcome: DeliveryOutcome) -> Result<WebhookDelivery, WebhookError> {
    let row = sqlx::query(
        r#"
        UPDATE webhook_deliveries
        SET status = $2, response_status = $3, response_headers = $4, response_body = $5, duration_ms = $6,
            attempt_count = attempt_count + 1,
            next_attempt_at = CASE WHEN $7::bigint IS NULL THEN NULL ELSE now() + ($7::bigint * interval '1 second') END,
            delivered_at = CASE WHEN $2 IN ('delivered', 'failed') THEN now() ELSE delivered_at END
        WHERE id = $1
        RETURNING id, webhook_id, event, request_headers, request_body, response_status, response_headers, response_body, duration_ms, redelivery_of, delivered_at, status, attempt_count, next_attempt_at, created_at, updated_at
        "#,
    )
    .bind(delivery_id)
    .bind(outcome.status.as_str())
    .bind(outcome.response_status)
    .bind(outcome.response_headers)
    .bind(outcome.response_body)
    .bind(outcome.duration_ms)
    .bind(outcome.retry_after_seconds)
    .fetch_one(pool)
    .await?;
    delivery_from_row(row)
}

async fn get_webhook(pool: &PgPool, webhook_id: Uuid) -> Result<Option<Webhook>, WebhookError> {
    sqlx::query(
        r#"SELECT id, repository_id, scope_type, scope_id, url, content_type, secret_ciphertext, events, active, ssl_verify, created_by_user_id, created_at, updated_at FROM webhooks WHERE id = $1"#,
    )
    .bind(webhook_id)
    .fetch_optional(pool)
    .await?
    .map(webhook_from_row)
    .transpose()
}

async fn get_delivery(pool: &PgPool, delivery_id: Uuid) -> Result<Option<WebhookDelivery>, WebhookError> {
    sqlx::query(
        r#"SELECT id, webhook_id, event, request_headers, request_body, response_status, response_headers, response_body, duration_ms, redelivery_of, delivered_at, status, attempt_count, next_attempt_at, created_at, updated_at FROM webhook_deliveries WHERE id = $1"#,
    )
    .bind(delivery_id)
    .fetch_optional(pool)
    .await?
    .map(delivery_from_row)
    .transpose()
}

async fn list_recent_deliveries(pool: &PgPool, webhook_id: Uuid, limit: i64) -> Result<Vec<WebhookDelivery>, WebhookError> {
    let rows = sqlx::query(
        r#"SELECT id, webhook_id, event, request_headers, request_body, response_status, response_headers, response_body, duration_ms, redelivery_of, delivered_at, status, attempt_count, next_attempt_at, created_at, updated_at FROM webhook_deliveries WHERE webhook_id = $1 ORDER BY created_at DESC LIMIT $2"#,
    )
    .bind(webhook_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(delivery_from_row).collect()
}

async fn require_scope_admin(pool: &PgPool, scope_type: &WebhookScopeType, scope_id: Uuid, user_id: Uuid) -> Result<(), WebhookError> {
    match scope_type {
        WebhookScopeType::Repository => require_repository_role(pool, scope_id, user_id, RepositoryRole::Admin).await,
        WebhookScopeType::Organization => {
            let allowed = sqlx::query_scalar::<_, bool>(
                r#"SELECT EXISTS (SELECT 1 FROM organization_memberships WHERE organization_id = $1 AND user_id = $2 AND role IN ('owner', 'admin'))"#,
            )
            .bind(scope_id)
            .bind(user_id)
            .fetch_one(pool)
            .await?;
            if allowed { Ok(()) } else { Err(WebhookError::AccessDenied) }
        }
    }
}

async fn require_repository_role(pool: &PgPool, repository_id: Uuid, user_id: Uuid, required_role: RepositoryRole) -> Result<(), WebhookError> {
    let permission = repository_permission_for_user(pool, repository_id, user_id).await.map_err(map_repository_error)?.ok_or(WebhookError::AccessDenied)?;
    let allowed = match required_role {
        RepositoryRole::Read => permission.role.can_read(),
        RepositoryRole::Write => permission.role.can_write(),
        RepositoryRole::Admin => permission.role.can_admin(),
        RepositoryRole::Owner => permission.role == RepositoryRole::Owner,
    };
    if allowed { Ok(()) } else { Err(WebhookError::AccessDenied) }
}

pub async fn repository_scope_by_owner_name(pool: &PgPool, owner: &str, repo: &str) -> Result<Option<Uuid>, WebhookError> {
    Ok(get_repository_by_owner_name(pool, owner, repo).await.map_err(map_repository_error)?.map(|repository| repository.id))
}

pub async fn organization_scope_by_slug(pool: &PgPool, org: &str) -> Result<Option<Uuid>, WebhookError> {
    Ok(sqlx::query_scalar::<_, Uuid>("SELECT id FROM organizations WHERE lower(slug) = lower($1)")
        .bind(org)
        .fetch_optional(pool)
        .await?)
}

fn map_repository_error(error: RepositoryError) -> WebhookError {
    match error {
        RepositoryError::Sqlx(error) => WebhookError::Sqlx(error),
        _ => WebhookError::AccessDenied,
    }
}

fn webhook_from_row(row: sqlx::postgres::PgRow) -> Result<Webhook, WebhookError> {
    let secret_ciphertext: Option<String> = row.get("secret_ciphertext");
    Ok(Webhook {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        scope_type: WebhookScopeType::try_from(row.get::<String, _>("scope_type").as_str())?,
        scope_id: row.get("scope_id"),
        url: row.get("url"),
        content_type: row.get("content_type"),
        has_secret: secret_ciphertext.as_ref().is_some_and(|secret| !secret.is_empty()),
        secret_ciphertext,
        events: row.get("events"),
        active: row.get("active"),
        ssl_verify: row.get("ssl_verify"),
        created_by_user_id: row.get("created_by_user_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn delivery_from_row(row: sqlx::postgres::PgRow) -> Result<WebhookDelivery, WebhookError> {
    let status: String = row.get("status");
    Ok(WebhookDelivery {
        id: row.get("id"),
        webhook_id: row.get("webhook_id"),
        event: row.get("event"),
        request_headers: row.get("request_headers"),
        request_body: row.get::<Option<String>, _>("request_body").unwrap_or_default(),
        response_status: row.get("response_status"),
        response_headers: row.get("response_headers"),
        response_body: row.get("response_body"),
        duration_ms: row.get("duration_ms"),
        redelivery_of: row.get("redelivery_of"),
        delivered_at: row.get("delivered_at"),
        status: DeliveryStatus::try_from(status.as_str())?,
        attempt_count: row.get("attempt_count"),
        next_attempt_at: row.get("next_attempt_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
