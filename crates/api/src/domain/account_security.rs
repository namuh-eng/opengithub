use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
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
