use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, QueryBuilder, Row};
use uuid::Uuid;

use crate::{
    api_types::{normalize_pagination, ListEnvelope},
    domain::markdown::{render_markdown, MarkdownError, RenderMarkdownInput},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageOwnerKind {
    User,
    Organization,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OwnerPackageList {
    #[serde(flatten)]
    pub envelope: ListEnvelope<OwnerPackageListItem>,
    pub owner: OwnerPackageOwner,
    pub mode: String,
    pub filters: OwnerPackageFilters,
    pub linked_artifacts: LinkedArtifactsPlaceholder,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OwnerPackageOwner {
    pub id: Uuid,
    pub login: String,
    pub kind: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OwnerPackageListItem {
    pub id: Uuid,
    pub name: String,
    pub package_type: String,
    pub type_label: String,
    pub visibility: String,
    pub href: String,
    pub published_at: DateTime<Utc>,
    pub publisher: OwnerPackagePublisher,
    pub linked_repository: Option<OwnerPackageRepository>,
    pub download_count: i64,
    pub latest_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OwnerPackagePublisher {
    pub id: Uuid,
    pub login: String,
    pub name: Option<String>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OwnerPackageRepository {
    pub id: Uuid,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub href: String,
    pub visibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OwnerPackageFilters {
    pub query: Option<String>,
    pub package_type: String,
    pub visibility: String,
    pub sort: String,
    pub artifact_tab: String,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LinkedArtifactsPlaceholder {
    pub enabled: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageDetail {
    pub id: Uuid,
    pub name: String,
    pub package_type: String,
    pub type_label: String,
    pub visibility: String,
    pub href: String,
    pub owner: OwnerPackageOwner,
    pub publisher: OwnerPackagePublisher,
    pub linked_repository: Option<OwnerPackageRepository>,
    pub published_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub download_count: i64,
    pub selected_version: Option<PackageDetailVersion>,
    pub versions: Vec<PackageDetailVersion>,
    pub install_commands: Vec<PackageInstallCommand>,
    pub blobs: Vec<PackageBlobSummary>,
    pub about: PackageAboutContent,
    pub admin: PackageAdminState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageDownloadMetadata {
    pub package_id: Uuid,
    pub version_id: Option<Uuid>,
    pub version: Option<String>,
    pub digest: Option<String>,
    pub short_digest: Option<String>,
    pub command: Option<String>,
    pub download_count: i64,
    pub storage_available: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageDetailVersion {
    pub id: Uuid,
    pub version: String,
    pub digest: Option<String>,
    pub short_digest: Option<String>,
    pub platform_os: Option<String>,
    pub platform_arch: Option<String>,
    pub size_bytes: Option<i64>,
    pub published_at: DateTime<Utc>,
    pub publisher: OwnerPackagePublisher,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageBlobSummary {
    pub id: Uuid,
    pub version_id: Option<Uuid>,
    pub digest: String,
    pub short_digest: String,
    pub media_type: Option<String>,
    pub platform_os: Option<String>,
    pub platform_arch: Option<String>,
    pub size_bytes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageInstallCommand {
    pub label: String,
    pub command: String,
    pub version: Option<String>,
    pub digest: Option<String>,
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageAboutContent {
    pub source: String,
    pub markdown: Option<String>,
    pub html: Option<String>,
    pub empty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageAdminState {
    pub can_admin: bool,
    pub settings_href: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageSettings {
    pub package: PackageSettingsSummary,
    pub owner: OwnerPackageOwner,
    pub linked_repositories: Vec<OwnerPackageRepository>,
    pub explicit_permissions: Vec<PackagePermissionSummary>,
    pub inherited_repository_access: Vec<PackageRepositoryAccessSummary>,
    pub recent_activity: Vec<PackageActivitySummary>,
    pub registry_write_capabilities: Vec<PackageCapabilitySummary>,
    pub admin: PackageAdminState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageSettingsSummary {
    pub id: Uuid,
    pub name: String,
    pub package_type: String,
    pub type_label: String,
    pub visibility: String,
    pub href: String,
    pub download_count: i64,
    pub latest_version: Option<String>,
    pub latest_digest: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackagePermissionSummary {
    pub user_id: Uuid,
    pub login: String,
    pub display_name: Option<String>,
    pub role: String,
    pub href: String,
    pub granted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageRepositoryAccessSummary {
    pub repository: OwnerPackageRepository,
    pub user_id: Uuid,
    pub login: String,
    pub role: String,
    pub source: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageActivitySummary {
    pub kind: String,
    pub label: String,
    pub actor: Option<OwnerPackagePublisher>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PackageCapabilitySummary {
    pub key: String,
    pub label: String,
    pub enabled: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Copy)]
pub struct OwnerPackageListQuery<'a> {
    pub query: Option<&'a str>,
    pub package_type: Option<&'a str>,
    pub visibility: Option<&'a str>,
    pub sort: Option<&'a str>,
    pub artifact_tab: Option<&'a str>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Copy)]
pub struct PackageDetailQuery<'a> {
    pub version: Option<&'a str>,
}

#[derive(Debug, thiserror::Error)]
pub enum PackageListError {
    #[error("package owner was not found")]
    NotFound,
    #[error("{0}")]
    InvalidFilter(String),
    #[error("database error")]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PackageDetailError {
    #[error("package was not found")]
    NotFound,
    #[error("package settings require admin access")]
    Forbidden,
    #[error("{0}")]
    InvalidSelection(String),
    #[error("markdown rendering failed")]
    Markdown(#[from] MarkdownError),
    #[error("database error")]
    Sqlx(#[from] sqlx::Error),
}

struct OwnerRow {
    id: Uuid,
    login: String,
    kind: String,
}

pub async fn owner_packages(
    pool: &PgPool,
    owner_login: &str,
    owner_kind: PackageOwnerKind,
    actor_user_id: Option<Uuid>,
    query: OwnerPackageListQuery<'_>,
) -> Result<OwnerPackageList, PackageListError> {
    let owner = resolve_owner(pool, owner_login, owner_kind).await?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let filters = normalize_filters(query, pagination.page, pagination.page_size)?;
    let visibility_sql = visibility_predicate(owner_kind, actor_user_id.is_some());

    let mut count_builder: QueryBuilder<'_, sqlx::Postgres> =
        QueryBuilder::new("SELECT COUNT(*)::bigint FROM packages p WHERE ");
    push_owner_predicate(&mut count_builder, owner_kind, owner.id);
    count_builder.push(" AND ");
    count_builder.push(visibility_sql.as_str());
    bind_auth(&mut count_builder, actor_user_id);
    push_filter_predicates(&mut count_builder, &filters);
    let total = count_builder
        .build_query_scalar::<i64>()
        .fetch_one(pool)
        .await?;

    let mut rows_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        r#"
        SELECT p.id,
               p.name,
               p.package_type,
               p.visibility,
               p.created_at AS published_at,
               p.created_by_user_id AS publisher_id,
               COALESCE(NULLIF(publisher.username, ''), split_part(publisher.email, '@', 1)) AS publisher_login,
               publisher.display_name AS publisher_name,
               linked_repo.id AS linked_repository_id,
               COALESCE(linked_owner_user.username, linked_owner_org.slug) AS linked_repository_owner,
               linked_repo.name AS linked_repository_name,
               linked_repo.visibility AS linked_repository_visibility,
               latest.version AS latest_version,
               COALESCE(downloads.download_count, 0)::bigint AS download_count
        FROM packages p
        JOIN users publisher ON publisher.id = p.created_by_user_id
        LEFT JOIN LATERAL (
            SELECT pr.repository_id
            FROM package_repository_links pr
            WHERE pr.package_id = p.id
            ORDER BY pr.created_at DESC
            LIMIT 1
        ) package_link ON true
        LEFT JOIN repositories linked_repo ON linked_repo.id = COALESCE(package_link.repository_id, p.repository_id)
        LEFT JOIN users linked_owner_user ON linked_owner_user.id = linked_repo.owner_user_id
        LEFT JOIN organizations linked_owner_org ON linked_owner_org.id = linked_repo.owner_organization_id
        LEFT JOIN LATERAL (
            SELECT version
            FROM package_versions pv
            WHERE pv.package_id = p.id
            ORDER BY pv.created_at DESC, lower(pv.version)
            LIMIT 1
        ) latest ON true
        LEFT JOIN LATERAL (
            SELECT COALESCE(SUM(pd.download_count), 0)::bigint AS download_count
            FROM package_downloads pd
            WHERE pd.package_id = p.id
        ) downloads ON true
        WHERE "#,
    );
    push_owner_predicate(&mut rows_builder, owner_kind, owner.id);
    rows_builder.push(" AND ");
    rows_builder.push(visibility_sql.as_str());
    bind_auth(&mut rows_builder, actor_user_id);
    push_filter_predicates(&mut rows_builder, &filters);
    push_order(&mut rows_builder, &filters.sort);
    rows_builder.push(" LIMIT ");
    rows_builder.push_bind(filters.page_size);
    rows_builder.push(" OFFSET ");
    rows_builder.push_bind((filters.page - 1) * filters.page_size);

    let rows = rows_builder.build().fetch_all(pool).await?;
    let items = rows
        .into_iter()
        .map(|row| package_item_from_row(row, owner_kind, &owner.login))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(OwnerPackageList {
        envelope: ListEnvelope {
            items,
            total,
            page: filters.page,
            page_size: filters.page_size,
        },
        owner: OwnerPackageOwner {
            id: owner.id,
            login: owner.login.clone(),
            kind: owner.kind,
            href: owner_href(owner_kind, &owner.login),
        },
        mode: "packages".to_owned(),
        filters,
        linked_artifacts: LinkedArtifactsPlaceholder {
            enabled: false,
            message: "Linked artifact provenance is not implemented yet; package repository links are shown in the package list.".to_owned(),
        },
    })
}

pub async fn package_detail(
    pool: &PgPool,
    owner_login: &str,
    owner_kind: PackageOwnerKind,
    package_type: &str,
    package_name: &str,
    actor_user_id: Option<Uuid>,
    query: PackageDetailQuery<'_>,
) -> Result<PackageDetail, PackageDetailError> {
    let normalized_type = package_type.trim().to_ascii_lowercase();
    if !matches!(
        normalized_type.as_str(),
        "container" | "npm" | "rubygems" | "maven" | "nuget" | "generic"
    ) {
        return Err(PackageDetailError::NotFound);
    }
    let owner =
        resolve_owner(pool, owner_login, owner_kind)
            .await
            .map_err(|error| match error {
                PackageListError::NotFound => PackageDetailError::NotFound,
                PackageListError::InvalidFilter(message) => {
                    PackageDetailError::InvalidSelection(message)
                }
                PackageListError::Sqlx(error) => PackageDetailError::Sqlx(error),
            })?;

    let mut package_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
        r#"
        SELECT p.id,
               p.name,
               p.package_type,
               p.visibility,
               p.created_at AS published_at,
               p.updated_at,
               p.created_by_user_id AS publisher_id,
               COALESCE(NULLIF(publisher.username, ''), split_part(publisher.email, '@', 1)) AS publisher_login,
               publisher.display_name AS publisher_name,
               linked_repo.id AS linked_repository_id,
               COALESCE(linked_owner_user.username, linked_owner_org.slug) AS linked_repository_owner,
               linked_repo.name AS linked_repository_name,
               linked_repo.visibility AS linked_repository_visibility,
               COALESCE(downloads.download_count, 0)::bigint AS download_count,
               (p.visibility = 'public') AS public_readable,
               COALESCE((p.owner_user_id = "#,
    );
    package_builder.push_bind(actor_user_id);
    package_builder.push(
        r#"), false) AS actor_owns_user_package,
               EXISTS (
                   SELECT 1
                   FROM organization_memberships om
                   WHERE om.organization_id = p.owner_organization_id
                     AND om.user_id = "#,
    );
    package_builder.push_bind(actor_user_id);
    package_builder.push(
        r#"
               ) AS actor_is_org_member,
               EXISTS (
                   SELECT 1
                   FROM organization_memberships om
                   WHERE om.organization_id = p.owner_organization_id
                     AND om.user_id = "#,
    );
    package_builder.push_bind(actor_user_id);
    package_builder.push(
        r#"
                     AND om.role IN ('owner', 'admin')
               ) AS actor_admins_org,
               EXISTS (
                   SELECT 1
                   FROM package_permissions pp
                   WHERE pp.package_id = p.id
                     AND pp.user_id = "#,
    );
    package_builder.push_bind(actor_user_id);
    package_builder.push(
        r#"
                     AND pp.role IN ('read', 'write', 'admin')
               ) AS actor_can_read_package,
               EXISTS (
                   SELECT 1
                   FROM package_permissions pp
                   WHERE pp.package_id = p.id
                     AND pp.user_id = "#,
    );
    package_builder.push_bind(actor_user_id);
    package_builder.push(
        r#"
                     AND pp.role = 'admin'
               ) AS actor_admins_package,
               EXISTS (
                   SELECT 1
                   FROM repository_permissions rp
                   WHERE rp.user_id = "#,
    );
    package_builder.push_bind(actor_user_id);
    package_builder.push(
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
               ) AS actor_can_read_linked_repo,
               EXISTS (
                   SELECT 1
                   FROM repository_permissions rp
                   WHERE rp.user_id = "#,
    );
    package_builder.push_bind(actor_user_id);
    package_builder.push(
        r#"
                     AND rp.role IN ('owner', 'admin')
                     AND (
                         rp.repository_id = p.repository_id
                         OR EXISTS (
                             SELECT 1
                             FROM package_repository_links pr
                             WHERE pr.package_id = p.id
                               AND pr.repository_id = rp.repository_id
                         )
                     )
               ) AS actor_admins_linked_repo
        FROM packages p
        JOIN users publisher ON publisher.id = p.created_by_user_id
        LEFT JOIN LATERAL (
            SELECT pr.repository_id
            FROM package_repository_links pr
            WHERE pr.package_id = p.id
            ORDER BY pr.created_at DESC
            LIMIT 1
        ) package_link ON true
        LEFT JOIN repositories linked_repo ON linked_repo.id = COALESCE(package_link.repository_id, p.repository_id)
        LEFT JOIN users linked_owner_user ON linked_owner_user.id = linked_repo.owner_user_id
        LEFT JOIN organizations linked_owner_org ON linked_owner_org.id = linked_repo.owner_organization_id
        LEFT JOIN LATERAL (
            SELECT COALESCE(SUM(pd.download_count), 0)::bigint AS download_count
            FROM package_downloads pd
            WHERE pd.package_id = p.id
        ) downloads ON true
        WHERE "#,
    );
    push_owner_predicate(&mut package_builder, owner_kind, owner.id);
    package_builder.push(" AND lower(p.package_type) = lower(");
    package_builder.push_bind(&normalized_type);
    package_builder.push(") AND lower(p.name) = lower(");
    package_builder.push_bind(package_name.trim());
    package_builder.push(")");

    let Some(row) = package_builder.build().fetch_optional(pool).await? else {
        return Err(PackageDetailError::NotFound);
    };

    let public_readable: bool = row.try_get("public_readable")?;
    let actor_owns_user_package: bool = row.try_get("actor_owns_user_package")?;
    let actor_is_org_member: bool = row.try_get("actor_is_org_member")?;
    let actor_admins_org: bool = row.try_get("actor_admins_org")?;
    let actor_can_read_package: bool = row.try_get("actor_can_read_package")?;
    let actor_admins_package: bool = row.try_get("actor_admins_package")?;
    let actor_can_read_linked_repo: bool = row.try_get("actor_can_read_linked_repo")?;
    let actor_admins_linked_repo: bool = row.try_get("actor_admins_linked_repo")?;
    let can_read = public_readable
        || actor_owns_user_package
        || actor_is_org_member
        || actor_can_read_package
        || actor_can_read_linked_repo;
    if !can_read {
        return Err(PackageDetailError::NotFound);
    }
    let can_admin = actor_owns_user_package
        || actor_admins_org
        || actor_admins_package
        || actor_admins_linked_repo;

    let package_id: Uuid = row.try_get("id")?;
    let package_type: String = row.try_get("package_type")?;
    let name: String = row.try_get("name")?;
    let publisher_login: String = row.try_get("publisher_login")?;
    let linked_repository = linked_repository_from_row(&row)?;
    let versions = package_versions(
        pool,
        owner_kind,
        &owner.login,
        &package_type,
        &name,
        package_id,
    )
    .await?;
    let selected_version = select_version(&versions, query.version)?;
    let blobs = package_blobs(
        pool,
        package_id,
        selected_version.as_ref().map(|version| version.id),
    )
    .await?;
    let install_commands = install_commands(
        &owner.login,
        &package_type,
        &name,
        selected_version.as_ref(),
        &blobs,
    );
    let about = package_about(
        pool,
        package_id,
        linked_repository.as_ref().map(|repo| repo.id),
    )
    .await?;
    let href = package_href(owner_kind, &owner.login, &package_type, &name);

    Ok(PackageDetail {
        id: package_id,
        name,
        type_label: package_type_label(&package_type).to_owned(),
        package_type,
        visibility: row.try_get("visibility")?,
        href: href.clone(),
        owner: OwnerPackageOwner {
            id: owner.id,
            login: owner.login.clone(),
            kind: owner.kind,
            href: owner_href(owner_kind, &owner.login),
        },
        publisher: OwnerPackagePublisher {
            id: row.try_get("publisher_id")?,
            href: format!("/{}", url_component(&publisher_login)),
            login: publisher_login,
            name: row.try_get("publisher_name")?,
        },
        linked_repository,
        published_at: row.try_get("published_at")?,
        updated_at: row.try_get("updated_at")?,
        download_count: row.try_get("download_count")?,
        selected_version,
        versions,
        install_commands,
        blobs,
        about,
        admin: PackageAdminState {
            can_admin,
            settings_href: can_admin.then(|| format!("{href}/settings")),
            reason: (!can_admin).then(|| {
                "Package settings require owner, package admin, or linked repository admin access."
                    .to_owned()
            }),
        },
    })
}

pub async fn record_package_download_metadata(
    pool: &PgPool,
    owner_login: &str,
    owner_kind: PackageOwnerKind,
    package_type: &str,
    package_name: &str,
    actor_user_id: Option<Uuid>,
    query: PackageDetailQuery<'_>,
) -> Result<PackageDownloadMetadata, PackageDetailError> {
    let detail = package_detail(
        pool,
        owner_login,
        owner_kind,
        package_type,
        package_name,
        actor_user_id,
        query,
    )
    .await?;
    let selected_version = detail.selected_version.as_ref();
    sqlx::query(
        r#"
        INSERT INTO package_downloads (package_id, package_version_id, downloaded_by_user_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(detail.id)
    .bind(selected_version.map(|version| version.id))
    .bind(actor_user_id)
    .execute(pool)
    .await?;
    let download_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(SUM(download_count), 0)::bigint
        FROM package_downloads
        WHERE package_id = $1
        "#,
    )
    .bind(detail.id)
    .fetch_one(pool)
    .await?;

    Ok(PackageDownloadMetadata {
        package_id: detail.id,
        version_id: selected_version.map(|version| version.id),
        version: selected_version.map(|version| version.version.clone()),
        digest: selected_version.and_then(|version| version.digest.clone()),
        short_digest: selected_version.and_then(|version| version.short_digest.clone()),
        command: detail
            .install_commands
            .iter()
            .find(|command| command.digest.is_some())
            .or_else(|| detail.install_commands.first())
            .map(|command| command.command.clone()),
        download_count,
        storage_available: false,
        message: "Download metadata was recorded. Registry blob transfer is implemented in the package registry protocol slice.".to_owned(),
    })
}

pub async fn package_settings(
    pool: &PgPool,
    owner_login: &str,
    owner_kind: PackageOwnerKind,
    package_type: &str,
    package_name: &str,
    actor_user_id: Option<Uuid>,
) -> Result<PackageSettings, PackageDetailError> {
    let detail = package_detail(
        pool,
        owner_login,
        owner_kind,
        package_type,
        package_name,
        actor_user_id,
        PackageDetailQuery { version: None },
    )
    .await?;

    if !detail.admin.can_admin {
        return Err(PackageDetailError::Forbidden);
    }

    let linked_repositories = package_linked_repositories(pool, detail.id).await?;
    let explicit_permissions = package_permission_summaries(pool, detail.id).await?;
    let inherited_repository_access = package_repository_access_summaries(pool, detail.id).await?;
    let recent_activity = package_recent_activity(pool, detail.id).await?;

    Ok(PackageSettings {
        package: PackageSettingsSummary {
            id: detail.id,
            name: detail.name,
            package_type: detail.package_type,
            type_label: detail.type_label,
            visibility: detail.visibility,
            href: detail.href,
            download_count: detail.download_count,
            latest_version: detail
                .selected_version
                .as_ref()
                .map(|version| version.version.clone()),
            latest_digest: detail
                .selected_version
                .as_ref()
                .and_then(|version| version.digest.clone()),
            updated_at: detail.updated_at,
        },
        owner: detail.owner,
        linked_repositories,
        explicit_permissions,
        inherited_repository_access,
        recent_activity,
        registry_write_capabilities: package_registry_capabilities(),
        admin: detail.admin,
    })
}

async fn resolve_owner(
    pool: &PgPool,
    owner_login: &str,
    owner_kind: PackageOwnerKind,
) -> Result<OwnerRow, PackageListError> {
    let row = match owner_kind {
        PackageOwnerKind::User => {
            sqlx::query(
                r#"
            SELECT id,
                   COALESCE(NULLIF(username, ''), split_part(email, '@', 1)) AS login,
                   'user' AS kind
            FROM users
            WHERE lower(COALESCE(username, split_part(email, '@', 1))) = lower($1)
            "#,
            )
            .bind(owner_login)
            .fetch_optional(pool)
            .await?
        }
        PackageOwnerKind::Organization => {
            sqlx::query(
                r#"
            SELECT id, slug AS login, 'organization' AS kind
            FROM organizations
            WHERE lower(slug) = lower($1)
            "#,
            )
            .bind(owner_login)
            .fetch_optional(pool)
            .await?
        }
    }
    .ok_or(PackageListError::NotFound)?;

    Ok(OwnerRow {
        id: row.try_get("id")?,
        login: row.try_get("login")?,
        kind: row.try_get("kind")?,
    })
}

fn normalize_filters(
    query: OwnerPackageListQuery<'_>,
    page: i64,
    page_size: i64,
) -> Result<OwnerPackageFilters, PackageListError> {
    let package_type = query
        .package_type
        .unwrap_or("all")
        .trim()
        .to_ascii_lowercase();
    let visibility = query
        .visibility
        .unwrap_or("all")
        .trim()
        .to_ascii_lowercase();
    let sort = query
        .sort
        .unwrap_or("downloads-desc")
        .trim()
        .to_ascii_lowercase();
    let artifact_tab = query
        .artifact_tab
        .unwrap_or("packages")
        .trim()
        .to_ascii_lowercase();
    if !matches!(
        package_type.as_str(),
        "all" | "container" | "npm" | "rubygems" | "maven" | "nuget" | "generic"
    ) {
        return Err(PackageListError::InvalidFilter(
            "package type must be all, container, npm, RubyGems, Maven, NuGet, or generic"
                .to_owned(),
        ));
    }
    if !matches!(
        visibility.as_str(),
        "all" | "public" | "internal" | "private"
    ) {
        return Err(PackageListError::InvalidFilter(
            "visibility must be all, public, internal, or private".to_owned(),
        ));
    }
    if !matches!(sort.as_str(), "downloads-desc" | "downloads-asc") {
        return Err(PackageListError::InvalidFilter(
            "sort must be downloads-desc or downloads-asc".to_owned(),
        ));
    }
    if !matches!(artifact_tab.as_str(), "packages" | "artifacts") {
        return Err(PackageListError::InvalidFilter(
            "artifact tab must be packages or artifacts".to_owned(),
        ));
    }

    Ok(OwnerPackageFilters {
        query: query.query.and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_owned())
        }),
        package_type,
        visibility,
        sort,
        artifact_tab,
        page,
        page_size,
    })
}

fn push_owner_predicate(
    builder: &mut QueryBuilder<'_, sqlx::Postgres>,
    owner_kind: PackageOwnerKind,
    owner_id: Uuid,
) {
    match owner_kind {
        PackageOwnerKind::User => {
            builder.push("p.owner_user_id = ");
        }
        PackageOwnerKind::Organization => {
            builder.push("p.owner_organization_id = ");
        }
    }
    builder.push_bind(owner_id);
}

fn push_filter_predicates(
    builder: &mut QueryBuilder<'_, sqlx::Postgres>,
    filters: &OwnerPackageFilters,
) {
    if let Some(query) = filters.query.as_ref() {
        builder.push(" AND p.name ILIKE ");
        builder.push_bind(format!("%{}%", escape_like(query)));
        builder.push(" ESCAPE '\\'");
    }
    if filters.package_type != "all" {
        builder.push(" AND p.package_type = ");
        builder.push_bind(filters.package_type.clone());
    }
    if filters.visibility != "all" {
        builder.push(" AND p.visibility = ");
        builder.push_bind(filters.visibility.clone());
    }
}

fn push_order(builder: &mut QueryBuilder<'_, sqlx::Postgres>, sort: &str) {
    if sort == "downloads-asc" {
        builder.push(" ORDER BY COALESCE(downloads.download_count, 0) ASC, lower(p.name) ASC, p.created_at DESC");
    } else {
        builder.push(" ORDER BY COALESCE(downloads.download_count, 0) DESC, lower(p.name) ASC, p.created_at DESC");
    }
}

fn bind_auth(builder: &mut QueryBuilder<'_, sqlx::Postgres>, actor_user_id: Option<Uuid>) {
    if let Some(actor_user_id) = actor_user_id {
        builder.push_bind(actor_user_id);
    }
}

fn visibility_predicate(owner_kind: PackageOwnerKind, authenticated: bool) -> String {
    if !authenticated {
        return "p.visibility = 'public'".to_owned();
    }
    match owner_kind {
        PackageOwnerKind::User => "(p.visibility = 'public' OR p.owner_user_id = $2 OR EXISTS (SELECT 1 FROM package_permissions pp WHERE pp.package_id = p.id AND pp.user_id = $2 AND pp.role IN ('read', 'write', 'admin')) OR EXISTS (SELECT 1 FROM repository_permissions rp WHERE rp.repository_id = p.repository_id AND rp.user_id = $2 AND rp.role IN ('read', 'write', 'admin', 'owner')))".to_owned(),
        PackageOwnerKind::Organization => "(p.visibility = 'public' OR EXISTS (SELECT 1 FROM organization_memberships om WHERE om.organization_id = p.owner_organization_id AND om.user_id = $2) OR EXISTS (SELECT 1 FROM package_permissions pp WHERE pp.package_id = p.id AND pp.user_id = $2 AND pp.role IN ('read', 'write', 'admin')) OR EXISTS (SELECT 1 FROM repository_permissions rp WHERE rp.repository_id = p.repository_id AND rp.user_id = $2 AND rp.role IN ('read', 'write', 'admin', 'owner')))".to_owned(),
    }
}

fn package_item_from_row(
    row: sqlx::postgres::PgRow,
    owner_kind: PackageOwnerKind,
    owner_login: &str,
) -> Result<OwnerPackageListItem, sqlx::Error> {
    let package_type: String = row.try_get("package_type")?;
    let repository_id: Option<Uuid> = row.try_get("linked_repository_id")?;
    let repository_owner: Option<String> = row.try_get("linked_repository_owner")?;
    let repository_name: Option<String> = row.try_get("linked_repository_name")?;
    let repository_visibility: Option<String> = row.try_get("linked_repository_visibility")?;
    let linked_repository = match (
        repository_id,
        repository_owner,
        repository_name,
        repository_visibility,
    ) {
        (Some(id), Some(owner), Some(name), Some(visibility)) => Some(OwnerPackageRepository {
            id,
            full_name: format!("{owner}/{name}"),
            href: format!("/{}/{}", url_component(&owner), url_component(&name)),
            owner,
            name,
            visibility,
        }),
        _ => None,
    };
    let name: String = row.try_get("name")?;
    let id: Uuid = row.try_get("id")?;
    let publisher_login: String = row.try_get("publisher_login")?;
    Ok(OwnerPackageListItem {
        id,
        href: package_href(owner_kind, owner_login, &package_type, &name),
        name,
        type_label: package_type_label(&package_type).to_owned(),
        package_type,
        visibility: row.try_get("visibility")?,
        published_at: row.try_get("published_at")?,
        publisher: OwnerPackagePublisher {
            id: row.try_get("publisher_id")?,
            href: format!("/{}", url_component(&publisher_login)),
            login: publisher_login,
            name: row.try_get("publisher_name")?,
        },
        linked_repository,
        download_count: row.try_get("download_count")?,
        latest_version: row.try_get("latest_version")?,
    })
}

fn linked_repository_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<Option<OwnerPackageRepository>, sqlx::Error> {
    let repository_id: Option<Uuid> = row.try_get("linked_repository_id")?;
    let repository_owner: Option<String> = row.try_get("linked_repository_owner")?;
    let repository_name: Option<String> = row.try_get("linked_repository_name")?;
    let repository_visibility: Option<String> = row.try_get("linked_repository_visibility")?;
    Ok(
        match (
            repository_id,
            repository_owner,
            repository_name,
            repository_visibility,
        ) {
            (Some(id), Some(owner), Some(name), Some(visibility)) => Some(OwnerPackageRepository {
                id,
                full_name: format!("{owner}/{name}"),
                href: format!("/{}/{}", url_component(&owner), url_component(&name)),
                owner,
                name,
                visibility,
            }),
            _ => None,
        },
    )
}

async fn package_versions(
    pool: &PgPool,
    owner_kind: PackageOwnerKind,
    owner_login: &str,
    package_type: &str,
    package_name: &str,
    package_id: Uuid,
) -> Result<Vec<PackageDetailVersion>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT pv.id,
               pv.version,
               pv.digest,
               pv.platform_os,
               pv.platform_arch,
               pv.size_bytes,
               pv.created_at AS published_at,
               pv.published_by_user_id AS publisher_id,
               COALESCE(NULLIF(publisher.username, ''), split_part(publisher.email, '@', 1)) AS publisher_login,
               publisher.display_name AS publisher_name
        FROM package_versions pv
        JOIN users publisher ON publisher.id = pv.published_by_user_id
        WHERE pv.package_id = $1
        ORDER BY pv.created_at DESC, lower(pv.version) ASC
        LIMIT 30
        "#,
    )
    .bind(package_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let digest: Option<String> = row.try_get("digest")?;
            let version: String = row.try_get("version")?;
            let publisher_login: String = row.try_get("publisher_login")?;
            let href = format!(
                "{}?version={}",
                package_href(owner_kind, owner_login, package_type, package_name),
                url_component(&version)
            );
            Ok(PackageDetailVersion {
                id: row.try_get("id")?,
                version,
                short_digest: digest.as_deref().map(short_digest),
                digest,
                platform_os: row.try_get("platform_os")?,
                platform_arch: row.try_get("platform_arch")?,
                size_bytes: row.try_get("size_bytes")?,
                published_at: row.try_get("published_at")?,
                publisher: OwnerPackagePublisher {
                    id: row.try_get("publisher_id")?,
                    href: format!("/{}", url_component(&publisher_login)),
                    login: publisher_login,
                    name: row.try_get("publisher_name")?,
                },
                href,
            })
        })
        .collect()
}

fn select_version(
    versions: &[PackageDetailVersion],
    selected: Option<&str>,
) -> Result<Option<PackageDetailVersion>, PackageDetailError> {
    let Some(selected) = selected.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(versions.first().cloned());
    };
    versions
        .iter()
        .find(|version| {
            version.version.eq_ignore_ascii_case(selected)
                || version
                    .digest
                    .as_deref()
                    .is_some_and(|digest| digest.eq_ignore_ascii_case(selected))
        })
        .cloned()
        .map(Some)
        .ok_or_else(|| {
            PackageDetailError::InvalidSelection(
                "selected package version or digest was not found".to_owned(),
            )
        })
}

async fn package_blobs(
    pool: &PgPool,
    package_id: Uuid,
    selected_version_id: Option<Uuid>,
) -> Result<Vec<PackageBlobSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id,
               package_version_id,
               digest,
               media_type,
               platform_os,
               platform_arch,
               size_bytes
        FROM package_blobs
        WHERE package_id = $1
          AND ($2::uuid IS NULL OR package_version_id = $2)
        ORDER BY created_at DESC, lower(digest) ASC
        LIMIT 30
        "#,
    )
    .bind(package_id)
    .bind(selected_version_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let digest: String = row.try_get("digest")?;
            Ok(PackageBlobSummary {
                id: row.try_get("id")?,
                version_id: row.try_get("package_version_id")?,
                short_digest: short_digest(&digest),
                digest,
                media_type: row.try_get("media_type")?,
                platform_os: row.try_get("platform_os")?,
                platform_arch: row.try_get("platform_arch")?,
                size_bytes: row.try_get("size_bytes")?,
            })
        })
        .collect()
}

async fn package_linked_repositories(
    pool: &PgPool,
    package_id: Uuid,
) -> Result<Vec<OwnerPackageRepository>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT repo.id,
               COALESCE(owner_user.username, owner_org.slug) AS owner,
               repo.name,
               repo.visibility
        FROM packages p
        JOIN repositories repo ON repo.id = p.repository_id
        LEFT JOIN users owner_user ON owner_user.id = repo.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = repo.owner_organization_id
        WHERE p.id = $1
        UNION
        SELECT linked_repo.id,
               COALESCE(linked_owner_user.username, linked_owner_org.slug) AS owner,
               linked_repo.name,
               linked_repo.visibility
        FROM package_repository_links pr
        JOIN repositories linked_repo ON linked_repo.id = pr.repository_id
        LEFT JOIN users linked_owner_user ON linked_owner_user.id = linked_repo.owner_user_id
        LEFT JOIN organizations linked_owner_org ON linked_owner_org.id = linked_repo.owner_organization_id
        WHERE pr.package_id = $1
        ORDER BY id
        "#,
    )
    .bind(package_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let owner: String = row.try_get("owner")?;
            let name: String = row.try_get("name")?;
            Ok(OwnerPackageRepository {
                id: row.try_get("id")?,
                full_name: format!("{owner}/{name}"),
                href: format!("/{}/{}", url_component(&owner), url_component(&name)),
                owner,
                name,
                visibility: row.try_get("visibility")?,
            })
        })
        .collect()
}

async fn package_permission_summaries(
    pool: &PgPool,
    package_id: Uuid,
) -> Result<Vec<PackagePermissionSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT pp.user_id,
               COALESCE(NULLIF(u.username, ''), split_part(u.email, '@', 1)) AS login,
               u.display_name,
               pp.role,
               pp.created_at AS granted_at
        FROM package_permissions pp
        JOIN users u ON u.id = pp.user_id
        WHERE pp.package_id = $1
        ORDER BY
            CASE pp.role WHEN 'admin' THEN 1 WHEN 'write' THEN 2 ELSE 3 END,
            lower(COALESCE(NULLIF(u.username, ''), split_part(u.email, '@', 1)))
        LIMIT 50
        "#,
    )
    .bind(package_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let login: String = row.try_get("login")?;
            Ok(PackagePermissionSummary {
                user_id: row.try_get("user_id")?,
                href: format!("/{}", url_component(&login)),
                login,
                display_name: row.try_get("display_name")?,
                role: row.try_get("role")?,
                granted_at: row.try_get("granted_at")?,
            })
        })
        .collect()
}

async fn package_repository_access_summaries(
    pool: &PgPool,
    package_id: Uuid,
) -> Result<Vec<PackageRepositoryAccessSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        WITH linked AS (
            SELECT p.repository_id
            FROM packages p
            WHERE p.id = $1
            UNION
            SELECT pr.repository_id
            FROM package_repository_links pr
            WHERE pr.package_id = $1
        )
        SELECT repo.id AS repository_id,
               COALESCE(owner_user.username, owner_org.slug) AS repository_owner,
               repo.name AS repository_name,
               repo.visibility AS repository_visibility,
               rp.user_id,
               COALESCE(NULLIF(u.username, ''), split_part(u.email, '@', 1)) AS login,
               rp.role,
               rp.source
        FROM linked
        JOIN repositories repo ON repo.id = linked.repository_id
        JOIN repository_permissions rp ON rp.repository_id = repo.id
        JOIN users u ON u.id = rp.user_id
        LEFT JOIN users owner_user ON owner_user.id = repo.owner_user_id
        LEFT JOIN organizations owner_org ON owner_org.id = repo.owner_organization_id
        ORDER BY lower(repo.name), CASE rp.role WHEN 'owner' THEN 1 WHEN 'admin' THEN 2 WHEN 'write' THEN 3 ELSE 4 END, lower(COALESCE(NULLIF(u.username, ''), split_part(u.email, '@', 1)))
        LIMIT 50
        "#,
    )
    .bind(package_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let owner: String = row.try_get("repository_owner")?;
            let name: String = row.try_get("repository_name")?;
            let login: String = row.try_get("login")?;
            Ok(PackageRepositoryAccessSummary {
                repository: OwnerPackageRepository {
                    id: row.try_get("repository_id")?,
                    full_name: format!("{owner}/{name}"),
                    href: format!("/{}/{}", url_component(&owner), url_component(&name)),
                    owner,
                    name,
                    visibility: row.try_get("repository_visibility")?,
                },
                user_id: row.try_get("user_id")?,
                href: format!("/{}", url_component(&login)),
                login,
                role: row.try_get("role")?,
                source: row.try_get("source")?,
            })
        })
        .collect()
}

async fn package_recent_activity(
    pool: &PgPool,
    package_id: Uuid,
) -> Result<Vec<PackageActivitySummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT 'version' AS kind,
               'Published ' || pv.version AS label,
               pv.created_at AS occurred_at,
               pv.published_by_user_id AS actor_id,
               COALESCE(NULLIF(u.username, ''), split_part(u.email, '@', 1)) AS actor_login,
               u.display_name AS actor_name
        FROM package_versions pv
        JOIN users u ON u.id = pv.published_by_user_id
        WHERE pv.package_id = $1
        UNION ALL
        SELECT 'download' AS kind,
               'Recorded ' || COALESCE(SUM(pd.download_count), 0)::text || ' downloads' AS label,
               MAX(pd.downloaded_at) AS occurred_at,
               NULL::uuid AS actor_id,
               NULL::text AS actor_login,
               NULL::text AS actor_name
        FROM package_downloads pd
        WHERE pd.package_id = $1
        GROUP BY pd.package_id
        ORDER BY occurred_at DESC
        LIMIT 8
        "#,
    )
    .bind(package_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let actor_id: Option<Uuid> = row.try_get("actor_id")?;
            let actor_login: Option<String> = row.try_get("actor_login")?;
            let actor_name: Option<String> = row.try_get("actor_name")?;
            Ok(PackageActivitySummary {
                kind: row.try_get("kind")?,
                label: row.try_get("label")?,
                occurred_at: row.try_get("occurred_at")?,
                actor: actor_id
                    .zip(actor_login)
                    .map(|(id, login)| OwnerPackagePublisher {
                        id,
                        href: format!("/{}", url_component(&login)),
                        login,
                        name: actor_name,
                    }),
            })
        })
        .collect()
}

