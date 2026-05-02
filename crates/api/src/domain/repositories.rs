use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::api_types::ListEnvelope;

use super::permissions::RepositoryRole;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryVisibility {
    #[default]
    Public,
    Private,
    Internal,
}

impl RepositoryVisibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::Internal => "internal",
        }
    }
}

impl TryFrom<&str> for RepositoryVisibility {
    type Error = RepositoryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            "internal" => Ok(Self::Internal),
            other => Err(RepositoryError::InvalidVisibility(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryOwner {
    User { id: Uuid },
    Organization { id: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Organization {
    pub id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub owner_user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Repository {
    pub id: Uuid,
    pub owner_user_id: Option<Uuid>,
    pub owner_organization_id: Option<Uuid>,
    pub owner_login: String,
    pub name: String,
    pub description: Option<String>,
    pub visibility: RepositoryVisibility,
    pub default_branch: String,
    pub is_archived: bool,
    pub created_by_user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryFile {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub commit_id: Uuid,
    pub path: String,
    pub content: String,
    pub oid: String,
    pub byte_size: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryTreeEntry {
    pub kind: String,
    pub name: String,
    pub path: String,
    pub href: String,
    pub byte_size: Option<i64>,
    pub latest_commit_message: Option<String>,
    pub latest_commit_href: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPathBreadcrumb {
    pub name: String,
    pub path: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPathOverview {
    #[serde(flatten)]
    pub repository: Repository,
    pub viewer_permission: Option<String>,
    pub ref_name: String,
    pub resolved_ref: RepositoryResolvedRef,
    pub default_branch_href: String,
    pub recovery_href: String,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub has_more: bool,
    pub path: String,
    pub path_name: String,
    pub breadcrumbs: Vec<RepositoryPathBreadcrumb>,
    pub parent_href: Option<String>,
    pub entries: Vec<RepositoryTreeEntry>,
    pub readme: Option<RepositoryFile>,
    pub latest_commit: Option<RepositoryLatestCommit>,
    pub history_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBlobView {
    #[serde(flatten)]
    pub repository: Repository,
    pub viewer_permission: Option<String>,
    pub ref_name: String,
    pub resolved_ref: RepositoryResolvedRef,
    pub default_branch_href: String,
    pub recovery_href: String,
    pub path: String,
    pub path_name: String,
    pub breadcrumbs: Vec<RepositoryPathBreadcrumb>,
    pub parent_href: Option<String>,
    pub file: RepositoryFile,
    pub language: Option<String>,
    pub is_binary: bool,
    pub is_large: bool,
    pub line_count: i64,
    pub loc_count: i64,
    pub size_label: String,
    pub mime_type: String,
    pub render_mode: String,
    pub display_content: Option<String>,
    pub latest_commit: Option<RepositoryLatestCommit>,
    pub latest_path_commit: Option<RepositoryLatestCommit>,
    pub history_href: String,
    pub raw_href: String,
    pub download_href: String,
    pub raw_api_href: String,
    pub download_api_href: String,
    pub permalink_href: String,
    pub symbols: Vec<RepositoryCodeSymbol>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCodeSymbol {
    pub kind: String,
    pub name: String,
    pub line_number: i64,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBlameView {
    #[serde(flatten)]
    pub blob: RepositoryBlobView,
    pub lines: Vec<RepositoryBlameLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBlameLine {
    pub line_number: i64,
    pub content: String,
    pub commit: RepositoryBlameCommit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryBlameCommit {
    pub oid: String,
    pub short_oid: String,
    pub message: String,
    pub href: String,
    pub committed_at: DateTime<Utc>,
    pub author_login: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCommitHistoryItem {
    pub oid: String,
    pub short_oid: String,
    pub message: String,
    pub href: String,
    pub committed_at: DateTime<Utc>,
    pub author_login: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct RepositoryCommitHistoryQuery<'a> {
    pub ref_name: Option<&'a str>,
    pub path: Option<&'a str>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct RepositoryRefsQuery<'a> {
    pub query: Option<&'a str>,
    pub current_path: Option<&'a str>,
    pub active_ref: Option<&'a str>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct RepositoryPathQuery<'a> {
    pub ref_name: Option<&'a str>,
    pub path: &'a str,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct RepositoryFileFinderQuery<'a> {
    pub ref_name: Option<&'a str>,
    pub query: Option<&'a str>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryLatestCommit {
    pub oid: String,
    pub short_oid: String,
    pub message: String,
    pub href: String,
    pub committed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryLanguageSummary {
    pub language: String,
    pub color: String,
    pub byte_count: i64,
    pub percentage: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCloneUrls {
    pub https: String,
    pub git: String,
    pub zip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySidebarMetadata {
    pub about: Option<String>,
    pub website_url: Option<String>,
    pub topics: Vec<String>,
    pub stars_count: i64,
    pub watchers_count: i64,
    pub forks_count: i64,
    pub releases_count: i64,
    pub deployments_count: i64,
    pub contributors_count: i64,
    pub languages: Vec<RepositoryLanguageSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryViewerState {
    pub starred: bool,
    pub watching: bool,
    pub forked_repository_href: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySocialState {
    pub starred: bool,
    pub watching: bool,
    pub stars_count: i64,
    pub watchers_count: i64,
    pub forks_count: i64,
    pub forked_repository_href: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryForkResult {
    pub source_repository_id: Uuid,
    pub fork_repository: Repository,
    pub fork_href: String,
    pub social: RepositorySocialState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryOverview {
    #[serde(flatten)]
    pub repository: Repository,
    pub viewer_permission: Option<String>,
    pub branch_count: i64,
    pub tag_count: i64,
    pub default_branch_ref: Option<GitRef>,
    pub latest_commit: Option<RepositoryLatestCommit>,
    pub root_entries: Vec<RepositoryTreeEntry>,
    pub files: Vec<RepositoryFile>,
    pub readme: Option<RepositoryFile>,
    pub sidebar: RepositorySidebarMetadata,
    pub viewer_state: RepositoryViewerState,
    pub clone_urls: RepositoryCloneUrls,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySettingsFeatureFlags {
    pub issues: bool,
    pub projects: bool,
    pub wiki: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySettingsMergeMethods {
    pub merge_commit: bool,
    pub squash: bool,
    pub rebase: bool,
    pub auto_merge: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySettingsCapabilities {
    pub rename: bool,
    pub archive: bool,
    pub transfer: bool,
    pub change_visibility: bool,
    pub delete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySettingsView {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub description: Option<String>,
    pub visibility: RepositoryVisibility,
    pub default_branch: String,
    pub is_archived: bool,
    pub is_template: bool,
    pub allow_forking: bool,
    pub web_commit_signoff_required: bool,
    pub features: RepositorySettingsFeatureFlags,
    pub merge_methods: RepositorySettingsMergeMethods,
    pub capabilities: RepositorySettingsCapabilities,
    pub viewer_permission: String,
    pub audit_event_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRepositorySettings {
    pub name: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<RepositoryVisibility>,
    pub default_branch: Option<String>,
    pub is_template: Option<bool>,
    pub allow_forking: Option<bool>,
    pub web_commit_signoff_required: Option<bool>,
    pub features: Option<RepositorySettingsFeatureFlags>,
    pub merge_methods: Option<RepositorySettingsMergeMethods>,
}

pub struct RepositoryPermission {
    pub repository_id: Uuid,
    pub user_id: Uuid,
    pub role: RepositoryRole,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WritableRepositoryOwner {
    pub owner_type: String,
    pub id: Uuid,
    pub login: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryTemplateOption {
    pub slug: String,
    pub display_name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitignoreTemplateOption {
    pub slug: String,
    pub display_name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LicenseTemplateOption {
    pub slug: String,
    pub display_name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryCreationOptions {
    pub owners: Vec<WritableRepositoryOwner>,
    pub templates: Vec<RepositoryTemplateOption>,
    pub gitignore_templates: Vec<GitignoreTemplateOption>,
    pub license_templates: Vec<LicenseTemplateOption>,
    pub suggested_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryNameAvailability {
    pub owner_type: String,
    pub owner_id: Uuid,
    pub owner_login: String,
    pub requested_name: String,
    pub normalized_name: String,
    pub available: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Commit {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub oid: String,
    pub author_user_id: Option<Uuid>,
    pub committer_user_id: Option<Uuid>,
    pub message: String,
    pub tree_oid: Option<String>,
    pub parent_oids: Vec<String>,
    pub committed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitRef {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub name: String,
    pub kind: String,
    pub target_commit_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryResolvedRef {
    pub kind: String,
    pub short_name: String,
    pub qualified_name: String,
    pub target_oid: Option<String>,
    pub recovery_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryRefSummary {
    pub name: String,
    pub short_name: String,
    pub kind: String,
    pub href: String,
    pub same_path_href: String,
    pub active: bool,
    pub target_short_oid: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryFileFinderItem {
    pub path: String,
    pub name: String,
    pub kind: String,
    pub href: String,
    pub byte_size: i64,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryFileFinderResult {
    #[serde(flatten)]
    pub envelope: ListEnvelope<RepositoryFileFinderItem>,
    pub resolved_ref: RepositoryResolvedRef,
    pub default_branch_href: String,
    pub recovery_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrganization {
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub owner_user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepository {
    pub owner: RepositoryOwner,
    pub name: String,
    pub description: Option<String>,
    pub visibility: RepositoryVisibility,
    pub default_branch: Option<String>,
    pub created_by_user_id: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepositoryBootstrapRequest {
    pub initialize_readme: bool,
    pub template_slug: Option<String>,
    pub gitignore_template_slug: Option<String>,
    pub license_template_slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BootstrapFile {
    pub path: String,
    pub content: String,
    pub oid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositorySnapshotFile {
    pub path: String,
    pub content: String,
    pub oid: String,
    pub byte_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySnapshot {
    pub commit: CreateCommit,
    pub branch_name: String,
    pub files: Vec<RepositorySnapshotFile>,
}

struct IndexedSearchFile {
    path: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommit {
    pub oid: String,
    pub author_user_id: Option<Uuid>,
    pub committer_user_id: Option<Uuid>,
    pub message: String,
    pub tree_oid: Option<String>,
    pub parent_oids: Vec<String>,
    pub committed_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("repository owner was not found")]
    OwnerNotFound,
    #[error("user does not have permission to create repositories for this owner")]
    OwnerPermissionDenied,
    #[error("user does not have repository access")]
    PermissionDenied,
    #[error("repository was not found")]
    NotFound,
    #[error("repository path was not found")]
    PathNotFound,
    #[error("repository ref was not found")]
    RefNotFound,
    #[error("repository ref `{ref_name}` was not found")]
    RefNotFoundWithRecovery {
        ref_name: String,
        recovery_href: String,
        default_branch_href: String,
    },
    #[error("repository path `{path}` was not found")]
    PathNotFoundWithRecovery {
        path: String,
        recovery_href: String,
        default_branch_href: String,
    },
    #[error("invalid repository visibility `{0}`")]
    InvalidVisibility(String),
    #[error("invalid repository name `{0}`")]
    InvalidName(String),
    #[error("invalid repository description `{0}`")]
    InvalidDescription(String),
    #[error("unknown repository template `{0}`")]
    UnknownTemplate(String),
    #[error("unknown gitignore template `{0}`")]
    UnknownGitignoreTemplate(String),
    #[error("unknown license template `{0}`")]
    UnknownLicenseTemplate(String),
    #[error("at least one pull request merge method must stay enabled")]
    NoMergeMethodEnabled,
    #[error("repository has already been forked by this user")]
    ForkAlreadyExists,
    #[error("repository git storage failed")]
    GitStorageFailed,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

pub async fn repository_creation_options(
    pool: &PgPool,
    actor_user_id: Uuid,
) -> Result<RepositoryCreationOptions, RepositoryError> {
    let owner_rows = sqlx::query(
        r#"
        SELECT 'user' AS owner_type,
               users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               COALESCE(users.display_name, users.email) AS display_name,
               users.avatar_url,
               0 AS sort_order
        FROM users
        WHERE users.id = $1

        UNION ALL

        SELECT 'organization' AS owner_type,
               organizations.id,
               organizations.slug AS login,
               organizations.display_name,
               NULL::text AS avatar_url,
               1 AS sort_order
        FROM organizations
        JOIN organization_memberships
          ON organization_memberships.organization_id = organizations.id
        WHERE organization_memberships.user_id = $1
          AND organization_memberships.role IN ('owner', 'admin')
        ORDER BY sort_order ASC, login ASC
        "#,
    )
    .bind(actor_user_id)
    .fetch_all(pool)
    .await?;

    let owners = owner_rows
        .into_iter()
        .map(|row| WritableRepositoryOwner {
            owner_type: row.get("owner_type"),
            id: row.get("id"),
            login: row.get("login"),
            display_name: row.get("display_name"),
            avatar_url: row.get("avatar_url"),
        })
        .collect();

    let templates = sqlx::query(
        r#"
        SELECT slug, display_name, description
        FROM repository_creation_templates
        ORDER BY sort_order ASC, display_name ASC
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| RepositoryTemplateOption {
        slug: row.get("slug"),
        display_name: row.get("display_name"),
        description: row.get("description"),
    })
    .collect();

    let gitignore_templates = sqlx::query(
        r#"
        SELECT slug, display_name, description
        FROM gitignore_templates
        ORDER BY sort_order ASC, display_name ASC
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| GitignoreTemplateOption {
        slug: row.get("slug"),
        display_name: row.get("display_name"),
        description: row.get("description"),
    })
    .collect();

    let license_templates = sqlx::query(
        r#"
        SELECT slug, display_name, description
        FROM license_templates
        ORDER BY sort_order ASC, display_name ASC
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| LicenseTemplateOption {
        slug: row.get("slug"),
        display_name: row.get("display_name"),
        description: row.get("description"),
    })
    .collect();

    Ok(RepositoryCreationOptions {
        owners,
        templates,
        gitignore_templates,
        license_templates,
        suggested_name: suggested_repository_name(actor_user_id),
    })
}

pub async fn repository_name_availability(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner: RepositoryOwner,
    requested_name: &str,
) -> Result<RepositoryNameAvailability, RepositoryError> {
    ensure_owner_can_create(pool, &owner, actor_user_id).await?;
    let (owner_type, owner_id, owner_login) = repository_owner_login(pool, &owner).await?;
    let normalized_name = normalize_repository_name(requested_name);
    let mut reason = validate_repository_name(&normalized_name).err();
    let exists = if reason.is_none() {
        repository_exists_for_owner(pool, &owner, &normalized_name).await?
    } else {
        false
    };
    if exists {
        reason = Some("A repository with this name already exists for this owner.".to_owned());
    }

    Ok(RepositoryNameAvailability {
        owner_type,
        owner_id,
        owner_login,
        requested_name: requested_name.to_owned(),
        normalized_name,
        available: reason.is_none() && !exists,
        reason,
    })
}

pub async fn create_organization(
    pool: &PgPool,
    input: CreateOrganization,
) -> Result<Organization, RepositoryError> {
    let row = sqlx::query(
        r#"
        INSERT INTO organizations (slug, display_name, description, owner_user_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, slug, display_name, description, owner_user_id, created_at, updated_at
        "#,
    )
    .bind(&input.slug)
    .bind(&input.display_name)
    .bind(&input.description)
    .bind(input.owner_user_id)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO organization_memberships (organization_id, user_id, role)
        VALUES ($1, $2, 'owner')
        ON CONFLICT (organization_id, user_id) DO UPDATE SET role = 'owner'
        "#,
    )
    .bind(row.get::<Uuid, _>("id"))
    .bind(input.owner_user_id)
    .execute(pool)
    .await?;

    Ok(organization_from_row(row))
}

pub async fn create_repository(
    pool: &PgPool,
    input: CreateRepository,
) -> Result<Repository, RepositoryError> {
    create_repository_with_bootstrap(pool, input, RepositoryBootstrapRequest::default()).await
}

pub async fn create_repository_with_bootstrap(
    pool: &PgPool,
    input: CreateRepository,
    bootstrap: RepositoryBootstrapRequest,
) -> Result<Repository, RepositoryError> {
    ensure_owner_can_create(pool, &input.owner, input.created_by_user_id).await?;
    let normalized_name = normalize_repository_name(&input.name);
    validate_repository_name(&normalized_name).map_err(RepositoryError::InvalidName)?;
    let description = normalize_repository_description(input.description)?;

    let (owner_user_id, owner_organization_id) = match input.owner {
        RepositoryOwner::User { id } => (Some(id), None),
        RepositoryOwner::Organization { id } => (None, Some(id)),
    };

    let repository_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO repositories (
            owner_user_id,
            owner_organization_id,
            name,
            description,
            visibility,
            default_branch,
            created_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, COALESCE($6, 'main'), $7)
        RETURNING id
        "#,
    )
    .bind(owner_user_id)
    .bind(owner_organization_id)
    .bind(&normalized_name)
    .bind(&description)
    .bind(input.visibility.as_str())
    .bind(&input.default_branch)
    .bind(input.created_by_user_id)
    .fetch_one(pool)
    .await?;

    let repository = get_repository(pool, repository_id)
        .await?
        .ok_or(RepositoryError::NotFound)?;
    grant_repository_permission(
        pool,
        repository.id,
        input.created_by_user_id,
        RepositoryRole::Owner,
        "owner",
    )
    .await?;
    ensure_default_repository_labels(pool, repository.id).await?;
    bootstrap_repository(pool, &repository, input.created_by_user_id, &bootstrap).await?;
    Ok(repository)
}

pub async fn repository_overview_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryOverview>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    Ok(Some(
        repository_overview_for_actor(pool, repository, actor_user_id).await?,
    ))
}

pub async fn repository_overview_for_viewer_by_owner_name(
    pool: &PgPool,
    actor_user_id: Option<Uuid>,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryOverview>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    repository_overview_for_viewer(pool, repository, actor_user_id)
        .await
        .map(Some)
}

pub async fn repository_refs_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    query: RepositoryRefsQuery<'_>,
) -> Result<Option<ListEnvelope<RepositoryRefSummary>>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    let normalized_query = query.query.unwrap_or("").trim().to_lowercase();
    let current_path = normalize_repository_path(query.current_path.unwrap_or(""))?;
    let active_ref = query
        .active_ref
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&repository.default_branch);
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, 100);

    let rows = sqlx::query(
        r#"
        SELECT repository_git_refs.name,
               repository_git_refs.kind,
               repository_git_refs.updated_at,
               commits.oid AS target_oid,
               commits.id AS target_commit_id
        FROM repository_git_refs
        LEFT JOIN commits ON commits.id = repository_git_refs.target_commit_id
        WHERE repository_git_refs.repository_id = $1
        ORDER BY repository_git_refs.kind ASC, lower(repository_git_refs.name) ASC
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;

    let mut items = Vec::new();
    for row in rows {
        let name: String = row.get("name");
        let kind: String = row.get("kind");
        let short_name = name
            .strip_prefix("refs/heads/")
            .or_else(|| name.strip_prefix("refs/tags/"))
            .unwrap_or(&name)
            .to_owned();
        if !normalized_query.is_empty()
            && !short_name.to_lowercase().contains(&normalized_query)
            && !name.to_lowercase().contains(&normalized_query)
        {
            continue;
        }
        let target_commit_id = row.get::<Option<Uuid>, _>("target_commit_id");
        let same_path_href = if current_path.is_empty()
            || repository_path_exists_for_commit(
                pool,
                repository.id,
                target_commit_id,
                &current_path,
            )
            .await?
        {
            repository_tree_href(&repository, &short_name, &current_path)
        } else {
            repository_tree_href(&repository, &short_name, "")
        };
        let active = ref_matches_active(&name, &short_name, active_ref);
        items.push(RepositoryRefSummary {
            href: same_path_href.clone(),
            same_path_href,
            active,
            target_short_oid: row
                .get::<Option<String>, _>("target_oid")
                .map(|oid| oid.chars().take(7).collect()),
            updated_at: row.get("updated_at"),
            name,
            short_name,
            kind,
        });
    }
    let total = items.len() as i64;
    let offset = ((page - 1) * page_size) as usize;
    items = items
        .into_iter()
        .skip(offset)
        .take(page_size as usize)
        .collect();

    Ok(Some(ListEnvelope {
        total,
        page,
        page_size,
        items,
    }))
}

pub async fn repository_file_finder_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    query: RepositoryFileFinderQuery<'_>,
) -> Result<Option<RepositoryFileFinderResult>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    let resolved_ref = resolve_repository_ref(pool, &repository, query.ref_name).await?;
    let normalized_query = query.query.unwrap_or("").trim().to_lowercase();
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, 100);
    let files = list_repository_files_for_resolved_ref(pool, repository.id, &resolved_ref).await?;
    let mut items = files
        .into_iter()
        .filter(|file| {
            normalized_query.is_empty() || file.path.to_lowercase().contains(&normalized_query)
        })
        .map(|file| {
            let name = file
                .path
                .rsplit('/')
                .next()
                .filter(|value| !value.is_empty())
                .unwrap_or(&file.path)
                .to_owned();
            RepositoryFileFinderItem {
                href: format!(
                    "/{}/{}/blob/{}/{}",
                    repository.owner_login,
                    repository.name,
                    percent_encode_segment(&resolved_ref.short_name),
                    percent_encode_path(&file.path)
                ),
                language: language_for_path(&file.path),
                byte_size: file.byte_size,
                kind: "file".to_owned(),
                name,
                path: file.path,
            }
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.path.to_lowercase().cmp(&right.path.to_lowercase()));
    let total = items.len() as i64;
    let offset = ((page - 1) * page_size) as usize;
    items = items
        .into_iter()
        .skip(offset)
        .take(page_size as usize)
        .collect();

    Ok(Some(RepositoryFileFinderResult {
        default_branch_href: repository_default_branch_href(&repository),
        recovery_href: repository_default_branch_href(&repository),
        resolved_ref,
        envelope: ListEnvelope {
            total,
            page,
            page_size,
            items,
        },
    }))
}

pub async fn repository_overview_for_actor(
    pool: &PgPool,
    repository: Repository,
    actor_user_id: Uuid,
) -> Result<RepositoryOverview, RepositoryError> {
    repository_overview_for_viewer(pool, repository, Some(actor_user_id)).await
}

pub async fn repository_overview_for_viewer(
    pool: &PgPool,
    repository: Repository,
    actor_user_id: Option<Uuid>,
) -> Result<RepositoryOverview, RepositoryError> {
    let files = match resolve_repository_ref(pool, &repository, None).await {
        Ok(resolved_ref) => {
            list_repository_files_for_resolved_ref(pool, repository.id, &resolved_ref).await?
        }
        Err(RepositoryError::RefNotFoundWithRecovery { .. }) => Vec::new(),
        Err(error) => return Err(error),
    };
    let readme = files
        .iter()
        .find(|file| file.path.eq_ignore_ascii_case("README.md"))
        .cloned();
    let viewer_permission = match actor_user_id {
        Some(user_id) => repository_permission_for_user(pool, repository.id, user_id)
            .await?
            .map(|permission| permission.role.as_str().to_owned())
            .or_else(|| {
                if repository.visibility == RepositoryVisibility::Public {
                    Some("read".to_owned())
                } else {
                    None
                }
            }),
        None if repository.visibility == RepositoryVisibility::Public => Some("read".to_owned()),
        None => None,
    };
    if viewer_permission.is_none() {
        return Err(RepositoryError::PermissionDenied);
    }
    if let Some(user_id) = actor_user_id {
        record_recent_repository_visit(pool, user_id, repository.id).await?;
    }
    let branch_count = count_repository_refs(pool, repository.id, "branch").await?;
    let tag_count = count_repository_refs(pool, repository.id, "tag").await?;
    let default_branch_ref = get_repository_ref(
        pool,
        repository.id,
        &format!("refs/heads/{}", repository.default_branch),
    )
    .await?;
    let latest_commit = latest_commit_for_repository(pool, &repository).await?;
    let root_entries = repository_root_entries(&repository, &files, latest_commit.as_ref());
    let sidebar = repository_sidebar_metadata(pool, &repository).await?;
    let viewer_state = match actor_user_id {
        Some(user_id) => repository_viewer_state(pool, &repository, user_id).await?,
        None => RepositoryViewerState {
            starred: false,
            watching: false,
            forked_repository_href: None,
        },
    };
    let clone_urls = repository_clone_urls(&repository);
    Ok(RepositoryOverview {
        repository,
        viewer_permission,
        branch_count,
        tag_count,
        default_branch_ref,
        latest_commit,
        root_entries,
        files,
        readme,
        sidebar,
        viewer_state,
        clone_urls,
    })
}

pub async fn set_repository_star_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    starred: bool,
) -> Result<Option<RepositorySocialState>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };

    if starred {
        sqlx::query(
            r#"
            INSERT INTO repository_stars (user_id, repository_id)
            VALUES ($1, $2)
            ON CONFLICT (user_id, repository_id) DO NOTHING
            "#,
        )
        .bind(actor_user_id)
        .bind(repository.id)
        .execute(pool)
        .await?;
        insert_repository_social_feed_event(pool, &repository, actor_user_id, "star").await?;
    } else {
        sqlx::query("DELETE FROM repository_stars WHERE user_id = $1 AND repository_id = $2")
            .bind(actor_user_id)
            .bind(repository.id)
            .execute(pool)
            .await?;
    }

    repository_social_state(pool, &repository, actor_user_id)
        .await
        .map(Some)
}

pub async fn set_repository_watch_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    watching: bool,
) -> Result<Option<RepositorySocialState>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };

    if watching {
        sqlx::query(
            r#"
            INSERT INTO repository_watches (user_id, repository_id, reason)
            VALUES ($1, $2, 'subscribed')
            ON CONFLICT (user_id, repository_id) DO UPDATE SET reason = EXCLUDED.reason
            "#,
        )
        .bind(actor_user_id)
        .bind(repository.id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query("DELETE FROM repository_watches WHERE user_id = $1 AND repository_id = $2")
            .bind(actor_user_id)
            .bind(repository.id)
            .execute(pool)
            .await?;
    }

    repository_social_state(pool, &repository, actor_user_id)
        .await
        .map(Some)
}

pub async fn fork_repository_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryForkResult>, RepositoryError> {
    let Some(source_repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };

    if existing_fork_href_for_user(pool, source_repository.id, actor_user_id)
        .await?
        .is_some()
    {
        return Err(RepositoryError::ForkAlreadyExists);
    }

    let fork_repository = create_repository_with_bootstrap(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: actor_user_id },
            name: source_repository.name.clone(),
            description: source_repository.description.clone(),
            visibility: source_repository.visibility.clone(),
            default_branch: Some(source_repository.default_branch.clone()),
            created_by_user_id: actor_user_id,
        },
        RepositoryBootstrapRequest::default(),
    )
    .await?;

    copy_repository_snapshot(pool, &source_repository, &fork_repository, actor_user_id).await?;
    sqlx::query(
        r#"
        INSERT INTO repository_forks (source_repository_id, fork_repository_id, forked_by_user_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(source_repository.id)
    .bind(fork_repository.id)
    .bind(actor_user_id)
    .execute(pool)
    .await?;
    insert_repository_fork_feed_event(pool, &source_repository, &fork_repository, actor_user_id)
        .await?;
    let fork_href = format!("/{}/{}", fork_repository.owner_login, fork_repository.name);
    let social = repository_social_state(pool, &source_repository, actor_user_id).await?;

    Ok(Some(RepositoryForkResult {
        source_repository_id: source_repository.id,
        fork_repository,
        fork_href,
        social,
    }))
}

pub async fn repository_path_overview_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    query: RepositoryPathQuery<'_>,
) -> Result<Option<RepositoryPathOverview>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    repository_path_overview_for_actor(pool, repository, actor_user_id, query)
        .await
        .map(Some)
}

pub async fn repository_blob_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    ref_name: Option<&str>,
    path: &str,
) -> Result<Option<RepositoryBlobView>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    repository_blob_for_actor(pool, repository, actor_user_id, ref_name, path)
        .await
        .map(Some)
}

pub async fn repository_blame_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    ref_name: Option<&str>,
    path: &str,
) -> Result<Option<RepositoryBlameView>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    repository_blame_for_actor(pool, repository, actor_user_id, ref_name, path)
        .await
        .map(Some)
}

pub async fn repository_commit_history_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    query: RepositoryCommitHistoryQuery<'_>,
) -> Result<Option<ListEnvelope<RepositoryCommitHistoryItem>>, RepositoryError> {
    let Some(repository) =
        get_repository_for_actor_by_owner_name(pool, actor_user_id, owner_login, name).await?
    else {
        return Ok(None);
    };
    let resolved_ref = resolve_repository_ref(pool, &repository, query.ref_name).await?;
    let path = normalize_repository_path(query.path.unwrap_or(""))?;
    let files = list_repository_files_for_resolved_ref(pool, repository.id, &resolved_ref).await?;
    if !path.is_empty()
        && !files
            .iter()
            .any(|file| file.path == path || file.path.starts_with(&format!("{path}/")))
    {
        return Err(repository_path_not_found_error(&repository, &path));
    }
    repository_commit_history(
        pool,
        &repository,
        &resolved_ref.short_name,
        Some(path.as_str()).filter(|value| !value.is_empty()),
        query.page,
        query.page_size,
    )
    .await
    .map(Some)
}

async fn repository_path_overview_for_actor(
    pool: &PgPool,
    repository: Repository,
    actor_user_id: Uuid,
    query: RepositoryPathQuery<'_>,
) -> Result<RepositoryPathOverview, RepositoryError> {
    let resolved_ref = resolve_repository_ref(pool, &repository, query.ref_name).await?;
    let ref_name = resolved_ref.short_name.clone();
    let path = normalize_repository_path(query.path)?;
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, 100);
    let files = list_repository_files_for_resolved_ref(pool, repository.id, &resolved_ref).await?;
    let all_entries = repository_entries_for_path(&repository, &ref_name, &files, &path);
    let readme = readme_for_path(&files, &path);
    if !path.is_empty() && all_entries.is_empty() && readme.is_none() {
        return Err(repository_path_not_found_error(&repository, &path));
    }
    let total = all_entries.len() as i64;
    let offset = ((page - 1) * page_size) as usize;
    let entries = all_entries
        .into_iter()
        .skip(offset)
        .take(page_size as usize)
        .collect::<Vec<_>>();
    let has_more = (offset as i64) + (entries.len() as i64) < total;
    let latest_commit = latest_commit_for_repository(pool, &repository).await?;
    let viewer_permission = viewer_permission_for_user(pool, &repository, actor_user_id).await?;
    let history_href = repository_history_href(&repository, &ref_name, &path);

    Ok(RepositoryPathOverview {
        viewer_permission,
        ref_name: ref_name.clone(),
        resolved_ref,
        default_branch_href: repository_default_branch_href(&repository),
        recovery_href: repository_tree_href(&repository, &ref_name, &path),
        total,
        page,
        page_size,
        has_more,
        path_name: path
            .rsplit('/')
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or(&repository.name)
            .to_owned(),
        breadcrumbs: repository_breadcrumbs(&repository, &ref_name, &path),
        parent_href: repository_parent_tree_href(&repository, &ref_name, &path),
        entries,
        readme,
        latest_commit,
        history_href,
        path,
        repository,
    })
}

async fn repository_blob_for_actor(
    pool: &PgPool,
    repository: Repository,
    actor_user_id: Uuid,
    ref_name: Option<&str>,
    path: &str,
) -> Result<RepositoryBlobView, RepositoryError> {
    let resolved_ref = resolve_repository_ref(pool, &repository, ref_name).await?;
    let ref_name = resolved_ref.short_name.clone();
    let path = normalize_repository_path(path)?;
    if path.is_empty() {
        return Err(repository_path_not_found_error(&repository, &path));
    }
    let files = list_repository_files_for_resolved_ref(pool, repository.id, &resolved_ref).await?;
    let file = files
        .iter()
        .find(|file| file.path == path)
        .cloned()
        .ok_or_else(|| repository_path_not_found_error(&repository, &path))?;
    let latest_commit = latest_commit_for_repository(pool, &repository).await?;
    let latest_path_commit = latest_commit_for_file(pool, &repository, &file).await?;
    let viewer_permission = viewer_permission_for_user(pool, &repository, actor_user_id).await?;
    record_recent_repository_visit(pool, actor_user_id, repository.id).await?;
    let encoded_path = percent_encode_path(&path);
    let base = format!(
        "/{}/{}/{}",
        repository.owner_login, repository.name, encoded_path
    );
    let ref_segment = percent_encode_segment(&ref_name);
    let api_base = format!(
        "/api/repos/{}/{}/blobs/{}?ref={}",
        percent_encode_segment(&repository.owner_login),
        percent_encode_segment(&repository.name),
        encoded_path,
        ref_segment
    );
    let is_binary = is_probably_binary(&file.content);
    let is_large = file.byte_size > 512 * 1024;
    let display_content = if is_binary || is_large {
        None
    } else {
        Some(file.content.chars().take(256 * 1024).collect())
    };

    Ok(RepositoryBlobView {
        viewer_permission,
        ref_name: ref_name.clone(),
        resolved_ref,
        default_branch_href: repository_default_branch_href(&repository),
        recovery_href: repository_tree_href(&repository, &ref_name, parent_path(&path)),
        path_name: path
            .rsplit('/')
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or(&path)
            .to_owned(),
        breadcrumbs: repository_breadcrumbs(&repository, &ref_name, &path),
        parent_href: repository_parent_tree_href(&repository, &ref_name, &path),
        language: language_for_path(&path),
        is_binary,
        is_large,
        line_count: line_count(&file.content),
        loc_count: loc_count(&file.content),
        size_label: format_byte_size(file.byte_size),
        mime_type: mime_type_for_path(&path, is_binary),
        render_mode: render_mode(is_binary, is_large).to_owned(),
        display_content,
        history_href: repository_history_href(&repository, &ref_name, &path),
        raw_href: format!("{base}?raw=1"),
        download_href: format!("{base}?download=1"),
        raw_api_href: format!("{api_base}&raw=1"),
        download_api_href: format!("{api_base}&download=1"),
        permalink_href: latest_path_commit
            .as_ref()
            .map(|commit| {
                format!(
                    "/{}/{}/blob/{}/{}",
                    repository.owner_login,
                    repository.name,
                    percent_encode_segment(&commit.oid),
                    encoded_path
                )
            })
            .unwrap_or_else(|| repository_blob_href(&repository, &ref_name, &path)),
        symbols: if is_binary || is_large {
            Vec::new()
        } else {
            symbols_for_file(&path, &file.content)
        },
        path,
        file,
        latest_commit,
        latest_path_commit,
        repository,
    })
}

async fn repository_blame_for_actor(
    pool: &PgPool,
    repository: Repository,
    actor_user_id: Uuid,
    ref_name: Option<&str>,
    path: &str,
) -> Result<RepositoryBlameView, RepositoryError> {
    let blob =
        repository_blob_for_actor(pool, repository.clone(), actor_user_id, ref_name, path).await?;
    if blob.is_binary || blob.is_large {
        return Err(repository_path_not_found_error(&repository, &blob.path));
    }
    let attribution = blame_commit_for_file(pool, &repository, &blob.file).await?;
    let commit = attribution.or_else(|| {
        blob.latest_path_commit
            .as_ref()
            .map(|latest| RepositoryBlameCommit {
                oid: latest.oid.clone(),
                short_oid: latest.short_oid.clone(),
                message: latest.message.clone(),
                href: latest.href.clone(),
                committed_at: latest.committed_at,
                author_login: None,
            })
    });
    let commit = commit.ok_or(RepositoryError::NotFound)?;
    let content = blob
        .display_content
        .as_deref()
        .unwrap_or(blob.file.content.as_str());
    let lines = blame_lines(content)
        .into_iter()
        .enumerate()
        .map(|(index, content)| RepositoryBlameLine {
            line_number: (index + 1) as i64,
            content,
            commit: commit.clone(),
        })
        .collect();

    Ok(RepositoryBlameView { blob, lines })
}

pub async fn repository_overview(
    pool: &PgPool,
    repository: Repository,
) -> Result<RepositoryOverview, RepositoryError> {
    repository_overview_for_actor(pool, repository.clone(), repository.created_by_user_id).await
}

pub async fn list_repositories_for_user(
    pool: &PgPool,
    user_id: Uuid,
    page: i64,
    page_size: i64,
) -> Result<ListEnvelope<Repository>, RepositoryError> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(DISTINCT repositories.id)
        FROM repositories
        JOIN repository_permissions
          ON repository_permissions.repository_id = repositories.id
        WHERE repository_permissions.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT DISTINCT
            repositories.id,
            repositories.owner_user_id,
            repositories.owner_organization_id,
            COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug) AS owner_login,
            repositories.name,
            repositories.description,
            repositories.visibility,
            repositories.default_branch,
            repositories.is_archived,
            repositories.created_by_user_id,
            repositories.created_at,
            repositories.updated_at
        FROM repositories
        JOIN repository_permissions
          ON repository_permissions.repository_id = repositories.id
        LEFT JOIN users owner_user
          ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations
          ON organizations.id = repositories.owner_organization_id
        WHERE repository_permissions.user_id = $1
        ORDER BY repositories.updated_at DESC, repositories.name ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(repository_from_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListEnvelope {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn get_repository_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<Repository>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };

    if can_read_repository(pool, &repository, actor_user_id).await? {
        Ok(Some(repository))
    } else {
        Err(RepositoryError::PermissionDenied)
    }
}

pub async fn repository_settings_for_admin_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositorySettingsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_admin_repository(pool, &repository, actor_user_id).await? {
        return Err(RepositoryError::PermissionDenied);
    }
    repository_settings_view(pool, &repository, "admin")
        .await
        .map(Some)
}

pub async fn update_repository_settings_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    input: UpdateRepositorySettings,
) -> Result<Option<RepositorySettingsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    let permission = repository_permission_for_user(pool, repository.id, actor_user_id).await?;
    let role = permission
        .map(|permission| permission.role)
        .unwrap_or_else(|| {
            if repository.owner_user_id == Some(actor_user_id) {
                RepositoryRole::Owner
            } else {
                RepositoryRole::Read
            }
        });
    if !role.can_admin() {
        return Err(RepositoryError::PermissionDenied);
    }

    let current = repository_settings_view(pool, &repository, role.as_str()).await?;
    let next_name = match input.name {
        Some(value) => {
            let normalized = value.trim().to_owned();
            validate_repository_name(&normalized).map_err(RepositoryError::InvalidName)?;
            normalized
        }
        None => repository.name.clone(),
    };
    let next_description = match input.description {
        Some(value) => normalize_repository_description(Some(value))?,
        None => repository.description.clone(),
    };
    let next_visibility = input
        .visibility
        .unwrap_or_else(|| repository.visibility.clone());
    let next_default_branch = match input.default_branch {
        Some(value) => {
            let normalized = value.trim().to_owned();
            if normalized.is_empty() || normalized.len() > 255 {
                return Err(RepositoryError::InvalidName("default branch".to_owned()));
            }
            normalized
        }
        None => repository.default_branch.clone(),
    };
    let next_template = input.is_template.unwrap_or(current.is_template);
    let next_allow_forking = input.allow_forking.unwrap_or(current.allow_forking);
    let next_signoff = input
        .web_commit_signoff_required
        .unwrap_or(current.web_commit_signoff_required);
    let next_features = input.features.unwrap_or(current.features.clone());
    let next_merge = input.merge_methods.unwrap_or(current.merge_methods.clone());
    if !next_merge.merge_commit && !next_merge.squash && !next_merge.rebase {
        return Err(RepositoryError::NoMergeMethodEnabled);
    }

    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE repositories
        SET name = $2,
            description = $3,
            visibility = $4,
            default_branch = $5,
            is_template = $6,
            has_issues = $7,
            has_projects = $8,
            has_wiki = $9,
            allow_forking = $10,
            web_commit_signoff_required = $11
        WHERE id = $1
        "#,
    )
    .bind(repository.id)
    .bind(&next_name)
    .bind(&next_description)
    .bind(next_visibility.as_str())
    .bind(&next_default_branch)
    .bind(next_template)
    .bind(next_features.issues)
    .bind(next_features.projects)
    .bind(next_features.wiki)
    .bind(next_allow_forking)
    .bind(next_signoff)
    .execute(&mut *tx)
    .await?;

    let default_method = if next_merge.squash {
        "squash"
    } else if next_merge.merge_commit {
        "merge_commit"
    } else {
        "rebase"
    };
    sqlx::query(
        r#"
        INSERT INTO repository_merge_settings (
            repository_id, allow_squash, allow_merge_commit, allow_rebase, default_method
        )
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (repository_id) DO UPDATE
        SET allow_squash = EXCLUDED.allow_squash,
            allow_merge_commit = EXCLUDED.allow_merge_commit,
            allow_rebase = EXCLUDED.allow_rebase,
            default_method = CASE
                WHEN repository_merge_settings.default_method = 'squash' AND EXCLUDED.allow_squash THEN 'squash'
                WHEN repository_merge_settings.default_method = 'merge_commit' AND EXCLUDED.allow_merge_commit THEN 'merge_commit'
                WHEN repository_merge_settings.default_method = 'rebase' AND EXCLUDED.allow_rebase THEN 'rebase'
                ELSE EXCLUDED.default_method
            END
        "#,
    )
    .bind(repository.id)
    .bind(next_merge.squash)
    .bind(next_merge.merge_commit)
    .bind(next_merge.rebase)
    .bind(default_method)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO repository_settings_audit_events (
            repository_id, actor_user_id, event_type, before_state, after_state
        )
        VALUES ($1, $2, 'repository_settings.updated', $3, $4)
        "#,
    )
    .bind(repository.id)
    .bind(actor_user_id)
    .bind(json!(current))
    .bind(json!({
        "name": next_name,
        "description": next_description,
        "visibility": next_visibility,
        "defaultBranch": next_default_branch,
        "isTemplate": next_template,
        "allowForking": next_allow_forking,
        "webCommitSignoffRequired": next_signoff,
        "features": next_features,
        "mergeMethods": next_merge,
    }))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    let updated = get_repository(pool, repository.id)
        .await?
        .ok_or(RepositoryError::NotFound)?;
    repository_settings_view(pool, &updated, role.as_str())
        .await
        .map(Some)
}

pub async fn can_admin_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<bool, RepositoryError> {
    if repository.owner_user_id == Some(actor_user_id) {
        return Ok(true);
    }
    Ok(
        repository_permission_for_user(pool, repository.id, actor_user_id)
            .await?
            .is_some_and(|permission| permission.role.can_admin()),
    )
}

async fn repository_settings_view(
    pool: &PgPool,
    repository: &Repository,
    viewer_permission: &str,
) -> Result<RepositorySettingsView, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT repositories.is_template,
               repositories.has_issues,
               repositories.has_projects,
               repositories.has_wiki,
               repositories.allow_forking,
               repositories.web_commit_signoff_required,
               COALESCE(repository_merge_settings.allow_merge_commit, true) AS allow_merge_commit,
               COALESCE(repository_merge_settings.allow_squash, true) AS allow_squash,
               COALESCE(repository_merge_settings.allow_rebase, true) AS allow_rebase,
               COALESCE(repository_merge_settings.allow_auto_merge, false) AS allow_auto_merge,
               (SELECT count(*) FROM repository_settings_audit_events WHERE repository_id = repositories.id) AS audit_event_count
        FROM repositories
        LEFT JOIN repository_merge_settings
          ON repository_merge_settings.repository_id = repositories.id
        WHERE repositories.id = $1
        "#,
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;

    Ok(RepositorySettingsView {
        id: repository.id,
        owner_login: repository.owner_login.clone(),
        name: repository.name.clone(),
        description: repository.description.clone(),
        visibility: repository.visibility.clone(),
        default_branch: repository.default_branch.clone(),
        is_archived: repository.is_archived,
        is_template: row.get("is_template"),
        allow_forking: row.get("allow_forking"),
        web_commit_signoff_required: row.get("web_commit_signoff_required"),
        features: RepositorySettingsFeatureFlags {
            issues: row.get("has_issues"),
            projects: row.get("has_projects"),
            wiki: row.get("has_wiki"),
        },
        merge_methods: RepositorySettingsMergeMethods {
            merge_commit: row.get("allow_merge_commit"),
            squash: row.get("allow_squash"),
            rebase: row.get("allow_rebase"),
            auto_merge: row.get("allow_auto_merge"),
        },
        capabilities: RepositorySettingsCapabilities {
            rename: true,
            archive: false,
            transfer: false,
            change_visibility: true,
            delete: false,
        },
        viewer_permission: viewer_permission.to_owned(),
        audit_event_count: row.get::<i64, _>("audit_event_count"),
        updated_at: repository.updated_at,
    })
}

pub async fn get_repository_by_owner_name(
    pool: &PgPool,
    owner_login: &str,
    name: &str,
) -> Result<Option<Repository>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT
            repositories.id,
            repositories.owner_user_id,
            repositories.owner_organization_id,
            COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug) AS owner_login,
            repositories.name,
            repositories.description,
            repositories.visibility,
            repositories.default_branch,
            repositories.is_archived,
            repositories.created_by_user_id,
            repositories.created_at,
            repositories.updated_at
        FROM repositories
        LEFT JOIN users owner_user
          ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations
          ON organizations.id = repositories.owner_organization_id
        WHERE (
            lower(COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug)) = lower($1)
            OR lower(owner_user.email) = lower($1)
        )
          AND lower(repositories.name) = lower($2)
        "#,
    )
    .bind(owner_login)
    .bind(name)
    .fetch_optional(pool)
    .await?;

    row.map(repository_from_row).transpose()
}

pub async fn get_repository(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Option<Repository>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT
            repositories.id,
            repositories.owner_user_id,
            repositories.owner_organization_id,
            COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug) AS owner_login,
            repositories.name,
            repositories.description,
            repositories.visibility,
            repositories.default_branch,
            repositories.is_archived,
            repositories.created_by_user_id,
            repositories.created_at,
            repositories.updated_at
        FROM repositories
        LEFT JOIN users owner_user
          ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations
          ON organizations.id = repositories.owner_organization_id
        WHERE repositories.id = $1
        "#,
    )
    .bind(repository_id)
    .fetch_optional(pool)
    .await?;

    row.map(repository_from_row).transpose()
}

pub async fn grant_repository_permission(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
    role: RepositoryRole,
    source: &str,
) -> Result<RepositoryPermission, RepositoryError> {
    let row = sqlx::query(
        r#"
        INSERT INTO repository_permissions (repository_id, user_id, role, source)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (repository_id, user_id)
        DO UPDATE SET role = EXCLUDED.role, source = EXCLUDED.source
        RETURNING repository_id, user_id, role, source
        "#,
    )
    .bind(repository_id)
    .bind(user_id)
    .bind(role.as_str())
    .bind(source)
    .fetch_one(pool)
    .await?;

    repository_permission_from_row(row)
}

pub async fn repository_permission_for_user(
    pool: &PgPool,
    repository_id: Uuid,
    user_id: Uuid,
) -> Result<Option<RepositoryPermission>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT repository_id, user_id, role, source
        FROM repository_permissions
        WHERE repository_id = $1 AND user_id = $2
        "#,
    )
    .bind(repository_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    row.map(repository_permission_from_row).transpose()
}

pub async fn can_read_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<bool, RepositoryError> {
    if repository.visibility == RepositoryVisibility::Public {
        return Ok(true);
    }

    if repository.owner_user_id == Some(actor_user_id) {
        return Ok(true);
    }

    if let Some(organization_id) = repository.owner_organization_id {
        let is_org_member = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM organization_memberships
                WHERE organization_id = $1 AND user_id = $2
            )
            "#,
        )
        .bind(organization_id)
        .bind(actor_user_id)
        .fetch_one(pool)
        .await?;

        if is_org_member && repository.visibility == RepositoryVisibility::Internal {
            return Ok(true);
        }
    }

    Ok(
        repository_permission_for_user(pool, repository.id, actor_user_id)
            .await?
            .is_some_and(|permission| permission.role.can_read()),
    )
}

pub async fn can_write_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<bool, RepositoryError> {
    if repository.owner_user_id == Some(actor_user_id) {
        return Ok(true);
    }

    Ok(
        repository_permission_for_user(pool, repository.id, actor_user_id)
            .await?
            .is_some_and(|permission| permission.role.can_write()),
    )
}

pub async fn insert_commit(
    pool: &PgPool,
    repository_id: Uuid,
    input: CreateCommit,
) -> Result<Commit, RepositoryError> {
    let row = sqlx::query(
        r#"
        INSERT INTO commits (
            repository_id,
            oid,
            author_user_id,
            committer_user_id,
            message,
            tree_oid,
            parent_oids,
            committed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
            id,
            repository_id,
            oid,
            author_user_id,
            committer_user_id,
            message,
            tree_oid,
            parent_oids,
            committed_at,
            created_at
        "#,
    )
    .bind(repository_id)
    .bind(&input.oid)
    .bind(input.author_user_id)
    .bind(input.committer_user_id)
    .bind(&input.message)
    .bind(&input.tree_oid)
    .bind(&input.parent_oids)
    .bind(input.committed_at)
    .fetch_one(pool)
    .await?;

    Ok(commit_from_row(row))
}

pub async fn upsert_git_ref(
    pool: &PgPool,
    repository_id: Uuid,
    name: &str,
    kind: &str,
    target_commit_id: Option<Uuid>,
) -> Result<GitRef, RepositoryError> {
    let row = sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (repository_id, name)
        DO UPDATE SET kind = EXCLUDED.kind, target_commit_id = EXCLUDED.target_commit_id
        RETURNING id, repository_id, name, kind, target_commit_id, created_at, updated_at
        "#,
    )
    .bind(repository_id)
    .bind(name)
    .bind(kind)
    .bind(target_commit_id)
    .fetch_one(pool)
    .await?;

    Ok(git_ref_from_row(row))
}

pub async fn replace_repository_snapshot(
    pool: &PgPool,
    repository_id: Uuid,
    snapshot: RepositorySnapshot,
) -> Result<Commit, RepositoryError> {
    let mut transaction = pool.begin().await?;
    let existing_commit_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM commits WHERE repository_id = $1 AND oid = $2",
    )
    .bind(repository_id)
    .bind(&snapshot.commit.oid)
    .fetch_optional(&mut *transaction)
    .await?;

    let commit = if let Some(commit_id) = existing_commit_id {
        sqlx::query(
            r#"
            SELECT id, repository_id, oid, author_user_id, committer_user_id, message,
                   tree_oid, parent_oids, committed_at, created_at
            FROM commits
            WHERE id = $1
            "#,
        )
        .bind(commit_id)
        .fetch_one(&mut *transaction)
        .await
        .map(commit_from_row)?
    } else {
        let row = sqlx::query(
            r#"
            INSERT INTO commits (
                repository_id,
                oid,
                author_user_id,
                committer_user_id,
                message,
                tree_oid,
                parent_oids,
                committed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id,
                repository_id,
                oid,
                author_user_id,
                committer_user_id,
                message,
                tree_oid,
                parent_oids,
                committed_at,
                created_at
            "#,
        )
        .bind(repository_id)
        .bind(&snapshot.commit.oid)
        .bind(snapshot.commit.author_user_id)
        .bind(snapshot.commit.committer_user_id)
        .bind(&snapshot.commit.message)
        .bind(&snapshot.commit.tree_oid)
        .bind(&snapshot.commit.parent_oids)
        .bind(snapshot.commit.committed_at)
        .fetch_one(&mut *transaction)
        .await?;
        commit_from_row(row)
    };

    if let Some(tree_oid) = snapshot.commit.tree_oid.as_deref() {
        sqlx::query(
            r#"
            INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
            VALUES ($1, $2, 'tree', $3)
            ON CONFLICT (repository_id, oid) DO NOTHING
            "#,
        )
        .bind(repository_id)
        .bind(tree_oid)
        .bind(snapshot.files.len() as i64)
        .execute(&mut *transaction)
        .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
        VALUES ($1, $2, 'commit', 0)
        ON CONFLICT (repository_id, oid) DO NOTHING
        "#,
    )
    .bind(repository_id)
    .bind(&commit.oid)
    .execute(&mut *transaction)
    .await?;

    sqlx::query("DELETE FROM repository_files WHERE repository_id = $1 AND commit_id = $2")
        .bind(repository_id)
        .bind(commit.id)
        .execute(&mut *transaction)
        .await?;

    for file in &snapshot.files {
        sqlx::query(
            r#"
            INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
            VALUES ($1, $2, 'blob', $3)
            ON CONFLICT (repository_id, oid) DO NOTHING
            "#,
        )
        .bind(repository_id)
        .bind(&file.oid)
        .bind(file.byte_size)
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(repository_id)
        .bind(commit.id)
        .bind(&file.path)
        .bind(&file.content)
        .bind(&file.oid)
        .bind(file.byte_size)
        .execute(&mut *transaction)
        .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, $2, 'branch', $3)
        ON CONFLICT (repository_id, name)
        DO UPDATE SET kind = EXCLUDED.kind, target_commit_id = EXCLUDED.target_commit_id
        "#,
    )
    .bind(repository_id)
    .bind(format!("refs/heads/{}", snapshot.branch_name))
    .bind(commit.id)
    .execute(&mut *transaction)
    .await?;

    let indexed_files = snapshot
        .files
        .iter()
        .map(|file| IndexedSearchFile {
            path: file.path.clone(),
            content: file.content.clone(),
        })
        .collect::<Vec<_>>();
    transaction.commit().await?;
    if let Some(repository) = get_repository(pool, repository_id).await? {
        upsert_repository_search_index(pool, &repository, &commit, &indexed_files).await?;
    }
    super::git_transport::materialize_bare_repository_by_id(pool, repository_id)
        .await
        .map_err(|_| RepositoryError::GitStorageFailed)?;
    Ok(commit)
}

async fn bootstrap_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    request: &RepositoryBootstrapRequest,
) -> Result<(), RepositoryError> {
    let files = bootstrap_files(pool, repository, request).await?;
    if files.is_empty() {
        return Ok(());
    }

    let tree_oid = deterministic_oid(
        "tree",
        &files
            .iter()
            .map(|file| format!("{}:{}", file.path, file.oid))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    let commit_oid = deterministic_oid(
        "commit",
        &format!(
            "{}:{}:{}",
            repository.id, repository.default_branch, tree_oid
        ),
    );
    let commit = insert_commit(
        pool,
        repository.id,
        CreateCommit {
            oid: commit_oid.clone(),
            author_user_id: Some(actor_user_id),
            committer_user_id: Some(actor_user_id),
            message: "Initial commit".to_owned(),
            tree_oid: Some(tree_oid.clone()),
            parent_oids: Vec::new(),
            committed_at: Utc::now(),
        },
    )
    .await?;

    sqlx::query(
        r#"
        INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
        VALUES ($1, $2, 'tree', $3)
        ON CONFLICT (repository_id, oid) DO NOTHING
        "#,
    )
    .bind(repository.id)
    .bind(&tree_oid)
    .bind(files.len() as i64)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
        VALUES ($1, $2, 'commit', 0)
        ON CONFLICT (repository_id, oid) DO NOTHING
        "#,
    )
    .bind(repository.id)
    .bind(&commit_oid)
    .execute(pool)
    .await?;

    for file in &files {
        sqlx::query(
            r#"
            INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
            VALUES ($1, $2, 'blob', $3)
            ON CONFLICT (repository_id, oid) DO NOTHING
            "#,
        )
        .bind(repository.id)
        .bind(&file.oid)
        .bind(file.content.len() as i64)
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (repository_id, commit_id, lower(path))
            DO UPDATE SET commit_id = EXCLUDED.commit_id,
                          content = EXCLUDED.content,
                          oid = EXCLUDED.oid,
                          byte_size = EXCLUDED.byte_size
            "#,
        )
        .bind(repository.id)
        .bind(commit.id)
        .bind(&file.path)
        .bind(&file.content)
        .bind(&file.oid)
        .bind(file.content.len() as i64)
        .execute(pool)
        .await?;
    }

    upsert_git_ref(
        pool,
        repository.id,
        &format!("refs/heads/{}", repository.default_branch),
        "branch",
        Some(commit.id),
    )
    .await?;

    super::git_transport::materialize_bare_repository(pool, repository)
        .await
        .map_err(|_| RepositoryError::GitStorageFailed)?;

    let indexed_files = files
        .iter()
        .map(|file| IndexedSearchFile {
            path: file.path.clone(),
            content: file.content.clone(),
        })
        .collect::<Vec<_>>();
    upsert_repository_search_index(pool, repository, &commit, &indexed_files).await?;

    Ok(())
}

async fn bootstrap_files(
    pool: &PgPool,
    repository: &Repository,
    request: &RepositoryBootstrapRequest,
) -> Result<Vec<BootstrapFile>, RepositoryError> {
    let mut files = Vec::new();

    let template_slug = request.template_slug.as_deref().unwrap_or("blank").trim();
    if !template_slug.is_empty() && template_slug != "blank" {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM repository_creation_templates WHERE slug = $1)",
        )
        .bind(template_slug)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(RepositoryError::UnknownTemplate(template_slug.to_owned()));
        }
        files.extend(template_files(template_slug, repository));
    }

    if request.initialize_readme {
        files.push(make_bootstrap_file(
            "README.md",
            &format!(
                "# {}\n\n{}{}\n",
                repository.name,
                repository
                    .description
                    .as_deref()
                    .unwrap_or("A new opengithub repository."),
                if template_slug == "blank" {
                    ""
                } else {
                    "\n\nGenerated from a repository template."
                }
            ),
        ));
    }

    if let Some(slug) = request
        .gitignore_template_slug
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let content = sqlx::query_scalar::<_, String>(
            "SELECT content FROM gitignore_templates WHERE slug = $1",
        )
        .bind(slug)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RepositoryError::UnknownGitignoreTemplate(slug.to_owned()))?;
        files.push(make_bootstrap_file(".gitignore", &content));
    }

    if let Some(slug) = request
        .license_template_slug
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let content = sqlx::query_scalar::<_, String>(
            "SELECT content FROM license_templates WHERE slug = $1",
        )
        .bind(slug)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RepositoryError::UnknownLicenseTemplate(slug.to_owned()))?;
        let owner = repository.owner_login.clone();
        files.push(make_bootstrap_file(
            "LICENSE",
            &content
                .replace("{{year}}", &Utc::now().format("%Y").to_string())
                .replace("{{owner}}", &owner),
        ));
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    files.dedup_by(|left, right| left.path.eq_ignore_ascii_case(&right.path));
    Ok(files)
}

fn template_files(slug: &str, repository: &Repository) -> Vec<BootstrapFile> {
    match slug {
        "node-typescript" => vec![
            make_bootstrap_file("package.json", &format!("{{\n  \"name\": \"{}\",\n  \"version\": \"0.1.0\",\n  \"type\": \"module\"\n}}\n", repository.name)),
            make_bootstrap_file("src/index.ts", "export function main() {\n  return \"hello from opengithub\";\n}\n"),
        ],
        "rust-axum" => vec![
            make_bootstrap_file("Cargo.toml", &format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\naxum = \"0.7\"\ntokio = {{ version = \"1\", features = [\"full\"] }}\n", repository.name.replace('-', "_"))),
            make_bootstrap_file("src/main.rs", "use axum::{routing::get, Router};\n\n#[tokio::main]\nasync fn main() {\n    let app = Router::new().route(\"/\", get(|| async { \"ok\" }));\n    let listener = tokio::net::TcpListener::bind(\"0.0.0.0:3000\").await.unwrap();\n    axum::serve(listener, app).await.unwrap();\n}\n"),
        ],
        _ => Vec::new(),
    }
}

fn make_bootstrap_file(path: &str, content: &str) -> BootstrapFile {
    BootstrapFile {
        path: path.to_owned(),
        content: content.to_owned(),
        oid: deterministic_oid("blob", &format!("{path}\0{content}")),
    }
}

fn deterministic_oid(kind: &str, content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update(b"\0");
    hasher.update(content.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

async fn list_repository_files(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<RepositoryFile>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT id, repository_id, commit_id, path, content, oid, byte_size, created_at
        FROM repository_files
        WHERE repository_id = $1
        ORDER BY lower(path) ASC
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(repository_file_from_row).collect())
}

async fn list_repository_files_for_resolved_ref(
    pool: &PgPool,
    repository_id: Uuid,
    resolved_ref: &RepositoryResolvedRef,
) -> Result<Vec<RepositoryFile>, RepositoryError> {
    let Some(target_oid) = resolved_ref.target_oid.as_deref() else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query(
        r#"
        SELECT repository_files.id,
               repository_files.repository_id,
               repository_files.commit_id,
               repository_files.path,
               repository_files.content,
               repository_files.oid,
               repository_files.byte_size,
               repository_files.created_at
        FROM repository_files
        JOIN commits ON commits.id = repository_files.commit_id
        WHERE repository_files.repository_id = $1
          AND commits.oid = $2
        ORDER BY lower(repository_files.path) ASC
        "#,
    )
    .bind(repository_id)
    .bind(target_oid)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(repository_file_from_row).collect())
}

async fn repository_path_exists_for_commit(
    pool: &PgPool,
    repository_id: Uuid,
    commit_id: Option<Uuid>,
    path: &str,
) -> Result<bool, RepositoryError> {
    let Some(commit_id) = commit_id else {
        return Ok(false);
    };
    if path.is_empty() {
        return Ok(true);
    }
    let path_prefix = format!("{path}/%");
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM repository_files
            WHERE repository_id = $1
              AND commit_id = $2
              AND (path = $3 OR path LIKE $4)
        )
        "#,
    )
    .bind(repository_id)
    .bind(commit_id)
    .bind(path)
    .bind(path_prefix)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

async fn count_repository_refs(
    pool: &PgPool,
    repository_id: Uuid,
    kind: &str,
) -> Result<i64, RepositoryError> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_git_refs WHERE repository_id = $1 AND kind = $2",
    )
    .bind(repository_id)
    .bind(kind)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

async fn get_repository_ref(
    pool: &PgPool,
    repository_id: Uuid,
    name: &str,
) -> Result<Option<GitRef>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT id, repository_id, name, kind, target_commit_id, created_at, updated_at
        FROM repository_git_refs
        WHERE repository_id = $1 AND name = $2
        "#,
    )
    .bind(repository_id)
    .bind(name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(git_ref_from_row))
}

