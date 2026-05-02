use chrono::{Datelike, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("profile was not found")]
    NotFound,
    #[error("viewer cannot mutate their own relationship state")]
    SelfRelationship,
    #[error("invalid report reason")]
    InvalidReportReason,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PublicProfileView {
    pub identity: ProfileIdentity,
    pub viewer: ProfileViewerState,
    pub tabs: ProfileTabCounts,
    pub readme: Option<ProfileReadme>,
    pub pinned_items: Vec<ProfilePinnedItem>,
    pub achievements: Vec<ProfileAchievement>,
    pub contributions: ProfileContributionSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileIdentity {
    pub id: Uuid,
    pub login: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    pub private_profile: bool,
    pub follower_count: i64,
    pub following_count: i64,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileViewerState {
    pub authenticated: bool,
    pub is_self: bool,
    pub following: bool,
    pub blocked: bool,
    pub can_follow: bool,
    pub can_block: bool,
    pub can_report: bool,
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
pub struct ProfileReadme {
    pub body: String,
    pub rendered_body: Option<String>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfilePinnedItem {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub description: Option<String>,
    pub href: Option<String>,
    pub language: Option<String>,
    pub stars_count: i64,
    pub forks_count: i64,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAchievement {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub awarded_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileContributionSummary {
    pub year: i32,
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
    pub repository_name: Option<String>,
    pub target_href: Option<String>,
    pub occurred_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FollowState {
    pub following: bool,
    pub follower_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BlockState {
    pub blocked: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportInput {
    pub reason: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReportReceipt {
    pub id: Uuid,
    pub status: String,
}

pub async fn profile_by_login(
    pool: &PgPool,
    viewer_id: Option<Uuid>,
    login: &str,
    year: Option<i32>,
) -> Result<Option<PublicProfileView>, ProfileError> {
    let profile = profile_identity(pool, login).await?;
    let Some(mut identity) = profile else { return Ok(None); };
    let is_private_for_viewer = identity.private_profile && viewer_id != Some(identity.id);
    let viewer = viewer_state(pool, viewer_id, identity.id).await?;
    if is_private_for_viewer {
        identity.follower_count = 0;
        identity.following_count = 0;
    }
    let tabs = if is_private_for_viewer {
        ProfileTabCounts { repositories: 0, projects: 0, packages: 0, stars: 0 }
    } else {
        tab_counts(pool, identity.id).await?
    };
    let readme = profile_readme(pool, identity.id).await?;
    let (pinned_items, achievements, contributions) = if is_private_for_viewer {
        (Vec::new(), Vec::new(), empty_contributions(year.unwrap_or_else(|| Utc::now().year())))
    } else {
        (
            pinned_items(pool, identity.id).await?,
            achievements(pool, identity.id).await?,
            contributions(pool, identity.id, year.unwrap_or_else(|| Utc::now().year())).await?,
        )
    };
    Ok(Some(PublicProfileView { identity, viewer, tabs, readme, pinned_items, achievements, contributions }))
}

pub async fn set_follow_state(
    pool: &PgPool,
    actor_id: Uuid,
    login: &str,
    following: bool,
) -> Result<Option<FollowState>, ProfileError> {
    let Some(target_id) = user_id_by_login(pool, login).await? else { return Ok(None); };
    if target_id == actor_id { return Err(ProfileError::SelfRelationship); }
    if following {
        sqlx::query("INSERT INTO user_follows (follower_user_id, followed_user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(actor_id).bind(target_id).execute(pool).await?;
    } else {
        sqlx::query("DELETE FROM user_follows WHERE follower_user_id = $1 AND followed_user_id = $2")
            .bind(actor_id).bind(target_id).execute(pool).await?;
    }
    let follower_count = follower_count(pool, target_id).await?;
    Ok(Some(FollowState { following, follower_count }))
}

pub async fn set_block_state(
    pool: &PgPool,
    actor_id: Uuid,
    login: &str,
    blocked: bool,
) -> Result<Option<BlockState>, ProfileError> {
    let Some(target_id) = user_id_by_login(pool, login).await? else { return Ok(None); };
    if target_id == actor_id { return Err(ProfileError::SelfRelationship); }
    if blocked {
        sqlx::query("INSERT INTO user_blocks (blocker_user_id, blocked_user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(actor_id).bind(target_id).execute(pool).await?;
        sqlx::query("DELETE FROM user_follows WHERE (follower_user_id = $1 AND followed_user_id = $2) OR (follower_user_id = $2 AND followed_user_id = $1)")
            .bind(actor_id).bind(target_id).execute(pool).await?;
    } else {
        sqlx::query("DELETE FROM user_blocks WHERE blocker_user_id = $1 AND blocked_user_id = $2")
            .bind(actor_id).bind(target_id).execute(pool).await?;
    }
    Ok(Some(BlockState { blocked }))
}

pub async fn report_user(
    pool: &PgPool,
    actor_id: Uuid,
    login: &str,
    input: ReportInput,
) -> Result<Option<ReportReceipt>, ProfileError> {
    let Some(target_id) = user_id_by_login(pool, login).await? else { return Ok(None); };
    if target_id == actor_id { return Err(ProfileError::SelfRelationship); }
    let reason = input.reason.trim();
    if reason.is_empty() || reason.len() > 80 { return Err(ProfileError::InvalidReportReason); }
    let details = input.details.as_deref().map(str::trim).filter(|v| !v.is_empty()).map(|v| v.chars().take(2000).collect::<String>());
    let id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO user_reports (reporter_user_id, reported_user_id, reason, details) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(actor_id).bind(target_id).bind(reason).bind(details).fetch_one(pool).await?;
    Ok(Some(ReportReceipt { id, status: "received".to_owned() }))
}

async fn profile_identity(pool: &PgPool, login: &str) -> Result<Option<ProfileIdentity>, ProfileError> {
    let row = sqlx::query(
        r#"
        SELECT id,
               COALESCE(username, regexp_replace(lower(split_part(email, '@', 1)), '[^a-z0-9-]+', '-', 'g')) AS login,
               display_name, avatar_url, bio, company, location, website_url, private_profile,
               (SELECT count(*) FROM user_follows WHERE followed_user_id = users.id) AS follower_count,
               (SELECT count(*) FROM user_follows WHERE follower_user_id = users.id) AS following_count,
               created_at
        FROM users
        WHERE lower(COALESCE(username, regexp_replace(lower(split_part(email, '@', 1)), '[^a-z0-9-]+', '-', 'g'))) = lower($1)
        "#,
    )
    .bind(login)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| ProfileIdentity {
        id: row.get("id"),
        login: row.get("login"),
        display_name: row.get("display_name"),
        avatar_url: row.get("avatar_url"),
        bio: row.get("bio"),
        company: row.get("company"),
        location: row.get("location"),
        website_url: row.get("website_url"),
        private_profile: row.get("private_profile"),
        follower_count: row.get::<i64, _>("follower_count"),
        following_count: row.get::<i64, _>("following_count"),
        created_at: row.get("created_at"),
    }))
}

async fn user_id_by_login(pool: &PgPool, login: &str) -> Result<Option<Uuid>, ProfileError> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM users WHERE lower(COALESCE(username, regexp_replace(lower(split_part(email, '@', 1)), '[^a-z0-9-]+', '-', 'g'))) = lower($1)",
    )
    .bind(login)
    .fetch_optional(pool)
    .await?)
}

async fn viewer_state(pool: &PgPool, viewer_id: Option<Uuid>, target_id: Uuid) -> Result<ProfileViewerState, ProfileError> {
    let Some(viewer_id) = viewer_id else {
        return Ok(ProfileViewerState { authenticated: false, is_self: false, following: false, blocked: false, can_follow: false, can_block: false, can_report: false });
    };
    let is_self = viewer_id == target_id;
    let following = if is_self { false } else { sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM user_follows WHERE follower_user_id = $1 AND followed_user_id = $2)").bind(viewer_id).bind(target_id).fetch_one(pool).await? };
    let blocked = if is_self { false } else { sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM user_blocks WHERE blocker_user_id = $1 AND blocked_user_id = $2)").bind(viewer_id).bind(target_id).fetch_one(pool).await? };
    Ok(ProfileViewerState { authenticated: true, is_self, following, blocked, can_follow: !is_self && !blocked, can_block: !is_self, can_report: !is_self })
}

async fn tab_counts(pool: &PgPool, user_id: Uuid) -> Result<ProfileTabCounts, ProfileError> {
    let row = sqlx::query(
        r#"
        SELECT
          (SELECT count(*) FROM repositories WHERE owner_user_id = $1 AND visibility = 'public') AS repositories,
          0::bigint AS projects,
          0::bigint AS packages,
          (SELECT count(*) FROM repository_stars WHERE user_id = $1) AS stars
        "#,
    ).bind(user_id).fetch_one(pool).await?;
    Ok(ProfileTabCounts { repositories: row.get("repositories"), projects: row.get("projects"), packages: row.get("packages"), stars: row.get("stars") })
}

async fn profile_readme(pool: &PgPool, user_id: Uuid) -> Result<Option<ProfileReadme>, ProfileError> {
    let row = sqlx::query("SELECT body, rendered_body, updated_at FROM profile_readmes WHERE user_id = $1")
        .bind(user_id).fetch_optional(pool).await?;
    Ok(row.map(|row| ProfileReadme { body: row.get("body"), rendered_body: row.get("rendered_body"), updated_at: row.get("updated_at") }))
}

async fn pinned_items(pool: &PgPool, user_id: Uuid) -> Result<Vec<ProfilePinnedItem>, ProfileError> {
    let rows = sqlx::query(
        r#"
        WITH explicit_pins AS (
          SELECT profile_pins.id::text AS id, 'repository'::text AS kind, repositories.name AS title,
                 repositories.description, '/' || COALESCE(users.username, regexp_replace(lower(split_part(users.email, '@', 1)), '[^a-z0-9-]+', '-', 'g')) || '/' || repositories.name AS href,
                 (SELECT language FROM repository_languages WHERE repository_id = repositories.id ORDER BY byte_count DESC, language ASC LIMIT 1) AS language,
                 (SELECT count(*) FROM repository_stars WHERE repository_id = repositories.id) AS stars_count,
                 (SELECT count(*) FROM repository_forks WHERE source_repository_id = repositories.id) AS forks_count,
                 repositories.updated_at, profile_pins.position
          FROM profile_pins
          JOIN repositories ON repositories.id = profile_pins.repository_id
          JOIN users ON users.id = repositories.owner_user_id
          WHERE profile_pins.user_id = $1 AND repositories.visibility = 'public'
        ), fallback_repos AS (
          SELECT repositories.id::text AS id, 'repository'::text AS kind, repositories.name AS title,
                 repositories.description, '/' || COALESCE(users.username, regexp_replace(lower(split_part(users.email, '@', 1)), '[^a-z0-9-]+', '-', 'g')) || '/' || repositories.name AS href,
                 (SELECT language FROM repository_languages WHERE repository_id = repositories.id ORDER BY byte_count DESC, language ASC LIMIT 1) AS language,
                 (SELECT count(*) FROM repository_stars WHERE repository_id = repositories.id) AS stars_count,
                 (SELECT count(*) FROM repository_forks WHERE source_repository_id = repositories.id) AS forks_count,
                 repositories.updated_at, 1000 AS position
          FROM repositories
          JOIN users ON users.id = repositories.owner_user_id
          WHERE repositories.owner_user_id = $1 AND repositories.visibility = 'public'
            AND NOT EXISTS (SELECT 1 FROM profile_pins WHERE profile_pins.user_id = $1)
          ORDER BY repositories.updated_at DESC, repositories.name ASC
          LIMIT 6
        )
        SELECT * FROM explicit_pins
        UNION ALL
        SELECT * FROM fallback_repos
        ORDER BY position ASC, updated_at DESC, title ASC
        LIMIT 6
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|row| ProfilePinnedItem {
        id: row.get("id"), kind: row.get("kind"), title: row.get("title"), description: row.get("description"), href: row.get("href"), language: row.get("language"), stars_count: row.get("stars_count"), forks_count: row.get("forks_count"), updated_at: row.get("updated_at"),
    }).collect())
}

async fn achievements(pool: &PgPool, user_id: Uuid) -> Result<Vec<ProfileAchievement>, ProfileError> {
    let rows = sqlx::query(
        r#"
        SELECT achievements.slug, achievements.name, achievements.description, user_achievements.awarded_at
        FROM user_achievements
        JOIN achievements ON achievements.id = user_achievements.achievement_id
        JOIN users ON users.id = user_achievements.user_id
        WHERE user_achievements.user_id = $1 AND users.achievements_enabled
        UNION ALL
        SELECT achievements.slug, achievements.name, achievements.description, min(repositories.created_at) AS awarded_at
        FROM achievements, repositories
        WHERE achievements.slug = 'first-repository' AND repositories.owner_user_id = $1 AND repositories.visibility = 'public'
          AND NOT EXISTS (SELECT 1 FROM user_achievements WHERE user_id = $1)
        GROUP BY achievements.slug, achievements.name, achievements.description
        ORDER BY awarded_at DESC
        LIMIT 6
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|row| ProfileAchievement { slug: row.get("slug"), name: row.get("name"), description: row.get("description"), awarded_at: row.get("awarded_at") }).collect())
}

async fn contributions(pool: &PgPool, user_id: Uuid, year: i32) -> Result<ProfileContributionSummary, ProfileError> {
    let start = NaiveDate::from_ymd_opt(year, 1, 1).expect("valid year start");
    let end = NaiveDate::from_ymd_opt(year, 12, 31).expect("valid year end");
    let rows = sqlx::query(
        r#"
        SELECT contribution_date, contribution_count::bigint AS contribution_count
        FROM profile_contribution_days
        WHERE user_id = $1 AND contribution_date BETWEEN $2 AND $3
        "#,
    ).bind(user_id).bind(start).bind(end).fetch_all(pool).await?;
    let mut by_date = std::collections::BTreeMap::new();
    for row in rows { by_date.insert(row.get::<NaiveDate, _>("contribution_date"), row.get::<i64, _>("contribution_count")); }
    let commit_rows = sqlx::query(
        r#"
        SELECT committed_at::date AS contribution_date, count(*)::bigint AS contribution_count
        FROM commits
        WHERE author_user_id = $1 AND committed_at::date BETWEEN $2 AND $3
        GROUP BY committed_at::date
        "#,
    ).bind(user_id).bind(start).bind(end).fetch_all(pool).await?;
    for row in commit_rows { *by_date.entry(row.get::<NaiveDate, _>("contribution_date")).or_insert(0) += row.get::<i64, _>("contribution_count"); }
    let mut days = Vec::new();
    let mut date = start;
    let mut total = 0;
    while date <= end {
        let count = *by_date.get(&date).unwrap_or(&0);
        total += count;
        days.push(ProfileContributionDay { date, count, intensity: intensity_for_count(count) });
        date += Duration::days(1);
    }
    let recent_events = recent_events(pool, user_id).await?;
    Ok(ProfileContributionSummary { year, total, days, recent_events })
}

fn empty_contributions(year: i32) -> ProfileContributionSummary {
    let start = NaiveDate::from_ymd_opt(year, 1, 1).expect("valid year start");
    let end = NaiveDate::from_ymd_opt(year, 12, 31).expect("valid year end");
    let mut days = Vec::new();
    let mut date = start;
    while date <= end { days.push(ProfileContributionDay { date, count: 0, intensity: 0 }); date += Duration::days(1); }
    ProfileContributionSummary { year, total: 0, days, recent_events: Vec::new() }
}

async fn recent_events(pool: &PgPool, user_id: Uuid) -> Result<Vec<ProfileContributionEvent>, ProfileError> {
    let rows = sqlx::query(
        r#"
        SELECT profile_contribution_events.id, event_type, title, repositories.name AS repository_name, target_href, occurred_at
        FROM profile_contribution_events
        LEFT JOIN repositories ON repositories.id = profile_contribution_events.repository_id
        WHERE profile_contribution_events.user_id = $1
        ORDER BY occurred_at DESC
        LIMIT 8
        "#,
    ).bind(user_id).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|row| ProfileContributionEvent { id: row.get("id"), event_type: row.get("event_type"), title: row.get("title"), repository_name: row.get("repository_name"), target_href: row.get("target_href"), occurred_at: row.get("occurred_at") }).collect())
}

async fn follower_count(pool: &PgPool, user_id: Uuid) -> Result<i64, ProfileError> {
    Ok(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM user_follows WHERE followed_user_id = $1").bind(user_id).fetch_one(pool).await?)
}

fn intensity_for_count(count: i64) -> i64 {
    match count { 0 => 0, 1..=2 => 1, 3..=5 => 2, 6..=9 => 3, _ => 4 }
}
