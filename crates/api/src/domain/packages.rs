use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, QueryBuilder, Row};
use uuid::Uuid;

use crate::api_types::{normalize_pagination, ListEnvelope};

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

#[derive(Debug, thiserror::Error)]
pub enum PackageListError {
    #[error("package owner was not found")]
    NotFound,
    #[error("{0}")]
    InvalidFilter(String),
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
