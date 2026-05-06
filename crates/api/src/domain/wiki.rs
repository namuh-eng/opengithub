use chrono::{DateTime, Utc};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use url::Url;
use uuid::Uuid;

const MARKDOWN_MODE: &str = "markdown";

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiPagesIndex {
    pub repository: WikiRepositorySummary,
    pub viewer: WikiViewer,
    pub pages: Vec<WikiPageSummary>,
    pub links: WikiLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiPageEditView {
    pub repository: WikiRepositorySummary,
    pub viewer: WikiViewer,
    pub page: WikiEditablePage,
    pub supported_formats: Vec<SupportedMarkupFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiEditablePage {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub path: String,
    pub markdown: String,
    pub latest_revision_id: Uuid,
    pub edit_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SupportedMarkupFormat {
    pub mode: String,
    pub label: String,
    pub extension: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiPageSaveRequest {
    pub title: String,
    pub markdown: String,
    pub message: String,
    #[serde(default)]
    pub edit_mode: Option<String>,
    #[serde(default)]
    pub expected_revision_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiPagePreviewRequest {
    pub markdown: String,
    #[serde(default)]
    pub edit_mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WikiImageReference {
    source_url: String,
    alt_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiPreviewResult {
    pub html: String,
    pub content_sha: String,
    pub outline: Vec<WikiHeading>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiPageMutationResult {
    pub page: WikiPageView,
    pub git_commit: WikiGitCommitSummary,
    pub redirect_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiGitCommitSummary {
    pub id: Uuid,
    pub oid: String,
    pub short_oid: String,
    pub branch: String,
    pub message: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiHistoryView {
    pub repository: WikiRepositorySummary,
    pub viewer: WikiViewer,
    pub scope: WikiHistoryScope,
    pub revisions: Vec<WikiHistoryRevisionRow>,
    pub pagination: WikiHistoryPagination,
    pub links: WikiHistoryLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiHistoryScope {
    pub kind: WikiHistoryScopeKind,
    pub page: Option<WikiPageSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WikiHistoryScopeKind {
    AllPages,
    Page,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiHistoryRevisionRow {
    pub id: Uuid,
    pub page_id: Uuid,
    pub page_title: String,
    pub page_slug: String,
    pub page_href: String,
    pub author: Option<WikiAuthor>,
    pub message: String,
    pub commit_oid: Option<String>,
    pub short_oid: Option<String>,
    pub created_at: DateTime<Utc>,
    pub href: String,
    pub revision_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiHistoryPagination {
    pub page: i64,
    pub page_size: i64,
    pub has_newer: bool,
    pub has_older: bool,
    pub newer_href: Option<String>,
    pub older_href: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiHistoryLinks {
    pub home_href: String,
    pub pages_href: String,
    pub history_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiRevisionView {
    pub repository: WikiRepositorySummary,
    pub viewer: WikiViewer,
    pub page: WikiPageView,
    pub revision_context: WikiRevisionContext,
    pub pages: Vec<WikiPageSummary>,
    pub links: WikiRevisionLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiRevisionContext {
    pub selected_revision: WikiRevisionSummary,
    pub latest_href: String,
    pub history_href: String,
    pub previous_revision_href: Option<String>,
    pub next_revision_href: Option<String>,
    pub is_latest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WikiRevisionLinks {
    pub home_href: String,
    pub pages_href: String,
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

pub async fn repository_wiki_history_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Option<Uuid>,
    owner_login: &str,
    name: &str,
    slug: Option<&str>,
    page: i64,
    page_size: i64,
) -> Result<Option<WikiHistoryView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    let can_read = repository_can_read(pool, &repository, actor_user_id).await?;
    if !can_read {
        return Ok(None);
    }
    let permission = viewer_permission(pool, &repository, actor_user_id).await?;
    let can_edit_wiki = match actor_user_id {
        Some(user_id) => can_write_repository(pool, &repository, user_id).await?,
        None => false,
    };
    let repository_summary = WikiRepositorySummary {
        id: repository.id,
        owner_login: repository.owner_login.clone(),
        name: repository.name.clone(),
        visibility: repository.visibility.as_str().to_owned(),
        default_branch: repository.default_branch.clone(),
        wiki_enabled: true,
    };
    let viewer = WikiViewer {
        permission,
        can_read: true,
        can_edit_wiki,
    };

    if !repository_wiki_enabled(pool, repository.id).await? {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "Wiki is disabled for this repository.".to_owned(),
        ));
    }
    let Some(wiki_repository_id) = wiki_repository_id(pool, repository.id).await? else {
        return Ok(Some(WikiHistoryView {
            repository: repository_summary,
            viewer,
            scope: WikiHistoryScope {
                kind: WikiHistoryScopeKind::AllPages,
                page: None,
            },
            revisions: Vec::new(),
            pagination: WikiHistoryPagination {
                page,
                page_size,
                has_newer: page > 1,
                has_older: false,
                newer_href: (page > 1)
                    .then(|| wiki_history_href(&repository, None, page - 1, page_size)),
                older_href: None,
            },
            links: WikiHistoryLinks {
                home_href: wiki_home_href(&repository),
                pages_href: format!("{}/_pages", wiki_home_href(&repository)),
                history_href: wiki_history_href(&repository, None, 1, page_size),
            },
        }));
    };

    let scoped_page = match slug {
        Some(slug) => {
            let Some(row) =
                wiki_page_summary_by_slug(pool, &repository, wiki_repository_id, slug).await?
            else {
                return Err(RepositoryError::InvalidSecurityPolicy(
                    "Wiki page was not found.".to_owned(),
                ));
            };
            Some(row)
        }
        None => None,
    };
    let offset = (page - 1) * page_size;
    let mut query = String::from(
        r#"
        SELECT wiki_page_revisions.id AS revision_id,
               wiki_page_revisions.page_id,
               wiki_pages.title AS page_title,
               wiki_pages.slug AS page_slug,
               wiki_page_revisions.message,
               wiki_page_revisions.commit_oid,
               wiki_page_revisions.created_at,
               users.id AS author_id,
               COALESCE(NULLIF(users.username, ''), users.email) AS author_login,
               users.display_name AS author_display_name,
               users.avatar_url AS author_avatar_url
        FROM wiki_page_revisions
        JOIN wiki_pages ON wiki_pages.id = wiki_page_revisions.page_id
        LEFT JOIN users ON users.id = wiki_page_revisions.author_user_id
        WHERE wiki_pages.wiki_repository_id = $1
          AND wiki_pages.is_sidebar = false
          AND wiki_pages.is_footer = false
        "#,
    );
    let revision_page_filter = scoped_page.is_some();
    if revision_page_filter {
        query.push_str(" AND wiki_pages.id = $2");
    }
    query.push_str(if revision_page_filter {
        r#"
            ORDER BY wiki_page_revisions.created_at DESC, wiki_page_revisions.id DESC
            LIMIT $3 OFFSET $4
            "#
    } else {
        r#"
            ORDER BY wiki_page_revisions.created_at DESC, wiki_page_revisions.id DESC
            LIMIT $2 OFFSET $3
            "#
    });

    let limit = page_size + 1;
    let mut sql = sqlx::query(&query).bind(wiki_repository_id);
    if let Some(scoped_page) = &scoped_page {
        sql = sql.bind(scoped_page.id);
    }
    let rows = sql
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(RepositoryError::from)?;
    let has_older = rows.len() as i64 > page_size;
    let revisions = rows
        .into_iter()
        .take(page_size as usize)
        .map(|row| {
            let page_slug: String = row.get("page_slug");
            let commit_oid: Option<String> = row.get("commit_oid");
            let href = wiki_revision_history_href(&repository, &page_slug, commit_oid.as_deref());
            WikiHistoryRevisionRow {
                id: row.get("revision_id"),
                page_id: row.get("page_id"),
                page_title: row.get("page_title"),
                page_href: wiki_page_href(&repository, &page_slug),
                revision_href: href.clone(),
                page_slug,
                author: author_from_row(&row),
                message: row.get("message"),
                short_oid: commit_oid
                    .as_ref()
                    .map(|oid| oid.chars().take(7).collect::<String>()),
                commit_oid,
                created_at: row.get("created_at"),
                href,
            }
        })
        .collect::<Vec<_>>();
    let scope_slug = scoped_page.as_ref().map(|page| page.slug.clone());

    Ok(Some(WikiHistoryView {
        repository: repository_summary,
        viewer,
        scope: WikiHistoryScope {
            kind: if scoped_page.is_some() {
                WikiHistoryScopeKind::Page
            } else {
                WikiHistoryScopeKind::AllPages
            },
            page: scoped_page,
        },
        revisions,
        pagination: WikiHistoryPagination {
            page,
            page_size,
            has_newer: page > 1,
            has_older,
            newer_href: (page > 1).then(|| {
                wiki_history_href(&repository, scope_slug.as_deref(), page - 1, page_size)
            }),
            older_href: has_older.then(|| {
                wiki_history_href(&repository, scope_slug.as_deref(), page + 1, page_size)
            }),
        },
        links: WikiHistoryLinks {
            home_href: wiki_home_href(&repository),
            pages_href: format!("{}/_pages", wiki_home_href(&repository)),
            history_href: wiki_history_href(&repository, scope_slug.as_deref(), 1, page_size),
        },
    }))
}

pub async fn repository_wiki_revision_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Option<Uuid>,
    owner_login: &str,
    name: &str,
    slug: &str,
    revision: &str,
) -> Result<Option<WikiRevisionView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    let can_read = repository_can_read(pool, &repository, actor_user_id).await?;
    if !can_read {
        return Ok(None);
    }
    if !repository_wiki_enabled(pool, repository.id).await? {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "Wiki is disabled for this repository.".to_owned(),
        ));
    }
    let Some(wiki_repository_id) = wiki_repository_id(pool, repository.id).await? else {
        return Err(RepositoryError::NotFound);
    };
    let Some(page_summary) =
        wiki_page_summary_by_slug(pool, &repository, wiki_repository_id, slug).await?
    else {
        return Err(RepositoryError::NotFound);
    };

    let row = wiki_revision_page_row(pool, page_summary.id, revision).await?;
    let page = wiki_page_from_row(pool, &repository, row, false).await?;
    let pages = wiki_page_summaries(pool, wiki_repository_id, Some(&page.slug)).await?;
    let permission = viewer_permission(pool, &repository, actor_user_id).await?;
    let can_edit_wiki = match actor_user_id {
        Some(user_id) => {
            can_write_repository(pool, &repository, user_id).await? && !repository.is_archived
        }
        None => false,
    };
    let previous_revision_href =
        adjacent_revision_href(pool, &repository, page.id, page.revision.created_at, false).await?;
    let next_revision_href =
        adjacent_revision_href(pool, &repository, page.id, page.revision.created_at, true).await?;
    let is_latest = next_revision_href.is_none();

    Ok(Some(WikiRevisionView {
        repository: WikiRepositorySummary {
            id: repository.id,
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            visibility: repository.visibility.as_str().to_owned(),
            default_branch: repository.default_branch.clone(),
            wiki_enabled: true,
        },
        viewer: WikiViewer {
            permission,
            can_read: true,
            can_edit_wiki,
        },
        revision_context: WikiRevisionContext {
            selected_revision: page.revision.clone(),
            latest_href: page.href.clone(),
            history_href: page.history_href.clone(),
            previous_revision_href,
            next_revision_href,
            is_latest,
        },
        page,
        pages,
        links: WikiRevisionLinks {
            home_href: wiki_home_href(&repository),
            pages_href: format!("{}/_pages", wiki_home_href(&repository)),
        },
    }))
}

pub async fn repository_wiki_pages_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<WikiPagesIndex>, RepositoryError> {
    let Some((repository, repository_summary, viewer, links)) =
        editable_wiki_context(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    let wiki_repository_id = ensure_wiki_repository(pool, &repository).await?;
    let pages = wiki_page_summaries(pool, wiki_repository_id, None).await?;
    Ok(Some(WikiPagesIndex {
        repository: repository_summary,
        viewer,
        pages,
        links,
    }))
}

pub async fn repository_wiki_edit_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    slug: &str,
) -> Result<Option<WikiPageEditView>, RepositoryError> {
    let Some((repository, repository_summary, viewer, _links)) =
        editable_wiki_context(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    let Some(wiki_repository_id) = wiki_repository_id(pool, repository.id).await? else {
        return Err(RepositoryError::NotFound);
    };
    let row = wiki_page_row_by_slug(pool, wiki_repository_id, &normalize_slug(slug))
        .await?
        .ok_or(RepositoryError::NotFound)?;
    Ok(Some(WikiPageEditView {
        repository: repository_summary,
        viewer,
        page: WikiEditablePage {
            id: row.get("page_id"),
            title: row.get("title"),
            slug: row.get("slug"),
            path: row.get("path"),
            markdown: row.get("markdown"),
            latest_revision_id: row.get("revision_id"),
            edit_mode: MARKDOWN_MODE.to_owned(),
        },
        supported_formats: supported_markup_formats(),
    }))
}

pub async fn preview_repository_wiki_page_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    request: WikiPagePreviewRequest,
) -> Result<Option<WikiPreviewResult>, RepositoryError> {
    let Some((repository, _repository_summary, _viewer, _links)) =
        editable_wiki_context(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    validate_edit_mode(request.edit_mode.as_deref())?;
    if request.markdown.trim().is_empty() {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "wiki page body is required".to_owned(),
        ));
    }
    let rendered = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            markdown: request.markdown,
            repository_id: Some(repository.id),
            owner: Some(repository.owner_login),
            repo: Some(repository.name),
            ref_name: Some("wiki:preview".to_owned()),
            enable_task_toggles: Some(false),
        },
    )
    .await
    .map_err(markdown_error)?;
    Ok(Some(WikiPreviewResult {
        outline: wiki_heading_outline(&rendered.html),
        content_sha: rendered.content_sha,
        html: rendered.html,
    }))
}

pub async fn create_repository_wiki_page_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    request: WikiPageSaveRequest,
) -> Result<Option<WikiPageMutationResult>, RepositoryError> {
    save_wiki_page(pool, actor_user_id, owner_login, name, None, request).await
}

pub async fn update_repository_wiki_page_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    slug: &str,
    request: WikiPageSaveRequest,
) -> Result<Option<WikiPageMutationResult>, RepositoryError> {
    save_wiki_page(pool, actor_user_id, owner_login, name, Some(slug), request).await
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

async fn editable_wiki_context(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<(Repository, WikiRepositorySummary, WikiViewer, WikiLinks)>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    if !repository_wiki_enabled(pool, repository.id).await? {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "Wiki is disabled for this repository.".to_owned(),
        ));
    }
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    if !can_write_repository(pool, &repository, actor_user_id).await? {
        return Err(RepositoryError::PermissionDenied);
    }
    let permission = viewer_permission(pool, &repository, Some(actor_user_id)).await?;
    let repository_summary = WikiRepositorySummary {
        id: repository.id,
        owner_login: repository.owner_login.clone(),
        name: repository.name.clone(),
        visibility: repository.visibility.as_str().to_owned(),
        default_branch: repository.default_branch.clone(),
        wiki_enabled: true,
    };
    let viewer = WikiViewer {
        permission,
        can_read: true,
        can_edit_wiki: true,
    };
    let links = WikiLinks {
        home_href: wiki_home_href(&repository),
        new_page_href: Some(format!("{}/_new", wiki_home_href(&repository))),
    };
    Ok(Some((repository, repository_summary, viewer, links)))
}

async fn ensure_wiki_repository(
    pool: &PgPool,
    repository: &Repository,
) -> Result<Uuid, RepositoryError> {
    let storage_path = wiki_storage_path(repository);
    sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO wiki_repositories (repository_id, git_storage_kind, git_storage_path, default_branch)
        VALUES ($1, 'local_bare', $2, 'master')
        ON CONFLICT (repository_id)
        DO UPDATE SET git_storage_path = COALESCE(wiki_repositories.git_storage_path, EXCLUDED.git_storage_path)
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(storage_path.to_string_lossy().to_string())
    .fetch_one(pool)
    .await
    .map_err(RepositoryError::from)
}

async fn save_wiki_page(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    existing_slug: Option<&str>,
    request: WikiPageSaveRequest,
) -> Result<Option<WikiPageMutationResult>, RepositoryError> {
    let Some((repository, _repository_summary, _viewer, _links)) =
        editable_wiki_context(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    validate_edit_mode(request.edit_mode.as_deref())?;
    let title = validate_title(&request.title)?;
    let slug = title_to_slug(&title)?;
    let markdown = validate_markdown(&request.markdown)?;
    let image_references = extract_image_references(&markdown)?;
    let message = validate_commit_message(&request.message)?;
    let wiki_repository_id = ensure_wiki_repository(pool, &repository).await?;
    let default_branch: String =
        sqlx::query_scalar("SELECT default_branch FROM wiki_repositories WHERE id = $1")
            .bind(wiki_repository_id)
            .fetch_one(pool)
            .await?;

    let mut tx = pool.begin().await?;
    let existing = match existing_slug {
        Some(slug) => {
            sqlx::query(
                r#"
                SELECT wiki_pages.id, wiki_pages.latest_revision_id
                FROM wiki_pages
                WHERE wiki_pages.wiki_repository_id = $1 AND lower(wiki_pages.slug) = lower($2)
                LIMIT 1
                "#,
            )
            .bind(wiki_repository_id)
            .bind(normalize_slug(slug))
            .fetch_optional(&mut *tx)
            .await?
        }
        None => None,
    };
    if existing_slug.is_some() && existing.is_none() {
        return Err(RepositoryError::NotFound);
    }
    if let Some(row) = existing.as_ref() {
        let latest_revision_id: Uuid = row.get("latest_revision_id");
        if request
            .expected_revision_id
            .is_some_and(|expected| expected != latest_revision_id)
        {
            return Err(RepositoryError::SecurityPolicyConflict);
        }
    }
    if existing_slug.is_none() {
        let duplicate = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM wiki_pages
                WHERE wiki_repository_id = $1 AND lower(slug) = lower($2)
            )
            "#,
        )
        .bind(wiki_repository_id)
        .bind(&slug)
        .fetch_one(&mut *tx)
        .await?;
        if duplicate {
            return Err(RepositoryError::SecurityPolicyConflict);
        }
    } else if let Some(row) = existing.as_ref() {
        let page_id: Uuid = row.get("id");
        let duplicate = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM wiki_pages
                WHERE wiki_repository_id = $1
                  AND lower(slug) = lower($2)
                  AND id <> $3
            )
            "#,
        )
        .bind(wiki_repository_id)
        .bind(&slug)
        .bind(page_id)
        .fetch_one(&mut *tx)
        .await?;
        if duplicate {
            return Err(RepositoryError::SecurityPolicyConflict);
        }
    }

    let path = wiki_path_from_slug(&slug);
    let (page_id, page_slug) = if let Some(row) = existing {
        let page_id: Uuid = row.get("id");
        sqlx::query(
            r#"
            UPDATE wiki_pages
            SET title = $1,
                slug = $2,
                path = $3,
                is_sidebar = lower($2) = '_sidebar',
                is_footer = lower($2) = '_footer'
            WHERE id = $4
            "#,
        )
        .bind(&title)
        .bind(&slug)
        .bind(&path)
        .bind(page_id)
        .execute(&mut *tx)
        .await?;
        (page_id, slug)
    } else {
        let page_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO wiki_pages (wiki_repository_id, title, slug, path, is_sidebar, is_footer)
            VALUES ($1, $2, $3, $4, lower($3) = '_sidebar', lower($3) = '_footer')
            RETURNING id
            "#,
        )
        .bind(wiki_repository_id)
        .bind(&title)
        .bind(&slug)
        .bind(&path)
        .fetch_one(&mut *tx)
        .await?;
        (page_id, slug)
    };
    let content_sha = wiki_content_sha(&markdown);
    let commit_oid = wiki_commit_oid(&repository, &page_slug, &markdown, &message);
    let revision_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO wiki_page_revisions (page_id, author_user_id, commit_oid, message, markdown, content_sha)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(page_id)
    .bind(actor_user_id)
    .bind(&commit_oid)
    .bind(&message)
    .bind(&markdown)
    .bind(&content_sha)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("UPDATE wiki_pages SET latest_revision_id = $1 WHERE id = $2")
        .bind(revision_id)
        .bind(page_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM wiki_page_toc_cache WHERE page_id = $1")
        .bind(page_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM wiki_assets WHERE page_id = $1")
        .bind(page_id)
        .execute(&mut *tx)
        .await?;
    for image in &image_references {
        sqlx::query(
            r#"
            INSERT INTO wiki_assets (wiki_repository_id, page_id, revision_id, source_url, alt_text, storage_kind)
            VALUES ($1, $2, $3, $4, $5, 'remote_url')
            "#,
        )
        .bind(wiki_repository_id)
        .bind(page_id)
        .bind(revision_id)
        .bind(&image.source_url)
        .bind(&image.alt_text)
        .execute(&mut *tx)
        .await?;
    }

    let storage_path = publish_local_wiki_commit(
        &repository,
        &page_slug,
        &path,
        &markdown,
        &message,
        &default_branch,
    )?;
    let commit_row = sqlx::query(
        r#"
        INSERT INTO wiki_git_commits (
            wiki_repository_id, page_id, revision_id, actor_user_id, branch,
            commit_oid, message, storage_path
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, created_at
        "#,
    )
    .bind(wiki_repository_id)
    .bind(page_id)
    .bind(revision_id)
    .bind(actor_user_id)
    .bind(&default_branch)
    .bind(&commit_oid)
    .bind(&message)
    .bind(storage_path.to_string_lossy().to_string())
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE wiki_repositories SET latest_commit_oid = $1, latest_published_at = now() WHERE id = $2",
    )
    .bind(&commit_oid)
    .bind(wiki_repository_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO repository_activity_events (repository_id, actor_user_id, event_type, target_type, target_id, message, metadata)
        VALUES ($1, $2, 'repository.wiki_page.updated', 'wiki_page', $3, $4, $5)
        "#,
    )
    .bind(repository.id)
    .bind(actor_user_id)
    .bind(page_id)
    .bind(&message)
    .bind(serde_json::json!({ "slug": page_slug, "revisionId": revision_id }))
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'repository.wiki_page.save', 'wiki_page', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(page_id.to_string())
    .bind(serde_json::json!({
        "repositoryId": repository.id,
        "slug": page_slug,
        "commitOid": commit_oid,
        "imageReferences": image_references.len()
    }))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    let page = repository_wiki_for_actor_by_owner_name(
        pool,
        Some(actor_user_id),
        owner_login,
        name,
        Some(&page_slug),
        &Url::parse("http://localhost:3015").expect("local URL"),
    )
    .await?
    .and_then(|wiki| wiki.page)
    .ok_or(RepositoryError::NotFound)?;
    let created_at: DateTime<Utc> = commit_row.get("created_at");
    Ok(Some(WikiPageMutationResult {
        redirect_href: page.href.clone(),
        page,
        git_commit: WikiGitCommitSummary {
            id: commit_row.get("id"),
            oid: commit_oid.clone(),
            short_oid: commit_oid.chars().take(7).collect(),
            branch: default_branch,
            message,
            storage_path: storage_path.to_string_lossy().to_string(),
            created_at,
        },
    }))
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

async fn wiki_page_summary_by_slug(
    pool: &PgPool,
    repository: &Repository,
    wiki_repository_id: Uuid,
    slug: &str,
) -> Result<Option<WikiPageSummary>, RepositoryError> {
    let row = sqlx::query(
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
          AND lower(wiki_pages.slug) = lower($2)
          AND wiki_pages.is_sidebar = false
          AND wiki_pages.is_footer = false
        LIMIT 1
        "#,
    )
    .bind(wiki_repository_id)
    .bind(normalize_slug(slug))
    .fetch_optional(pool)
    .await
    .map_err(RepositoryError::from)?;

    Ok(row.map(|row| {
        let slug: String = row.get("slug");
        WikiPageSummary {
            id: row.get("id"),
            title: row.get("title"),
            href: wiki_page_href(repository, &slug),
            active: true,
            slug,
            has_outline: row.get("has_outline"),
            updated_at: row.get("updated_at"),
        }
    }))
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
    wiki_page_from_row(pool, repository, row, can_edit)
        .await
        .map(Some)
}

async fn wiki_page_from_row(
    pool: &PgPool,
    repository: &Repository,
    row: sqlx::postgres::PgRow,
    can_edit: bool,
) -> Result<WikiPageView, RepositoryError> {
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

    Ok(WikiPageView {
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
    })
}

async fn wiki_revision_page_row(
    pool: &PgPool,
    page_id: Uuid,
    revision: &str,
) -> Result<sqlx::postgres::PgRow, RepositoryError> {
    let revision = revision.trim();
    if revision.is_empty() {
        return Err(RepositoryError::NotFound);
    }
    if let Ok(revision_id) = Uuid::parse_str(revision) {
        return sqlx::query(&revision_select_sql(
            "WHERE wiki_pages.id = $1 AND wiki_page_revisions.id = $2",
        ))
        .bind(page_id)
        .bind(revision_id)
        .fetch_optional(pool)
        .await?
        .ok_or(RepositoryError::NotFound);
    }

    let rows = sqlx::query(&revision_select_sql(
        "WHERE wiki_pages.id = $1 AND wiki_page_revisions.commit_oid ILIKE $2",
    ))
    .bind(page_id)
    .bind(format!("{revision}%"))
    .fetch_all(pool)
    .await?;
    match rows.len() {
        1 => Ok(rows.into_iter().next().expect("one row")),
        0 => Err(RepositoryError::NotFound),
        _ => Err(RepositoryError::InvalidSecurityPolicy(
            "Wiki revision reference is ambiguous.".to_owned(),
        )),
    }
}

async fn adjacent_revision_href(
    pool: &PgPool,
    repository: &Repository,
    page_id: Uuid,
    created_at: DateTime<Utc>,
    newer: bool,
) -> Result<Option<String>, RepositoryError> {
    let comparator = if newer { ">" } else { "<" };
    let ordering = if newer { "ASC" } else { "DESC" };
    let row = sqlx::query(&format!(
        r#"
        SELECT wiki_pages.slug, wiki_page_revisions.commit_oid
        FROM wiki_page_revisions
        JOIN wiki_pages ON wiki_pages.id = wiki_page_revisions.page_id
        WHERE wiki_page_revisions.page_id = $1
          AND wiki_page_revisions.created_at {comparator} $2
        ORDER BY wiki_page_revisions.created_at {ordering}, wiki_page_revisions.id {ordering}
        LIMIT 1
        "#
    ))
    .bind(page_id)
    .bind(created_at)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| {
        let slug: String = row.get("slug");
        let commit_oid: Option<String> = row.get("commit_oid");
        wiki_revision_history_href(repository, &slug, commit_oid.as_deref())
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

fn revision_select_sql(where_clause: &str) -> String {
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
        FROM wiki_page_revisions
        JOIN wiki_pages ON wiki_pages.id = wiki_page_revisions.page_id
        LEFT JOIN users ON users.id = wiki_page_revisions.author_user_id
        {where_clause}
        ORDER BY wiki_page_revisions.created_at DESC, wiki_page_revisions.id DESC
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

fn author_from_row(row: &sqlx::postgres::PgRow) -> Option<WikiAuthor> {
    row.get::<Option<Uuid>, _>("author_id").map(|id| {
        let login: String = row.get("author_login");
        WikiAuthor {
            id,
            href: format!("/{}", percent_encode_segment(&login)),
            login,
            display_name: row.get("author_display_name"),
            avatar_url: row.get("author_avatar_url"),
        }
    })
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

fn wiki_history_href(
    repository: &Repository,
    slug: Option<&str>,
    page: i64,
    page_size: i64,
) -> String {
    let base = slug
        .map(|slug| format!("{}/_history", wiki_page_href(repository, slug)))
        .unwrap_or_else(|| format!("{}/_history", wiki_home_href(repository)));
    let mut params = Vec::new();
    if page > 1 {
        params.push(format!("page={page}"));
    }
    if page_size != 30 {
        params.push(format!("pageSize={page_size}"));
    }
    if params.is_empty() {
        base
    } else {
        format!("{base}?{}", params.join("&"))
    }
}

fn wiki_revision_history_href(
    repository: &Repository,
    slug: &str,
    commit_oid: Option<&str>,
) -> String {
    let revision = commit_oid.unwrap_or("unknown");
    format!(
        "{}/_history/{}",
        wiki_page_href(repository, slug),
        percent_encode_segment(revision)
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

fn supported_markup_formats() -> Vec<SupportedMarkupFormat> {
    vec![SupportedMarkupFormat {
        mode: MARKDOWN_MODE.to_owned(),
        label: "Markdown".to_owned(),
        extension: ".md".to_owned(),
    }]
}

fn validate_edit_mode(mode: Option<&str>) -> Result<(), RepositoryError> {
    let mode = mode.unwrap_or(MARKDOWN_MODE).trim().to_ascii_lowercase();
    match mode.as_str() {
        MARKDOWN_MODE => Ok(()),
        _ => Err(RepositoryError::InvalidSecurityPolicy(format!(
            "wiki edit mode `{mode}` is not supported"
        ))),
    }
}

fn validate_title(title: &str) -> Result<String, RepositoryError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "wiki page title is required".to_owned(),
        ));
    }
    if title.len() > 120 || title.contains('\0') {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "wiki page title is invalid".to_owned(),
        ));
    }
    let slug = title_to_slug(title)?;
    if slug.split('/').any(|segment| {
        segment.is_empty() || segment == "." || segment == ".." || segment.ends_with(".git")
    }) {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "wiki page path is invalid".to_owned(),
        ));
    }
    Ok(title.to_owned())
}

fn title_to_slug(title: &str) -> Result<String, RepositoryError> {
    let slug = normalize_slug(title);
    if slug.is_empty()
        || slug.starts_with('_')
            && !matches!(slug.to_ascii_lowercase().as_str(), "_sidebar" | "_footer")
    {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "wiki page slug is invalid".to_owned(),
        ));
    }
    Ok(slug)
}

