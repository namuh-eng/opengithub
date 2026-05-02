use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PublicOrganizationProfile {
    pub identity: OrganizationIdentity,
    pub verified_domains: Vec<OrganizationVerifiedDomain>,
    pub pinned_repositories: Vec<OrganizationRepositoryPreview>,
    pub repository_preview: Vec<OrganizationRepositoryPreview>,
    pub people_preview: Vec<OrganizationPersonPreview>,
    pub top_languages: Vec<OrganizationLanguageSummary>,
    pub top_topics: Vec<OrganizationTopicSummary>,
    pub sponsorship: OrganizationSponsorshipState,
    pub tab_counts: OrganizationTabCounts,
    pub viewer_state: OrganizationViewerState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationIdentity {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub website_url: Option<String>,
    pub location: Option<String>,
    pub html_url: String,
    pub profile_visibility: String,
    pub is_private: bool,
    pub follower_count: i64,
    pub public_member_count: i64,
    pub repository_count: i64,
    pub created_at: DateTime<Utc>,
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
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub visibility: String,
    pub href: String,
    pub default_branch: String,
    pub primary_language: Option<OrganizationLanguageSummary>,
    pub languages: Vec<OrganizationLanguageSummary>,
    pub topics: Vec<String>,
    pub stars_count: i64,
    pub forks_count: i64,
    pub open_issues_count: i64,
    pub open_pull_requests_count: i64,
    pub is_archived: bool,
    pub is_template: bool,
    pub is_mirror: bool,
    pub license: Option<OrganizationRepositoryLicense>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationRepositoryLicense {
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationPersonPreview {
    pub id: Uuid,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub href: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationLanguageSummary {
    pub language: String,
    pub color: String,
    pub byte_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationTopicSummary {
    pub topic: String,
    pub count: i64,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationSponsorshipState {
    pub enabled: bool,
    pub sponsor_count: i64,
    pub href: Option<String>,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationTabCounts {
    pub repositories: i64,
    pub projects: i64,
    pub packages: i64,
    pub people: i64,
    pub sponsoring: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationViewerState {
    pub authenticated: bool,
    pub is_member: bool,
    pub role: Option<String>,
    pub can_view_internal: bool,
    pub can_admin: bool,
    pub is_following: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum OrganizationProfileError {
    #[error("organization profile was not found")]
    NotFound,
    #[error("database error")]
    Sqlx(#[from] sqlx::Error),
}

struct OrganizationRow {
    id: Uuid,
    slug: String,
    display_name: String,
    description: Option<String>,
    avatar_url: Option<String>,
    website_url: Option<String>,
    location: Option<String>,
    profile_visibility: String,
    public_members_visible: bool,
    created_at: DateTime<Utc>,
}

pub async fn public_organization_profile(
    pool: &PgPool,
    slug: &str,
    viewer_user_id: Option<Uuid>,
) -> Result<PublicOrganizationProfile, OrganizationProfileError> {
    let organization = organization_by_slug(pool, slug).await?;
    let viewer_role = viewer_role(pool, organization.id, viewer_user_id).await?;
    let is_member = viewer_role.is_some();

    if organization.profile_visibility == "private" && !is_member {
        return Err(OrganizationProfileError::NotFound);
    }

    let viewer_state = OrganizationViewerState {
        authenticated: viewer_user_id.is_some(),
        is_member,
        role: viewer_role.clone(),
        can_view_internal: is_member,
        can_admin: matches!(viewer_role.as_deref(), Some("owner" | "admin")),
        is_following: is_following(pool, organization.id, viewer_user_id).await?,
    };
    let visible_repository_ids =
        visible_repository_ids(pool, organization.id, viewer_user_id, is_member).await?;
    let repository_count = visible_repository_ids.len() as i64;
    let public_member_count = public_member_count(pool, &organization).await?;
    let follower_count = follower_count(pool, organization.id).await?;
    let pinned_repositories = pinned_repositories(
        pool,
        organization.id,
        &organization.slug,
        &visible_repository_ids,
    )
    .await?;
    let repository_preview = repository_preview(
        pool,
        organization.id,
        &organization.slug,
        &visible_repository_ids,
    )
    .await?;
    let people_preview = people_preview(pool, &organization, is_member).await?;
    let top_languages = top_languages(pool, &visible_repository_ids).await?;
    let top_topics = top_topics(pool, &visible_repository_ids, &organization.slug).await?;
    let packages = packages_count(pool, organization.id, is_member).await?;

    Ok(PublicOrganizationProfile {
        identity: OrganizationIdentity {
            id: organization.id,
            slug: organization.slug.clone(),
            name: organization.display_name,
            description: organization.description,
            avatar_url: organization.avatar_url,
            website_url: organization.website_url,
            location: organization.location,
            html_url: format!("/orgs/{}", organization.slug),
            profile_visibility: organization.profile_visibility.clone(),
            is_private: organization.profile_visibility == "private",
            follower_count,
            public_member_count,
            repository_count,
            created_at: organization.created_at,
        },
        verified_domains: verified_domains(pool, organization.id).await?,
        pinned_repositories,
        repository_preview,
        people_preview,
        top_languages,
        top_topics,
        sponsorship: OrganizationSponsorshipState {
            enabled: false,
            sponsor_count: 0,
            href: None,
            unavailable_reason: Some(
                "Sponsorships are not available in opengithub MVP.".to_owned(),
            ),
        },
        tab_counts: OrganizationTabCounts {
            repositories: repository_count,
            projects: 0,
            packages,
            people: public_member_count,
            sponsoring: 0,
        },
        viewer_state,
    })
}

async fn organization_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<OrganizationRow, OrganizationProfileError> {
    let row = sqlx::query(
        r#"
        SELECT id, slug, display_name, description, avatar_url, website_url, location,
               profile_visibility, public_members_visible, created_at
        FROM organizations
        WHERE lower(slug) = lower($1)
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?
    .ok_or(OrganizationProfileError::NotFound)?;

    Ok(OrganizationRow {
        id: row.get("id"),
        slug: row.get("slug"),
        display_name: row.get("display_name"),
        description: row.get("description"),
        avatar_url: row.get("avatar_url"),
        website_url: row.get("website_url"),
        location: row.get("location"),
        profile_visibility: row.get("profile_visibility"),
        public_members_visible: row.get("public_members_visible"),
        created_at: row.get("created_at"),
    })
}

async fn viewer_role(
    pool: &PgPool,
    organization_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<Option<String>, sqlx::Error> {
    let Some(viewer_user_id) = viewer_user_id else {
        return Ok(None);
    };
    sqlx::query_scalar(
        r#"
        SELECT role
        FROM organization_memberships
        WHERE organization_id = $1 AND user_id = $2
        "#,
    )
    .bind(organization_id)
    .bind(viewer_user_id)
    .fetch_optional(pool)
    .await
}

async fn is_following(
    pool: &PgPool,
    organization_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<bool, sqlx::Error> {
    let Some(viewer_user_id) = viewer_user_id else {
        return Ok(false);
    };
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM organization_follows
            WHERE organization_id = $1 AND user_id = $2
        )
        "#,
    )
    .bind(organization_id)
    .bind(viewer_user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

async fn visible_repository_ids(
    pool: &PgPool,
    organization_id: Uuid,
    viewer_user_id: Option<Uuid>,
    is_member: bool,
) -> Result<Vec<Uuid>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id
        FROM repositories
        WHERE owner_organization_id = $1
          AND (
            visibility = 'public'
            OR $3
            OR EXISTS (
                SELECT 1
                FROM repository_permissions
                WHERE repository_permissions.repository_id = repositories.id
                  AND repository_permissions.user_id = $2
                  AND repository_permissions.role IN ('owner', 'admin', 'write', 'read')
            )
          )
        ORDER BY updated_at DESC, lower(name) ASC
        "#,
    )
    .bind(organization_id)
    .bind(viewer_user_id)
    .bind(is_member)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|row| row.get("id")).collect())
}

async fn pinned_repositories(
    pool: &PgPool,
    organization_id: Uuid,
    owner_slug: &str,
    visible_repository_ids: &[Uuid],
) -> Result<Vec<OrganizationRepositoryPreview>, sqlx::Error> {
    if visible_repository_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT repositories.id,
               repositories.name,
               repositories.description,
               repositories.visibility,
               repositories.default_branch,
               repositories.is_archived,
               repositories.is_template,
               repositories.is_mirror,
               repositories.license_template_slug,
               license_templates.display_name AS license_name,
               repositories.updated_at,
               COALESCE(star_counts.total, 0)::bigint AS stars_count,
               COALESCE(fork_counts.total, 0)::bigint AS forks_count,
               COALESCE(issue_counts.total, 0)::bigint AS open_issues_count,
               COALESCE(pr_counts.total, 0)::bigint AS open_pull_requests_count
        FROM organization_profile_pins
        JOIN repositories ON repositories.id = organization_profile_pins.repository_id
        LEFT JOIN license_templates ON license_templates.slug = repositories.license_template_slug
        LEFT JOIN (
            SELECT repository_id, COUNT(*) AS total FROM repository_stars GROUP BY repository_id
        ) star_counts ON star_counts.repository_id = repositories.id
        LEFT JOIN (
            SELECT source_repository_id AS repository_id, COUNT(*) AS total FROM repository_forks GROUP BY source_repository_id
        ) fork_counts ON fork_counts.repository_id = repositories.id
        LEFT JOIN (
            SELECT repository_id, COUNT(*) AS total FROM issues WHERE state = 'open' GROUP BY repository_id
        ) issue_counts ON issue_counts.repository_id = repositories.id
        LEFT JOIN (
            SELECT repository_id, COUNT(*) AS total FROM pull_requests WHERE state <> 'closed' GROUP BY repository_id
        ) pr_counts ON pr_counts.repository_id = repositories.id
        WHERE organization_profile_pins.organization_id = $1
          AND repositories.id = ANY($2)
        ORDER BY organization_profile_pins.position ASC, lower(repositories.name) ASC
        LIMIT 6
        "#,
    )
    .bind(organization_id)
    .bind(visible_repository_ids)
    .fetch_all(pool)
    .await?;
    repository_previews_from_rows(pool, owner_slug, rows).await
}

async fn repository_preview(
    pool: &PgPool,
    organization_id: Uuid,
    owner_slug: &str,
    visible_repository_ids: &[Uuid],
) -> Result<Vec<OrganizationRepositoryPreview>, sqlx::Error> {
    if visible_repository_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT repositories.id,
               repositories.name,
               repositories.description,
               repositories.visibility,
               repositories.default_branch,
               repositories.is_archived,
               repositories.is_template,
               repositories.is_mirror,
               repositories.license_template_slug,
               license_templates.display_name AS license_name,
               repositories.updated_at,
               COALESCE(star_counts.total, 0)::bigint AS stars_count,
               COALESCE(fork_counts.total, 0)::bigint AS forks_count,
               COALESCE(issue_counts.total, 0)::bigint AS open_issues_count,
               COALESCE(pr_counts.total, 0)::bigint AS open_pull_requests_count
        FROM repositories
        LEFT JOIN license_templates ON license_templates.slug = repositories.license_template_slug
        LEFT JOIN (
            SELECT repository_id, COUNT(*) AS total FROM repository_stars GROUP BY repository_id
        ) star_counts ON star_counts.repository_id = repositories.id
        LEFT JOIN (
            SELECT source_repository_id AS repository_id, COUNT(*) AS total FROM repository_forks GROUP BY source_repository_id
        ) fork_counts ON fork_counts.repository_id = repositories.id
        LEFT JOIN (
            SELECT repository_id, COUNT(*) AS total FROM issues WHERE state = 'open' GROUP BY repository_id
        ) issue_counts ON issue_counts.repository_id = repositories.id
        LEFT JOIN (
            SELECT repository_id, COUNT(*) AS total FROM pull_requests WHERE state <> 'closed' GROUP BY repository_id
        ) pr_counts ON pr_counts.repository_id = repositories.id
        WHERE repositories.owner_organization_id = $1
          AND repositories.id = ANY($2)
        ORDER BY repositories.updated_at DESC, lower(repositories.name) ASC
        LIMIT 8
        "#,
    )
    .bind(organization_id)
    .bind(visible_repository_ids)
    .fetch_all(pool)
    .await?;
    repository_previews_from_rows(pool, owner_slug, rows).await
}

async fn repository_previews_from_rows(
    pool: &PgPool,
    owner_slug: &str,
    rows: Vec<sqlx::postgres::PgRow>,
) -> Result<Vec<OrganizationRepositoryPreview>, sqlx::Error> {
    let mut repositories = Vec::with_capacity(rows.len());
    for row in rows {
        let repository_id = row.get("id");
        let name: String = row.get("name");
        let languages = repository_languages(pool, repository_id).await?;
        let topics = repository_topics(pool, repository_id).await?;
        let license_slug = row.try_get::<Option<String>, _>("license_template_slug")?;
        let license = license_slug.map(|slug| OrganizationRepositoryLicense {
            slug,
            name: row
                .try_get::<Option<String>, _>("license_name")
                .ok()
                .flatten()
                .unwrap_or_else(|| "License".to_owned()),
        });
        repositories.push(OrganizationRepositoryPreview {
            id: repository_id,
            owner: owner_slug.to_owned(),
            name: name.clone(),
            full_name: format!("{owner_slug}/{name}"),
            description: row.get("description"),
            visibility: row.get("visibility"),
            href: format!("/{owner_slug}/{name}"),
            default_branch: row.get("default_branch"),
            primary_language: languages.first().cloned(),
            languages,
            topics,
            stars_count: row.get("stars_count"),
            forks_count: row.get("forks_count"),
            open_issues_count: row.get("open_issues_count"),
            open_pull_requests_count: row.get("open_pull_requests_count"),
            is_archived: row.get("is_archived"),
            is_template: row.get("is_template"),
            is_mirror: row.get("is_mirror"),
            license,
            updated_at: row.get("updated_at"),
        });
    }
    Ok(repositories)
}

async fn repository_languages(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<OrganizationLanguageSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT language, color, byte_count
        FROM repository_languages
        WHERE repository_id = $1
        ORDER BY byte_count DESC, language ASC
        LIMIT 5
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| OrganizationLanguageSummary {
            language: row.get("language"),
            color: row.get("color"),
            byte_count: row.get("byte_count"),
        })
        .collect())
}

async fn repository_topics(pool: &PgPool, repository_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT topic
        FROM repository_topics
        WHERE repository_id = $1
        ORDER BY lower(topic) ASC
        LIMIT 8
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|row| row.get("topic")).collect())
}

async fn verified_domains(
    pool: &PgPool,
    organization_id: Uuid,
) -> Result<Vec<OrganizationVerifiedDomain>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT domain, verified_at
        FROM organization_verified_domains
        WHERE organization_id = $1
        ORDER BY verified_at DESC, lower(domain) ASC
        "#,
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let domain: String = row.get("domain");
            OrganizationVerifiedDomain {
                href: format!("https://{domain}"),
                domain,
                verified_at: row.get("verified_at"),
            }
        })
        .collect())
}

