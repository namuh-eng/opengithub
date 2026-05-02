use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::time::Instant;
use uuid::Uuid;

use crate::api_types::ListEnvelope;

use super::repositories::{repository_permission_for_user, RepositoryVisibility};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchDocumentKind {
    Repository,
    Code,
    Commit,
    Issue,
    PullRequest,
    User,
    Organization,
    Package,
}

impl SearchDocumentKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Repository => "repository",
            Self::Code => "code",
            Self::Commit => "commit",
            Self::Issue => "issue",
            Self::PullRequest => "pull_request",
            Self::User => "user",
            Self::Organization => "organization",
            Self::Package => "package",
        }
    }
}

impl TryFrom<&str> for SearchDocumentKind {
    type Error = SearchError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "repository" => Ok(Self::Repository),
            "code" => Ok(Self::Code),
            "commit" => Ok(Self::Commit),
            "issue" => Ok(Self::Issue),
            "pull_request" => Ok(Self::PullRequest),
            "user" => Ok(Self::User),
            "organization" => Ok(Self::Organization),
            "package" => Ok(Self::Package),
            other => Err(SearchError::InvalidKind(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchDocument {
    pub id: Uuid,
    pub repository_id: Option<Uuid>,
    pub owner_user_id: Option<Uuid>,
    pub owner_organization_id: Option<Uuid>,
    pub kind: SearchDocumentKind,
    pub resource_id: String,
    pub title: String,
    pub body: String,
    pub path: Option<String>,
    pub language: Option<String>,
    pub branch: Option<String>,
    pub visibility: RepositoryVisibility,
    pub metadata: Value,
    pub indexed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertSearchDocument {
    pub repository_id: Option<Uuid>,
    pub owner_user_id: Option<Uuid>,
    pub owner_organization_id: Option<Uuid>,
    pub kind: SearchDocumentKind,
    pub resource_id: String,
    pub title: String,
    pub body: Option<String>,
    pub path: Option<String>,
    pub language: Option<String>,
    pub branch: Option<String>,
    pub visibility: RepositoryVisibility,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub actor_user_id: Uuid,
    pub query: String,
    pub kind: Option<SearchDocumentKind>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CodeSearchQuery {
    pub actor_user_id: Uuid,
    pub query: String,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CodeSearchResponse {
    pub items: Vec<SearchResult>,
    pub total: i64,
    pub page: i64,
    #[serde(rename = "pageSize")]
    pub page_size: i64,
    pub type_counts: Vec<CodeSearchTypeCount>,
    pub facets: CodeSearchFacets,
    pub active_chips: Vec<CodeSearchChip>,
    pub query_duration_ms: i64,
    pub diagnostics: Vec<CodeSearchDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationSearchQuery {
    pub actor_user_id: Uuid,
    pub query: String,
    pub kind: SearchDocumentKind,
    pub page: i64,
    pub page_size: i64,
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationSearchResponse {
    pub items: Vec<SearchResult>,
    pub total: i64,
    pub page: i64,
    #[serde(rename = "pageSize")]
    pub page_size: i64,
    pub type_counts: Vec<CodeSearchTypeCount>,
    pub facets: CollaborationSearchFacets,
    pub active_chips: Vec<CodeSearchChip>,
    pub sort_options: Vec<CollaborationSearchSortOption>,
    pub active_sort: String,
    pub query_duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationSearchFacets {
    pub states: Vec<CodeSearchFacetValue>,
    pub labels: Vec<CodeSearchFacetValue>,
    pub assignees: Vec<CodeSearchFacetValue>,
    pub reviewers: Vec<CodeSearchFacetValue>,
    pub milestones: Vec<CodeSearchFacetValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationSearchSortOption {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedCollaborationSearchQuery {
    terms: String,
    chips: Vec<CodeSearchChip>,
    state: Option<String>,
    label: Option<String>,
    author: Option<String>,
    assignee: Option<String>,
    reviewer: Option<String>,
    milestone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeSearchTypeCount {
    pub result_type: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeSearchFacets {
    pub languages: Vec<CodeSearchFacetValue>,
    pub paths: Vec<CodeSearchFacetValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeSearchFacetValue {
    pub value: String,
    pub label: String,
    pub count: i64,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeSearchChip {
    pub qualifier: String,
    pub value: String,
    pub label: String,
    pub remove_query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeSearchDiagnostic {
    pub code: String,
    pub message: String,
    pub qualifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedCodeSearchQuery {
    terms: String,
    chips: Vec<CodeSearchChip>,
    repo: Option<(String, String)>,
    owner: Option<String>,
    language: Option<String>,
    path: Option<String>,
    symbol: Option<String>,
    archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchMatchRange {
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchSnippet {
    pub path: String,
    pub branch: String,
    pub line_number: Option<i64>,
    pub fragment: String,
    pub language: Option<String>,
    pub match_ranges: Vec<SearchMatchRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchCommitSummary {
    pub oid: String,
    pub short_oid: String,
    pub message_title: String,
    pub message_body: Option<String>,
    pub author_login: Option<String>,
    pub committed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub document: SearchDocument,
    pub rank: f64,
    #[serde(rename = "type")]
    pub result_type: String,
    pub href: String,
    pub title: String,
    pub summary: Option<String>,
    pub owner_login: Option<String>,
    pub repository_name: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub visibility: RepositoryVisibility,
    pub updated_at: DateTime<Utc>,
    pub snippet: Option<SearchSnippet>,
    pub snippets: Vec<SearchSnippet>,
    pub match_count: i64,
    pub hidden_match_count: i64,
    pub blob_href: Option<String>,
    pub commit: Option<SearchCommitSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchSuggestionQuery {
    pub actor_user_id: Uuid,
    pub query: String,
    pub scope: Option<String>,
    pub limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateSavedSearchInput {
    pub actor_user_id: Uuid,
    pub name: String,
    pub query: String,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchSuggestionDashboard {
    pub query: String,
    pub scope: String,
    pub token: Option<SearchSuggestionToken>,
    pub groups: Vec<SearchSuggestionGroup>,
    pub saved_searches: Vec<SavedSearchSuggestion>,
    pub recent_searches: Vec<RecentSearchSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchSuggestionToken {
    pub prefix: Option<String>,
    pub value: String,
    pub replace_from: usize,
    pub replace_to: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchSuggestionGroup {
    pub id: String,
    pub title: String,
    pub items: Vec<SearchSuggestionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchSuggestionItem {
    pub id: String,
    pub kind: String,
    pub action: SearchSuggestionAction,
    pub title: String,
    pub description: Option<String>,
    pub href: Option<String>,
    pub next_query: Option<String>,
    pub scope: Option<String>,
    pub owner_login: Option<String>,
    pub repository_name: Option<String>,
    pub visibility: Option<RepositoryVisibility>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchSuggestionAction {
    Navigate,
    SubmitSearch,
    ReplaceToken,
    OpenSavedSearchDialog,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SavedSearchSuggestion {
    pub id: Uuid,
    pub name: String,
    pub query: String,
    pub scope: String,
    pub href: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecentSearchSuggestion {
    pub id: Uuid,
    pub query: String,
    pub scope: String,
    pub result_type: Option<String>,
    pub href: String,
    pub searched_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("search query must contain at least two non-whitespace characters")]
    QueryTooShort,
    #[error("{0}")]
    Validation(String),
    #[error("saved search name already exists")]
    DuplicateSavedSearchName,
    #[error("saved search not found")]
    SavedSearchNotFound,
    #[error("user does not have repository access")]
    RepositoryAccessDenied,
    #[error("invalid search document kind `{0}`")]
    InvalidKind(String),
    #[error(transparent)]
    Repository(#[from] super::repositories::RepositoryError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub async fn upsert_search_document(
    pool: &PgPool,
    actor_user_id: Uuid,
    input: UpsertSearchDocument,
) -> Result<SearchDocument, SearchError> {
    if let Some(repository_id) = input.repository_id {
        let permission = repository_permission_for_user(pool, repository_id, actor_user_id).await?;
        if !permission
            .as_ref()
            .is_some_and(|permission| permission.role.can_write())
        {
            return Err(SearchError::RepositoryAccessDenied);
        }
    }

    let row = sqlx::query(
        r#"
        INSERT INTO search_documents (
            repository_id,
            owner_user_id,
            owner_organization_id,
            kind,
            resource_id,
            title,
            body,
            path,
            language,
            branch,
            visibility,
            metadata,
            indexed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, ''), $8, $9, $10, $11, $12, now())
        ON CONFLICT (kind, resource_id) DO UPDATE SET
            repository_id = EXCLUDED.repository_id,
            owner_user_id = EXCLUDED.owner_user_id,
            owner_organization_id = EXCLUDED.owner_organization_id,
            title = EXCLUDED.title,
            body = EXCLUDED.body,
            path = EXCLUDED.path,
            language = EXCLUDED.language,
            branch = EXCLUDED.branch,
            visibility = EXCLUDED.visibility,
            metadata = EXCLUDED.metadata,
            indexed_at = now()
        RETURNING id, repository_id, owner_user_id, owner_organization_id, kind, resource_id,
                  title, body, path, language, branch, visibility, metadata, indexed_at,
                  created_at, updated_at
        "#,
    )
    .bind(input.repository_id)
    .bind(input.owner_user_id)
    .bind(input.owner_organization_id)
    .bind(input.kind.as_str())
    .bind(&input.resource_id)
    .bind(&input.title)
    .bind(&input.body)
    .bind(&input.path)
    .bind(&input.language)
    .bind(&input.branch)
    .bind(input.visibility.as_str())
    .bind(&input.metadata)
    .fetch_one(pool)
    .await?;

    document_from_row(row)
}

pub async fn search_documents(
    pool: &PgPool,
    input: SearchQuery,
) -> Result<ListEnvelope<SearchResult>, SearchError> {
    let query = input.query.trim();
    if query.chars().count() < 2 {
        return Err(SearchError::QueryTooShort);
    }

    let page = input.page.max(1);
    let page_size = input.page_size.clamp(1, 50);
    let offset = (page - 1) * page_size;
    let kind = input.kind.as_ref().map(SearchDocumentKind::as_str);

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM search_documents
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = search_documents.repository_id
         AND repository_permissions.user_id = $1
        WHERE ($2::text IS NULL OR search_documents.kind = $2)
          AND (
              search_documents.visibility = 'public'
              OR repository_permissions.user_id IS NOT NULL
              OR search_documents.owner_user_id = $1
          )
          AND (
              search_documents.search_vector @@ plainto_tsquery('simple', $3)
              OR search_documents.title ILIKE '%' || $3 || '%'
              OR search_documents.body ILIKE '%' || $3 || '%'
              OR search_documents.path ILIKE '%' || $3 || '%'
          )
        "#,
    )
    .bind(input.actor_user_id)
    .bind(kind)
    .bind(query)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT search_documents.id,
               search_documents.repository_id,
               search_documents.owner_user_id,
               search_documents.owner_organization_id,
               search_documents.kind,
               search_documents.resource_id,
               search_documents.title,
               search_documents.body,
               search_documents.path,
               search_documents.language,
               search_documents.branch,
               search_documents.visibility,
               search_documents.metadata,
               search_documents.indexed_at,
               search_documents.created_at,
               search_documents.updated_at,
               COALESCE(
                   NULLIF(repo_owner_user.username, ''),
                   repo_owner_user.email,
                   repo_owner_org.slug,
                   NULLIF(owner_user.username, ''),
                   owner_user.email,
                   owner_org.slug,
                   search_documents.metadata->>'ownerLogin'
               ) AS owner_login,
               repositories.name AS repository_name,
               COALESCE(
                   NULLIF(search_documents.metadata->>'description', ''),
                   repositories.description,
                   search_documents.body
               ) AS result_summary,
               COALESCE(
                   NULLIF(owner_user.display_name, ''),
                   NULLIF(owner_user.username, ''),
                   owner_user.email,
                   owner_org.display_name,
                   search_documents.metadata->>'displayName',
                   search_documents.title
               ) AS display_name,
               COALESCE(owner_user.avatar_url, search_documents.metadata->>'avatarUrl') AS avatar_url,
               (
                   ts_rank(search_documents.search_vector, plainto_tsquery('simple', $3))
                   + similarity(search_documents.title, $3)
                   + COALESCE(similarity(search_documents.path, $3), 0)
               )::float8 AS rank
        FROM search_documents
        LEFT JOIN repositories
          ON repositories.id = search_documents.repository_id
        LEFT JOIN users repo_owner_user
          ON repo_owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations repo_owner_org
          ON repo_owner_org.id = repositories.owner_organization_id
        LEFT JOIN users owner_user
          ON owner_user.id = search_documents.owner_user_id
        LEFT JOIN organizations owner_org
          ON owner_org.id = search_documents.owner_organization_id
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = search_documents.repository_id
         AND repository_permissions.user_id = $1
        WHERE ($2::text IS NULL OR search_documents.kind = $2)
          AND (
              search_documents.visibility = 'public'
              OR repository_permissions.user_id IS NOT NULL
              OR search_documents.owner_user_id = $1
          )
          AND (
              search_documents.search_vector @@ plainto_tsquery('simple', $3)
              OR search_documents.title ILIKE '%' || $3 || '%'
              OR search_documents.body ILIKE '%' || $3 || '%'
              OR search_documents.path ILIKE '%' || $3 || '%'
          )
        ORDER BY rank DESC, search_documents.updated_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(input.actor_user_id)
    .bind(kind)
    .bind(query)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let rank = row.get::<f64, _>("rank");
        let owner_login: Option<String> = row.get("owner_login");
        let repository_name: Option<String> = row.get("repository_name");
        let summary: Option<String> = row.get("result_summary");
        let display_name: Option<String> = row.get("display_name");
        let avatar_url: Option<String> = row.get("avatar_url");
        let document = document_from_row(row)?;
        let result_type = ui_type_for_kind(&document.kind).to_owned();
        let href = result_href(
            &document,
            owner_login.as_deref(),
            repository_name.as_deref(),
        );
        let snippet = code_snippet_for_document(&document, query);
        let snippets = code_snippets_for_document(&document, query);
        let match_count = snippets.len() as i64;
        let hidden_match_count = (match_count - 3).max(0);
        let blob_href = code_blob_href(
            &document,
            owner_login.as_deref(),
            repository_name.as_deref(),
        );
        let commit = commit_summary_for_document(&document);
        items.push(SearchResult {
            title: document.title.clone(),
            visibility: document.visibility.clone(),
            updated_at: document.updated_at,
            document,
            rank,
            result_type,
            href,
            summary,
            owner_login,
            repository_name,
            display_name,
            avatar_url,
            snippet,
            snippets,
            match_count,
            hidden_match_count,
            blob_href,
            commit,
        });
    }

    Ok(ListEnvelope {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn search_collaboration_results(
    pool: &PgPool,
    input: CollaborationSearchQuery,
) -> Result<CollaborationSearchResponse, SearchError> {
    if !matches!(
        input.kind,
        SearchDocumentKind::Issue | SearchDocumentKind::PullRequest
    ) {
        return Err(SearchError::InvalidKind(input.kind.as_str().to_owned()));
    }
    let query = input.query.trim();
    if query.chars().count() < 2 {
        return Err(SearchError::QueryTooShort);
    }

    let started_at = Instant::now();
    let parsed = parse_collaboration_query(query);
    let page = input.page.max(1);
    let page_size = input.page_size.clamp(1, 50);
    let offset = (page - 1) * page_size;
    let kind = input.kind.as_str();
    let sort = normalize_collaboration_sort(input.sort.as_deref());
    let order_by = collaboration_sort_order_by(&sort);

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM search_documents
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = search_documents.repository_id
         AND repository_permissions.user_id = $1
        WHERE search_documents.kind = $2
          AND (
              search_documents.visibility = 'public'
              OR repository_permissions.user_id IS NOT NULL
              OR search_documents.owner_user_id = $1
          )
          AND (
              $3 = ''
              OR search_documents.search_vector @@ plainto_tsquery('simple', $3)
              OR search_documents.title ILIKE '%' || $3 || '%'
              OR search_documents.body ILIKE '%' || $3 || '%'
          )
          AND ($4::text IS NULL OR lower(search_documents.metadata->>'state') = lower($4))
          AND ($5::text IS NULL OR EXISTS (
              SELECT 1
              FROM jsonb_array_elements(COALESCE(search_documents.metadata->'labels', '[]'::jsonb)) AS label(value)
              WHERE lower(label.value->>'name') = lower($5)
          ))
          AND ($6::text IS NULL OR lower(search_documents.metadata->>'authorLogin') = lower($6))
          AND ($7::text IS NULL OR EXISTS (
              SELECT 1
              FROM jsonb_array_elements(COALESCE(search_documents.metadata->'assignees', '[]'::jsonb)) AS assignee(value)
              WHERE lower(COALESCE(assignee.value->>'login', assignee.value->>'name')) = lower($7)
          ))
          AND ($8::text IS NULL OR EXISTS (
              SELECT 1
              FROM jsonb_array_elements(COALESCE(search_documents.metadata->'reviewers', '[]'::jsonb)) AS reviewer(value)
              WHERE lower(COALESCE(reviewer.value->>'login', reviewer.value->>'name')) = lower($8)
          ))
          AND ($9::text IS NULL OR lower(COALESCE(search_documents.metadata->'milestone'->>'title', search_documents.metadata->>'milestone')) = lower($9))
        "#,
    )
    .bind(input.actor_user_id)
    .bind(kind)
    .bind(&parsed.terms)
    .bind(parsed.state.as_deref())
    .bind(parsed.label.as_deref())
    .bind(parsed.author.as_deref())
    .bind(parsed.assignee.as_deref())
    .bind(parsed.reviewer.as_deref())
    .bind(parsed.milestone.as_deref())
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(&format!(
        r#"
        SELECT search_documents.id,
               search_documents.repository_id,
               search_documents.owner_user_id,
               search_documents.owner_organization_id,
               search_documents.kind,
               search_documents.resource_id,
               search_documents.title,
               search_documents.body,
               search_documents.path,
               search_documents.language,
               search_documents.branch,
               search_documents.visibility,
               search_documents.metadata,
               search_documents.indexed_at,
               search_documents.created_at,
               search_documents.updated_at,
               COALESCE(
                   NULLIF(repo_owner_user.username, ''),
                   repo_owner_user.email,
                   repo_owner_org.slug,
                   NULLIF(owner_user.username, ''),
                   owner_user.email,
                   owner_org.slug,
                   search_documents.metadata->>'ownerLogin'
               ) AS owner_login,
               repositories.name AS repository_name,
               COALESCE(
                   NULLIF(search_documents.metadata->>'description', ''),
                   search_documents.body
               ) AS result_summary,
               search_documents.title AS display_name,
               COALESCE(owner_user.avatar_url, search_documents.metadata->>'avatarUrl') AS avatar_url,
               (
                   CASE WHEN $3 = '' THEN 0 ELSE ts_rank(search_documents.search_vector, plainto_tsquery('simple', $3)) END
                   + CASE WHEN $3 = '' THEN 0 ELSE similarity(search_documents.title, $3) END
               )::float8 AS rank
        FROM search_documents
        LEFT JOIN repositories
          ON repositories.id = search_documents.repository_id
        LEFT JOIN users repo_owner_user
          ON repo_owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations repo_owner_org
          ON repo_owner_org.id = repositories.owner_organization_id
        LEFT JOIN users owner_user
          ON owner_user.id = search_documents.owner_user_id
        LEFT JOIN organizations owner_org
          ON owner_org.id = search_documents.owner_organization_id
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = search_documents.repository_id
         AND repository_permissions.user_id = $1
        WHERE search_documents.kind = $2
          AND (
              search_documents.visibility = 'public'
              OR repository_permissions.user_id IS NOT NULL
              OR search_documents.owner_user_id = $1
          )
          AND (
              $3 = ''
              OR search_documents.search_vector @@ plainto_tsquery('simple', $3)
              OR search_documents.title ILIKE '%' || $3 || '%'
              OR search_documents.body ILIKE '%' || $3 || '%'
          )
          AND ($4::text IS NULL OR lower(search_documents.metadata->>'state') = lower($4))
          AND ($5::text IS NULL OR EXISTS (
              SELECT 1
              FROM jsonb_array_elements(COALESCE(search_documents.metadata->'labels', '[]'::jsonb)) AS label(value)
              WHERE lower(label.value->>'name') = lower($5)
          ))
          AND ($6::text IS NULL OR lower(search_documents.metadata->>'authorLogin') = lower($6))
          AND ($7::text IS NULL OR EXISTS (
              SELECT 1
              FROM jsonb_array_elements(COALESCE(search_documents.metadata->'assignees', '[]'::jsonb)) AS assignee(value)
              WHERE lower(COALESCE(assignee.value->>'login', assignee.value->>'name')) = lower($7)
          ))
          AND ($8::text IS NULL OR EXISTS (
              SELECT 1
              FROM jsonb_array_elements(COALESCE(search_documents.metadata->'reviewers', '[]'::jsonb)) AS reviewer(value)
              WHERE lower(COALESCE(reviewer.value->>'login', reviewer.value->>'name')) = lower($8)
          ))
          AND ($9::text IS NULL OR lower(COALESCE(search_documents.metadata->'milestone'->>'title', search_documents.metadata->>'milestone')) = lower($9))
        ORDER BY {order_by}
        LIMIT $10 OFFSET $11
        "#
    ))
    .bind(input.actor_user_id)
    .bind(kind)
    .bind(&parsed.terms)
    .bind(parsed.state.as_deref())
    .bind(parsed.label.as_deref())
    .bind(parsed.author.as_deref())
    .bind(parsed.assignee.as_deref())
    .bind(parsed.reviewer.as_deref())
    .bind(parsed.milestone.as_deref())
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let rank = row.get::<f64, _>("rank");
        let owner_login: Option<String> = row.get("owner_login");
        let repository_name: Option<String> = row.get("repository_name");
        let summary: Option<String> = row.get("result_summary");
        let display_name: Option<String> = row.get("display_name");
        let avatar_url: Option<String> = row.get("avatar_url");
        let document = document_from_row(row)?;
        let result_type = ui_type_for_kind(&document.kind).to_owned();
        let href = result_href(
            &document,
            owner_login.as_deref(),
            repository_name.as_deref(),
        );
        items.push(SearchResult {
            title: document.title.clone(),
            visibility: document.visibility.clone(),
            updated_at: document.updated_at,
            document,
            rank,
            result_type,
            href,
            summary,
            owner_login,
            repository_name,
            display_name,
            avatar_url,
            snippet: None,
            snippets: Vec::new(),
            match_count: 0,
            hidden_match_count: 0,
            blob_href: None,
            commit: None,
        });
    }

    Ok(CollaborationSearchResponse {
        items,
        total,
        page,
        page_size,
        type_counts: collaboration_type_counts(pool, input.actor_user_id, &parsed.terms).await?,
        facets: collaboration_search_facets(pool, input.actor_user_id, kind, &parsed).await?,
        active_chips: parsed.chips,
        sort_options: collaboration_sort_options(&sort),
        active_sort: sort,
        query_duration_ms: started_at.elapsed().as_millis().min(i64::MAX as u128) as i64,
    })
}

pub async fn search_code_results(
    pool: &PgPool,
    input: CodeSearchQuery,
) -> Result<CodeSearchResponse, SearchError> {
    let started_at = Instant::now();
    let parsed = parse_code_search_query(&input.query)?;
    if parsed.terms.chars().count() < 2 {
        return Err(SearchError::QueryTooShort);
    }

    let page = input.page.max(1);
    let page_size = input.page_size.clamp(1, 50);
    let offset = (page - 1) * page_size;
    let repo_owner = parsed.repo.as_ref().map(|(owner, _)| owner.as_str());
    let repo_name = parsed.repo.as_ref().map(|(_, name)| name.as_str());

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM search_documents
        LEFT JOIN repositories ON repositories.id = search_documents.repository_id
        LEFT JOIN users repo_owner_user ON repo_owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations repo_owner_org ON repo_owner_org.id = repositories.owner_organization_id
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = search_documents.repository_id
         AND repository_permissions.user_id = $1
        WHERE search_documents.kind = 'code'
          AND (
              search_documents.visibility = 'public'
              OR repository_permissions.user_id IS NOT NULL
              OR search_documents.owner_user_id = $1
          )
          AND (
              search_documents.search_vector @@ plainto_tsquery('simple', $2)
              OR search_documents.title ILIKE '%' || $2 || '%'
              OR search_documents.body ILIKE '%' || $2 || '%'
              OR search_documents.path ILIKE '%' || $2 || '%'
          )
          AND ($3::text IS NULL OR lower(search_documents.language) = lower($3))
          AND ($4::text IS NULL OR search_documents.path ILIKE '%' || $4 || '%')
          AND ($5::text IS NULL OR lower(COALESCE(repo_owner_user.username, repo_owner_user.email, repo_owner_org.slug)) = lower($5))
          AND ($6::text IS NULL OR lower(repositories.name) = lower($6))
          AND ($7::text IS NULL OR lower(COALESCE(repo_owner_user.username, repo_owner_user.email, repo_owner_org.slug)) = lower($7))
          AND ($8::text IS NULL OR search_documents.metadata->>'symbol' ILIKE '%' || $8 || '%' OR search_documents.body ILIKE '%' || $8 || '%')
          AND ($9::boolean IS NULL OR repositories.is_archived = $9)
        "#,
    )
    .bind(input.actor_user_id)
    .bind(&parsed.terms)
    .bind(parsed.language.as_deref())
    .bind(parsed.path.as_deref())
    .bind(repo_owner)
    .bind(repo_name)
    .bind(parsed.owner.as_deref())
    .bind(parsed.symbol.as_deref())
    .bind(parsed.archived)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT search_documents.id,
               search_documents.repository_id,
               search_documents.owner_user_id,
               search_documents.owner_organization_id,
               search_documents.kind,
               search_documents.resource_id,
               search_documents.title,
               search_documents.body,
               search_documents.path,
               search_documents.language,
               search_documents.branch,
               search_documents.visibility,
               search_documents.metadata,
               search_documents.indexed_at,
               search_documents.created_at,
               search_documents.updated_at,
               COALESCE(NULLIF(repo_owner_user.username, ''), repo_owner_user.email, repo_owner_org.slug, search_documents.metadata->>'ownerLogin') AS owner_login,
               repositories.name AS repository_name,
               COALESCE(NULLIF(search_documents.metadata->>'description', ''), repositories.description, search_documents.body) AS result_summary,
               search_documents.title AS display_name,
               NULL::text AS avatar_url,
               (
                   ts_rank(search_documents.search_vector, plainto_tsquery('simple', $2))
                   + similarity(search_documents.title, $2)
                   + COALESCE(similarity(search_documents.path, $2), 0)
               )::float8 AS rank
        FROM search_documents
        LEFT JOIN repositories ON repositories.id = search_documents.repository_id
        LEFT JOIN users repo_owner_user ON repo_owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations repo_owner_org ON repo_owner_org.id = repositories.owner_organization_id
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = search_documents.repository_id
         AND repository_permissions.user_id = $1
        WHERE search_documents.kind = 'code'
          AND (
              search_documents.visibility = 'public'
              OR repository_permissions.user_id IS NOT NULL
              OR search_documents.owner_user_id = $1
          )
          AND (
              search_documents.search_vector @@ plainto_tsquery('simple', $2)
              OR search_documents.title ILIKE '%' || $2 || '%'
              OR search_documents.body ILIKE '%' || $2 || '%'
              OR search_documents.path ILIKE '%' || $2 || '%'
          )
          AND ($3::text IS NULL OR lower(search_documents.language) = lower($3))
          AND ($4::text IS NULL OR search_documents.path ILIKE '%' || $4 || '%')
          AND ($5::text IS NULL OR lower(COALESCE(repo_owner_user.username, repo_owner_user.email, repo_owner_org.slug)) = lower($5))
          AND ($6::text IS NULL OR lower(repositories.name) = lower($6))
          AND ($7::text IS NULL OR lower(COALESCE(repo_owner_user.username, repo_owner_user.email, repo_owner_org.slug)) = lower($7))
          AND ($8::text IS NULL OR search_documents.metadata->>'symbol' ILIKE '%' || $8 || '%' OR search_documents.body ILIKE '%' || $8 || '%')
          AND ($9::boolean IS NULL OR repositories.is_archived = $9)
        ORDER BY rank DESC, search_documents.updated_at DESC, search_documents.path ASC
        LIMIT $10 OFFSET $11
        "#,
    )
    .bind(input.actor_user_id)
    .bind(&parsed.terms)
    .bind(parsed.language.as_deref())
    .bind(parsed.path.as_deref())
    .bind(repo_owner)
    .bind(repo_name)
    .bind(parsed.owner.as_deref())
    .bind(parsed.symbol.as_deref())
    .bind(parsed.archived)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let rank = row.get::<f64, _>("rank");
        let owner_login: Option<String> = row.get("owner_login");
        let repository_name: Option<String> = row.get("repository_name");
        let summary: Option<String> = row.get("result_summary");
        let display_name: Option<String> = row.get("display_name");
        let avatar_url: Option<String> = row.get("avatar_url");
        let document = document_from_row(row)?;
        let href = result_href(
            &document,
            owner_login.as_deref(),
            repository_name.as_deref(),
        );
        let snippet = code_snippet_for_document(&document, &parsed.terms);
        let snippets = code_snippets_for_document(&document, &parsed.terms);
        let match_count = snippets.len() as i64;
        let hidden_match_count = (match_count - 3).max(0);
        let blob_href = code_blob_href(
            &document,
            owner_login.as_deref(),
            repository_name.as_deref(),
        );
        items.push(SearchResult {
            title: document.title.clone(),
            visibility: document.visibility.clone(),
            updated_at: document.updated_at,
            document,
            rank,
            result_type: "code".to_owned(),
            href,
            summary,
            owner_login,
            repository_name,
            display_name,
            avatar_url,
            snippet,
            snippets,
            match_count,
            hidden_match_count,
            blob_href,
            commit: None,
        });
    }

    Ok(CodeSearchResponse {
        items,
        total,
        page,
        page_size,
        type_counts: code_search_type_counts(pool, input.actor_user_id, &parsed.terms).await?,
        facets: code_search_facets(pool, input.actor_user_id, &parsed).await?,
        active_chips: parsed.chips,
        query_duration_ms: started_at.elapsed().as_millis().min(i64::MAX as u128) as i64,
        diagnostics: Vec::new(),
    })
}

pub async fn create_saved_search(
    pool: &PgPool,
    input: CreateSavedSearchInput,
) -> Result<SavedSearchSuggestion, SearchError> {
    let name = normalize_saved_search_name(&input.name)?;
    let query = normalize_saved_search_query(&input.query)?;
    let scope = normalize_saved_search_scope(input.scope.as_deref());

    let row = sqlx::query(
        r#"
        INSERT INTO saved_searches (user_id, name, query, scope)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, query, scope, updated_at
        "#,
    )
    .bind(input.actor_user_id)
    .bind(&name)
    .bind(&query)
    .bind(&scope)
    .fetch_one(pool)
    .await
    .map_err(|error| {
        if let sqlx::Error::Database(database_error) = &error {
            if database_error.constraint() == Some("saved_searches_user_name_lower_unique") {
                return SearchError::DuplicateSavedSearchName;
            }
        }
        SearchError::Sqlx(error)
    })?;

    record_recent_search(pool, input.actor_user_id, &query, &scope, Some(&scope)).await?;
    saved_search_from_row(row)
}

pub async fn delete_saved_search(
    pool: &PgPool,
    actor_user_id: Uuid,
    saved_search_id: Uuid,
) -> Result<(), SearchError> {
    let deleted = sqlx::query(
        r#"
        DELETE FROM saved_searches
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(saved_search_id)
    .bind(actor_user_id)
    .execute(pool)
    .await?
    .rows_affected();

    if deleted == 0 {
        return Err(SearchError::SavedSearchNotFound);
    }

    Ok(())
}

pub async fn record_recent_search(
    pool: &PgPool,
    actor_user_id: Uuid,
    query: &str,
    scope: &str,
    result_type: Option<&str>,
) -> Result<RecentSearchSuggestion, SearchError> {
    let query = normalize_saved_search_query(query)?;
    let scope = normalize_saved_search_scope(Some(scope));
    let result_type = result_type
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(80).collect::<String>());
    let row = sqlx::query(
        r#"
        INSERT INTO recent_searches (user_id, query, scope, result_type, searched_at)
        VALUES ($1, $2, $3, $4, now())
        ON CONFLICT (user_id, lower(query), scope, COALESCE(result_type, ''))
        DO UPDATE SET searched_at = now()
        RETURNING id, query, scope, result_type, searched_at
        "#,
    )
    .bind(actor_user_id)
    .bind(&query)
    .bind(&scope)
    .bind(&result_type)
    .fetch_one(pool)
    .await?;

    let query: String = row.get("query");
    let scope: String = row.get("scope");
    let result_type: Option<String> = row.get("result_type");
    let selected_type = result_type.as_deref().unwrap_or(&scope);
    Ok(RecentSearchSuggestion {
        id: row.get("id"),
        href: format!(
            "/search?q={}&type={}",
            percent_encode_query(&query),
            percent_encode_query(selected_type)
        ),
        query,
        scope,
        result_type,
        searched_at: row.get("searched_at"),
    })
}

pub async fn search_suggestions(
    pool: &PgPool,
    input: SearchSuggestionQuery,
) -> Result<SearchSuggestionDashboard, SearchError> {
    let query = input.query.trim().chars().take(256).collect::<String>();
    let scope = input
        .scope
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("all")
        .chars()
        .take(120)
        .collect::<String>();
    let limit = input.limit.clamp(1, 12);
    let token = suggestion_token(&query);
    let mut groups = Vec::new();

    groups.push(SearchSuggestionGroup {
        id: "scopes".to_owned(),
        title: "Search scopes".to_owned(),
        items: scoped_search_suggestions(&query, &scope),
    });

    let qualifier_items = qualifier_suggestions(&query, token.as_ref());
    if !qualifier_items.is_empty() {
        groups.push(SearchSuggestionGroup {
            id: "qualifiers".to_owned(),
            title: "Query qualifiers".to_owned(),
            items: qualifier_items,
        });
    }

    let repository_items =
        repository_and_code_suggestions(pool, input.actor_user_id, &query, limit).await?;
    if !repository_items.is_empty() {
        groups.push(SearchSuggestionGroup {
            id: "repositories".to_owned(),
            title: "Repositories and code".to_owned(),
            items: repository_items,
        });
    }

    let people_items = people_and_org_suggestions(pool, &query, limit).await?;
    if !people_items.is_empty() {
        groups.push(SearchSuggestionGroup {
            id: "people".to_owned(),
            title: "People and organizations".to_owned(),
            items: people_items,
        });
    }

    let team_items = team_suggestions(pool, input.actor_user_id, &query, limit).await?;
    if !team_items.is_empty() {
        groups.push(SearchSuggestionGroup {
            id: "teams".to_owned(),
            title: "Teams".to_owned(),
            items: team_items,
        });
    }

    Ok(SearchSuggestionDashboard {
        query,
        scope,
        token,
        groups,
        saved_searches: saved_search_suggestions(pool, input.actor_user_id, limit).await?,
        recent_searches: recent_search_suggestions(pool, input.actor_user_id, limit).await?,
    })
}

fn scoped_search_suggestions(query: &str, scope: &str) -> Vec<SearchSuggestionItem> {
    let encoded = percent_encode_query(query);
    [
        (
            "all",
            "All opengithub",
            "Search across every repository you can read",
        ),
        (
            "repositories",
            "Repositories",
            "Search repository names and descriptions",
        ),
        (
            "code",
            "Code",
            "Search indexed file paths and code snippets",
        ),
        (
            "issues",
            "Issues",
            "Search issues and pull request discussions",
        ),
    ]
    .into_iter()
    .map(|(id, title, description)| SearchSuggestionItem {
        id: format!("scope-{id}"),
        kind: "submit_search".to_owned(),
        action: SearchSuggestionAction::SubmitSearch,
        title: title.to_owned(),
        description: Some(description.to_owned()),
        href: Some(format!("/search?q={encoded}&type={id}")),
        next_query: Some(query.to_owned()),
        scope: Some(if id == "all" {
            scope.to_owned()
        } else {
            id.to_owned()
        }),
        owner_login: None,
        repository_name: None,
        visibility: None,
    })
    .collect()
}

fn qualifier_suggestions(
    query: &str,
    token: Option<&SearchSuggestionToken>,
) -> Vec<SearchSuggestionItem> {
    const QUALIFIERS: [(&str, &str, &str); 8] = [
        ("repo", "repo:owner/name", "Limit results to a repository"),
        ("org", "org:name", "Limit results to an organization"),
        ("user", "user:name", "Limit results to a user"),
        (
            "language",
            "language:rust",
            "Limit code results by language",
        ),
        ("path", "path:src/", "Limit code results by path"),
        ("symbol", "symbol:name", "Search indexed symbols"),
        ("is", "is:open", "Filter by issue or pull request state"),
        ("state", "state:open", "Filter by open or closed state"),
    ];
    let typed = token
        .map(|token| token.value.as_str())
        .unwrap_or(query)
        .trim()
        .trim_start_matches(|c: char| c == '/' || c.is_whitespace());
    if typed.is_empty() {
        return QUALIFIERS
            .iter()
            .take(5)
            .map(|(prefix, title, description)| {
                qualifier_item(query, token, prefix, title, description)
            })
            .collect();
    }
    QUALIFIERS
        .iter()
        .filter(|(prefix, title, _)| {
            prefix.starts_with(typed.trim_end_matches(':'))
                || title.starts_with(typed)
                || format!("{prefix}:").starts_with(typed)
        })
        .take(6)
        .map(|(prefix, title, description)| {
            qualifier_item(query, token, prefix, title, description)
        })
        .collect()
}

fn qualifier_item(
    query: &str,
    token: Option<&SearchSuggestionToken>,
    prefix: &str,
    title: &str,
    description: &str,
) -> SearchSuggestionItem {
    let replacement = format!("{prefix}:");
    let replacement = if token
        .and_then(|token| token.prefix.as_deref())
        .is_some_and(|typed_prefix| typed_prefix == prefix)
    {
        title.to_owned()
    } else {
        replacement
    };
    SearchSuggestionItem {
        id: format!("qualifier-{prefix}"),
        kind: "replace_token".to_owned(),
        action: SearchSuggestionAction::ReplaceToken,
        title: title.to_owned(),
        description: Some(description.to_owned()),
        href: None,
        next_query: Some(replace_token(query, token, &replacement)),
        scope: None,
        owner_login: None,
        repository_name: None,
        visibility: None,
    }
}

async fn repository_and_code_suggestions(
    pool: &PgPool,
    actor_user_id: Uuid,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchSuggestionItem>, SearchError> {
    let rows = sqlx::query(
        r#"
        SELECT search_documents.id,
               search_documents.kind,
               search_documents.title,
               search_documents.path,
               search_documents.branch,
               search_documents.visibility,
               search_documents.metadata,
               repositories.name AS repository_name,
               COALESCE(repo_owner_user.username, repo_owner_user.email, repo_owner_org.slug) AS owner_login,
               COALESCE(NULLIF(search_documents.metadata->>'description', ''), repositories.description, search_documents.body) AS description
        FROM search_documents
        LEFT JOIN repositories ON repositories.id = search_documents.repository_id
        LEFT JOIN users repo_owner_user ON repo_owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations repo_owner_org ON repo_owner_org.id = repositories.owner_organization_id
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = search_documents.repository_id
         AND repository_permissions.user_id = $1
        WHERE search_documents.kind IN ('repository', 'code')
          AND (
              search_documents.visibility = 'public'
              OR repository_permissions.user_id IS NOT NULL
              OR search_documents.owner_user_id = $1
          )
          AND (
              $2 = ''
              OR search_documents.title ILIKE '%' || $2 || '%'
              OR search_documents.path ILIKE '%' || $2 || '%'
              OR repositories.name ILIKE '%' || $2 || '%'
              OR COALESCE(repo_owner_user.username, repo_owner_user.email, repo_owner_org.slug) ILIKE '%' || $2 || '%'
          )
        ORDER BY
          CASE search_documents.kind WHEN 'repository' THEN 0 ELSE 1 END,
          similarity(search_documents.title, NULLIF($2, '')) DESC NULLS LAST,
          search_documents.updated_at DESC
        LIMIT $3
        "#,
    )
    .bind(actor_user_id)
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            let kind: String = row.get("kind");
            let title: String = row.get("title");
            let path: Option<String> = row.get("path");
            let branch: Option<String> = row.get("branch");
            let owner_login: Option<String> = row.get("owner_login");
            let repository_name: Option<String> = row.get("repository_name");
            let visibility =
                RepositoryVisibility::try_from(row.get::<String, _>("visibility").as_str()).ok();
            let metadata: Value = row.get("metadata");
            let href = metadata
                .get("href")
                .and_then(Value::as_str)
                .filter(|href| href.starts_with('/') && !href.starts_with("//"))
                .map(ToOwned::to_owned)
                .or_else(|| {
                    suggestion_href(
                        &kind,
                        owner_login.as_deref(),
                        repository_name.as_deref(),
                        branch.as_deref(),
                        path.as_deref(),
                    )
                });
            SearchSuggestionItem {
                id: id.to_string(),
                kind: if kind == "code" {
                    "direct_code_jump"
                } else {
                    "direct_repository_jump"
                }
                .to_owned(),
                action: SearchSuggestionAction::Navigate,
                title: if kind == "code" {
                    path.clone().unwrap_or(title)
                } else {
                    title
                },
                description: row.get("description"),
                href,
                next_query: None,
                scope: None,
                owner_login,
                repository_name,
                visibility,
            }
        })
        .collect())
}

async fn people_and_org_suggestions(
    pool: &PgPool,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchSuggestionItem>, SearchError> {
    let rows = sqlx::query(
        r#"
        SELECT id::text AS id,
               'user' AS kind,
               COALESCE(username, email) AS slug,
               COALESCE(display_name, username, email) AS title,
               email AS description,
               '/' || COALESCE(username, email) AS href,
               updated_at
        FROM users
        WHERE $1 = '' OR username ILIKE '%' || $1 || '%' OR display_name ILIKE '%' || $1 || '%' OR email ILIKE '%' || $1 || '%'
        UNION ALL
        SELECT id::text AS id,
               'organization' AS kind,
               slug,
               display_name AS title,
               description,
               '/orgs/' || slug AS href,
               updated_at
        FROM organizations
        WHERE $1 = '' OR slug ILIKE '%' || $1 || '%' OR display_name ILIKE '%' || $1 || '%'
        ORDER BY updated_at DESC
        LIMIT $2
        "#,
    )
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| SearchSuggestionItem {
            id: row.get("id"),
            kind: row.get("kind"),
            action: SearchSuggestionAction::Navigate,
            title: row.get("title"),
            description: row.get("description"),
            href: row.get("href"),
            next_query: None,
            scope: None,
            owner_login: Some(row.get("slug")),
            repository_name: None,
            visibility: None,
        })
        .collect())
}

async fn team_suggestions(
    pool: &PgPool,
    actor_user_id: Uuid,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchSuggestionItem>, SearchError> {
    let rows = sqlx::query(
        r#"
        SELECT teams.id,
               organizations.slug AS org_slug,
               teams.slug,
               teams.name,
               teams.description
        FROM teams
        JOIN organizations ON organizations.id = teams.organization_id
        JOIN organization_memberships
          ON organization_memberships.organization_id = organizations.id
         AND organization_memberships.user_id = $1
        WHERE $2 = '' OR teams.slug ILIKE '%' || $2 || '%' OR teams.name ILIKE '%' || $2 || '%'
        ORDER BY teams.updated_at DESC
        LIMIT $3
        "#,
    )
    .bind(actor_user_id)
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let org_slug: String = row.get("org_slug");
            let slug: String = row.get("slug");
            SearchSuggestionItem {
                id: row.get::<Uuid, _>("id").to_string(),
                kind: "team".to_owned(),
                action: SearchSuggestionAction::Navigate,
                title: row.get("name"),
                description: row.get("description"),
                href: Some(format!("/orgs/{org_slug}/teams/{slug}")),
                next_query: None,
                scope: Some(format!("org:{org_slug}")),
                owner_login: Some(org_slug),
                repository_name: None,
                visibility: None,
            }
        })
        .collect())
}

async fn saved_search_suggestions(
    pool: &PgPool,
    actor_user_id: Uuid,
    limit: i64,
) -> Result<Vec<SavedSearchSuggestion>, SearchError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, query, scope, updated_at
        FROM saved_searches
        WHERE user_id = $1
        ORDER BY updated_at DESC
        LIMIT $2
        "#,
    )
    .bind(actor_user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let query: String = row.get("query");
            let scope: String = row.get("scope");
            SavedSearchSuggestion {
                id: row.get("id"),
                name: row.get("name"),
                href: format!(
                    "/search?q={}&type={}",
                    percent_encode_query(&query),
                    percent_encode_query(&scope)
                ),
                query,
                scope,
                updated_at: row.get("updated_at"),
            }
        })
        .collect())
}

fn saved_search_from_row(row: sqlx::postgres::PgRow) -> Result<SavedSearchSuggestion, SearchError> {
    let query: String = row.get("query");
    let scope: String = row.get("scope");
    Ok(SavedSearchSuggestion {
        id: row.get("id"),
        name: row.get("name"),
        href: format!(
            "/search?q={}&type={}",
            percent_encode_query(&query),
            percent_encode_query(&scope)
        ),
        query,
        scope,
        updated_at: row.get("updated_at"),
    })
}

fn normalize_saved_search_name(name: &str) -> Result<String, SearchError> {
    let normalized = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return Err(SearchError::Validation(
            "saved search name is required".to_owned(),
        ));
    }
    if normalized.chars().count() > 80 {
        return Err(SearchError::Validation(
            "saved search name must be 80 characters or fewer".to_owned(),
        ));
    }
    Ok(normalized)
}

fn normalize_saved_search_query(query: &str) -> Result<String, SearchError> {
    let normalized = query.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return Err(SearchError::Validation(
            "saved search query is required".to_owned(),
        ));
    }
    if normalized.chars().count() > 256 {
        return Err(SearchError::Validation(
            "saved search query must be 256 characters or fewer".to_owned(),
        ));
    }
    Ok(normalized)
}

fn normalize_saved_search_scope(scope: Option<&str>) -> String {
    scope
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(80).collect::<String>())
        .unwrap_or_else(|| "repositories".to_owned())
}

async fn recent_search_suggestions(
    pool: &PgPool,
    actor_user_id: Uuid,
    limit: i64,
) -> Result<Vec<RecentSearchSuggestion>, SearchError> {
    let rows = sqlx::query(
        r#"
        SELECT id, query, scope, result_type, searched_at
        FROM recent_searches
        WHERE user_id = $1
        ORDER BY searched_at DESC
        LIMIT $2
        "#,
    )
    .bind(actor_user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let query: String = row.get("query");
            let scope: String = row.get("scope");
            let result_type: Option<String> = row.get("result_type");
            let selected_type = result_type.as_deref().unwrap_or(&scope);
            RecentSearchSuggestion {
                id: row.get("id"),
                href: format!(
                    "/search?q={}&type={}",
                    percent_encode_query(&query),
                    percent_encode_query(selected_type)
                ),
                query,
                scope,
                result_type,
                searched_at: row.get("searched_at"),
            }
        })
        .collect())
}

async fn code_search_type_counts(
    pool: &PgPool,
    actor_user_id: Uuid,
    terms: &str,
) -> Result<Vec<CodeSearchTypeCount>, SearchError> {
    let rows = sqlx::query(
        r#"
        SELECT search_documents.kind, count(*) AS count
        FROM search_documents
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = search_documents.repository_id
         AND repository_permissions.user_id = $1
        WHERE (
              search_documents.visibility = 'public'
              OR repository_permissions.user_id IS NOT NULL
              OR search_documents.owner_user_id = $1
          )
          AND (
              search_documents.search_vector @@ plainto_tsquery('simple', $2)
              OR search_documents.title ILIKE '%' || $2 || '%'
              OR search_documents.body ILIKE '%' || $2 || '%'
              OR search_documents.path ILIKE '%' || $2 || '%'
          )
        GROUP BY search_documents.kind
        "#,
    )
    .bind(actor_user_id)
    .bind(terms)
    .fetch_all(pool)
    .await?;

    let mut counts = std::collections::HashMap::new();
    for row in rows {
        let kind: String = row.get("kind");
        counts.insert(kind, row.get::<i64, _>("count"));
    }

    Ok([
        ("code", "Code"),
        ("repository", "Repositories"),
        ("issue", "Issues"),
        ("pull_request", "Pull requests"),
        ("commit", "Commits"),
        ("package", "Packages"),
        ("user", "Users"),
        ("organization", "Organizations"),
    ]
    .into_iter()
    .map(|(kind, label)| CodeSearchTypeCount {
        result_type: ui_type_for_kind_str(kind).to_owned(),
        label: label.to_owned(),
        count: counts.get(kind).copied().unwrap_or(0),
    })
    .collect())
}

fn parse_collaboration_query(query: &str) -> ParsedCollaborationSearchQuery {
    let mut terms = Vec::new();
    let mut parsed = ParsedCollaborationSearchQuery {
        terms: String::new(),
        chips: Vec::new(),
        state: None,
        label: None,
        author: None,
        assignee: None,
        reviewer: None,
        milestone: None,
    };

    for token in query.split_whitespace() {
        let Some((qualifier, raw_value)) = token.split_once(':') else {
            terms.push(token.to_owned());
            continue;
        };
        let value = raw_value.trim_matches('"').trim();
        if value.is_empty() {
            terms.push(token.to_owned());
            continue;
        }
        match qualifier {
            "state" => parsed.state = Some(value.to_owned()),
            "is" if matches!(value, "open" | "closed" | "merged") => {
                parsed.state = Some(value.to_owned())
            }
            "label" => parsed.label = Some(value.to_owned()),
            "author" => parsed.author = Some(value.to_owned()),
            "assignee" => parsed.assignee = Some(value.to_owned()),
            "reviewer" | "reviewed-by" | "review-requested" => {
                parsed.reviewer = Some(value.to_owned())
            }
            "milestone" => parsed.milestone = Some(value.to_owned()),
            _ => {
                terms.push(token.to_owned());
                continue;
            }
        }
        parsed.chips.push(CodeSearchChip {
            qualifier: qualifier.to_owned(),
            value: value.to_owned(),
            label: format!("{qualifier}:{value}"),
            remove_query: remove_first_qualifier_token(query, qualifier, value),
        });
    }

    parsed.terms = terms.join(" ");
    parsed
}

fn remove_first_qualifier_token(query: &str, qualifier: &str, value: &str) -> String {
    let target = format!("{qualifier}:{value}");
    let quoted_target = format!("{qualifier}:\"{value}\"");
    let mut removed = false;
    query
        .split_whitespace()
        .filter(|token| {
            if !removed && (*token == target || *token == quoted_target) {
                removed = true;
                return false;
            }
            true
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_collaboration_sort(sort: Option<&str>) -> String {
    match sort.unwrap_or("best-match") {
        "comments-desc" | "comments-asc" | "created-desc" | "created-asc" | "updated-desc"
        | "updated-asc" | "interactions-desc" | "interactions-asc" => sort.unwrap().to_owned(),
        _ => "best-match".to_owned(),
    }
}

fn collaboration_sort_order_by(sort: &str) -> &'static str {
    match sort {
        "comments-desc" => "(COALESCE((search_documents.metadata->>'commentCount')::bigint, 0)) DESC, rank DESC, search_documents.updated_at DESC",
        "comments-asc" => "(COALESCE((search_documents.metadata->>'commentCount')::bigint, 0)) ASC, rank DESC, search_documents.updated_at DESC",
        "created-desc" => "search_documents.created_at DESC, rank DESC",
        "created-asc" => "search_documents.created_at ASC, rank DESC",
        "updated-desc" => "search_documents.updated_at DESC, rank DESC",
        "updated-asc" => "search_documents.updated_at ASC, rank DESC",
        "interactions-desc" => "(COALESCE((search_documents.metadata->>'interactionCount')::bigint, 0)) DESC, rank DESC, search_documents.updated_at DESC",
        "interactions-asc" => "(COALESCE((search_documents.metadata->>'interactionCount')::bigint, 0)) ASC, rank DESC, search_documents.updated_at DESC",
        _ => "rank DESC, search_documents.updated_at DESC",
    }
}

fn collaboration_sort_options(active: &str) -> Vec<CollaborationSearchSortOption> {
    [
        ("best-match", "Best match"),
        ("comments-desc", "Most commented"),
        ("comments-asc", "Least commented"),
        ("created-desc", "Newest"),
        ("created-asc", "Oldest"),
        ("updated-desc", "Recently updated"),
        ("updated-asc", "Least recently updated"),
        ("interactions-desc", "Most interactions"),
        ("interactions-asc", "Least interactions"),
    ]
    .into_iter()
    .map(|(value, label)| CollaborationSearchSortOption {
        value: value.to_owned(),
        label: label.to_owned(),
        selected: value == active,
    })
    .collect()
}

async fn collaboration_type_counts(
    pool: &PgPool,
    actor_user_id: Uuid,
    terms: &str,
) -> Result<Vec<CodeSearchTypeCount>, SearchError> {
    let rows = sqlx::query(
        r#"
        SELECT search_documents.kind, count(*) AS count
        FROM search_documents
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = search_documents.repository_id
         AND repository_permissions.user_id = $1
        WHERE search_documents.kind IN ('issue', 'pull_request')
          AND (
              search_documents.visibility = 'public'
              OR repository_permissions.user_id IS NOT NULL
              OR search_documents.owner_user_id = $1
          )
          AND (
              $2 = ''
              OR search_documents.search_vector @@ plainto_tsquery('simple', $2)
              OR search_documents.title ILIKE '%' || $2 || '%'
              OR search_documents.body ILIKE '%' || $2 || '%'
          )
        GROUP BY search_documents.kind
        "#,
    )
    .bind(actor_user_id)
    .bind(terms)
    .fetch_all(pool)
    .await?;

    let count_for = |kind: &str| {
        rows.iter()
            .find(|row| row.get::<String, _>("kind") == kind)
            .map(|row| row.get::<i64, _>("count"))
            .unwrap_or(0)
    };
    Ok(vec![
        CodeSearchTypeCount {
            result_type: "issues".to_owned(),
            label: "Issues".to_owned(),
            count: count_for("issue"),
        },
        CodeSearchTypeCount {
            result_type: "pull_requests".to_owned(),
            label: "Pull requests".to_owned(),
            count: count_for("pull_request"),
        },
    ])
}

async fn collaboration_search_facets(
    pool: &PgPool,
    actor_user_id: Uuid,
    kind: &str,
    parsed: &ParsedCollaborationSearchQuery,
) -> Result<CollaborationSearchFacets, SearchError> {
    Ok(CollaborationSearchFacets {
        states: collaboration_scalar_facet(
            pool,
            actor_user_id,
            kind,
            &parsed.terms,
            "state",
            parsed.state.as_deref(),
        )
        .await?,
        labels: collaboration_array_facet(
            pool,
            actor_user_id,
            kind,
            &parsed.terms,
            "labels",
            parsed.label.as_deref(),
        )
        .await?,
        assignees: collaboration_array_facet(
            pool,
            actor_user_id,
            kind,
            &parsed.terms,
            "assignees",
            parsed.assignee.as_deref(),
        )
        .await?,
        reviewers: collaboration_array_facet(
            pool,
            actor_user_id,
            kind,
            &parsed.terms,
            "reviewers",
            parsed.reviewer.as_deref(),
        )
        .await?,
        milestones: collaboration_milestone_facet(
            pool,
            actor_user_id,
            kind,
            &parsed.terms,
            parsed.milestone.as_deref(),
        )
        .await?,
    })
}

async fn collaboration_scalar_facet(
    pool: &PgPool,
    actor_user_id: Uuid,
    kind: &str,
    terms: &str,
    key: &str,
    selected: Option<&str>,
) -> Result<Vec<CodeSearchFacetValue>, SearchError> {
    let rows = sqlx::query(
        r#"
        SELECT search_documents.metadata->>$4 AS value, count(*) AS count
        FROM search_documents
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = search_documents.repository_id
         AND repository_permissions.user_id = $1
        WHERE search_documents.kind = $2
          AND search_documents.metadata ? $4
          AND (
              search_documents.visibility = 'public'
              OR repository_permissions.user_id IS NOT NULL
              OR search_documents.owner_user_id = $1
          )
          AND (
              $3 = ''
              OR search_documents.search_vector @@ plainto_tsquery('simple', $3)
              OR search_documents.title ILIKE '%' || $3 || '%'
              OR search_documents.body ILIKE '%' || $3 || '%'
          )
        GROUP BY search_documents.metadata->>$4
        ORDER BY count DESC, value ASC
        LIMIT 12
        "#,
    )
    .bind(actor_user_id)
    .bind(kind)
    .bind(terms)
    .bind(key)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let value: Option<String> = row.get("value");
            let value = value?;
            Some(CodeSearchFacetValue {
                label: value.clone(),
                selected: selected.is_some_and(|active| active.eq_ignore_ascii_case(&value)),
                value,
                count: row.get("count"),
            })
        })
        .collect())
}

async fn collaboration_array_facet(
    pool: &PgPool,
    actor_user_id: Uuid,
    kind: &str,
    terms: &str,
    key: &str,
    selected: Option<&str>,
) -> Result<Vec<CodeSearchFacetValue>, SearchError> {
    let rows = sqlx::query(
        r#"
        SELECT COALESCE(item.value->>'login', item.value->>'name') AS value, count(*) AS count
        FROM search_documents
        CROSS JOIN LATERAL jsonb_array_elements(COALESCE(search_documents.metadata->$4, '[]'::jsonb)) AS item(value)
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = search_documents.repository_id
         AND repository_permissions.user_id = $1
        WHERE search_documents.kind = $2
          AND (
              search_documents.visibility = 'public'
              OR repository_permissions.user_id IS NOT NULL
              OR search_documents.owner_user_id = $1
          )
          AND (
              $3 = ''
              OR search_documents.search_vector @@ plainto_tsquery('simple', $3)
              OR search_documents.title ILIKE '%' || $3 || '%'
              OR search_documents.body ILIKE '%' || $3 || '%'
          )
        GROUP BY COALESCE(item.value->>'login', item.value->>'name')
        ORDER BY count DESC, value ASC
        LIMIT 12
        "#
    )
    .bind(actor_user_id)
    .bind(kind)
    .bind(terms)
    .bind(key)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let value: Option<String> = row.get("value");
            let value = value?.trim().to_owned();
            if value.is_empty() {
                return None;
            }
            Some(CodeSearchFacetValue {
                label: value.clone(),
                selected: selected.is_some_and(|active| active.eq_ignore_ascii_case(&value)),
                value,
                count: row.get("count"),
            })
        })
        .collect())
}

async fn collaboration_milestone_facet(
    pool: &PgPool,
    actor_user_id: Uuid,
    kind: &str,
    terms: &str,
    selected: Option<&str>,
) -> Result<Vec<CodeSearchFacetValue>, SearchError> {
    let rows = sqlx::query(
        r#"
        SELECT COALESCE(search_documents.metadata->'milestone'->>'title', search_documents.metadata->>'milestone') AS value,
               count(*) AS count
        FROM search_documents
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = search_documents.repository_id
         AND repository_permissions.user_id = $1
        WHERE search_documents.kind = $2
          AND COALESCE(search_documents.metadata->'milestone'->>'title', search_documents.metadata->>'milestone') IS NOT NULL
          AND (
              search_documents.visibility = 'public'
              OR repository_permissions.user_id IS NOT NULL
              OR search_documents.owner_user_id = $1
          )
          AND (
              $3 = ''
              OR search_documents.search_vector @@ plainto_tsquery('simple', $3)
              OR search_documents.title ILIKE '%' || $3 || '%'
              OR search_documents.body ILIKE '%' || $3 || '%'
          )
        GROUP BY COALESCE(search_documents.metadata->'milestone'->>'title', search_documents.metadata->>'milestone')
        ORDER BY count DESC, value ASC
        LIMIT 12
        "#
    )
    .bind(actor_user_id)
    .bind(kind)
    .bind(terms)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let value: Option<String> = row.get("value");
            let value = value?.trim().to_owned();
            if value.is_empty() {
                return None;
            }
            Some(CodeSearchFacetValue {
                label: value.clone(),
                selected: selected.is_some_and(|active| active.eq_ignore_ascii_case(&value)),
                value,
                count: row.get("count"),
            })
        })
        .collect())
}

async fn code_search_facets(
    pool: &PgPool,
    actor_user_id: Uuid,
    parsed: &ParsedCodeSearchQuery,
) -> Result<CodeSearchFacets, SearchError> {
    let repo_owner = parsed.repo.as_ref().map(|(owner, _)| owner.as_str());
    let repo_name = parsed.repo.as_ref().map(|(_, name)| name.as_str());
    let language_rows = sqlx::query(
        r#"
        SELECT search_documents.language AS value, count(*) AS count
        FROM search_documents
        LEFT JOIN repositories ON repositories.id = search_documents.repository_id
        LEFT JOIN users repo_owner_user ON repo_owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations repo_owner_org ON repo_owner_org.id = repositories.owner_organization_id
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = search_documents.repository_id
         AND repository_permissions.user_id = $1
        WHERE search_documents.kind = 'code'
          AND search_documents.language IS NOT NULL
          AND (
              search_documents.visibility = 'public'
              OR repository_permissions.user_id IS NOT NULL
              OR search_documents.owner_user_id = $1
          )
          AND (
              search_documents.search_vector @@ plainto_tsquery('simple', $2)
              OR search_documents.title ILIKE '%' || $2 || '%'
              OR search_documents.body ILIKE '%' || $2 || '%'
              OR search_documents.path ILIKE '%' || $2 || '%'
          )
          AND ($3::text IS NULL OR search_documents.path ILIKE '%' || $3 || '%')
          AND ($4::text IS NULL OR lower(COALESCE(repo_owner_user.username, repo_owner_user.email, repo_owner_org.slug)) = lower($4))
          AND ($5::text IS NULL OR lower(repositories.name) = lower($5))
          AND ($6::text IS NULL OR lower(COALESCE(repo_owner_user.username, repo_owner_user.email, repo_owner_org.slug)) = lower($6))
          AND ($7::text IS NULL OR search_documents.metadata->>'symbol' ILIKE '%' || $7 || '%' OR search_documents.body ILIKE '%' || $7 || '%')
          AND ($8::boolean IS NULL OR repositories.is_archived = $8)
        GROUP BY search_documents.language
        ORDER BY count DESC, search_documents.language ASC
        LIMIT 12
        "#,
    )
    .bind(actor_user_id)
    .bind(&parsed.terms)
    .bind(parsed.path.as_deref())
    .bind(repo_owner)
    .bind(repo_name)
    .bind(parsed.owner.as_deref())
    .bind(parsed.symbol.as_deref())
    .bind(parsed.archived)
    .fetch_all(pool)
    .await?;

    let path_rows = sqlx::query(
        r#"
        SELECT split_part(search_documents.path, '/', 1) AS value, count(*) AS count
        FROM search_documents
        LEFT JOIN repositories ON repositories.id = search_documents.repository_id
        LEFT JOIN users repo_owner_user ON repo_owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations repo_owner_org ON repo_owner_org.id = repositories.owner_organization_id
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = search_documents.repository_id
         AND repository_permissions.user_id = $1
        WHERE search_documents.kind = 'code'
          AND search_documents.path IS NOT NULL
          AND (
              search_documents.visibility = 'public'
              OR repository_permissions.user_id IS NOT NULL
              OR search_documents.owner_user_id = $1
          )
          AND (
              search_documents.search_vector @@ plainto_tsquery('simple', $2)
              OR search_documents.title ILIKE '%' || $2 || '%'
              OR search_documents.body ILIKE '%' || $2 || '%'
              OR search_documents.path ILIKE '%' || $2 || '%'
          )
          AND ($3::text IS NULL OR lower(search_documents.language) = lower($3))
          AND ($4::text IS NULL OR lower(COALESCE(repo_owner_user.username, repo_owner_user.email, repo_owner_org.slug)) = lower($4))
          AND ($5::text IS NULL OR lower(repositories.name) = lower($5))
          AND ($6::text IS NULL OR lower(COALESCE(repo_owner_user.username, repo_owner_user.email, repo_owner_org.slug)) = lower($6))
          AND ($7::text IS NULL OR search_documents.metadata->>'symbol' ILIKE '%' || $7 || '%' OR search_documents.body ILIKE '%' || $7 || '%')
          AND ($8::boolean IS NULL OR repositories.is_archived = $8)
        GROUP BY split_part(search_documents.path, '/', 1)
        ORDER BY count DESC, value ASC
        LIMIT 12
        "#,
    )
    .bind(actor_user_id)
    .bind(&parsed.terms)
    .bind(parsed.language.as_deref())
    .bind(repo_owner)
    .bind(repo_name)
    .bind(parsed.owner.as_deref())
    .bind(parsed.symbol.as_deref())
    .bind(parsed.archived)
    .fetch_all(pool)
    .await?;

    Ok(CodeSearchFacets {
        languages: language_rows
            .into_iter()
            .filter_map(|row| {
                let value: Option<String> = row.get("value");
                value.map(|value| CodeSearchFacetValue {
                    selected: parsed
                        .language
                        .as_ref()
                        .is_some_and(|selected| selected.eq_ignore_ascii_case(&value)),
                    label: value.clone(),
                    value,
                    count: row.get("count"),
                })
            })
            .collect(),
        paths: path_rows
            .into_iter()
            .filter_map(|row| {
                let value: Option<String> = row.get("value");
                value
                    .filter(|value| !value.is_empty())
                    .map(|value| CodeSearchFacetValue {
                        selected: parsed.path.as_ref().is_some_and(|selected| {
                            selected.eq_ignore_ascii_case(&value)
                                || selected
                                    .trim_end_matches('/')
                                    .eq_ignore_ascii_case(value.as_str())
                        }),
                        label: value.clone(),
                        value,
                        count: row.get("count"),
                    })
            })
            .collect(),
    })
}

fn parse_code_search_query(query: &str) -> Result<ParsedCodeSearchQuery, SearchError> {
    let normalized = query.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() > 256 {
        return Err(SearchError::Validation(
            "code search query must be 256 characters or fewer".to_owned(),
        ));
    }

    let mut terms = Vec::new();
    let mut qualifiers = Vec::new();
    let mut parsed = ParsedCodeSearchQuery {
        terms: String::new(),
        chips: Vec::new(),
        repo: None,
        owner: None,
        language: None,
        path: None,
        symbol: None,
        archived: None,
    };

    for token in normalized.split_whitespace() {
        if let Some((qualifier, value)) = token.split_once(':') {
            let qualifier = qualifier.to_ascii_lowercase();
            let value = value.trim();
            if is_probable_qualifier(&qualifier) {
                if value.is_empty() {
                    return Err(SearchError::Validation(format!(
                        "{qualifier}: requires a value"
                    )));
                }
                match qualifier.as_str() {
                    "repo" => {
                        let Some((owner, name)) = value.split_once('/') else {
                            return Err(SearchError::Validation(
                                "repo: requires owner/name".to_owned(),
                            ));
                        };
                        parsed.repo = Some((owner.to_owned(), name.to_owned()));
                    }
                    "org" | "user" => parsed.owner = Some(value.to_owned()),
                    "language" => parsed.language = Some(value.to_owned()),
                    "path" => parsed.path = Some(value.trim_matches('"').to_owned()),
                    "symbol" => parsed.symbol = Some(value.to_owned()),
                    "archived" => parsed.archived = Some(parse_bool_qualifier(value)?),
                    "is" => {
                        if value.eq_ignore_ascii_case("archived") {
                            parsed.archived = Some(true);
                        } else if value.eq_ignore_ascii_case("unarchived") {
                            parsed.archived = Some(false);
                        } else {
                            return Err(SearchError::Validation(format!(
                                "is:{value} is not supported for code search"
                            )));
                        }
                    }
                    _ => {
                        return Err(SearchError::Validation(format!(
                            "{qualifier}: is not supported for code search"
                        )));
                    }
                }
                qualifiers.push((qualifier, value.to_owned()));
                continue;
            }
        }
        terms.push(token.to_owned());
    }

    parsed.terms = terms.join(" ");
    parsed.chips = qualifiers
        .iter()
        .map(|(qualifier, value)| CodeSearchChip {
            qualifier: qualifier.clone(),
            value: value.clone(),
            label: format!("{qualifier}:{value}"),
            remove_query: remove_qualifier_token(&normalized, qualifier, value),
        })
        .collect();

    Ok(parsed)
}

fn is_probable_qualifier(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphabetic() || character == '_' || character == '-'
        })
}

fn parse_bool_qualifier(value: &str) -> Result<bool, SearchError> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" | "archived" => Ok(true),
        "false" | "no" | "unarchived" => Ok(false),
        _ => Err(SearchError::Validation(format!(
            "archived:{value} must be true or false"
        ))),
    }
}

