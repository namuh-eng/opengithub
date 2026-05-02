use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api_types::{normalize_pagination, ListEnvelope};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationRepositoryList {
    #[serde(flatten)]
    pub envelope: ListEnvelope<OrganizationRepositoryListItem>,
    pub mode: String,
    pub filters: OrganizationRepositoryFilters,
    pub available_languages: Vec<OrganizationRepositoryFilterOption>,
    pub available_types: Vec<OrganizationRepositoryFilterOption>,
    pub tab_counts: OrganizationTabCounts,
    pub viewer_state: OrganizationViewerState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationRepositoryListItem {
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
    pub license: Option<OrganizationRepositoryLicense>,
    pub is_archived: bool,
    pub is_fork: bool,
    pub is_template: bool,
    pub is_mirror: bool,
    pub can_admin: bool,
    pub contributed_by_viewer: bool,
    pub fork_source: Option<OrganizationRepositoryForkSource>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationRepositoryForkSource {
    pub owner: String,
    pub name: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationRepositoryFilters {
    pub query: Option<String>,
    pub repository_type: String,
    pub language: Option<String>,
    pub sort: String,
    pub density: String,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationRepositoryFilterOption {
    pub value: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct OrganizationRepositoryListQuery<'a> {
    pub query: Option<&'a str>,
    pub repository_type: Option<&'a str>,
    pub language: Option<&'a str>,
    pub sort: Option<&'a str>,
    pub density: Option<&'a str>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, thiserror::Error)]
pub enum OrganizationProfileError {
    #[error("organization profile was not found")]
    NotFound,
    #[error("invalid organization repository filter: {0}")]
    InvalidRepositoryFilter(String),
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
    let people_count = visible_people_count(pool, &organization, is_member).await?;
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
            public_member_count: people_count,
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
            people: people_count,
            sponsoring: 0,
        },
        viewer_state,
    })
}

pub async fn organization_repositories(
    pool: &PgPool,
    slug: &str,
    viewer_user_id: Option<Uuid>,
    query: OrganizationRepositoryListQuery<'_>,
) -> Result<OrganizationRepositoryList, OrganizationProfileError> {
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
    let filters = normalize_organization_repository_filters(query)?;
    let mut repositories =
        visible_organization_repository_rows(pool, &organization, viewer_user_id, &viewer_state)
            .await?;
    let available_languages = organization_repository_language_options(&repositories);
    let available_types = organization_repository_type_options(&repositories);
    let people_count = visible_people_count(pool, &organization, is_member).await?;
    let packages = packages_count(pool, organization.id, is_member).await?;

    apply_organization_repository_filters(&mut repositories, &filters);
    sort_organization_repositories(&mut repositories, &filters.sort);

    let total = repositories.len() as i64;
    let offset = ((filters.page - 1) * filters.page_size) as usize;
    let limit = filters.page_size as usize;
    let items = repositories.into_iter().skip(offset).take(limit).collect();

    Ok(OrganizationRepositoryList {
        envelope: ListEnvelope {
            items,
            total,
            page: filters.page,
            page_size: filters.page_size,
        },
        mode: "repositories".to_owned(),
        filters,
        available_languages,
        available_types,
        tab_counts: OrganizationTabCounts {
            repositories: total,
            projects: 0,
            packages,
            people: people_count,
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

fn normalize_organization_repository_filters(
    query: OrganizationRepositoryListQuery<'_>,
) -> Result<OrganizationRepositoryFilters, OrganizationProfileError> {
    let pagination = normalize_pagination(query.page, query.page_size);
    let normalized_query = query.query.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.chars().take(120).collect::<String>())
    });
    let repository_type = match query.repository_type.unwrap_or("all").trim() {
        "" | "all" => "all",
        "contributed" | "contributed-by-me" => "contributed",
        "admin" | "admin-access" => "admin",
        "public" => "public",
        "source" | "sources" => "sources",
        "fork" | "forks" => "forks",
        "archived" => "archived",
        "template" | "templates" => "templates",
        other => {
            return Err(OrganizationProfileError::InvalidRepositoryFilter(format!(
                "unsupported organization repository type filter: {other}"
            )));
        }
    };
    let language = query.language.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty() && trimmed != "all").then(|| trimmed.chars().take(80).collect())
    });
    let sort = match query.sort.unwrap_or("updated-desc").trim() {
        "" | "updated" | "updated-desc" | "last-updated" => "updated-desc",
        "name" | "name-asc" => "name-asc",
        "stars" | "stars-desc" => "stars-desc",
        other => {
            return Err(OrganizationProfileError::InvalidRepositoryFilter(format!(
                "unsupported organization repository sort: {other}"
            )));
        }
    };
    let density = match query.density.unwrap_or("comfortable").trim() {
        "" | "comfortable" => "comfortable",
        "compact" => "compact",
        other => {
            return Err(OrganizationProfileError::InvalidRepositoryFilter(format!(
                "unsupported organization repository density: {other}"
            )));
        }
    };

    Ok(OrganizationRepositoryFilters {
        query: normalized_query,
        repository_type: repository_type.to_owned(),
        language,
        sort: sort.to_owned(),
        density: density.to_owned(),
        page: pagination.page,
        page_size: pagination.page_size,
    })
}

