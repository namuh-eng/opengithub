use std::time::{Duration, Instant};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use serde_json::{json, Value};
use sha2::Sha256;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    domain::webhooks::{
        mark_webhook_delivery_request, record_webhook_delivery_worker_result, DeliveryStatus,
        WebhookDelivery, WebhookDeliveryWorkerResult, WebhookError,
    },
    jobs::{acquire_job_lease, complete_job_lease, fail_job_lease, JobLeaseError},
};

type HmacSha256 = Hmac<Sha256>;

const WEBHOOK_QUEUE: &str = "webhook-delivery";
const WEBHOOK_LEASE_SECONDS: i64 = 60;
const DEFAULT_TIMEOUT_SECONDS: u64 = 10;
const DEFAULT_MAX_ATTEMPTS: i32 = 3;
const DEFAULT_MAX_BODY_EXCERPT_BYTES: usize = 4096;
const DEFAULT_MAX_PAYLOAD_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone)]
pub struct WebhookDeliveryWorkerConfig {
    pub timeout: Duration,
    pub max_attempts: i32,
    pub max_body_excerpt_bytes: usize,
    pub max_payload_bytes: usize,
}

impl Default for WebhookDeliveryWorkerConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECONDS),
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            max_body_excerpt_bytes: DEFAULT_MAX_BODY_EXCERPT_BYTES,
            max_payload_bytes: DEFAULT_MAX_PAYLOAD_BYTES,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebhookDeliveryWorkerError {
    #[error("webhook delivery was not found")]
    NotFound,
    #[error("webhook delivery failed: {0}")]
    Delivery(String),
    #[error(transparent)]
    Webhook(#[from] WebhookError),
    #[error(transparent)]
    JobLease(#[from] JobLeaseError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug)]
struct WebhookDeliveryWorkItem {
    delivery_guid: Uuid,
    webhook_id: Uuid,
    event: String,
    payload: Value,
    status: DeliveryStatus,
    attempt_count: i32,
    url: String,
    secret_hash: Option<String>,
    active: bool,
    events: Vec<String>,
    content_type: String,
}

pub async fn run_next_webhook_delivery(
    pool: &PgPool,
    worker_id: &str,
) -> Result<Option<WebhookDelivery>, WebhookDeliveryWorkerError> {
    let delivery_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT webhook_deliveries.id
        FROM webhook_deliveries
        JOIN webhooks ON webhooks.id = webhook_deliveries.webhook_id
        JOIN job_leases ON job_leases.queue = $1
          AND job_leases.lease_key = webhook_deliveries.id::text
        WHERE webhook_deliveries.status = 'queued'
          AND (webhook_deliveries.next_attempt_at IS NULL OR webhook_deliveries.next_attempt_at <= now())
          AND job_leases.completed_at IS NULL
          AND (job_leases.locked_until IS NULL OR job_leases.locked_until <= now())
        ORDER BY webhook_deliveries.created_at ASC
        LIMIT 1
        "#,
    )
    .bind(WEBHOOK_QUEUE)
    .fetch_optional(pool)
    .await?;

    let Some(delivery_id) = delivery_id else {
        return Ok(None);
    };
    run_webhook_delivery_once(pool, delivery_id, worker_id).await
}

pub async fn run_webhook_delivery_once(
    pool: &PgPool,
    delivery_id: Uuid,
    worker_id: &str,
) -> Result<Option<WebhookDelivery>, WebhookDeliveryWorkerError> {
    run_webhook_delivery_once_with_config(
        pool,
        delivery_id,
        worker_id,
        &WebhookDeliveryWorkerConfig::default(),
    )
    .await
}

pub async fn run_webhook_delivery_once_with_config(
    pool: &PgPool,
    delivery_id: Uuid,
    worker_id: &str,
    config: &WebhookDeliveryWorkerConfig,
) -> Result<Option<WebhookDelivery>, WebhookDeliveryWorkerError> {
    let Some(lease) = acquire_job_lease(
        pool,
        WEBHOOK_QUEUE,
        &delivery_id.to_string(),
        worker_id,
        WEBHOOK_LEASE_SECONDS,
    )
    .await?
    else {
        return Ok(None);
    };

    let work_item = load_delivery_work_item(pool, delivery_id)
        .await?
        .ok_or(WebhookDeliveryWorkerError::NotFound)?;

    if work_item.status != DeliveryStatus::Queued {
        complete_job_lease(pool, lease.id, worker_id).await?;
        return Ok(Some(delivery_from_work_item(pool, delivery_id).await?));
    }

    if !work_item.active || !subscribed_to_event(&work_item.events, &work_item.event) {
        let delivery = record_webhook_delivery_worker_result(
            pool,
            delivery_id,
            WebhookDeliveryWorkerResult {
                status: DeliveryStatus::Failed,
                response_status: None,
                response_headers: json!({}),
                response_body_excerpt: None,
                duration_ms: None,
                terminal_error: Some("webhook_inactive_or_unsubscribed".to_owned()),
                retry_after_seconds: None,
            },
        )
        .await?;
        complete_job_lease(pool, lease.id, worker_id).await?;
        return Ok(Some(delivery));
    }

    let body = delivery_body(&work_item);
    if body.len() > config.max_payload_bytes {
        let delivery = record_webhook_delivery_worker_result(
            pool,
            delivery_id,
            WebhookDeliveryWorkerResult {
                status: DeliveryStatus::Failed,
                response_status: None,
                response_headers: json!({}),
                response_body_excerpt: None,
                duration_ms: None,
                terminal_error: Some("payload_too_large".to_owned()),
                retry_after_seconds: None,
            },
        )
        .await?;
        complete_job_lease(pool, lease.id, worker_id).await?;
        return Ok(Some(delivery));
    }

    let request_headers = request_headers(&work_item, &body)?;
    mark_webhook_delivery_request(
        pool,
        delivery_id,
        headers_to_json(&request_headers),
        excerpt(&body, config.max_body_excerpt_bytes),
        storage_key_if_truncated(
            delivery_id,
            "request",
            body.len(),
            config.max_body_excerpt_bytes,
        ),
    )
    .await?;

    let client = reqwest::Client::builder()
        .timeout(config.timeout)
        .build()
        .map_err(|error| {
            WebhookDeliveryWorkerError::Delivery(format!("client_build_failed:{error}"))
        })?;
    let started = Instant::now();
    let result = client
        .post(&work_item.url)
        .headers(request_headers)
        .body(body)
        .send()
        .await;
    let duration_ms = started.elapsed().as_millis().min(i64::MAX as u128) as i64;

    let delivery = match result {
        Ok(response) => {
            let status_code = response.status().as_u16() as i32;
            let response_headers = headers_to_json(response.headers());
            let response_body = response.text().await.unwrap_or_default();
            let delivered = (200..300).contains(&status_code);
            let terminal = !delivered && work_item.attempt_count + 1 >= config.max_attempts;
            let next_status = if delivered {
                DeliveryStatus::Delivered
            } else if terminal {
                DeliveryStatus::Failed
            } else {
                DeliveryStatus::Queued
            };
            let retry_after = (!delivered && !terminal)
                .then_some(retry_delay_seconds(work_item.attempt_count + 1));
            let terminal_error = if delivered {
                None
            } else if terminal {
                Some(format!("http_status_{status_code}"))
            } else {
                Some(format!("retryable_http_status_{status_code}"))
            };
            record_webhook_delivery_worker_result(
                pool,
                delivery_id,
                WebhookDeliveryWorkerResult {
                    status: next_status,
                    response_status: Some(status_code),
                    response_headers,
                    response_body_excerpt: Some(excerpt(
                        response_body.as_bytes(),
                        config.max_body_excerpt_bytes,
                    )),
                    duration_ms: Some(duration_ms),
                    terminal_error,
                    retry_after_seconds: retry_after,
                },
            )
            .await?
        }
        Err(error) => {
            let terminal = work_item.attempt_count + 1 >= config.max_attempts;
            let next_status = if terminal {
                DeliveryStatus::Failed
            } else {
                DeliveryStatus::Queued
            };
            let error_category = if error.is_timeout() {
                "timeout"
            } else if error.is_connect() {
                "connect_error"
            } else {
                "request_error"
            };
            record_webhook_delivery_worker_result(
                pool,
                delivery_id,
                WebhookDeliveryWorkerResult {
                    status: next_status,
                    response_status: None,
                    response_headers: json!({}),
                    response_body_excerpt: None,
                    duration_ms: Some(duration_ms),
                    terminal_error: Some(error_category.to_owned()),
                    retry_after_seconds: (!terminal)
                        .then_some(retry_delay_seconds(work_item.attempt_count + 1)),
                },
            )
            .await?
        }
    };

    if delivery.status == DeliveryStatus::Queued {
        fail_job_lease(
            pool,
            lease.id,
            worker_id,
            delivery
                .response_status
                .map(|status| format!("http_status_{status}"))
                .as_deref()
                .unwrap_or("webhook_delivery_retryable"),
            retry_delay_seconds(delivery.attempt_count),
        )
        .await?;
    } else {
        complete_job_lease(pool, lease.id, worker_id).await?;
    }

    tracing::info!(
        delivery_id = %delivery.id,
        webhook_id = %delivery.webhook_id,
        event = %delivery.event,
        attempts = delivery.attempt_count,
        status = ?delivery.status,
        duration_ms = ?duration_ms,
        "webhook delivery worker completed attempt"
    );
    Ok(Some(delivery))
}

fn delivery_body(work_item: &WebhookDeliveryWorkItem) -> Vec<u8> {
    let rendered = if work_item.content_type == "application/x-www-form-urlencoded" {
        format!("payload={}", work_item.payload)
    } else {
        work_item.payload.to_string()
    };
    rendered.into_bytes()
}

fn request_headers(
    work_item: &WebhookDeliveryWorkItem,
    body: &[u8],
) -> Result<HeaderMap, WebhookDeliveryWorkerError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("opengithub-webhooks/1.0"),
    );
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(&work_item.content_type)
            .map_err(|error| WebhookDeliveryWorkerError::Delivery(error.to_string()))?,
    );
    headers.insert(
        "x-github-event",
        HeaderValue::from_str(&work_item.event)
            .map_err(|error| WebhookDeliveryWorkerError::Delivery(error.to_string()))?,
    );
    headers.insert(
        "x-github-delivery",
        HeaderValue::from_str(&work_item.delivery_guid.to_string())
            .map_err(|error| WebhookDeliveryWorkerError::Delivery(error.to_string()))?,
    );
    headers.insert(
        "x-opengithub-hook-id",
        HeaderValue::from_str(&work_item.webhook_id.to_string())
            .map_err(|error| WebhookDeliveryWorkerError::Delivery(error.to_string()))?,
    );
    if let Some(secret_hash) = work_item.secret_hash.as_deref() {
        headers.insert(
            "x-hub-signature-256",
            HeaderValue::from_str(&signature_header(secret_hash, body))
                .map_err(|error| WebhookDeliveryWorkerError::Delivery(error.to_string()))?,
        );
    }
    Ok(headers)
}

