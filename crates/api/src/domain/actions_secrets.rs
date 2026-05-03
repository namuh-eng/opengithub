use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::repositories::{
    can_admin_repository, get_repository_by_owner_name, Repository, RepositoryError,
    RepositoryVisibility,
};

const MAX_SECRET_VALUE_BYTES: usize = 64 * 1024;
const MAX_VARIABLE_VALUE_BYTES: usize = 48 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ActionsSecretsError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error("invalid Actions setting: {0}")]
    Invalid(String),
    #[error("Actions setting already exists")]
    Conflict,
    #[error("Actions setting was not found")]
    NotFound,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryActionsSecretsSettings {
    pub repository_id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub visibility: RepositoryVisibility,
    pub viewer_permission: String,
    pub can_edit: bool,
    pub secrets: Vec<ActionsSecretSummary>,
    pub variables: Vec<ActionsVariableSummary>,
    pub inherited_secrets: Vec<InheritedActionsSecretSummary>,
    pub inherited_variables: Vec<InheritedActionsVariableSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsSecretSummary {
    pub id: Uuid,
    pub name: String,
    pub scope: ActionsSettingScope,
    pub secret_configured: bool,
    pub storage_kind: String,
    pub visibility_policy: String,
    pub updated_by: Option<ActionsSettingActor>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsVariableSummary {
    pub id: Uuid,
    pub name: String,
    pub value: Option<String>,
    pub scope: ActionsSettingScope,
    pub visibility_policy: String,
    pub updated_by: Option<ActionsSettingActor>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InheritedActionsSecretSummary {
    pub name: String,
    pub scope: ActionsSettingScope,
    pub secret_configured: bool,
    pub visibility_policy: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InheritedActionsVariableSummary {
    pub name: String,
    pub value: Option<String>,
    pub scope: ActionsSettingScope,
    pub visibility_policy: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsSettingScope {
    pub kind: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsSettingActor {
    pub id: Uuid,
    pub login: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionsSecretMutation {
    pub name: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionsVariableMutation {
    pub name: Option<String>,
    pub value: String,
}

pub async fn repository_actions_secrets_settings_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryActionsSecretsSettings>, ActionsSecretsError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    repository_actions_secrets_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn create_repository_actions_secret_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    repo_name: &str,
    mutation: ActionsSecretMutation,
) -> Result<Option<RepositoryActionsSecretsSettings>, ActionsSecretsError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, repo_name).await? else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let name = normalize_setting_name(mutation.name.as_deref())?;
    let envelope = encrypt_secret_value(&mutation.value)?;
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO actions_secrets (
            repository_id, name, encrypted_value_ciphertext, encrypted_value_nonce,
            value_fingerprint, created_by_user_id, updated_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $6)
        "#,
    )
    .bind(repository.id)
    .bind(&name)
    .bind(&envelope.ciphertext)
    .bind(&envelope.nonce)
    .bind(&envelope.fingerprint)
    .bind(actor_user_id)
    .execute(&mut *transaction)
    .await
    .map_err(map_unique_conflict)?;
    insert_actions_settings_audit_tx(
        &mut transaction,
        repository.id,
        actor_user_id,
        "repository.actions_secret.create",
        vec!["secret".to_owned()],
        json!(null),
        json!({ "name": name, "scope": "repository", "secretConfigured": true }),
    )
    .await?;
    transaction.commit().await?;
    repository_actions_secrets_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn update_repository_actions_secret_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    repo_name: &str,
    secret_name: &str,
    mutation: ActionsSecretMutation,
) -> Result<Option<RepositoryActionsSecretsSettings>, ActionsSecretsError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, repo_name).await? else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let current_name = normalize_setting_name(Some(secret_name))?;
    let next_name = mutation
        .name
        .as_deref()
        .map(|value| normalize_setting_name(Some(value)))
        .transpose()?
        .unwrap_or_else(|| current_name.clone());
    let before = secret_audit_state(pool, repository.id, &current_name)
        .await?
        .ok_or(ActionsSecretsError::NotFound)?;
    let envelope = encrypt_secret_value(&mutation.value)?;
    let mut transaction = pool.begin().await?;
    let affected = sqlx::query(
        r#"
        UPDATE actions_secrets
        SET name = $3,
            encrypted_value_ciphertext = $4,
            encrypted_value_nonce = $5,
            value_fingerprint = $6,
            updated_by_user_id = $7
        WHERE repository_id = $1 AND scope_kind = 'repository' AND name = $2
        "#,
    )
    .bind(repository.id)
    .bind(&current_name)
    .bind(&next_name)
    .bind(&envelope.ciphertext)
    .bind(&envelope.nonce)
    .bind(&envelope.fingerprint)
    .bind(actor_user_id)
    .execute(&mut *transaction)
    .await
    .map_err(map_unique_conflict)?
    .rows_affected();
    if affected == 0 {
        return Err(ActionsSecretsError::NotFound);
    }
    insert_actions_settings_audit_tx(
        &mut transaction,
        repository.id,
        actor_user_id,
        "repository.actions_secret.update",
        vec!["secret".to_owned()],
        before,
        json!({ "name": next_name, "scope": "repository", "secretConfigured": true }),
    )
    .await?;
    transaction.commit().await?;
    repository_actions_secrets_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn delete_repository_actions_secret_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    repo_name: &str,
    secret_name: &str,
) -> Result<Option<RepositoryActionsSecretsSettings>, ActionsSecretsError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, repo_name).await? else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let name = normalize_setting_name(Some(secret_name))?;
    let before = secret_audit_state(pool, repository.id, &name)
        .await?
        .ok_or(ActionsSecretsError::NotFound)?;
    let mut transaction = pool.begin().await?;
    sqlx::query("DELETE FROM actions_secrets WHERE repository_id = $1 AND scope_kind = 'repository' AND name = $2")
        .bind(repository.id)
        .bind(&name)
        .execute(&mut *transaction)
        .await?;
    insert_actions_settings_audit_tx(
        &mut transaction,
        repository.id,
        actor_user_id,
        "repository.actions_secret.delete",
        vec!["secret".to_owned()],
        before,
        json!(null),
    )
    .await?;
    transaction.commit().await?;
    repository_actions_secrets_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn create_repository_actions_variable_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    repo_name: &str,
    mutation: ActionsVariableMutation,
) -> Result<Option<RepositoryActionsSecretsSettings>, ActionsSecretsError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, repo_name).await? else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let name = normalize_setting_name(mutation.name.as_deref())?;
    let value = normalize_variable_value(&mutation.value)?;
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO actions_variables (repository_id, name, value, created_by_user_id, updated_by_user_id)
        VALUES ($1, $2, $3, $4, $4)
        "#,
    )
    .bind(repository.id)
    .bind(&name)
    .bind(&value)
    .bind(actor_user_id)
    .execute(&mut *transaction)
    .await
    .map_err(map_unique_conflict)?;
    insert_actions_settings_audit_tx(
        &mut transaction,
        repository.id,
        actor_user_id,
        "repository.actions_variable.create",
        vec!["variable".to_owned()],
        json!(null),
        json!({ "name": name, "scope": "repository", "valueLength": value.len() }),
    )
    .await?;
    transaction.commit().await?;
    repository_actions_secrets_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn update_repository_actions_variable_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    repo_name: &str,
    variable_name: &str,
    mutation: ActionsVariableMutation,
) -> Result<Option<RepositoryActionsSecretsSettings>, ActionsSecretsError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, repo_name).await? else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let current_name = normalize_setting_name(Some(variable_name))?;
    let next_name = mutation
        .name
        .as_deref()
        .map(|value| normalize_setting_name(Some(value)))
        .transpose()?
        .unwrap_or_else(|| current_name.clone());
    let value = normalize_variable_value(&mutation.value)?;
    let before = variable_audit_state(pool, repository.id, &current_name)
        .await?
        .ok_or(ActionsSecretsError::NotFound)?;
    let mut transaction = pool.begin().await?;
    let affected = sqlx::query(
        r#"
        UPDATE actions_variables
        SET name = $3, value = $4, updated_by_user_id = $5
        WHERE repository_id = $1 AND scope_kind = 'repository' AND name = $2
        "#,
    )
    .bind(repository.id)
    .bind(&current_name)
    .bind(&next_name)
    .bind(&value)
    .bind(actor_user_id)
    .execute(&mut *transaction)
    .await
    .map_err(map_unique_conflict)?
    .rows_affected();
    if affected == 0 {
        return Err(ActionsSecretsError::NotFound);
    }
    insert_actions_settings_audit_tx(
        &mut transaction,
        repository.id,
        actor_user_id,
        "repository.actions_variable.update",
        vec!["variable".to_owned()],
        before,
        json!({ "name": next_name, "scope": "repository", "valueLength": value.len() }),
    )
    .await?;
    transaction.commit().await?;
    repository_actions_secrets_settings_for_repository(pool, &repository, actor_user_id).await
}