async fn latest_commit_for_repository(
    pool: &PgPool,
    repository: &Repository,
) -> Result<Option<RepositoryLatestCommit>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT oid, message, committed_at
        FROM commits
        WHERE repository_id = $1
        ORDER BY committed_at DESC, created_at DESC
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| {
        let oid: String = row.get("oid");
        RepositoryLatestCommit {
            short_oid: oid.chars().take(7).collect(),
            href: format!(
                "/{}/{}/commit/{}",
                repository.owner_login, repository.name, oid
            ),
            oid,
            message: row.get("message"),
            committed_at: row.get("committed_at"),
        }
    }))
}

async fn latest_commit_for_file(
    pool: &PgPool,
    repository: &Repository,
    file: &RepositoryFile,
) -> Result<Option<RepositoryLatestCommit>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT oid, message, committed_at
        FROM commits
        WHERE id = $1
          AND repository_id = $2
        LIMIT 1
        "#,
    )
    .bind(file.commit_id)
    .bind(repository.id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| {
        let oid: String = row.get("oid");
        RepositoryLatestCommit {
            short_oid: oid.chars().take(7).collect(),
            href: format!(
                "/{}/{}/commit/{}",
                repository.owner_login, repository.name, oid
            ),
            oid,
            message: row.get("message"),
            committed_at: row.get("committed_at"),
        }
    }))
}

