use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api_types::{normalize_pagination, ListEnvelope};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PublicUserProfile {
    pub identity: ProfileIdentity,
    pub readme: Option<ProfileReadme>,
    pub pinned_repositories: Vec<ProfilePinnedRepository>,
    pub achievements: Vec<ProfileAchievement>,
    pub organizations: Vec<ProfileOrganization>,
    pub contribution_summary: ProfileContributionSummary,
    pub tab_counts: ProfileTabCounts,
    pub viewer_state: ProfileViewerState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRepositoryList {
    #[serde(flatten)]
    pub envelope: ListEnvelope<ProfileRepositoryListItem>,
    pub filters: ProfileRepositoryFilters,
    pub available_languages: Vec<ProfileRepositoryFilterOption>,
    pub available_types: Vec<ProfileRepositoryFilterOption>,
    pub tab_counts: ProfileTabCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRepositoryListItem {
    pub id: Uuid,
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub visibility: String,
    pub href: String,
    pub default_branch: String,
    pub primary_language: Option<ProfileRepositoryLanguage>,
    pub languages: Vec<ProfileRepositoryLanguage>,
    pub stars_count: i64,
    pub forks_count: i64,
    pub open_issues_count: i64,
    pub open_pull_requests_count: i64,
    pub license: Option<ProfileRepositoryLicense>,
    pub is_archived: bool,
    pub is_fork: bool,
    pub is_template: bool,
    pub is_mirror: bool,
    pub can_be_sponsored: bool,
    pub fork_source: Option<ProfileRepositoryForkSource>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRepositoryLicense {
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRepositoryForkSource {
    pub owner: String,
    pub name: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRepositoryFilters {
    pub query: Option<String>,
    pub repository_type: String,
    pub language: Option<String>,
    pub sort: String,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRepositoryFilterOption {
    pub value: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileIdentity {
    pub id: Uuid,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    pub html_url: String,
    pub profile_visibility: String,
    pub is_private: bool,
    pub joined_at: DateTime<Utc>,
    pub follower_count: Option<i64>,
    pub following_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileReadme {
    pub body: String,
    pub rendered_html: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePinnedRepository {
    pub id: Uuid,
    pub owner: String,
    pub name: String,
    pub description: Option<String>,
    pub visibility: String,
    pub href: String,
    pub default_branch: String,
    pub primary_language: Option<ProfileRepositoryLanguage>,
    pub languages: Vec<ProfileRepositoryLanguage>,
    pub stars_count: i64,
    pub forks_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRepositoryLanguage {
    pub language: String,
    pub color: String,
    pub byte_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAchievement {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub awarded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileOrganization {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileContributionSummary {
    pub total: i64,
    pub year: i32,
    pub days: Vec<ProfileContributionDay>,
    pub recent_events: Vec<ProfileContributionEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileContributionDay {
    pub date: NaiveDate,
    pub count: i64,
    pub intensity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileContributionEvent {
    pub id: Uuid,
    pub event_type: String,
    pub title: String,
    pub target_href: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub repository: Option<ProfileEventRepository>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileEventRepository {
    pub owner: String,
    pub name: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileTabCounts {
    pub repositories: i64,
    pub projects: i64,
    pub packages: i64,
    pub stars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileViewerState {
    pub authenticated: bool,
    pub is_self: bool,
    pub is_following: bool,
    pub is_blocking: bool,
    pub can_follow: bool,
    pub can_block: bool,
    pub can_report: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileActionState {
    pub viewer_state: ProfileViewerState,
    pub follower_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileReport {
    pub id: Uuid,
    pub viewer_state: ProfileViewerState,
}

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("user profile was not found")]
    NotFound,
    #[error("profile action cannot target your own account")]
    SelfAction,
    #[error("profile action is not available for private profiles")]
    PrivateProfile,
    #[error("report reason is required")]
    BlankReportReason,
    #[error("{0}")]
    InvalidRepositoryFilter(String),
    #[error("database error")]
    Sqlx(#[from] sqlx::Error),
}

struct ProfileUserRow {
    id: Uuid,
    login: String,
    display_name: Option<String>,
    avatar_url: Option<String>,
    bio: Option<String>,
    company: Option<String>,
    location: Option<String>,
    website_url: Option<String>,
    profile_visibility: String,
    created_at: DateTime<Utc>,
}

pub async fn public_user_profile(
    pool: &PgPool,
    username: &str,
    viewer_user_id: Option<Uuid>,
    app_url: &url::Url,
    contribution_year: Option<i32>,
) -> Result<PublicUserProfile, ProfileError> {
    let profile_user = profile_user_by_login(pool, username)
        .await?
        .ok_or(ProfileError::NotFound)?;
    let is_private = profile_user.profile_visibility == "private";
    let public_details_visible = !is_private;
    let viewer_state = viewer_state(pool, profile_user.id, viewer_user_id, is_private).await?;
    let (follower_count, following_count) = if public_details_visible {
        (
            Some(count_followers(pool, profile_user.id).await?),
            Some(count_following(pool, profile_user.id).await?),
        )
    } else {
        (None, None)
    };

    let readme = profile_readme(pool, profile_user.id).await?;
    let pinned_repositories = if public_details_visible {
        pinned_repositories(pool, profile_user.id, viewer_user_id).await?
    } else {
        Vec::new()
    };
    let achievements = if public_details_visible {
        achievements(pool, profile_user.id).await?
    } else {
        Vec::new()
    };
    let organizations = if public_details_visible {
        organizations(pool, profile_user.id).await?
    } else {
        Vec::new()
    };
    let contribution_summary = if public_details_visible {
        contribution_summary(pool, profile_user.id, viewer_user_id, contribution_year).await?
    } else {
        ProfileContributionSummary {
            total: 0,
            year: clamped_contribution_year(contribution_year),
            days: Vec::new(),
            recent_events: Vec::new(),
        }
    };
    let tab_counts = if public_details_visible {
        tab_counts(pool, profile_user.id, viewer_user_id).await?
    } else {
        ProfileTabCounts {
            repositories: 0,
            projects: 0,
            packages: 0,
            stars: 0,
        }
    };
    let html_url = app_url
        .join(&format!("/{}", profile_user.login))
        .map(|url| url.to_string())
        .unwrap_or_else(|_| format!("/{}", profile_user.login));

    Ok(PublicUserProfile {
        identity: ProfileIdentity {
            id: profile_user.id,
            login: profile_user.login,
            name: profile_user.display_name,
            avatar_url: profile_user.avatar_url,
            bio: profile_user.bio,
            company: profile_user.company,
            location: profile_user.location,
            website_url: profile_user.website_url,
            html_url,
            profile_visibility: profile_user.profile_visibility,
            is_private,
            joined_at: profile_user.created_at,
            follower_count,
            following_count,
        },
        readme,
        pinned_repositories,
        achievements,
        organizations,
        contribution_summary,
        tab_counts,
        viewer_state,
    })
}

pub async fn profile_repositories(
    pool: &PgPool,
    username: &str,
    viewer_user_id: Option<Uuid>,
    query: ProfileRepositoryListQuery<'_>,
) -> Result<ProfileRepositoryList, ProfileError> {
    let profile_user = profile_user_by_login(pool, username)
        .await?
        .ok_or(ProfileError::NotFound)?;
    if profile_user.profile_visibility == "private" {
        return empty_profile_repository_list(
            query,
            ProfileTabCounts {
                repositories: 0,
                projects: 0,
                packages: 0,
                stars: 0,
            },
        );
    }

    let filters = normalize_repository_filters(query)?;
    let mut repositories =
        visible_profile_repository_rows(pool, profile_user.id, viewer_user_id).await?;
    let available_languages = repository_language_options(&repositories);
    let available_types = repository_type_options(&repositories);
    let tab_counts = tab_counts(pool, profile_user.id, viewer_user_id).await?;

    apply_repository_filters(&mut repositories, &filters);
    sort_profile_repositories(&mut repositories, &filters.sort);

    let total = repositories.len() as i64;
    let offset = ((filters.page - 1) * filters.page_size) as usize;
    let limit = filters.page_size as usize;
    let items = repositories.into_iter().skip(offset).take(limit).collect();

    Ok(ProfileRepositoryList {
        envelope: ListEnvelope {
            items,
            total,
            page: filters.page,
            page_size: filters.page_size,
        },
        filters,
        available_languages,
        available_types,
        tab_counts,
    })
}

#[derive(Debug, Clone, Copy)]
pub struct ProfileRepositoryListQuery<'a> {
    pub query: Option<&'a str>,
    pub repository_type: Option<&'a str>,
    pub language: Option<&'a str>,
    pub sort: Option<&'a str>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn follow_user(
    pool: &PgPool,
    actor_user_id: Uuid,
    username: &str,
) -> Result<ProfileActionState, ProfileError> {
    let profile_user = action_target(pool, actor_user_id, username).await?;
    sqlx::query(
        r#"
        INSERT INTO user_follows (follower_user_id, followed_user_id)
        VALUES ($1, $2)
        ON CONFLICT (follower_user_id, followed_user_id) DO NOTHING
        "#,
    )
    .bind(actor_user_id)
    .bind(profile_user.id)
    .execute(pool)
    .await?;
    insert_profile_audit_event(pool, actor_user_id, "profile.follow", profile_user.id).await?;

    profile_action_state(pool, actor_user_id, profile_user.id).await
}

pub async fn unfollow_user(
    pool: &PgPool,
    actor_user_id: Uuid,
    username: &str,
) -> Result<ProfileActionState, ProfileError> {
    let profile_user = action_target(pool, actor_user_id, username).await?;
    sqlx::query("DELETE FROM user_follows WHERE follower_user_id = $1 AND followed_user_id = $2")
        .bind(actor_user_id)
        .bind(profile_user.id)
        .execute(pool)
        .await?;
    insert_profile_audit_event(pool, actor_user_id, "profile.unfollow", profile_user.id).await?;

    profile_action_state(pool, actor_user_id, profile_user.id).await
}

pub async fn block_user(
    pool: &PgPool,
    actor_user_id: Uuid,
    username: &str,
    reason: Option<&str>,
) -> Result<ProfileActionState, ProfileError> {
    let profile_user = action_target(pool, actor_user_id, username).await?;
    let normalized_reason = reason.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.chars().take(240).collect::<String>())
    });
    sqlx::query(
        r#"
        INSERT INTO user_blocks (blocker_user_id, blocked_user_id, reason)
        VALUES ($1, $2, $3)
        ON CONFLICT (blocker_user_id, blocked_user_id)
        DO UPDATE SET reason = COALESCE(EXCLUDED.reason, user_blocks.reason)
        "#,
    )
    .bind(actor_user_id)
    .bind(profile_user.id)
    .bind(normalized_reason)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        DELETE FROM user_follows
        WHERE (follower_user_id = $1 AND followed_user_id = $2)
           OR (follower_user_id = $2 AND followed_user_id = $1)
        "#,
    )
    .bind(actor_user_id)
    .bind(profile_user.id)
    .execute(pool)
    .await?;
    insert_profile_audit_event(pool, actor_user_id, "profile.block", profile_user.id).await?;

    profile_action_state(pool, actor_user_id, profile_user.id).await
}

pub async fn report_user(
    pool: &PgPool,
    actor_user_id: Uuid,
    username: &str,
    reason: &str,
    details: Option<&str>,
) -> Result<ProfileReport, ProfileError> {
    let profile_user = action_target(pool, actor_user_id, username).await?;
    let normalized_reason = reason.trim();
    if normalized_reason.is_empty() {
        return Err(ProfileError::BlankReportReason);
    }
    let normalized_details = details.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.chars().take(2000).collect::<String>())
    });
    let report_id = sqlx::query_scalar(
        r#"
        INSERT INTO user_reports (reporter_user_id, reported_user_id, reason, details)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(actor_user_id)
    .bind(profile_user.id)
    .bind(normalized_reason.chars().take(120).collect::<String>())
    .bind(normalized_details)
    .fetch_one(pool)
    .await?;
    insert_profile_audit_event(pool, actor_user_id, "profile.report", profile_user.id).await?;
    let viewer_state = viewer_state(pool, profile_user.id, Some(actor_user_id), false).await?;

    Ok(ProfileReport {
        id: report_id,
        viewer_state,
    })
}

async fn action_target(
    pool: &PgPool,
    actor_user_id: Uuid,
    username: &str,
) -> Result<ProfileUserRow, ProfileError> {
    let profile_user = profile_user_by_login(pool, username)
        .await?
        .ok_or(ProfileError::NotFound)?;
    if profile_user.id == actor_user_id {
        return Err(ProfileError::SelfAction);
    }
    if profile_user.profile_visibility == "private" {
        return Err(ProfileError::PrivateProfile);
    }
    Ok(profile_user)
}

async fn profile_action_state(
    pool: &PgPool,
    actor_user_id: Uuid,
    profile_user_id: Uuid,
) -> Result<ProfileActionState, ProfileError> {
    Ok(ProfileActionState {
        viewer_state: viewer_state(pool, profile_user_id, Some(actor_user_id), false).await?,
        follower_count: Some(count_followers(pool, profile_user_id).await?),
    })
}

async fn insert_profile_audit_event(
    pool: &PgPool,
    actor_user_id: Uuid,
    event_type: &str,
    profile_user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, $2, 'user', $3, $4)
        "#,
    )
    .bind(actor_user_id)
    .bind(event_type)
    .bind(profile_user_id.to_string())
    .bind(json!({ "profileUserId": profile_user_id }))
    .execute(pool)
    .await?;
    Ok(())
}

async fn profile_user_by_login(
    pool: &PgPool,
    username: &str,
) -> Result<Option<ProfileUserRow>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, username, display_name, avatar_url, bio, company, location, website_url,
               profile_visibility, created_at
        FROM users
        WHERE lower(username) = lower($1)
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| ProfileUserRow {
        id: row.get("id"),
        login: row
            .try_get::<Option<String>, _>("username")
            .ok()
            .flatten()
            .unwrap_or_else(|| "user".to_owned()),
        display_name: row.get("display_name"),
        avatar_url: row.get("avatar_url"),
        bio: row.get("bio"),
        company: row.get("company"),
        location: row.get("location"),
        website_url: row.get("website_url"),
        profile_visibility: row.get("profile_visibility"),
        created_at: row.get("created_at"),
    }))
}

async fn count_followers(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*)::bigint FROM user_follows WHERE followed_user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
}

async fn count_following(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*)::bigint FROM user_follows WHERE follower_user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
}

async fn viewer_state(
    pool: &PgPool,
    profile_user_id: Uuid,
    viewer_user_id: Option<Uuid>,
    is_private: bool,
) -> Result<ProfileViewerState, sqlx::Error> {
    let Some(viewer_user_id) = viewer_user_id else {
        return Ok(ProfileViewerState {
            authenticated: false,
            is_self: false,
            is_following: false,
            is_blocking: false,
            can_follow: !is_private,
            can_block: !is_private,
            can_report: !is_private,
        });
    };
    let is_self = viewer_user_id == profile_user_id;
    let is_following = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM user_follows WHERE follower_user_id = $1 AND followed_user_id = $2)",
    )
    .bind(viewer_user_id)
    .bind(profile_user_id)
    .fetch_one(pool)
    .await?;
    let is_blocking = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM user_blocks WHERE blocker_user_id = $1 AND blocked_user_id = $2)",
    )
    .bind(viewer_user_id)
    .bind(profile_user_id)
    .fetch_one(pool)
    .await?;

    Ok(ProfileViewerState {
        authenticated: true,
        is_self,
        is_following,
        is_blocking,
        can_follow: !is_self && !is_private,
        can_block: !is_self && !is_private,
        can_report: !is_self && !is_private,
    })
}

async fn profile_readme(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<ProfileReadme>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT body, rendered_html, updated_at
        FROM user_profile_readmes
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| ProfileReadme {
        body: row.get("body"),
        rendered_html: row.get("rendered_html"),
        updated_at: row.get("updated_at"),
    }))
}