pub async fn delete_repository_actions_variable_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    repo_name: &str,
    variable_name: &str,
) -> Result<Option<RepositoryActionsSecretsSettings>, ActionsSecretsError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, repo_name).await? else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let name = normalize_setting_name(Some(variable_name))?;
    let before = variable_audit_state(pool, repository.id, &name)
        .await?
        .ok_or(ActionsSecretsError::NotFound)?;
    let mut transaction = pool.begin().await?;
    sqlx::query("DELETE FROM actions_variables WHERE repository_id = $1 AND scope_kind = 'repository' AND name = $2")
        .bind(repository.id)
        .bind(&name)
        .execute(&mut *transaction)
        .await?;
    insert_actions_settings_audit_tx(
        &mut transaction,
        repository.id,
        actor_user_id,
        "repository.actions_variable.delete",
        vec!["variable".to_owned()],
        before,
        json!(null),
    )
    .await?;
    transaction.commit().await?;
    repository_actions_secrets_settings_for_repository(pool, &repository, actor_user_id).await
}

async fn repository_actions_secrets_settings_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<Option<RepositoryActionsSecretsSettings>, ActionsSecretsError> {
    let permission = "admin".to_owned();
    let secrets = list_repository_secrets(pool, repository.id).await?;
    let variables = list_repository_variables(pool, repository.id, true).await?;
    Ok(Some(RepositoryActionsSecretsSettings {
        repository_id: repository.id,
        owner_login: repository.owner_login.clone(),
        name: repository.name.clone(),
        visibility: repository.visibility.clone(),
        viewer_permission: permission,
        can_edit: can_admin_repository(pool, repository, actor_user_id).await?,
        secrets,
        variables,
        inherited_secrets: Vec::new(),
        inherited_variables: Vec::new(),
    }))
}

