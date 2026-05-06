use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::tokens::{create_sudo_grant, sudo_state, CreateSudoGrantRequest, SudoState};

#[derive(Debug, thiserror::Error)]
pub enum AccountSecurityError {
    #[error("sudo confirmation is invalid")]
    InvalidSudoConfirmation,
    #[error("sudo mode is required")]
    SudoRequired,
    #[error("the last sign-in method cannot be removed")]
    LastIdentity,
    #[error("identity is not available")]
    Forbidden,
    #[error("session is not available")]
    SessionNotFound,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountSecuritySettings {
    #[serde(rename = "signInMethods")]
    pub sign_in_methods: Vec<SignInMethodSummary>,
    #[serde(rename = "sudo")]
    pub sudo: SudoState,
    #[serde(rename = "twoFactor")]
    pub two_factor: TwoFactorSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignInMethodSummary {
    pub id: Uuid,
    pub provider: String,
    pub email: String,
    #[serde(rename = "displayLabel")]
    pub display_label: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    #[serde(rename = "linkedAt")]
    pub linked_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "canUnlink")]
    pub can_unlink: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TwoFactorSummary {
    pub enabled: bool,
    pub available: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnlinkSignInMethodResponse {
    #[serde(rename = "removedId")]
    pub removed_id: Uuid,
    pub settings: AccountSecuritySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountSessions {
    pub sessions: Vec<AccountSessionSummary>,
    #[serde(rename = "activeCount")]
    pub active_count: i64,
    #[serde(rename = "currentSessionId")]
    pub current_session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountSessionSummary {
    pub id: String,
    pub device: String,
    pub browser: String,
    pub location: String,
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    #[serde(rename = "userAgent")]
    pub user_agent: Option<String>,
    #[serde(rename = "signedInAt")]
    pub signed_in_at: DateTime<Utc>,
    #[serde(rename = "lastActiveAt")]
    pub last_active_at: DateTime<Utc>,
    #[serde(rename = "expiresAt")]
    pub expires_at: DateTime<Utc>,
    #[serde(rename = "isCurrent")]
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RevokeAccountSessionResponse {
    #[serde(rename = "revokedId")]
    pub revoked_id: String,
    pub sessions: AccountSessions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignOutEverywhereResponse {
    #[serde(rename = "revokedCount")]
    pub revoked_count: i64,
    pub sessions: AccountSessions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountSecurityLog {
    pub events: Vec<AccountSecurityLogEvent>,
    pub actions: Vec<String>,
    pub filters: AccountSecurityLogFilters,
    pub pagination: AccountSecurityLogPagination,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountSecurityLogEvent {
    pub id: Uuid,
    pub action: String,
    pub location: String,
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    #[serde(rename = "userAgent")]
    pub user_agent: Option<String>,
    pub metadata: Value,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountSecurityLogFilters {
    pub action: Option<String>,
    pub page: i64,
    #[serde(rename = "pageSize")]
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountSecurityLogPagination {
    pub total: i64,
    pub page: i64,
    #[serde(rename = "pageSize")]
    pub page_size: i64,
    #[serde(rename = "totalPages")]
    pub total_pages: i64,
    #[serde(rename = "hasPrevious")]
    pub has_previous: bool,
    #[serde(rename = "hasNext")]
    pub has_next: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountSecurityLogQuery {
    pub action: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

impl AccountSecurityLogQuery {
    pub fn normalized(action: Option<String>, page: Option<i64>, page_size: Option<i64>) -> Self {
        let action = action
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty() && value != "all");
        Self {
            action,
            page: page.unwrap_or(1).clamp(1, 10_000),
            page_size: page_size.unwrap_or(50).clamp(1, 50),
        }
    }
}

pub async fn account_security_settings(
    pool: &PgPool,
    user_id: Uuid,
    session_id: Option<&str>,
) -> Result<AccountSecuritySettings, AccountSecurityError> {
    let rows = sqlx::query(
        r#"
        SELECT oauth_accounts.id, oauth_accounts.provider, oauth_accounts.email,
               oauth_accounts.created_at, oauth_accounts.updated_at,
               users.avatar_url
        FROM oauth_accounts
        JOIN users ON users.id = oauth_accounts.user_id
        WHERE oauth_accounts.user_id = $1
        ORDER BY oauth_accounts.created_at ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    let count = rows.len();
    let sign_in_methods = rows
        .into_iter()
        .map(|row| SignInMethodSummary {
            id: row.get("id"),
            provider: row.get("provider"),
            email: row.get("email"),
            display_label: provider_label(row.get("provider")),
            avatar_url: row.get("avatar_url"),
            linked_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            can_unlink: count > 1,
        })
        .collect();

    let mut sudo = sudo_state(pool, user_id, session_id)
        .await
        .map_err(map_sudo_error)?;
    sudo.required_for = vec![
        "link_google_account".to_owned(),
        "unlink_sign_in_method".to_owned(),
        "change_primary_email".to_owned(),
    ];

    Ok(AccountSecuritySettings {
        sign_in_methods,
        sudo,
        two_factor: TwoFactorSummary {
            enabled: false,
            available: false,
            reason: "Two-factor authentication is planned after Google-only auth hardening."
                .to_owned(),
        },
    })
}

pub async fn update_current_session_metadata(
    pool: &PgPool,
    user_id: Uuid,
    session_id: &str,
    user_agent: Option<&str>,
    ip_address: Option<&str>,
) -> Result<(), AccountSecurityError> {
    sqlx::query(
        r#"
        UPDATE sessions
        SET user_agent = COALESCE($3, user_agent),
            ip_inet = COALESCE(NULLIF($4, '')::inet, ip_inet),
            last_active_at = now(),
            last_seen_at = now()
        WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL AND expires_at > now()
        "#,
    )
    .bind(session_id)
    .bind(user_id)
    .bind(user_agent.filter(|value| !value.trim().is_empty()))
    .bind(ip_address.filter(|value| !value.trim().is_empty()))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn account_sessions(
    pool: &PgPool,
    user_id: Uuid,
    current_session_id: &str,
) -> Result<AccountSessions, AccountSecurityError> {
    let rows = sqlx::query(
        r#"
        SELECT id, user_agent, ip_inet::text AS ip_address, created_at,
               COALESCE(last_active_at, last_seen_at) AS last_active_at, expires_at
        FROM sessions
        WHERE user_id = $1
          AND revoked_at IS NULL
          AND expires_at > now()
        ORDER BY (id = $2) DESC, COALESCE(last_active_at, last_seen_at) DESC, created_at DESC
        "#,
    )
    .bind(user_id)
    .bind(current_session_id)
    .fetch_all(pool)
    .await?;

    let sessions = rows
        .into_iter()
        .map(|row| {
            let user_agent: Option<String> = row.get("user_agent");
            let ip_address: Option<String> = row.get("ip_address");
            AccountSessionSummary {
                id: row.get("id"),
                device: device_label(user_agent.as_deref()),
                browser: browser_label(user_agent.as_deref()),
                location: location_label(ip_address.as_deref()),
                ip_address,
                user_agent,
                signed_in_at: row.get("created_at"),
                last_active_at: row.get("last_active_at"),
                expires_at: row.get("expires_at"),
                is_current: row.get::<String, _>("id") == current_session_id,
            }
        })
        .collect::<Vec<_>>();

    Ok(AccountSessions {
        active_count: sessions.len() as i64,
        sessions,
        current_session_id: current_session_id.to_owned(),
    })
}

pub async fn account_security_log(
    pool: &PgPool,
    user_id: Uuid,
    query: AccountSecurityLogQuery,
) -> Result<AccountSecurityLog, AccountSecurityError> {
    let actions: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT event_type
        FROM security_audit_events
        WHERE actor_user_id = $1
        ORDER BY event_type ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
        FROM security_audit_events
        WHERE actor_user_id = $1
          AND ($2::text IS NULL OR event_type = $2)
        "#,
    )
    .bind(user_id)
    .bind(query.action.as_deref())
    .fetch_one(pool)
    .await?;

    let offset = (query.page - 1) * query.page_size;
    let rows = sqlx::query(
        r#"
        SELECT id, event_type, metadata, created_at,
               metadata->>'ipAddress' AS metadata_ip,
               metadata->>'userAgent' AS metadata_user_agent
        FROM security_audit_events
        WHERE actor_user_id = $1
          AND ($2::text IS NULL OR event_type = $2)
        ORDER BY created_at DESC, id DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(user_id)
    .bind(query.action.as_deref())
    .bind(query.page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let events = rows.into_iter().map(security_log_event_from_row).collect();
    let total_pages = if total == 0 {
        1
    } else {
        (total + query.page_size - 1) / query.page_size
    };

    Ok(AccountSecurityLog {
        events,
        actions,
        filters: AccountSecurityLogFilters {
            action: query.action,
            page: query.page,
            page_size: query.page_size,
        },
        pagination: AccountSecurityLogPagination {
            total,
            page: query.page,
            page_size: query.page_size,
            total_pages,
            has_previous: query.page > 1,
            has_next: query.page < total_pages,
        },
    })
}

pub async fn account_security_log_export(
    pool: &PgPool,
    user_id: Uuid,
    action: Option<String>,
    format: &str,
) -> Result<(String, String, String), AccountSecurityError> {
    let query = AccountSecurityLogQuery::normalized(action, Some(1), Some(50));
    let rows = sqlx::query(
        r#"
        SELECT id, event_type, metadata, created_at,
               metadata->>'ipAddress' AS metadata_ip,
               metadata->>'userAgent' AS metadata_user_agent
        FROM security_audit_events
        WHERE actor_user_id = $1
          AND ($2::text IS NULL OR event_type = $2)
        ORDER BY created_at DESC, id DESC
        "#,
    )
    .bind(user_id)
    .bind(query.action.as_deref())
    .fetch_all(pool)
    .await?;
    let events = rows
        .into_iter()
        .map(security_log_event_from_row)
        .collect::<Vec<_>>();

    match format {
        "json" => Ok((
            serde_json::to_string_pretty(&events).unwrap_or_else(|_| "[]".to_owned()),
            "application/json; charset=utf-8".to_owned(),
            "opengithub-security-log.json".to_owned(),
        )),
        _ => Ok((
            security_events_to_csv(&events),
            "text/csv; charset=utf-8".to_owned(),
            "opengithub-security-log.csv".to_owned(),
        )),
    }
}

pub async fn revoke_account_session(
    pool: &PgPool,
    user_id: Uuid,
    current_session_id: &str,
    target_session_id: &str,
) -> Result<RevokeAccountSessionResponse, AccountSecurityError> {
    if target_session_id == current_session_id {
        return Err(AccountSecurityError::Forbidden);
    }
    let result = sqlx::query(
        r#"
        UPDATE sessions
        SET revoked_at = now()
        WHERE id = $1
          AND user_id = $2
          AND revoked_at IS NULL
          AND expires_at > now()
        "#,
    )
    .bind(target_session_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AccountSecurityError::SessionNotFound);
    }
    record_session_audit(pool, user_id, "session.revoke", target_session_id, None).await?;
    Ok(RevokeAccountSessionResponse {
        revoked_id: target_session_id.to_owned(),
        sessions: account_sessions(pool, user_id, current_session_id).await?,
    })
}

pub async fn sign_out_everywhere(
    pool: &PgPool,
    user_id: Uuid,
    current_session_id: &str,
) -> Result<SignOutEverywhereResponse, AccountSecurityError> {
    let result = sqlx::query(
        r#"
        UPDATE sessions
        SET revoked_at = now()
        WHERE user_id = $1
          AND id <> $2
          AND revoked_at IS NULL
          AND expires_at > now()
        "#,
    )
    .bind(user_id)
    .bind(current_session_id)
    .execute(pool)
    .await?;
    let revoked_count = result.rows_affected() as i64;
    record_session_audit(
        pool,
        user_id,
        "session.sign_out_everywhere",
        current_session_id,
        Some(revoked_count),
    )
    .await?;
    Ok(SignOutEverywhereResponse {
        revoked_count,
        sessions: account_sessions(pool, user_id, current_session_id).await?,
    })
}

pub async fn create_account_security_sudo_grant(
    pool: &PgPool,
    user_id: Uuid,
    session_id: &str,
    request: CreateSudoGrantRequest,
) -> Result<AccountSecuritySettings, AccountSecurityError> {
    create_sudo_grant(pool, user_id, session_id, request)
        .await
        .map_err(map_sudo_error)?;
    sqlx::query("UPDATE sessions SET elevated_until = $3 WHERE id = $1 AND user_id = $2")
        .bind(session_id)
        .bind(user_id)
        .bind(Utc::now() + chrono::Duration::minutes(30))
        .execute(pool)
        .await?;
    account_security_settings(pool, user_id, Some(session_id)).await
}

pub async fn unlink_sign_in_method(
    pool: &PgPool,
    user_id: Uuid,
    session_id: &str,
    account_id: Uuid,
) -> Result<UnlinkSignInMethodResponse, AccountSecurityError> {
    if !sudo_state(pool, user_id, Some(session_id))
        .await
        .map_err(map_sudo_error)?
        .active
    {
        return Err(AccountSecurityError::SudoRequired);
    }

    let active_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM oauth_accounts WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    if active_count <= 1 {
        return Err(AccountSecurityError::LastIdentity);
    }

    let deleted = sqlx::query(
        r#"
        DELETE FROM oauth_accounts
        WHERE id = $1 AND user_id = $2
        RETURNING id, provider, email
        "#,
    )
    .bind(account_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    let Some(row) = deleted else {
        return Err(AccountSecurityError::Forbidden);
    };

    sqlx::query(
        r#"
        INSERT INTO security_audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'sign_in_method.unlink', 'oauth_account', $2, $3)
        "#,
    )
    .bind(user_id)
    .bind(account_id)
    .bind(json!({
        "provider": row.get::<String, _>("provider"),
        "email": row.get::<String, _>("email"),
    }))
    .execute(pool)
    .await?;

    Ok(UnlinkSignInMethodResponse {
        removed_id: account_id,
        settings: account_security_settings(pool, user_id, Some(session_id)).await?,
    })
}

pub async fn require_account_security_sudo(
    pool: &PgPool,
    user_id: Uuid,
    session_id: &str,
) -> Result<(), AccountSecurityError> {
    if sudo_state(pool, user_id, Some(session_id))
        .await
        .map_err(map_sudo_error)?
        .active
    {
        Ok(())
    } else {
        Err(AccountSecurityError::SudoRequired)
    }
}

fn provider_label(provider: String) -> String {
    match provider.as_str() {
        "google" => "Google".to_owned(),
        other => other.to_owned(),
    }
}

fn browser_label(user_agent: Option<&str>) -> String {
    let ua = user_agent.unwrap_or_default();
    if ua.contains("Edg/") {
        "Edge".to_owned()
    } else if ua.contains("Chrome/") || ua.contains("CriOS/") {
        "Chrome".to_owned()
    } else if ua.contains("Firefox/") || ua.contains("FxiOS/") {
        "Firefox".to_owned()
    } else if ua.contains("Safari/") {
        "Safari".to_owned()
    } else if ua.trim().is_empty() {
        "Unknown browser".to_owned()
    } else {
        "Browser".to_owned()
    }
}

fn device_label(user_agent: Option<&str>) -> String {
    let ua = user_agent.unwrap_or_default();
    let family = if ua.contains("iPhone") {
        "iPhone"
    } else if ua.contains("iPad") {
        "iPad"
    } else if ua.contains("Android") {
        "Android"
    } else if ua.contains("Mac OS X") || ua.contains("Macintosh") {
        "Mac"
    } else if ua.contains("Windows") {
        "Windows PC"
    } else if ua.contains("Linux") {
        "Linux"
    } else if ua.trim().is_empty() {
        "Unknown device"
    } else {
        "Device"
    };
    format!("{family} · {}", browser_label(user_agent))
}

fn location_label(ip_address: Option<&str>) -> String {
    match ip_address {
        Some("127.0.0.1") | Some("::1") => "Localhost".to_owned(),
        Some(value) if value.starts_with("10.") || value.starts_with("192.168.") => {
            "Private network".to_owned()
        }
        Some(value) if value.starts_with("172.") => "Private network".to_owned(),
        Some(_) => "Approximate location unavailable".to_owned(),
        None => "Unknown location".to_owned(),
    }
}

async fn record_session_audit(
    pool: &PgPool,
    user_id: Uuid,
    event_type: &str,
    target_session_id: &str,
    revoked_count: Option<i64>,
) -> Result<(), AccountSecurityError> {
    sqlx::query(
        r#"
        INSERT INTO security_audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, $2, 'session', NULL, $3)
        "#,
    )
    .bind(user_id)
    .bind(event_type)
    .bind(json!({
        "sessionId": target_session_id,
        "revokedCount": revoked_count,
    }))
    .execute(pool)
    .await?;
    Ok(())
}

fn security_log_event_from_row(row: sqlx::postgres::PgRow) -> AccountSecurityLogEvent {
    let metadata: Value = row.get("metadata");
    let ip_address = row.get::<Option<String>, _>("metadata_ip").or_else(|| {
        metadata
            .get("ipAddress")
            .and_then(Value::as_str)
            .map(str::to_owned)
    });
    let user_agent = row
        .get::<Option<String>, _>("metadata_user_agent")
        .or_else(|| {
            metadata
                .get("userAgent")
                .and_then(Value::as_str)
                .map(str::to_owned)
        });
    AccountSecurityLogEvent {
        id: row.get("id"),
        action: row.get("event_type"),
        location: location_label(ip_address.as_deref()),
        ip_address,
        user_agent,
        metadata,
        created_at: row.get("created_at"),
    }
}

fn security_events_to_csv(events: &[AccountSecurityLogEvent]) -> String {
    let mut csv = String::from("timestamp,action,ip,location,user_agent\n");
    for event in events {
        let row = [
            event.created_at.to_rfc3339(),
            event.action.clone(),
            event.ip_address.clone().unwrap_or_default(),
            event.location.clone(),
            event.user_agent.clone().unwrap_or_default(),
        ];
        csv.push_str(
            &row.into_iter()
                .map(csv_escape)
                .collect::<Vec<_>>()
                .join(","),
        );
        csv.push('\n');
    }
    csv
}

fn csv_escape(value: String) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value
    }
}

fn map_sudo_error(error: crate::domain::tokens::PersonalAccessTokenError) -> AccountSecurityError {
    match error {
        crate::domain::tokens::PersonalAccessTokenError::InvalidSudoConfirmation => {
            AccountSecurityError::InvalidSudoConfirmation
        }
        crate::domain::tokens::PersonalAccessTokenError::SudoRequired => {
            AccountSecurityError::SudoRequired
        }
        crate::domain::tokens::PersonalAccessTokenError::Sqlx(error) => {
            AccountSecurityError::Sqlx(error)
        }
        _ => AccountSecurityError::Forbidden,
    }
}