fn remove_qualifier_token(query: &str, qualifier: &str, value: &str) -> String {
    let exact = format!("{qualifier}:{value}");
    let remainder = query
        .split_whitespace()
        .filter(|token| *token != exact)
        .collect::<Vec<_>>()
        .join(" ");
    if remainder.is_empty() {
        query.to_owned()
    } else {
        remainder
    }
}

fn code_snippet_for_document(document: &SearchDocument, query: &str) -> Option<SearchSnippet> {
    if document.kind != SearchDocumentKind::Code {
        return None;
    }

    let path = document.path.clone()?;
    let branch = document
        .branch
        .clone()
        .or_else(|| metadata_string(&document.metadata, "branch"))
        .unwrap_or_else(|| "main".to_owned());
    let line_number = document
        .metadata
        .get("lineNumber")
        .and_then(serde_json::Value::as_i64)
        .or_else(|| {
            document
                .metadata
                .get("line_number")
                .and_then(serde_json::Value::as_i64)
        });
    let fragment = metadata_string(&document.metadata, "fragment")
        .or_else(|| matching_line(&document.body, query))
        .unwrap_or_else(|| document.body.lines().next().unwrap_or("").trim().to_owned());

    Some(SearchSnippet {
        path,
        branch,
        line_number,
        match_ranges: match_ranges_for_fragment(&fragment, query),
        fragment,
        language: document.language.clone(),
    })
}