async fn blame_commit_for_file(
    pool: &PgPool,
    repository: &Repository,
    file: &RepositoryFile,
) -> Result<Option<RepositoryBlameCommit>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT commits.oid, commits.message, commits.committed_at,
               COALESCE(NULLIF(users.username, ''), users.email) AS author_login
        FROM commits
        LEFT JOIN users ON users.id = commits.author_user_id
        WHERE commits.id = $1
          AND commits.repository_id = $2
        LIMIT 1
        "#,
    )
    .bind(file.commit_id)
    .bind(repository.id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| {
        let oid: String = row.get("oid");
        RepositoryBlameCommit {
            short_oid: oid.chars().take(7).collect(),
            href: format!(
                "/{}/{}/commit/{}",
                repository.owner_login, repository.name, oid
            ),
            oid,
            message: row.get("message"),
            committed_at: row.get("committed_at"),
            author_login: row.get("author_login"),
        }
    }))
}

async fn record_recent_repository_visit(
    pool: &PgPool,
    user_id: Uuid,
    repository_id: Uuid,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO recent_repository_visits (user_id, repository_id, visited_at)
        VALUES ($1, $2, now())
        ON CONFLICT (user_id, repository_id)
        DO UPDATE SET visited_at = EXCLUDED.visited_at
        "#,
    )
    .bind(user_id)
    .bind(repository_id)
    .execute(pool)
    .await?;

    Ok(())
}

