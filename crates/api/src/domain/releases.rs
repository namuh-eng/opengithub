use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
    #[error("markdown rendering failed")]
    Markdown,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
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

fn short_tag_name(tag_name: &str) -> String {
    clean_tag_name(tag_name)
}

fn short_oid(oid: &str) -> String {
    oid.chars().take(7).collect()
}