fn code_snippets_for_document(document: &SearchDocument, query: &str) -> Vec<SearchSnippet> {
    if document.kind != SearchDocumentKind::Code {
        return Vec::new();
    }

    let Some(path) = document.path.clone() else {
        return Vec::new();
    };
    let branch = document
        .branch
        .clone()
        .or_else(|| metadata_string(&document.metadata, "branch"))
        .unwrap_or_else(|| "main".to_owned());
    let language = document.language.clone();

    let mut snippets =
        metadata_snippets(&document.metadata, query, &path, &branch, language.clone());
    if snippets.is_empty() {
        snippets = body_snippets(document, query, &path, &branch, language);
    }
    if snippets.is_empty() {
        if let Some(snippet) = code_snippet_for_document(document, query) {
            snippets.push(snippet);
        }
    }
    snippets.sort_by_key(|snippet| snippet.line_number.unwrap_or(i64::MAX));
    snippets
}

fn metadata_snippets(
    metadata: &serde_json::Value,
    query: &str,
    path: &str,
    branch: &str,
    language: Option<String>,
) -> Vec<SearchSnippet> {
    let Some(values) = metadata
        .get("snippets")
        .or_else(|| metadata.get("matches"))
        .and_then(serde_json::Value::as_array)
    else {
        return Vec::new();
    };

    values
        .iter()
        .filter_map(|value| {
            let fragment = metadata_string(value, "fragment")
                .or_else(|| metadata_string(value, "text"))
                .or_else(|| metadata_string(value, "line"))?;
            let fragment = fragment.trim();
            if fragment.is_empty() {
                return None;
            }
            let line_number = value
                .get("lineNumber")
                .and_then(serde_json::Value::as_i64)
                .or_else(|| value.get("line_number").and_then(serde_json::Value::as_i64));
            Some(SearchSnippet {
                path: path.to_owned(),
                branch: branch.to_owned(),
                line_number,
                fragment: fragment.to_owned(),
                language: language.clone(),
                match_ranges: match_ranges_for_fragment(fragment, query),
            })
        })
        .collect()
}