async fn pinned_repositories(
    pool: &PgPool,
    user_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<Vec<ProfilePinnedRepository>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT repositories.id,
               COALESCE(owner_user.username, organizations.slug) AS owner_login,
               repositories.name,
               repositories.description,
               repositories.visibility,
               repositories.default_branch,
               repositories.updated_at,
               COALESCE(star_counts.total, 0)::bigint AS stars_count,
               COALESCE(fork_counts.total, 0)::bigint AS forks_count
        FROM profile_pins
        JOIN repositories ON repositories.id = profile_pins.repository_id
        LEFT JOIN users AS owner_user ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
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
        WHERE profile_pins.user_id = $1
          AND (
            repositories.visibility = 'public'
            OR repositories.owner_user_id = $2
            OR EXISTS (
                SELECT 1
                FROM repository_permissions
                WHERE repository_permissions.repository_id = repositories.id
                  AND repository_permissions.user_id = $2
                  AND repository_permissions.role IN ('owner', 'admin', 'write', 'read')
            )
            OR (
                repositories.visibility = 'internal'
                AND repositories.owner_organization_id IS NOT NULL
                AND EXISTS (
                    SELECT 1
                    FROM organization_memberships
                    WHERE organization_memberships.organization_id = repositories.owner_organization_id
                      AND organization_memberships.user_id = $2
                )
            )
          )
        ORDER BY profile_pins.position ASC, repositories.name ASC
        LIMIT 6
        "#,
    )
    .bind(user_id)
    .bind(viewer_user_id)
    .fetch_all(pool)
    .await?;

    let mut pinned = Vec::with_capacity(rows.len());
    for row in rows {
        let repository_id = row.get("id");
        let languages = repository_languages(pool, repository_id).await?;
        let primary_language = languages.first().cloned();
        let owner: String = row.get("owner_login");
        let name: String = row.get("name");
        pinned.push(ProfilePinnedRepository {
            id: repository_id,
            owner: owner.clone(),
            name: name.clone(),
            description: row.get("description"),
            visibility: row.get("visibility"),
            href: format!("/{owner}/{name}"),
            default_branch: row.get("default_branch"),
            primary_language,
            languages,
            stars_count: row.get("stars_count"),
            forks_count: row.get("forks_count"),
            updated_at: row.get("updated_at"),
        });
    }

    Ok(pinned)
}