fn repository_root_entries(
    repository: &Repository,
    files: &[RepositoryFile],
    latest_commit: Option<&RepositoryLatestCommit>,
) -> Vec<RepositoryTreeEntry> {
    let mut folders: BTreeMap<String, DateTime<Utc>> = BTreeMap::new();
    let mut entries = Vec::new();

    for file in files {
        if let Some((folder, _)) = file.path.split_once('/') {
            folders
                .entry(folder.to_owned())
                .and_modify(|updated_at| {
                    if file.created_at > *updated_at {
                        *updated_at = file.created_at;
                    }
                })
                .or_insert(file.created_at);
        } else {
            entries.push(RepositoryTreeEntry {
                kind: "file".to_owned(),
                name: file.path.clone(),
                path: file.path.clone(),
                href: format!(
                    "/{}/{}/blob/{}/{}",
                    repository.owner_login, repository.name, repository.default_branch, file.path
                ),
                byte_size: Some(file.byte_size),
                latest_commit_message: latest_commit.map(|commit| commit.message.clone()),
                latest_commit_href: latest_commit.map(|commit| commit.href.clone()),
                updated_at: file.created_at,
            });
        }
    }

    for (folder, updated_at) in folders {
        entries.push(RepositoryTreeEntry {
            kind: "folder".to_owned(),
            name: folder.clone(),
            path: folder.clone(),
            href: format!(
                "/{}/{}/tree/{}/{}",
                repository.owner_login, repository.name, repository.default_branch, folder
            ),
            byte_size: None,
            latest_commit_message: latest_commit.map(|commit| commit.message.clone()),
            latest_commit_href: latest_commit.map(|commit| commit.href.clone()),
            updated_at,
        });
    }

    entries.sort_by(
        |left, right| match (left.kind.as_str(), right.kind.as_str()) {
            ("folder", "file") => std::cmp::Ordering::Less,
            ("file", "folder") => std::cmp::Ordering::Greater,
            _ => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
        },
    );
    entries
}