fn body_snippets(
    document: &SearchDocument,
    query: &str,
    path: &str,
    branch: &str,
    language: Option<String>,
) -> Vec<SearchSnippet> {
    let terms = query_terms(query);
    document
        .body
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let fragment = line.trim();
            if fragment.is_empty() {
                return None;
            }
            let lower = fragment.to_ascii_lowercase();
            if !terms.is_empty() && !terms.iter().any(|term| lower.contains(term)) {
                return None;
            }
            Some(SearchSnippet {
                path: path.to_owned(),
                branch: branch.to_owned(),
                line_number: Some((index + 1) as i64),
                fragment: fragment.to_owned(),
                language: language.clone(),
                match_ranges: match_ranges_for_fragment(fragment, query),
            })
        })
        .take(20)
        .collect()
}

fn query_terms(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .filter(|token| !token.contains(':'))
        .map(|token| {
            token
                .trim_matches(|character: char| {
                    character == '"' || character == '\'' || !character.is_alphanumeric()
                })
                .to_ascii_lowercase()
        })
        .filter(|token| token.len() >= 2)
        .collect()
}

fn commit_summary_for_document(document: &SearchDocument) -> Option<SearchCommitSummary> {
    if document.kind != SearchDocumentKind::Commit {
        return None;
    }

    let (message_title, message_body) = split_commit_message(&document.title, &document.body);
    Some(SearchCommitSummary {
        oid: document.resource_id.clone(),
        short_oid: document.resource_id.chars().take(12).collect(),
        message_title,
        message_body,
        author_login: metadata_string(&document.metadata, "authorLogin"),
        committed_at: metadata_string(&document.metadata, "committedAt")
            .and_then(|value| value.parse::<DateTime<Utc>>().ok()),
    })
}