async fn list_repository_secrets(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<ActionsSecretSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT actions_secrets.id, actions_secrets.name, actions_secrets.scope_kind, actions_secrets.scope_name,
               actions_secrets.storage_kind, actions_secrets.visibility_policy, actions_secrets.created_at,
               actions_secrets.updated_at, users.id AS actor_id,
               COALESCE(NULLIF(users.username, ''), users.email) AS actor_login,
               COALESCE(users.display_name, users.email) AS actor_display_name
        FROM actions_secrets
        LEFT JOIN users ON users.id = actions_secrets.updated_by_user_id
        WHERE actions_secrets.repository_id = $1 AND actions_secrets.scope_kind = 'repository'
        ORDER BY actions_secrets.updated_at DESC, actions_secrets.name ASC
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(secret_summary_from_row).collect()
}

async fn list_repository_variables(
    pool: &PgPool,
    repository_id: Uuid,
    include_values: bool,
) -> Result<Vec<ActionsVariableSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT actions_variables.id, actions_variables.name, actions_variables.value,
               actions_variables.scope_kind, actions_variables.scope_name,
               actions_variables.visibility_policy, actions_variables.created_at,
               actions_variables.updated_at, users.id AS actor_id,
               COALESCE(NULLIF(users.username, ''), users.email) AS actor_login,
               COALESCE(users.display_name, users.email) AS actor_display_name
        FROM actions_variables
        LEFT JOIN users ON users.id = actions_variables.updated_by_user_id
        WHERE actions_variables.repository_id = $1 AND actions_variables.scope_kind = 'repository'
        ORDER BY actions_variables.updated_at DESC, actions_variables.name ASC
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| variable_summary_from_row(row, include_values))
        .collect()
}