async fn people_preview(
    pool: &PgPool,
    organization: &OrganizationRow,
    is_member: bool,
) -> Result<Vec<OrganizationPersonPreview>, sqlx::Error> {
    if !is_member && !organization.public_members_visible {
        return Ok(Vec::new());
    }
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
        ORDER BY
            CASE organization_memberships.role
                WHEN 'owner' THEN 0
                WHEN 'admin' THEN 1
                ELSE 2
            END ASC,
            lower(COALESCE(NULLIF(users.username, ''), users.email)) ASC
        LIMIT 12
        "#,
    )
    .bind(organization.id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let login: String = row.get("login");
            OrganizationPersonPreview {
                id: row.get("id"),
                login: login.clone(),
                name: row.get("display_name"),
                avatar_url: row.get("avatar_url"),
                href: format!("/{login}"),
                role: if is_member {
                    Some(row.get("role"))
                } else {
                    None
                },
            }
        })
        .collect())
}

async fn top_languages(
    pool: &PgPool,
    visible_repository_ids: &[Uuid],
) -> Result<Vec<OrganizationLanguageSummary>, sqlx::Error> {
    if visible_repository_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT language, MIN(color) AS color, SUM(byte_count)::bigint AS byte_count
        FROM repository_languages
        WHERE repository_id = ANY($1)
        GROUP BY language
        ORDER BY SUM(byte_count) DESC, language ASC
        LIMIT 8
        "#,
    )
    .bind(visible_repository_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| OrganizationLanguageSummary {
            language: row.get("language"),
            color: row.get("color"),
            byte_count: row.get("byte_count"),
        })
        .collect())
}

