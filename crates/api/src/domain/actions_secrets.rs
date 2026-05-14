use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use std::collections::BTreeMap;
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
    pub scope_kind: Option<String>,
    pub scope_name: Option<String>,
    pub current_scope_kind: Option<String>,
    pub current_scope_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionsVariableMutation {
    pub name: Option<String>,
    pub value: String,
    pub scope_kind: Option<String>,
    pub scope_name: Option<String>,
    pub current_scope_kind: Option<String>,
    pub current_scope_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRuntimeResolution {
    #[serde(skip_serializing)]
    pub secrets: BTreeMap<String, String>,
    pub variables: BTreeMap<String, String>,
    pub diagnostics: ActionsRuntimeResolutionDiagnostics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRuntimeResolutionDiagnostics {
    pub secret_count: usize,
    pub variable_count: usize,
    pub blocked_secret_count: usize,
    pub blocked_variable_count: usize,
    pub scopes: Vec<ActionsRuntimeScopeCount>,
    pub blocked_reasons: Vec<String>,
    pub redaction_marker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionsRuntimeScopeCount {
    pub scope: String,
    pub secrets: usize,
    pub variables: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionsRuntimeResolutionRequest {
    pub repository_id: Uuid,
    pub event: String,
    pub fork_pull_request: bool,
    pub environment: Option<String>,
    pub environment_approved: bool,
    pub explicit_secret_names: Option<Vec<String>>,
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
    let scope = normalize_setting_scope(
        mutation.scope_kind.as_deref(),
        mutation.scope_name.as_deref(),
    )?;
    ensure_secret_name_available(pool, repository.id, &scope, &name).await?;
    let envelope = encrypt_secret_value(&mutation.value)?;
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO actions_secrets (
            repository_id, scope_kind, scope_name, name, encrypted_value_ciphertext, encrypted_value_nonce,
            value_fingerprint, created_by_user_id, updated_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
        "#,
    )
    .bind(repository.id)
    .bind(&scope.kind)
    .bind(&scope.name)
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
        json!({ "name": name, "scope": scope.kind, "scopeName": scope.name, "secretConfigured": true }),
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
    let current_scope = normalize_setting_scope(
        mutation.current_scope_kind.as_deref(),
        mutation.current_scope_name.as_deref(),
    )?;
    let next_name = mutation
        .name
        .as_deref()
        .map(|value| normalize_setting_name(Some(value)))
        .transpose()?
        .unwrap_or_else(|| current_name.clone());
    let before = secret_audit_state(pool, repository.id, &current_scope, &current_name)
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
        WHERE repository_id = $1
          AND name = $2
          AND scope_kind = $8
          AND COALESCE(scope_name, '') = COALESCE($9, '')
        "#,
    )
    .bind(repository.id)
    .bind(&current_name)
    .bind(&next_name)
    .bind(&envelope.ciphertext)
    .bind(&envelope.nonce)
    .bind(&envelope.fingerprint)
    .bind(actor_user_id)
    .bind(&current_scope.kind)
    .bind(&current_scope.name)
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
        json!({ "name": next_name, "scope": current_scope.kind, "scopeName": current_scope.name, "secretConfigured": true }),
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
    scope_kind: Option<&str>,
    scope_name: Option<&str>,
) -> Result<Option<RepositoryActionsSecretsSettings>, ActionsSecretsError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, repo_name).await? else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let name = normalize_setting_name(Some(secret_name))?;
    let scope = normalize_setting_scope(scope_kind, scope_name)?;
    let before = secret_audit_state(pool, repository.id, &scope, &name)
        .await?
        .ok_or(ActionsSecretsError::NotFound)?;
    let mut transaction = pool.begin().await?;
    sqlx::query(
        "DELETE FROM actions_secrets WHERE repository_id = $1 AND name = $2 AND scope_kind = $3 AND COALESCE(scope_name, '') = COALESCE($4, '')",
    )
        .bind(repository.id)
        .bind(&name)
        .bind(&scope.kind)
        .bind(&scope.name)
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
    let scope = normalize_setting_scope(
        mutation.scope_kind.as_deref(),
        mutation.scope_name.as_deref(),
    )?;
    ensure_variable_name_available(pool, repository.id, &scope, &name).await?;
    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO actions_variables (repository_id, scope_kind, scope_name, name, value, created_by_user_id, updated_by_user_id)
        VALUES ($1, $2, $3, $4, $5, $6, $6)
        "#,
    )
    .bind(repository.id)
    .bind(&scope.kind)
    .bind(&scope.name)
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
        json!({ "name": name, "scope": scope.kind, "scopeName": scope.name, "valueLength": value.len() }),
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
    let current_scope = normalize_setting_scope(
        mutation.current_scope_kind.as_deref(),
        mutation.current_scope_name.as_deref(),
    )?;
    let next_name = mutation
        .name
        .as_deref()
        .map(|value| normalize_setting_name(Some(value)))
        .transpose()?
        .unwrap_or_else(|| current_name.clone());
    let value = normalize_variable_value(&mutation.value)?;
    let before = variable_audit_state(pool, repository.id, &current_scope, &current_name)
        .await?
        .ok_or(ActionsSecretsError::NotFound)?;
    let mut transaction = pool.begin().await?;
    let affected = sqlx::query(
        r#"
        UPDATE actions_variables
        SET name = $3, value = $4, updated_by_user_id = $5
        WHERE repository_id = $1
          AND name = $2
          AND scope_kind = $6
          AND COALESCE(scope_name, '') = COALESCE($7, '')
        "#,
    )
    .bind(repository.id)
    .bind(&current_name)
    .bind(&next_name)
    .bind(&value)
    .bind(actor_user_id)
    .bind(&current_scope.kind)
    .bind(&current_scope.name)
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
        json!({ "name": next_name, "scope": current_scope.kind, "scopeName": current_scope.name, "valueLength": value.len() }),
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
    scope_kind: Option<&str>,
    scope_name: Option<&str>,
) -> Result<Option<RepositoryActionsSecretsSettings>, ActionsSecretsError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, repo_name).await? else {
        return Ok(None);
    };
    require_repository_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let name = normalize_setting_name(Some(variable_name))?;
    let scope = normalize_setting_scope(scope_kind, scope_name)?;
    let before = variable_audit_state(pool, repository.id, &scope, &name)
        .await?
        .ok_or(ActionsSecretsError::NotFound)?;
    let mut transaction = pool.begin().await?;
    sqlx::query(
        "DELETE FROM actions_variables WHERE repository_id = $1 AND name = $2 AND scope_kind = $3 AND COALESCE(scope_name, '') = COALESCE($4, '')",
    )
        .bind(repository.id)
        .bind(&name)
        .bind(&scope.kind)
        .bind(&scope.name)
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

