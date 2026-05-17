use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::jobs::{acquire_job_lease, complete_job_lease, fail_job_lease, JobLeaseError};

pub const EMAIL_QUEUE: &str = "email_delivery";
const EMAIL_LEASE_SECONDS: i64 = 120;
const EMAIL_RETRY_SECONDS: i64 = 300;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailProviderKind {
    Noop,
    Log,
    Ses,
}

impl EmailProviderKind {
    fn from_env_value(value: Option<String>) -> Self {
        match value
            .as_deref()
            .unwrap_or("noop")
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "ses" | "aws-ses" => Self::Ses,
            "log" | "local" => Self::Log,
            _ => Self::Noop,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Noop => "noop",
            Self::Log => "log",
            Self::Ses => "ses",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmailDeliveryConfig {
    pub provider: EmailProviderKind,
    pub from_address: Option<String>,
    pub aws_region: Option<String>,
    pub configuration_set: Option<String>,
}

impl EmailDeliveryConfig {
    pub fn from_env() -> Self {
        Self {
            provider: EmailProviderKind::from_env_value(
                std::env::var("EMAIL_DELIVERY_PROVIDER").ok(),
            ),
            from_address: non_empty_env("EMAIL_FROM_ADDRESS"),
            aws_region: non_empty_env("AWS_REGION").or_else(|| non_empty_env("AWS_DEFAULT_REGION")),
            configuration_set: non_empty_env("SES_CONFIGURATION_SET"),
        }
    }

    pub fn local_noop() -> Self {
        Self {
            provider: EmailProviderKind::Noop,
            from_address: Some("OpenGitHub <noreply@opengithub.local>".to_owned()),
            aws_region: Some("us-east-1".to_owned()),
            configuration_set: None,
        }
    }

    pub fn validate(&self) -> Result<(), EmailDeliveryError> {
        if matches!(self.provider, EmailProviderKind::Ses) {
            if self
                .from_address
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                return Err(EmailDeliveryError::Configuration(
                    "EMAIL_FROM_ADDRESS is required when EMAIL_DELIVERY_PROVIDER=ses".to_owned(),
                ));
            }
            if self
                .aws_region
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                return Err(EmailDeliveryError::Configuration(
                    "AWS_REGION or AWS_DEFAULT_REGION is required when EMAIL_DELIVERY_PROVIDER=ses"
                        .to_owned(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub text_body: String,
    pub html_body: Option<String>,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmailDeliveryRecord {
    pub id: Uuid,
    pub job_lease_id: Uuid,
    pub recipient: String,
    pub subject: String,
    pub provider: String,
    pub status: String,
    pub provider_message_id: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub attempt_count: i32,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum EmailDeliveryError {
    #[error("email delivery configuration error: {0}")]
    Configuration(String),
    #[error("email job payload is invalid: {0}")]
    InvalidPayload(String),
    #[error("email recipient could not be resolved")]
    MissingRecipient,
    #[error("email delivery failed: {0}")]
    Provider(String),
    #[error(transparent)]
    JobLease(#[from] JobLeaseError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub async fn enqueue_test_email(
    pool: &PgPool,
    recipient: &str,
) -> Result<crate::jobs::JobLease, EmailDeliveryError> {
    let recipient = sanitize_email(recipient).ok_or(EmailDeliveryError::MissingRecipient)?;
    Ok(crate::jobs::enqueue_job(
        pool,
        EMAIL_QUEUE,
        &format!("test:{}", Uuid::new_v4()),
        serde_json::json!({
            "kind": "test",
            "to": recipient,
            "subject": "OpenGitHub SES delivery test",
            "body": "This is an OpenGitHub email delivery smoke test."
        }),
    )
    .await?)
}

pub async fn run_next_email_delivery(
    pool: &PgPool,
    worker_id: &str,
    config: &EmailDeliveryConfig,
) -> Result<Option<EmailDeliveryRecord>, EmailDeliveryError> {
    let lease_key = sqlx::query_scalar::<_, String>(
        r#"
        SELECT lease_key
        FROM job_leases
        WHERE queue = $1
          AND completed_at IS NULL
          AND (locked_until IS NULL OR locked_until <= now())
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .bind(EMAIL_QUEUE)
    .fetch_optional(pool)
    .await?;

    let Some(lease_key) = lease_key else {
        return Ok(None);
    };
    run_email_delivery_once(pool, &lease_key, worker_id, config).await
}

pub async fn run_email_delivery_once(
    pool: &PgPool,
    lease_key: &str,
    worker_id: &str,
    config: &EmailDeliveryConfig,
) -> Result<Option<EmailDeliveryRecord>, EmailDeliveryError> {
    config.validate()?;
    let Some(lease) =
        acquire_job_lease(pool, EMAIL_QUEUE, lease_key, worker_id, EMAIL_LEASE_SECONDS).await?
    else {
        return Ok(None);
    };
    let message = match message_from_payload(pool, &lease.payload).await {
        Ok(message) => message,
        Err(error) => {
            let record = upsert_email_delivery_record(
                pool,
                lease.id,
                "unknown",
                "invalid email job payload",
                config.provider.as_str(),
                "failed",
                None,
                Some("invalid_payload"),
                Some(&error.to_string()),
            )
            .await?;
            fail_job_lease(
                pool,
                lease.id,
                worker_id,
                "invalid_payload",
                EMAIL_RETRY_SECONDS,
            )
            .await?;
            tracing::warn!(job_lease_id = %lease.id, error = %error, "email delivery job payload rejected");
            return Ok(Some(record));
        }
    };

    let result = send_email(config, &message).await;
    match result {
        Ok(provider_message_id) => {
            let record = upsert_email_delivery_record(
                pool,
                lease.id,
                &message.to,
                &message.subject,
                config.provider.as_str(),
                "sent",
                provider_message_id.as_deref(),
                None,
                None,
            )
            .await?;
            complete_job_lease(pool, lease.id, worker_id).await?;
            tracing::info!(job_lease_id = %lease.id, recipient = %message.to, provider = config.provider.as_str(), "email delivery sent");
            Ok(Some(record))
        }
        Err(error) => {
            let safe_error = redact_secrets(&error.to_string());
            let record = upsert_email_delivery_record(
                pool,
                lease.id,
                &message.to,
                &message.subject,
                config.provider.as_str(),
                "failed",
                None,
                Some("provider_error"),
                Some(&safe_error),
            )
            .await?;
            fail_job_lease(
                pool,
                lease.id,
                worker_id,
                "provider_error",
                EMAIL_RETRY_SECONDS,
            )
            .await?;
            tracing::warn!(job_lease_id = %lease.id, recipient = %message.to, provider = config.provider.as_str(), error = %safe_error, "email delivery failed; job will retry");
            Ok(Some(record))
        }
    }
}

async fn message_from_payload(
    pool: &PgPool,
    payload: &Value,
) -> Result<EmailMessage, EmailDeliveryError> {
    let subject = payload
        .get("subject")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| EmailDeliveryError::InvalidPayload("subject is required".to_owned()))?
        .to_owned();
    let text_body = payload
        .get("body")
        .or_else(|| payload.get("textBody"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| EmailDeliveryError::InvalidPayload("body is required".to_owned()))?
        .to_owned();
    let html_body = payload
        .get("htmlBody")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned);
    let kind = payload
        .get("kind")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let to = if let Some(to) = payload
        .get("to")
        .and_then(Value::as_str)
        .and_then(sanitize_email)
    {
        to
    } else if let Some(user_id) = payload
        .get("userId")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
    {
        user_email(pool, user_id)
            .await?
            .ok_or(EmailDeliveryError::MissingRecipient)?
    } else {
        return Err(EmailDeliveryError::MissingRecipient);
    };
    Ok(EmailMessage {
        to,
        subject,
        text_body,
        html_body,
        kind,
    })
}

async fn user_email(pool: &PgPool, user_id: Uuid) -> Result<Option<String>, sqlx::Error> {
    let email = sqlx::query_scalar::<_, String>("SELECT email FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(email.and_then(|value| sanitize_email(&value)))
}

async fn send_email(
    config: &EmailDeliveryConfig,
    message: &EmailMessage,
) -> Result<Option<String>, EmailDeliveryError> {
    match config.provider {
        EmailProviderKind::Noop => Ok(Some("noop".to_owned())),
        EmailProviderKind::Log => {
            tracing::info!(recipient = %message.to, subject = %message.subject, "local email delivery log provider accepted message");
            Ok(Some("log".to_owned()))
        }
        EmailProviderKind::Ses => send_email_with_ses(config, message).await,
    }
}

#[derive(Debug, Clone)]
struct AwsCredentials {
    access_key_id: String,
    secret_access_key: String,
    session_token: Option<String>,
}

async fn send_email_with_ses(
    config: &EmailDeliveryConfig,
    message: &EmailMessage,
) -> Result<Option<String>, EmailDeliveryError> {
    let region = config.aws_region.clone().ok_or_else(|| {
        EmailDeliveryError::Configuration(
            "AWS_REGION or AWS_DEFAULT_REGION is required when EMAIL_DELIVERY_PROVIDER=ses"
                .to_owned(),
        )
    })?;
    let from = config.from_address.clone().ok_or_else(|| {
        EmailDeliveryError::Configuration(
            "EMAIL_FROM_ADDRESS is required when EMAIL_DELIVERY_PROVIDER=ses".to_owned(),
        )
    })?;
    let credentials = resolve_aws_credentials().await?;
    let endpoint = format!("https://email.{region}.amazonaws.com/v2/email/outbound-emails");
    let host = format!("email.{region}.amazonaws.com");
    let now = Utc::now();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();
    let body = serde_json::json!({
        "FromEmailAddress": from,
        "Destination": { "ToAddresses": [message.to] },
        "Content": {
            "Simple": {
                "Subject": { "Data": message.subject, "Charset": "UTF-8" },
                "Body": {
                    "Text": { "Data": message.text_body, "Charset": "UTF-8" },
                    "Html": message.html_body.as_ref().map(|html| serde_json::json!({ "Data": html, "Charset": "UTF-8" }))
                }
            }
        },
        "ConfigurationSetName": config.configuration_set
    });
    let body = serde_json::to_string(&body)
        .map_err(|error| EmailDeliveryError::Provider(error.to_string()))?;
    let payload_hash = sha256_hex(body.as_bytes());
    let signed_headers = if credentials.session_token.is_some() {
        "content-type;host;x-amz-date;x-amz-security-token"
    } else {
        "content-type;host;x-amz-date"
    };
    let canonical_headers = if let Some(token) = &credentials.session_token {
        format!(
            "content-type:application/json\nhost:{host}\nx-amz-date:{amz_date}\nx-amz-security-token:{token}\n"
        )
    } else {
        format!("content-type:application/json\nhost:{host}\nx-amz-date:{amz_date}\n")
    };
    let canonical_request = format!(
        "POST\n/v2/email/outbound-emails\n\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
    );
    let credential_scope = format!("{date_stamp}/{region}/ses/aws4_request");
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
        sha256_hex(canonical_request.as_bytes())
    );
    let signing_key =
        aws_v4_signing_key(&credentials.secret_access_key, &date_stamp, &region, "ses");
    let signature = hmac_sha256_hex(&signing_key, string_to_sign.as_bytes());
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}",
        credentials.access_key_id
    );

    let client = reqwest::Client::new();
    let mut request = client
        .post(endpoint)
        .header("content-type", "application/json")
        .header("host", host)
        .header("x-amz-date", amz_date)
        .header("authorization", authorization)
        .body(body);
    if let Some(token) = credentials.session_token {
        request = request.header("x-amz-security-token", token);
    }
    let response = request
        .send()
        .await
        .map_err(|error| EmailDeliveryError::Provider(redact_secrets(&error.to_string())))?;
    let status = response.status();
    let response_body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(EmailDeliveryError::Provider(redact_secrets(&format!(
            "SES SendEmail returned HTTP {status}: {}",
            truncate_for_log(&response_body, 500)
        ))));
    }
    let message_id = serde_json::from_str::<serde_json::Value>(&response_body)
        .ok()
        .and_then(|value| {
            value
                .get("MessageId")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        });
    Ok(message_id)
}

async fn resolve_aws_credentials() -> Result<AwsCredentials, EmailDeliveryError> {
    if let (Some(access_key_id), Some(secret_access_key)) = (
        non_empty_env("AWS_ACCESS_KEY_ID"),
        non_empty_env("AWS_SECRET_ACCESS_KEY"),
    ) {
        return Ok(AwsCredentials {
            access_key_id,
            secret_access_key,
            session_token: non_empty_env("AWS_SESSION_TOKEN"),
        });
    }

    let relative_uri = non_empty_env("AWS_CONTAINER_CREDENTIALS_RELATIVE_URI");
    let full_uri = non_empty_env("AWS_CONTAINER_CREDENTIALS_FULL_URI");
    let Some(uri) = relative_uri
        .map(|path| format!("http://169.254.170.2{path}"))
        .or(full_uri)
    else {
        return Err(EmailDeliveryError::Configuration(
            "AWS credentials were not found in env or ECS task metadata".to_owned(),
        ));
    };

    let mut request = reqwest::Client::new().get(uri);
    if let Some(token) = non_empty_env("AWS_CONTAINER_AUTHORIZATION_TOKEN") {
        request = request.header("authorization", token);
    }
    let value = request
        .send()
        .await
        .map_err(|error| EmailDeliveryError::Provider(redact_secrets(&error.to_string())))?
        .json::<serde_json::Value>()
        .await
        .map_err(|error| EmailDeliveryError::Provider(redact_secrets(&error.to_string())))?;
    let access_key_id = value
        .get("AccessKeyId")
        .and_then(serde_json::Value::as_str)
        .or_else(|| value.get("AccessKeyID").and_then(serde_json::Value::as_str))
        .ok_or_else(|| {
            EmailDeliveryError::Provider("ECS metadata did not return AccessKeyId".to_owned())
        })?
        .to_owned();
    let secret_access_key = value
        .get("SecretAccessKey")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            EmailDeliveryError::Provider("ECS metadata did not return SecretAccessKey".to_owned())
        })?
        .to_owned();
    let session_token = value
        .get("Token")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    Ok(AwsCredentials {
        access_key_id,
        secret_access_key,
        session_token,
    })
}

fn aws_v4_signing_key(secret: &str, date_stamp: &str, region: &str, service: &str) -> Vec<u8> {
    let k_date = hmac_sha256(format!("AWS4{secret}").as_bytes(), date_stamp.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, service.as_bytes());
    hmac_sha256(&k_service, b"aws4_request")
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(key).expect("HMAC accepts keys of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn hmac_sha256_hex(key: &[u8], data: &[u8]) -> String {
    hex_lower(&hmac_sha256(key, data))
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::Digest;
    hex_lower(&sha2::Sha256::digest(data))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn truncate_for_log(value: &str, limit: usize) -> String {
    if value.len() <= limit {
        value.to_owned()
    } else {
        format!("{}…", &value[..limit])
    }
}

#[allow(clippy::too_many_arguments)]
async fn upsert_email_delivery_record(
    pool: &PgPool,
    job_lease_id: Uuid,
    recipient: &str,
    subject: &str,
    provider: &str,
    status: &str,
    provider_message_id: Option<&str>,
    error_code: Option<&str>,
    error_message: Option<&str>,
) -> Result<EmailDeliveryRecord, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO email_deliveries (
            job_lease_id, recipient, subject, provider, status, provider_message_id,
            error_code, error_message, attempt_count, sent_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1, CASE WHEN $5 = 'sent' THEN now() ELSE NULL END)
        ON CONFLICT (job_lease_id) DO UPDATE SET
            recipient = EXCLUDED.recipient,
            subject = EXCLUDED.subject,
            provider = EXCLUDED.provider,
            status = EXCLUDED.status,
            provider_message_id = EXCLUDED.provider_message_id,
            error_code = EXCLUDED.error_code,
            error_message = EXCLUDED.error_message,
            attempt_count = email_deliveries.attempt_count + 1,
            sent_at = CASE WHEN EXCLUDED.status = 'sent' THEN now() ELSE email_deliveries.sent_at END,
            updated_at = now()
        RETURNING id, job_lease_id, recipient, subject, provider, status, provider_message_id,
                  error_code, error_message, attempt_count, sent_at, created_at, updated_at
        "#,
    )
    .bind(job_lease_id)
    .bind(recipient)
    .bind(subject)
    .bind(provider)
    .bind(status)
    .bind(provider_message_id)
    .bind(error_code)
    .bind(error_message)
    .fetch_one(pool)
    .await?;
    Ok(email_delivery_from_row(row))
}

fn email_delivery_from_row(row: sqlx::postgres::PgRow) -> EmailDeliveryRecord {
    EmailDeliveryRecord {
        id: row.get("id"),
        job_lease_id: row.get("job_lease_id"),
        recipient: row.get("recipient"),
        subject: row.get("subject"),
        provider: row.get("provider"),
        status: row.get("status"),
        provider_message_id: row.get("provider_message_id"),
        error_code: row.get("error_code"),
        error_message: row.get("error_message"),
        attempt_count: row.get("attempt_count"),
        sent_at: row.get("sent_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn sanitize_email(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.len() > 320 || trimmed.contains('\n') || trimmed.contains('\r') {
        return None;
    }
    let (local, domain) = trimmed.split_once('@')?;
    if local.is_empty() || domain.is_empty() || !domain.contains('.') {
        return None;
    }
    Some(trimmed.to_ascii_lowercase())
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn redact_secrets(value: &str) -> String {
    let mut redacted = value.to_owned();
    for name in [
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "AWS_SESSION_TOKEN",
    ] {
        if let Ok(secret) = std::env::var(name) {
            if !secret.trim().is_empty() {
                redacted = redacted.replace(&secret, "[redacted]");
            }
        }
    }
    redacted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ses_config_requires_sender_and_region() {
        let config = EmailDeliveryConfig {
            provider: EmailProviderKind::Ses,
            from_address: None,
            aws_region: None,
            configuration_set: None,
        };
        let error = config.validate().unwrap_err().to_string();
        assert!(error.contains("EMAIL_FROM_ADDRESS"));
    }

    #[test]
    fn local_provider_does_not_require_aws_config() {
        EmailDeliveryConfig {
            provider: EmailProviderKind::Noop,
            from_address: None,
            aws_region: None,
            configuration_set: None,
        }
        .validate()
        .unwrap();
    }

    #[test]
    fn invalid_recipient_is_rejected() {
        assert!(sanitize_email("bad\n@example.com").is_none());
        assert!(sanitize_email("not-an-email").is_none());
        assert_eq!(
            sanitize_email("USER@Example.COM").unwrap(),
            "user@example.com"
        );
    }
}
