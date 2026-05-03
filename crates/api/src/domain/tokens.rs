use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
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
    #[error("sudo confirmation is invalid")]
    InvalidSudoConfirmation,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersonalAccessTokenList {
    pub tokens: Vec<PersonalAccessTokenSummary>,
    #[serde(rename = "sudo")]
    pub sudo: SudoState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersonalAccessTokenSummary {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub token_type: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    #[serde(rename = "resourceOwner")]
    pub resource_owner: TokenResourceOwner,
    #[serde(rename = "repositoryAccess")]
    pub repository_access: String,
    #[serde(rename = "selectedRepositories")]
    pub selected_repositories: Vec<TokenRepositorySummary>,
    pub status: String,
    #[serde(rename = "lastUsedAt")]
    pub last_used_at: Option<DateTime<Utc>>,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<DateTime<Utc>>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersonalAccessTokenNewContext {
    #[serde(rename = "sudo")]
    pub sudo: SudoState,
    #[serde(rename = "resourceOwners")]
    pub resource_owners: Vec<TokenResourceOwner>,
    pub repositories: Vec<TokenRepositorySummary>,
    #[serde(rename = "permissionGroups")]
    pub permission_groups: Vec<TokenPermissionGroup>,
    #[serde(rename = "defaultExpirationDays")]
    pub default_expiration_days: i64,
    #[serde(rename = "maxExpirationDays")]
    pub max_expiration_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenResourceOwner {
    pub id: Uuid,
    pub kind: String,
    pub login: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenRepositorySummary {
    pub id: Uuid,
    pub owner: String,
    pub name: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub visibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenPermissionGroup {
    pub key: String,
    pub label: String,
    pub permissions: Vec<TokenPermissionChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenPermissionChoice {
    pub key: String,
    pub label: String,
    pub levels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SudoState {
    pub active: bool,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(rename = "requiredFor")]
    pub required_for: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSudoGrantRequest {
    pub confirmation: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SudoGrantResponse {
    #[serde(rename = "sudo")]
    pub sudo: SudoState,
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

pub async fn personal_access_token_list(
    pool: &PgPool,
    user_id: Uuid,
    session_id: Option<&str>,
) -> Result<PersonalAccessTokenList, PersonalAccessTokenError> {
    let rows = sqlx::query(
        r#"
        SELECT
            pat.id, pat.name, pat.description, pat.token_type, pat.prefix, pat.scopes,
            pat.repository_access, pat.status, pat.last_used_at, pat.expires_at,
            pat.revoked_at, pat.created_at,
            owner_user.id AS owner_user_id,
            owner_user.username AS owner_username,
            owner_user.email AS owner_email,
            owner_user.display_name AS owner_display_name,
            owner_user.avatar_url AS owner_avatar_url,
            owner_org.id AS owner_org_id,
            owner_org.slug AS owner_org_slug,
            owner_org.display_name AS owner_org_display_name
        FROM personal_access_tokens pat
        LEFT JOIN users owner_user ON owner_user.id = pat.resource_owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = pat.resource_owner_organization_id
        WHERE pat.user_id = $1
        ORDER BY pat.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut tokens = Vec::with_capacity(rows.len());
    for row in rows {
        let token_id: Uuid = row.get("id");
        tokens.push(PersonalAccessTokenSummary {
            id: token_id,
            name: row.get("name"),
            description: row.get("description"),
            token_type: row.get("token_type"),
            prefix: row.get("prefix"),
            scopes: row.get("scopes"),
            resource_owner: resource_owner_from_token_row(&row),
            repository_access: row.get("repository_access"),
            selected_repositories: selected_repositories_for_token(pool, token_id).await?,
            status: effective_token_status(
                row.get("status"),
                row.get("revoked_at"),
                row.get("expires_at"),
            ),
            last_used_at: row.get("last_used_at"),
            expires_at: row.get("expires_at"),
            revoked_at: row.get("revoked_at"),
            created_at: row.get("created_at"),
        });
    }

    Ok(PersonalAccessTokenList {
        tokens,
        sudo: sudo_state(pool, user_id, session_id).await?,
    })
}

pub async fn personal_access_token_new_context(
    pool: &PgPool,
    user_id: Uuid,
    session_id: Option<&str>,
) -> Result<PersonalAccessTokenNewContext, PersonalAccessTokenError> {
    Ok(PersonalAccessTokenNewContext {
        sudo: sudo_state(pool, user_id, session_id).await?,
        resource_owners: token_resource_owners(pool, user_id).await?,
        repositories: token_repositories(pool, user_id).await?,
        permission_groups: default_permission_groups(),
        default_expiration_days: 30,
        max_expiration_days: 366,
    })
}

pub async fn create_sudo_grant(
    pool: &PgPool,
    user_id: Uuid,
    session_id: &str,
    request: CreateSudoGrantRequest,
) -> Result<SudoGrantResponse, PersonalAccessTokenError> {
    let email: String = sqlx::query_scalar("SELECT email FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    let confirmation = request
        .confirmation
        .or(request.email)
        .unwrap_or_default()
        .trim()
        .to_owned();
    if !confirmation.eq_ignore_ascii_case(&email) {
        return Err(PersonalAccessTokenError::InvalidSudoConfirmation);
    }

    let expires_at = Utc::now() + Duration::minutes(30);
    sqlx::query(
        r#"
        INSERT INTO sudo_grants (session_id, user_id, method, expires_at)
        VALUES ($1, $2, 'session_confirmation', $3)
        "#,
    )
    .bind(session_id)
    .bind(user_id)
    .bind(expires_at)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO security_audit_events (actor_user_id, event_type, metadata)
        VALUES ($1, 'sudo.grant.create', $2)
        "#,
    )
    .bind(user_id)
    .bind(json!({ "sessionId": session_id, "expiresAt": expires_at }))
    .execute(pool)
    .await?;

    Ok(SudoGrantResponse {
        sudo: sudo_state(pool, user_id, Some(session_id)).await?,
    })
}

pub async fn sudo_state(
    pool: &PgPool,
    user_id: Uuid,
    session_id: Option<&str>,
) -> Result<SudoState, PersonalAccessTokenError> {
    let expires_at = if let Some(session_id) = session_id {
        sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            r#"
            SELECT max(expires_at)
            FROM sudo_grants
            WHERE session_id = $1
              AND user_id = $2
              AND revoked_at IS NULL
              AND expires_at > now()
            "#,
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?
    } else {
        None
    };

    Ok(SudoState {
        active: expires_at.is_some(),
        expires_at,
        required_for: vec![
            "create_personal_access_token".to_owned(),
            "revoke_personal_access_token".to_owned(),
        ],
    })
}

async fn token_resource_owners(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<TokenResourceOwner>, PersonalAccessTokenError> {
    let user_row = sqlx::query(
        r#"
        SELECT id, username, email, display_name, avatar_url
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    let mut owners = vec![TokenResourceOwner {
        id: user_row.get("id"),
        kind: "user".to_owned(),
        login: user_login(user_row.get("username"), user_row.get("email")),
        display_name: user_row
            .get::<Option<String>, _>("display_name")
            .unwrap_or_else(|| user_row.get("email")),
        avatar_url: user_row.get("avatar_url"),
    }];

    let org_rows = sqlx::query(
        r#"
        SELECT organizations.id, organizations.slug, organizations.display_name
        FROM organizations
        JOIN organization_memberships
          ON organization_memberships.organization_id = organizations.id
        WHERE organization_memberships.user_id = $1
        ORDER BY lower(organizations.slug)
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    owners.extend(org_rows.into_iter().map(|row| TokenResourceOwner {
        id: row.get("id"),
        kind: "organization".to_owned(),
        login: row.get("slug"),
        display_name: row.get("display_name"),
        avatar_url: None,
    }));

    Ok(owners)
}

async fn token_repositories(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<TokenRepositorySummary>, PersonalAccessTokenError> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT repositories.id,
            COALESCE(owner_user.username, owner_user.email, owner_org.slug) AS owner,
            repositories.name,
            repositories.visibility
        FROM repositories
        LEFT JOIN users owner_user ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = repositories.owner_organization_id
        LEFT JOIN organization_memberships
          ON organization_memberships.organization_id = repositories.owner_organization_id
         AND organization_memberships.user_id = $1
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = repositories.id
         AND repository_permissions.user_id = $1
        WHERE repositories.owner_user_id = $1
           OR organization_memberships.user_id IS NOT NULL
           OR repository_permissions.user_id IS NOT NULL
        ORDER BY owner, repositories.name
        LIMIT 200
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(repository_from_row).collect())
}

async fn selected_repositories_for_token(
    pool: &PgPool,
    token_id: Uuid,
) -> Result<Vec<TokenRepositorySummary>, PersonalAccessTokenError> {
    let rows = sqlx::query(
        r#"
        SELECT repositories.id,
            COALESCE(owner_user.username, owner_user.email, owner_org.slug) AS owner,
            repositories.name,
            repositories.visibility
        FROM personal_access_token_repositories selected
        JOIN repositories ON repositories.id = selected.repository_id
        LEFT JOIN users owner_user ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = repositories.owner_organization_id
        WHERE selected.token_id = $1
        ORDER BY owner, repositories.name
        "#,
    )
    .bind(token_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(repository_from_row).collect())
}

fn repository_from_row(row: sqlx::postgres::PgRow) -> TokenRepositorySummary {
    let owner: String = row.get("owner");
    let name: String = row.get("name");
    TokenRepositorySummary {
        id: row.get("id"),
        full_name: format!("{owner}/{name}"),
        owner,
        name,
        visibility: row.get("visibility"),
    }
}

fn resource_owner_from_token_row(row: &sqlx::postgres::PgRow) -> TokenResourceOwner {
    if let Some(id) = row.get::<Option<Uuid>, _>("owner_org_id") {
        return TokenResourceOwner {
            id,
            kind: "organization".to_owned(),
            login: row.get("owner_org_slug"),
            display_name: row.get("owner_org_display_name"),
            avatar_url: None,
        };
    }

    let id = row
        .get::<Option<Uuid>, _>("owner_user_id")
        .unwrap_or_else(|| row.get("id"));
    let email = row
        .get::<Option<String>, _>("owner_email")
        .unwrap_or_else(|| "unknown@opengithub.local".to_owned());
    TokenResourceOwner {
        id,
        kind: "user".to_owned(),
        login: user_login(row.get("owner_username"), email.clone()),
        display_name: row
            .get::<Option<String>, _>("owner_display_name")
            .unwrap_or(email),
        avatar_url: row.get("owner_avatar_url"),
    }
}

fn user_login(username: Option<String>, email: String) -> String {
    username.unwrap_or_else(|| {
        email
            .split('@')
            .next()
            .unwrap_or("user")
            .replace('.', "-")
            .to_ascii_lowercase()
    })
}

fn effective_token_status(
    status: String,
    revoked_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
) -> String {
    if revoked_at.is_some() {
        "revoked".to_owned()
    } else if expires_at.is_some_and(|expires_at| expires_at <= Utc::now()) {
        "expired".to_owned()
    } else {
        status
    }
}

fn default_permission_groups() -> Vec<TokenPermissionGroup> {
    vec![
        TokenPermissionGroup {
            key: "repositories".to_owned(),
            label: "Repositories".to_owned(),
            permissions: vec![
                TokenPermissionChoice {
                    key: "contents".to_owned(),
                    label: "Contents".to_owned(),
                    levels: vec!["none".to_owned(), "read".to_owned(), "write".to_owned()],
                },
                TokenPermissionChoice {
                    key: "pull_requests".to_owned(),
                    label: "Pull requests".to_owned(),
                    levels: vec!["none".to_owned(), "read".to_owned(), "write".to_owned()],
                },
                TokenPermissionChoice {
                    key: "issues".to_owned(),
                    label: "Issues".to_owned(),
                    levels: vec!["none".to_owned(), "read".to_owned(), "write".to_owned()],
                },
            ],
        },
        TokenPermissionGroup {
            key: "packages".to_owned(),
            label: "Packages".to_owned(),
            permissions: vec![TokenPermissionChoice {
                key: "packages".to_owned(),
                label: "Packages".to_owned(),
                levels: vec!["none".to_owned(), "read".to_owned(), "write".to_owned()],
            }],
        },
        TokenPermissionGroup {
            key: "account".to_owned(),
            label: "Account".to_owned(),
            permissions: vec![TokenPermissionChoice {
                key: "profile".to_owned(),
                label: "Profile metadata".to_owned(),
                levels: vec!["none".to_owned(), "read".to_owned()],
            }],
        },
    ]
}