fn code_blob_href(
    document: &SearchDocument,
    owner_login: Option<&str>,
    repository_name: Option<&str>,
) -> Option<String> {
    if document.kind != SearchDocumentKind::Code {
        return None;
    }
    owner_login.zip(repository_name).map(|(owner, repo)| {
        let branch = document.branch.as_deref().unwrap_or("main");
        let path = document.path.as_deref().unwrap_or("");
        format!(
            "/{}/{}/blob/{}/{}",
            percent_encode_segment(owner),
            percent_encode_segment(repo),
            percent_encode_segment(branch),
            percent_encode_path(path)
        )
    })
}

fn metadata_string(metadata: &serde_json::Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn matching_line(body: &str, query: &str) -> Option<String> {
    let query = query.trim().to_ascii_lowercase();
    body.lines()
        .find(|line| line.to_ascii_lowercase().contains(&query))
        .or_else(|| body.lines().next())
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

fn match_ranges_for_fragment(fragment: &str, query: &str) -> Vec<SearchMatchRange> {
    let needle = query.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    let haystack = fragment.to_ascii_lowercase();
    let mut ranges = Vec::new();
    let mut offset = 0;
    while let Some(index) = haystack[offset..].find(&needle) {
        let start = offset + index;
        let end = start + needle.len();
        ranges.push(SearchMatchRange {
            start: start as i64,
            end: end as i64,
        });
        offset = end;
    }
    ranges
}

fn split_commit_message(title: &str, body: &str) -> (String, Option<String>) {
    let title = title.trim();
    let body = body.trim();
    let message_title = if title.is_empty() {
        body.lines().next().unwrap_or(body).trim().to_owned()
    } else {
        title.to_owned()
    };
    let message_body = if body.is_empty() {
        String::new()
    } else if let Some(rest) = body.strip_prefix(&message_title) {
        rest.trim().to_owned()
    } else if title.is_empty() {
        body.lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_owned()
    } else {
        body.to_owned()
    };
    (
        message_title,
        if message_body.is_empty() {
            None
        } else {
            Some(message_body)
        },
    )
}

fn ui_type_for_kind(kind: &SearchDocumentKind) -> &'static str {
    match kind {
        SearchDocumentKind::Repository => "repositories",
        SearchDocumentKind::Code => "code",
        SearchDocumentKind::Commit => "commits",
        SearchDocumentKind::Issue => "issues",
        SearchDocumentKind::PullRequest => "pull_requests",
        SearchDocumentKind::User => "users",
        SearchDocumentKind::Organization => "organizations",
        SearchDocumentKind::Package => "packages",
    }
}

fn ui_type_for_kind_str(kind: &str) -> &'static str {
    match kind {
        "repository" => "repositories",
        "code" => "code",
        "commit" => "commits",
        "issue" => "issues",
        "pull_request" => "pull_requests",
        "user" => "users",
        "organization" => "organizations",
        "package" => "packages",
        _ => "repositories",
    }
}