pub async fn resolve_actions_runtime_context(
    pool: &PgPool,
    request: ActionsRuntimeResolutionRequest,
) -> Result<ActionsRuntimeResolution, ActionsSecretsError> {
    let mut secrets = BTreeMap::new();
    let mut variables = BTreeMap::new();
    let mut blocked_reasons = Vec::new();
    let mut blocked_secret_count = 0usize;
    let mut blocked_variable_count = 0usize;
    let fork_blocks_secrets =
        request.fork_pull_request && request.event.eq_ignore_ascii_case("pull_request");
    let explicit_secret_names = request.explicit_secret_names.as_ref().map(|names| {
        names
            .iter()
            .filter_map(|name| normalize_setting_name(Some(name)).ok())
            .collect::<Vec<_>>()
    });

    let secret_rows = sqlx::query(
        r#"
        SELECT name, scope_kind, scope_name, encrypted_value_ciphertext, encrypted_value_nonce
        FROM actions_secrets
        WHERE repository_id = $1
        ORDER BY CASE scope_kind
                   WHEN 'organization' THEN 1
                   WHEN 'repository' THEN 2
                   WHEN 'environment' THEN 3
                   ELSE 4
                 END,
                 name
        "#,
    )
    .bind(request.repository_id)
    .fetch_all(pool)
    .await?;

    let variable_rows = sqlx::query(
        r#"
        SELECT name, value, scope_kind, scope_name
        FROM actions_variables
        WHERE repository_id = $1
        ORDER BY CASE scope_kind
                   WHEN 'organization' THEN 1
                   WHEN 'repository' THEN 2
                   WHEN 'environment' THEN 3
                   ELSE 4
                 END,
                 name
        "#,
    )
    .bind(request.repository_id)
    .fetch_all(pool)
    .await?;

    let environment_rows = sqlx::query(
        r#"
        SELECT lower(name) AS name, protection_rules_enabled
        FROM actions_environments
        WHERE repository_id = $1
        "#,
    )
    .bind(request.repository_id)
    .fetch_all(pool)
    .await?;
    let protected_environments = environment_rows
        .into_iter()
        .map(|row| {
            (
                row.get::<String, _>("name"),
                row.get::<bool, _>("protection_rules_enabled"),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut scope_counts = BTreeMap::<String, (usize, usize)>::new();

    for row in secret_rows {
        let name: String = row.get("name");
        let scope_kind: String = row.get("scope_kind");
        let scope_name: Option<String> = row.get("scope_name");
        let scope = runtime_scope_label(&scope_kind, scope_name.as_deref());
        if explicit_secret_names
            .as_ref()
            .is_some_and(|names| !names.iter().any(|allowed| allowed == &name))
        {
            blocked_secret_count += 1;
            push_unique_reason(&mut blocked_reasons, "not_explicitly_allowed");
            continue;
        }
        if fork_blocks_secrets {
            blocked_secret_count += 1;
            push_unique_reason(&mut blocked_reasons, "fork_pull_request");
            continue;
        }
        if environment_scope_blocked(
            &scope_kind,
            scope_name.as_deref(),
            &request,
            &protected_environments,
        ) {
            blocked_secret_count += 1;
            push_unique_reason(&mut blocked_reasons, "environment_not_approved");
            continue;
        }
        let plaintext = decrypt_secret_value(
            row.get("encrypted_value_ciphertext"),
            row.get("encrypted_value_nonce"),
        )?;
        secrets.insert(name, plaintext);
        scope_counts.entry(scope).or_default().0 += 1;
    }

    for row in variable_rows {
        let name: String = row.get("name");
        let scope_kind: String = row.get("scope_kind");
        let scope_name: Option<String> = row.get("scope_name");
        let scope = runtime_scope_label(&scope_kind, scope_name.as_deref());
        if environment_scope_blocked(
            &scope_kind,
            scope_name.as_deref(),
            &request,
            &protected_environments,
        ) {
            blocked_variable_count += 1;
            push_unique_reason(&mut blocked_reasons, "environment_not_approved");
            continue;
        }
        variables.insert(name, row.get("value"));
        scope_counts.entry(scope).or_default().1 += 1;
    }

    let scopes = scope_counts
        .into_iter()
        .map(
            |(scope, (secret_count, variable_count))| ActionsRuntimeScopeCount {
                scope,
                secrets: secret_count,
                variables: variable_count,
            },
        )
        .collect::<Vec<_>>();

    Ok(ActionsRuntimeResolution {
        diagnostics: ActionsRuntimeResolutionDiagnostics {
            secret_count: secrets.len(),
            variable_count: variables.len(),
            blocked_secret_count,
            blocked_variable_count,
            scopes,
            blocked_reasons,
            redaction_marker: "::add-mask::***".to_owned(),
        },
        secrets,
        variables,
    })
}

pub fn mask_actions_secret_values(content: &str, secret_values: &[String]) -> String {
    let mut masked = content.to_owned();
    for secret in secret_values {
        if secret.len() < 3 {
            continue;
        }
        masked = masked.replace(secret, "***");
        let encoded = STANDARD_NO_PAD.encode(secret.as_bytes());
        if encoded.len() >= 3 {
            masked = masked.replace(&encoded, "***");
        }
    }
    masked
}

pub async fn actions_secret_redaction_values(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<String>, ActionsSecretsError> {
    let rows = sqlx::query(
        r#"
        SELECT encrypted_value_ciphertext, encrypted_value_nonce
        FROM actions_secrets
        WHERE repository_id = $1
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            decrypt_secret_value(
                row.get("encrypted_value_ciphertext"),
                row.get("encrypted_value_nonce"),
            )
        })
        .collect()
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
        WHERE actions_secrets.repository_id = $1
        ORDER BY CASE actions_secrets.scope_kind
                   WHEN 'repository' THEN 1
                   WHEN 'environment' THEN 2
                   WHEN 'organization' THEN 3
                   ELSE 4
                 END,
                 actions_secrets.updated_at DESC, actions_secrets.name ASC
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
        WHERE actions_variables.repository_id = $1
        ORDER BY CASE actions_variables.scope_kind
                   WHEN 'repository' THEN 1
                   WHEN 'environment' THEN 2
                   WHEN 'organization' THEN 3
                   ELSE 4
                 END,
                 actions_variables.updated_at DESC, actions_variables.name ASC
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
    scope: &NormalizedSettingScope,
    name: &str,
) -> Result<Option<serde_json::Value>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT name, scope_kind, scope_name, storage_kind, visibility_policy, updated_at
        FROM actions_secrets
        WHERE repository_id = $1
          AND scope_kind = $2
          AND COALESCE(scope_name, '') = COALESCE($3, '')
          AND name = $4
        "#,
    )
    .bind(repository_id)
    .bind(&scope.kind)
    .bind(&scope.name)
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
    scope: &NormalizedSettingScope,
    name: &str,
) -> Result<Option<serde_json::Value>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT name, scope_kind, scope_name, visibility_policy, length(value) AS value_length, updated_at
        FROM actions_variables
        WHERE repository_id = $1
          AND scope_kind = $2
          AND COALESCE(scope_name, '') = COALESCE($3, '')
          AND name = $4
        "#,
    )
    .bind(repository_id)
    .bind(&scope.kind)
    .bind(&scope.name)
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

