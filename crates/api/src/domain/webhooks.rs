use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use url::Url;
use uuid::Uuid;

use crate::jobs::enqueue_job;

use super::{
    permissions::RepositoryRole,
    repositories::{
        can_admin_repository, get_repository_by_owner_name, repository_permission_for_user,
        Repository, RepositoryError, RepositoryVisibility,
    },
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookContentType {
    Json,
    Form,
}

impl WebhookContentType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::Form => "application/x-www-form-urlencoded",
        }
    }
}

impl TryFrom<&str> for WebhookContentType {
    type Error = WebhookError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "application/json" | "json" => Ok(Self::Json),
            "application/x-www-form-urlencoded" | "form" => Ok(Self::Form),
            other => Err(WebhookError::InvalidWebhook(format!(
                "unsupported webhook content type `{other}`"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventSelection {
    Push,
    Everything,
    Selected,
}

impl WebhookEventSelection {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Push => "push",
            Self::Everything => "everything",
            Self::Selected => "selected",
        }
    }
}

impl TryFrom<&str> for WebhookEventSelection {
    type Error = WebhookError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "push" => Ok(Self::Push),
            "everything" => Ok(Self::Everything),
            "selected" => Ok(Self::Selected),
            other => Err(WebhookError::InvalidWebhook(format!(
                "unsupported webhook event selection `{other}`"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryWebhookSettings {
    pub repository_id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub visibility: RepositoryVisibility,
    pub viewer_permission: String,
    pub can_edit: bool,
    pub event_definitions: Vec<WebhookEventDefinition>,
    pub hooks: Vec<RepositoryWebhookSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationWebhookSettings {
    pub organization_id: Uuid,
    pub slug: String,
    pub name: String,
    pub viewer_role: String,
    pub can_edit: bool,
    pub event_definitions: Vec<WebhookEventDefinition>,
    pub hooks: Vec<RepositoryWebhookSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WebhookEventDefinition {
    pub name: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryWebhookSummary {
    pub id: Uuid,
    pub payload_url: String,
    pub content_type: WebhookContentType,
    pub ssl_verify: bool,
    pub event_selection: WebhookEventSelection,
    pub events: Vec<String>,
    pub active: bool,
    pub disabled_reason: Option<String>,
    pub secret_configured: bool,
    pub secret_updated_at: Option<DateTime<Utc>>,
    pub latest_delivery: Option<WebhookDeliverySummary>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryWebhookDetail {
    pub hook: RepositoryWebhookSummary,
    pub deliveries: Vec<WebhookDeliverySummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WebhookDeliverySummary {
    pub id: Uuid,
    pub guid: Uuid,
    pub event: String,
    pub status: DeliveryStatus,
    pub attempt_count: i32,
    pub response_status: Option<i32>,
    pub duration_ms: Option<i64>,
    pub redelivery_of_id: Option<Uuid>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WebhookDeliveryDetail {
    pub summary: WebhookDeliverySummary,
    pub request_headers: Value,
    pub request_body_excerpt: Option<String>,
    pub request_body_storage_key: Option<String>,
    pub response_headers: Value,
    pub response_body_excerpt: Option<String>,
    pub response_body_storage_key: Option<String>,
    pub terminal_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueuedWebhookDelivery {
    pub webhook_id: Uuid,
    pub delivery_id: Uuid,
    pub event: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WebhookDeliveryWorkerResult {
    pub status: DeliveryStatus,
    pub response_status: Option<i32>,
    pub response_headers: Value,
    pub response_body_excerpt: Option<String>,
    pub duration_ms: Option<i64>,
    pub terminal_error: Option<String>,
    pub retry_after_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookMutation {
    pub payload_url: String,
    pub content_type: Option<WebhookContentType>,
    pub secret: Option<String>,
    pub ssl_verify: Option<bool>,
    pub event_selection: Option<WebhookEventSelection>,
    pub events: Option<Vec<String>>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WebhookPingResult {
    pub settings: RepositoryWebhookSettings,
    pub delivery: WebhookDeliverySummary,
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
    #[error("repository was not found")]
    RepositoryNotFound,
    #[error("user does not have repository admin access")]
    RepositoryAccessDenied,
    #[error("organization was not found")]
    OrganizationNotFound,
    #[error("user does not have organization owner access")]
    OrganizationAccessDenied,
    #[error("webhook was not found")]
    WebhookNotFound,
    #[error("webhook delivery was not found")]
    DeliveryNotFound,
    #[error("invalid webhook: {0}")]
    InvalidWebhook(String),
    #[error("invalid delivery status `{0}`")]
    InvalidDeliveryStatus(String),
    #[error("webhook delivery queue failed: {0}")]
    DeliveryQueue(String),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, Clone)]
struct OrganizationHookScope {
    id: Uuid,
    slug: String,
    name: String,
    viewer_role: String,
}

pub async fn enqueue_repository_webhook_event(
    pool: &PgPool,
    repository_id: Uuid,
    event: &str,
    payload: Value,
) -> Result<Vec<QueuedWebhookDelivery>, WebhookError> {
    let event = event.trim();
    if event.is_empty() {
        return Err(WebhookError::InvalidWebhook(
            "webhook event must not be blank".to_owned(),
        ));
    }
    validate_event_for_delivery(event)?;

    let rows = sqlx::query(
        r#"
        SELECT webhooks.id
        FROM webhooks
        LEFT JOIN repositories ON repositories.id = $1
        WHERE (
             webhooks.repository_id = $1
             OR (
             webhooks.organization_id IS NOT NULL
             AND webhooks.organization_id = repositories.owner_organization_id
           )
          )
          AND active = true
          AND (events @> ARRAY[$2]::text[] OR events @> ARRAY['*']::text[])
        ORDER BY created_at ASC
        "#,
    )
    .bind(repository_id)
    .bind(event)
    .fetch_all(pool)
    .await?;

    let mut queued = Vec::with_capacity(rows.len());
    for row in rows {
        let hook_id: Uuid = row.get("id");
        let delivery_id = insert_delivery(
            pool,
            hook_id,
            event,
            json!({
                "event": event,
                "repositoryId": repository_id,
                "payload": payload
            }),
            None,
        )
        .await?;
        enqueue_job(
            pool,
            "webhook-delivery",
            &delivery_id.to_string(),
            json!({
                "deliveryId": delivery_id,
                "webhookId": hook_id,
                "repositoryId": repository_id,
                "event": event
            }),
        )
        .await
        .map_err(|error| WebhookError::DeliveryQueue(error.to_string()))?;
        queued.push(QueuedWebhookDelivery {
            webhook_id: hook_id,
            delivery_id,
            event: event.to_owned(),
        });
    }

    Ok(queued)
}

pub async fn repository_webhook_settings_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryWebhookSettings>, WebhookError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name)
        .await
        .map_err(map_repository_error)?
    else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    repository_webhook_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn organization_webhook_settings_for_actor_by_slug(
    pool: &PgPool,
    actor_user_id: Uuid,
    slug: &str,
) -> Result<Option<OrganizationWebhookSettings>, WebhookError> {
    let Some(organization) = organization_hook_scope(pool, slug, actor_user_id).await? else {
        return Ok(None);
    };
    organization_webhook_settings_for_scope(pool, &organization)
        .await
        .map(Some)
}

pub async fn organization_webhook_detail_for_actor_by_slug(
    pool: &PgPool,
    actor_user_id: Uuid,
    slug: &str,
    hook_id: Uuid,
) -> Result<Option<RepositoryWebhookDetail>, WebhookError> {
    let Some(organization) = organization_hook_scope(pool, slug, actor_user_id).await? else {
        return Ok(None);
    };
    let Some(hook) = organization_webhook_summary(pool, organization.id, hook_id).await? else {
        return Err(WebhookError::WebhookNotFound);
    };
    let deliveries = list_delivery_summaries(pool, hook_id, 30).await?;
    Ok(Some(RepositoryWebhookDetail { hook, deliveries }))
}

pub async fn organization_webhook_delivery_for_actor_by_slug(
    pool: &PgPool,
    actor_user_id: Uuid,
    slug: &str,
    hook_id: Uuid,
    delivery_id: Uuid,
) -> Result<Option<WebhookDeliveryDetail>, WebhookError> {
    let Some(organization) = organization_hook_scope(pool, slug, actor_user_id).await? else {
        return Ok(None);
    };
    if organization_webhook_summary(pool, organization.id, hook_id)
        .await?
        .is_none()
    {
        return Err(WebhookError::WebhookNotFound);
    }
    delivery_detail(pool, hook_id, delivery_id)
        .await?
        .ok_or(WebhookError::DeliveryNotFound)
        .map(Some)
}

pub async fn create_organization_webhook_by_slug(
    pool: &PgPool,
    actor_user_id: Uuid,
    slug: &str,
    mutation: WebhookMutation,
) -> Result<Option<(OrganizationWebhookSettings, WebhookDeliverySummary)>, WebhookError> {
    let Some(organization) = organization_hook_scope(pool, slug, actor_user_id).await? else {
        return Ok(None);
    };
    let normalized = normalize_mutation(mutation, None)?;
    let secret_hash = normalized.secret.as_deref().map(encode_secret_material);
    let mut transaction = pool.begin().await?;
    let hook_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO webhooks (
            organization_id, url, secret_hash, events, active, created_by_user_id,
            content_type, ssl_verify, event_selection, secret_configured, secret_updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, CASE WHEN $10 THEN now() ELSE NULL END)
        RETURNING id
        "#,
    )
    .bind(organization.id)
    .bind(&normalized.payload_url)
    .bind(&secret_hash)
    .bind(&normalized.events)
    .bind(normalized.active)
    .bind(actor_user_id)
    .bind(normalized.content_type.as_str())
    .bind(normalized.ssl_verify)
    .bind(normalized.event_selection.as_str())
    .bind(secret_hash.is_some())
    .fetch_one(&mut *transaction)
    .await?;
    let delivery_id = insert_delivery_tx(
        &mut transaction,
        hook_id,
        "ping",
        json!({
            "zen": "Keep it logically awesome.",
            "hookId": hook_id,
            "organization": { "id": organization.id, "login": organization.slug }
        }),
        None,
    )
    .await?;
    insert_organization_webhook_audit_tx(
        &mut transaction,
        organization.id,
        actor_user_id,
        "organization.webhook.create",
        audit_hook_state(
            &normalized.payload_url,
            &normalized.events,
            normalized.active,
            secret_hash.is_some(),
        ),
    )
    .await?;
    transaction.commit().await?;
    let settings = organization_webhook_settings_for_scope(pool, &organization).await?;
    let delivery = delivery_summary_by_id(pool, hook_id, delivery_id)
        .await?
        .ok_or(WebhookError::DeliveryNotFound)?;
    Ok(Some((settings, delivery)))
}

pub async fn repository_webhook_detail_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    hook_id: Uuid,
) -> Result<Option<RepositoryWebhookDetail>, WebhookError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name)
        .await
        .map_err(map_repository_error)?
    else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    let Some(hook) = repository_webhook_summary(pool, repository.id, hook_id).await? else {
        return Err(WebhookError::WebhookNotFound);
    };
    let deliveries = list_delivery_summaries(pool, hook_id, 30).await?;
    Ok(Some(RepositoryWebhookDetail { hook, deliveries }))
}

pub async fn repository_webhook_delivery_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    hook_id: Uuid,
    delivery_id: Uuid,
) -> Result<Option<WebhookDeliveryDetail>, WebhookError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name)
        .await
        .map_err(map_repository_error)?
    else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    if repository_webhook_summary(pool, repository.id, hook_id)
        .await?
        .is_none()
    {
        return Err(WebhookError::WebhookNotFound);
    }
    delivery_detail(pool, hook_id, delivery_id)
        .await?
        .ok_or(WebhookError::DeliveryNotFound)
        .map(Some)
}

pub async fn create_repository_webhook_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    mutation: WebhookMutation,
) -> Result<Option<WebhookPingResult>, WebhookError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name)
        .await
        .map_err(map_repository_error)?
    else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    let normalized = normalize_mutation(mutation, None)?;
    let secret_hash = normalized.secret.as_deref().map(encode_secret_material);
    let mut transaction = pool.begin().await?;
    let hook_row = sqlx::query(
        r#"
        INSERT INTO webhooks (
            repository_id, url, secret_hash, events, active, created_by_user_id,
            content_type, ssl_verify, event_selection, secret_configured, secret_updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, CASE WHEN $10 THEN now() ELSE NULL END)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(&normalized.payload_url)
    .bind(&secret_hash)
    .bind(&normalized.events)
    .bind(normalized.active)
    .bind(actor_user_id)
    .bind(normalized.content_type.as_str())
    .bind(normalized.ssl_verify)
    .bind(normalized.event_selection.as_str())
    .bind(secret_hash.is_some())
    .fetch_one(&mut *transaction)
    .await?;
    let hook_id: Uuid = hook_row.get("id");
    let delivery_id = insert_delivery_tx(
        &mut transaction,
        hook_id,
        "ping",
        json!({
            "zen": "Keep it logically awesome.",
            "hookId": hook_id,
            "repository": {
                "id": repository.id,
                "owner": repository.owner_login,
                "name": repository.name
            }
        }),
        None,
    )
    .await?;
    insert_webhook_audit_tx(
        &mut transaction,
        repository.id,
        actor_user_id,
        "repository.webhook.create",
        vec![
            "payloadUrl".to_owned(),
            "events".to_owned(),
            "active".to_owned(),
        ],
        json!(null),
        audit_hook_state(
            &normalized.payload_url,
            &normalized.events,
            normalized.active,
            secret_hash.is_some(),
        ),
    )
    .await?;
    transaction.commit().await?;

    let settings = repository_webhook_settings_for_repository(pool, &repository, actor_user_id)
        .await?
        .ok_or(WebhookError::RepositoryNotFound)?;
    let delivery = delivery_summary_by_id(pool, hook_id, delivery_id)
        .await?
        .ok_or(WebhookError::DeliveryNotFound)?;
    Ok(Some(WebhookPingResult { settings, delivery }))
}

pub async fn update_repository_webhook_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    hook_id: Uuid,
    mutation: WebhookMutation,
) -> Result<Option<RepositoryWebhookSettings>, WebhookError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name)
        .await
        .map_err(map_repository_error)?
    else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    let before = repository_webhook_summary(pool, repository.id, hook_id)
        .await?
        .ok_or(WebhookError::WebhookNotFound)?;
    let normalized = normalize_mutation(mutation, Some(&before))?;
    let secret_hash = normalized.secret.as_deref().map(encode_secret_material);
    let mut changed_fields = vec![
        "payloadUrl".to_owned(),
        "contentType".to_owned(),
        "sslVerify".to_owned(),
        "eventSelection".to_owned(),
        "events".to_owned(),
        "active".to_owned(),
    ];
    if secret_hash.is_some() {
        changed_fields.push("secret".to_owned());
    }
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE webhooks
        SET url = $3,
            content_type = $4,
            ssl_verify = $5,
            event_selection = $6,
            events = $7,
            active = $8,
            secret_hash = COALESCE($9, secret_hash),
            secret_configured = CASE WHEN $9::text IS NULL THEN secret_configured ELSE true END,
            secret_updated_at = CASE WHEN $9::text IS NULL THEN secret_updated_at ELSE now() END,
            disabled_reason = CASE WHEN $8 THEN NULL ELSE COALESCE(disabled_reason, 'disabled by repository admin') END
        WHERE repository_id = $1 AND id = $2
        "#,
    )
    .bind(repository.id)
    .bind(hook_id)
    .bind(&normalized.payload_url)
    .bind(normalized.content_type.as_str())
    .bind(normalized.ssl_verify)
    .bind(normalized.event_selection.as_str())
    .bind(&normalized.events)
    .bind(normalized.active)
    .bind(&secret_hash)
    .execute(&mut *transaction)
    .await?;
    insert_webhook_audit_tx(
        &mut transaction,
        repository.id,
        actor_user_id,
        "repository.webhook.update",
        changed_fields,
        json!({
            "id": before.id,
            "payloadUrl": before.payload_url,
            "events": before.events,
            "active": before.active,
            "secretConfigured": before.secret_configured
        }),
        audit_hook_state(
            &normalized.payload_url,
            &normalized.events,
            normalized.active,
            before.secret_configured || secret_hash.is_some(),
        ),
    )
    .await?;
    transaction.commit().await?;
    repository_webhook_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn update_organization_webhook_by_slug(
    pool: &PgPool,
    actor_user_id: Uuid,
    slug: &str,
    hook_id: Uuid,
    mutation: WebhookMutation,
) -> Result<Option<OrganizationWebhookSettings>, WebhookError> {
    let Some(organization) = organization_hook_scope(pool, slug, actor_user_id).await? else {
        return Ok(None);
    };
    let before = organization_webhook_summary(pool, organization.id, hook_id)
        .await?
        .ok_or(WebhookError::WebhookNotFound)?;
    let normalized = normalize_mutation(mutation, Some(&before))?;
    let secret_hash = normalized.secret.as_deref().map(encode_secret_material);
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE webhooks
        SET url = $3,
            content_type = $4,
            ssl_verify = $5,
            event_selection = $6,
            events = $7,
            active = $8,
            secret_hash = COALESCE($9, secret_hash),
            secret_configured = CASE WHEN $9::text IS NULL THEN secret_configured ELSE true END,
            secret_updated_at = CASE WHEN $9::text IS NULL THEN secret_updated_at ELSE now() END,
            disabled_reason = CASE WHEN $8 THEN NULL ELSE COALESCE(disabled_reason, 'disabled by organization owner') END
        WHERE organization_id = $1 AND id = $2
        "#,
    )
    .bind(organization.id)
    .bind(hook_id)
    .bind(&normalized.payload_url)
    .bind(normalized.content_type.as_str())
    .bind(normalized.ssl_verify)
    .bind(normalized.event_selection.as_str())
    .bind(&normalized.events)
    .bind(normalized.active)
    .bind(&secret_hash)
    .execute(&mut *transaction)
    .await?;
    insert_organization_webhook_audit_tx(
        &mut transaction,
        organization.id,
        actor_user_id,
        "organization.webhook.update",
        audit_hook_state(
            &normalized.payload_url,
            &normalized.events,
            normalized.active,
            before.secret_configured || secret_hash.is_some(),
        ),
    )
    .await?;
    transaction.commit().await?;
    organization_webhook_settings_for_scope(pool, &organization)
        .await
        .map(Some)
}

pub async fn delete_organization_webhook_by_slug(
    pool: &PgPool,
    actor_user_id: Uuid,
    slug: &str,
    hook_id: Uuid,
) -> Result<Option<OrganizationWebhookSettings>, WebhookError> {
    let Some(organization) = organization_hook_scope(pool, slug, actor_user_id).await? else {
        return Ok(None);
    };
    organization_webhook_summary(pool, organization.id, hook_id)
        .await?
        .ok_or(WebhookError::WebhookNotFound)?;
    let mut transaction = pool.begin().await?;
    sqlx::query("DELETE FROM webhooks WHERE organization_id = $1 AND id = $2")
        .bind(organization.id)
        .bind(hook_id)
        .execute(&mut *transaction)
        .await?;
    insert_organization_webhook_audit_tx(
        &mut transaction,
        organization.id,
        actor_user_id,
        "organization.webhook.delete",
        json!({ "hookId": hook_id }),
    )
    .await?;
    transaction.commit().await?;
    organization_webhook_settings_for_scope(pool, &organization)
        .await
        .map(Some)
}

pub async fn ping_organization_webhook_by_slug(
    pool: &PgPool,
    actor_user_id: Uuid,
    slug: &str,
    hook_id: Uuid,
) -> Result<Option<(OrganizationWebhookSettings, WebhookDeliverySummary)>, WebhookError> {
    create_organization_hook_delivery_by_slug(pool, actor_user_id, slug, hook_id, "ping", None)
        .await
}

pub async fn redeliver_organization_webhook_delivery_by_slug(
    pool: &PgPool,
    actor_user_id: Uuid,
    slug: &str,
    hook_id: Uuid,
    delivery_id: Uuid,
) -> Result<Option<(OrganizationWebhookSettings, WebhookDeliverySummary)>, WebhookError> {
    create_organization_hook_delivery_by_slug(
        pool,
        actor_user_id,
        slug,
        hook_id,
        "redelivery",
        Some(delivery_id),
    )
    .await
}

pub async fn delete_repository_webhook_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    hook_id: Uuid,
) -> Result<Option<RepositoryWebhookSettings>, WebhookError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name)
        .await
        .map_err(map_repository_error)?
    else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    let before = repository_webhook_summary(pool, repository.id, hook_id)
        .await?
        .ok_or(WebhookError::WebhookNotFound)?;
    let mut transaction = pool.begin().await?;
    sqlx::query("DELETE FROM webhooks WHERE repository_id = $1 AND id = $2")
        .bind(repository.id)
        .bind(hook_id)
        .execute(&mut *transaction)
        .await?;
    insert_webhook_audit_tx(
        &mut transaction,
        repository.id,
        actor_user_id,
        "repository.webhook.delete",
        vec!["deleted".to_owned()],
        json!({
            "id": before.id,
            "payloadUrl": before.payload_url,
            "events": before.events,
            "active": before.active,
            "secretConfigured": before.secret_configured
        }),
        json!(null),
    )
    .await?;
    transaction.commit().await?;
    repository_webhook_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn ping_repository_webhook_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    hook_id: Uuid,
) -> Result<Option<WebhookPingResult>, WebhookError> {
    create_hook_delivery_by_owner_name(
        pool,
        actor_user_id,
        owner_login,
        name,
        hook_id,
        "ping",
        None,
    )
    .await
}