fn wiki_path_from_slug(slug: &str) -> String {
    format!("{slug}.md")
}

fn extract_image_references(markdown: &str) -> Result<Vec<WikiImageReference>, RepositoryError> {
    let image_regex = Regex::new(r#"!\[([^\]]*)\]\(([^)\s]+)(?:\s+"[^"]*")?\)"#)
        .expect("wiki image reference regex");
    let mut images = Vec::new();
    for captures in image_regex.captures_iter(markdown) {
        let alt_text = captures
            .get(1)
            .map(|value| value.as_str().trim().to_owned())
            .unwrap_or_default();
        let source_url = captures
            .get(2)
            .map(|value| value.as_str().trim().to_owned())
            .unwrap_or_default();
        if source_url.is_empty() {
            continue;
        }
        if source_url.len() > 2048
            || source_url.contains('\0')
            || !(source_url.starts_with("https://")
                || source_url.starts_with("http://")
                || source_url.starts_with('/')
                || source_url.starts_with("./")
                || source_url.starts_with("../"))
        {
            return Err(RepositoryError::InvalidSecurityPolicy(
                "wiki image URL is invalid".to_owned(),
            ));
        }
        if alt_text.len() > 240 || alt_text.contains('\0') {
            return Err(RepositoryError::InvalidSecurityPolicy(
                "wiki image alt text is invalid".to_owned(),
            ));
        }
        images.push(WikiImageReference {
            source_url,
            alt_text,
        });
    }
    Ok(images)
}

fn validate_markdown(markdown: &str) -> Result<String, RepositoryError> {
    let markdown = markdown.trim();
    if markdown.is_empty() {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "wiki page body is required".to_owned(),
        ));
    }
    if markdown.len() > 1_000_000 {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "wiki page body is too large".to_owned(),
        ));
    }
    Ok(markdown.to_owned())
}

