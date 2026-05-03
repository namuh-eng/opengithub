use axum::http::HeaderMap;
use base64::Engine as _;
use serde::Serialize;
use serde_json::Value;
use sqlx::{PgPool, QueryBuilder, Row};
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
    Token { user_id: Uuid, token_id: Uuid },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegistryToken {
    pub token: String,
    pub access_token: String,
    pub expires_in: i64,
    pub issued_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("registry credentials are required")]
    Unauthorized,
    #[error("registry token is invalid")]
    InvalidToken,
    #[error("package token is missing packages:read scope")]
    InsufficientScope,
    #[error("manifest was not found")]
    NotFound,
    #[error("{0}")]
    InvalidReference(String),
    #[error("requested manifest media type is not acceptable")]
    NotAcceptable,
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

pub async fn read_registry_manifest(
    pool: &PgPool,
    namespace: &str,
    image: &str,
    reference: &str,
    accept: Option<&str>,
    auth: &RegistryAuth,
    user_agent: Option<&str>,
) -> Result<RegistryManifestRead, RegistryError> {
    validate_name_component(namespace, "namespace")?;
    validate_name_component(image, "image")?;
    let reference = validate_reference(reference)?;
    let actor_user_id = auth.actor_user_id();

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
    builder.push_bind(namespace);
    builder.push(") AND lower(p.name) = lower(");
    builder.push_bind(image);
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
        return match auth {
            RegistryAuth::Anonymous => Err(RegistryError::Unauthorized),
            RegistryAuth::Token { .. } => Err(RegistryError::NotFound),
        };
    }

    let media_type: String = row.try_get("manifest_media_type")?;
    if !accepts_manifest_media_type(accept, &media_type) {
        return Err(RegistryError::NotAcceptable);
    }

    let package_id: Uuid = row.try_get("package_id")?;
    let package_version_id: Uuid = row.try_get("package_version_id")?;
    let digest: Option<String> = row.try_get("digest")?;
    sqlx::query(
        r#"
        INSERT INTO package_registry_audit_events (
            package_id, package_version_id, actor_user_id, event_type, reference, digest, user_agent
        )
        VALUES ($1, $2, $3, 'manifest.read', $4, $5, $6)
        "#,
    )
    .bind(package_id)
    .bind(package_version_id)
    .bind(actor_user_id)
    .bind(&reference)
    .bind(&digest)
    .bind(user_agent)
    .execute(pool)
    .await?;

    Ok(RegistryManifestRead {
        package_id,
        package_version_id,
        package_name: row.try_get("package_name")?,
        namespace: namespace.to_owned(),
        reference,
        digest,
        media_type,
        manifest: row.try_get("manifest")?,
        manifest_size_bytes: row.try_get("manifest_size_bytes")?,
    })
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