fn secret_summary_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<ActionsSecretSummary, sqlx::Error> {
    Ok(ActionsSecretSummary {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        scope: ActionsSettingScope {
            kind: row.try_get("scope_kind")?,
            name: row.try_get("scope_name")?,
        },
        secret_configured: true,
        storage_kind: row.try_get("storage_kind")?,
        visibility_policy: row.try_get("visibility_policy")?,
        updated_by: actor_from_row(&row)?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn variable_summary_from_row(
    row: sqlx::postgres::PgRow,
    include_values: bool,
) -> Result<ActionsVariableSummary, sqlx::Error> {
    Ok(ActionsVariableSummary {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        value: if include_values {
            Some(row.try_get("value")?)
        } else {
            None
        },
        scope: ActionsSettingScope {
            kind: row.try_get("scope_kind")?,
            name: row.try_get("scope_name")?,
        },
        visibility_policy: row.try_get("visibility_policy")?,
        updated_by: actor_from_row(&row)?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn actor_from_row(row: &sqlx::postgres::PgRow) -> Result<Option<ActionsSettingActor>, sqlx::Error> {
    Ok(row
        .try_get::<Option<Uuid>, _>("actor_id")?
        .map(|id| ActionsSettingActor {
            id,
            login: row
                .try_get::<Option<String>, _>("actor_login")
                .ok()
                .flatten()
                .unwrap_or_else(|| "unknown".to_owned()),
            display_name: row
                .try_get::<Option<String>, _>("actor_display_name")
                .ok()
                .flatten()
                .unwrap_or_else(|| "Unknown user".to_owned()),
        }))
}

async fn secret_audit_state(
    pool: &PgPool,
    repository_id: Uuid,
    name: &str,
) -> Result<Option<serde_json::Value>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT name, scope_kind, scope_name, storage_kind, visibility_policy, updated_at
        FROM actions_secrets
        WHERE repository_id = $1 AND scope_kind = 'repository' AND name = $2
        "#,
    )
    .bind(repository_id)
    .bind(name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| {
        json!({
            "name": row.get::<String, _>("name"),
            "scope": row.get::<String, _>("scope_kind"),
            "scopeName": row.get::<Option<String>, _>("scope_name"),
            "storageKind": row.get::<String, _>("storage_kind"),
            "visibilityPolicy": row.get::<String, _>("visibility_policy"),
            "secretConfigured": true,
            "updatedAt": row.get::<DateTime<Utc>, _>("updated_at"),
        })
    }))
}

async fn variable_audit_state(
    pool: &PgPool,
    repository_id: Uuid,
    name: &str,
) -> Result<Option<serde_json::Value>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT name, scope_kind, scope_name, visibility_policy, length(value) AS value_length, updated_at
        FROM actions_variables
        WHERE repository_id = $1 AND scope_kind = 'repository' AND name = $2
        "#,
    )
    .bind(repository_id)
    .bind(name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| {
        json!({
            "name": row.get::<String, _>("name"),
            "scope": row.get::<String, _>("scope_kind"),
            "scopeName": row.get::<Option<String>, _>("scope_name"),
            "visibilityPolicy": row.get::<String, _>("visibility_policy"),
            "valueLength": row.get::<Option<i32>, _>("value_length").unwrap_or_default(),
            "updatedAt": row.get::<DateTime<Utc>, _>("updated_at"),
        })
    }))
}

