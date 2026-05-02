use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::repositories::{RepositoryError, RepositoryLanguageSummary, RepositoryVisibility};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationOverview {
    pub id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub website_url: Option<String>,
    pub location: Option<String>,
    pub verified_domain: Option<OrganizationVerifiedDomain>,
    pub viewer_role: Option<String>,
    pub viewer_can_admin: bool,
    pub follower_count: i64,
    pub member_count: i64,
    pub repository_count: i64,
    pub pinned_repositories: Vec<OrganizationRepositoryPreview>,
    pub repositories: Vec<OrganizationRepositoryPreview>,
    pub members: Vec<OrganizationMemberPreview>,
    pub languages: Vec<RepositoryLanguageSummary>,
    pub topics: Vec<OrganizationTopicSummary>,
    pub sponsorship: OrganizationSponsorshipSummary,
    pub projects_href: String,
    pub settings_href: Option<String>,
    pub people_href: String,
    pub repositories_href: String,
    pub packages_href: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationVerifiedDomain {
    pub domain: String,
    pub verified_at: DateTime<Utc>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationRepositoryPreview {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub visibility: RepositoryVisibility,
    pub href: String,
    pub primary_language: Option<RepositoryLanguageSummary>,
    pub topics: Vec<String>,
    pub stars_count: i64,
    pub forks_count: i64,
    pub open_issues_count: i64,
    pub open_pull_requests_count: i64,
    pub updated_at: DateTime<Utc>,
    pub is_pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationMemberPreview {
    pub id: Uuid,
    pub login: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationTopicSummary {
    pub topic: String,
    pub repository_count: i64,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationSponsorshipSummary {
    pub enabled: bool,
    pub sponsor_href: Option<String>,
    pub note: String,
}

pub async fn organization_overview_for_viewer(
    pool: &PgPool,
    slug: &str,
    actor_user_id: Option<Uuid>,
) -> Result<Option<OrganizationOverview>, RepositoryError> {
    let Some(org_row) = sqlx::query(
        r#"
        SELECT id, slug, display_name, description, updated_at
        FROM organizations
        WHERE lower(slug) = lower($1)
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let org_id: Uuid = org_row.get("id");
    let org_slug: String = org_row.get("slug");
    let viewer_role = match actor_user_id {
        Some(user_id) => sqlx::query_scalar::<_, Option<String>>(
            r#"
            SELECT role
            FROM organization_memberships
            WHERE organization_id = $1 AND user_id = $2
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .flatten(),
        None => None,
    };
    let viewer_can_admin = matches!(viewer_role.as_deref(), Some("owner" | "admin"));

    let verified_domain = verified_domain(pool, org_id, &org_slug).await?;
    let follower_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM organization_follows WHERE organization_id = $1",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await?;
    let member_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM organization_memberships WHERE organization_id = $1",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await?;
    let repository_count =
        visible_repository_count(pool, org_id, actor_user_id, viewer_role.as_deref()).await?;
    let pinned_repositories = organization_repositories(
        pool,
        org_id,
        &org_slug,
        actor_user_id,
        viewer_role.as_deref(),
        true,
        6,
    )
    .await?;
    let repositories = organization_repositories(
        pool,
        org_id,
        &org_slug,
        actor_user_id,
        viewer_role.as_deref(),
        false,
        8,
    )
    .await?;
    let members = organization_members(pool, org_id, 8).await?;
    let languages =
        organization_languages(pool, org_id, actor_user_id, viewer_role.as_deref()).await?;
    let topics = organization_topics(
        pool,
        org_id,
        &org_slug,
        actor_user_id,
        viewer_role.as_deref(),
    )
    .await?;

    Ok(Some(OrganizationOverview {
        id: org_id,
        slug: org_slug.clone(),
        display_name: org_row.get("display_name"),
        description: org_row.get("description"),
        avatar_url: None,
        website_url: verified_domain
            .as_ref()
            .map(|domain| format!("https://{}", domain.domain)),
        location: None,
        verified_domain,
        viewer_role,
        viewer_can_admin,
        follower_count,
        member_count,
        repository_count,
        pinned_repositories,
        repositories,
        members,
        languages,
        topics,
        sponsorship: OrganizationSponsorshipSummary {
            enabled: false,
            sponsor_href: None,
            note: "Sponsorships are not enabled in this OpenGitHub MVP.".to_owned(),
        },
        projects_href: format!("/orgs/{org_slug}/projects"),
        settings_href: viewer_can_admin.then(|| format!("/orgs/{org_slug}/settings")),
        people_href: format!("/orgs/{org_slug}?tab=people"),
        repositories_href: format!("/orgs/{org_slug}?tab=repositories"),
        packages_href: format!("/orgs/{org_slug}?tab=packages"),
        updated_at: org_row.get("updated_at"),
    }))
}

async fn verified_domain(
    pool: &PgPool,
    org_id: Uuid,
    org_slug: &str,
) -> Result<Option<OrganizationVerifiedDomain>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT domain, verified_at
        FROM organization_verified_domains
        WHERE organization_id = $1
        ORDER BY verified_at DESC, domain ASC
        LIMIT 1
        "#,
    )
    .bind(org_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| OrganizationVerifiedDomain {
        domain: row.get("domain"),
        verified_at: row.get("verified_at"),
        href: format!("/orgs/{org_slug}/settings/verified-domains"),
    }))
}

async fn visible_repository_count(
    pool: &PgPool,
    org_id: Uuid,
    actor_user_id: Option<Uuid>,
    viewer_role: Option<&str>,
) -> Result<i64, RepositoryError> {
    let can_see_internal = viewer_role.is_some();
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(DISTINCT repositories.id)
        FROM repositories
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = repositories.id
         AND repository_permissions.user_id = $2
        WHERE repositories.owner_organization_id = $1
          AND (
              repositories.visibility = 'public'
              OR ($3 AND repositories.visibility = 'internal')
              OR repository_permissions.user_id IS NOT NULL
          )
        "#,
    )
    .bind(org_id)
    .bind(actor_user_id)
    .bind(can_see_internal)
    .fetch_one(pool)
    .await
    .map_err(RepositoryError::from)
}

async fn organization_repositories(
    pool: &PgPool,
    org_id: Uuid,
    org_slug: &str,
    actor_user_id: Option<Uuid>,
    viewer_role: Option<&str>,
    pinned_only: bool,
    limit: i64,
) -> Result<Vec<OrganizationRepositoryPreview>, RepositoryError> {
    let can_see_internal = viewer_role.is_some();
    let rows = sqlx::query(
        r#"
        SELECT DISTINCT repositories.id,
               repositories.name,
               repositories.description,
               repositories.visibility,
               repositories.updated_at,
               (profile_pins.repository_id IS NOT NULL) AS is_pinned
        FROM repositories
        LEFT JOIN profile_pins
          ON profile_pins.repository_id = repositories.id
         AND profile_pins.owner_organization_id = $1
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = repositories.id
         AND repository_permissions.user_id = $2
        WHERE repositories.owner_organization_id = $1
          AND ($4 = false OR profile_pins.repository_id IS NOT NULL)
          AND (
              repositories.visibility = 'public'
              OR ($3 AND repositories.visibility = 'internal')
              OR repository_permissions.user_id IS NOT NULL
          )
        ORDER BY is_pinned DESC,
                 repositories.updated_at DESC,
                 repositories.name ASC
        LIMIT $5
        "#,
    )
    .bind(org_id)
    .bind(actor_user_id)
    .bind(can_see_internal)
    .bind(pinned_only)
    .bind(limit.clamp(1, 12))
    .fetch_all(pool)
    .await?;

    let mut previews = Vec::with_capacity(rows.len());
    for row in rows {
        let repository_id: Uuid = row.get("id");
        previews.push(repository_preview_from_row(pool, row, repository_id, org_slug).await?);
    }
    Ok(previews)
}

async fn repository_preview_from_row(
    pool: &PgPool,
    row: sqlx::postgres::PgRow,
    repository_id: Uuid,
    org_slug: &str,
) -> Result<OrganizationRepositoryPreview, RepositoryError> {
    let language = primary_language(pool, repository_id).await?;
    let topics = repository_topics(pool, repository_id).await?;
    let stars_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_stars WHERE repository_id = $1",
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    let forks_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_forks WHERE source_repository_id = $1",
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    let open_issues_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM issues
        WHERE repository_id = $1
          AND state = 'open'
          AND NOT EXISTS (
              SELECT 1 FROM pull_requests WHERE pull_requests.issue_id = issues.id
          )
        "#,
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    let open_pull_requests_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM pull_requests WHERE repository_id = $1 AND state = 'open'",
    )
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    let visibility: String = row.get("visibility");
    let name: String = row.get("name");

    Ok(OrganizationRepositoryPreview {
        id: repository_id,
        name: name.clone(),
        description: row.get("description"),
        visibility: RepositoryVisibility::try_from(visibility.as_str())?,
        href: format!("/{org_slug}/{name}"),
        primary_language: language,
        topics,
        stars_count,
        forks_count,
        open_issues_count,
        open_pull_requests_count,
        updated_at: row.get("updated_at"),
        is_pinned: row.get("is_pinned"),
    })
}

async fn primary_language(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Option<RepositoryLanguageSummary>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT language, color, byte_count,
               CASE WHEN sum(byte_count) OVER () > 0
                    THEN (byte_count * 100 / sum(byte_count) OVER ())::bigint
                    ELSE 100::bigint
               END AS percentage
        FROM repository_languages
        WHERE repository_id = $1
        ORDER BY byte_count DESC, language ASC
        LIMIT 1
        "#,
    )
    .bind(repository_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| RepositoryLanguageSummary {
        language: row.get("language"),
        color: row.get("color"),
        byte_count: row.get("byte_count"),
        percentage: row.get("percentage"),
    }))
}