fn validate_commit_message(message: &str) -> Result<String, RepositoryError> {
    let message = message.trim();
    if message.is_empty() {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "wiki edit message is required".to_owned(),
        ));
    }
    if message.len() > 240 {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "wiki edit message is too long".to_owned(),
        ));
    }
    Ok(message.to_owned())
}

fn wiki_commit_oid(repository: &Repository, slug: &str, markdown: &str, message: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(repository.id.as_bytes());
    hasher.update(slug.as_bytes());
    hasher.update(markdown.as_bytes());
    hasher.update(message.as_bytes());
    hasher.update(
        Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_default()
            .to_string(),
    );
    format!("{:x}", hasher.finalize())
}

fn publish_local_wiki_commit(
    repository: &Repository,
    slug: &str,
    path: &str,
    markdown: &str,
    message: &str,
    default_branch: &str,
) -> Result<PathBuf, RepositoryError> {
    let bare_path = wiki_storage_path(repository);
    if let Some(parent) = bare_path.parent() {
        fs::create_dir_all(parent).map_err(|_| RepositoryError::GitStorageFailed)?;
    }
    if !bare_path.exists() {
        run_git(
            Path::new("."),
            ["init", "--bare", bare_path.to_string_lossy().as_ref()],
        )?;
    }
    let work_path = std::env::temp_dir().join(format!("opengithub-wiki-{}", Uuid::new_v4()));
    fs::create_dir_all(&work_path).map_err(|_| RepositoryError::GitStorageFailed)?;
    let result = publish_local_wiki_commit_inner(
        &bare_path,
        &work_path,
        slug,
        path,
        markdown,
        message,
        default_branch,
    );
    let _ = fs::remove_dir_all(&work_path);
    result.map(|_| bare_path)
}