async fn top_topics(
    pool: &PgPool,
    visible_repository_ids: &[Uuid],
    owner_slug: &str,
) -> Result<Vec<OrganizationTopicSummary>, sqlx::Error> {
    if visible_repository_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT topic, COUNT(*)::bigint AS total
        FROM repository_topics
        WHERE repository_id = ANY($1)
        GROUP BY topic
        ORDER BY COUNT(*) DESC, lower(topic) ASC
        LIMIT 12
        "#,
    )
    .bind(visible_repository_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let topic: String = row.get("topic");
            OrganizationTopicSummary {
                href: format!("/orgs/{owner_slug}/repositories?q=topic%3A{topic}"),
                topic,
                count: row.get("total"),
            }
        })
        .collect())
}

async fn public_member_count(
    pool: &PgPool,
    organization: &OrganizationRow,
) -> Result<i64, sqlx::Error> {
    if !organization.public_members_visible {
        return Ok(0);
    }
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM organization_memberships
        WHERE organization_id = $1
        "#,
    )
    .bind(organization.id)
    .fetch_one(pool)
    .await
}

async fn follower_count(pool: &PgPool, organization_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM organization_follows
        WHERE organization_id = $1
        "#,
    )
    .bind(organization_id)
    .fetch_one(pool)
    .await
}

async fn packages_count(
    pool: &PgPool,
    organization_id: Uuid,
    is_member: bool,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM packages
        WHERE owner_organization_id = $1
          AND (visibility = 'public' OR $2)
        "#,
    )
    .bind(organization_id)
    .bind(is_member)
    .fetch_one(pool)
    .await
}