async fn repository_topics(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<String>, RepositoryError> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT topic
        FROM repository_topics
        WHERE repository_id = $1
        ORDER BY topic ASC
        LIMIT 6
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

async fn organization_members(
    pool: &PgPool,
    org_id: Uuid,
    limit: i64,
) -> Result<Vec<OrganizationMemberPreview>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.display_name,
               users.avatar_url,
               organization_memberships.role
        FROM organization_memberships
        JOIN users ON users.id = organization_memberships.user_id
        WHERE organization_memberships.organization_id = $1
        ORDER BY CASE organization_memberships.role
                   WHEN 'owner' THEN 0
                   WHEN 'admin' THEN 1
                   ELSE 2
                 END,
                 login ASC
        LIMIT $2
        "#,
    )
    .bind(org_id)
    .bind(limit.clamp(1, 24))
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let login: String = row.get("login");
            OrganizationMemberPreview {
                id: row.get("id"),
                login: login.clone(),
                display_name: row.get("display_name"),
                avatar_url: row.get("avatar_url"),
                role: row.get("role"),
                href: format!("/{login}"),
            }
        })
        .collect())
}

async fn organization_languages(
    pool: &PgPool,
    org_id: Uuid,
    actor_user_id: Option<Uuid>,
    viewer_role: Option<&str>,
) -> Result<Vec<RepositoryLanguageSummary>, RepositoryError> {
    let can_see_internal = viewer_role.is_some();
    let rows = sqlx::query(
        r#"
        WITH visible_repositories AS (
          SELECT DISTINCT repositories.id
          FROM repositories
          LEFT JOIN repository_permissions
            ON repository_permissions.repository_id = repositories.id
           AND repository_permissions.user_id = $2
          WHERE repositories.owner_organization_id = $1
            AND (
                repositories.visibility = 'public'
                OR ($3 AND repositories.visibility = 'internal')
                OR repository_permissions.user_id IS NOT NULL
            )
        ), language_totals AS (
          SELECT repository_languages.language,
                 min(repository_languages.color) AS color,
                 sum(repository_languages.byte_count)::bigint AS byte_count
          FROM repository_languages
          JOIN visible_repositories ON visible_repositories.id = repository_languages.repository_id
          GROUP BY repository_languages.language
        )
        SELECT language, color, byte_count,
               CASE WHEN sum(byte_count) OVER () > 0
                    THEN (byte_count * 100 / sum(byte_count) OVER ())::bigint
                    ELSE 0::bigint
               END AS percentage
        FROM language_totals
        ORDER BY byte_count DESC, language ASC
        LIMIT 6
        "#,
    )
    .bind(org_id)
    .bind(actor_user_id)
    .bind(can_see_internal)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| RepositoryLanguageSummary {
            language: row.get("language"),
            color: row.get("color"),
            byte_count: row.get("byte_count"),
            percentage: row.get("percentage"),
        })
        .collect())
}