pub async fn redeliver_repository_webhook_delivery_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    hook_id: Uuid,
    delivery_id: Uuid,
) -> Result<Option<WebhookPingResult>, WebhookError> {
    create_hook_delivery_by_owner_name(
        pool,
        actor_user_id,
        owner_login,
        name,
        hook_id,
        "redelivery",
        Some(delivery_id),
    )
    .await
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

pub async fn record_webhook_delivery_worker_result(
    pool: &PgPool,
    delivery_id: Uuid,
    result: WebhookDeliveryWorkerResult,
) -> Result<WebhookDelivery, WebhookError> {
    let row = sqlx::query(
        r#"
        UPDATE webhook_deliveries
        SET status = $2,
            response_status = $3,
            response_headers = $4,
            response_body = $5,
            duration_ms = $6,
            terminal_error = $7,
            attempt_count = attempt_count + 1,
            next_attempt_at = CASE
                WHEN $8::bigint IS NULL THEN NULL
                ELSE now() + ($8::bigint * interval '1 second')
            END,
            delivered_at = CASE WHEN $2 = 'delivered' THEN now() ELSE delivered_at END
        WHERE id = $1
        RETURNING id, webhook_id, event, payload, status, attempt_count, next_attempt_at,
                  response_status, response_body, delivered_at, created_at, updated_at
        "#,
    )
    .bind(delivery_id)
    .bind(result.status.as_str())
    .bind(result.response_status)
    .bind(result.response_headers)
    .bind(&result.response_body_excerpt)
    .bind(result.duration_ms)
    .bind(&result.terminal_error)
    .bind(result.retry_after_seconds)
    .fetch_optional(pool)
    .await?
    .ok_or(WebhookError::DeliveryNotFound)?;

    delivery_from_row(row)
}

pub async fn mark_webhook_delivery_request(
    pool: &PgPool,
    delivery_id: Uuid,
    request_headers: Value,
    request_body_excerpt: String,
    request_body_storage_key: Option<String>,
) -> Result<(), WebhookError> {
    sqlx::query(
        r#"
        UPDATE webhook_deliveries
        SET request_headers = $2,
            request_body_excerpt = $3,
            request_body_storage_key = $4
        WHERE id = $1
        "#,
    )
    .bind(delivery_id)
    .bind(request_headers)
    .bind(request_body_excerpt)
    .bind(request_body_storage_key)
    .execute(pool)
    .await?;
    Ok(())
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

async fn require_repository_admin(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<(), WebhookError> {
    if can_admin_repository(pool, repository, actor_user_id)
        .await
        .map_err(map_repository_error)?
    {
        Ok(())
    } else {
        Err(WebhookError::RepositoryAccessDenied)
    }
}

async fn repository_webhook_settings_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<Option<RepositoryWebhookSettings>, WebhookError> {
    let viewer_permission = repository_permission_label(pool, repository, actor_user_id).await?;
    let hooks = list_hook_summaries(pool, repository.id).await?;
    Ok(Some(RepositoryWebhookSettings {
        repository_id: repository.id,
        owner_login: repository.owner_login.clone(),
        name: repository.name.clone(),
        visibility: repository.visibility.clone(),
        viewer_permission,
        can_edit: true,
        event_definitions: webhook_event_definitions(),
        hooks,
    }))
}

async fn repository_permission_label(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<String, WebhookError> {
    if repository.owner_user_id == Some(actor_user_id) {
        return Ok("owner".to_owned());
    }
    let permission = repository_permission_for_user(pool, repository.id, actor_user_id)
        .await
        .map_err(map_repository_error)?;
    Ok(permission
        .map(|permission| permission.role.as_str().to_owned())
        .unwrap_or_else(|| "admin".to_owned()))
}

async fn organization_hook_scope(
    pool: &PgPool,
    slug: &str,
    actor_user_id: Uuid,
) -> Result<Option<OrganizationHookScope>, WebhookError> {
    let row = sqlx::query(
        r#"
        SELECT organizations.id,
               organizations.slug,
               organizations.display_name,
               organizations.profile_visibility,
               organization_memberships.role
        FROM organizations
        LEFT JOIN organization_memberships
          ON organization_memberships.organization_id = organizations.id
         AND organization_memberships.user_id = $2
        WHERE lower(organizations.slug) = lower($1)
        "#,
    )
    .bind(slug)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let visibility: String = row.get("profile_visibility");
    let role: Option<String> = row.get("role");
    if visibility == "private" && role.is_none() {
        return Ok(None);
    }
    let Some(role) = role else {
        return Err(WebhookError::OrganizationAccessDenied);
    };
    if role != "owner" {
        return Err(WebhookError::OrganizationAccessDenied);
    }
    Ok(Some(OrganizationHookScope {
        id: row.get("id"),
        slug: row.get("slug"),
        name: row.get("display_name"),
        viewer_role: role,
    }))
}

async fn organization_webhook_settings_for_scope(
    pool: &PgPool,
    organization: &OrganizationHookScope,
) -> Result<OrganizationWebhookSettings, WebhookError> {
    Ok(OrganizationWebhookSettings {
        organization_id: organization.id,
        slug: organization.slug.clone(),
        name: organization.name.clone(),
        viewer_role: organization.viewer_role.clone(),
        can_edit: true,
        event_definitions: webhook_event_definitions(),
        hooks: list_organization_hook_summaries(pool, organization.id).await?,
    })
}

fn webhook_event_definitions() -> Vec<WebhookEventDefinition> {
    [
        ("push", "Pushes", "Git branch and tag updates."),
        (
            "issues",
            "Issues",
            "Issue open, edit, close, label, and comment activity.",
        ),
        (
            "pull_request",
            "Pull requests",
            "Pull request lifecycle activity.",
        ),
        (
            "pull_request_review",
            "Pull request reviews",
            "Pull request review submit, edit, and dismissal activity.",
        ),
        (
            "release",
            "Releases",
            "Release publish, edit, and delete activity.",
        ),
        (
            "issue_comment",
            "Issue comments",
            "Issue and pull request conversation activity.",
        ),
        (
            "workflow_run",
            "Workflow runs",
            "Actions workflow run state changes.",
        ),
        (
            "check_run",
            "Check runs",
            "Check run creation, rerequest, completion, and annotations.",
        ),
        (
            "package",
            "Packages",
            "Package publish and delete activity.",
        ),
        (
            "page_build",
            "Pages",
            "Pages build and deployment activity.",
        ),
    ]
    .into_iter()
    .map(|(name, label, description)| WebhookEventDefinition {
        name: name.to_owned(),
        label: label.to_owned(),
        description: description.to_owned(),
    })
    .collect()
}

struct NormalizedWebhookMutation {
    payload_url: String,
    content_type: WebhookContentType,
    secret: Option<String>,
    ssl_verify: bool,
    event_selection: WebhookEventSelection,
    events: Vec<String>,
    active: bool,
}

fn normalize_mutation(
    mutation: WebhookMutation,
    current: Option<&RepositoryWebhookSummary>,
) -> Result<NormalizedWebhookMutation, WebhookError> {
    let payload_url = mutation.payload_url.trim().to_owned();
    validate_payload_url(&payload_url)?;
    let content_type = mutation
        .content_type
        .or_else(|| current.map(|hook| hook.content_type.clone()))
        .unwrap_or(WebhookContentType::Json);
    let event_selection = mutation
        .event_selection
        .or_else(|| current.map(|hook| hook.event_selection.clone()))
        .unwrap_or(WebhookEventSelection::Push);
    let mut events = match event_selection {
        WebhookEventSelection::Push => vec!["push".to_owned()],
        WebhookEventSelection::Everything => vec!["*".to_owned()],
        WebhookEventSelection::Selected => mutation
            .events
            .or_else(|| current.map(|hook| hook.events.clone()))
            .unwrap_or_default(),
    };
    events = normalize_events(events, &event_selection)?;
    let secret = mutation
        .secret
        .map(|secret| secret.trim().to_owned())
        .filter(|secret| !secret.is_empty());
    if let Some(secret) = secret.as_ref() {
        if secret.len() < 8 || secret.len() > 256 {
            return Err(WebhookError::InvalidWebhook(
                "webhook secret must be between 8 and 256 characters".to_owned(),
            ));
        }
    }
    Ok(NormalizedWebhookMutation {
        payload_url,
        content_type,
        secret,
        ssl_verify: mutation
            .ssl_verify
            .or_else(|| current.map(|hook| hook.ssl_verify))
            .unwrap_or(true),
        event_selection,
        events,
        active: mutation
            .active
            .or_else(|| current.map(|hook| hook.active))
            .unwrap_or(true),
    })
}

fn validate_payload_url(payload_url: &str) -> Result<(), WebhookError> {
    let url = Url::parse(payload_url).map_err(|_| {
        WebhookError::InvalidWebhook("webhook payload URL must be a valid HTTPS URL".to_owned())
    })?;
    if url.scheme() != "https" {
        return Err(WebhookError::InvalidWebhook(
            "webhook payload URL must use HTTPS".to_owned(),
        ));
    }
    if url.host_str().is_none() {
        return Err(WebhookError::InvalidWebhook(
            "webhook payload URL must include a host".to_owned(),
        ));
    }
    Ok(())
}

fn normalize_events(
    events: Vec<String>,
    selection: &WebhookEventSelection,
) -> Result<Vec<String>, WebhookError> {
    if matches!(selection, WebhookEventSelection::Everything) {
        return Ok(vec!["*".to_owned()]);
    }
    if matches!(selection, WebhookEventSelection::Push) {
        return Ok(vec!["push".to_owned()]);
    }
    let supported = webhook_event_definitions()
        .into_iter()
        .map(|definition| definition.name)
        .collect::<std::collections::BTreeSet<_>>();
    let normalized = events
        .into_iter()
        .map(|event| event.trim().to_owned())
        .filter(|event| !event.is_empty())
        .collect::<std::collections::BTreeSet<_>>();
    if normalized.is_empty() {
        return Err(WebhookError::InvalidWebhook(
            "at least one webhook event must be selected".to_owned(),
        ));
    }
    if let Some(unsupported) = normalized.iter().find(|event| !supported.contains(*event)) {
        return Err(WebhookError::InvalidWebhook(format!(
            "unsupported webhook event `{unsupported}`"
        )));
    }
    Ok(normalized.into_iter().collect())
}

fn encode_secret_material(secret: &str) -> String {
    format!("secret:v1:{}", STANDARD.encode(secret.as_bytes()))
}

fn audit_hook_state(
    payload_url: &str,
    events: &[String],
    active: bool,
    secret_configured: bool,
) -> Value {
    json!({
        "payloadUrl": payload_url,
        "events": events,
        "active": active,
        "secretConfigured": secret_configured
    })
}

async fn insert_webhook_audit_tx(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    repository_id: Uuid,
    actor_user_id: Uuid,
    event_type: &str,
    changed_fields: Vec<String>,
    before_state: Value,
    after_state: Value,
) -> Result<(), WebhookError> {
    sqlx::query(
        r#"
        INSERT INTO repository_settings_audit_events (
            repository_id, actor_user_id, event_type, changed_fields, before_state, after_state
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(repository_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(changed_fields)
    .bind(before_state)
    .bind(after_state)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

async fn insert_organization_webhook_audit_tx(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    organization_id: Uuid,
    actor_user_id: Uuid,
    event_type: &str,
    metadata: Value,
) -> Result<(), WebhookError> {
    sqlx::query(
        r#"
        INSERT INTO organization_audit_events (
            organization_id, actor_user_id, event_type, metadata
        )
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(organization_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(metadata)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

async fn insert_delivery_tx(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    hook_id: Uuid,
    event: &str,
    payload: Value,
    redelivery_of_id: Option<Uuid>,
) -> Result<Uuid, WebhookError> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO webhook_deliveries (
            webhook_id, event, payload, request_headers, request_body_excerpt, redelivery_of_id
        )
        VALUES ($1, $2, $3, '{}'::jsonb, left($3::text, 4096), $4)
        RETURNING id
        "#,
    )
    .bind(hook_id)
    .bind(event)
    .bind(payload)
    .bind(redelivery_of_id)
    .fetch_one(&mut **transaction)
    .await?;
    Ok(id)
}

async fn insert_delivery(
    pool: &PgPool,
    hook_id: Uuid,
    event: &str,
    payload: Value,
    redelivery_of_id: Option<Uuid>,
) -> Result<Uuid, WebhookError> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO webhook_deliveries (
            webhook_id, event, payload, request_headers, request_body_excerpt, redelivery_of_id
        )
        VALUES ($1, $2, $3, '{}'::jsonb, left($3::text, 4096), $4)
        RETURNING id
        "#,
    )
    .bind(hook_id)
    .bind(event)
    .bind(payload)
    .bind(redelivery_of_id)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

fn validate_event_for_delivery(event: &str) -> Result<(), WebhookError> {
    if event == "ping" || event == "redelivery" {
        return Ok(());
    }
    let supported = webhook_event_definitions()
        .into_iter()
        .map(|definition| definition.name)
        .collect::<std::collections::BTreeSet<_>>();
    if supported.contains(event) {
        Ok(())
    } else {
        Err(WebhookError::InvalidWebhook(format!(
            "unsupported webhook event `{event}`"
        )))
    }
}

async fn create_hook_delivery_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    hook_id: Uuid,
    event: &str,
    redelivery_of_id: Option<Uuid>,
) -> Result<Option<WebhookPingResult>, WebhookError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name)
        .await
        .map_err(map_repository_error)?
    else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    if repository_webhook_summary(pool, repository.id, hook_id)
        .await?
        .is_none()
    {
        return Err(WebhookError::WebhookNotFound);
    }
    if let Some(original_id) = redelivery_of_id {
        if delivery_summary_by_id(pool, hook_id, original_id)
            .await?
            .is_none()
        {
            return Err(WebhookError::DeliveryNotFound);
        }
    }
    let mut transaction = pool.begin().await?;
    let delivery_id = insert_delivery_tx(
        &mut transaction,
        hook_id,
        event,
        json!({
            "hookId": hook_id,
            "repository": {
                "id": repository.id,
                "owner": repository.owner_login,
                "name": repository.name
            },
            "redeliveryOfId": redelivery_of_id
        }),
        redelivery_of_id,
    )
    .await?;
    insert_webhook_audit_tx(
        &mut transaction,
        repository.id,
        actor_user_id,
        if redelivery_of_id.is_some() {
            "repository.webhook.redeliver"
        } else {
            "repository.webhook.ping"
        },
        vec!["delivery".to_owned()],
        json!({ "hookId": hook_id, "redeliveryOfId": redelivery_of_id }),
        json!({ "deliveryId": delivery_id }),
    )
    .await?;
    transaction.commit().await?;

    let settings = repository_webhook_settings_for_repository(pool, &repository, actor_user_id)
        .await?
        .ok_or(WebhookError::RepositoryNotFound)?;
    let delivery = delivery_summary_by_id(pool, hook_id, delivery_id)
        .await?
        .ok_or(WebhookError::DeliveryNotFound)?;
    Ok(Some(WebhookPingResult { settings, delivery }))
}

async fn create_organization_hook_delivery_by_slug(
    pool: &PgPool,
    actor_user_id: Uuid,
    slug: &str,
    hook_id: Uuid,
    event: &str,
    redelivery_of_id: Option<Uuid>,
) -> Result<Option<(OrganizationWebhookSettings, WebhookDeliverySummary)>, WebhookError> {
    let Some(organization) = organization_hook_scope(pool, slug, actor_user_id).await? else {
        return Ok(None);
    };
    if organization_webhook_summary(pool, organization.id, hook_id)
        .await?
        .is_none()
    {
        return Err(WebhookError::WebhookNotFound);
    }
    if let Some(original_id) = redelivery_of_id {
        if delivery_summary_by_id(pool, hook_id, original_id)
            .await?
            .is_none()
        {
            return Err(WebhookError::DeliveryNotFound);
        }
    }
    let mut transaction = pool.begin().await?;
    let delivery_id = insert_delivery_tx(
        &mut transaction,
        hook_id,
        event,
        json!({
            "hookId": hook_id,
            "organization": { "id": organization.id, "login": organization.slug },
            "redeliveryOfId": redelivery_of_id
        }),
        redelivery_of_id,
    )
    .await?;
    insert_organization_webhook_audit_tx(
        &mut transaction,
        organization.id,
        actor_user_id,
        if redelivery_of_id.is_some() {
            "organization.webhook.redeliver"
        } else {
            "organization.webhook.ping"
        },
        json!({ "hookId": hook_id, "deliveryId": delivery_id }),
    )
    .await?;
    transaction.commit().await?;
    let settings = organization_webhook_settings_for_scope(pool, &organization).await?;
    let delivery = delivery_summary_by_id(pool, hook_id, delivery_id)
        .await?
        .ok_or(WebhookError::DeliveryNotFound)?;
    Ok(Some((settings, delivery)))
}

async fn list_hook_summaries(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<RepositoryWebhookSummary>, WebhookError> {
    let rows = sqlx::query(
        r#"
        SELECT
            webhooks.id,
            webhooks.url,
            webhooks.content_type,
            webhooks.ssl_verify,
            webhooks.event_selection,
            webhooks.events,
            webhooks.active,
            webhooks.disabled_reason,
            webhooks.secret_configured,
            webhooks.secret_updated_at,
            webhooks.created_at,
            webhooks.updated_at,
            latest.id AS latest_delivery_id,
            latest.delivery_guid AS latest_delivery_guid,
            latest.event AS latest_event,
            latest.status AS latest_status,
            latest.attempt_count AS latest_attempt_count,
            latest.response_status AS latest_response_status,
            latest.duration_ms AS latest_duration_ms,
            latest.redelivery_of_id AS latest_redelivery_of_id,
            latest.delivered_at AS latest_delivered_at,
            latest.created_at AS latest_created_at,
            latest.updated_at AS latest_updated_at
        FROM webhooks
        LEFT JOIN LATERAL (
            SELECT *
            FROM webhook_deliveries
            WHERE webhook_deliveries.webhook_id = webhooks.id
            ORDER BY webhook_deliveries.created_at DESC
            LIMIT 1
        ) latest ON true
        WHERE webhooks.repository_id = $1
        ORDER BY webhooks.updated_at DESC
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(hook_summary_from_row).collect()
}

async fn list_organization_hook_summaries(
    pool: &PgPool,
    organization_id: Uuid,
) -> Result<Vec<RepositoryWebhookSummary>, WebhookError> {
    let rows = sqlx::query(
        r#"
        SELECT
            webhooks.id,
            webhooks.url,
            webhooks.content_type,
            webhooks.ssl_verify,
            webhooks.event_selection,
            webhooks.events,
            webhooks.active,
            webhooks.disabled_reason,
            webhooks.secret_configured,
            webhooks.secret_updated_at,
            webhooks.created_at,
            webhooks.updated_at,
            latest.id AS latest_delivery_id,
            latest.delivery_guid AS latest_delivery_guid,
            latest.event AS latest_event,
            latest.status AS latest_status,
            latest.attempt_count AS latest_attempt_count,
            latest.response_status AS latest_response_status,
            latest.duration_ms AS latest_duration_ms,
            latest.redelivery_of_id AS latest_redelivery_of_id,
            latest.delivered_at AS latest_delivered_at,
            latest.created_at AS latest_created_at,
            latest.updated_at AS latest_updated_at
        FROM webhooks
        LEFT JOIN LATERAL (
            SELECT *
            FROM webhook_deliveries
            WHERE webhook_deliveries.webhook_id = webhooks.id
            ORDER BY webhook_deliveries.created_at DESC
            LIMIT 1
        ) latest ON true
        WHERE webhooks.organization_id = $1
        ORDER BY webhooks.updated_at DESC
        "#,
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(hook_summary_from_row).collect()
}

async fn organization_webhook_summary(
    pool: &PgPool,
    organization_id: Uuid,
    hook_id: Uuid,
) -> Result<Option<RepositoryWebhookSummary>, WebhookError> {
    let rows = list_organization_hook_summaries(pool, organization_id).await?;
    Ok(rows.into_iter().find(|hook| hook.id == hook_id))
}

async fn repository_webhook_summary(
    pool: &PgPool,
    repository_id: Uuid,
    hook_id: Uuid,
) -> Result<Option<RepositoryWebhookSummary>, WebhookError> {
    let rows = list_hook_summaries(pool, repository_id).await?;
    Ok(rows.into_iter().find(|hook| hook.id == hook_id))
}

async fn list_delivery_summaries(
    pool: &PgPool,
    hook_id: Uuid,
    limit: i64,
) -> Result<Vec<WebhookDeliverySummary>, WebhookError> {
    let rows = sqlx::query(
        r#"
        SELECT id, delivery_guid, event, status, attempt_count, response_status, duration_ms,
               redelivery_of_id, delivered_at, created_at, updated_at
        FROM webhook_deliveries
        WHERE webhook_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(hook_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(delivery_summary_from_row).collect()
}

async fn delivery_summary_by_id(
    pool: &PgPool,
    hook_id: Uuid,
    delivery_id: Uuid,
) -> Result<Option<WebhookDeliverySummary>, WebhookError> {
    let row = sqlx::query(
        r#"
        SELECT id, delivery_guid, event, status, attempt_count, response_status, duration_ms,
               redelivery_of_id, delivered_at, created_at, updated_at
        FROM webhook_deliveries
        WHERE webhook_id = $1 AND id = $2
        "#,
    )
    .bind(hook_id)
    .bind(delivery_id)
    .fetch_optional(pool)
    .await?;
    row.map(delivery_summary_from_row).transpose()
}

async fn delivery_detail(
    pool: &PgPool,
    hook_id: Uuid,
    delivery_id: Uuid,
) -> Result<Option<WebhookDeliveryDetail>, WebhookError> {
    let row = sqlx::query(
        r#"
        SELECT id, delivery_guid, event, status, attempt_count, response_status, duration_ms,
               redelivery_of_id, delivered_at, created_at, updated_at, request_headers,
               request_body_excerpt, request_body_storage_key, response_headers, response_body,
               response_body_storage_key, terminal_error
        FROM webhook_deliveries
        WHERE webhook_id = $1 AND id = $2
        "#,
    )
    .bind(hook_id)
    .bind(delivery_id)
    .fetch_optional(pool)
    .await?;
    row.map(|row| {
        let summary = delivery_summary_from_row_ref(&row)?;
        Ok(WebhookDeliveryDetail {
            summary,
            request_headers: row.get("request_headers"),
            request_body_excerpt: row.get("request_body_excerpt"),
            request_body_storage_key: row.get("request_body_storage_key"),
            response_headers: row.get("response_headers"),
            response_body_excerpt: row.get("response_body"),
            response_body_storage_key: row.get("response_body_storage_key"),
            terminal_error: row.get("terminal_error"),
        })
    })
    .transpose()
}

fn hook_summary_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<RepositoryWebhookSummary, WebhookError> {
    let latest_delivery = if row
        .try_get::<Option<Uuid>, _>("latest_delivery_id")?
        .is_some()
    {
        Some(WebhookDeliverySummary {
            id: row.get("latest_delivery_id"),
            guid: row.get("latest_delivery_guid"),
            event: row.get("latest_event"),
            status: DeliveryStatus::try_from(row.get::<String, _>("latest_status").as_str())?,
            attempt_count: row.get("latest_attempt_count"),
            response_status: row.get("latest_response_status"),
            duration_ms: row.get("latest_duration_ms"),
            redelivery_of_id: row.get("latest_redelivery_of_id"),
            delivered_at: row.get("latest_delivered_at"),
            created_at: row.get("latest_created_at"),
            updated_at: row.get("latest_updated_at"),
        })
    } else {
        None
    };
    Ok(RepositoryWebhookSummary {
        id: row.get("id"),
        payload_url: row.get("url"),
        content_type: WebhookContentType::try_from(row.get::<String, _>("content_type").as_str())?,
        ssl_verify: row.get("ssl_verify"),
        event_selection: WebhookEventSelection::try_from(
            row.get::<String, _>("event_selection").as_str(),
        )?,
        events: row.get("events"),
        active: row.get("active"),
        disabled_reason: row.get("disabled_reason"),
        secret_configured: row.get("secret_configured"),
        secret_updated_at: row.get("secret_updated_at"),
        latest_delivery,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn delivery_summary_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<WebhookDeliverySummary, WebhookError> {
    delivery_summary_from_row_ref(&row)
}

fn delivery_summary_from_row_ref(
    row: &sqlx::postgres::PgRow,
) -> Result<WebhookDeliverySummary, WebhookError> {
    Ok(WebhookDeliverySummary {
        id: row.get("id"),
        guid: row.get("delivery_guid"),
        event: row.get("event"),
        status: DeliveryStatus::try_from(row.get::<String, _>("status").as_str())?,
        attempt_count: row.get("attempt_count"),
        response_status: row.get("response_status"),
        duration_ms: row.get("duration_ms"),
        redelivery_of_id: row.get("redelivery_of_id"),
        delivered_at: row.get("delivered_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
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