pub fn signature_header(secret_material: &str, body: &[u8]) -> String {
    let signing_secret = signing_secret_material(secret_material);
    let mut mac = HmacSha256::new_from_slice(signing_secret.as_bytes()).expect("HMAC key");
    mac.update(body);
    format!("sha256={}", hex(mac.finalize().into_bytes().as_slice()))
}

fn signing_secret_material(stored_secret: &str) -> String {
    stored_secret
        .strip_prefix("secret:v1:")
        .and_then(|encoded| STANDARD.decode(encoded).ok())
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_else(|| stored_secret.to_owned())
}

fn subscribed_to_event(events: &[String], event: &str) -> bool {
    events
        .iter()
        .any(|candidate| candidate == "*" || candidate == event)
}

fn retry_delay_seconds(attempt_count: i32) -> i64 {
    match attempt_count {
        count if count <= 1 => 30,
        2 => 120,
        _ => 300,
    }
}

fn excerpt(body: &[u8], limit: usize) -> String {
    String::from_utf8_lossy(&body[..body.len().min(limit)]).into_owned()
}

fn storage_key_if_truncated(
    delivery_id: Uuid,
    kind: &str,
    len: usize,
    limit: usize,
) -> Option<String> {
    (len > limit).then(|| format!("webhook-deliveries/{delivery_id}/{kind}.body"))
}