fn repository_entries_for_path(
    repository: &Repository,
    ref_name: &str,
    files: &[RepositoryFile],
    path: &str,
) -> Vec<RepositoryTreeEntry> {
    let prefix = if path.is_empty() {
        String::new()
    } else {
        format!("{path}/")
    };
    let mut folders: BTreeMap<String, DateTime<Utc>> = BTreeMap::new();
    let mut entries = Vec::new();

    for file in files {
        if !file.path.starts_with(&prefix) {
            continue;
        }
        let remainder = &file.path[prefix.len()..];
        if remainder.is_empty() {
            continue;
        }
        if let Some((folder, _)) = remainder.split_once('/') {
            let child_path = if path.is_empty() {
                folder.to_owned()
            } else {
                format!("{path}/{folder}")
            };
            folders
                .entry(child_path)
                .and_modify(|updated_at| {
                    if file.created_at > *updated_at {
                        *updated_at = file.created_at;
                    }
                })
                .or_insert(file.created_at);
        } else {
            entries.push(RepositoryTreeEntry {
                kind: "file".to_owned(),
                name: remainder.to_owned(),
                path: file.path.clone(),
                href: format!(
                    "/{}/{}/blob/{}/{}",
                    repository.owner_login,
                    repository.name,
                    percent_encode_segment(ref_name),
                    percent_encode_path(&file.path)
                ),
                byte_size: Some(file.byte_size),
                latest_commit_message: None,
                latest_commit_href: None,
                updated_at: file.created_at,
            });
        }
    }

    for (folder_path, updated_at) in folders {
        entries.push(RepositoryTreeEntry {
            kind: "folder".to_owned(),
            name: folder_path
                .rsplit('/')
                .next()
                .unwrap_or(folder_path.as_str())
                .to_owned(),
            path: folder_path.clone(),
            href: format!(
                "/{}/{}/tree/{}/{}",
                repository.owner_login,
                repository.name,
                percent_encode_segment(ref_name),
                percent_encode_path(&folder_path)
            ),
            byte_size: None,
            latest_commit_message: None,
            latest_commit_href: None,
            updated_at,
        });
    }

    entries.sort_by(
        |left, right| match (left.kind.as_str(), right.kind.as_str()) {
            ("folder", "file") => std::cmp::Ordering::Less,
            ("file", "folder") => std::cmp::Ordering::Greater,
            _ => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
        },
    );
    entries
}