async fn visible_organization_repository_rows(
    pool: &PgPool,
    organization: &OrganizationRow,
    viewer_user_id: Option<Uuid>,
    viewer_state: &OrganizationViewerState,
) -> Result<Vec<OrganizationRepositoryListItem>, sqlx::Error> {
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
               repositories.created_by_user_id,
               repositories.created_at,
               repositories.updated_at,
               COALESCE(star_counts.total, 0)::bigint AS stars_count,
               COALESCE(fork_counts.total, 0)::bigint AS forks_count,
               COALESCE(open_issue_counts.total, 0)::bigint AS open_issues_count,
               COALESCE(open_pull_counts.total, 0)::bigint AS open_pull_requests_count,
               source_repositories.name AS fork_source_name,
               COALESCE(source_owner_user.username, source_organizations.slug) AS fork_source_owner,
               viewer_permissions.role AS viewer_repository_role
        FROM repositories
        LEFT JOIN license_templates ON license_templates.slug = repositories.license_template_slug
        LEFT JOIN (
            SELECT repository_id, COUNT(*) AS total
            FROM repository_stars
            GROUP BY repository_id
        ) star_counts ON star_counts.repository_id = repositories.id
        LEFT JOIN (
            SELECT source_repository_id AS repository_id, COUNT(*) AS total
            FROM repository_forks
            GROUP BY source_repository_id
        ) fork_counts ON fork_counts.repository_id = repositories.id
        LEFT JOIN (
            SELECT repository_id, COUNT(*) AS total
            FROM issues
            WHERE state = 'open'
            GROUP BY repository_id
        ) open_issue_counts ON open_issue_counts.repository_id = repositories.id
        LEFT JOIN (
            SELECT repository_id, COUNT(*) AS total
            FROM pull_requests
            WHERE state = 'open'
            GROUP BY repository_id
        ) open_pull_counts ON open_pull_counts.repository_id = repositories.id
        LEFT JOIN repository_forks AS fork_edge
          ON fork_edge.fork_repository_id = repositories.id
        LEFT JOIN repositories AS source_repositories
          ON source_repositories.id = fork_edge.source_repository_id
        LEFT JOIN users AS source_owner_user
          ON source_owner_user.id = source_repositories.owner_user_id
        LEFT JOIN organizations AS source_organizations
          ON source_organizations.id = source_repositories.owner_organization_id
        LEFT JOIN repository_permissions AS viewer_permissions
          ON viewer_permissions.repository_id = repositories.id
         AND viewer_permissions.user_id = $2
        WHERE repositories.owner_organization_id = $1
          AND (
            repositories.visibility = 'public'
            OR $3
            OR EXISTS (
                SELECT 1
                FROM repository_permissions
                WHERE repository_permissions.repository_id = repositories.id
                  AND repository_permissions.user_id = $2
                  AND repository_permissions.role IN ('owner', 'admin', 'write', 'read')
            )
          )
        ORDER BY repositories.updated_at DESC, lower(repositories.name) ASC
        "#,
    )
    .bind(organization.id)
    .bind(viewer_user_id)
    .bind(viewer_state.can_view_internal)
    .fetch_all(pool)
    .await?;

    let org_admin = viewer_state.can_admin;
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
        let fork_source_owner = row.try_get::<Option<String>, _>("fork_source_owner")?;
        let fork_source_name = row.try_get::<Option<String>, _>("fork_source_name")?;
        let fork_source = fork_source_owner
            .zip(fork_source_name)
            .map(|(owner, name)| OrganizationRepositoryForkSource {
                href: format!("/{owner}/{name}"),
                owner,
                name,
            });
        let viewer_repository_role = row.try_get::<Option<String>, _>("viewer_repository_role")?;
        let created_by_user_id: Uuid = row.get("created_by_user_id");
        let can_admin =
            org_admin || matches!(viewer_repository_role.as_deref(), Some("owner" | "admin"));
        let contributed_by_viewer = viewer_user_id.is_some_and(|viewer_user_id| {
            viewer_user_id == created_by_user_id || viewer_repository_role.is_some()
        });

        repositories.push(OrganizationRepositoryListItem {
            id: repository_id,
            owner: organization.slug.clone(),
            name: name.clone(),
            full_name: format!("{}/{name}", organization.slug),
            description: row.get("description"),
            visibility: row.get("visibility"),
            href: format!("/{}/{name}", organization.slug),
            default_branch: row.get("default_branch"),
            primary_language: languages.first().cloned(),
            languages,
            topics,
            stars_count: row.get("stars_count"),
            forks_count: row.get("forks_count"),
            open_issues_count: row.get("open_issues_count"),
            open_pull_requests_count: row.get("open_pull_requests_count"),
            license,
            is_archived: row.get("is_archived"),
            is_fork: fork_source.is_some(),
            is_template: row.get("is_template"),
            is_mirror: row.get("is_mirror"),
            can_admin,
            contributed_by_viewer,
            fork_source,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });
    }

    Ok(repositories)
}