fn normalize_setting_name(value: Option<&str>) -> Result<String, ActionsSecretsError> {
    let raw = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ActionsSecretsError::Invalid("name is required".to_owned()))?;
    if raw.len() > 100 {
        return Err(ActionsSecretsError::Invalid(
            "name must be 100 characters or fewer".to_owned(),
        ));
    }
    let normalized = raw.to_ascii_uppercase();
    if !normalized.chars().enumerate().all(|(index, character)| {
        character == '_'
            || character.is_ascii_uppercase()
            || (index > 0 && character.is_ascii_digit())
    }) || normalized
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        return Err(ActionsSecretsError::Invalid(
            "name must start with a letter or underscore and contain only letters, numbers, and underscores".to_owned(),
        ));
    }
    if is_reserved_actions_name(&normalized) {
        return Err(ActionsSecretsError::Invalid(
            "name is reserved by Actions runtime".to_owned(),
        ));
    }
    Ok(normalized)
}

fn normalize_variable_value(value: &str) -> Result<String, ActionsSecretsError> {
    if value.len() > MAX_VARIABLE_VALUE_BYTES {
        return Err(ActionsSecretsError::Invalid(
            "variable value is too large".to_owned(),
        ));
    }
    Ok(value.to_owned())
}

fn encrypt_secret_value(value: &str) -> Result<SecretEnvelope, ActionsSecretsError> {
    if value.is_empty() {
        return Err(ActionsSecretsError::Invalid(
            "secret value is required".to_owned(),
        ));
    }
    if value.len() > MAX_SECRET_VALUE_BYTES {
        return Err(ActionsSecretsError::Invalid(
            "secret value is too large".to_owned(),
        ));
    }
    let key = std::env::var("ACTIONS_SECRETS_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "opengithub-local-actions-secrets-key".to_owned());
    let nonce = Uuid::new_v4().to_string();
    let bytes = value.as_bytes();
    let mut encrypted = Vec::with_capacity(bytes.len());
    let mut offset = 0usize;
    let mut counter = 0u64;
    while offset < bytes.len() {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hasher.update(nonce.as_bytes());
        hasher.update(counter.to_le_bytes());
        let block = hasher.finalize();
        for byte in block {
            if offset == bytes.len() {
                break;
            }
            encrypted.push(bytes[offset] ^ byte);
            offset += 1;
        }
        counter += 1;
    }
    let fingerprint = format!("sha256:{}", hex_sha256(value.as_bytes()));
    Ok(SecretEnvelope {
        ciphertext: STANDARD_NO_PAD.encode(encrypted),
        nonce,
        fingerprint,
    })
}

fn hex_sha256(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn is_reserved_actions_name(name: &str) -> bool {
    name == "GITHUB_TOKEN"
        || name == "ACTIONS_RUNTIME_TOKEN"
        || name == "ACTIONS_ID_TOKEN_REQUEST_TOKEN"
        || name.starts_with("GITHUB_")
        || name.starts_with("RUNNER_")
        || name.starts_with("ACTIONS_")
}

fn ensure_repository_mutable(repository: &Repository) -> Result<(), ActionsSecretsError> {
    if repository.is_archived {
        Err(ActionsSecretsError::Repository(
            RepositoryError::ArchivedRepositoryReadOnly,
        ))
    } else {
        Ok(())
    }
}

async fn require_repository_admin(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<(), ActionsSecretsError> {
    if can_admin_repository(pool, repository, actor_user_id).await? {
        Ok(())
    } else {
        Err(ActionsSecretsError::Repository(
            RepositoryError::PermissionDenied,
        ))
    }
}

async fn insert_actions_settings_audit_tx(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    repository_id: Uuid,
    actor_user_id: Uuid,
    event_type: &str,
    changed_fields: Vec<String>,
    before_state: serde_json::Value,
    after_state: serde_json::Value,
) -> Result<(), sqlx::Error> {
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

fn map_unique_conflict(error: sqlx::Error) -> ActionsSecretsError {
    match &error {
        sqlx::Error::Database(database_error) if database_error.is_unique_violation() => {
            ActionsSecretsError::Conflict
        }
        _ => ActionsSecretsError::Sqlx(error),
    }
}

struct SecretEnvelope {
    ciphertext: String,
    nonce: String,
    fingerprint: String,
}
