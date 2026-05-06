use chrono::{DateTime, Utc};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use url::Url;
use uuid::Uuid;

use super::{
    markdown::{render_markdown, RenderMarkdownInput},
    repositories::{
        can_read_repository, can_write_repository, get_repository_by_owner_name,
        repository_permission_for_user, Repository, RepositoryError, RepositoryVisibility,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryWikiView {
    pub repository: WikiRepositorySummary,
    pub viewer: WikiViewer,
    pub state: WikiState,
    pub page: Option<WikiPageView>,
    pub pages: Vec<WikiPageSummary>,
    pub sidebar: Option<WikiRenderedBlock>,
    pub footer: Option<WikiRenderedBlock>,
    pub clone: WikiCloneInfo,
    pub links: WikiLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiRepositorySummary {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub visibility: String,
    pub default_branch: String,
    pub wiki_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiViewer {
    pub permission: Option<String>,
    pub can_read: bool,
    pub can_edit_wiki: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WikiStateKind {
    Ready,
    Empty,
    Disabled,
    MissingPage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiState {
    pub kind: WikiStateKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiPageView {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub path: String,
    pub href: String,
    pub revision: WikiRevisionSummary,
    pub markdown: String,
    pub html: String,
    pub content_sha: String,
    pub outline: Vec<WikiHeading>,
    pub edit_href: Option<String>,
    pub history_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiPageSummary {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub href: String,
    pub active: bool,
    pub has_outline: bool,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiRevisionSummary {
    pub id: Uuid,
    pub author: Option<WikiAuthor>,
    pub message: String,
    pub commit_oid: Option<String>,
    pub short_oid: Option<String>,
    pub created_at: DateTime<Utc>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiAuthor {
    pub id: Uuid,
    pub login: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiHeading {
    pub id: String,
    pub level: i32,
    pub text: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiRenderedBlock {
    pub title: String,
    pub slug: String,
    pub href: String,
    pub html: String,
    pub outline: Vec<WikiHeading>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiCloneInfo {
    pub https_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiLinks {
    pub home_href: String,
    pub new_page_href: Option<String>,
}

pub async fn repository_wiki_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Option<Uuid>,
    owner_login: &str,
    name: &str,
    slug: Option<&str>,
    app_url: &Url,
) -> Result<Option<RepositoryWikiView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    let can_read = repository_can_read(pool, &repository, actor_user_id).await?;
    if !can_read {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }

    let wiki_enabled = repository_wiki_enabled(pool, repository.id).await?;
    let can_edit_wiki = match actor_user_id {
        Some(user_id) => {
            can_write_repository(pool, &repository, user_id).await? && !repository.is_archived
        }
        None => false,
    };
    let permission = viewer_permission(pool, &repository, actor_user_id).await?;
    let repository_summary = WikiRepositorySummary {
        id: repository.id,
        owner_login: repository.owner_login.clone(),
        name: repository.name.clone(),
        visibility: repository.visibility.as_str().to_owned(),
        default_branch: repository.default_branch.clone(),
        wiki_enabled,
    };
    let viewer = WikiViewer {
        permission,
        can_read,
        can_edit_wiki,
    };
    let links = WikiLinks {
        home_href: wiki_home_href(&repository),
        new_page_href: can_edit_wiki.then(|| format!("{}/_new", wiki_home_href(&repository))),
    };
    let clone = WikiCloneInfo {
        https_url: wiki_clone_url(app_url, &repository),
    };

    if !wiki_enabled {
        return Ok(Some(RepositoryWikiView {
            repository: repository_summary,
            viewer,
            state: WikiState {
                kind: WikiStateKind::Disabled,
                message: "Wiki is disabled for this repository.".to_owned(),
            },
            page: None,
            pages: Vec::new(),
            sidebar: None,
            footer: None,
            clone,
            links,
        }));
    }

    let Some(wiki_repository_id) = wiki_repository_id(pool, repository.id).await? else {
        return Ok(Some(RepositoryWikiView {
            repository: repository_summary,
            viewer,
            state: WikiState {
                kind: WikiStateKind::Empty,
                message: "This repository wiki has no pages yet.".to_owned(),
            },
            page: None,
            pages: Vec::new(),
            sidebar: None,
            footer: None,
            clone,
            links,
        }));
    };

    let pages = wiki_page_summaries(pool, wiki_repository_id, slug).await?;
    let page = wiki_page(pool, &repository, wiki_repository_id, slug, can_edit_wiki).await?;
    let sidebar = wiki_special_block(
        pool,
        &repository,
        wiki_repository_id,
        WikiSpecialPage::Sidebar,
    )
    .await?;
    let footer = wiki_special_block(
        pool,
        &repository,
        wiki_repository_id,
        WikiSpecialPage::Footer,
    )
    .await?;
    let state = if page.is_some() {
        WikiState {
            kind: WikiStateKind::Ready,
            message: "Wiki page is ready.".to_owned(),
        }
    } else {
        WikiState {
            kind: WikiStateKind::MissingPage,
            message: "Wiki page was not found.".to_owned(),
        }
    };

    Ok(Some(RepositoryWikiView {
        repository: repository_summary,
        viewer,
        state,
        page,
        pages,
        sidebar,
        footer,
        clone,
        links,
    }))
}

async fn repository_can_read(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Option<Uuid>,
) -> Result<bool, RepositoryError> {
    match actor_user_id {
        Some(user_id) => can_read_repository(pool, repository, user_id).await,
        None => Ok(repository.visibility == RepositoryVisibility::Public),
    }
}

async fn viewer_permission(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Option<Uuid>,
) -> Result<Option<String>, RepositoryError> {
    let Some(user_id) = actor_user_id else {
        return Ok(
            (repository.visibility == RepositoryVisibility::Public).then(|| "read".to_owned())
        );
    };
    if repository.owner_user_id == Some(user_id) {
        return Ok(Some("owner".to_owned()));
    }
    Ok(repository_permission_for_user(pool, repository.id, user_id)
        .await?
        .map(|permission| permission.role.as_str().to_owned())
        .or_else(|| {
            (repository.visibility == RepositoryVisibility::Public).then(|| "read".to_owned())
        }))
}

async fn repository_wiki_enabled(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<bool, RepositoryError> {
    sqlx::query_scalar::<_, bool>(
        "SELECT COALESCE(wiki_enabled, true) FROM repositories WHERE id = $1",
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await
    .map_err(RepositoryError::from)
}

async fn wiki_repository_id(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Option<Uuid>, RepositoryError> {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM wiki_repositories WHERE repository_id = $1")
        .bind(repository_id)
        .fetch_optional(pool)
        .await
        .map_err(RepositoryError::from)
}

async fn wiki_page_summaries(
    pool: &PgPool,
    wiki_repository_id: Uuid,
    active_slug: Option<&str>,
) -> Result<Vec<WikiPageSummary>, RepositoryError> {
    let active_slug = active_slug.map(normalize_slug);
    let rows = sqlx::query(
        r#"
        SELECT wiki_pages.id,
               wiki_pages.title,
               wiki_pages.slug,
               wiki_pages.updated_at,
               EXISTS (
                   SELECT 1
                   FROM wiki_page_toc_cache
                   WHERE wiki_page_toc_cache.page_id = wiki_pages.id
               ) AS has_outline
        FROM wiki_pages
        WHERE wiki_pages.wiki_repository_id = $1
          AND wiki_pages.is_sidebar = false
          AND wiki_pages.is_footer = false
        ORDER BY
          CASE WHEN lower(wiki_pages.slug) = 'home' THEN 0 ELSE 1 END,
          wiki_pages.position ASC,
          lower(wiki_pages.title) ASC
        LIMIT 50
        "#,
    )
    .bind(wiki_repository_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let slug: String = row.get("slug");
            WikiPageSummary {
                id: row.get("id"),
                title: row.get("title"),
                href: wiki_page_href_from_parts(&slug),
                active: active_slug
                    .as_deref()
                    .map(|active| active.eq_ignore_ascii_case(&slug))
                    .unwrap_or_else(|| slug.eq_ignore_ascii_case("home")),
                slug,
                has_outline: row.get("has_outline"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect())
}

async fn wiki_page(
    pool: &PgPool,
    repository: &Repository,
    wiki_repository_id: Uuid,
    slug: Option<&str>,
    can_edit: bool,
) -> Result<Option<WikiPageView>, RepositoryError> {
    let row = match slug {
        Some(slug) => {
            wiki_page_row_by_slug(pool, wiki_repository_id, &normalize_slug(slug)).await?
        }
        None => wiki_home_page_row(pool, wiki_repository_id).await?,
    };
    let Some(row) = row else {
        return Ok(None);
    };
    let page_id: Uuid = row.get("page_id");
    let title: String = row.get("title");
    let slug: String = row.get("slug");
    let path: String = row.get("path");
    let revision_id: Uuid = row.get("revision_id");
    let markdown: String = row.get("markdown");
    let rendered = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            markdown: markdown.clone(),
            repository_id: Some(repository.id),
            owner: Some(repository.owner_login.clone()),
            repo: Some(repository.name.clone()),
            ref_name: Some(format!("wiki:{revision_id}")),
            enable_task_toggles: Some(false),
        },
    )
    .await
    .map_err(markdown_error)?;
    let outline = wiki_heading_outline(&rendered.html);
    cache_outline(pool, page_id, revision_id, &outline).await?;

    Ok(Some(WikiPageView {
        id: page_id,
        title,
        slug: slug.clone(),
        path,
        href: wiki_page_href(repository, &slug),
        revision: revision_from_row(repository, &slug, &row),
        markdown,
        html: rendered.html,
        content_sha: rendered.content_sha,
        outline,
        edit_href: can_edit.then(|| format!("{}/_edit", wiki_page_href(repository, &slug))),
        history_href: format!("{}/_history", wiki_page_href(repository, &slug)),
    }))
}

async fn wiki_home_page_row(
    pool: &PgPool,
    wiki_repository_id: Uuid,
) -> Result<Option<sqlx::postgres::PgRow>, RepositoryError> {
    sqlx::query(&page_select_sql(
        r#"
        WHERE wiki_pages.wiki_repository_id = $1
          AND wiki_pages.is_sidebar = false
          AND wiki_pages.is_footer = false
        ORDER BY
          CASE WHEN lower(wiki_pages.slug) = 'home' THEN 0 ELSE 1 END,
          wiki_pages.position ASC,
          lower(wiki_pages.title) ASC
        LIMIT 1
        "#,
    ))
    .bind(wiki_repository_id)
    .fetch_optional(pool)
    .await
    .map_err(RepositoryError::from)
}

async fn wiki_page_row_by_slug(
    pool: &PgPool,
    wiki_repository_id: Uuid,
    slug: &str,
) -> Result<Option<sqlx::postgres::PgRow>, RepositoryError> {
    sqlx::query(&page_select_sql(
        r#"
        WHERE wiki_pages.wiki_repository_id = $1
          AND lower(wiki_pages.slug) = lower($2)
        LIMIT 1
        "#,
    ))
    .bind(wiki_repository_id)
    .bind(slug)
    .fetch_optional(pool)
    .await
    .map_err(RepositoryError::from)
}

fn page_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT wiki_pages.id AS page_id,
               wiki_pages.title,
               wiki_pages.slug,
               wiki_pages.path,
               wiki_page_revisions.id AS revision_id,
               wiki_page_revisions.message,
               wiki_page_revisions.commit_oid,
               wiki_page_revisions.markdown,
               wiki_page_revisions.created_at,
               users.id AS author_id,
               COALESCE(NULLIF(users.username, ''), users.email) AS author_login,
               users.display_name AS author_display_name,
               users.avatar_url AS author_avatar_url
        FROM wiki_pages
        JOIN wiki_page_revisions
          ON wiki_page_revisions.id = wiki_pages.latest_revision_id
        LEFT JOIN users ON users.id = wiki_page_revisions.author_user_id
        {where_clause}
        "#
    )
}

enum WikiSpecialPage {
    Sidebar,
    Footer,
}

async fn wiki_special_block(
    pool: &PgPool,
    repository: &Repository,
    wiki_repository_id: Uuid,
    special: WikiSpecialPage,
) -> Result<Option<WikiRenderedBlock>, RepositoryError> {
    let (slug, flag_column) = match special {
        WikiSpecialPage::Sidebar => ("_sidebar", "is_sidebar"),
        WikiSpecialPage::Footer => ("_footer", "is_footer"),
    };
    let row = sqlx::query(&format!(
        "{} WHERE wiki_pages.wiki_repository_id = $1 AND (wiki_pages.{flag_column} = true OR lower(wiki_pages.slug) = $2) LIMIT 1",
        page_select_sql("")
    ))
    .bind(wiki_repository_id)
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let markdown: String = row.get("markdown");
    let title: String = row.get("title");
    let slug: String = row.get("slug");
    let revision_id: Uuid = row.get("revision_id");
    let rendered = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            markdown,
            repository_id: Some(repository.id),
            owner: Some(repository.owner_login.clone()),
            repo: Some(repository.name.clone()),
            ref_name: Some(format!("wiki:{revision_id}")),
            enable_task_toggles: Some(false),
        },
    )
    .await
    .map_err(markdown_error)?;
    Ok(Some(WikiRenderedBlock {
        title,
        href: wiki_page_href(repository, &slug),
        slug,
        outline: wiki_heading_outline(&rendered.html),
        html: rendered.html,
    }))
}

async fn cache_outline(
    pool: &PgPool,
    page_id: Uuid,
    revision_id: Uuid,
    outline: &[WikiHeading],
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO wiki_page_toc_cache (page_id, revision_id, outline)
        VALUES ($1, $2, $3)
        ON CONFLICT (revision_id)
        DO UPDATE SET outline = EXCLUDED.outline, generated_at = now()
        "#,
    )
    .bind(page_id)
    .bind(revision_id)
    .bind(serde_json::to_value(outline).unwrap_or_else(|_| serde_json::json!([])))
    .execute(pool)
    .await?;
    Ok(())
}

fn revision_from_row(
    repository: &Repository,
    page_slug: &str,
    row: &sqlx::postgres::PgRow,
) -> WikiRevisionSummary {
    let commit_oid: Option<String> = row.get("commit_oid");
    let revision_id: Uuid = row.get("revision_id");
    WikiRevisionSummary {
        id: revision_id,
        author: row.get::<Option<Uuid>, _>("author_id").map(|id| {
            let login: String = row.get("author_login");
            WikiAuthor {
                id,
                href: format!("/{}", percent_encode_segment(&login)),
                login,
                display_name: row.get("author_display_name"),
                avatar_url: row.get("author_avatar_url"),
            }
        }),
        message: row.get("message"),
        short_oid: commit_oid
            .as_ref()
            .map(|oid| oid.chars().take(7).collect::<String>()),
        commit_oid,
        created_at: row.get("created_at"),
        href: format!(
            "{}/_compare/{}",
            wiki_page_href(repository, page_slug),
            revision_id
        ),
    }
}

fn wiki_heading_outline(html: &str) -> Vec<WikiHeading> {
    Regex::new(r#"<h([1-6]) id="([^"]+)">(.*?)</h[1-6]>"#)
        .expect("wiki heading outline regex")
        .captures_iter(html)
        .map(|captures| {
            let level = captures[1].parse::<i32>().unwrap_or(1);
            let id = captures[2].to_owned();
            let text = strip_tags(&captures[3])
                .trim()
                .trim_start_matches('#')
                .trim()
                .to_owned();
            WikiHeading {
                href: format!("#{id}"),
                id,
                level,
                text,
            }
        })
        .collect()
}

fn strip_tags(value: &str) -> String {
    Regex::new(r"<[^>]+>")
        .expect("wiki strip tags regex")
        .replace_all(value, |captures: &Captures<'_>| {
            if captures[0].starts_with("</") {
                " "
            } else {
                ""
            }
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_slug(slug: &str) -> String {
    slug.trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != "." && *segment != "..")
        .collect::<Vec<_>>()
        .join("/")
}

fn wiki_clone_url(app_url: &Url, repository: &Repository) -> String {
    let mut base = app_url.clone();
    base.set_path("");
    base.set_query(None);
    base.set_fragment(None);
    format!(
        "{}{}/{}.wiki.git",
        base.as_str().trim_end_matches('/'),
        percent_encode_segment(&repository.owner_login),
        percent_encode_segment(&repository.name)
    )
}

fn wiki_home_href(repository: &Repository) -> String {
    format!("/{}/{}/wiki", repository.owner_login, repository.name)
}

fn wiki_page_href(repository: &Repository, slug: &str) -> String {
    if slug.eq_ignore_ascii_case("home") {
        return wiki_home_href(repository);
    }
    format!(
        "{}/{}",
        wiki_home_href(repository),
        percent_encode_path(slug)
    )
}

fn wiki_page_href_from_parts(slug: &str) -> String {
    if slug.eq_ignore_ascii_case("home") {
        "/wiki".to_owned()
    } else {
        format!("/wiki/{}", percent_encode_path(slug))
    }
}

fn percent_encode_path(path: &str) -> String {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .map(percent_encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}

fn percent_encode_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn markdown_error(error: super::markdown::MarkdownError) -> RepositoryError {
    match error {
        super::markdown::MarkdownError::Sqlx(error) => RepositoryError::Sqlx(error),
        super::markdown::MarkdownError::TooLarge | super::markdown::MarkdownError::TaskNotFound => {
            RepositoryError::InvalidSecurityPolicy("wiki markdown could not be rendered".to_owned())
        }
    }
}

pub fn wiki_content_sha(markdown: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(markdown.as_bytes());
    format!("{:x}", hasher.finalize())
}
