use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api_types::{normalize_pagination, ListEnvelope};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GistOwner {
    pub id: Uuid,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GistFile {
    pub id: Uuid,
    pub filename: String,
    pub language: Option<String>,
    pub size_bytes: i64,
    pub content_sha: String,
    pub content: String,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GistSummary {
    pub id: Uuid,
    pub description: Option<String>,
    pub is_public: bool,
    pub owner: GistOwner,
    pub files: Vec<GistFile>,
    pub comments_count: i64,
    pub stars_count: i64,
    pub forks_count: i64,
    pub clone_url: String,
    pub embed_url: String,
    pub href: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GistDetail {
    #[serde(flatten)]
    pub summary: GistSummary,
    pub comments: Vec<GistComment>,
    pub viewer: GistViewer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GistViewer {
    pub authenticated: bool,
    pub can_edit: bool,
    pub is_starred: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GistComment {
    pub id: Uuid,
    pub body: String,
    pub author: GistOwner,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GistRevision {
    pub id: Uuid,
    pub version: i64,
    pub description: Option<String>,
    pub files: Vec<GistFile>,
    pub author: GistOwner,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GistRevisionList {
    pub gist: GistSummary,
    pub revisions: Vec<GistRevision>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GistList {
    #[serde(flatten)]
    pub envelope: ListEnvelope<GistSummary>,
    pub scope: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GistFileInput {
    pub filename: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GistMutation {
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub files: Vec<GistFileInput>,
}

#[derive(Debug, thiserror::Error)]
pub enum GistError {
    #[error("gist was not found")]
    NotFound,
    #[error("you do not have permission to modify this gist")]
    Forbidden,
    #[error("{0}")]
    Validation(String),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Copy)]
pub struct GistListQuery<'a> {
    pub username: Option<&'a str>,
    pub scope: Option<&'a str>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn list_gists(
    pool: &PgPool,
    actor_user_id: Option<Uuid>,
    query: GistListQuery<'_>,
    app_url: &url::Url,
) -> Result<GistList, GistError> {
    let pagination = normalize_pagination(query.page, query.page_size);
    let offset = (pagination.page - 1) * pagination.page_size;
    let scope = query.scope.unwrap_or("public");

    let mut where_sql = String::from("WHERE (g.is_public = true");
    if actor_user_id.is_some() {
        where_sql.push_str(" OR g.owner_id = $1");
    }
    where_sql.push(')');
    if query.username.is_some() {
        where_sql.push_str(if actor_user_id.is_some() {
            " AND lower(u.username) = lower($2)"
        } else {
            " AND lower(u.username) = lower($1)"
        });
    } else if scope == "mine" {
        if actor_user_id.is_none() {
            return Ok(GistList {
                envelope: ListEnvelope {
                    items: vec![],
                    total: 0,
                    page: pagination.page,
                    page_size: pagination.page_size,
                },
                scope: scope.to_owned(),
            });
        }
        where_sql = String::from("WHERE g.owner_id = $1");
    }

    let count_sql = format!("SELECT count(*)::bigint AS total FROM gists g JOIN users u ON u.id = g.owner_id {where_sql}");
    let mut count_query = sqlx::query(&count_sql);
    if let Some(actor) = actor_user_id {
        count_query = count_query.bind(actor);
    }
    if let Some(username) = query.username {
        count_query = count_query.bind(username);
    }
    let total = count_query.fetch_one(pool).await?.get::<i64, _>("total");

    let list_sql = format!(
        "SELECT g.id FROM gists g JOIN users u ON u.id = g.owner_id {where_sql} ORDER BY g.updated_at DESC LIMIT $limit OFFSET $offset"
    )
    .replace("$limit", &(if actor_user_id.is_some() { if query.username.is_some() { 3 } else { 2 } } else if query.username.is_some() { 2 } else { 1 }).to_string())
    .replace("$offset", &(if actor_user_id.is_some() { if query.username.is_some() { 4 } else { 3 } } else if query.username.is_some() { 3 } else { 2 }).to_string());
    let mut list_query = sqlx::query(&list_sql);
    if let Some(actor) = actor_user_id {
        list_query = list_query.bind(actor);
    }
    if let Some(username) = query.username {
        list_query = list_query.bind(username);
    }
    list_query = list_query.bind(pagination.page_size).bind(offset);

    let rows = list_query.fetch_all(pool).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(gist_summary(pool, row.get("id"), app_url).await?);
    }

    Ok(GistList {
        envelope: ListEnvelope {
            items,
            total,
            page: pagination.page,
            page_size: pagination.page_size,
        },
        scope: scope.to_owned(),
    })
}

pub async fn create_gist(
    pool: &PgPool,
    actor_user_id: Uuid,
    input: GistMutation,
    app_url: &url::Url,
) -> Result<GistDetail, GistError> {
    let files = validate_files(input.files)?;
    let gist_id = sqlx::query(
        "INSERT INTO gists (owner_id, description, is_public, git_storage_path)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
    .bind(actor_user_id)
    .bind(trim_optional(input.description))
    .bind(input.is_public.unwrap_or(true))
    .bind(format!("gists/{actor_user_id}/{}", Uuid::new_v4()))
    .fetch_one(pool)
    .await?
    .get::<Uuid, _>("id");
    replace_files(pool, gist_id, &files).await?;
    insert_revision(pool, gist_id, actor_user_id, 1, None, &files).await?;
    get_gist(pool, gist_id, Some(actor_user_id), app_url).await
}

pub async fn update_gist(
    pool: &PgPool,
    actor_user_id: Uuid,
    gist_id: Uuid,
    input: GistMutation,
    app_url: &url::Url,
) -> Result<GistDetail, GistError> {
    let owner_id = gist_owner_id(pool, gist_id).await?;
    if owner_id != actor_user_id {
        return Err(GistError::Forbidden);
    }
    let files = validate_files(input.files)?;
    sqlx::query("UPDATE gists SET description = $1, is_public = $2 WHERE id = $3")
        .bind(trim_optional(input.description))
        .bind(input.is_public.unwrap_or(true))
        .bind(gist_id)
        .execute(pool)
        .await?;
    replace_files(pool, gist_id, &files).await?;
    let version = sqlx::query("SELECT coalesce(max(version), 0)::bigint + 1 AS version FROM gist_revisions WHERE gist_id = $1")
        .bind(gist_id)
        .fetch_one(pool)
        .await?
        .get::<i64, _>("version");
    insert_revision(pool, gist_id, actor_user_id, version, None, &files).await?;
    get_gist(pool, gist_id, Some(actor_user_id), app_url).await
}

pub async fn get_gist(
    pool: &PgPool,
    gist_id: Uuid,
    actor_user_id: Option<Uuid>,
    app_url: &url::Url,
) -> Result<GistDetail, GistError> {
    let summary = gist_summary(pool, gist_id, app_url).await?;
    if !summary.is_public && Some(summary.owner.id) != actor_user_id {
        return Err(GistError::NotFound);
    }
    let comments = gist_comments(pool, gist_id, app_url).await?;
    let is_starred = match actor_user_id {
        Some(actor) => sqlx::query("SELECT EXISTS (SELECT 1 FROM gist_stars WHERE gist_id = $1 AND user_id = $2) AS starred")
            .bind(gist_id)
            .bind(actor)
            .fetch_one(pool)
            .await?
            .get::<bool, _>("starred"),
        None => false,
    };
    Ok(GistDetail {
        viewer: GistViewer {
            authenticated: actor_user_id.is_some(),
            can_edit: Some(summary.owner.id) == actor_user_id,
            is_starred,
        },
        summary,
        comments,
    })
}

pub async fn gist_revisions(
    pool: &PgPool,
    gist_id: Uuid,
    actor_user_id: Option<Uuid>,
    app_url: &url::Url,
) -> Result<GistRevisionList, GistError> {
    let detail = get_gist(pool, gist_id, actor_user_id, app_url).await?;
    let rows = sqlx::query(
        "SELECT gr.id, gr.version, gr.description, gr.files_snapshot, gr.created_at,
                u.id AS author_id, u.username AS author_username,
                u.display_name AS author_display_name, u.avatar_url AS author_avatar_url
         FROM gist_revisions gr
         JOIN users u ON u.id = gr.author_user_id
         WHERE gr.gist_id = $1
         ORDER BY gr.version DESC",
    )
    .bind(gist_id)
    .fetch_all(pool)
    .await?;
    let revisions = rows
        .into_iter()
        .map(|row| GistRevision {
            id: row.get("id"),
            version: row.get("version"),
            description: row.get("description"),
            files: serde_json::from_value(row.get("files_snapshot")).unwrap_or_default(),
            author: owner_from_row(&row, "author", app_url),
            created_at: row.get("created_at"),
        })
        .collect();
    Ok(GistRevisionList {
        gist: detail.summary,
        revisions,
    })
}

pub async fn star_gist(
    pool: &PgPool,
    actor_user_id: Uuid,
    gist_id: Uuid,
    app_url: &url::Url,
) -> Result<GistDetail, GistError> {
    let _ = get_gist(pool, gist_id, Some(actor_user_id), app_url).await?;
    sqlx::query("INSERT INTO gist_stars (user_id, gist_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(actor_user_id)
        .bind(gist_id)
        .execute(pool)
        .await?;
    get_gist(pool, gist_id, Some(actor_user_id), app_url).await
}

pub async fn unstar_gist(
    pool: &PgPool,
    actor_user_id: Uuid,
    gist_id: Uuid,
    app_url: &url::Url,
) -> Result<GistDetail, GistError> {
    sqlx::query("DELETE FROM gist_stars WHERE user_id = $1 AND gist_id = $2")
        .bind(actor_user_id)
        .bind(gist_id)
        .execute(pool)
        .await?;
    get_gist(pool, gist_id, Some(actor_user_id), app_url).await
}

pub async fn fork_gist(
    pool: &PgPool,
    actor_user_id: Uuid,
    source_gist_id: Uuid,
    app_url: &url::Url,
) -> Result<GistDetail, GistError> {
    let source = get_gist(pool, source_gist_id, Some(actor_user_id), app_url).await?;
    let fork = create_gist(
        pool,
        actor_user_id,
        GistMutation {
            description: source.summary.description.clone(),
            is_public: Some(source.summary.is_public),
            files: source
                .summary
                .files
                .iter()
                .map(|file| GistFileInput {
                    filename: file.filename.clone(),
                    content: file.content.clone(),
                })
                .collect(),
        },
        app_url,
    )
    .await?;
    sqlx::query("UPDATE gists SET forked_from_gist_id = $1 WHERE id = $2")
        .bind(source_gist_id)
        .bind(fork.summary.id)
        .execute(pool)
        .await?;
    sqlx::query(
        "INSERT INTO gist_forks (source_gist_id, fork_gist_id, forked_by_user_id)
         VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(source_gist_id)
    .bind(fork.summary.id)
    .bind(actor_user_id)
    .execute(pool)
    .await?;
    get_gist(pool, fork.summary.id, Some(actor_user_id), app_url).await
}

async fn gist_owner_id(pool: &PgPool, gist_id: Uuid) -> Result<Uuid, GistError> {
    sqlx::query("SELECT owner_id FROM gists WHERE id = $1")
        .bind(gist_id)
        .fetch_optional(pool)
        .await?
        .map(|row| row.get("owner_id"))
        .ok_or(GistError::NotFound)
}

async fn gist_summary(
    pool: &PgPool,
    gist_id: Uuid,
    app_url: &url::Url,
) -> Result<GistSummary, GistError> {
    let row = sqlx::query(
        "SELECT g.id, g.description, g.is_public, g.created_at, g.updated_at,
                u.id AS owner_id, u.username AS owner_username,
                u.display_name AS owner_display_name, u.avatar_url AS owner_avatar_url,
                (SELECT count(*)::bigint FROM gist_comments gc WHERE gc.gist_id = g.id) AS comments_count,
                (SELECT count(*)::bigint FROM gist_stars gs WHERE gs.gist_id = g.id) AS stars_count,
                (SELECT count(*)::bigint FROM gist_forks gf WHERE gf.source_gist_id = g.id) AS forks_count
         FROM gists g
         JOIN users u ON u.id = g.owner_id
         WHERE g.id = $1",
    )
    .bind(gist_id)
    .fetch_optional(pool)
    .await?
    .ok_or(GistError::NotFound)?;
    let id: Uuid = row.get("id");
    let href = format!("/gist/{id}");
    Ok(GistSummary {
        id,
        description: row.get("description"),
        is_public: row.get("is_public"),
        owner: owner_from_row(&row, "owner", app_url),
        files: gist_files(pool, id).await?,
        comments_count: row.get("comments_count"),
        stars_count: row.get("stars_count"),
        forks_count: row.get("forks_count"),
        clone_url: app_url
            .join(&format!("gist/{id}.git"))
            .map(|url| url.to_string())
            .unwrap_or_else(|_| format!("/gist/{id}.git")),
        embed_url: app_url
            .join(&format!("api/gists/{id}/embed.js"))
            .map(|url| url.to_string())
            .unwrap_or_else(|_| format!("/api/gists/{id}/embed.js")),
        href,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

async fn gist_files(pool: &PgPool, gist_id: Uuid) -> Result<Vec<GistFile>, GistError> {
    Ok(sqlx::query(
        "SELECT id, filename, language, size_bytes, content_sha, content, position
         FROM gist_files
         WHERE gist_id = $1
         ORDER BY position ASC, filename ASC",
    )
    .bind(gist_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| GistFile {
        id: row.get("id"),
        filename: row.get("filename"),
        language: row.get("language"),
        size_bytes: row.get("size_bytes"),
        content_sha: row.get("content_sha"),
        content: row.get("content"),
        position: row.get("position"),
    })
    .collect())
}

async fn gist_comments(
    pool: &PgPool,
    gist_id: Uuid,
    app_url: &url::Url,
) -> Result<Vec<GistComment>, GistError> {
    Ok(sqlx::query(
        "SELECT gc.id, gc.body, gc.created_at, gc.updated_at,
                u.id AS author_id, u.username AS author_username,
                u.display_name AS author_display_name, u.avatar_url AS author_avatar_url
         FROM gist_comments gc
         JOIN users u ON u.id = gc.author_user_id
         WHERE gc.gist_id = $1
         ORDER BY gc.created_at ASC",
    )
    .bind(gist_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| GistComment {
        id: row.get("id"),
        body: row.get("body"),
        author: owner_from_row(&row, "author", app_url),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
    .collect())
}

async fn replace_files(pool: &PgPool, gist_id: Uuid, files: &[GistFile]) -> Result<(), GistError> {
    sqlx::query("DELETE FROM gist_files WHERE gist_id = $1")
        .bind(gist_id)
        .execute(pool)
        .await?;
    for file in files {
        sqlx::query(
            "INSERT INTO gist_files (gist_id, filename, language, size_bytes, content_sha, content, position)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(gist_id)
        .bind(&file.filename)
        .bind(&file.language)
        .bind(file.size_bytes)
        .bind(&file.content_sha)
        .bind(&file.content)
        .bind(file.position)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn insert_revision(
    pool: &PgPool,
    gist_id: Uuid,
    author_user_id: Uuid,
    version: i64,
    description: Option<String>,
    files: &[GistFile],
) -> Result<(), GistError> {
    sqlx::query(
        "INSERT INTO gist_revisions (gist_id, author_user_id, version, description, files_snapshot)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(gist_id)
    .bind(author_user_id)
    .bind(version)
    .bind(description)
    .bind(json!(files))
    .execute(pool)
    .await?;
    Ok(())
}

fn validate_files(files: Vec<GistFileInput>) -> Result<Vec<GistFile>, GistError> {
    if files.is_empty() {
        return Err(GistError::Validation("Add at least one file".to_owned()));
    }
    let mut seen = std::collections::BTreeSet::new();
    files
        .into_iter()
        .enumerate()
        .map(|(index, file)| {
            let filename = file.filename.trim().to_owned();
            if filename.is_empty() {
                return Err(GistError::Validation(
                    "File names cannot be blank".to_owned(),
                ));
            }
            if !seen.insert(filename.to_lowercase()) {
                return Err(GistError::Validation(
                    "File names must be unique".to_owned(),
                ));
            }
            let size_bytes = file.content.len() as i64;
            if size_bytes > 1_000_000 {
                return Err(GistError::Validation(
                    "Gist files must be 1 MB or smaller".to_owned(),
                ));
            }
            let content_sha = format!("{:x}", Sha256::digest(file.content.as_bytes()));
            Ok(GistFile {
                id: Uuid::new_v4(),
                language: language_for_filename(&filename),
                filename,
                size_bytes,
                content_sha,
                content: file.content,
                position: index as i32,
            })
        })
        .collect()
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim().to_owned();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn language_for_filename(filename: &str) -> Option<String> {
    let extension = filename.rsplit_once('.')?.1.to_lowercase();
    let language = match extension.as_str() {
        "rs" => "Rust",
        "ts" | "tsx" => "TypeScript",
        "js" | "jsx" => "JavaScript",
        "py" => "Python",
        "rb" => "Ruby",
        "go" => "Go",
        "md" | "mdx" => "Markdown",
        "json" => "JSON",
        "yml" | "yaml" => "YAML",
        "css" => "CSS",
        "html" => "HTML",
        "sql" => "SQL",
        "sh" => "Shell",
        _ => return None,
    };
    Some(language.to_owned())
}

fn owner_from_row(row: &sqlx::postgres::PgRow, prefix: &str, app_url: &url::Url) -> GistOwner {
    let login: String = row
        .try_get(format!("{prefix}_username").as_str())
        .ok()
        .flatten()
        .unwrap_or_else(|| "unknown".to_owned());
    GistOwner {
        id: row.get(format!("{prefix}_id").as_str()),
        login: login.clone(),
        name: row.get(format!("{prefix}_display_name").as_str()),
        avatar_url: row.get(format!("{prefix}_avatar_url").as_str()),
        href: app_url
            .join(&login)
            .map(|url| url.path().to_owned())
            .unwrap_or_else(|_| format!("/{login}")),
    }
}