fn headers_to_json(headers: &HeaderMap) -> Value {
    let mut object = serde_json::Map::new();
    for (name, value) in headers {
        if let Ok(value) = value.to_str() {
            object.insert(name.as_str().to_owned(), Value::String(value.to_owned()));
        }
    }
    Value::Object(object)
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::signature_header;

    #[test]
    fn signature_header_uses_decoded_configured_secret_for_new_webhooks() {
        let body = br#"{"ref":"refs/heads/main"}"#;

        assert_eq!(
            signature_header("secret:v1:c3VwZXItc2VjcmV0LXZhbHVl", body),
            signature_header("super-secret-value", body)
        );
    }
}

async fn load_delivery_work_item(
    pool: &PgPool,
    delivery_id: Uuid,
) -> Result<Option<WebhookDeliveryWorkItem>, WebhookDeliveryWorkerError> {
    let row = sqlx::query(
        r#"
        SELECT webhook_deliveries.id,
               webhook_deliveries.delivery_guid,
               webhook_deliveries.webhook_id,
               webhook_deliveries.event,
               webhook_deliveries.payload,
               webhook_deliveries.status,
               webhook_deliveries.attempt_count,
               webhooks.url,
               webhooks.secret_hash,
               webhooks.active,
               webhooks.events,
               webhooks.content_type
        FROM webhook_deliveries
        JOIN webhooks ON webhooks.id = webhook_deliveries.webhook_id
        WHERE webhook_deliveries.id = $1
        "#,
    )
    .bind(delivery_id)
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        Ok(WebhookDeliveryWorkItem {
            delivery_guid: row.get("delivery_guid"),
            webhook_id: row.get("webhook_id"),
            event: row.get("event"),
            payload: row.get("payload"),
            status: DeliveryStatus::try_from(row.get::<String, _>("status").as_str())?,
            attempt_count: row.get("attempt_count"),
            url: row.get("url"),
            secret_hash: row.get("secret_hash"),
            active: row.get("active"),
            events: row.get("events"),
            content_type: row.get("content_type"),
        })
    })
    .transpose()
    .map_err(WebhookDeliveryWorkerError::Webhook)
}

async fn delivery_from_work_item(
    pool: &PgPool,
    delivery_id: Uuid,
) -> Result<WebhookDelivery, WebhookDeliveryWorkerError> {
    let row = sqlx::query(
        r#"
        SELECT id, webhook_id, event, payload, status, attempt_count, next_attempt_at,
               response_status, response_body, delivered_at, created_at, updated_at
        FROM webhook_deliveries
        WHERE id = $1
        "#,
    )
    .bind(delivery_id)
    .fetch_optional(pool)
    .await?
    .ok_or(WebhookDeliveryWorkerError::NotFound)?;

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
    .map_err(WebhookDeliveryWorkerError::Webhook)
}