fn package_registry_capabilities() -> Vec<PackageCapabilitySummary> {
    vec![
        PackageCapabilitySummary {
            key: "visibility".to_owned(),
            label: "Change package visibility".to_owned(),
            enabled: false,
            reason: "Visibility writes are reserved for the package registry management slice.".to_owned(),
        },
        PackageCapabilitySummary {
            key: "access".to_owned(),
            label: "Manage package access".to_owned(),
            enabled: false,
            reason: "Access mutation APIs are intentionally not enabled until package registry auth lands.".to_owned(),
        },
        PackageCapabilitySummary {
            key: "delete".to_owned(),
            label: "Delete or restore package versions".to_owned(),
            enabled: false,
            reason: "Deletion and restore require OCI registry audit semantics from packages-003.".to_owned(),
        },
    ]
}

fn install_commands(
    owner_login: &str,
    package_type: &str,
    package_name: &str,
    selected_version: Option<&PackageDetailVersion>,
    blobs: &[PackageBlobSummary],
) -> Vec<PackageInstallCommand> {
    let version = selected_version.map(|version| version.version.clone());
    let digest = selected_version.and_then(|version| version.digest.clone());
    let namespace = format!("{}/{}", owner_login, package_name);
    let base = match package_type {
        "container" => {
            let tag = version.as_deref().unwrap_or("latest");
            match digest.as_deref() {
                Some(digest) => format!("docker pull ghcr.io/{namespace}:{tag}@{digest}"),
                None => format!("docker pull ghcr.io/{namespace}:{tag}"),
            }
        }
        "npm" => format!("npm install @{}/{}", owner_login, package_name),
        "maven" => format!(
            "mvn dependency:get -Dartifact={owner_login}:{package_name}:{}",
            version.as_deref().unwrap_or("latest")
        ),
        "nuget" => format!(
            "dotnet add package {package_name} --version {}",
            version.as_deref().unwrap_or("latest")
        ),
        "rubygems" => format!(
            "gem install {package_name} -v {}",
            version.as_deref().unwrap_or("latest")
        ),
        _ => format!(
            "curl -O https://packages.opengithub.local/{namespace}/{}",
            version.as_deref().unwrap_or("latest")
        ),
    };
    let mut commands = vec![PackageInstallCommand {
        label: "Default".to_owned(),
        command: base,
        version: version.clone(),
        digest: digest.clone(),
        platform: None,
    }];
    for blob in blobs.iter().take(4) {
        let platform = match (&blob.platform_os, &blob.platform_arch) {
            (Some(os), Some(arch)) => Some(format!("{os}/{arch}")),
            (Some(os), None) => Some(os.clone()),
            (None, Some(arch)) => Some(arch.clone()),
            _ => None,
        };
        if package_type == "container" {
            commands.push(PackageInstallCommand {
                label: platform.clone().unwrap_or_else(|| "Blob digest".to_owned()),
                command: format!("docker pull ghcr.io/{namespace}@{}", blob.digest),
                version: version.clone(),
                digest: Some(blob.digest.clone()),
                platform,
            });
        }
    }
    commands
}

