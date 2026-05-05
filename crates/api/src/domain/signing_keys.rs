use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::tokens::{sudo_state, PersonalAccessTokenError, SudoState};

#[derive(Debug, thiserror::Error)]
pub enum SigningKeyError {
    #[error("sudo mode is required")]
    SudoRequired,
    #[error("key request is invalid: {0}")]
    Validation(String),
    #[error("key was not found")]
    NotFound,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeySettings {
    #[serde(rename = "sshKeys")]
    pub ssh_keys: Vec<SshKeySummary>,
    #[serde(rename = "gpgKeys")]
    pub gpg_keys: Vec<GpgKeySummary>,
    #[serde(rename = "vigilantMode")]
    pub vigilant_mode: bool,
    #[serde(rename = "sudo")]
    pub sudo: SudoState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshKeySummary {
    pub id: Uuid,
    pub title: String,
    #[serde(rename = "keyType")]
    pub key_type: String,
    #[serde(rename = "fingerprintSha256")]
    pub fingerprint_sha256: String,
    #[serde(rename = "accessMode")]
    pub access_mode: String,
    pub source: String,
    #[serde(rename = "lastUsedAt")]
    pub last_used_at: Option<DateTime<Utc>>,
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<DateTime<Utc>>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GpgKeySummary {
    pub id: Uuid,
    pub title: String,
    #[serde(rename = "primaryFingerprint")]
    pub primary_fingerprint: String,
    #[serde(rename = "keyId")]
    pub key_id: Option<String>,
    pub emails: Vec<String>,
    pub source: String,
    #[serde(rename = "lastUsedAt")]
    pub last_used_at: Option<DateTime<Utc>>,
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<DateTime<Utc>>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSshKeyRequest {
    pub title: String,
    #[serde(rename = "keyType")]
    pub key_type: Option<String>,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(rename = "accessMode")]
    pub access_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateSshKeyResponse {
    #[serde(rename = "sshKey")]
    pub ssh_key: SshKeySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RevokeSshKeyResponse {
    #[serde(rename = "sshKey")]
    pub ssh_key: SshKeySummary,
    #[serde(rename = "revokedAt")]
    pub revoked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateGpgKeyRequest {
    pub title: String,
    #[serde(rename = "armoredPublicKey")]
    pub armored_public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateGpgKeyResponse {
    #[serde(rename = "gpgKey")]
    pub gpg_key: GpgKeySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RevokeGpgKeyResponse {
    #[serde(rename = "gpgKey")]
    pub gpg_key: GpgKeySummary,
    #[serde(rename = "revokedAt")]
    pub revoked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateVigilantModeRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateVigilantModeResponse {
    #[serde(rename = "vigilantMode")]
    pub vigilant_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshKeyAuthPrincipal {
    #[serde(rename = "userId")]
    pub user_id: Uuid,
    #[serde(rename = "keyId")]
    pub key_id: Uuid,
    #[serde(rename = "accessMode")]
    pub access_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignatureVerificationState {
    Verified,
    Unverified,
    VigilantUnverified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignaturePresentation {
    pub verified: bool,
    #[serde(rename = "signatureState")]
    pub signature_state: SignatureVerificationState,
    #[serde(rename = "signatureSummary")]
    pub signature_summary: Option<String>,
}

pub async fn key_settings(
    pool: &PgPool,
    user_id: Uuid,
    session_id: Option<&str>,
) -> Result<KeySettings, SigningKeyError> {
    let vigilant_mode: bool = sqlx::query_scalar("SELECT vigilant_mode FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(KeySettings {
        ssh_keys: ssh_key_summaries(pool, user_id).await?,
        gpg_keys: gpg_key_summaries(pool, user_id).await?,
        vigilant_mode,
        sudo: signing_sudo_state(pool, user_id, session_id).await?,
    })
}

pub async fn create_ssh_key(
    pool: &PgPool,
    user_id: Uuid,
    request: CreateSshKeyRequest,
) -> Result<CreateSshKeyResponse, SigningKeyError> {
    let title = non_blank_bounded(&request.title, 80, "title")?;
    let parsed = parse_ssh_public_key(&request.public_key, request.key_type.as_deref())?;
    let access_mode = normalize_access_mode(request.access_mode.as_deref())?;

    let key_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO ssh_keys (
            user_id, title, key_type, public_key, fingerprint_sha256, access_mode
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(&title)
    .bind(&parsed.key_type)
    .bind(&parsed.normalized_public_key)
    .bind(&parsed.fingerprint_sha256)
    .bind(&access_mode)
    .fetch_one(pool)
    .await
    .map_err(map_unique_validation(
        "An active SSH key with this fingerprint already exists",
    ))?;

    insert_security_audit(
        pool,
        user_id,
        "signing_key.ssh.create",
        "ssh_key",
        key_id,
        json!({
            "title": title,
            "keyType": parsed.key_type,
            "fingerprintSha256": parsed.fingerprint_sha256,
            "accessMode": access_mode,
        }),
    )
    .await?;

    Ok(CreateSshKeyResponse {
        ssh_key: ssh_key_summary_by_id(pool, user_id, key_id).await?,
    })
}

pub async fn revoke_ssh_key(
    pool: &PgPool,
    user_id: Uuid,
    session_id: &str,
    key_id: Uuid,
) -> Result<RevokeSshKeyResponse, SigningKeyError> {
    require_sudo(pool, user_id, session_id).await?;
    let revoked_at = sqlx::query_scalar::<_, DateTime<Utc>>(
        r#"
        UPDATE ssh_keys
        SET revoked_at = now(), revoked_reason = 'user_revoked'
        WHERE id = $1
          AND user_id = $2
          AND revoked_at IS NULL
        RETURNING revoked_at
        "#,
    )
    .bind(key_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(SigningKeyError::NotFound)?;

    insert_security_audit(
        pool,
        user_id,
        "signing_key.ssh.revoke",
        "ssh_key",
        key_id,
        json!({ "revokedAt": revoked_at, "reason": "user_revoked" }),
    )
    .await?;

    Ok(RevokeSshKeyResponse {
        ssh_key: ssh_key_summary_by_id(pool, user_id, key_id).await?,
        revoked_at,
    })
}

pub async fn create_gpg_key(
    pool: &PgPool,
    user_id: Uuid,
    request: CreateGpgKeyRequest,
) -> Result<CreateGpgKeyResponse, SigningKeyError> {
    let title = non_blank_bounded(&request.title, 80, "title")?;
    let parsed = parse_gpg_public_key(&request.armored_public_key)?;
    let mut tx = pool.begin().await?;
    let key_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO gpg_keys (
            user_id, title, armored_public_key, primary_fingerprint, key_id
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(&title)
    .bind(&parsed.normalized_armored_key)
    .bind(&parsed.primary_fingerprint)
    .bind(&parsed.key_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_unique_validation(
        "An active GPG key with this fingerprint already exists",
    ))?;

    for email in &parsed.emails {
        sqlx::query("INSERT INTO gpg_key_emails (gpg_key_id, email) VALUES ($1, $2)")
            .bind(key_id)
            .bind(email)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO security_audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'signing_key.gpg.create', 'gpg_key', $2, $3)
        "#,
    )
    .bind(user_id)
    .bind(key_id)
    .bind(json!({
        "title": title,
        "primaryFingerprint": parsed.primary_fingerprint,
        "keyId": parsed.key_id,
        "emails": parsed.emails,
    }))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(CreateGpgKeyResponse {
        gpg_key: gpg_key_summary_by_id(pool, user_id, key_id).await?,
    })
}

pub async fn revoke_gpg_key(
    pool: &PgPool,
    user_id: Uuid,
    session_id: &str,
    key_id: Uuid,
) -> Result<RevokeGpgKeyResponse, SigningKeyError> {
    require_sudo(pool, user_id, session_id).await?;
    let revoked_at = sqlx::query_scalar::<_, DateTime<Utc>>(
        r#"
        UPDATE gpg_keys
        SET revoked_at = now(), revoked_reason = 'user_revoked'
        WHERE id = $1
          AND user_id = $2
          AND revoked_at IS NULL
        RETURNING revoked_at
        "#,
    )
    .bind(key_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(SigningKeyError::NotFound)?;

    insert_security_audit(
        pool,
        user_id,
        "signing_key.gpg.revoke",
        "gpg_key",
        key_id,
        json!({ "revokedAt": revoked_at, "reason": "user_revoked" }),
    )
    .await?;

    Ok(RevokeGpgKeyResponse {
        gpg_key: gpg_key_summary_by_id(pool, user_id, key_id).await?,
        revoked_at,
    })
}

pub async fn update_vigilant_mode(
    pool: &PgPool,
    user_id: Uuid,
    request: UpdateVigilantModeRequest,
) -> Result<UpdateVigilantModeResponse, SigningKeyError> {
    let previous: bool = sqlx::query_scalar("SELECT vigilant_mode FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    let vigilant_mode: bool = sqlx::query_scalar(
        "UPDATE users SET vigilant_mode = $2 WHERE id = $1 RETURNING vigilant_mode",
    )
    .bind(user_id)
    .bind(request.enabled)
    .fetch_one(pool)
    .await?;

    if previous != vigilant_mode {
        insert_security_audit(
            pool,
            user_id,
            "vigilant_mode.update",
            "user",
            user_id,
            json!({ "previous": previous, "enabled": vigilant_mode }),
        )
        .await?;
    }

    Ok(UpdateVigilantModeResponse { vigilant_mode })
}

pub async fn lookup_active_ssh_key_by_fingerprint(
    pool: &PgPool,
    fingerprint_sha256: &str,
) -> Result<Option<SshKeyAuthPrincipal>, SigningKeyError> {
    let fingerprint = normalize_fingerprint(fingerprint_sha256);
    if fingerprint.is_empty() {
        return Ok(None);
    }
    let row = sqlx::query(
        r#"
        SELECT user_id, id, access_mode
        FROM ssh_keys
        WHERE lower(fingerprint_sha256) = lower($1)
          AND revoked_at IS NULL
        ORDER BY created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(fingerprint)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| SshKeyAuthPrincipal {
        user_id: row.get("user_id"),
        key_id: row.get("id"),
        access_mode: row.get("access_mode"),
    }))
}

pub async fn mark_ssh_key_used(pool: &PgPool, key_id: Uuid) -> Result<(), SigningKeyError> {
    sqlx::query(
        r#"
        UPDATE ssh_keys
        SET last_used_at = now()
        WHERE id = $1 AND revoked_at IS NULL
        "#,
    )
    .bind(key_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn signature_presentation_for_user(
    pool: &PgPool,
    author_user_id: Option<Uuid>,
    signature_fingerprint: Option<&str>,
    stored_summary: Option<&str>,
) -> Result<SignaturePresentation, SigningKeyError> {
    let Some(author_user_id) = author_user_id else {
        return Ok(SignaturePresentation {
            verified: false,
            signature_state: SignatureVerificationState::Unverified,
            signature_summary: stored_summary.map(str::to_owned),
        });
    };
    let vigilant_mode: bool = sqlx::query_scalar("SELECT vigilant_mode FROM users WHERE id = $1")
        .bind(author_user_id)
        .fetch_optional(pool)
        .await?
        .unwrap_or(false);

    if let Some(fingerprint) = signature_fingerprint
        .map(normalize_fingerprint)
        .filter(|value| !value.is_empty())
    {
        let matched = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM gpg_keys
                WHERE user_id = $1
                  AND lower(primary_fingerprint) = lower($2)
                  AND revoked_at IS NULL
            )
            "#,
        )
        .bind(author_user_id)
        .bind(&fingerprint)
        .fetch_one(pool)
        .await?;
        if matched {
            return Ok(SignaturePresentation {
                verified: true,
                signature_state: SignatureVerificationState::Verified,
                signature_summary: Some(
                    stored_summary
                        .filter(|summary| !summary.trim().is_empty())
                        .unwrap_or("Verified signature from an active GPG key.")
                        .to_owned(),
                ),
            });
        }
    }

    if vigilant_mode {
        return Ok(SignaturePresentation {
            verified: false,
            signature_state: SignatureVerificationState::VigilantUnverified,
            signature_summary: Some(
                "Unsigned or untrusted commit by a vigilant-mode author.".to_owned(),
            ),
        });
    }

    Ok(SignaturePresentation {
        verified: false,
        signature_state: SignatureVerificationState::Unverified,
        signature_summary: stored_summary.map(str::to_owned),
    })
}

async fn require_sudo(
    pool: &PgPool,
    user_id: Uuid,
    session_id: &str,
) -> Result<(), SigningKeyError> {
    if signing_sudo_state(pool, user_id, Some(session_id))
        .await?
        .active
    {
        Ok(())
    } else {
        Err(SigningKeyError::SudoRequired)
    }
}

async fn signing_sudo_state(
    pool: &PgPool,
    user_id: Uuid,
    session_id: Option<&str>,
) -> Result<SudoState, SigningKeyError> {
    let mut state = sudo_state(pool, user_id, session_id)
        .await
        .map_err(map_token_sudo_error)?;
    for required in [
        "revoke_ssh_key",
        "revoke_gpg_key",
        "delete_ssh_key",
        "delete_gpg_key",
    ] {
        if !state
            .required_for
            .iter()
            .any(|existing| existing == required)
        {
            state.required_for.push(required.to_owned());
        }
    }
    Ok(state)
}

async fn ssh_key_summaries(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<SshKeySummary>, SigningKeyError> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, key_type, fingerprint_sha256, access_mode, source,
            last_used_at, revoked_at, created_at
        FROM ssh_keys
        WHERE user_id = $1
        ORDER BY revoked_at NULLS FIRST, created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(ssh_key_from_row).collect())
}

async fn ssh_key_summary_by_id(
    pool: &PgPool,
    user_id: Uuid,
    key_id: Uuid,
) -> Result<SshKeySummary, SigningKeyError> {
    sqlx::query(
        r#"
        SELECT id, title, key_type, fingerprint_sha256, access_mode, source,
            last_used_at, revoked_at, created_at
        FROM ssh_keys
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(key_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .map(ssh_key_from_row)
    .ok_or(SigningKeyError::NotFound)
}

async fn gpg_key_summaries(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<GpgKeySummary>, SigningKeyError> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, primary_fingerprint, key_id, source,
            last_used_at, revoked_at, created_at
        FROM gpg_keys
        WHERE user_id = $1
        ORDER BY revoked_at NULLS FIRST, created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    let mut keys = Vec::with_capacity(rows.len());
    for row in rows {
        keys.push(gpg_key_from_row(pool, row).await?);
    }
    Ok(keys)
}

async fn gpg_key_summary_by_id(
    pool: &PgPool,
    user_id: Uuid,
    key_id: Uuid,
) -> Result<GpgKeySummary, SigningKeyError> {
    let row = sqlx::query(
        r#"
        SELECT id, title, primary_fingerprint, key_id, source,
            last_used_at, revoked_at, created_at
        FROM gpg_keys
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(key_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(SigningKeyError::NotFound)?;
    gpg_key_from_row(pool, row).await
}

fn ssh_key_from_row(row: sqlx::postgres::PgRow) -> SshKeySummary {
    SshKeySummary {
        id: row.get("id"),
        title: row.get("title"),
        key_type: row.get("key_type"),
        fingerprint_sha256: row.get("fingerprint_sha256"),
        access_mode: row.get("access_mode"),
        source: row.get("source"),
        last_used_at: row.get("last_used_at"),
        revoked_at: row.get("revoked_at"),
        created_at: row.get("created_at"),
    }
}

async fn gpg_key_from_row(
    pool: &PgPool,
    row: sqlx::postgres::PgRow,
) -> Result<GpgKeySummary, SigningKeyError> {
    let key_id_value: Uuid = row.get("id");
    let emails = sqlx::query_scalar::<_, String>(
        "SELECT email FROM gpg_key_emails WHERE gpg_key_id = $1 ORDER BY lower(email)",
    )
    .bind(key_id_value)
    .fetch_all(pool)
    .await?;
    Ok(GpgKeySummary {
        id: key_id_value,
        title: row.get("title"),
        primary_fingerprint: row.get("primary_fingerprint"),
        key_id: row.get("key_id"),
        emails,
        source: row.get("source"),
        last_used_at: row.get("last_used_at"),
        revoked_at: row.get("revoked_at"),
        created_at: row.get("created_at"),
    })
}

async fn insert_security_audit(
    pool: &PgPool,
    actor_user_id: Uuid,
    event_type: &str,
    target_type: &str,
    target_id: Uuid,
    metadata: serde_json::Value,
) -> Result<(), SigningKeyError> {
    sqlx::query(
        r#"
        INSERT INTO security_audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(actor_user_id)
    .bind(event_type)
    .bind(target_type)
    .bind(target_id)
    .bind(metadata)
    .execute(pool)
    .await?;
    Ok(())
}

fn parse_ssh_public_key(
    public_key: &str,
    requested_key_type: Option<&str>,
) -> Result<ParsedSshKey, SigningKeyError> {
    let public_key = public_key.trim();
    let mut parts = public_key.split_whitespace();
    let key_type = parts
        .next()
        .ok_or_else(|| SigningKeyError::Validation("SSH public key is required".to_owned()))?;
    let encoded = parts
        .next()
        .ok_or_else(|| SigningKeyError::Validation("SSH public key body is required".to_owned()))?;
    if parts.next().is_none() && public_key.split_whitespace().count() < 2 {
        return Err(SigningKeyError::Validation(
            "SSH public key must include a key type and body".to_owned(),
        ));
    }
    if let Some(requested_key_type) = requested_key_type {
        let requested_key_type = requested_key_type.trim();
        if !requested_key_type.is_empty() && requested_key_type != key_type {
            return Err(SigningKeyError::Validation(
                "SSH key type does not match the public key".to_owned(),
            ));
        }
    }
    if !allowed_ssh_key_type(key_type) {
        return Err(SigningKeyError::Validation(format!(
            "SSH key type {key_type} is not supported"
        )));
    }
    let blob = STANDARD.decode(encoded.as_bytes()).map_err(|_| {
        SigningKeyError::Validation("SSH public key body is not valid base64".to_owned())
    })?;
    let embedded_type = ssh_wire_key_type(&blob)?;
    if embedded_type != key_type {
        return Err(SigningKeyError::Validation(
            "SSH public key body does not match the declared key type".to_owned(),
        ));
    }
    let fingerprint_sha256 = format!("SHA256:{}", base64_no_pad(&Sha256::digest(&blob)));
    Ok(ParsedSshKey {
        key_type: key_type.to_owned(),
        normalized_public_key: format!("{key_type} {encoded}"),
        fingerprint_sha256,
    })
}

fn parse_gpg_public_key(armored_public_key: &str) -> Result<ParsedGpgKey, SigningKeyError> {
    let normalized = armored_public_key.trim().replace("\r\n", "\n");
    if !normalized.starts_with("-----BEGIN PGP PUBLIC KEY BLOCK-----")
        || !normalized.contains("-----END PGP PUBLIC KEY BLOCK-----")
    {
        return Err(SigningKeyError::Validation(
            "GPG key must be an armored PGP public key block".to_owned(),
        ));
    }
    let mut in_body = false;
    let mut body = String::new();
    for line in normalized.lines() {
        let line = line.trim();
        if line == "-----BEGIN PGP PUBLIC KEY BLOCK-----" {
            in_body = true;
            continue;
        }
        if line == "-----END PGP PUBLIC KEY BLOCK-----" {
            break;
        }
        if !in_body || line.is_empty() || line.contains(':') {
            continue;
        }
        body.push_str(line);
    }
    let decoded = STANDARD.decode(body.as_bytes()).map_err(|_| {
        SigningKeyError::Validation("GPG public key armor is not valid base64".to_owned())
    })?;
    if decoded.len() < 16 {
        return Err(SigningKeyError::Validation(
            "GPG public key payload is too short".to_owned(),
        ));
    }
    let digest = Sha256::digest(&decoded);
    let primary_fingerprint = hex_upper(&digest);
    let key_id = primary_fingerprint
        .get(primary_fingerprint.len().saturating_sub(16)..)
        .map(ToOwned::to_owned);
    let emails = extract_emails(&normalized)?;
    Ok(ParsedGpgKey {
        normalized_armored_key: normalized,
        primary_fingerprint,
        key_id,
        emails,
    })
}

fn ssh_wire_key_type(blob: &[u8]) -> Result<&str, SigningKeyError> {
    if blob.len() < 4 {
        return Err(SigningKeyError::Validation(
            "SSH public key body is too short".to_owned(),
        ));
    }
    let len = u32::from_be_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
    let end = 4usize
        .checked_add(len)
        .ok_or_else(|| SigningKeyError::Validation("SSH public key body is invalid".to_owned()))?;
    if end > blob.len() {
        return Err(SigningKeyError::Validation(
            "SSH public key body is truncated".to_owned(),
        ));
    }
    std::str::from_utf8(&blob[4..end])
        .map_err(|_| SigningKeyError::Validation("SSH key type is not UTF-8".to_owned()))
}

fn allowed_ssh_key_type(key_type: &str) -> bool {
    matches!(
        key_type,
        "ssh-ed25519"
            | "ssh-rsa"
            | "ecdsa-sha2-nistp256"
            | "ecdsa-sha2-nistp384"
            | "ecdsa-sha2-nistp521"
            | "sk-ssh-ed25519@openssh.com"
            | "sk-ecdsa-sha2-nistp256@openssh.com"
    )
}

fn normalize_access_mode(access_mode: Option<&str>) -> Result<String, SigningKeyError> {
    match access_mode.unwrap_or("read_write").trim() {
        "" | "read_write" => Ok("read_write".to_owned()),
        "read_only" => Ok("read_only".to_owned()),
        _ => Err(SigningKeyError::Validation(
            "accessMode must be read_write or read_only".to_owned(),
        )),
    }
}

fn non_blank_bounded(value: &str, max_len: usize, field: &str) -> Result<String, SigningKeyError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(SigningKeyError::Validation(format!("{field} is required")));
    }
    if trimmed.chars().count() > max_len {
        return Err(SigningKeyError::Validation(format!(
            "{field} must be at most {max_len} characters"
        )));
    }
    Ok(trimmed.to_owned())
}

fn extract_emails(value: &str) -> Result<Vec<String>, SigningKeyError> {
    let email_regex = Regex::new(r"(?i)[A-Z0-9._%+\-]+@[A-Z0-9.\-]+\.[A-Z]{2,}")
        .map_err(|_| SigningKeyError::Validation("email parser is unavailable".to_owned()))?;
    let mut emails = Vec::new();
    for capture in email_regex.find_iter(value) {
        let email = capture.as_str().to_ascii_lowercase();
        if !emails.iter().any(|existing| existing == &email) {
            emails.push(email);
        }
    }
    Ok(emails)
}

fn base64_no_pad(bytes: &[u8]) -> String {
    STANDARD.encode(bytes).trim_end_matches('=').to_owned()
}

fn hex_upper(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}

fn normalize_fingerprint(value: &str) -> String {
    value
        .trim()
        .strip_prefix("SHA256:")
        .map(|stripped| format!("SHA256:{}", stripped.trim()))
        .unwrap_or_else(|| value.trim().replace([' ', ':'], "").to_ascii_uppercase())
}

fn map_unique_validation(
    message: &'static str,
) -> impl Fn(sqlx::Error) -> SigningKeyError + Copy + 'static {
    move |error| match &error {
        sqlx::Error::Database(database_error) if database_error.is_unique_violation() => {
            SigningKeyError::Validation(message.to_owned())
        }
        _ => SigningKeyError::Sqlx(error),
    }
}

fn map_token_sudo_error(error: PersonalAccessTokenError) -> SigningKeyError {
    match error {
        PersonalAccessTokenError::Sqlx(error) => SigningKeyError::Sqlx(error),
        PersonalAccessTokenError::SudoRequired => SigningKeyError::SudoRequired,
        PersonalAccessTokenError::Validation(message) => SigningKeyError::Validation(message),
        PersonalAccessTokenError::Forbidden
        | PersonalAccessTokenError::Invalid
        | PersonalAccessTokenError::InvalidSudoConfirmation => SigningKeyError::SudoRequired,
    }
}

struct ParsedSshKey {
    key_type: String,
    normalized_public_key: String,
    fingerprint_sha256: String,
}

struct ParsedGpgKey {
    normalized_armored_key: String,
    primary_fingerprint: String,
    key_id: Option<String>,
    emails: Vec<String>,
}