struct NormalizedSettingScope {
    kind: String,
    name: Option<String>,
}

fn normalize_setting_scope(
    kind: Option<&str>,
    name: Option<&str>,
) -> Result<NormalizedSettingScope, ActionsSecretsError> {
    let kind = kind
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("repository")
        .to_ascii_lowercase();
    if !matches!(kind.as_str(), "repository" | "environment") {
        return Err(ActionsSecretsError::Invalid(
            "scope kind must be repository or environment".to_owned(),
        ));
    }
    let name = name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    if kind == "repository" {
        return Ok(NormalizedSettingScope { kind, name: None });
    }
    let Some(name) = name else {
        return Err(ActionsSecretsError::Invalid(
            "environment scope requires an environment name".to_owned(),
        ));
    };
    if name.len() > 64 {
        return Err(ActionsSecretsError::Invalid(
            "environment name must be 64 characters or fewer".to_owned(),
        ));
    }
    Ok(NormalizedSettingScope {
        kind,
        name: Some(name),
    })
}

async fn ensure_secret_name_available(
    pool: &PgPool,
    repository_id: Uuid,
    scope: &NormalizedSettingScope,
    name: &str,
) -> Result<(), ActionsSecretsError> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM actions_secrets WHERE repository_id = $1 AND scope_kind = $2 AND COALESCE(scope_name, '') = COALESCE($3, '') AND name = $4)",
    )
    .bind(repository_id)
    .bind(&scope.kind)
    .bind(&scope.name)
    .bind(name)
    .fetch_one(pool)
    .await?;
    if exists {
        Err(ActionsSecretsError::Conflict)
    } else {
        Ok(())
    }
}