async fn package_about(
    pool: &PgPool,
    package_id: Uuid,
    linked_repository_id: Option<Uuid>,
) -> Result<PackageAboutContent, PackageDetailError> {
    if let Some(markdown) = sqlx::query_scalar::<_, String>(
        "SELECT markdown FROM package_about_overrides WHERE package_id = $1",
    )
    .bind(package_id)
    .fetch_optional(pool)
    .await?
    {
        return render_about(pool, "package", markdown, None).await;
    }
    if let Some(repository_id) = linked_repository_id {
        let markdown = sqlx::query_scalar::<_, String>(
            r#"
            SELECT rf.content
            FROM repository_files rf
            JOIN repositories r ON r.id = rf.repository_id
            LEFT JOIN repository_git_refs ref
              ON ref.repository_id = r.id
             AND ref.name = 'refs/heads/' || r.default_branch
            WHERE rf.repository_id = $1
              AND lower(rf.path) = 'readme.md'
              AND (ref.target_commit_id IS NULL OR rf.commit_id = ref.target_commit_id)
            ORDER BY rf.created_at DESC
            LIMIT 1
            "#,
        )
        .bind(repository_id)
        .fetch_optional(pool)
        .await?;
        if let Some(markdown) = markdown {
            return render_about(
                pool,
                "linked_repository_readme",
                markdown,
                Some(repository_id),
            )
            .await;
        }
    }
    Ok(PackageAboutContent {
        source: "empty".to_owned(),
        markdown: None,
        html: None,
        empty: true,
    })
}

