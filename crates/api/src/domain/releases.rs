use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    api_types::ListEnvelope,
    domain::webhooks::{enqueue_repository_webhook_event, WebhookError},
    jobs::{enqueue_job, JobLeaseError},
};

use super::{
    markdown::{render_markdown, MarkdownError, RenderMarkdownInput},
    repositories::{
        can_read_repository, get_repository_by_owner_name, Repository, RepositoryError,
        RepositoryVisibility,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryReleaseSummary {
    pub id: Uuid,
    pub tag_name: String,
    pub title: String,
    pub body_excerpt: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub latest: bool,
    pub verified: bool,
    pub target_oid: Option<String>,
    pub short_oid: Option<String>,
    pub author: ReleaseActor,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub assets: Vec<ReleaseAsset>,
    pub reactions: ReleaseReactionSummary,
    pub contributors: Vec<ReleaseContributorSummary>,
    pub links: ReleaseLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryReleaseDetail {
    #[serde(flatten)]
    pub summary: RepositoryReleaseSummary,
    pub body: Option<String>,
    pub body_html: String,
    pub immutable: bool,
    pub tag_signature_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseAsset {
    pub id: Uuid,
    pub name: String,
    pub label: Option<String>,
    pub content_type: String,
    pub byte_size: i64,
    pub download_count: i64,
    pub checksum_sha256: Option<String>,
    pub href: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseTagSummary {
    pub id: Uuid,
    pub name: String,
    pub target_oid: Option<String>,
    pub short_oid: Option<String>,
    pub commit_message: Option<String>,
    pub committed_at: Option<DateTime<Utc>>,
    pub verified: bool,
    pub signature_summary: Option<String>,
    pub release_id: Option<Uuid>,
    pub release_href: Option<String>,
    pub zipball_href: String,
    pub tarball_href: String,
    pub compare_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseActor {
    pub id: Option<Uuid>,
    pub login: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseContributorSummary {
    pub id: Uuid,
    pub login: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseReactionSummary {
    pub total_count: i64,
    pub thumbs_up: i64,
    pub thumbs_down: i64,
    pub laugh: i64,
    pub hooray: i64,
    pub confused: i64,
    pub heart: i64,
    pub rocket: i64,
    pub eyes: i64,
    pub viewer_reaction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseLinks {
    pub html_href: String,
    pub api_href: String,
    pub tag_href: String,
    pub zipball_href: String,
    pub tarball_href: String,
    pub compare_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseArchiveMetadata {
    pub tag_name: String,
    pub format: String,
    pub href: String,
    pub authorization: String,
    pub target_oid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseManagementContext {
    pub repository_id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub can_write: bool,
    pub archived: bool,
    pub release: Option<RepositoryReleaseDetail>,
    pub available_tags: Vec<ReleaseRefOption>,
    pub available_refs: Vec<ReleaseRefOption>,
    pub default_target: String,
    pub previous_tag_candidates: Vec<ReleaseRefOption>,
    pub latest_policy_options: Vec<ReleaseLatestPolicyOption>,
    pub upload_limits: ReleaseUploadLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseRefOption {
    pub name: String,
    pub short_name: String,
    pub kind: String,
    pub target_oid: Option<String>,
    pub short_oid: Option<String>,
    pub committed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseLatestPolicyOption {
    pub value: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseUploadLimits {
    pub max_asset_bytes: i64,
    pub max_asset_count: i64,
    pub allowed_storage_kinds: Vec<String>,
    pub expires_in_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedReleaseNotesRequest {
    pub target: String,
    pub previous_tag: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedReleaseNotesPreview {
    pub title: String,
    pub body: String,
    pub target: ReleaseRefOption,
    pub previous_tag: Option<ReleaseRefOption>,
    pub commit_count: i64,
    pub merged_pull_request_count: i64,
    pub contributors: Vec<ReleaseContributorSummary>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseUploadIntentRequest {
    pub name: String,
    pub content_type: Option<String>,
    pub byte_size: i64,
    pub checksum_sha256: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseUploadCompleteRequest {
    pub release_id: Uuid,
    pub handoff_token: String,
    pub label: Option<String>,
    pub checksum_sha256: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseUploadCancelRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseUploadIntent {
    pub id: Uuid,
    pub asset_name: String,
    pub content_type: String,
    pub byte_size: i64,
    pub checksum_sha256: Option<String>,
    pub storage_kind: String,
    pub upload_url: String,
    pub handoff_token: String,
    pub status: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseAssetDownloadMetadata {
    pub asset: ReleaseAsset,
    pub release_id: Uuid,
    pub release_tag_name: String,
    pub download_href: String,
    pub authorization: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseMutation {
    pub tag_name: Option<String>,
    pub target: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub draft: Option<bool>,
    pub prerelease: Option<bool>,
    pub latest_policy: Option<String>,
    pub delete_tag: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseAssetMutation {
    pub name: String,
    pub label: Option<String>,
    pub content_type: Option<String>,
    pub byte_size: Option<i64>,
    pub checksum_sha256: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ReleasesError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error("release was not found")]
    NotFound,
    #[error("release tag was not found")]
    TagNotFound,
    #[error("unsupported archive format")]
    UnsupportedArchiveFormat,
    #[error("unsupported reaction")]
    UnsupportedReaction,
    #[error("authentication is required")]
    AuthenticationRequired,
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Conflict(String),
    #[error("archived repositories cannot modify releases")]
    ArchivedRepository,
    #[error("immutable releases cannot be modified")]
    ImmutableRelease,
    #[error("markdown rendering failed")]
    Markdown,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Webhook(#[from] WebhookError),
    #[error(transparent)]
    Job(#[from] JobLeaseError),
}

pub async fn repository_release_management_context_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    release_id: Option<Uuid>,
    actor_user_id: Option<Uuid>,
) -> Result<ReleaseManagementContext, ReleasesError> {
    let actor_user_id = actor_user_id.ok_or(ReleasesError::AuthenticationRequired)?;
    let repository = writable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    let release = match release_id {
        Some(release_id) => Some(
            release_detail_for_repository(pool, &repository, Some(actor_user_id), release_id)
                .await?,
        ),
        None => None,
    };
    let available_refs = release_ref_options(pool, repository.id, &["branch", "tag"]).await?;
    let available_tags = available_refs
        .iter()
        .filter(|option| option.kind == "tag")
        .cloned()
        .collect::<Vec<_>>();
    Ok(ReleaseManagementContext {
        repository_id: repository.id,
        owner_login: repository.owner_login.clone(),
        name: repository.name.clone(),
        can_write: true,
        archived: repository.is_archived,
        release,
        available_tags: available_tags.clone(),
        available_refs,
        default_target: repository.default_branch.clone(),
        previous_tag_candidates: available_tags,
        latest_policy_options: latest_policy_options(),
        upload_limits: ReleaseUploadLimits {
            max_asset_bytes: 2_147_483_648,
            max_asset_count: 100,
            allowed_storage_kinds: vec!["local".to_owned(), "s3".to_owned()],
            expires_in_seconds: 900,
        },
    })
}

pub async fn generate_repository_release_notes_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    actor_user_id: Option<Uuid>,
    request: GeneratedReleaseNotesRequest,
) -> Result<GeneratedReleaseNotesPreview, ReleasesError> {
    let actor_user_id = actor_user_id.ok_or(ReleasesError::AuthenticationRequired)?;
    let repository = writable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let target = resolve_release_ref(pool, &repository, &request.target).await?;
    let previous_tag = match request
        .previous_tag
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(tag) => Some(resolve_release_tag_ref(pool, &repository, tag).await?),
        None => previous_tag_before(pool, &repository, target.committed_at).await?,
    };
    let commits = commits_between_refs(
        pool,
        repository.id,
        previous_tag.as_ref(),
        target.committed_at,
    )
    .await?;
    let prs = merged_pull_requests_between_refs(
        pool,
        repository.id,
        previous_tag.as_ref(),
        target.committed_at,
    )
    .await?;
    let title = request
        .title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Generated release notes")
        .chars()
        .take(180)
        .collect::<String>();
    let mut body = format!("## {title}\n\n");
    if commits.is_empty() && prs.is_empty() {
        body.push_str("No commits or merged pull requests were found for this range.\n");
    } else {
        if !prs.is_empty() {
            body.push_str("### Merged pull requests\n");
            for row in &prs {
                let number: i64 = row.get("number");
                let pr_title: String = row.get("title");
                body.push_str(&format!("- #{number} {pr_title}\n"));
            }
            body.push('\n');
        }
        if !commits.is_empty() {
            body.push_str("### Commits\n");
            for row in commits.iter().take(25) {
                let message: String = row.get("message");
                let oid: String = row.get("oid");
                body.push_str(&format!("- {} {}\n", short_oid(&oid), first_line(&message)));
            }
        }
    }
    let contributors = generated_note_contributors(
        pool,
        repository.id,
        previous_tag.as_ref(),
        target.committed_at,
    )
    .await?;
    audit_release_event(
        pool,
        repository.id,
        None,
        actor_user_id,
        "release.notes.generated",
        &["target", "previous_tag"],
        json!({}),
        json!({
            "target": target.short_name,
            "previousTag": previous_tag.as_ref().map(|tag| tag.short_name.clone()),
            "commitCount": commits.len(),
            "mergedPullRequestCount": prs.len()
        }),
    )
    .await?;
    Ok(GeneratedReleaseNotesPreview {
        title,
        body: body.chars().take(20_000).collect(),
        target,
        previous_tag,
        commit_count: commits.len() as i64,
        merged_pull_request_count: prs.len() as i64,
        contributors,
    })
}

pub async fn create_repository_release_upload_intent_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    actor_user_id: Option<Uuid>,
    request: ReleaseUploadIntentRequest,
) -> Result<ReleaseUploadIntent, ReleasesError> {
    let actor_user_id = actor_user_id.ok_or(ReleasesError::AuthenticationRequired)?;
    let repository = writable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let asset_name = normalize_asset_name(&request.name)?;
    if request.byte_size <= 0 || request.byte_size > 2_147_483_648 {
        return Err(ReleasesError::Validation(
            "asset byte size must be between 1 byte and 2 GiB".to_owned(),
        ));
    }
    let content_type = request
        .content_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("application/octet-stream")
        .to_owned();
    validate_content_type(&content_type)?;
    let checksum_sha256 = request
        .checksum_sha256
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(validate_sha256)
        .transpose()?;
    let intent_id = Uuid::new_v4();
    let storage_key = format!("releases/pending/{}/{}", repository.id, intent_id);
    let handoff_token = format!("local-upload-{intent_id}");
    let expires_at = Utc::now() + chrono::Duration::seconds(900);
    sqlx::query(
        r#"
        INSERT INTO release_asset_upload_intents (
            id, repository_id, asset_name, content_type, byte_size, checksum_sha256,
            storage_kind, storage_key, handoff_token, created_by_user_id, expires_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'local', $7, $8, $9, $10)
        "#,
    )
    .bind(intent_id)
    .bind(repository.id)
    .bind(&asset_name)
    .bind(&content_type)
    .bind(request.byte_size)
    .bind(&checksum_sha256)
    .bind(&storage_key)
    .bind(&handoff_token)
    .bind(actor_user_id)
    .bind(expires_at)
    .execute(pool)
    .await?;
    audit_release_event(
        pool,
        repository.id,
        None,
        actor_user_id,
        "release.asset.upload_intent.created",
        &["asset"],
        json!({}),
        json!({
            "intentId": intent_id,
            "assetName": asset_name,
            "byteSize": request.byte_size,
            "contentType": content_type,
            "storageKind": "local"
        }),
    )
    .await?;
    enqueue_job(
        pool,
        "release-asset-upload-intent",
        &intent_id.to_string(),
        json!({
            "repositoryId": repository.id,
            "intentId": intent_id,
            "assetName": asset_name,
            "byteSize": request.byte_size,
            "contentType": content_type,
            "storageKind": "local",
            "storageKey": storage_key,
            "expiresAt": expires_at
        }),
    )
    .await?;
    Ok(ReleaseUploadIntent {
        id: intent_id,
        asset_name,
        content_type,
        byte_size: request.byte_size,
        checksum_sha256,
        storage_kind: "local".to_owned(),
        upload_url: format!(
            "/api/repos/{}/{}/releases/manage/upload-intents/{intent_id}/local-upload",
            repository.owner_login, repository.name
        ),
        handoff_token,
        status: "pending".to_owned(),
        expires_at,
    })
}

pub async fn complete_repository_release_upload_intent_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    intent_id: Uuid,
    actor_user_id: Option<Uuid>,
    request: ReleaseUploadCompleteRequest,
) -> Result<RepositoryReleaseDetail, ReleasesError> {
    let actor_user_id = actor_user_id.ok_or(ReleasesError::AuthenticationRequired)?;
    let repository = writable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    ensure_release_mutable(pool, repository.id, request.release_id).await?;
    let intent = sqlx::query(
        r#"
        SELECT id, asset_name, content_type, byte_size, checksum_sha256, storage_kind,
               storage_key, handoff_token, status, expires_at
        FROM release_asset_upload_intents
        WHERE id = $1 AND repository_id = $2
        "#,
    )
    .bind(intent_id)
    .bind(repository.id)
    .fetch_optional(pool)
    .await?
    .ok_or(ReleasesError::NotFound)?;

    let status: String = intent.get("status");
    if status != "pending" {
        return Err(ReleasesError::Conflict(
            "release asset upload intent is no longer pending".to_owned(),
        ));
    }
    let expires_at: DateTime<Utc> = intent.get("expires_at");
    if expires_at <= Utc::now() {
        sqlx::query(
            "UPDATE release_asset_upload_intents SET status = 'expired' WHERE id = $1 AND status = 'pending'",
        )
        .bind(intent_id)
        .execute(pool)
        .await?;
        return Err(ReleasesError::Conflict(
            "release asset upload intent has expired".to_owned(),
        ));
    }
    let handoff_token: String = intent.get("handoff_token");
    if request.handoff_token.trim() != handoff_token {
        return Err(ReleasesError::Validation(
            "release asset upload token is invalid".to_owned(),
        ));
    }

    let name: String = intent.get("asset_name");
    let content_type: String = intent.get("content_type");
    let byte_size: i64 = intent.get("byte_size");
    let storage_kind: String = intent.get("storage_kind");
    let storage_key: String = intent.get("storage_key");
    let checksum_sha256 = match request
        .checksum_sha256
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => Some(validate_sha256(value)?),
        None => intent.get("checksum_sha256"),
    };
    let label = request
        .label
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let asset_id = sqlx::query(
        r#"
        INSERT INTO release_assets (
            repository_id, release_id, name, label, content_type, byte_size,
            storage_kind, storage_key, checksum_sha256, uploaded_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(request.release_id)
    .bind(&name)
    .bind(label)
    .bind(&content_type)
    .bind(byte_size)
    .bind(&storage_kind)
    .bind(&storage_key)
    .bind(&checksum_sha256)
    .bind(actor_user_id)
    .fetch_one(pool)
    .await
    .map_err(map_release_write_error)?
    .get::<Uuid, _>("id");

    sqlx::query(
        r#"
        UPDATE release_asset_upload_intents
        SET release_id = $2,
            asset_id = $3,
            status = 'completed',
            completed_by_user_id = $4,
            completed_at = now()
        WHERE id = $1
        "#,
    )
    .bind(intent_id)
    .bind(request.release_id)
    .bind(asset_id)
    .bind(actor_user_id)
    .execute(pool)
    .await?;

    audit_release_event(
        pool,
        repository.id,
        Some(request.release_id),
        actor_user_id,
        "release.asset.upload_completed",
        &["asset"],
        json!({ "intentId": intent_id }),
        json!({
            "assetId": asset_id,
            "assetName": name,
            "byteSize": byte_size,
            "contentType": content_type,
            "storageKind": storage_kind,
            "checksumSha256": checksum_sha256
        }),
    )
    .await?;
    enqueue_release_side_effects(
        pool,
        repository.id,
        request.release_id,
        "release",
        json!({
            "action": "asset_uploaded",
            "releaseId": request.release_id,
            "assetId": asset_id,
            "assetName": name
        }),
    )
    .await?;
    release_detail_for_repository(pool, &repository, Some(actor_user_id), request.release_id).await
}

pub async fn cancel_repository_release_upload_intent_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    intent_id: Uuid,
    actor_user_id: Option<Uuid>,
    request: ReleaseUploadCancelRequest,
) -> Result<ReleaseUploadIntent, ReleasesError> {
    let actor_user_id = actor_user_id.ok_or(ReleasesError::AuthenticationRequired)?;
    let repository = writable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let row = sqlx::query(
        r#"
        UPDATE release_asset_upload_intents
        SET status = 'cancelled',
            cancelled_by_user_id = $3,
            cancelled_at = now()
        WHERE id = $1
          AND repository_id = $2
          AND status = 'pending'
        RETURNING id, asset_name, content_type, byte_size, checksum_sha256,
                  storage_kind, handoff_token, status, expires_at
        "#,
    )
    .bind(intent_id)
    .bind(repository.id)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ReleasesError::NotFound)?;
    let reason = request
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("cancelled");
    audit_release_event(
        pool,
        repository.id,
        None,
        actor_user_id,
        "release.asset.upload_cancelled",
        &["asset"],
        json!({ "intentId": intent_id }),
        json!({
            "intentId": intent_id,
            "assetName": row.get::<String, _>("asset_name"),
            "reason": reason
        }),
    )
    .await?;
    Ok(ReleaseUploadIntent {
        id: row.get("id"),
        asset_name: row.get("asset_name"),
        content_type: row.get("content_type"),
        byte_size: row.get("byte_size"),
        checksum_sha256: row.get("checksum_sha256"),
        storage_kind: row.get("storage_kind"),
        upload_url: format!(
            "/api/repos/{}/{}/releases/manage/upload-intents/{intent_id}/local-upload",
            repository.owner_login, repository.name
        ),
        handoff_token: row.get("handoff_token"),
        status: row.get("status"),
        expires_at: row.get("expires_at"),
    })
}

pub async fn create_repository_release_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    actor_user_id: Option<Uuid>,
    mutation: ReleaseMutation,
) -> Result<RepositoryReleaseDetail, ReleasesError> {
    let actor_user_id = actor_user_id.ok_or(ReleasesError::AuthenticationRequired)?;
    let repository = writable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let tag_name = normalize_tag_input(mutation.tag_name.as_deref())?;
    ensure_unique_active_tag(pool, repository.id, &tag_name, None).await?;
    let target_commit_id =
        ensure_release_tag_target(pool, &repository, &tag_name, mutation.target.as_deref()).await?;
    let title = mutation
        .title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&tag_name)
        .chars()
        .take(180)
        .collect::<String>();
    let body = mutation.body.unwrap_or_default();
    let prerelease = mutation.prerelease.unwrap_or(false);
    let draft = mutation.draft.unwrap_or(false);
    let latest_policy = normalize_latest_policy(mutation.latest_policy.as_deref())?;
    let body_html =
        render_release_markdown(pool, &repository, tag_name.clone(), Some(body.clone())).await?;
    let excerpt = release_excerpt(&body);
    let published_at = if draft { None } else { Some(Utc::now()) };
    let row = sqlx::query(
        r#"
        INSERT INTO releases (
            repository_id, tag_name, name, body, draft, prerelease, author_user_id,
            target_commit_id, body_html, rendered_body_excerpt, is_latest,
            published_at, updated_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, false, $11, $7)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(&tag_name)
    .bind(&title)
    .bind(&body)
    .bind(draft)
    .bind(prerelease)
    .bind(actor_user_id)
    .bind(target_commit_id)
    .bind(body_html)
    .bind(excerpt)
    .bind(published_at)
    .fetch_one(pool)
    .await
    .map_err(map_release_write_error)?;
    let release_id = row.get("id");
    apply_latest_policy(pool, repository.id, release_id, draft, latest_policy).await?;
    audit_release_event(
        pool,
        repository.id,
        Some(release_id),
        actor_user_id,
        "release.created",
        &["tag_name", "title", "draft", "prerelease"],
        json!({}),
        json!({
            "tagName": tag_name,
            "title": title,
            "draft": draft,
            "prerelease": prerelease
        }),
    )
    .await?;
    if !draft {
        enqueue_release_side_effects(
            pool,
            repository.id,
            release_id,
            "release",
            json!({ "action": "created", "releaseId": release_id, "tagName": tag_name }),
        )
        .await?;
    }
    release_detail_for_repository(pool, &repository, Some(actor_user_id), release_id).await
}

pub async fn update_repository_release_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    release_id: Uuid,
    actor_user_id: Option<Uuid>,
    mutation: ReleaseMutation,
) -> Result<RepositoryReleaseDetail, ReleasesError> {
    let actor_user_id = actor_user_id.ok_or(ReleasesError::AuthenticationRequired)?;
    let repository = writable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    ensure_release_mutable(pool, repository.id, release_id).await?;
    let before =
        release_detail_for_repository(pool, &repository, Some(actor_user_id), release_id).await?;
    let tag_name = match mutation.tag_name.as_deref() {
        Some(value) => normalize_tag_input(Some(value))?,
        None => before.summary.tag_name.clone(),
    };
    ensure_unique_active_tag(pool, repository.id, &tag_name, Some(release_id)).await?;
    let target_commit_id = if mutation.target.is_some() || tag_name != before.summary.tag_name {
        ensure_release_tag_target(pool, &repository, &tag_name, mutation.target.as_deref()).await?
    } else {
        sqlx::query_scalar::<_, Option<Uuid>>(
            "SELECT target_commit_id FROM releases WHERE id = $1 AND repository_id = $2",
        )
        .bind(release_id)
        .bind(repository.id)
        .fetch_one(pool)
        .await?
    };
    let title = mutation
        .title
        .unwrap_or_else(|| before.summary.title.clone())
        .trim()
        .to_owned();
    if title.is_empty() {
        return Err(ReleasesError::Validation(
            "release title cannot be blank".to_owned(),
        ));
    }
    let body = mutation
        .body
        .unwrap_or_else(|| before.body.unwrap_or_default());
    let draft = mutation.draft.unwrap_or(before.summary.draft);
    let prerelease = mutation.prerelease.unwrap_or(before.summary.prerelease);
    let latest_policy = normalize_latest_policy(mutation.latest_policy.as_deref())?;
    let was_draft = before.summary.draft;
    let published_at = if draft {
        None
    } else if was_draft {
        Some(Utc::now())
    } else {
        before.summary.published_at
    };
    let body_html =
        render_release_markdown(pool, &repository, tag_name.clone(), Some(body.clone())).await?;
    let excerpt = release_excerpt(&body);
    sqlx::query(
        r#"
        UPDATE releases
        SET tag_name = $3,
            name = $4,
            body = $5,
            body_html = $6,
            rendered_body_excerpt = $7,
            draft = $8,
            prerelease = $9,
            target_commit_id = $10,
            published_at = $11,
            updated_by_user_id = $12
        WHERE repository_id = $1 AND id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(repository.id)
    .bind(release_id)
    .bind(&tag_name)
    .bind(&title)
    .bind(&body)
    .bind(body_html)
    .bind(excerpt)
    .bind(draft)
    .bind(prerelease)
    .bind(target_commit_id)
    .bind(published_at)
    .bind(actor_user_id)
    .execute(pool)
    .await
    .map_err(map_release_write_error)?;
    apply_latest_policy(pool, repository.id, release_id, draft, latest_policy).await?;
    audit_release_event(
        pool,
        repository.id,
        Some(release_id),
        actor_user_id,
        if was_draft && !draft {
            "release.published"
        } else {
            "release.updated"
        },
        &["tag_name", "title", "body", "draft", "prerelease", "target"],
        json!({
            "tagName": before.summary.tag_name,
            "title": before.summary.title,
            "draft": before.summary.draft,
            "prerelease": before.summary.prerelease
        }),
        json!({
            "tagName": tag_name,
            "title": title,
            "draft": draft,
            "prerelease": prerelease
        }),
    )
    .await?;
    if was_draft && !draft {
        enqueue_release_side_effects(
            pool,
            repository.id,
            release_id,
            "release",
            json!({ "action": "published", "releaseId": release_id, "tagName": tag_name }),
        )
        .await?;
    }
    release_detail_for_repository(pool, &repository, Some(actor_user_id), release_id).await
}

pub async fn publish_repository_release_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    release_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<RepositoryReleaseDetail, ReleasesError> {
    update_repository_release_by_owner_name(
        pool,
        owner_login,
        repo_name,
        release_id,
        actor_user_id,
        ReleaseMutation {
            tag_name: None,
            target: None,
            title: None,
            body: None,
            draft: Some(false),
            prerelease: None,
            latest_policy: Some("automatic".to_owned()),
            delete_tag: None,
        },
    )
    .await
}

pub async fn delete_repository_release_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    release_id: Uuid,
    actor_user_id: Option<Uuid>,
    delete_tag: bool,
) -> Result<(), ReleasesError> {
    let actor_user_id = actor_user_id.ok_or(ReleasesError::AuthenticationRequired)?;
    let repository = writable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    ensure_release_mutable(pool, repository.id, release_id).await?;
    let tag_name = sqlx::query_scalar::<_, String>(
        "SELECT tag_name FROM releases WHERE repository_id = $1 AND id = $2 AND deleted_at IS NULL",
    )
    .bind(repository.id)
    .bind(release_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ReleasesError::NotFound)?;
    let affected = sqlx::query(
        r#"
        UPDATE releases
        SET deleted_at = now(), updated_by_user_id = $3
        WHERE repository_id = $1 AND id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(repository.id)
    .bind(release_id)
    .bind(actor_user_id)
    .execute(pool)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(ReleasesError::NotFound);
    }
    if delete_tag {
        sqlx::query(
            r#"
            DELETE FROM repository_git_refs
            WHERE repository_id = $1
              AND kind = 'tag'
              AND lower(regexp_replace(name, '^refs/tags/', '')) = lower($2)
            "#,
        )
        .bind(repository.id)
        .bind(&tag_name)
        .execute(pool)
        .await?;
    }
    refresh_latest_marker(pool, repository.id).await?;
    audit_release_event(
        pool,
        repository.id,
        Some(release_id),
        actor_user_id,
        "release.deleted",
        &["deleted_at", "delete_tag"],
        json!({}),
        json!({ "deleted": true, "deleteTag": delete_tag, "tagName": tag_name }),
    )
    .await?;
    enqueue_release_side_effects(
        pool,
        repository.id,
        release_id,
        "release",
        json!({ "action": "deleted", "releaseId": release_id, "deleteTag": delete_tag }),
    )
    .await?;
    Ok(())
}

pub async fn create_repository_release_asset_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    release_id: Uuid,
    actor_user_id: Option<Uuid>,
    mutation: ReleaseAssetMutation,
) -> Result<RepositoryReleaseDetail, ReleasesError> {
    let actor_user_id = actor_user_id.ok_or(ReleasesError::AuthenticationRequired)?;
    let repository = writable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    ensure_release_mutable(pool, repository.id, release_id).await?;
    let name = normalize_asset_name(&mutation.name)?;
    let content_type = mutation
        .content_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("application/octet-stream")
        .to_owned();
    let byte_size = mutation.byte_size.unwrap_or(0);
    if byte_size < 0 {
        return Err(ReleasesError::Validation(
            "asset byte size cannot be negative".to_owned(),
        ));
    }
    let storage_key = format!("releases/{release_id}/assets/{}", Uuid::new_v4());
    sqlx::query(
        r#"
        INSERT INTO release_assets (
            repository_id, release_id, name, label, content_type, byte_size,
            storage_kind, storage_key, checksum_sha256, uploaded_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'local', $7, $8, $9)
        "#,
    )
    .bind(repository.id)
    .bind(release_id)
    .bind(&name)
    .bind(
        mutation
            .label
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty()),
    )
    .bind(content_type)
    .bind(byte_size)
    .bind(storage_key)
    .bind(
        mutation
            .checksum_sha256
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty()),
    )
    .bind(actor_user_id)
    .execute(pool)
    .await
    .map_err(map_release_write_error)?;
    audit_release_event(
        pool,
        repository.id,
        Some(release_id),
        actor_user_id,
        "release.asset.created",
        &["asset"],
        json!({}),
        json!({ "assetName": name, "byteSize": byte_size }),
    )
    .await?;
    release_detail_for_repository(pool, &repository, Some(actor_user_id), release_id).await
}

pub async fn delete_repository_release_asset_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    release_id: Uuid,
    asset_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<RepositoryReleaseDetail, ReleasesError> {
    let actor_user_id = actor_user_id.ok_or(ReleasesError::AuthenticationRequired)?;
    let repository = writable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    ensure_release_mutable(pool, repository.id, release_id).await?;
    let affected = sqlx::query(
        r#"
        UPDATE release_assets
        SET deleted_at = now()
        WHERE repository_id = $1 AND release_id = $2 AND id = $3 AND deleted_at IS NULL
        "#,
    )
    .bind(repository.id)
    .bind(release_id)
    .bind(asset_id)
    .execute(pool)
    .await?
    .rows_affected();
    if affected == 0 {
        return Err(ReleasesError::NotFound);
    }
    audit_release_event(
        pool,
        repository.id,
        Some(release_id),
        actor_user_id,
        "release.asset.deleted",
        &["asset"],
        json!({ "assetId": asset_id }),
        json!({ "deleted": true }),
    )
    .await?;
    release_detail_for_repository(pool, &repository, Some(actor_user_id), release_id).await
}

pub async fn repository_release_list_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    actor_user_id: Option<Uuid>,
    page: i64,
    page_size: i64,
) -> Result<ListEnvelope<RepositoryReleaseSummary>, ReleasesError> {
    let repository = readable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    let include_drafts = can_write_if_actor(pool, &repository, actor_user_id).await?;
    let total = release_count(pool, repository.id, include_drafts).await?;
    let rows = sqlx::query(
        r#"
        SELECT releases.id,
               releases.tag_name,
               COALESCE(NULLIF(releases.name, ''), releases.tag_name) AS title,
               releases.rendered_body_excerpt,
               releases.draft,
               releases.prerelease,
               releases.is_latest,
               releases.tag_verified,
               COALESCE(target_commit.oid, ref_commit.oid) AS target_oid,
               releases.author_user_id,
               COALESCE(NULLIF(author.username, ''), author.email) AS author_login,
               author.display_name AS author_display_name,
               author.avatar_url AS author_avatar_url,
               releases.published_at,
               releases.created_at,
               releases.updated_at
        FROM releases
        LEFT JOIN users author ON author.id = releases.author_user_id
        LEFT JOIN commits target_commit ON target_commit.id = releases.target_commit_id
        LEFT JOIN repository_git_refs refs
          ON refs.repository_id = releases.repository_id
         AND lower(regexp_replace(refs.name, '^refs/tags/', '')) = lower(releases.tag_name)
        LEFT JOIN commits ref_commit ON ref_commit.id = refs.target_commit_id
        WHERE releases.repository_id = $1
          AND releases.deleted_at IS NULL
          AND ($2 OR releases.draft = false)
        ORDER BY releases.published_at DESC NULLS LAST, releases.created_at DESC, releases.id
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(repository.id)
    .bind(include_drafts)
    .bind(page_size)
    .bind((page - 1) * page_size)
    .fetch_all(pool)
    .await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(summary_from_row(pool, &repository, actor_user_id, row).await?);
    }
    Ok(ListEnvelope {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn repository_latest_release_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    actor_user_id: Option<Uuid>,
) -> Result<RepositoryReleaseDetail, ReleasesError> {
    let repository = readable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    let include_drafts = can_write_if_actor(pool, &repository, actor_user_id).await?;
    let row = sqlx::query(
        r#"
        SELECT releases.id
        FROM releases
        WHERE releases.repository_id = $1
          AND releases.deleted_at IS NULL
          AND releases.prerelease = false
          AND ($2 OR releases.draft = false)
          AND (releases.published_at IS NOT NULL OR $2)
        ORDER BY releases.is_latest DESC, releases.published_at DESC NULLS LAST, releases.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(include_drafts)
    .fetch_optional(pool)
    .await?
    .ok_or(ReleasesError::NotFound)?;
    release_detail_for_repository(pool, &repository, actor_user_id, row.get("id")).await
}

pub async fn repository_release_detail_by_id_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    release_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<RepositoryReleaseDetail, ReleasesError> {
    let repository = readable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    release_detail_for_repository(pool, &repository, actor_user_id, release_id).await
}

pub async fn repository_release_detail_by_tag_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    tag_name: &str,
    actor_user_id: Option<Uuid>,
) -> Result<RepositoryReleaseDetail, ReleasesError> {
    let repository = readable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    let include_drafts = can_write_if_actor(pool, &repository, actor_user_id).await?;
    let row = sqlx::query(
        r#"
        SELECT id
        FROM releases
        WHERE repository_id = $1
          AND lower(tag_name) = lower($2)
          AND deleted_at IS NULL
          AND ($3 OR draft = false)
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(clean_tag_name(tag_name))
    .bind(include_drafts)
    .fetch_optional(pool)
    .await?
    .ok_or(ReleasesError::NotFound)?;
    release_detail_for_repository(pool, &repository, actor_user_id, row.get("id")).await
}

pub async fn repository_release_tags_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    actor_user_id: Option<Uuid>,
    page: i64,
    page_size: i64,
) -> Result<ListEnvelope<ReleaseTagSummary>, ReleasesError> {
    let repository = readable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    let total = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_git_refs WHERE repository_id = $1 AND kind = 'tag'",
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let rows = sqlx::query(
        r#"
        SELECT refs.id,
               refs.name,
               commits.oid AS target_oid,
               commits.message AS commit_message,
               commits.committed_at,
               commits.author_user_id,
               releases.id AS release_id,
               COALESCE(releases.tag_verified, refs.verified, false) AS verified,
               refs.signature_fingerprint,
               COALESCE(releases.tag_signature_summary, refs.signature_summary) AS signature_summary
        FROM repository_git_refs refs
        LEFT JOIN commits ON commits.id = refs.target_commit_id
        LEFT JOIN releases
          ON releases.repository_id = refs.repository_id
         AND lower(releases.tag_name) = lower(regexp_replace(refs.name, '^refs/tags/', ''))
         AND releases.deleted_at IS NULL
         AND releases.draft = false
        WHERE refs.repository_id = $1 AND refs.kind = 'tag'
        ORDER BY COALESCE(commits.committed_at, refs.updated_at) DESC, lower(refs.name)
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(repository.id)
    .bind(page_size)
    .bind((page - 1) * page_size)
    .fetch_all(pool)
    .await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let release_verified: bool = row.get("verified");
        let author_user_id: Option<Uuid> = row.get("author_user_id");
        let signature_fingerprint: Option<String> = row.get("signature_fingerprint");
        let stored_summary: Option<String> = row.get("signature_summary");
        let mut tag = tag_from_row(&repository, row)?;
        if !release_verified {
            let signature = super::signing_keys::signature_presentation_for_user(
                pool,
                author_user_id,
                signature_fingerprint.as_deref(),
                stored_summary.as_deref(),
            )
            .await
            .map_err(|error| match error {
                super::signing_keys::SigningKeyError::Sqlx(error) => ReleasesError::Sqlx(error),
                _ => ReleasesError::Validation("signature metadata could not be read".to_owned()),
            })?;
            tag.verified = signature.verified;
            tag.signature_summary = signature.signature_summary;
        }
        items.push(tag);
    }
    Ok(ListEnvelope {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn repository_release_archive_metadata_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    tag_name: &str,
    format: &str,
    actor_user_id: Option<Uuid>,
) -> Result<ReleaseArchiveMetadata, ReleasesError> {
    let repository = readable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    let format = match format {
        "zipball" | "zip" => "zipball",
        "tarball" | "tar.gz" => "tarball",
        _ => return Err(ReleasesError::UnsupportedArchiveFormat),
    };
    let tag = tag_lookup(pool, &repository, tag_name).await?;
    let tag_name = short_tag_name(&tag.name);
    if let Some(release_id) =
        visible_release_id_for_tag(pool, &repository, &tag_name, actor_user_id).await?
    {
        sqlx::query(
            "INSERT INTO release_downloads (repository_id, release_id, user_id, source) VALUES ($1, $2, $3, $4)",
        )
        .bind(repository.id)
        .bind(release_id)
        .bind(actor_user_id)
        .bind(format)
        .execute(pool)
        .await?;
    }
    if let Some(target_oid) = tag.target_oid.as_deref() {
        cache_release_archive_metadata(
            pool,
            &repository,
            &tag_name,
            target_oid,
            format,
            actor_user_id,
        )
        .await?;
    }
    Ok(ReleaseArchiveMetadata {
        href: format!(
            "/api/repos/{}/{}/releases/{}/{}",
            repository.owner_login, repository.name, format, tag_name
        ),
        tag_name,
        format: format.to_owned(),
        authorization:
            "repository visibility and viewer permission checked before archive generation"
                .to_owned(),
        target_oid: tag.target_oid,
    })
}

pub async fn repository_release_asset_download_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    asset_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<ReleaseAssetDownloadMetadata, ReleasesError> {
    let repository = readable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    let include_drafts = can_write_if_actor(pool, &repository, actor_user_id).await?;
    let row = sqlx::query(
        r#"
        UPDATE release_assets
        SET download_count = download_count + 1
        FROM releases
        WHERE release_assets.id = $1
          AND release_assets.repository_id = $2
          AND release_assets.release_id = releases.id
          AND releases.repository_id = release_assets.repository_id
          AND release_assets.deleted_at IS NULL
          AND releases.deleted_at IS NULL
          AND ($3 OR releases.draft = false)
        RETURNING release_assets.id,
                  release_assets.name,
                  release_assets.label,
                  release_assets.content_type,
                  release_assets.byte_size,
                  release_assets.download_count,
                  release_assets.checksum_sha256,
                  release_assets.created_at,
                  releases.id AS release_id,
                  releases.tag_name
        "#,
    )
    .bind(asset_id)
    .bind(repository.id)
    .bind(include_drafts)
    .fetch_optional(pool)
    .await?
    .ok_or(ReleasesError::NotFound)?;
    let release_id: Uuid = row.get("release_id");
    sqlx::query(
        "INSERT INTO release_downloads (repository_id, release_id, asset_id, user_id, source) VALUES ($1, $2, $3, $4, 'asset')",
    )
    .bind(repository.id)
    .bind(release_id)
    .bind(asset_id)
    .bind(actor_user_id)
    .execute(pool)
    .await?;
    let asset = ReleaseAsset {
        id: row.get("id"),
        name: row.get("name"),
        label: row.get("label"),
        content_type: row.get("content_type"),
        byte_size: row.get("byte_size"),
        download_count: row.get("download_count"),
        checksum_sha256: row.get("checksum_sha256"),
        href: format!(
            "/api/repos/{}/{}/releases/assets/{}",
            repository.owner_login, repository.name, asset_id
        ),
        created_at: row.get("created_at"),
    };
    Ok(ReleaseAssetDownloadMetadata {
        download_href: format!(
            "/downloads/releases/{}/{}/{}",
            release_id,
            asset.id,
            sanitize_download_name(&asset.name)
        ),
        release_id,
        release_tag_name: row.get("tag_name"),
        asset,
        authorization: "repository visibility and viewer permission checked before asset download"
            .to_owned(),
    })
}

pub async fn toggle_repository_release_reaction_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    release_id: Uuid,
    actor_user_id: Option<Uuid>,
    reaction: &str,
) -> Result<ReleaseReactionSummary, ReleasesError> {
    let actor_user_id = actor_user_id.ok_or(ReleasesError::AuthenticationRequired)?;
    let reaction = normalize_reaction(reaction)?;
    let repository = readable_repository(pool, owner_login, repo_name, Some(actor_user_id)).await?;
    let include_drafts = can_write_if_actor(pool, &repository, Some(actor_user_id)).await?;
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM releases
            WHERE repository_id = $1
              AND id = $2
              AND deleted_at IS NULL
              AND ($3 OR draft = false)
        )
        "#,
    )
    .bind(repository.id)
    .bind(release_id)
    .bind(include_drafts)
    .fetch_one(pool)
    .await?;
    if !exists {
        return Err(ReleasesError::NotFound);
    }

    let deleted = sqlx::query(
        "DELETE FROM release_reactions WHERE release_id = $1 AND user_id = $2 AND reaction = $3",
    )
    .bind(release_id)
    .bind(actor_user_id)
    .bind(reaction)
    .execute(pool)
    .await?
    .rows_affected();
    if deleted == 0 {
        sqlx::query(
            r#"
            INSERT INTO release_reactions (repository_id, release_id, user_id, reaction)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (release_id, user_id, reaction) DO NOTHING
            "#,
        )
        .bind(repository.id)
        .bind(release_id)
        .bind(actor_user_id)
        .bind(reaction)
        .execute(pool)
        .await?;
    }

    Ok(release_reactions(pool, release_id, Some(actor_user_id)).await?)
}

async fn release_detail_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Option<Uuid>,
    release_id: Uuid,
) -> Result<RepositoryReleaseDetail, ReleasesError> {
    let include_drafts = can_write_if_actor(pool, repository, actor_user_id).await?;
    let row = sqlx::query(
        r#"
        SELECT releases.id,
               releases.tag_name,
               COALESCE(NULLIF(releases.name, ''), releases.tag_name) AS title,
               releases.body,
               releases.body_html,
               releases.rendered_body_excerpt,
               releases.draft,
               releases.prerelease,
               releases.is_latest,
               releases.tag_verified,
               releases.tag_signature_summary,
               releases.immutable,
               COALESCE(target_commit.oid, ref_commit.oid) AS target_oid,
               releases.author_user_id,
               COALESCE(NULLIF(author.username, ''), author.email) AS author_login,
               author.display_name AS author_display_name,
               author.avatar_url AS author_avatar_url,
               releases.published_at,
               releases.created_at,
               releases.updated_at
        FROM releases
        LEFT JOIN users author ON author.id = releases.author_user_id
        LEFT JOIN commits target_commit ON target_commit.id = releases.target_commit_id
        LEFT JOIN repository_git_refs refs
          ON refs.repository_id = releases.repository_id
         AND lower(regexp_replace(refs.name, '^refs/tags/', '')) = lower(releases.tag_name)
        LEFT JOIN commits ref_commit ON ref_commit.id = refs.target_commit_id
        WHERE releases.repository_id = $1
          AND releases.id = $2
          AND releases.deleted_at IS NULL
          AND ($3 OR releases.draft = false)
        "#,
    )
    .bind(repository.id)
    .bind(release_id)
    .bind(include_drafts)
    .fetch_optional(pool)
    .await?
    .ok_or(ReleasesError::NotFound)?;

    let body: Option<String> = row.get("body");
    let stored_html: String = row.get("body_html");
    let body_html = if stored_html.trim().is_empty() {
        render_release_markdown(pool, repository, row.get("tag_name"), body.clone()).await?
    } else {
        stored_html
    };
    Ok(RepositoryReleaseDetail {
        summary: summary_from_row(pool, repository, actor_user_id, row).await?,
        body,
        body_html,
        immutable: sqlx::query_scalar::<_, bool>(
            "SELECT immutable FROM releases WHERE id = $1 AND repository_id = $2",
        )
        .bind(release_id)
        .bind(repository.id)
        .fetch_one(pool)
        .await?,
        tag_signature_summary: sqlx::query_scalar::<_, Option<String>>(
            "SELECT tag_signature_summary FROM releases WHERE id = $1 AND repository_id = $2",
        )
        .bind(release_id)
        .bind(repository.id)
        .fetch_one(pool)
        .await?,
    })
}

async fn visible_release_id_for_tag(
    pool: &PgPool,
    repository: &Repository,
    tag_name: &str,
    actor_user_id: Option<Uuid>,
) -> Result<Option<Uuid>, ReleasesError> {
    let include_drafts = can_write_if_actor(pool, repository, actor_user_id).await?;
    sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM releases
        WHERE repository_id = $1
          AND lower(tag_name) = lower($2)
          AND deleted_at IS NULL
          AND ($3 OR draft = false)
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(clean_tag_name(tag_name))
    .bind(include_drafts)
    .fetch_optional(pool)
    .await
    .map_err(ReleasesError::from)
}

async fn readable_repository(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    actor_user_id: Option<Uuid>,
) -> Result<Repository, ReleasesError> {
    let repository = get_repository_by_owner_name(pool, owner_login, repo_name)
        .await?
        .ok_or(RepositoryError::NotFound)?;
    if repository.visibility == RepositoryVisibility::Public {
        return Ok(repository);
    }
    let Some(actor_user_id) = actor_user_id else {
        return Err(ReleasesError::Repository(RepositoryError::PermissionDenied));
    };
    if can_read_repository(pool, &repository, actor_user_id).await? {
        Ok(repository)
    } else {
        Err(ReleasesError::Repository(RepositoryError::PermissionDenied))
    }
}

async fn writable_repository(
    pool: &PgPool,
    owner_login: &str,
    repo_name: &str,
    actor_user_id: Uuid,
) -> Result<Repository, ReleasesError> {
    let repository = get_repository_by_owner_name(pool, owner_login, repo_name)
        .await?
        .ok_or(RepositoryError::NotFound)?;
    if super::repositories::can_write_repository(pool, &repository, actor_user_id).await? {
        Ok(repository)
    } else {
        Err(ReleasesError::Repository(RepositoryError::PermissionDenied))
    }
}

fn ensure_repository_mutable(repository: &Repository) -> Result<(), ReleasesError> {
    if repository.is_archived {
        Err(ReleasesError::ArchivedRepository)
    } else {
        Ok(())
    }
}

async fn ensure_release_mutable(
    pool: &PgPool,
    repository_id: Uuid,
    release_id: Uuid,
) -> Result<(), ReleasesError> {
    let immutable = sqlx::query_scalar::<_, bool>(
        "SELECT immutable FROM releases WHERE repository_id = $1 AND id = $2 AND deleted_at IS NULL",
    )
    .bind(repository_id)
    .bind(release_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ReleasesError::NotFound)?;
    if immutable {
        Err(ReleasesError::ImmutableRelease)
    } else {
        Ok(())
    }
}

async fn can_write_if_actor(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Option<Uuid>,
) -> Result<bool, RepositoryError> {
    let Some(actor_user_id) = actor_user_id else {
        return Ok(false);
    };
    super::repositories::can_write_repository(pool, repository, actor_user_id).await
}

async fn ensure_unique_active_tag(
    pool: &PgPool,
    repository_id: Uuid,
    tag_name: &str,
    except_release_id: Option<Uuid>,
) -> Result<(), ReleasesError> {
    let duplicate = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM releases
            WHERE repository_id = $1
              AND lower(tag_name) = lower($2)
              AND deleted_at IS NULL
              AND ($3::uuid IS NULL OR id <> $3)
        )
        "#,
    )
    .bind(repository_id)
    .bind(tag_name)
    .bind(except_release_id)
    .fetch_one(pool)
    .await?;
    if duplicate {
        Err(ReleasesError::Conflict(
            "an active release already exists for this tag".to_owned(),
        ))
    } else {
        Ok(())
    }
}

async fn ensure_release_tag_target(
    pool: &PgPool,
    repository: &Repository,
    tag_name: &str,
    target: Option<&str>,
) -> Result<Option<Uuid>, ReleasesError> {
    let target = target.map(str::trim).filter(|value| !value.is_empty());
    let ref_candidates = match target {
        Some(value) => vec![
            value.to_owned(),
            format!("refs/tags/{value}"),
            format!("refs/heads/{value}"),
        ],
        None => vec![format!("refs/heads/{}", repository.default_branch)],
    };
    let target_commit_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT target_commit_id
        FROM repository_git_refs
        WHERE repository_id = $1
          AND target_commit_id IS NOT NULL
          AND name = ANY($2)
        ORDER BY CASE WHEN kind = 'tag' THEN 0 ELSE 1 END
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(&ref_candidates)
    .fetch_optional(pool)
    .await?;
    let target_commit_id = match target_commit_id {
        Some(id) => Some(id),
        None => match target {
            Some(value) => {
                sqlx::query_scalar::<_, Uuid>(
                    "SELECT id FROM commits WHERE repository_id = $1 AND oid = $2 LIMIT 1",
                )
                .bind(repository.id)
                .bind(value)
                .fetch_optional(pool)
                .await?
            }
            None => None,
        },
    };
    let existing_tag = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM repository_git_refs
            WHERE repository_id = $1
              AND kind = 'tag'
              AND lower(regexp_replace(name, '^refs/tags/', '')) = lower($2)
        )
        "#,
    )
    .bind(repository.id)
    .bind(tag_name)
    .fetch_one(pool)
    .await?;
    if !existing_tag {
        let Some(commit_id) = target_commit_id else {
            return Err(ReleasesError::Validation(
                "select an existing branch, tag, or commit before creating this release tag"
                    .to_owned(),
            ));
        };
        sqlx::query(
            r#"
            INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
            VALUES ($1, $2, 'tag', $3)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(repository.id)
        .bind(format!("refs/tags/{tag_name}"))
        .bind(commit_id)
        .execute(pool)
        .await?;
    }
    Ok(target_commit_id)
}

async fn refresh_latest_marker(pool: &PgPool, repository_id: Uuid) -> Result<(), ReleasesError> {
    sqlx::query(
        r#"
        UPDATE releases
        SET is_latest = false
        WHERE repository_id = $1
        "#,
    )
    .bind(repository_id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        WITH latest AS (
            SELECT id
            FROM releases
            WHERE repository_id = $1
              AND deleted_at IS NULL
              AND draft = false
              AND prerelease = false
              AND published_at IS NOT NULL
            ORDER BY published_at DESC, created_at DESC
            LIMIT 1
        )
        UPDATE releases
        SET is_latest = true
        WHERE id IN (SELECT id FROM latest)
        "#,
    )
    .bind(repository_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn apply_latest_policy(
    pool: &PgPool,
    repository_id: Uuid,
    release_id: Uuid,
    draft: bool,
    policy: LatestPolicy,
) -> Result<(), ReleasesError> {
    match policy {
        LatestPolicy::Automatic => refresh_latest_marker(pool, repository_id).await,
        LatestPolicy::Latest => {
            if draft {
                return Err(ReleasesError::Validation(
                    "draft releases cannot be marked latest".to_owned(),
                ));
            }
            sqlx::query("UPDATE releases SET is_latest = false WHERE repository_id = $1")
                .bind(repository_id)
                .execute(pool)
                .await?;
            sqlx::query(
                "UPDATE releases SET is_latest = true WHERE repository_id = $1 AND id = $2",
            )
            .bind(repository_id)
            .bind(release_id)
            .execute(pool)
            .await?;
            Ok(())
        }
        LatestPolicy::NotLatest => {
            sqlx::query(
                "UPDATE releases SET is_latest = false WHERE repository_id = $1 AND id = $2",
            )
            .bind(repository_id)
            .bind(release_id)
            .execute(pool)
            .await?;
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LatestPolicy {
    Automatic,
    Latest,
    NotLatest,
}

fn normalize_latest_policy(policy: Option<&str>) -> Result<LatestPolicy, ReleasesError> {
    match policy.unwrap_or("automatic").trim() {
        "automatic" | "" => Ok(LatestPolicy::Automatic),
        "latest" => Ok(LatestPolicy::Latest),
        "not_latest" | "notLatest" => Ok(LatestPolicy::NotLatest),
        _ => Err(ReleasesError::Validation(
            "latest policy must be automatic, latest, or not_latest".to_owned(),
        )),
    }
}

fn latest_policy_options() -> Vec<ReleaseLatestPolicyOption> {
    vec![
        ReleaseLatestPolicyOption {
            value: "automatic".to_owned(),
            label: "Automatic".to_owned(),
            description: "Use the newest non-prerelease publication as latest.".to_owned(),
        },
        ReleaseLatestPolicyOption {
            value: "latest".to_owned(),
            label: "Set as latest".to_owned(),
            description: "Pin this published release as the latest release.".to_owned(),
        },
        ReleaseLatestPolicyOption {
            value: "not_latest".to_owned(),
            label: "Do not mark latest".to_owned(),
            description: "Keep this release out of the latest marker.".to_owned(),
        },
    ]
}

async fn enqueue_release_side_effects(
    pool: &PgPool,
    repository_id: Uuid,
    release_id: Uuid,
    event: &str,
    payload: serde_json::Value,
) -> Result<(), ReleasesError> {
    let _queued =
        enqueue_repository_webhook_event(pool, repository_id, event, payload.clone()).await?;
    enqueue_job(
        pool,
        "release-activity",
        &format!("{event}-{release_id}"),
        json!({
            "repositoryId": repository_id,
            "releaseId": release_id,
            "event": event,
            "payload": payload
        }),
    )
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn audit_release_event(
    pool: &PgPool,
    repository_id: Uuid,
    release_id: Option<Uuid>,
    actor_user_id: Uuid,
    event_type: &str,
    changed_fields: &[&str],
    before_state: serde_json::Value,
    after_state: serde_json::Value,
) -> Result<(), ReleasesError> {
    let fields = changed_fields
        .iter()
        .map(|field| (*field).to_owned())
        .collect::<Vec<_>>();
    sqlx::query(
        r#"
        INSERT INTO release_audit_events (
            repository_id, release_id, actor_user_id, event_type,
            changed_fields, before_state, after_state
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(repository_id)
    .bind(release_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(fields)
    .bind(before_state)
    .bind(after_state)
    .execute(pool)
    .await?;
    Ok(())
}

async fn release_ref_options(
    pool: &PgPool,
    repository_id: Uuid,
    kinds: &[&str],
) -> Result<Vec<ReleaseRefOption>, ReleasesError> {
    let kind_values = kinds
        .iter()
        .map(|kind| (*kind).to_owned())
        .collect::<Vec<_>>();
    let rows = sqlx::query(
        r#"
        SELECT refs.name, refs.kind, commits.oid AS target_oid, commits.committed_at
        FROM repository_git_refs refs
        LEFT JOIN commits ON commits.id = refs.target_commit_id
        WHERE refs.repository_id = $1 AND refs.kind = ANY($2)
        ORDER BY refs.kind, COALESCE(commits.committed_at, refs.updated_at) DESC, lower(refs.name)
        "#,
    )
    .bind(repository_id)
    .bind(kind_values)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(ref_option_from_row)
        .collect::<Vec<_>>())
}

async fn resolve_release_ref(
    pool: &PgPool,
    repository: &Repository,
    value: &str,
) -> Result<ReleaseRefOption, ReleasesError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ReleasesError::Validation(
            "target ref is required".to_owned(),
        ));
    }
    let candidates = vec![
        value.to_owned(),
        format!("refs/heads/{value}"),
        format!("refs/tags/{value}"),
    ];
    let row = sqlx::query(
        r#"
        SELECT refs.name, refs.kind, commits.oid AS target_oid, commits.committed_at
        FROM repository_git_refs refs
        LEFT JOIN commits ON commits.id = refs.target_commit_id
        WHERE refs.repository_id = $1 AND refs.name = ANY($2)
        ORDER BY CASE WHEN refs.kind = 'branch' THEN 0 ELSE 1 END
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(candidates)
    .fetch_optional(pool)
    .await?;
    if let Some(row) = row {
        return Ok(ref_option_from_row(row));
    }
    let row = sqlx::query(
        r#"
        SELECT $2::text AS name, 'commit'::text AS kind, oid AS target_oid, committed_at
        FROM commits
        WHERE repository_id = $1 AND oid = $2
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(value)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ReleasesError::Validation("target ref was not found".to_owned()))?;
    Ok(ref_option_from_row(row))
}

async fn resolve_release_tag_ref(
    pool: &PgPool,
    repository: &Repository,
    tag_name: &str,
) -> Result<ReleaseRefOption, ReleasesError> {
    let tag = clean_tag_name(tag_name);
    let row = sqlx::query(
        r#"
        SELECT refs.name, refs.kind, commits.oid AS target_oid, commits.committed_at
        FROM repository_git_refs refs
        LEFT JOIN commits ON commits.id = refs.target_commit_id
        WHERE refs.repository_id = $1
          AND refs.kind = 'tag'
          AND lower(regexp_replace(refs.name, '^refs/tags/', '')) = lower($2)
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(tag)
    .fetch_optional(pool)
    .await?
    .ok_or(ReleasesError::TagNotFound)?;
    Ok(ref_option_from_row(row))
}

async fn previous_tag_before(
    pool: &PgPool,
    repository: &Repository,
    before: Option<DateTime<Utc>>,
) -> Result<Option<ReleaseRefOption>, ReleasesError> {
    let row = sqlx::query(
        r#"
        SELECT refs.name, refs.kind, commits.oid AS target_oid, commits.committed_at
        FROM repository_git_refs refs
        LEFT JOIN commits ON commits.id = refs.target_commit_id
        WHERE refs.repository_id = $1
          AND refs.kind = 'tag'
          AND ($2::timestamptz IS NULL OR commits.committed_at < $2)
        ORDER BY commits.committed_at DESC NULLS LAST, refs.updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(before)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(ref_option_from_row))
}

async fn commits_between_refs(
    pool: &PgPool,
    repository_id: Uuid,
    previous_tag: Option<&ReleaseRefOption>,
    target_time: Option<DateTime<Utc>>,
) -> Result<Vec<sqlx::postgres::PgRow>, ReleasesError> {
    let rows = sqlx::query(
        r#"
        SELECT commits.oid, commits.message, commits.committed_at
        FROM commits
        WHERE commits.repository_id = $1
          AND ($2::timestamptz IS NULL OR commits.committed_at > $2)
          AND ($3::timestamptz IS NULL OR commits.committed_at <= $3)
        ORDER BY commits.committed_at DESC
        LIMIT 100
        "#,
    )
    .bind(repository_id)
    .bind(previous_tag.and_then(|tag| tag.committed_at))
    .bind(target_time)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

async fn merged_pull_requests_between_refs(
    pool: &PgPool,
    repository_id: Uuid,
    previous_tag: Option<&ReleaseRefOption>,
    target_time: Option<DateTime<Utc>>,
) -> Result<Vec<sqlx::postgres::PgRow>, ReleasesError> {
    let rows = sqlx::query(
        r#"
        SELECT number, title, merged_at
        FROM pull_requests
        WHERE repository_id = $1
          AND state = 'merged'
          AND merged_at IS NOT NULL
          AND ($2::timestamptz IS NULL OR merged_at > $2)
          AND ($3::timestamptz IS NULL OR merged_at <= $3)
        ORDER BY merged_at DESC
        LIMIT 50
        "#,
    )
    .bind(repository_id)
    .bind(previous_tag.and_then(|tag| tag.committed_at))
    .bind(target_time)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

async fn generated_note_contributors(
    pool: &PgPool,
    repository_id: Uuid,
    previous_tag: Option<&ReleaseRefOption>,
    target_time: Option<DateTime<Utc>>,
) -> Result<Vec<ReleaseContributorSummary>, ReleasesError> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.display_name,
               users.avatar_url
        FROM commits
        JOIN users ON users.id = commits.author_user_id
        WHERE commits.repository_id = $1
          AND ($2::timestamptz IS NULL OR commits.committed_at > $2)
          AND ($3::timestamptz IS NULL OR commits.committed_at <= $3)
        ORDER BY login
        LIMIT 20
        "#,
    )
    .bind(repository_id)
    .bind(previous_tag.and_then(|tag| tag.committed_at))
    .bind(target_time)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ReleaseContributorSummary {
            id: row.get("id"),
            login: row.get("login"),
            display_name: row.get("display_name"),
            avatar_url: row.get("avatar_url"),
        })
        .collect())
}

fn ref_option_from_row(row: sqlx::postgres::PgRow) -> ReleaseRefOption {
    let name: String = row.get("name");
    let kind: String = row.get("kind");
    let short_name = match kind.as_str() {
        "branch" => name.trim_start_matches("refs/heads/").to_owned(),
        "tag" => short_tag_name(&name),
        _ => name.clone(),
    };
    let target_oid: Option<String> = row.get("target_oid");
    ReleaseRefOption {
        name,
        short_name,
        kind,
        short_oid: target_oid.as_deref().map(short_oid),
        target_oid,
        committed_at: row.get("committed_at"),
    }
}

fn normalize_tag_input(tag_name: Option<&str>) -> Result<String, ReleasesError> {
    let Some(tag_name) = tag_name else {
        return Err(ReleasesError::Validation(
            "release tag name is required".to_owned(),
        ));
    };
    let tag_name = clean_tag_name(tag_name);
    if tag_name.is_empty()
        || tag_name.len() > 120
        || tag_name
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(ReleasesError::Validation(
            "release tag names cannot be blank or contain whitespace".to_owned(),
        ));
    }
    Ok(tag_name)
}

fn normalize_asset_name(name: &str) -> Result<String, ReleasesError> {
    let name = name.trim();
    if name.is_empty() || name.len() > 180 || name.contains('/') || name.contains('\\') {
        return Err(ReleasesError::Validation(
            "asset name cannot be blank or contain path separators".to_owned(),
        ));
    }
    Ok(name.to_owned())
}

fn validate_content_type(content_type: &str) -> Result<(), ReleasesError> {
    let valid = content_type.len() <= 120
        && content_type.contains('/')
        && content_type
            .chars()
            .all(|character| !character.is_control() && !character.is_whitespace());
    if valid {
        Ok(())
    } else {
        Err(ReleasesError::Validation(
            "asset content type must be a valid MIME type".to_owned(),
        ))
    }
}

fn validate_sha256(value: &str) -> Result<String, ReleasesError> {
    let value = value.trim();
    let valid = value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit());
    if valid {
        Ok(value.to_owned())
    } else {
        Err(ReleasesError::Validation(
            "asset checksum must be a 64 character SHA-256 hex digest".to_owned(),
        ))
    }
}

fn first_line(message: &str) -> String {
    message
        .lines()
        .next()
        .unwrap_or(message)
        .chars()
        .take(140)
        .collect()
}

fn release_excerpt(body: &str) -> String {
    body.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(280)
        .collect()
}

fn map_release_write_error(error: sqlx::Error) -> ReleasesError {
    if let sqlx::Error::Database(database_error) = &error {
        if database_error.is_unique_violation() {
            return ReleasesError::Conflict("release tag or asset name already exists".to_owned());
        }
    }
    ReleasesError::Sqlx(error)
}

async fn release_count(
    pool: &PgPool,
    repository_id: Uuid,
    include_drafts: bool,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM releases WHERE repository_id = $1 AND deleted_at IS NULL AND ($2 OR draft = false)",
    )
    .bind(repository_id)
    .bind(include_drafts)
    .fetch_one(pool)
    .await
}

async fn summary_from_row(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Option<Uuid>,
    row: sqlx::postgres::PgRow,
) -> Result<RepositoryReleaseSummary, ReleasesError> {
    let release_id: Uuid = row.get("id");
    let tag_name: String = row.get("tag_name");
    let target_oid: Option<String> = row.get("target_oid");
    Ok(RepositoryReleaseSummary {
        id: release_id,
        tag_name: tag_name.clone(),
        title: row.get("title"),
        body_excerpt: row.get("rendered_body_excerpt"),
        draft: row.get("draft"),
        prerelease: row.get("prerelease"),
        latest: row.get("is_latest"),
        verified: row.get("tag_verified"),
        short_oid: target_oid.as_deref().map(short_oid),
        target_oid,
        author: ReleaseActor {
            id: row.get("author_user_id"),
            login: row.get("author_login"),
            display_name: row.get("author_display_name"),
            avatar_url: row.get("author_avatar_url"),
        },
        published_at: row.get("published_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        assets: release_assets(pool, repository, release_id).await?,
        reactions: release_reactions(pool, release_id, actor_user_id).await?,
        contributors: release_contributors(pool, repository, release_id).await?,
        links: release_links(repository, &tag_name, release_id),
    })
}

async fn release_assets(
    pool: &PgPool,
    repository: &Repository,
    release_id: Uuid,
) -> Result<Vec<ReleaseAsset>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, label, content_type, byte_size, download_count, checksum_sha256, created_at
        FROM release_assets
        WHERE repository_id = $1 AND release_id = $2 AND deleted_at IS NULL
        ORDER BY created_at, lower(name)
        "#,
    )
    .bind(repository.id)
    .bind(release_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let id = row.get("id");
            Ok(ReleaseAsset {
                id,
                name: row.get("name"),
                label: row.get("label"),
                content_type: row.get("content_type"),
                byte_size: row.get("byte_size"),
                download_count: row.get("download_count"),
                checksum_sha256: row.get("checksum_sha256"),
                href: format!(
                    "/api/repos/{}/{}/releases/assets/{}",
                    repository.owner_login, repository.name, id
                ),
                created_at: row.get("created_at"),
            })
        })
        .collect()
}

async fn release_reactions(
    pool: &PgPool,
    release_id: Uuid,
    actor_user_id: Option<Uuid>,
) -> Result<ReleaseReactionSummary, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT reaction, count(*)::bigint AS count FROM release_reactions WHERE release_id = $1 GROUP BY reaction",
    )
    .bind(release_id)
    .fetch_all(pool)
    .await?;
    let mut summary = ReleaseReactionSummary {
        total_count: 0,
        thumbs_up: 0,
        thumbs_down: 0,
        laugh: 0,
        hooray: 0,
        confused: 0,
        heart: 0,
        rocket: 0,
        eyes: 0,
        viewer_reaction: None,
    };
    for row in rows {
        let count: i64 = row.get("count");
        summary.total_count += count;
        match row.get::<String, _>("reaction").as_str() {
            "thumbs_up" => summary.thumbs_up = count,
            "thumbs_down" => summary.thumbs_down = count,
            "laugh" => summary.laugh = count,
            "hooray" => summary.hooray = count,
            "confused" => summary.confused = count,
            "heart" => summary.heart = count,
            "rocket" => summary.rocket = count,
            "eyes" => summary.eyes = count,
            _ => {}
        }
    }
    if let Some(actor_user_id) = actor_user_id {
        summary.viewer_reaction = sqlx::query_scalar::<_, Option<String>>(
            "SELECT reaction FROM release_reactions WHERE release_id = $1 AND user_id = $2 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(release_id)
        .bind(actor_user_id)
        .fetch_optional(pool)
        .await?
        .flatten();
    }
    Ok(summary)
}

async fn release_contributors(
    pool: &PgPool,
    repository: &Repository,
    release_id: Uuid,
) -> Result<Vec<ReleaseContributorSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.display_name,
               users.avatar_url
        FROM releases
        JOIN commits ON commits.repository_id = releases.repository_id
        JOIN users ON users.id = commits.author_user_id
        WHERE releases.id = $1 AND releases.repository_id = $2
        ORDER BY login
        LIMIT 8
        "#,
    )
    .bind(release_id)
    .bind(repository.id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ReleaseContributorSummary {
            id: row.get("id"),
            login: row.get("login"),
            display_name: row.get("display_name"),
            avatar_url: row.get("avatar_url"),
        })
        .collect())
}

fn release_links(repository: &Repository, tag_name: &str, release_id: Uuid) -> ReleaseLinks {
    let tag = clean_tag_name(tag_name);
    ReleaseLinks {
        html_href: format!(
            "/{}/{}/releases/tag/{}",
            repository.owner_login, repository.name, tag
        ),
        api_href: format!(
            "/api/repos/{}/{}/releases/{}",
            repository.owner_login, repository.name, release_id
        ),
        tag_href: format!(
            "/{}/{}/tree/{}",
            repository.owner_login, repository.name, tag
        ),
        zipball_href: format!(
            "/api/repos/{}/{}/releases/zipball/{}",
            repository.owner_login, repository.name, tag
        ),
        tarball_href: format!(
            "/api/repos/{}/{}/releases/tarball/{}",
            repository.owner_login, repository.name, tag
        ),
        compare_href: format!(
            "/{}/{}/compare/{}",
            repository.owner_login, repository.name, tag
        ),
    }
}

fn tag_from_row(
    repository: &Repository,
    row: sqlx::postgres::PgRow,
) -> Result<ReleaseTagSummary, sqlx::Error> {
    let name: String = row.get("name");
    let short_name = short_tag_name(&name);
    let target_oid: Option<String> = row.get("target_oid");
    let release_id: Option<Uuid> = row.get("release_id");
    Ok(ReleaseTagSummary {
        id: row.get("id"),
        name: short_name.clone(),
        short_oid: target_oid.as_deref().map(short_oid),
        target_oid,
        commit_message: row.get("commit_message"),
        committed_at: row.get("committed_at"),
        verified: row.get("verified"),
        signature_summary: row.get("signature_summary"),
        release_id,
        release_href: release_id.map(|_| {
            format!(
                "/{}/{}/releases/tag/{}",
                repository.owner_login, repository.name, short_name
            )
        }),
        zipball_href: format!(
            "/api/repos/{}/{}/releases/zipball/{}",
            repository.owner_login, repository.name, short_name
        ),
        tarball_href: format!(
            "/api/repos/{}/{}/releases/tarball/{}",
            repository.owner_login, repository.name, short_name
        ),
        compare_href: format!(
            "/{}/{}/compare/{}...{}",
            repository.owner_login, repository.name, short_name, repository.default_branch
        ),
    })
}

async fn cache_release_archive_metadata(
    pool: &PgPool,
    repository: &Repository,
    tag_name: &str,
    target_oid: &str,
    format: &str,
    actor_user_id: Option<Uuid>,
) -> Result<(), ReleasesError> {
    let storage_format = match format {
        "zipball" => "zip",
        "tarball" => "tar",
        _ => return Ok(()),
    };
    let storage_key = format!(
        "release-archives/{}/{}/{}-{}.{}",
        repository.id,
        clean_tag_name(tag_name).replace('/', "-"),
        target_oid.get(..12).unwrap_or(target_oid),
        format,
        storage_format
    );
    sqlx::query(
        r#"
        INSERT INTO repository_archives (
            repository_id, ref_name, target_oid, format, storage_key, byte_size,
            status, created_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, 0, 'generating', $6)
        ON CONFLICT (repository_id, ref_name, target_oid, format)
        DO UPDATE SET status = repository_archives.status,
                      created_by_user_id = COALESCE(repository_archives.created_by_user_id, EXCLUDED.created_by_user_id)
        "#,
    )
    .bind(repository.id)
    .bind(format!("refs/tags/{}", clean_tag_name(tag_name)))
    .bind(target_oid)
    .bind(storage_format)
    .bind(storage_key)
    .bind(actor_user_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn tag_lookup(
    pool: &PgPool,
    repository: &Repository,
    tag_name: &str,
) -> Result<TagLookup, ReleasesError> {
    let clean = clean_tag_name(tag_name);
    let row = sqlx::query(
        r#"
        SELECT refs.name, commits.oid AS target_oid
        FROM repository_git_refs refs
        LEFT JOIN commits ON commits.id = refs.target_commit_id
        WHERE refs.repository_id = $1
          AND refs.kind = 'tag'
          AND lower(regexp_replace(refs.name, '^refs/tags/', '')) = lower($2)
        "#,
    )
    .bind(repository.id)
    .bind(clean)
    .fetch_optional(pool)
    .await?
    .ok_or(ReleasesError::TagNotFound)?;
    Ok(TagLookup {
        name: row.get("name"),
        target_oid: row.get("target_oid"),
    })
}

struct TagLookup {
    name: String,
    target_oid: Option<String>,
}

async fn render_release_markdown(
    pool: &PgPool,
    repository: &Repository,
    tag_name: String,
    body: Option<String>,
) -> Result<String, ReleasesError> {
    let rendered = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            markdown: body.unwrap_or_default(),
            repository_id: Some(repository.id),
            owner: Some(repository.owner_login.clone()),
            repo: Some(repository.name.clone()),
            ref_name: Some(tag_name),
            enable_task_toggles: Some(false),
        },
    )
    .await
    .map_err(|error| match error {
        MarkdownError::Sqlx(error) => ReleasesError::Sqlx(error),
        MarkdownError::TooLarge | MarkdownError::TaskNotFound => ReleasesError::Markdown,
    })?;
    Ok(rendered.html)
}

fn clean_tag_name(tag_name: &str) -> String {
    tag_name.trim().trim_start_matches("refs/tags/").to_owned()
}

fn normalize_reaction(reaction: &str) -> Result<&'static str, ReleasesError> {
    match reaction {
        "thumbs_up" | "+1" => Ok("thumbs_up"),
        "thumbs_down" | "-1" => Ok("thumbs_down"),
        "laugh" => Ok("laugh"),
        "hooray" => Ok("hooray"),
        "confused" => Ok("confused"),
        "heart" => Ok("heart"),
        "rocket" => Ok("rocket"),
        "eyes" => Ok("eyes"),
        _ => Err(ReleasesError::UnsupportedReaction),
    }
}

fn sanitize_download_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|character| match character {
            '"' | '\\' | '/' | '\r' | '\n' | '\t' => '_',
            character if character.is_control() => '_',
            character => character,
        })
        .collect::<String>();
    let trimmed = sanitized.trim_matches('.').trim();
    if trimmed.is_empty() {
        "release-asset".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn short_tag_name(tag_name: &str) -> String {
    clean_tag_name(tag_name)
}

fn short_oid(oid: &str) -> String {
    oid.chars().take(7).collect()
}