fn publish_local_wiki_commit_inner(
    bare_path: &Path,
    work_path: &Path,
    slug: &str,
    path: &str,
    markdown: &str,
    message: &str,
    default_branch: &str,
) -> Result<(), RepositoryError> {
    run_git(work_path, ["init"])?;
    run_git(work_path, ["config", "user.name", "OpenGitHub Wiki"])?;
    run_git(work_path, ["config", "user.email", "wiki@opengithub.local"])?;
    run_git(
        work_path,
        [
            "remote",
            "add",
            "origin",
            bare_path.to_string_lossy().as_ref(),
        ],
    )?;
    let _ = run_git(work_path, ["fetch", "origin", default_branch]);
    let _ = run_git(
        work_path,
        [
            "checkout",
            "-B",
            default_branch,
            &format!("origin/{default_branch}"),
        ],
    );
    let file_path = work_path.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(|_| RepositoryError::GitStorageFailed)?;
    }
    fs::write(&file_path, markdown).map_err(|_| RepositoryError::GitStorageFailed)?;
    run_git(work_path, ["add", path])?;
    run_git(work_path, ["commit", "-m", message])?;
    run_git(
        work_path,
        ["push", "origin", &format!("HEAD:{default_branch}")],
    )?;
    tracing::debug!(slug, path, "published local wiki commit");
    Ok(())
}

fn run_git<const N: usize>(cwd: &Path, args: [&str; N]) -> Result<(), RepositoryError> {
    let status = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()
        .map_err(|_| RepositoryError::GitStorageFailed)?;
    if status.success() {
        Ok(())
    } else {
        Err(RepositoryError::GitStorageFailed)
    }
}

fn wiki_storage_path(repository: &Repository) -> PathBuf {
    let root = std::env::var("OPENGITHUB_GIT_STORAGE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("opengithub-git-storage"));
    root.join(format!(
        "{}-{}.wiki.git",
        repository.owner_login, repository.name
    ))
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
