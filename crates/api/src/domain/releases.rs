use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api_types::ListEnvelope;

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
    refresh_latest_marker(pool, repository.id).await?;
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
    refresh_latest_marker(pool, repository.id).await?;
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
) -> Result<(), ReleasesError> {
    let actor_user_id = actor_user_id.ok_or(ReleasesError::AuthenticationRequired)?;
    let repository = writable_repository(pool, owner_login, repo_name, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    ensure_release_mutable(pool, repository.id, release_id).await?;
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
    refresh_latest_marker(pool, repository.id).await?;
    audit_release_event(
        pool,
        repository.id,
        Some(release_id),
        actor_user_id,
        "release.deleted",
        &["deleted_at"],
        json!({}),
        json!({ "deleted": true }),
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
               releases.id AS release_id,
               COALESCE(releases.tag_verified, false) AS verified
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
    let items = rows
        .into_iter()
        .map(|row| tag_from_row(&repository, row))
        .collect::<Result<Vec<_>, _>>()?;
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
            "/{}/{}/compare/{}",
            repository.owner_login, repository.name, short_name
        ),
    })
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