fn result_href(
    document: &SearchDocument,
    owner_login: Option<&str>,
    repository_name: Option<&str>,
) -> String {
    if let Some(href) = document
        .metadata
        .get("href")
        .and_then(serde_json::Value::as_str)
    {
        if href.starts_with('/') && !href.starts_with("//") {
            return href.to_owned();
        }
    }

    match document.kind {
        SearchDocumentKind::Repository => owner_login
            .zip(repository_name)
            .map(|(owner, repo)| format!("/{owner}/{repo}"))
            .unwrap_or_else(|| "/search?type=repositories".to_owned()),
        SearchDocumentKind::User => owner_login
            .map(|owner| format!("/{owner}"))
            .unwrap_or_else(|| "/search?type=users".to_owned()),
        SearchDocumentKind::Organization => owner_login
            .map(|org| format!("/orgs/{org}"))
            .unwrap_or_else(|| "/search?type=organizations".to_owned()),
        SearchDocumentKind::Code => owner_login
            .zip(repository_name)
            .map(|(owner, repo)| {
                let branch = document.branch.as_deref().unwrap_or("main");
                let path = document.path.as_deref().unwrap_or("");
                let line = document
                    .metadata
                    .get("lineNumber")
                    .and_then(serde_json::Value::as_i64)
                    .or_else(|| {
                        document
                            .metadata
                            .get("line_number")
                            .and_then(serde_json::Value::as_i64)
                    })
                    .filter(|line| *line > 0)
                    .map(|line| format!("#L{line}"))
                    .unwrap_or_default();
                format!(
                    "/{}/{}/blob/{}/{}{}",
                    percent_encode_segment(owner),
                    percent_encode_segment(repo),
                    percent_encode_segment(branch),
                    percent_encode_path(path),
                    line
                )
            })
            .unwrap_or_else(|| "/search?type=code".to_owned()),
        SearchDocumentKind::Commit => owner_login
            .zip(repository_name)
            .map(|(owner, repo)| {
                format!(
                    "/{}/{}/commit/{}",
                    percent_encode_segment(owner),
                    percent_encode_segment(repo),
                    percent_encode_segment(&document.resource_id)
                )
            })
            .unwrap_or_else(|| "/search?type=commits".to_owned()),
        SearchDocumentKind::Issue => owner_login
            .zip(repository_name)
            .map(|(owner, repo)| format!("/{owner}/{repo}/issues/{}", document.resource_id))
            .unwrap_or_else(|| "/search?type=issues".to_owned()),
        SearchDocumentKind::PullRequest => owner_login
            .zip(repository_name)
            .map(|(owner, repo)| format!("/{owner}/{repo}/pull/{}", document.resource_id))
            .unwrap_or_else(|| "/search?type=pull_requests".to_owned()),
        SearchDocumentKind::Package => "/search?type=packages".to_owned(),
    }
}

