use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::AppState;

pub const LATEST_API_VERSION: &str = "2022-11-28";

const CORE_LIMIT: i64 = 5_000;
const ANON_LIMIT: i64 = 60;
const SEARCH_LIMIT: i64 = 30;
const HOUR_SECONDS: i64 = 60 * 60;
const MINUTE_SECONDS: i64 = 60;

static MEMORY_BUCKETS: OnceLock<Mutex<HashMap<String, BucketState>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
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

    fn window_seconds(self) -> i64 {
        match self {
            Self::Core => HOUR_SECONDS,
            Self::Search => MINUTE_SECONDS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateLimitIdentity {
    Token { token_id: String },
    Ip { ip: IpAddr },
}

impl RateLimitIdentity {
    fn bucket_key(&self) -> String {
        match self {
            Self::Token { token_id } => format!("token:{token_id}"),
            Self::Ip { ip } => format!("ip:{ip}"),
        }
    }

    fn token_id(&self) -> Option<&str> {
        match self {
            Self::Token { token_id } => Some(token_id),
            Self::Ip { .. } => None,
        }
    }

    fn ip(&self) -> Option<IpAddr> {
        match self {
            Self::Token { .. } => None,
            Self::Ip { ip } => Some(*ip),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self, Self::Token { .. })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitView {
    pub limit: i64,
    pub remaining: i64,
    pub reset: i64,
    pub used: i64,
    pub resource: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct BucketState {
    window_start: i64,
    request_count: i64,
}

pub async fn enforce_rate_limit(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let resource = resource_for_path(req.uri().path());
    let identity = identity_from_headers(req.headers());
    let limit = limit_for(resource, &identity);
    let mut view = match consume_bucket(state.db.as_ref(), &identity, resource, limit).await {
        Ok(view) => view,
        Err(error) => {
            tracing::warn!(%error, "rate-limit storage failed; falling back to local bucket");
            consume_memory_bucket(&identity, resource, limit)
        }
    };
    let version = requested_api_version(req.headers());

    if view.used > view.limit {
        view.remaining = 0;
        let mut response = Response::new(Body::from(
            json!({
                "error": {
                    "code": "rate_limited",
                    "message": format!(
                        "API rate limit exceeded for the {} resource. Try again after {}.",
                        view.resource, view.reset
                    )
                }
            })
            .to_string(),
        ));
        *response.status_mut() = StatusCode::FORBIDDEN;
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        add_rate_limit_headers(response.headers_mut(), &view, &version);
        return response;
    }

    let mut response = next.run(req).await;
    add_rate_limit_headers(response.headers_mut(), &view, &version);
    response
}

pub fn resource_for_path(path: &str) -> RateLimitResource {
    if path.starts_with("/api/search") {
        RateLimitResource::Search
    } else {
        RateLimitResource::Core
    }
}

pub fn identity_from_headers(headers: &HeaderMap) -> RateLimitIdentity {
    if let Some(token) = bearer_token(headers) {
        return RateLimitIdentity::Token {
            token_id: stable_hash(&token),
        };
    }

    if let Some(cookie) = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return RateLimitIdentity::Token {
            token_id: stable_hash(cookie),
        };
    }

    RateLimitIdentity::Ip {
        ip: client_ip(headers),
    }
}

pub async fn current_rate_limits(
    pool: Option<&PgPool>,
    identity: &RateLimitIdentity,
) -> serde_json::Value {
    let core = peek_bucket(
        pool,
        identity,
        RateLimitResource::Core,
        limit_for(RateLimitResource::Core, identity),
    )
    .await
    .unwrap_or_else(|_| {
        peek_memory_bucket(
            identity,
            RateLimitResource::Core,
            limit_for(RateLimitResource::Core, identity),
        )
    });
    let search = peek_bucket(pool, identity, RateLimitResource::Search, SEARCH_LIMIT)
        .await
        .unwrap_or_else(|_| peek_memory_bucket(identity, RateLimitResource::Search, SEARCH_LIMIT));

    json!({
        "resources": {
            "core": core,
            "search": search,
        },
        "rate": core,
    })
}

async fn consume_bucket(
    pool: Option<&PgPool>,
    identity: &RateLimitIdentity,
    resource: RateLimitResource,
    limit: i64,
) -> Result<RateLimitView, sqlx::Error> {
    let Some(pool) = pool else {
        return Ok(consume_memory_bucket(identity, resource, limit));
    };
    let now = now_unix();
    let window_seconds = resource.window_seconds();
    let reset_threshold = now - window_seconds;
    let bucket_key = identity.bucket_key();
    let token_id = identity.token_id();
    let ip = identity.ip().map(|ip| ip.to_string());
    let row: (chrono::DateTime<chrono::Utc>, i64) = sqlx::query_as(
        r#"
        INSERT INTO rate_limit_buckets (bucket_key, token_id, ip, resource, window_start, request_count)
        VALUES ($1, $2, $3::inet, $4, to_timestamp($5), 1)
        ON CONFLICT (bucket_key, resource) DO UPDATE SET
            window_start = CASE
                WHEN rate_limit_buckets.window_start <= to_timestamp($6)
                THEN to_timestamp($5)
                ELSE rate_limit_buckets.window_start
            END,
            request_count = CASE
                WHEN rate_limit_buckets.window_start <= to_timestamp($6)
                THEN 1
                ELSE rate_limit_buckets.request_count + 1
            END
        RETURNING window_start, request_count
        "#,
    )
    .bind(&bucket_key)
    .bind(token_id)
    .bind(ip)
    .bind(resource.as_str())
    .bind(now as f64)
    .bind(reset_threshold as f64)
    .fetch_one(pool)
    .await?;

    Ok(view_from_state(limit, resource, row.0.timestamp(), row.1))
}

async fn peek_bucket(
    pool: Option<&PgPool>,
    identity: &RateLimitIdentity,
    resource: RateLimitResource,
    limit: i64,
) -> Result<RateLimitView, sqlx::Error> {
    let Some(pool) = pool else {
        return Ok(peek_memory_bucket(identity, resource, limit));
    };
    let now = now_unix();
    let bucket_key = identity.bucket_key();
    let row: Option<(chrono::DateTime<chrono::Utc>, i64)> = sqlx::query_as(
        r#"
        SELECT window_start, request_count
        FROM rate_limit_buckets
        WHERE bucket_key = $1 AND resource = $2
        "#,
    )
    .bind(&bucket_key)
    .bind(resource.as_str())
    .fetch_optional(pool)
    .await?;

    Ok(match row {
        Some((window_start, request_count))
            if window_start.timestamp() > now - resource.window_seconds() =>
        {
            view_from_state(limit, resource, window_start.timestamp(), request_count)
        }
        _ => view_from_state(limit, resource, now, 0),
    })
}

fn consume_memory_bucket(
    identity: &RateLimitIdentity,
    resource: RateLimitResource,
    limit: i64,
) -> RateLimitView {
    let now = now_unix();
    let key = format!("{}:{}", identity.bucket_key(), resource.as_str());
    let mut buckets = MEMORY_BUCKETS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .expect("rate-limit memory bucket mutex poisoned");
    let bucket = buckets.entry(key).or_insert(BucketState {
        window_start: now,
        request_count: 0,
    });
    if bucket.window_start <= now - resource.window_seconds() {
        bucket.window_start = now;
        bucket.request_count = 0;
    }
    bucket.request_count += 1;
    view_from_state(limit, resource, bucket.window_start, bucket.request_count)
}

fn peek_memory_bucket(
    identity: &RateLimitIdentity,
    resource: RateLimitResource,
    limit: i64,
) -> RateLimitView {
    let now = now_unix();
    let key = format!("{}:{}", identity.bucket_key(), resource.as_str());
    let buckets = MEMORY_BUCKETS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .expect("rate-limit memory bucket mutex poisoned");
    let state = buckets
        .get(&key)
        .copied()
        .filter(|bucket| bucket.window_start > now - resource.window_seconds());
    match state {
        Some(bucket) => view_from_state(limit, resource, bucket.window_start, bucket.request_count),
        None => view_from_state(limit, resource, now, 0),
    }
}

fn view_from_state(
    limit: i64,
    resource: RateLimitResource,
    window_start: i64,
    request_count: i64,
) -> RateLimitView {
    RateLimitView {
        limit,
        remaining: (limit - request_count).max(0),
        reset: window_start + resource.window_seconds(),
        used: request_count,
        resource: resource.as_str(),
    }
}

fn limit_for(resource: RateLimitResource, identity: &RateLimitIdentity) -> i64 {
    match resource {
        RateLimitResource::Search => SEARCH_LIMIT,
        RateLimitResource::Core if identity.is_authenticated() => CORE_LIMIT,
        RateLimitResource::Core => ANON_LIMIT,
    }
}

fn add_rate_limit_headers(headers: &mut HeaderMap, view: &RateLimitView, version: &str) {
    insert_i64_header(headers, "x-ratelimit-limit", view.limit);
    insert_i64_header(headers, "x-ratelimit-remaining", view.remaining);
    insert_i64_header(headers, "x-ratelimit-reset", view.reset);
    insert_i64_header(headers, "x-ratelimit-used", view.used);
    headers.insert(
        "x-ratelimit-resource",
        HeaderValue::from_static(view.resource),
    );
    if let Ok(value) = HeaderValue::from_str(version) {
        headers.insert("x-github-api-version", value);
    }
}

fn insert_i64_header(headers: &mut HeaderMap, name: &'static str, value: i64) {
    if let Ok(value) = HeaderValue::from_str(&value.to_string()) {
        headers.insert(name, value);
    }
}

fn requested_api_version(headers: &HeaderMap) -> String {
    headers
        .get("x-github-api-version")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(LATEST_API_VERSION)
        .to_owned()
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())?
        .trim();
    value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
}

fn client_ip(headers: &HeaderMap) -> IpAddr {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .and_then(|value| value.parse::<IpAddr>().ok())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.trim().parse::<IpAddr>().ok())
        })
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
}

fn stable_hash(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().min(i64::MAX as u64) as i64)
        .unwrap_or(0)
}