fn readme_for_path(files: &[RepositoryFile], path: &str) -> Option<RepositoryFile> {
    let prefix = if path.is_empty() {
        String::new()
    } else {
        format!("{path}/")
    };
    files
        .iter()
        .find(|file| {
            file.path
                .strip_prefix(&prefix)
                .is_some_and(|remainder| remainder.eq_ignore_ascii_case("README.md"))
        })
        .cloned()
}

fn repository_breadcrumbs(
    repository: &Repository,
    ref_name: &str,
    path: &str,
) -> Vec<RepositoryPathBreadcrumb> {
    let mut breadcrumbs = vec![RepositoryPathBreadcrumb {
        name: repository.name.clone(),
        path: String::new(),
        href: format!(
            "/{}/{}/tree/{}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(ref_name)
        ),
    }];
    let mut current = String::new();
    for segment in path.split('/').filter(|segment| !segment.is_empty()) {
        if current.is_empty() {
            current.push_str(segment);
        } else {
            current.push('/');
            current.push_str(segment);
        }
        breadcrumbs.push(RepositoryPathBreadcrumb {
            name: segment.to_owned(),
            path: current.clone(),
            href: format!(
                "/{}/{}/tree/{}/{}",
                repository.owner_login,
                repository.name,
                percent_encode_segment(ref_name),
                percent_encode_path(&current)
            ),
        });
    }
    breadcrumbs
}

fn repository_parent_tree_href(
    repository: &Repository,
    ref_name: &str,
    path: &str,
) -> Option<String> {
    if path.is_empty() {
        return None;
    }
    let parent = path
        .rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or("");
    Some(if parent.is_empty() {
        format!(
            "/{}/{}/tree/{}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(ref_name)
        )
    } else {
        format!(
            "/{}/{}/tree/{}/{}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(ref_name),
            percent_encode_path(parent)
        )
    })
}

fn repository_default_branch_href(repository: &Repository) -> String {
    repository_tree_href(repository, &repository.default_branch, "")
}

fn repository_path_not_found_error(repository: &Repository, path: &str) -> RepositoryError {
    RepositoryError::PathNotFoundWithRecovery {
        path: path.to_owned(),
        recovery_href: repository_default_branch_href(repository),
        default_branch_href: repository_default_branch_href(repository),
    }
}

fn repository_tree_href(repository: &Repository, ref_name: &str, path: &str) -> String {
    if path.is_empty() {
        format!(
            "/{}/{}/tree/{}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(ref_name)
        )
    } else {
        format!(
            "/{}/{}/tree/{}/{}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(ref_name),
            percent_encode_path(path)
        )
    }
}

fn repository_blob_href(repository: &Repository, ref_name: &str, path: &str) -> String {
    format!(
        "/{}/{}/blob/{}/{}",
        repository.owner_login,
        repository.name,
        percent_encode_segment(ref_name),
        percent_encode_path(path)
    )
}

fn ref_matches_active(qualified_name: &str, short_name: &str, active_ref: &str) -> bool {
    let normalized = active_ref.trim();
    qualified_name == normalized
        || short_name == normalized
        || qualified_name == format!("refs/heads/{normalized}")
        || qualified_name == format!("refs/tags/{normalized}")
}

fn parent_path(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or("")
}

fn repository_history_href(repository: &Repository, ref_name: &str, path: &str) -> String {
    if path.is_empty() {
        format!(
            "/{}/{}/commits/{}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(ref_name)
        )
    } else {
        format!(
            "/{}/{}/commits/{}/{}",
            repository.owner_login,
            repository.name,
            percent_encode_segment(ref_name),
            percent_encode_path(path)
        )
    }
}

async fn viewer_permission_for_user(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<Option<String>, RepositoryError> {
    Ok(
        repository_permission_for_user(pool, repository.id, actor_user_id)
            .await?
            .map(|permission| permission.role.as_str().to_owned())
            .or_else(|| {
                if repository.visibility == RepositoryVisibility::Public {
                    Some("read".to_owned())
                } else {
                    None
                }
            }),
    )
}

async fn resolve_repository_ref(
    pool: &PgPool,
    repository: &Repository,
    ref_name: Option<&str>,
) -> Result<RepositoryResolvedRef, RepositoryError> {
    let ref_name = ref_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&repository.default_branch);
    let normalized = normalize_repository_path(ref_name)?;
    let branch_ref = format!("refs/heads/{normalized}");
    let tag_ref = format!("refs/tags/{normalized}");
    let row = sqlx::query(
        r#"
        SELECT repository_git_refs.name,
               repository_git_refs.kind,
               commits.oid AS target_oid
        FROM repository_git_refs
        LEFT JOIN commits ON commits.id = repository_git_refs.target_commit_id
        WHERE repository_git_refs.repository_id = $1
          AND repository_git_refs.name IN ($2, $3, $4)
        ORDER BY CASE
            WHEN repository_git_refs.name = $2 THEN 0
            WHEN repository_git_refs.name = $3 THEN 1
            ELSE 2
        END
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(&branch_ref)
    .bind(&tag_ref)
    .bind(&normalized)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Err(RepositoryError::RefNotFoundWithRecovery {
            ref_name: normalized,
            recovery_href: repository_default_branch_href(repository),
            default_branch_href: repository_default_branch_href(repository),
        });
    };
    let qualified_name: String = row.get("name");
    let kind: String = row.get("kind");
    let short_name = qualified_name
        .strip_prefix("refs/heads/")
        .or_else(|| qualified_name.strip_prefix("refs/tags/"))
        .unwrap_or(&qualified_name)
        .to_owned();

    Ok(RepositoryResolvedRef {
        recovery_href: repository_tree_href(repository, &short_name, ""),
        kind,
        short_name,
        qualified_name,
        target_oid: row.get("target_oid"),
    })
}

fn normalize_repository_path(value: &str) -> Result<String, RepositoryError> {
    let trimmed = value.trim_matches('/');
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    let mut segments = Vec::new();
    for segment in trimmed.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") || segment.contains('\\') {
            return Err(RepositoryError::PathNotFound);
        }
        segments.push(segment);
    }
    Ok(segments.join("/"))
}