fn organization_repository_language_options(
    repositories: &[OrganizationRepositoryListItem],
) -> Vec<OrganizationRepositoryFilterOption> {
    let mut counts = std::collections::BTreeMap::<String, i64>::new();
    for repository in repositories {
        for language in &repository.languages {
            *counts.entry(language.language.clone()).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .map(|(language, count)| OrganizationRepositoryFilterOption {
            value: language.clone(),
            label: language,
            count,
        })
        .collect()
}

fn organization_repository_type_options(
    repositories: &[OrganizationRepositoryListItem],
) -> Vec<OrganizationRepositoryFilterOption> {
    vec![
        ("all", "All", repositories.len() as i64),
        (
            "contributed",
            "Contributed by me",
            repositories
                .iter()
                .filter(|repository| repository.contributed_by_viewer)
                .count() as i64,
        ),
        (
            "admin",
            "Admin access",
            repositories
                .iter()
                .filter(|repository| repository.can_admin)
                .count() as i64,
        ),
        (
            "public",
            "Public",
            repositories
                .iter()
                .filter(|repository| repository.visibility == "public")
                .count() as i64,
        ),
        (
            "sources",
            "Sources",
            repositories
                .iter()
                .filter(|repository| !repository.is_fork)
                .count() as i64,
        ),
        (
            "forks",
            "Forks",
            repositories
                .iter()
                .filter(|repository| repository.is_fork)
                .count() as i64,
        ),
        (
            "archived",
            "Archived",
            repositories
                .iter()
                .filter(|repository| repository.is_archived)
                .count() as i64,
        ),
        (
            "templates",
            "Templates",
            repositories
                .iter()
                .filter(|repository| repository.is_template)
                .count() as i64,
        ),
    ]
    .into_iter()
    .map(|(value, label, count)| OrganizationRepositoryFilterOption {
        value: value.to_owned(),
        label: label.to_owned(),
        count,
    })
    .collect()
}

fn apply_organization_repository_filters(
    repositories: &mut Vec<OrganizationRepositoryListItem>,
    filters: &OrganizationRepositoryFilters,
) {
    if let Some(query) = &filters.query {
        let needle = query.to_ascii_lowercase();
        repositories.retain(|repository| {
            repository.name.to_ascii_lowercase().contains(&needle)
                || repository
                    .description
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&needle)
                || repository
                    .topics
                    .iter()
                    .any(|topic| topic.to_ascii_lowercase().contains(&needle))
                || repository
                    .languages
                    .iter()
                    .any(|language| language.language.to_ascii_lowercase().contains(&needle))
        });
    }
    if let Some(language) = &filters.language {
        repositories.retain(|repository| {
            repository
                .languages
                .iter()
                .any(|repo_language| repo_language.language.eq_ignore_ascii_case(language))
        });
    }
    match filters.repository_type.as_str() {
        "all" => {}
        "contributed" => repositories.retain(|repository| repository.contributed_by_viewer),
        "admin" => repositories.retain(|repository| repository.can_admin),
        "public" => repositories.retain(|repository| repository.visibility == "public"),
        "sources" => repositories.retain(|repository| !repository.is_fork),
        "forks" => repositories.retain(|repository| repository.is_fork),
        "archived" => repositories.retain(|repository| repository.is_archived),
        "templates" => repositories.retain(|repository| repository.is_template),
        _ => {}
    }
}

fn sort_organization_repositories(repositories: &mut [OrganizationRepositoryListItem], sort: &str) {
    match sort {
        "name" | "name-asc" => repositories.sort_by(|a, b| {
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase())
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        }),
        "stars" | "stars-desc" => repositories.sort_by(|a, b| {
            b.stars_count
                .cmp(&a.stars_count)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
                .then_with(|| {
                    a.name
                        .to_ascii_lowercase()
                        .cmp(&b.name.to_ascii_lowercase())
                })
        }),
        _ => repositories.sort_by(|a, b| {
            b.updated_at.cmp(&a.updated_at).then_with(|| {
                a.name
                    .to_ascii_lowercase()
                    .cmp(&b.name.to_ascii_lowercase())
            })
        }),
    }
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

async fn visible_people_count(
    pool: &PgPool,
    organization: &OrganizationRow,
    is_member: bool,
) -> Result<i64, sqlx::Error> {
    if !is_member && !organization.public_members_visible {
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