async fn ensure_variable_name_available(
    pool: &PgPool,
    repository_id: Uuid,
    scope: &NormalizedSettingScope,
    name: &str,
) -> Result<(), ActionsSecretsError> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM actions_variables WHERE repository_id = $1 AND scope_kind = $2 AND COALESCE(scope_name, '') = COALESCE($3, '') AND name = $4)",
    )
    .bind(repository_id)
    .bind(&scope.kind)
    .bind(&scope.name)
    .bind(name)
    .fetch_one(pool)
    .await?;
    if exists {
        Err(ActionsSecretsError::Conflict)
    } else {
        Ok(())
    }
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

fn decrypt_secret_value(ciphertext: String, nonce: String) -> Result<String, ActionsSecretsError> {
    let encrypted = STANDARD_NO_PAD
        .decode(ciphertext.as_bytes())
        .map_err(|_| ActionsSecretsError::Invalid("secret envelope is invalid".to_owned()))?;
    let key = std::env::var("ACTIONS_SECRETS_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "opengithub-local-actions-secrets-key".to_owned());
    let mut plaintext = Vec::with_capacity(encrypted.len());
    let mut offset = 0usize;
    let mut counter = 0u64;
    while offset < encrypted.len() {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hasher.update(nonce.as_bytes());
        hasher.update(counter.to_le_bytes());
        let block = hasher.finalize();
        for byte in block {
            if offset == encrypted.len() {
                break;
            }
            plaintext.push(encrypted[offset] ^ byte);
            offset += 1;
        }
        counter += 1;
    }
    String::from_utf8(plaintext)
        .map_err(|_| ActionsSecretsError::Invalid("secret envelope is invalid".to_owned()))
}

fn runtime_scope_label(kind: &str, name: Option<&str>) -> String {
    name.filter(|value| !value.trim().is_empty())
        .map(|value| format!("{kind}:{value}"))
        .unwrap_or_else(|| kind.to_owned())
}

fn environment_scope_blocked(
    scope_kind: &str,
    scope_name: Option<&str>,
    request: &ActionsRuntimeResolutionRequest,
    protected_environments: &BTreeMap<String, bool>,
) -> bool {
    if scope_kind != "environment" {
        return false;
    }
    if request.environment.as_deref() != scope_name {
        return true;
    }
    let protection_enabled = scope_name
        .map(|name| name.to_ascii_lowercase())
        .and_then(|name| protected_environments.get(&name).copied())
        .unwrap_or(false);
    protection_enabled && !request.environment_approved
}

fn push_unique_reason(reasons: &mut Vec<String>, reason: &str) {
    if !reasons.iter().any(|existing| existing == reason) {
        reasons.push(reason.to_owned());
    }
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