async fn repository_languages(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<ProfileRepositoryLanguage>, sqlx::Error> {
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
        .map(|row| ProfileRepositoryLanguage {
            language: row.get("language"),
            color: row.get("color"),
            byte_count: row.get("byte_count"),
        })
        .collect())
}

fn empty_profile_repository_list(
    query: ProfileRepositoryListQuery<'_>,
    tab_counts: ProfileTabCounts,
) -> Result<ProfileRepositoryList, ProfileError> {
    let filters = normalize_repository_filters(query)?;
    Ok(ProfileRepositoryList {
        envelope: ListEnvelope {
            items: Vec::new(),
            total: 0,
            page: filters.page,
            page_size: filters.page_size,
        },
        filters,
        available_languages: Vec::new(),
        available_types: repository_type_options(&[]),
        tab_counts,
    })
}

fn normalize_repository_filters(
    query: ProfileRepositoryListQuery<'_>,
) -> Result<ProfileRepositoryFilters, ProfileError> {
    let pagination = normalize_pagination(query.page, query.page_size);
    let normalized_query = query.query.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.chars().take(120).collect::<String>())
    });
    let repository_type = match query.repository_type.unwrap_or("all").trim() {
        "" | "all" => "all",
        "source" | "sources" => "sources",
        "fork" | "forks" => "forks",
        "archived" => "archived",
        "sponsorable" | "can-be-sponsored" | "can_be_sponsored" => "can-be-sponsored",
        "mirror" | "mirrors" => "mirrors",
        "template" | "templates" => "templates",
        other => {
            return Err(ProfileError::InvalidRepositoryFilter(format!(
                "unsupported repository type filter: {other}"
            )));
        }
    };
    let language = query.language.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty() && trimmed != "all").then(|| trimmed.chars().take(80).collect())
    });
    let sort = match query.sort.unwrap_or("updated").trim() {
        "" | "updated" | "last-updated" => "updated",
        "name" => "name",
        "stars" => "stars",
        other => {
            return Err(ProfileError::InvalidRepositoryFilter(format!(
                "unsupported repository sort: {other}"
            )));
        }
    };

    Ok(ProfileRepositoryFilters {
        query: normalized_query,
        repository_type: repository_type.to_owned(),
        language,
        sort: sort.to_owned(),
        page: pagination.page,
        page_size: pagination.page_size,
    })
}

