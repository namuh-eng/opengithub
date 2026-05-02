use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

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
        contribution_summary(pool, profile_user.id, viewer_user_id).await?
    } else {
        ProfileContributionSummary {
            total: 0,
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
) -> Result<ProfileContributionSummary, sqlx::Error> {
    let day_rows = sqlx::query(
        r#"
        SELECT day, contribution_count::bigint AS contribution_count
        FROM profile_contribution_days
        WHERE user_id = $1
          AND day >= (CURRENT_DATE - INTERVAL '370 days')
        ORDER BY day ASC
        "#,
    )
    .bind(user_id)
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
    let recent_events = recent_contribution_events(pool, user_id, viewer_user_id).await?;

    Ok(ProfileContributionSummary {
        total,
        days,
        recent_events,
    })
}

async fn recent_contribution_events(
    pool: &PgPool,
    user_id: Uuid,
    viewer_user_id: Option<Uuid>,
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