async fn render_about(
    pool: &PgPool,
    source: &str,
    markdown: String,
    repository_id: Option<Uuid>,
) -> Result<PackageAboutContent, PackageDetailError> {
    let rendered = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            markdown: markdown.clone(),
            repository_id,
            ref_name: None,
            owner: None,
            repo: None,
            enable_task_toggles: Some(false),
        },
    )
    .await?;
    Ok(PackageAboutContent {
        source: source.to_owned(),
        markdown: Some(markdown),
        html: Some(rendered.html),
        empty: false,
    })
}

fn short_digest(digest: &str) -> String {
    digest
        .strip_prefix("sha256:")
        .unwrap_or(digest)
        .chars()
        .take(12)
        .collect()
}

fn package_type_label(package_type: &str) -> &'static str {
    match package_type {
        "container" => "Container",
        "npm" => "npm",
        "rubygems" => "RubyGems",
        "maven" => "Maven",
        "nuget" => "NuGet",
        "generic" => "Generic",
        _ => "Package",
    }
}

fn owner_href(owner_kind: PackageOwnerKind, login: &str) -> String {
    match owner_kind {
        PackageOwnerKind::User => format!("/{}", url_component(login)),
        PackageOwnerKind::Organization => format!("/orgs/{}", url_component(login)),
    }
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn package_href(
    owner_kind: PackageOwnerKind,
    owner_login: &str,
    package_type: &str,
    name: &str,
) -> String {
    match owner_kind {
        PackageOwnerKind::User => format!(
            "/{}/{}/{}",
            url_component(owner_login),
            url_component(package_type),
            url_component(name)
        ),
        PackageOwnerKind::Organization => format!(
            "/orgs/{}/packages/{}/{}",
            url_component(owner_login),
            url_component(package_type),
            url_component(name)
        ),
    }
}

fn url_component(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}