fn blame_lines(content: &str) -> Vec<String> {
    let mut lines = content
        .split('\n')
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

async fn repository_commit_history(
    pool: &PgPool,
    repository: &Repository,
    _ref_name: &str,
    path: Option<&str>,
    page: i64,
    page_size: i64,
) -> Result<ListEnvelope<RepositoryCommitHistoryItem>, RepositoryError> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);
    let offset = (page - 1) * page_size;
    let path = path.unwrap_or("");
    let path_prefix = if path.is_empty() {
        None
    } else {
        Some(format!("{path}/%"))
    };

    let total = if path.is_empty() {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM commits WHERE repository_id = $1")
            .bind(repository.id)
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(DISTINCT commits.id)
            FROM commits
            JOIN repository_files ON repository_files.commit_id = commits.id
            WHERE commits.repository_id = $1
              AND (repository_files.path = $2 OR repository_files.path LIKE $3)
            "#,
        )
        .bind(repository.id)
        .bind(path)
        .bind(path_prefix.as_deref().unwrap_or(""))
        .fetch_one(pool)
        .await?
    };

    let rows = if path.is_empty() {
        sqlx::query(
            r#"
            SELECT commits.oid, commits.message, commits.committed_at,
                   COALESCE(NULLIF(users.username, ''), users.email) AS author_login
            FROM commits
            LEFT JOIN users ON users.id = commits.author_user_id
            WHERE commits.repository_id = $1
            ORDER BY commits.committed_at DESC, commits.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(repository.id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT DISTINCT commits.oid, commits.message, commits.committed_at,
                   COALESCE(NULLIF(users.username, ''), users.email) AS author_login
            FROM commits
            JOIN repository_files ON repository_files.commit_id = commits.id
            LEFT JOIN users ON users.id = commits.author_user_id
            WHERE commits.repository_id = $1
              AND (repository_files.path = $2 OR repository_files.path LIKE $3)
            ORDER BY commits.committed_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(repository.id)
        .bind(path)
        .bind(path_prefix.as_deref().unwrap_or(""))
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    let items = rows
        .into_iter()
        .map(|row| {
            let oid: String = row.get("oid");
            RepositoryCommitHistoryItem {
                short_oid: oid.chars().take(7).collect(),
                href: format!(
                    "/{}/{}/commit/{}",
                    repository.owner_login, repository.name, oid
                ),
                oid,
                message: row.get("message"),
                committed_at: row.get("committed_at"),
                author_login: row.get("author_login"),
            }
        })
        .collect();

    Ok(ListEnvelope {
        items,
        total,
        page,
        page_size,
    })
}

async fn upsert_repository_search_index(
    pool: &PgPool,
    repository: &Repository,
    commit: &Commit,
    files: &[IndexedSearchFile],
) -> Result<(), RepositoryError> {
    let author_login = if let Some(author_user_id) = commit.author_user_id {
        sqlx::query_scalar::<_, String>(
            "SELECT COALESCE(NULLIF(username, ''), email) FROM users WHERE id = $1",
        )
        .bind(author_user_id)
        .fetch_optional(pool)
        .await?
    } else {
        None
    };
    let href = format!(
        "/{}/{}/commit/{}",
        percent_encode_segment(&repository.owner_login),
        percent_encode_segment(&repository.name),
        percent_encode_segment(&commit.oid)
    );

    sqlx::query(
        r#"
        INSERT INTO search_documents (
            repository_id,
            owner_user_id,
            owner_organization_id,
            kind,
            resource_id,
            title,
            body,
            visibility,
            metadata,
            indexed_at
        )
        VALUES ($1, $2, $3, 'commit', $4, $5, $6, $7, $8, now())
        ON CONFLICT (kind, resource_id) DO UPDATE SET
            repository_id = EXCLUDED.repository_id,
            owner_user_id = EXCLUDED.owner_user_id,
            owner_organization_id = EXCLUDED.owner_organization_id,
            title = EXCLUDED.title,
            body = EXCLUDED.body,
            visibility = EXCLUDED.visibility,
            metadata = EXCLUDED.metadata,
            indexed_at = now()
        "#,
    )
    .bind(repository.id)
    .bind(repository.owner_user_id)
    .bind(repository.owner_organization_id)
    .bind(&commit.oid)
    .bind(commit.message.lines().next().unwrap_or(&commit.message))
    .bind(&commit.message)
    .bind(repository.visibility.as_str())
    .bind(json!({
        "href": href,
        "ownerLogin": repository.owner_login,
        "repositoryName": repository.name,
        "authorLogin": author_login,
        "committedAt": commit.committed_at,
    }))
    .execute(pool)
    .await?;

    sqlx::query(
        "DELETE FROM search_documents WHERE repository_id = $1 AND kind = 'code' AND branch = $2",
    )
    .bind(repository.id)
    .bind(&repository.default_branch)
    .execute(pool)
    .await?;

    for file in files {
        let Some((line_number, fragment)) = first_searchable_line(&file.content) else {
            continue;
        };
        let href = format!(
            "/{}/{}/blob/{}/{}#L{}",
            percent_encode_segment(&repository.owner_login),
            percent_encode_segment(&repository.name),
            percent_encode_segment(&repository.default_branch),
            percent_encode_path(&file.path),
            line_number
        );
        sqlx::query(
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
            VALUES ($1, $2, $3, 'code', $4, $5, $6, $7, $8, $9, $10, $11, now())
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
            "#,
        )
        .bind(repository.id)
        .bind(repository.owner_user_id)
        .bind(repository.owner_organization_id)
        .bind(format!(
            "{}:{}:{}",
            repository.id, repository.default_branch, file.path
        ))
        .bind(&file.path)
        .bind(&file.content)
        .bind(&file.path)
        .bind(language_for_path(&file.path))
        .bind(&repository.default_branch)
        .bind(repository.visibility.as_str())
        .bind(json!({
            "href": href,
            "ownerLogin": repository.owner_login,
            "repositoryName": repository.name,
            "lineNumber": line_number,
            "fragment": fragment,
            "commitOid": commit.oid,
        }))
        .execute(pool)
        .await?;
    }

    Ok(())
}

fn first_searchable_line(content: &str) -> Option<(i64, String)> {
    content.lines().enumerate().find_map(|(index, line)| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some((index as i64 + 1, trimmed.chars().take(240).collect()))
        }
    })
}

fn language_for_path(path: &str) -> Option<String> {
    let extension = path.rsplit('.').next()?;
    let language = match extension.to_ascii_lowercase().as_str() {
        "md" | "markdown" => "Markdown",
        "rs" => "Rust",
        "ts" | "tsx" => "TypeScript",
        "js" | "jsx" => "JavaScript",
        "json" => "JSON",
        "toml" => "TOML",
        "yml" | "yaml" => "YAML",
        "sql" => "SQL",
        "css" => "CSS",
        "html" => "HTML",
        _ => return None,
    };
    Some(language.to_owned())
}

fn mime_type_for_path(path: &str, is_binary: bool) -> String {
    if is_binary {
        return "application/octet-stream".to_owned();
    }
    let mime_type = match path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "md" | "markdown" => "text/markdown; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "jsx" | "ts" | "tsx" => "text/plain; charset=utf-8",
        "toml" | "yml" | "yaml" | "sql" | "rs" => "text/plain; charset=utf-8",
        _ => "text/plain; charset=utf-8",
    };
    mime_type.to_owned()
}

fn is_probably_binary(content: &str) -> bool {
    let mut control_count = 0usize;
    let mut total_count = 0usize;
    for character in content.chars() {
        total_count += 1;
        if character.is_control() && !matches!(character, '\n' | '\r' | '\t') {
            control_count += 1;
        }
    }
    total_count > 0 && control_count * 3 >= total_count
}

fn render_mode(is_binary: bool, is_large: bool) -> &'static str {
    if is_binary {
        "binary"
    } else if is_large {
        "large"
    } else {
        "text"
    }
}

fn line_count(content: &str) -> i64 {
    if content.is_empty() {
        0
    } else {
        content.lines().count() as i64 + i64::from(content.ends_with('\n'))
    }
}

fn loc_count(content: &str) -> i64 {
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count() as i64
}

fn symbols_for_file(path: &str, content: &str) -> Vec<RepositoryCodeSymbol> {
    let language = language_for_path(path)
        .unwrap_or_default()
        .to_ascii_lowercase();
    content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            symbol_for_line(&language, line).map(|(kind, name)| RepositoryCodeSymbol {
                kind,
                name,
                line_number: (index + 1) as i64,
                preview: line.trim().chars().take(120).collect(),
            })
        })
        .take(50)
        .collect()
}

fn symbol_for_line(language: &str, line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    if language == "markdown" {
        return markdown_symbol(trimmed);
    }
    if language == "json" {
        return json_symbol(trimmed);
    }
    if matches!(language, "rust" | "typescript" | "javascript" | "python") {
        return function_symbol(trimmed);
    }
    None
}

fn markdown_symbol(trimmed: &str) -> Option<(String, String)> {
    let hashes = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=6).contains(&hashes) || !trimmed.chars().nth(hashes).is_some_and(char::is_whitespace) {
        return None;
    }
    let name = trimmed[hashes..].trim();
    if name.is_empty() {
        None
    } else {
        Some(("heading".to_owned(), name.chars().take(80).collect()))
    }
}

fn json_symbol(trimmed: &str) -> Option<(String, String)> {
    if !trimmed.starts_with('"') || trimmed.starts_with("\"$schema\"") {
        return None;
    }
    let end = trimmed[1..].find('"')? + 1;
    if !trimmed[end + 1..].trim_start().starts_with(':') {
        return None;
    }
    Some(("key".to_owned(), trimmed[1..end].chars().take(80).collect()))
}

fn function_symbol(trimmed: &str) -> Option<(String, String)> {
    let candidates = [
        "pub async fn ",
        "pub fn ",
        "async fn ",
        "fn ",
        "export async function ",
        "export function ",
        "function ",
        "export const ",
        "const ",
        "def ",
        "async def ",
    ];
    let candidate = candidates
        .iter()
        .find_map(|prefix| trimmed.strip_prefix(prefix).map(|rest| (*prefix, rest)))?;
    let mut name = candidate.1.trim_start();
    if candidate.0.ends_with("const ") {
        name = name.split('=').next()?.trim();
    } else {
        name = name
            .split(|character: char| character == '(' || character.is_whitespace())
            .next()?;
    }
    if name.is_empty() {
        None
    } else {
        Some(("function".to_owned(), name.chars().take(80).collect()))
    }
}

fn format_byte_size(byte_size: i64) -> String {
    if byte_size < 1024 {
        return format!("{byte_size} bytes");
    }
    let kib = byte_size as f64 / 1024.0;
    if kib < 1024.0 {
        return format!("{kib:.1} KB");
    }
    let mib = kib / 1024.0;
    format!("{mib:.1} MB")
}

fn percent_encode_path(path: &str) -> String {
    path.split('/')
        .map(percent_encode_segment)
        .collect::<Vec<_>>()
        .join("/")
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

async fn repository_sidebar_metadata(
    pool: &PgPool,
    repository: &Repository,
) -> Result<RepositorySidebarMetadata, RepositoryError> {
    let stars_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_stars WHERE repository_id = $1",
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let watchers_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_watches WHERE repository_id = $1",
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let forks_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_forks WHERE source_repository_id = $1",
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let releases_count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM releases WHERE repository_id = $1")
            .bind(repository.id)
            .fetch_one(pool)
            .await?;
    let contributors_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(DISTINCT contributor_id)
        FROM (
            SELECT author_user_id AS contributor_id FROM commits WHERE repository_id = $1
            UNION
            SELECT committer_user_id AS contributor_id FROM commits WHERE repository_id = $1
        ) contributors
        WHERE contributor_id IS NOT NULL
        "#,
    )
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let language_rows = sqlx::query(
        r#"
        SELECT language, color, byte_count
        FROM repository_languages
        WHERE repository_id = $1
        ORDER BY byte_count DESC, language ASC
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;
    let total_language_bytes = language_rows
        .iter()
        .map(|row| row.get::<i64, _>("byte_count"))
        .sum::<i64>()
        .max(1);
    let languages = language_rows
        .into_iter()
        .map(|row| {
            let byte_count = row.get::<i64, _>("byte_count");
            RepositoryLanguageSummary {
                language: row.get("language"),
                color: row.get("color"),
                byte_count,
                percentage: byte_count * 100 / total_language_bytes,
            }
        })
        .collect();

    Ok(RepositorySidebarMetadata {
        about: repository.description.clone(),
        website_url: None,
        topics: Vec::new(),
        stars_count,
        watchers_count,
        forks_count,
        releases_count,
        deployments_count: 0,
        contributors_count,
        languages,
    })
}

async fn repository_viewer_state(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<RepositoryViewerState, RepositoryError> {
    let starred = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM repository_stars WHERE user_id = $1 AND repository_id = $2
        )
        "#,
    )
    .bind(actor_user_id)
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let watching = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM repository_watches WHERE user_id = $1 AND repository_id = $2
        )
        "#,
    )
    .bind(actor_user_id)
    .bind(repository.id)
    .fetch_one(pool)
    .await?;
    let forked_repository_href =
        existing_fork_href_for_user(pool, repository.id, actor_user_id).await?;

    Ok(RepositoryViewerState {
        starred,
        watching,
        forked_repository_href,
    })
}

async fn repository_social_state(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<RepositorySocialState, RepositoryError> {
    let sidebar = repository_sidebar_metadata(pool, repository).await?;
    let viewer_state = repository_viewer_state(pool, repository, actor_user_id).await?;
    Ok(RepositorySocialState {
        starred: viewer_state.starred,
        watching: viewer_state.watching,
        stars_count: sidebar.stars_count,
        watchers_count: sidebar.watchers_count,
        forks_count: sidebar.forks_count,
        forked_repository_href: viewer_state.forked_repository_href,
    })
}

async fn existing_fork_href_for_user(
    pool: &PgPool,
    source_repository_id: Uuid,
    actor_user_id: Uuid,
) -> Result<Option<String>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT COALESCE(NULLIF(owner_user.username, ''), owner_user.email, organizations.slug) AS owner_login,
               forks.name
        FROM repository_forks
        JOIN repositories forks ON forks.id = repository_forks.fork_repository_id
        LEFT JOIN users owner_user ON owner_user.id = forks.owner_user_id
        LEFT JOIN organizations ON organizations.id = forks.owner_organization_id
        WHERE repository_forks.source_repository_id = $1
          AND repository_forks.forked_by_user_id = $2
        ORDER BY repository_forks.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(source_repository_id)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| {
        format!(
            "/{}/{}",
            row.get::<String, _>("owner_login"),
            row.get::<String, _>("name")
        )
    }))
}