async fn organization_topics(
    pool: &PgPool,
    org_id: Uuid,
    org_slug: &str,
    actor_user_id: Option<Uuid>,
    viewer_role: Option<&str>,
) -> Result<Vec<OrganizationTopicSummary>, RepositoryError> {
    let can_see_internal = viewer_role.is_some();
    let rows = sqlx::query(
        r#"
        SELECT repository_topics.topic,
               count(DISTINCT repositories.id)::bigint AS repository_count
        FROM repository_topics
        JOIN repositories ON repositories.id = repository_topics.repository_id
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = repositories.id
         AND repository_permissions.user_id = $2
        WHERE repositories.owner_organization_id = $1
          AND (
              repositories.visibility = 'public'
              OR ($3 AND repositories.visibility = 'internal')
              OR repository_permissions.user_id IS NOT NULL
          )
        GROUP BY repository_topics.topic
        ORDER BY repository_count DESC, repository_topics.topic ASC
        LIMIT 12
        "#,
    )
    .bind(org_id)
    .bind(actor_user_id)
    .bind(can_see_internal)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let topic: String = row.get("topic");
            OrganizationTopicSummary {
                href: format!(
                    "/orgs/{org_slug}?tab=repositories&q=topic%3A{}",
                    url::form_urlencoded::byte_serialize(topic.as_bytes()).collect::<String>()
                ),
                topic,
                repository_count: row.get("repository_count"),
            }
        })
        .collect())
}