fn percent_encode_path(path: &str) -> String {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .map(percent_encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}

fn suggestion_href(
    kind: &str,
    owner_login: Option<&str>,
    repository_name: Option<&str>,
    branch: Option<&str>,
    path: Option<&str>,
) -> Option<String> {
    match kind {
        "repository" => owner_login
            .zip(repository_name)
            .map(|(owner, repo)| format!("/{owner}/{repo}")),
        "code" => owner_login.zip(repository_name).map(|(owner, repo)| {
            format!(
                "/{}/{}/blob/{}/{}",
                percent_encode_segment(owner),
                percent_encode_segment(repo),
                percent_encode_segment(branch.unwrap_or("main")),
                percent_encode_path(path.unwrap_or_default())
            )
        }),
        _ => None,
    }
}

fn suggestion_token(query: &str) -> Option<SearchSuggestionToken> {
    let trimmed_end = query.trim_end();
    if trimmed_end.is_empty() {
        return None;
    }
    let replace_to = trimmed_end.len();
    let replace_from = trimmed_end
        .char_indices()
        .rev()
        .find(|(_, ch)| ch.is_whitespace())
        .map(|(index, ch)| index + ch.len_utf8())
        .unwrap_or(0);
    let value = trimmed_end[replace_from..replace_to].to_owned();
    let prefix = value
        .split_once(':')
        .map(|(prefix, _)| prefix)
        .filter(|prefix| !prefix.is_empty())
        .map(ToOwned::to_owned);
    Some(SearchSuggestionToken {
        prefix,
        value,
        replace_from,
        replace_to,
    })
}

fn replace_token(query: &str, token: Option<&SearchSuggestionToken>, replacement: &str) -> String {
    let Some(token) = token else {
        return replacement.to_owned();
    };
    let mut next = String::new();
    next.push_str(&query[..token.replace_from]);
    next.push_str(replacement);
    if token.replace_to < query.len() {
        next.push_str(&query[token.replace_to..]);
    }
    next
}

fn percent_encode_query(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else if byte == b' ' {
            encoded.push('+');
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn percent_encode_segment(segment: &str) -> String {
    let mut encoded = String::new();
    for byte in segment.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn document_from_row(row: sqlx::postgres::PgRow) -> Result<SearchDocument, SearchError> {
    Ok(SearchDocument {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        owner_user_id: row.get("owner_user_id"),
        owner_organization_id: row.get("owner_organization_id"),
        kind: SearchDocumentKind::try_from(row.get::<String, _>("kind").as_str())?,
        resource_id: row.get("resource_id"),
        title: row.get("title"),
        body: row.get("body"),
        path: row.get("path"),
        language: row.get("language"),
        branch: row.get("branch"),
        visibility: RepositoryVisibility::try_from(row.get::<String, _>("visibility").as_str())?,
        metadata: row.get("metadata"),
        indexed_at: row.get("indexed_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
