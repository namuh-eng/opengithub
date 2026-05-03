use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedPersonalAccessToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub scopes: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PersonalAccessTokenError {
    #[error("personal access token is invalid")]
    Invalid,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl VerifiedPersonalAccessToken {
    pub fn allows_repo_read(&self) -> bool {
        self.scopes.iter().any(|scope| {
            matches!(
                scope.as_str(),
                "repo" | "repo:read" | "repo:write" | "repository:read" | "repository:write"
            )
        })
    }

    pub fn allows_repo_write(&self) -> bool {
        self.scopes
            .iter()
            .any(|scope| matches!(scope.as_str(), "repo" | "repo:write" | "repository:write"))
    }

    pub fn allows_package_read(&self) -> bool {
        self.scopes.iter().any(|scope| {
            matches!(
                scope.as_str(),
                "packages:read"
                    | "packages:write"
                    | "packages:admin"
                    | "read:packages"
                    | "write:packages"
                    | "admin:packages"
            )
        })
    }

    pub fn allows_package_write(&self) -> bool {
        self.scopes.iter().any(|scope| {
            matches!(
                scope.as_str(),
                "packages:write" | "packages:admin" | "write:packages" | "admin:packages"
            )
        })
    }
}

pub async fn verify_personal_access_token(
    pool: &PgPool,
    token: &str,
) -> Result<VerifiedPersonalAccessToken, PersonalAccessTokenError> {
    let token = token.trim();
    if token.is_empty() {
        return Err(PersonalAccessTokenError::Invalid);
    }

    let rows = sqlx::query(
        r#"
        SELECT id, user_id, token_hash, scopes, expires_at
        FROM personal_access_tokens
        WHERE revoked_at IS NULL
          AND $1 LIKE prefix || '%'
        ORDER BY length(prefix) DESC
        LIMIT 8
        "#,
    )
    .bind(token)
    .fetch_all(pool)
    .await?;

    let expected_hash = hash_personal_access_token(token);
    for row in rows {
        let token_hash: String = row.get("token_hash");
        let expires_at: Option<DateTime<Utc>> = row.get("expires_at");
        if token_hash != expected_hash
            || expires_at.is_some_and(|expires_at| expires_at <= Utc::now())
        {
            continue;
        }

        let verified = VerifiedPersonalAccessToken {
            id: row.get("id"),
            user_id: row.get("user_id"),
            scopes: row.get("scopes"),
        };
        sqlx::query("UPDATE personal_access_tokens SET last_used_at = now() WHERE id = $1")
            .bind(verified.id)
            .execute(pool)
            .await?;
        return Ok(verified);
    }

    Err(PersonalAccessTokenError::Invalid)
}

pub fn hash_personal_access_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    format!("sha256:{hex}")
}
