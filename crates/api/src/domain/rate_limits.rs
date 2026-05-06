use axum::http::HeaderMap;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::{auth::session, config::AppConfig, domain::tokens::hash_personal_access_token};

pub const API_VERSION_HEADER: &str = "x-github-api-version";
pub const LATEST_API_VERSION: &str = "2022-11-28";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitResource {
    Core,
    Search,
}

impl RateLimitResource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Search => "search",
        }
    }

    pub fn limit(self, authenticated: bool) -> i32 {
        match self {
            Self::Core if authenticated => 5_000,
            Self::Core => 60,
            Self::Search => 30,
        }
    }

    pub fn window(self) -> Duration {
        match self {
            Self::Core => Duration::hours(1),
            Self::Search => Duration::minutes(1),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateLimitSubject {
    pub subject_type: String,
    pub subject_key: String,
    pub authenticated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitBucket {
    pub limit: i32,
    pub remaining: i32,
    pub reset: i64,
    pub used: i32,
    pub resource: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RateLimitResources {
    pub core: RateLimitBucket,
    pub search: RateLimitBucket,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RateLimitResponse {
    pub resources: RateLimitResources,
    pub rate: RateLimitBucket,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateLimitDecision {
    pub bucket: RateLimitBucket,
    pub allowed: bool,
}

pub fn resource_for_path(path: &str) -> RateLimitResource {
    if path == "/api/search" || path.starts_with("/api/search/") {
        RateLimitResource::Search
    } else {
        RateLimitResource::Core
    }
}

pub fn api_version(headers: &HeaderMap) -> String {
    headers
        .get(API_VERSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(LATEST_API_VERSION)
        .to_owned()
}

pub fn subject_from_headers(config: &AppConfig, headers: &HeaderMap) -> RateLimitSubject {
    if let Some(token) = bearer_token(headers) {
        return RateLimitSubject {
            subject_type: "token".to_owned(),
            subject_key: hash_personal_access_token(&token),
            authenticated: true,
        };
    }

    if let Ok(Some(session_id)) = session::session_id_from_headers(config, headers) {
        return RateLimitSubject {
            subject_type: "session".to_owned(),
            subject_key: sha256_subject(&session_id),
            authenticated: true,
        };
    }

    RateLimitSubject {
        subject_type: "ip".to_owned(),
        subject_key: client_ip(headers).unwrap_or_else(|| "unknown".to_owned()),
        authenticated: false,
    }
}

pub async fn check_rate_limit(
    pool: &PgPool,
    subject: &RateLimitSubject,
    resource: RateLimitResource,
    now: DateTime<Utc>,
) -> Result<RateLimitDecision, sqlx::Error> {
    let window_start = window_start(now, resource);
    let limit = resource.limit(subject.authenticated);
    let reset_at = window_start + resource.window();

    let request_count: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO rate_limit_buckets (
            subject_type,
            subject_key,
            resource,
            window_start,
            request_count
        )
        VALUES ($1, $2, $3, $4, 1)
        ON CONFLICT (subject_type, subject_key, resource, window_start)
        DO UPDATE SET request_count = rate_limit_buckets.request_count + 1
        RETURNING request_count
        "#,
    )
    .bind(&subject.subject_type)
    .bind(&subject.subject_key)
    .bind(resource.as_str())
    .bind(window_start)
    .fetch_one(pool)
    .await?;

    Ok(RateLimitDecision {
        bucket: RateLimitBucket {
            limit,
            remaining: (limit - request_count).max(0),
            reset: reset_at.timestamp(),
            used: request_count,
            resource: resource.as_str().to_owned(),
        },
        allowed: request_count <= limit,
    })
}

pub async fn rate_limit_status(
    pool: Option<&PgPool>,
    subject: &RateLimitSubject,
    now: DateTime<Utc>,
) -> Result<RateLimitResponse, sqlx::Error> {
    let core = bucket_status(pool, subject, RateLimitResource::Core, now).await?;
    let search = bucket_status(pool, subject, RateLimitResource::Search, now).await?;
    Ok(RateLimitResponse {
        resources: RateLimitResources {
            core: core.clone(),
            search,
        },
        rate: core,
    })
}

pub fn fallback_bucket(
    subject: &RateLimitSubject,
    resource: RateLimitResource,
    now: DateTime<Utc>,
) -> RateLimitBucket {
    let window_start = window_start(now, resource);
    RateLimitBucket {
        limit: resource.limit(subject.authenticated),
        remaining: resource.limit(subject.authenticated),
        reset: (window_start + resource.window()).timestamp(),
        used: 0,
        resource: resource.as_str().to_owned(),
    }
}

async fn bucket_status(
    pool: Option<&PgPool>,
    subject: &RateLimitSubject,
    resource: RateLimitResource,
    now: DateTime<Utc>,
) -> Result<RateLimitBucket, sqlx::Error> {
    let window_start = window_start(now, resource);
    let limit = resource.limit(subject.authenticated);
    let reset_at = window_start + resource.window();
    let used = if let Some(pool) = pool {
        sqlx::query_scalar::<_, i32>(
            r#"
            SELECT request_count
            FROM rate_limit_buckets
            WHERE subject_type = $1
              AND subject_key = $2
              AND resource = $3
              AND window_start = $4
            "#,
        )
        .bind(&subject.subject_type)
        .bind(&subject.subject_key)
        .bind(resource.as_str())
        .bind(window_start)
        .fetch_optional(pool)
        .await?
        .unwrap_or(0)
    } else {
        0
    };

    Ok(RateLimitBucket {
        limit,
        remaining: (limit - used).max(0),
        reset: reset_at.timestamp(),
        used,
        resource: resource.as_str().to_owned(),
    })
}

fn window_start(now: DateTime<Utc>, resource: RateLimitResource) -> DateTime<Utc> {
    let seconds = resource.window().num_seconds();
    let timestamp = now.timestamp() - now.timestamp().rem_euclid(seconds);
    DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or(now)
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .trim();
    value
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
}

fn client_ip(headers: &HeaderMap) -> Option<String> {
    header_str(headers, "x-forwarded-for")
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| header_str(headers, "x-real-ip"))
        .map(ToOwned::to_owned)
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

fn sha256_subject(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    format!("sha256:{hex}")
}