async fn visible_profile_repository_rows(
    pool: &PgPool,
    user_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<Vec<ProfileRepositoryListItem>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT repositories.id,
               COALESCE(owner_user.username, organizations.slug) AS owner_login,
               repositories.name,
               repositories.description,
               repositories.visibility,
               repositories.default_branch,
               repositories.is_archived,
               repositories.is_template,
               repositories.is_mirror,
               repositories.can_be_sponsored,
               repositories.license_template_slug,
               license_templates.display_name AS license_name,
               repositories.created_at,
               repositories.updated_at,
               COALESCE(star_counts.total, 0)::bigint AS stars_count,
               COALESCE(fork_counts.total, 0)::bigint AS forks_count,
               COALESCE(open_issue_counts.total, 0)::bigint AS open_issues_count,
               COALESCE(open_pull_counts.total, 0)::bigint AS open_pull_requests_count,
               source_repositories.name AS fork_source_name,
               COALESCE(source_owner_user.username, source_organizations.slug) AS fork_source_owner
        FROM repositories
        LEFT JOIN users AS owner_user ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
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
        WHERE repositories.owner_user_id = $1
          AND (
            repositories.visibility = 'public'
            OR repositories.owner_user_id = $2
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
    .bind(user_id)
    .bind(viewer_user_id)
    .fetch_all(pool)
    .await?;

    let mut repositories = Vec::with_capacity(rows.len());
    for row in rows {
        let repository_id = row.get("id");
        let owner: String = row.get("owner_login");
        let name: String = row.get("name");
        let languages = repository_languages(pool, repository_id).await?;
        let license_slug = row.try_get::<Option<String>, _>("license_template_slug")?;
        let license = license_slug.map(|slug| ProfileRepositoryLicense {
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
            .map(|(owner, name)| ProfileRepositoryForkSource {
                href: format!("/{owner}/{name}"),
                owner,
                name,
            });

        repositories.push(ProfileRepositoryListItem {
            id: repository_id,
            owner: owner.clone(),
            name: name.clone(),
            full_name: format!("{owner}/{name}"),
            description: row.get("description"),
            visibility: row.get("visibility"),
            href: format!("/{owner}/{name}"),
            default_branch: row.get("default_branch"),
            primary_language: languages.first().cloned(),
            languages,
            stars_count: row.get("stars_count"),
            forks_count: row.get("forks_count"),
            open_issues_count: row.get("open_issues_count"),
            open_pull_requests_count: row.get("open_pull_requests_count"),
            license,
            is_archived: row.get("is_archived"),
            is_fork: fork_source.is_some(),
            is_template: row.get("is_template"),
            is_mirror: row.get("is_mirror"),
            can_be_sponsored: row.get("can_be_sponsored"),
            fork_source,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });
    }

    Ok(repositories)
}

fn repository_language_options(
    repositories: &[ProfileRepositoryListItem],
) -> Vec<ProfileRepositoryFilterOption> {
    let mut counts = std::collections::BTreeMap::<String, i64>::new();
    for repository in repositories {
        for language in &repository.languages {
            *counts.entry(language.language.clone()).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .map(|(language, count)| ProfileRepositoryFilterOption {
            value: language.clone(),
            label: language,
            count,
        })
        .collect()
}

fn repository_type_options(
    repositories: &[ProfileRepositoryListItem],
) -> Vec<ProfileRepositoryFilterOption> {
    let mut options = vec![
        ("all", "All", repositories.len() as i64),
        (
            "sources",
            "Sources",
            repositories.iter().filter(|repo| !repo.is_fork).count() as i64,
        ),
        (
            "forks",
            "Forks",
            repositories.iter().filter(|repo| repo.is_fork).count() as i64,
        ),
        (
            "archived",
            "Archived",
            repositories.iter().filter(|repo| repo.is_archived).count() as i64,
        ),
        (
            "can-be-sponsored",
            "Can be sponsored",
            repositories
                .iter()
                .filter(|repo| repo.can_be_sponsored)
                .count() as i64,
        ),
        (
            "mirrors",
            "Mirrors",
            repositories.iter().filter(|repo| repo.is_mirror).count() as i64,
        ),
        (
            "templates",
            "Templates",
            repositories.iter().filter(|repo| repo.is_template).count() as i64,
        ),
    ];
    options
        .drain(..)
        .map(|(value, label, count)| ProfileRepositoryFilterOption {
            value: value.to_owned(),
            label: label.to_owned(),
            count,
        })
        .collect()
}

fn apply_repository_filters(
    repositories: &mut Vec<ProfileRepositoryListItem>,
    filters: &ProfileRepositoryFilters,
) {
    if let Some(query) = &filters.query {
        let needle = query.to_ascii_lowercase();
        repositories.retain(|repo| {
            repo.name.to_ascii_lowercase().contains(&needle)
                || repo
                    .description
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&needle)
                || repo
                    .languages
                    .iter()
                    .any(|language| language.language.to_ascii_lowercase().contains(&needle))
        });
    }
    if let Some(language) = &filters.language {
        repositories.retain(|repo| {
            repo.languages
                .iter()
                .any(|repo_language| repo_language.language.eq_ignore_ascii_case(language))
        });
    }
    match filters.repository_type.as_str() {
        "all" => {}
        "sources" => repositories.retain(|repo| !repo.is_fork),
        "forks" => repositories.retain(|repo| repo.is_fork),
        "archived" => repositories.retain(|repo| repo.is_archived),
        "can-be-sponsored" => repositories.retain(|repo| repo.can_be_sponsored),
        "mirrors" => repositories.retain(|repo| repo.is_mirror),
        "templates" => repositories.retain(|repo| repo.is_template),
        _ => {}
    }
}

fn sort_profile_repositories(repositories: &mut [ProfileRepositoryListItem], sort: &str) {
    match sort {
        "name" => repositories.sort_by(|a, b| {
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase())
                .then_with(|| a.updated_at.cmp(&b.updated_at).reverse())
        }),
        "stars" => repositories.sort_by(|a, b| {
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

async fn achievements(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ProfileAchievement>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT achievements.slug,
               achievements.name,
               achievements.description,
               achievements.icon,
               user_achievements.awarded_at
        FROM user_achievements
        JOIN achievements ON achievements.id = user_achievements.achievement_id
        WHERE user_achievements.user_id = $1
        ORDER BY user_achievements.awarded_at DESC, achievements.name ASC
        LIMIT 12
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ProfileAchievement {
            slug: row.get("slug"),
            name: row.get("name"),
            description: row.get("description"),
            icon: row.get("icon"),
            awarded_at: row.get("awarded_at"),
        })
        .collect())
}

async fn organizations(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ProfileOrganization>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT organizations.id, organizations.slug, organizations.display_name
        FROM organization_memberships
        JOIN organizations ON organizations.id = organization_memberships.organization_id
        WHERE organization_memberships.user_id = $1
        ORDER BY organizations.display_name ASC, organizations.slug ASC
        LIMIT 20
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let slug: String = row.get("slug");
            ProfileOrganization {
                id: row.get("id"),
                slug: slug.clone(),
                name: row.get("display_name"),
                avatar_url: None,
                href: format!("/{slug}"),
            }
        })
        .collect())
}

async fn contribution_summary(
    pool: &PgPool,
    user_id: Uuid,
    viewer_user_id: Option<Uuid>,
    requested_year: Option<i32>,
) -> Result<ProfileContributionSummary, sqlx::Error> {
    let year = clamped_contribution_year(requested_year);
    let start = NaiveDate::from_ymd_opt(year, 1, 1).expect("valid contribution year start");
    let end = NaiveDate::from_ymd_opt(year, 12, 31).expect("valid contribution year end");
    let day_rows = sqlx::query(
        r#"
        SELECT day, contribution_count::bigint AS contribution_count
        FROM profile_contribution_days
        WHERE user_id = $1
          AND day >= $2
          AND day <= $3
        ORDER BY day ASC
        "#,
    )
    .bind(user_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;
    let mut total = 0;
    let days = day_rows
        .into_iter()
        .map(|row| {
            let count: i64 = row.get("contribution_count");
            total += count;
            ProfileContributionDay {
                date: row.get("day"),
                count,
                intensity: contribution_intensity(count),
            }
        })
        .collect();
    let recent_events =
        recent_contribution_events(pool, user_id, viewer_user_id, start, end).await?;

    Ok(ProfileContributionSummary {
        total,
        year,
        days,
        recent_events,
    })
}

async fn recent_contribution_events(
    pool: &PgPool,
    user_id: Uuid,
    viewer_user_id: Option<Uuid>,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<ProfileContributionEvent>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT profile_contribution_events.id,
               profile_contribution_events.event_type,
               profile_contribution_events.title,
               profile_contribution_events.target_href,
               profile_contribution_events.occurred_at,
               COALESCE(owner_user.username, organizations.slug) AS owner_login,
               repositories.name AS repository_name
        FROM profile_contribution_events
        LEFT JOIN repositories ON repositories.id = profile_contribution_events.repository_id
        LEFT JOIN users AS owner_user ON owner_user.id = repositories.owner_user_id
        LEFT JOIN organizations ON organizations.id = repositories.owner_organization_id
        WHERE profile_contribution_events.user_id = $1
          AND profile_contribution_events.occurred_at >= $3::date
          AND profile_contribution_events.occurred_at < ($4::date + INTERVAL '1 day')
          AND (
            repositories.id IS NULL
            OR repositories.visibility = 'public'
            OR repositories.owner_user_id = $2
            OR EXISTS (
                SELECT 1
                FROM repository_permissions
                WHERE repository_permissions.repository_id = repositories.id
                  AND repository_permissions.user_id = $2
                  AND repository_permissions.role IN ('owner', 'admin', 'write', 'read')
            )
          )
        ORDER BY profile_contribution_events.occurred_at DESC
        LIMIT 20
        "#,
    )
    .bind(user_id)
    .bind(viewer_user_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let owner = row.try_get::<String, _>("owner_login").ok();
            let name = row.try_get::<String, _>("repository_name").ok();
            let repository = owner.zip(name).map(|(owner, name)| ProfileEventRepository {
                href: format!("/{owner}/{name}"),
                owner,
                name,
            });
            ProfileContributionEvent {
                id: row.get("id"),
                event_type: row.get("event_type"),
                title: row.get("title"),
                target_href: row.get("target_href"),
                occurred_at: row.get("occurred_at"),
                repository,
            }
        })
        .collect())
}

