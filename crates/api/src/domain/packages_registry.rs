use axum::http::HeaderMap;
use base64::Engine as _;
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, QueryBuilder, Row};
use std::path::{Path, PathBuf};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};
use uuid::Uuid;

use crate::domain::tokens::{verify_personal_access_token, PersonalAccessTokenError};

const OCI_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
const DOCKER_MANIFEST: &str = "application/vnd.docker.distribution.manifest.v2+json";
const DOCKER_MANIFEST_LIST: &str = "application/vnd.docker.distribution.manifest.list.v2+json";
const OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegistryManifestRead {
    pub package_id: Uuid,
    pub package_version_id: Uuid,
    pub package_name: String,
    pub namespace: String,
    pub reference: String,
    pub digest: Option<String>,
    pub media_type: String,
    pub manifest: Value,
    pub manifest_size_bytes: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryAuth {
    Anonymous,
    Token {
        user_id: Uuid,
        token_id: Uuid,
        can_write_packages: bool,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegistryToken {
    pub token: String,
    pub access_token: String,
    pub expires_in: i64,
    pub issued_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegistryUploadStart {
    pub upload_id: Uuid,
    pub location: String,
    pub range: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegistryUploadProgress {
    pub upload_id: Uuid,
    pub location: String,
    pub range: String,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegistryBlobRead {
    pub package_id: Uuid,
    pub package_version_id: Option<Uuid>,
    pub digest: String,
    pub media_type: String,
    pub size_bytes: i64,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegistryTagList {
    pub name: String,
    pub tags: Vec<String>,
}

pub struct RegistryManifestReadRequest<'a> {
    pub namespace: &'a str,
    pub image: &'a str,
    pub reference: &'a str,
    pub accept: Option<&'a str>,
    pub auth: &'a RegistryAuth,
    pub user_agent: Option<&'a str>,
    pub record_transfer: bool,
}

pub struct RegistryManifestPutRequest<'a> {
    pub namespace: &'a str,
    pub image: &'a str,
    pub reference: &'a str,
    pub manifest: Value,
    pub content_type: Option<&'a str>,
    pub auth: &'a RegistryAuth,
    pub user_agent: Option<&'a str>,
}

struct RegistryAuditEvent<'a> {
    package_id: Uuid,
    package_version_id: Option<Uuid>,
    actor_user_id: Option<Uuid>,
    event_type: &'a str,
    reference: Option<&'a str>,
    digest: Option<&'a str>,
    user_agent: Option<&'a str>,
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("registry credentials are required")]
    Unauthorized,
    #[error("registry token is invalid")]
    InvalidToken,
    #[error("package token is missing packages:read scope")]
    InsufficientScope,
    #[error("package token is missing packages:write scope")]
    InsufficientWriteScope,
    #[error("manifest was not found")]
    NotFound,
    #[error("blob was not found")]
    BlobNotFound,
    #[error("upload session was not found")]
    UploadNotFound,
    #[error("upload digest does not match content")]
    DigestMismatch,
    #[error("{0}")]
    InvalidReference(String),
    #[error("{0}")]
    InvalidManifest(String),
    #[error("requested manifest media type is not acceptable")]
    NotAcceptable,
    #[error("storage error")]
    Storage(#[from] std::io::Error),
    #[error("database error")]
    Sqlx(#[from] sqlx::Error),
}

impl RegistryAuth {
    pub fn actor_user_id(&self) -> Option<Uuid> {
        match self {
            RegistryAuth::Anonymous => None,
            RegistryAuth::Token { user_id, .. } => Some(*user_id),
        }
    }

    pub fn can_write_packages(&self) -> bool {
        matches!(
            self,
            RegistryAuth::Token {
                can_write_packages: true,
                ..
            }
        )
    }
}

pub async fn registry_auth_from_headers(
    pool: &PgPool,
    headers: &HeaderMap,
) -> Result<RegistryAuth, RegistryError> {
    let Some(token) = registry_token_from_headers(headers) else {
        return Ok(RegistryAuth::Anonymous);
    };
    let verified =
        verify_personal_access_token(pool, &token)
            .await
            .map_err(|error| match error {
                PersonalAccessTokenError::Invalid => RegistryError::InvalidToken,
                PersonalAccessTokenError::Sqlx(error) => RegistryError::Sqlx(error),
            })?;
    if !verified.allows_package_read() {
        return Err(RegistryError::InsufficientScope);
    }
    Ok(RegistryAuth::Token {
        user_id: verified.user_id,
        token_id: verified.id,
        can_write_packages: verified.allows_package_write(),
    })
}

pub async fn exchange_registry_token(
    pool: &PgPool,
    headers: &HeaderMap,
) -> Result<RegistryToken, RegistryError> {
    let Some(token) = registry_token_from_headers(headers) else {
        return Err(RegistryError::Unauthorized);
    };
    let verified =
        verify_personal_access_token(pool, &token)
            .await
            .map_err(|error| match error {
                PersonalAccessTokenError::Invalid => RegistryError::InvalidToken,
                PersonalAccessTokenError::Sqlx(error) => RegistryError::Sqlx(error),
            })?;
    if !verified.allows_package_read() {
        return Err(RegistryError::InsufficientScope);
    }
    Ok(RegistryToken {
        token: token.clone(),
        access_token: token,
        expires_in: 900,
        issued_at: chrono::Utc::now().to_rfc3339(),
    })
}

pub async fn start_blob_upload(
    pool: &PgPool,
    namespace: &str,
    image: &str,
    auth: &RegistryAuth,
) -> Result<RegistryUploadStart, RegistryError> {
    let package = require_package_write(pool, namespace, image, auth).await?;
    let upload_id = Uuid::new_v4();
    let storage_key = upload_storage_key(package.id, upload_id);
    let path = registry_storage_path(&storage_key)?;
    ensure_parent_dir(&path).await?;
    fs::write(&path, []).await?;
    sqlx::query(
        r#"
        INSERT INTO package_registry_uploads (
            id, package_id, actor_user_id, storage_kind, storage_key, status, expires_at
        )
        VALUES ($1, $2, $3, 'local', $4, 'active', $5)
        "#,
    )
    .bind(upload_id)
    .bind(package.id)
    .bind(auth.actor_user_id())
    .bind(&storage_key)
    .bind(Utc::now() + Duration::hours(1))
    .execute(pool)
    .await?;
    audit_registry_event(
        pool,
        RegistryAuditEvent {
            package_id: package.id,
            package_version_id: None,
            actor_user_id: auth.actor_user_id(),
            event_type: "blob.upload.start",
            reference: None,
            digest: None,
            user_agent: None,
        },
    )
    .await?;

    Ok(RegistryUploadStart {
        upload_id,
        location: upload_location(namespace, image, upload_id),
        range: "0-0".to_owned(),
    })
}

pub async fn append_blob_upload(
    pool: &PgPool,
    namespace: &str,
    image: &str,
    upload_id: Uuid,
    chunk: &[u8],
    auth: &RegistryAuth,
) -> Result<RegistryUploadProgress, RegistryError> {
    let upload = active_upload(pool, namespace, image, upload_id, auth).await?;
    let path = registry_storage_path(&upload.storage_key)?;
    ensure_parent_dir(&path).await?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await?;
    file.write_all(chunk).await?;
    file.flush().await?;
    let size = fs::metadata(&path).await?.len() as i64;
    sqlx::query("UPDATE package_registry_uploads SET size_bytes = $1 WHERE id = $2")
        .bind(size)
        .bind(upload_id)
        .execute(pool)
        .await?;

    Ok(RegistryUploadProgress {
        upload_id,
        location: upload_location(namespace, image, upload_id),
        range: registry_range(size),
        size_bytes: size,
    })
}

pub async fn complete_blob_upload(
    pool: &PgPool,
    namespace: &str,
    image: &str,
    upload_id: Uuid,
    digest: &str,
    final_chunk: &[u8],
    auth: &RegistryAuth,
) -> Result<RegistryUploadProgress, RegistryError> {
    let digest = validate_digest(digest)?;
    if !final_chunk.is_empty() {
        append_blob_upload(pool, namespace, image, upload_id, final_chunk, auth).await?;
    }
    let upload = active_upload(pool, namespace, image, upload_id, auth).await?;
    let path = registry_storage_path(&upload.storage_key)?;
    let bytes = fs::read(&path).await?;
    let actual_digest = sha256_digest(&bytes);
    if actual_digest != digest {
        return Err(RegistryError::DigestMismatch);
    }
    let storage_key = blob_storage_key(upload.package_id, &digest);
    let final_path = registry_storage_path(&storage_key)?;
    ensure_parent_dir(&final_path).await?;
    if fs::rename(&path, &final_path).await.is_err() {
        fs::copy(&path, &final_path).await?;
        fs::remove_file(&path).await?;
    }
    let size = bytes.len() as i64;
    sqlx::query(
        r#"
        UPDATE package_registry_uploads
        SET expected_digest = $1, storage_key = $2, size_bytes = $3, status = 'completed', completed_at = now()
        WHERE id = $4
        "#,
    )
    .bind(&digest)
    .bind(&storage_key)
    .bind(size)
    .bind(upload_id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO package_blobs (
            package_id, digest, media_type, size_bytes, byte_size, storage_kind, storage_key
        )
        VALUES ($1, $2, 'application/octet-stream', $3, $3, 'local', $4)
        ON CONFLICT (package_id, lower(digest)) DO UPDATE
        SET size_bytes = EXCLUDED.size_bytes,
            byte_size = EXCLUDED.byte_size,
            storage_kind = EXCLUDED.storage_kind,
            storage_key = EXCLUDED.storage_key
        "#,
    )
    .bind(upload.package_id)
    .bind(&digest)
    .bind(size)
    .bind(&storage_key)
    .execute(pool)
    .await?;
    audit_registry_event(
        pool,
        RegistryAuditEvent {
            package_id: upload.package_id,
            package_version_id: None,
            actor_user_id: auth.actor_user_id(),
            event_type: "blob.upload.complete",
            reference: None,
            digest: Some(&digest),
            user_agent: None,
        },
    )
    .await?;

    Ok(RegistryUploadProgress {
        upload_id,
        location: format!("/v2/{namespace}/{image}/blobs/{digest}"),
        range: registry_range(size),
        size_bytes: size,
    })
}

pub async fn cancel_blob_upload(
    pool: &PgPool,
    namespace: &str,
    image: &str,
    upload_id: Uuid,
    auth: &RegistryAuth,
) -> Result<(), RegistryError> {
    let upload = active_upload(pool, namespace, image, upload_id, auth).await?;
    sqlx::query(
        "UPDATE package_registry_uploads SET status = 'cancelled', cancelled_at = now() WHERE id = $1",
    )
    .bind(upload_id)
    .execute(pool)
    .await?;
    let _ = fs::remove_file(registry_storage_path(&upload.storage_key)?).await;
    audit_registry_event(
        pool,
        RegistryAuditEvent {
            package_id: upload.package_id,
            package_version_id: None,
            actor_user_id: auth.actor_user_id(),
            event_type: "blob.upload.cancel",
            reference: None,
            digest: None,
            user_agent: None,
        },
    )
    .await?;
    Ok(())
}

pub async fn put_registry_manifest(
    pool: &PgPool,
    request: RegistryManifestPutRequest<'_>,
) -> Result<RegistryManifestRead, RegistryError> {
    let package =
        require_package_write(pool, request.namespace, request.image, request.auth).await?;
    let reference = validate_reference(request.reference)?;
    if reference.starts_with("sha256:") {
        return Err(RegistryError::InvalidReference(
            "manifest pushes must target a tag, not a digest reference".to_owned(),
        ));
    }
    let media_type = request
        .manifest
        .get("mediaType")
        .and_then(Value::as_str)
        .or(request.content_type)
        .unwrap_or(OCI_MANIFEST)
        .to_owned();
    if !matches_manifest_media_type(&media_type) {
        return Err(RegistryError::InvalidManifest(
            "unsupported manifest media type".to_owned(),
        ));
    }
    validate_manifest_blobs(pool, package.id, &request.manifest).await?;
    let bytes = serde_json::to_vec(&request.manifest).map_err(|_| {
        RegistryError::InvalidManifest("manifest must serialize as JSON".to_owned())
    })?;
    let digest = sha256_digest(&bytes);
    let config_digest = request
        .manifest
        .get("config")
        .and_then(|config| config.get("digest"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let size_bytes = manifest_blob_size(&request.manifest);

    let row = sqlx::query(
        r#"
        INSERT INTO package_versions (
            package_id, version, digest, manifest, manifest_media_type,
            config_digest, manifest_size_bytes, size_bytes, published_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (package_id, lower(version)) DO UPDATE
        SET digest = EXCLUDED.digest,
            manifest = EXCLUDED.manifest,
            manifest_media_type = EXCLUDED.manifest_media_type,
            config_digest = EXCLUDED.config_digest,
            manifest_size_bytes = EXCLUDED.manifest_size_bytes,
            size_bytes = EXCLUDED.size_bytes,
            published_by_user_id = EXCLUDED.published_by_user_id,
            created_at = now()
        RETURNING id
        "#,
    )
    .bind(package.id)
    .bind(&reference)
    .bind(&digest)
    .bind(&request.manifest)
    .bind(&media_type)
    .bind(&config_digest)
    .bind(bytes.len() as i64)
    .bind(size_bytes)
    .bind(request.auth.actor_user_id().expect("write auth has actor"))
    .fetch_one(pool)
    .await?;
    let version_id: Uuid = row.try_get("id")?;
    attach_manifest_blobs(pool, package.id, version_id, &request.manifest).await?;
    audit_registry_event(
        pool,
        RegistryAuditEvent {
            package_id: package.id,
            package_version_id: Some(version_id),
            actor_user_id: request.auth.actor_user_id(),
            event_type: "manifest.write",
            reference: Some(&reference),
            digest: Some(&digest),
            user_agent: request.user_agent,
        },
    )
    .await?;

    Ok(RegistryManifestRead {
        package_id: package.id,
        package_version_id: version_id,
        package_name: package.name,
        namespace: request.namespace.to_owned(),
        reference,
        digest: Some(digest),
        media_type,
        manifest: request.manifest,
        manifest_size_bytes: bytes.len() as i64,
    })
}

pub async fn read_registry_blob(
    pool: &PgPool,
    namespace: &str,
    image: &str,
    digest: &str,
    auth: &RegistryAuth,
    user_agent: Option<&str>,
) -> Result<RegistryBlobRead, RegistryError> {
    let package = require_package_read(pool, namespace, image, auth).await?;
    let digest = validate_digest(digest)?;
    let Some(row) = sqlx::query(
        r#"
        SELECT id, package_version_id, digest, COALESCE(media_type, 'application/octet-stream') AS media_type,
               COALESCE(byte_size, size_bytes, 0)::bigint AS size_bytes, storage_key
        FROM package_blobs
        WHERE package_id = $1 AND lower(digest) = lower($2)
        LIMIT 1
        "#,
    )
    .bind(package.id)
    .bind(&digest)
    .fetch_optional(pool)
    .await? else {
        return Err(RegistryError::BlobNotFound);
    };
    let storage_key: String = row.try_get("storage_key")?;
    let bytes = fs::read(registry_storage_path(&storage_key)?).await?;
    let package_version_id: Option<Uuid> = row.try_get("package_version_id")?;
    record_download(pool, package.id, package_version_id, auth.actor_user_id()).await?;
    audit_registry_event(
        pool,
        RegistryAuditEvent {
            package_id: package.id,
            package_version_id,
            actor_user_id: auth.actor_user_id(),
            event_type: "blob.read",
            reference: None,
            digest: Some(&digest),
            user_agent,
        },
    )
    .await?;

    Ok(RegistryBlobRead {
        package_id: package.id,
        package_version_id,
        digest,
        media_type: row.try_get("media_type")?,
        size_bytes: row.try_get("size_bytes")?,
        bytes,
    })
}

pub async fn list_registry_tags(
    pool: &PgPool,
    namespace: &str,
    image: &str,
    auth: &RegistryAuth,
) -> Result<RegistryTagList, RegistryError> {
    let package = require_package_read(pool, namespace, image, auth).await?;
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT version
        FROM package_versions
        WHERE package_id = $1
        ORDER BY lower(version)
        "#,
    )
    .bind(package.id)
    .fetch_all(pool)
    .await?;
    Ok(RegistryTagList {
        name: format!("{namespace}/{image}"),
        tags: rows,
    })
}

pub async fn read_registry_manifest(
    pool: &PgPool,
    request: RegistryManifestReadRequest<'_>,
) -> Result<RegistryManifestRead, RegistryError> {
    validate_name_component(request.namespace, "namespace")?;
    validate_name_component(request.image, "image")?;
    let reference = validate_reference(request.reference)?;
    let actor_user_id = request.auth.actor_user_id();

    let mut builder = QueryBuilder::<sqlx::Postgres>::new(
        r#"
        SELECT p.id AS package_id,
               p.name AS package_name,
               p.visibility,
               pv.id AS package_version_id,
               pv.version,
               pv.digest,
               pv.manifest,
               COALESCE(pv.manifest_media_type, pv.manifest->>'mediaType', "#,
    );
    builder.push_bind(OCI_MANIFEST);
    builder.push(
        r#") AS manifest_media_type,
               COALESCE(pv.manifest_size_bytes, octet_length(pv.manifest::text)::bigint) AS manifest_size_bytes,
               (p.visibility = 'public') AS public_readable,
               COALESCE((p.owner_user_id = "#,
    );
    builder.push_bind(actor_user_id);
    builder.push(
        r#"), false) AS actor_owns_user_package,
               EXISTS (
                   SELECT 1
                   FROM organization_memberships om
                   WHERE om.organization_id = p.owner_organization_id
                     AND om.user_id = "#,
    );
    builder.push_bind(actor_user_id);
    builder.push(
        r#"
               ) AS actor_is_org_member,
               EXISTS (
                   SELECT 1
                   FROM package_permissions pp
                   WHERE pp.package_id = p.id
                     AND pp.user_id = "#,
    );
    builder.push_bind(actor_user_id);
    builder.push(
        r#"
                     AND pp.role IN ('read', 'write', 'admin')
               ) AS actor_can_read_package,
               EXISTS (
                   SELECT 1
                   FROM repository_permissions rp
                   WHERE rp.user_id = "#,
    );
    builder.push_bind(actor_user_id);
    builder.push(
        r#"
                     AND rp.role IN ('read', 'write', 'admin', 'owner')
                     AND (
                         rp.repository_id = p.repository_id
                         OR EXISTS (
                             SELECT 1
                             FROM package_repository_links pr
                             WHERE pr.package_id = p.id
                               AND pr.repository_id = rp.repository_id
                         )
                     )
               ) AS actor_can_read_linked_repo
        FROM packages p
        JOIN package_versions pv ON pv.package_id = p.id
        LEFT JOIN users owner_user ON owner_user.id = p.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = p.owner_organization_id
        WHERE p.package_type = 'container'
          AND lower(COALESCE(owner_user.username, owner_org.slug)) = lower("#,
    );
    builder.push_bind(request.namespace);
    builder.push(") AND lower(p.name) = lower(");
    builder.push_bind(request.image);
    builder.push(") AND (lower(pv.version) = lower(");
    builder.push_bind(&reference);
    builder.push(") OR lower(pv.digest) = lower(");
    builder.push_bind(&reference);
    builder.push(
        r#"))
        ORDER BY CASE WHEN lower(pv.version) = lower("#,
    );
    builder.push_bind(&reference);
    builder.push(") THEN 0 ELSE 1 END, pv.created_at DESC LIMIT 1");

    let Some(row) = builder.build().fetch_optional(pool).await? else {
        return Err(RegistryError::NotFound);
    };

    let public_readable: bool = row.try_get("public_readable")?;
    let actor_owns_user_package: bool = row.try_get("actor_owns_user_package")?;
    let actor_is_org_member: bool = row.try_get("actor_is_org_member")?;
    let actor_can_read_package: bool = row.try_get("actor_can_read_package")?;
    let actor_can_read_linked_repo: bool = row.try_get("actor_can_read_linked_repo")?;
    let can_read = public_readable
        || actor_owns_user_package
        || actor_is_org_member
        || actor_can_read_package
        || actor_can_read_linked_repo;
    if !can_read {
        return match request.auth {
            RegistryAuth::Anonymous => Err(RegistryError::Unauthorized),
            RegistryAuth::Token { .. } => Err(RegistryError::NotFound),
        };
    }

    let media_type: String = row.try_get("manifest_media_type")?;
    if !accepts_manifest_media_type(request.accept, &media_type) {
        return Err(RegistryError::NotAcceptable);
    }

    let package_id: Uuid = row.try_get("package_id")?;
    let package_version_id: Uuid = row.try_get("package_version_id")?;
    let digest: Option<String> = row.try_get("digest")?;
    audit_registry_event(
        pool,
        RegistryAuditEvent {
            package_id,
            package_version_id: Some(package_version_id),
            actor_user_id,
            event_type: "manifest.read",
            reference: Some(&reference),
            digest: digest.as_deref(),
            user_agent: request.user_agent,
        },
    )
    .await?;
    if request.record_transfer {
        record_download(pool, package_id, Some(package_version_id), actor_user_id).await?;
    }

    Ok(RegistryManifestRead {
        package_id,
        package_version_id,
        package_name: row.try_get("package_name")?,
        namespace: request.namespace.to_owned(),
        reference,
        digest,
        media_type,
        manifest: row.try_get("manifest")?,
        manifest_size_bytes: row.try_get("manifest_size_bytes")?,
    })
}

#[derive(Debug, Clone)]
struct RegistryPackage {
    id: Uuid,
    name: String,
}

#[derive(Debug, Clone)]
struct RegistryUpload {
    package_id: Uuid,
    storage_key: String,
    expires_at: DateTime<Utc>,
}

pub fn registry_challenge(scope: Option<&str>) -> String {
    let scope = scope.unwrap_or("registry:catalog:*");
    format!(
        r#"Bearer realm="http://localhost:3016/v2/token",service="opengithub-registry",scope="{scope}""#
    )
}

pub fn registry_token_from_headers(headers: &HeaderMap) -> Option<String> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())?;
    token_from_authorization(value)
}

fn token_from_authorization(value: &str) -> Option<String> {
    let value = value.trim();
    if let Some(token) = value.strip_prefix("Bearer ") {
        return Some(token.trim().to_owned()).filter(|token| !token.is_empty());
    }
    let encoded = value.strip_prefix("Basic ")?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded.trim())
        .ok()?;
    let decoded = String::from_utf8(decoded).ok()?;
    let (_username, password) = decoded.split_once(':')?;
    Some(password.trim().to_owned()).filter(|token| !token.is_empty())
}

fn accepts_manifest_media_type(accept: Option<&str>, media_type: &str) -> bool {
    let Some(accept) = accept else {
        return true;
    };
    accept.split(',').any(|part| {
        let token = part
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        token == "*/*"
            || token == "application/*"
            || token == media_type.to_ascii_lowercase()
            || matches!(
                token.as_str(),
                OCI_MANIFEST | DOCKER_MANIFEST | DOCKER_MANIFEST_LIST | OCI_INDEX
            )
    })
}

fn matches_manifest_media_type(media_type: &str) -> bool {
    matches!(
        media_type,
        OCI_MANIFEST | DOCKER_MANIFEST | DOCKER_MANIFEST_LIST | OCI_INDEX
    )
}

fn validate_reference(reference: &str) -> Result<String, RegistryError> {
    let reference = reference.trim();
    if reference.is_empty() || reference.len() > 255 {
        return Err(RegistryError::InvalidReference(
            "manifest reference must be 1-255 characters".to_owned(),
        ));
    }
    if reference.starts_with("sha256:") {
        let hex = reference.trim_start_matches("sha256:");
        if hex.len() == 64 && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Ok(reference.to_ascii_lowercase());
        }
        return Err(RegistryError::InvalidReference(
            "sha256 digest references must contain 64 hex characters".to_owned(),
        ));
    }
    if reference
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-'))
        && !reference.starts_with('.')
        && !reference.starts_with('-')
    {
        return Ok(reference.to_owned());
    }
    Err(RegistryError::InvalidReference(
        "tag references may contain only letters, numbers, underscore, period, and dash".to_owned(),
    ))
}

fn validate_digest(digest: &str) -> Result<String, RegistryError> {
    let digest = digest.trim();
    if let Some(hex) = digest.strip_prefix("sha256:") {
        if hex.len() == 64 && hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Ok(digest.to_ascii_lowercase());
        }
    }
    Err(RegistryError::InvalidReference(
        "sha256 digests must contain 64 hex characters".to_owned(),
    ))
}

fn validate_name_component(value: &str, label: &str) -> Result<(), RegistryError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 255
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-'))
    {
        return Err(RegistryError::InvalidReference(format!(
            "{label} may contain only letters, numbers, underscore, period, and dash"
        )));
    }
    Ok(())
}

async fn require_package_read(
    pool: &PgPool,
    namespace: &str,
    image: &str,
    auth: &RegistryAuth,
) -> Result<RegistryPackage, RegistryError> {
    package_for_auth(pool, namespace, image, auth, false).await
}

async fn require_package_write(
    pool: &PgPool,
    namespace: &str,
    image: &str,
    auth: &RegistryAuth,
) -> Result<RegistryPackage, RegistryError> {
    if matches!(auth, RegistryAuth::Anonymous) {
        return Err(RegistryError::Unauthorized);
    }
    if !auth.can_write_packages() {
        return Err(RegistryError::InsufficientWriteScope);
    }
    package_for_auth(pool, namespace, image, auth, true).await
}

async fn package_for_auth(
    pool: &PgPool,
    namespace: &str,
    image: &str,
    auth: &RegistryAuth,
    require_write: bool,
) -> Result<RegistryPackage, RegistryError> {
    validate_name_component(namespace, "namespace")?;
    validate_name_component(image, "image")?;
    let actor_user_id = auth.actor_user_id();
    let Some(row) = sqlx::query(
        r#"
        SELECT p.id, p.name, p.visibility,
               (p.visibility = 'public') AS public_readable,
               COALESCE((p.owner_user_id = $3), false) AS actor_owns_user_package,
               EXISTS (
                   SELECT 1 FROM organization_memberships om
                   WHERE om.organization_id = p.owner_organization_id
                     AND om.user_id = $3
                     AND ($4 = false OR om.role IN ('owner', 'admin'))
               ) AS actor_has_org_access,
               EXISTS (
                   SELECT 1 FROM package_permissions pp
                   WHERE pp.package_id = p.id
                     AND pp.user_id = $3
                     AND (
                         ($4 = false AND pp.role IN ('read', 'write', 'admin'))
                         OR ($4 = true AND pp.role IN ('write', 'admin'))
                     )
               ) AS actor_has_package_access,
               EXISTS (
                   SELECT 1 FROM repository_permissions rp
                   WHERE rp.user_id = $3
                     AND (
                         ($4 = false AND rp.role IN ('read', 'write', 'maintain', 'admin', 'owner'))
                         OR ($4 = true AND rp.role IN ('write', 'maintain', 'admin', 'owner'))
                     )
                     AND (
                         rp.repository_id = p.repository_id
                         OR EXISTS (
                             SELECT 1 FROM package_repository_links pr
                             WHERE pr.package_id = p.id AND pr.repository_id = rp.repository_id
                         )
                     )
               ) AS actor_has_repo_access
        FROM packages p
        LEFT JOIN users owner_user ON owner_user.id = p.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = p.owner_organization_id
        WHERE p.package_type = 'container'
          AND lower(COALESCE(owner_user.username, owner_org.slug)) = lower($1)
          AND lower(p.name) = lower($2)
        LIMIT 1
        "#,
    )
    .bind(namespace)
    .bind(image)
    .bind(actor_user_id)
    .bind(require_write)
    .fetch_optional(pool)
    .await?
    else {
        return Err(RegistryError::NotFound);
    };

    let can_access = if require_write {
        row.try_get("actor_owns_user_package")?
            || row.try_get("actor_has_org_access")?
            || row.try_get("actor_has_package_access")?
            || row.try_get("actor_has_repo_access")?
    } else {
        row.try_get("public_readable")?
            || row.try_get("actor_owns_user_package")?
            || row.try_get("actor_has_org_access")?
            || row.try_get("actor_has_package_access")?
            || row.try_get("actor_has_repo_access")?
    };
    if !can_access {
        return match auth {
            RegistryAuth::Anonymous => Err(RegistryError::Unauthorized),
            RegistryAuth::Token { .. } => Err(RegistryError::NotFound),
        };
    }
    Ok(RegistryPackage {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
    })
}

async fn active_upload(
    pool: &PgPool,
    namespace: &str,
    image: &str,
    upload_id: Uuid,
    auth: &RegistryAuth,
) -> Result<RegistryUpload, RegistryError> {
    let package = require_package_write(pool, namespace, image, auth).await?;
    let Some(row) = sqlx::query(
        r#"
        SELECT package_id, storage_key, expires_at
        FROM package_registry_uploads
        WHERE id = $1 AND package_id = $2 AND status = 'active'
        LIMIT 1
        "#,
    )
    .bind(upload_id)
    .bind(package.id)
    .fetch_optional(pool)
    .await?
    else {
        return Err(RegistryError::UploadNotFound);
    };
    let upload = RegistryUpload {
        package_id: row.try_get("package_id")?,
        storage_key: row.try_get("storage_key")?,
        expires_at: row.try_get("expires_at")?,
    };
    if upload.expires_at <= Utc::now() {
        sqlx::query("UPDATE package_registry_uploads SET status = 'expired' WHERE id = $1")
            .bind(upload_id)
            .execute(pool)
            .await?;
        return Err(RegistryError::UploadNotFound);
    }
    Ok(upload)
}

async fn validate_manifest_blobs(
    pool: &PgPool,
    package_id: Uuid,
    manifest: &Value,
) -> Result<(), RegistryError> {
    let mut digests = Vec::new();
    if let Some(config_digest) = manifest
        .get("config")
        .and_then(|config| config.get("digest"))
        .and_then(Value::as_str)
    {
        digests.push(validate_digest(config_digest)?);
    }
    if let Some(layers) = manifest.get("layers").and_then(Value::as_array) {
        for layer in layers {
            let digest = layer.get("digest").and_then(Value::as_str).ok_or_else(|| {
                RegistryError::InvalidManifest("manifest layers require digests".to_owned())
            })?;
            digests.push(validate_digest(digest)?);
        }
    }
    for digest in digests {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM package_blobs WHERE package_id = $1 AND lower(digest) = lower($2))",
        )
        .bind(package_id)
        .bind(&digest)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(RegistryError::InvalidManifest(format!(
                "manifest references unknown blob {digest}"
            )));
        }
    }
    Ok(())
}

async fn attach_manifest_blobs(
    pool: &PgPool,
    package_id: Uuid,
    package_version_id: Uuid,
    manifest: &Value,
) -> Result<(), RegistryError> {
    if let Some(config) = manifest.get("config") {
        if let Some(digest) = config.get("digest").and_then(Value::as_str) {
            let media_type = config.get("mediaType").and_then(Value::as_str);
            let size = config.get("size").and_then(Value::as_i64);
            update_blob_version(
                pool,
                package_id,
                package_version_id,
                digest,
                media_type,
                size,
            )
            .await?;
        }
    }
    if let Some(layers) = manifest.get("layers").and_then(Value::as_array) {
        for layer in layers {
            let digest = layer.get("digest").and_then(Value::as_str).ok_or_else(|| {
                RegistryError::InvalidManifest("manifest layers require digests".to_owned())
            })?;
            let media_type = layer.get("mediaType").and_then(Value::as_str);
            let size = layer.get("size").and_then(Value::as_i64);
            update_blob_version(
                pool,
                package_id,
                package_version_id,
                digest,
                media_type,
                size,
            )
            .await?;
        }
    }
    Ok(())
}

async fn update_blob_version(
    pool: &PgPool,
    package_id: Uuid,
    package_version_id: Uuid,
    digest: &str,
    media_type: Option<&str>,
    size: Option<i64>,
) -> Result<(), RegistryError> {
    sqlx::query(
        r#"
        UPDATE package_blobs
        SET package_version_id = $1,
            media_type = COALESCE($2, media_type),
            size_bytes = COALESCE($3, size_bytes),
            byte_size = COALESCE($3, byte_size)
        WHERE package_id = $4 AND lower(digest) = lower($5)
        "#,
    )
    .bind(package_version_id)
    .bind(media_type)
    .bind(size)
    .bind(package_id)
    .bind(validate_digest(digest)?)
    .execute(pool)
    .await?;
    Ok(())
}

async fn record_download(
    pool: &PgPool,
    package_id: Uuid,
    package_version_id: Option<Uuid>,
    actor_user_id: Option<Uuid>,
) -> Result<(), RegistryError> {
    sqlx::query(
        "INSERT INTO package_downloads (package_id, package_version_id, downloaded_by_user_id) VALUES ($1, $2, $3)",
    )
    .bind(package_id)
    .bind(package_version_id)
    .bind(actor_user_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn audit_registry_event(
    pool: &PgPool,
    event: RegistryAuditEvent<'_>,
) -> Result<(), RegistryError> {
    sqlx::query(
        r#"
        INSERT INTO package_registry_audit_events (
            package_id, package_version_id, actor_user_id, event_type, reference, digest, user_agent
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(event.package_id)
    .bind(event.package_version_id)
    .bind(event.actor_user_id)
    .bind(event.event_type)
    .bind(event.reference)
    .bind(event.digest)
    .bind(event.user_agent)
    .execute(pool)
    .await?;
    Ok(())
}

fn manifest_blob_size(manifest: &Value) -> Option<i64> {
    let config_size = manifest
        .get("config")
        .and_then(|config| config.get("size"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let layer_size = manifest
        .get("layers")
        .and_then(Value::as_array)
        .map(|layers| {
            layers
                .iter()
                .filter_map(|layer| layer.get("size").and_then(Value::as_i64))
                .sum::<i64>()
        })
        .unwrap_or(0);
    Some(config_size + layer_size)
}

fn registry_storage_root() -> PathBuf {
    std::env::var("OPENGITHUB_PACKAGE_REGISTRY_STORAGE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("opengithub-package-registry"))
}

fn registry_storage_path(storage_key: &str) -> Result<PathBuf, RegistryError> {
    if storage_key.contains("..") {
        return Err(RegistryError::InvalidReference(
            "storage key may not contain parent traversal".to_owned(),
        ));
    }
    Ok(registry_storage_root().join(storage_key))
}

fn upload_storage_key(package_id: Uuid, upload_id: Uuid) -> String {
    format!("uploads/{package_id}/{upload_id}")
}

fn blob_storage_key(package_id: Uuid, digest: &str) -> String {
    format!("blobs/{package_id}/{}", digest.replace(':', "-"))
}

async fn ensure_parent_dir(path: &Path) -> Result<(), RegistryError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    Ok(())
}

fn sha256_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    format!("sha256:{hex}")
}

fn upload_location(namespace: &str, image: &str, upload_id: Uuid) -> String {
    format!("/v2/{namespace}/{image}/blobs/uploads/{upload_id}")
}

fn registry_range(size: i64) -> String {
    if size <= 0 {
        "0-0".to_owned()
    } else {
        format!("0-{}", size - 1)
    }
}