async fn copy_repository_snapshot(
    pool: &PgPool,
    source: &Repository,
    destination: &Repository,
    actor_user_id: Uuid,
) -> Result<(), RepositoryError> {
    let files = list_repository_files(pool, source.id).await?;
    if files.is_empty() {
        return Ok(());
    }
    let source_commit = latest_commit_for_repository(pool, source).await?;
    let source_oid = source_commit
        .as_ref()
        .map(|commit| commit.oid.as_str())
        .unwrap_or("empty");
    let files_for_hash = files
        .iter()
        .map(|file| format!("{}:{}", file.path, file.oid))
        .collect::<Vec<_>>()
        .join("\n");
    let tree_oid = deterministic_oid(
        "fork-tree",
        &format!("{}:{}:{}", destination.id, source_oid, files_for_hash),
    );
    let commit_oid = deterministic_oid(
        "fork-commit",
        &format!("{}:{}:{}", destination.id, source.id, source_oid),
    );
    let snapshot = RepositorySnapshot {
        branch_name: destination.default_branch.clone(),
        commit: CreateCommit {
            oid: commit_oid,
            author_user_id: Some(actor_user_id),
            committer_user_id: Some(actor_user_id),
            message: format!("Forked from {}/{}", source.owner_login, source.name),
            tree_oid: Some(tree_oid),
            parent_oids: source_commit
                .as_ref()
                .map(|commit| vec![commit.oid.clone()])
                .unwrap_or_default(),
            committed_at: Utc::now(),
        },
        files: files
            .into_iter()
            .map(|file| RepositorySnapshotFile {
                path: file.path,
                content: file.content,
                oid: file.oid,
                byte_size: file.byte_size,
            })
            .collect(),
    };
    replace_repository_snapshot(pool, destination.id, snapshot).await?;
    Ok(())
}

fn repository_clone_urls(repository: &Repository) -> RepositoryCloneUrls {
    let path = format!("{}/{}", repository.owner_login, repository.name);
    let (https_base, ssh_host) = clone_url_hosts();
    RepositoryCloneUrls {
        https: format!("{https_base}/{path}.git"),
        git: format!("git@{ssh_host}:{path}.git"),
        zip: format!(
            "/{path}/archive/refs/heads/{}.zip",
            repository.default_branch
        ),
    }
}

fn clone_url_hosts() -> (String, String) {
    let raw = std::env::var("API_URL")
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "https://opengithub.namuh.co".to_owned());
    let https_base = raw.trim_end_matches('/').to_owned();
    let ssh_host = url::Url::parse(&https_base)
        .ok()
        .and_then(|u| u.host_str().map(String::from))
        .unwrap_or_else(|| "opengithub.namuh.co".to_owned());
    (https_base, ssh_host)
}

async fn ensure_owner_can_create(
    pool: &PgPool,
    owner: &RepositoryOwner,
    actor_user_id: Uuid,
) -> Result<(), RepositoryError> {
    match owner {
        RepositoryOwner::User { id } => {
            if *id == actor_user_id {
                Ok(())
            } else {
                Err(RepositoryError::OwnerPermissionDenied)
            }
        }
        RepositoryOwner::Organization { id } => {
            let allowed = sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM organization_memberships
                    WHERE organization_id = $1
                      AND user_id = $2
                      AND role IN ('owner', 'admin')
                )
                "#,
            )
            .bind(id)
            .bind(actor_user_id)
            .fetch_one(pool)
            .await?;

            if allowed {
                Ok(())
            } else {
                Err(RepositoryError::OwnerPermissionDenied)
            }
        }
    }
}

async fn repository_owner_login(
    pool: &PgPool,
    owner: &RepositoryOwner,
) -> Result<(String, Uuid, String), RepositoryError> {
    match owner {
        RepositoryOwner::User { id } => {
            let login = sqlx::query_scalar::<_, String>(
                "SELECT COALESCE(NULLIF(username, ''), email) FROM users WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(RepositoryError::OwnerNotFound)?;
            Ok(("user".to_owned(), *id, login))
        }
        RepositoryOwner::Organization { id } => {
            let login =
                sqlx::query_scalar::<_, String>("SELECT slug FROM organizations WHERE id = $1")
                    .bind(id)
                    .fetch_optional(pool)
                    .await?
                    .ok_or(RepositoryError::OwnerNotFound)?;
            Ok(("organization".to_owned(), *id, login))
        }
    }
}

async fn repository_exists_for_owner(
    pool: &PgPool,
    owner: &RepositoryOwner,
    name: &str,
) -> Result<bool, RepositoryError> {
    let exists = match owner {
        RepositoryOwner::User { id } => {
            sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS (
                    SELECT 1 FROM repositories
                    WHERE owner_user_id = $1 AND lower(name) = lower($2)
                )
                "#,
            )
            .bind(id)
            .bind(name)
            .fetch_one(pool)
            .await?
        }
        RepositoryOwner::Organization { id } => {
            sqlx::query_scalar::<_, bool>(
                r#"
                SELECT EXISTS (
                    SELECT 1 FROM repositories
                    WHERE owner_organization_id = $1 AND lower(name) = lower($2)
                )
                "#,
            )
            .bind(id)
            .bind(name)
            .fetch_one(pool)
            .await?
        }
    };

    Ok(exists)
}

pub fn normalize_repository_name(value: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_hyphen = false;

    for character in value.trim().chars() {
        let next = if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
            character
        } else {
            '-'
        };

        if next == '-' {
            if previous_was_hyphen {
                continue;
            }
            previous_was_hyphen = true;
        } else {
            previous_was_hyphen = false;
        }
        normalized.push(next);
    }

    normalized.trim_matches('-').to_owned()
}

fn normalize_repository_description(
    value: Option<String>,
) -> Result<Option<String>, RepositoryError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let trimmed = value.trim().to_owned();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > 350 {
        return Err(RepositoryError::InvalidDescription(
            "Repository description must be 350 characters or fewer.".to_owned(),
        ));
    }
    Ok(Some(trimmed))
}

fn validate_repository_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Repository name is required.".to_owned());
    }
    if name.len() > 100 {
        return Err("Repository name must be 100 characters or fewer.".to_owned());
    }
    if name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
    {
        Ok(())
    } else {
        Err(
            "Repository names can only include letters, numbers, dots, underscores, and hyphens."
                .to_owned(),
        )
    }
}

async fn ensure_default_repository_labels(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<(), RepositoryError> {
    const DEFAULT_LABELS: [(&str, &str, &str); 4] = [
        ("bug", "d73a4a", "Something is not working"),
        (
            "documentation",
            "0075ca",
            "Improvements or additions to documentation",
        ),
        ("enhancement", "a2eeef", "New feature or request"),
        ("good first issue", "7057ff", "Good for newcomers"),
    ];

    for (name, color, description) in DEFAULT_LABELS {
        sqlx::query(
            r#"
            INSERT INTO labels (repository_id, name, color, description, is_default)
            VALUES ($1, $2, $3, $4, true)
            ON CONFLICT (repository_id, lower(name)) DO NOTHING
            "#,
        )
        .bind(repository_id)
        .bind(name)
        .bind(color)
        .bind(description)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn insert_repository_create_feed_event(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO feed_events (
            actor_user_id,
            repository_id,
            event_type,
            title,
            excerpt,
            target_href,
            subject_type,
            subject_id,
            metadata
        )
        VALUES ($1, $2, 'repository_create', $3, $4, $5, 'repository', $2, $6)
        "#,
    )
    .bind(actor_user_id)
    .bind(repository.id)
    .bind(format!(
        "Created repository {}/{}",
        repository.owner_login, repository.name
    ))
    .bind(repository.description.as_deref())
    .bind(format!("/{}/{}", repository.owner_login, repository.name))
    .bind(serde_json::json!({
        "visibility": repository.visibility.as_str(),
        "defaultBranch": repository.default_branch,
    }))
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_repository_social_feed_event(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    event_type: &str,
) -> Result<(), RepositoryError> {
    let (title, excerpt) = match event_type {
        "star" => (
            format!("Starred {}/{}", repository.owner_login, repository.name),
            "Repository starred from the Code tab.",
        ),
        _ => return Ok(()),
    };
    sqlx::query(
        r#"
        INSERT INTO feed_events (
            actor_user_id,
            repository_id,
            event_type,
            title,
            excerpt,
            target_href,
            subject_type,
            subject_id,
            metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, 'repository', $2, $7)
        "#,
    )
    .bind(actor_user_id)
    .bind(repository.id)
    .bind(event_type)
    .bind(title)
    .bind(excerpt)
    .bind(format!("/{}/{}", repository.owner_login, repository.name))
    .bind(serde_json::json!({
        "source": "repository_header",
        "repository": format!("{}/{}", repository.owner_login, repository.name),
    }))
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_repository_fork_feed_event(
    pool: &PgPool,
    source_repository: &Repository,
    fork_repository: &Repository,
    actor_user_id: Uuid,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO feed_events (
            actor_user_id,
            repository_id,
            event_type,
            title,
            excerpt,
            target_href,
            subject_type,
            subject_id,
            metadata
        )
        VALUES ($1, $2, 'fork', $3, $4, $5, 'repository', $6, $7)
        "#,
    )
    .bind(actor_user_id)
    .bind(source_repository.id)
    .bind(format!(
        "Forked {}/{}",
        source_repository.owner_login, source_repository.name
    ))
    .bind(format!(
        "Created fork {}/{}",
        fork_repository.owner_login, fork_repository.name
    ))
    .bind(format!(
        "/{}/{}",
        fork_repository.owner_login, fork_repository.name
    ))
    .bind(fork_repository.id)
    .bind(serde_json::json!({
        "sourceRepository": format!("{}/{}", source_repository.owner_login, source_repository.name),
        "forkRepository": format!("{}/{}", fork_repository.owner_login, fork_repository.name),
    }))
    .execute(pool)
    .await?;

    Ok(())
}

fn suggested_repository_name(actor_user_id: Uuid) -> String {
    let words = [
        "silver-train",
        "probable-octo",
        "refactored-disco",
        "friendly-engine",
    ];
    let index = actor_user_id.as_bytes()[0] as usize % words.len();
    words[index].to_owned()
}

fn organization_from_row(row: sqlx::postgres::PgRow) -> Organization {
    Organization {
        id: row.get("id"),
        slug: row.get("slug"),
        display_name: row.get("display_name"),
        description: row.get("description"),
        owner_user_id: row.get("owner_user_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn repository_from_row(row: sqlx::postgres::PgRow) -> Result<Repository, RepositoryError> {
    let visibility: String = row.get("visibility");
    Ok(Repository {
        id: row.get("id"),
        owner_user_id: row.get("owner_user_id"),
        owner_organization_id: row.get("owner_organization_id"),
        owner_login: row.get("owner_login"),
        name: row.get("name"),
        description: row.get("description"),
        visibility: RepositoryVisibility::try_from(visibility.as_str())?,
        default_branch: row.get("default_branch"),
        is_archived: row.get("is_archived"),
        created_by_user_id: row.get("created_by_user_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn repository_permission_from_row(
    row: sqlx::postgres::PgRow,
) -> Result<RepositoryPermission, RepositoryError> {
    let role: String = row.get("role");
    Ok(RepositoryPermission {
        repository_id: row.get("repository_id"),
        user_id: row.get("user_id"),
        role: RepositoryRole::try_from(role.as_str())
            .map_err(|error| RepositoryError::Sqlx(sqlx::Error::Protocol(error.to_string())))?,
        source: row.get("source"),
    })
}

fn commit_from_row(row: sqlx::postgres::PgRow) -> Commit {
    Commit {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        oid: row.get("oid"),
        author_user_id: row.get("author_user_id"),
        committer_user_id: row.get("committer_user_id"),
        message: row.get("message"),
        tree_oid: row.get("tree_oid"),
        parent_oids: row.get("parent_oids"),
        committed_at: row.get("committed_at"),
        created_at: row.get("created_at"),
    }
}

fn git_ref_from_row(row: sqlx::postgres::PgRow) -> GitRef {
    GitRef {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        name: row.get("name"),
        kind: row.get("kind"),
        target_commit_id: row.get("target_commit_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn repository_file_from_row(row: sqlx::postgres::PgRow) -> RepositoryFile {
    RepositoryFile {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        commit_id: row.get("commit_id"),
        path: row.get("path"),
        content: row.get("content"),
        oid: row.get("oid"),
        byte_size: row.get("byte_size"),
        created_at: row.get("created_at"),
    }
}