async fn tab_counts(
    pool: &PgPool,
    user_id: Uuid,
    viewer_user_id: Option<Uuid>,
) -> Result<ProfileTabCounts, sqlx::Error> {
    let repositories = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)::bigint
        FROM repositories
        WHERE owner_user_id = $1
          AND (
            visibility = 'public'
            OR owner_user_id = $2
            OR EXISTS (
                SELECT 1
                FROM repository_permissions
                WHERE repository_permissions.repository_id = repositories.id
                  AND repository_permissions.user_id = $2
                  AND repository_permissions.role IN ('owner', 'admin', 'write', 'read')
            )
          )
        "#,
    )
    .bind(user_id)
    .bind(viewer_user_id)
    .fetch_one(pool)
    .await?;
    let stars = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM repository_stars WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(ProfileTabCounts {
        repositories,
        projects: 0,
        packages: 0,
        stars,
    })
}

fn contribution_intensity(count: i64) -> i64 {
    match count {
        0 => 0,
        1..=2 => 1,
        3..=5 => 2,
        6..=9 => 3,
        _ => 4,
    }
}

fn clamped_contribution_year(requested_year: Option<i32>) -> i32 {
    let current_year = Utc::now().date_naive().year();
    requested_year
        .unwrap_or(current_year)
        .clamp(2008, current_year)
}
