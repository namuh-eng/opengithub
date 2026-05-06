use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

const SOCIAL_PROVIDERS: [&str; 4] = ["x", "mastodon", "linkedin", "bluesky"];
const MAX_AVATAR_BYTES: i32 = 2 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersonalProfileSettings {
    pub user_id: Uuid,
    pub login: String,
    pub display_name: String,
    pub public_email_id: Option<Uuid>,
    pub public_email: Option<String>,
    pub emails: Vec<UserEmailAddress>,
    pub bio: String,
    pub pronouns: String,
    pub website_url: String,
    pub company: String,
    pub location: String,
    pub display_local_time: bool,
    pub time_zone: String,
    pub private_profile: bool,
    pub show_private_contribution_count: bool,
    pub achievements_enabled: bool,
    pub preferred_language: String,
    pub social_accounts: Vec<UserSocialAccount>,
    pub avatar: Option<UserAvatar>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceSettings {
    pub user_id: Uuid,
    pub theme: AppearanceTheme,
    pub font_size: AppearanceFontSize,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppearanceTheme {
    Light,
    Dark,
    System,
    DarkDimmed,
    DarkHighContrast,
}

impl AppearanceTheme {
    fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
            Self::DarkDimmed => "dark_dimmed",
            Self::DarkHighContrast => "dark_high_contrast",
        }
    }

    fn from_str(value: &str) -> Result<Self, PersonalSettingsError> {
        match value {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            "system" => Ok(Self::System),
            "dark_dimmed" => Ok(Self::DarkDimmed),
            "dark_high_contrast" => Ok(Self::DarkHighContrast),
            _ => Err(PersonalSettingsError::Validation(
                "Theme must be light, dark, system, dark_dimmed, or dark_high_contrast".to_owned(),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppearanceFontSize {
    Small,
    Default,
    Large,
}

impl AppearanceFontSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Default => "default",
            Self::Large => "large",
        }
    }

    fn from_str(value: &str) -> Result<Self, PersonalSettingsError> {
        match value {
            "small" => Ok(Self::Small),
            "default" => Ok(Self::Default),
            "large" => Ok(Self::Large),
            _ => Err(PersonalSettingsError::Validation(
                "Font size must be small, default, or large".to_owned(),
            )),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAppearanceSettings {
    pub theme: Option<AppearanceTheme>,
    pub font_size: Option<AppearanceFontSize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserEmailAddress {
    pub id: Uuid,
    pub email: String,
    pub is_primary: bool,
    pub is_public: bool,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserSocialAccount {
    pub provider: String,
    pub handle_or_url: String,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserAvatar {
    pub id: Uuid,
    pub url: String,
    pub content_type: String,
    pub byte_size: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePersonalProfileSettings {
    pub display_name: Option<String>,
    pub public_email_id: Option<Option<Uuid>>,
    pub bio: Option<String>,
    pub pronouns: Option<String>,
    pub website_url: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub display_local_time: Option<bool>,
    pub time_zone: Option<String>,
    pub private_profile: Option<bool>,
    pub show_private_contribution_count: Option<bool>,
    pub achievements_enabled: Option<bool>,
    pub preferred_language: Option<String>,
    pub social_accounts: Option<Vec<UserSocialAccountInput>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSocialAccountInput {
    pub provider: String,
    pub handle_or_url: String,
    pub position: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAvatarInput {
    pub action: AvatarAction,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub byte_size: Option<i32>,
    pub preview_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AvatarAction {
    Upload,
    Remove,
}

#[derive(Debug)]
pub enum PersonalSettingsError {
    Validation(String),
    EmailNotFound,
    Sqlx(sqlx::Error),
}

impl From<sqlx::Error> for PersonalSettingsError {
    fn from(error: sqlx::Error) -> Self {
        Self::Sqlx(error)
    }
}

pub async fn personal_profile_settings(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<PersonalProfileSettings, PersonalSettingsError> {
    ensure_primary_email(pool, user_id).await?;
    ensure_social_slots(pool, user_id).await?;

    let row = sqlx::query(
        r#"
        SELECT id, username, email, display_name, public_email_id, bio, pronouns, website_url,
               company, location, display_local_time, time_zone, private_profile,
               show_private_contribution_count, achievements_enabled, preferred_language,
               avatar_url, updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let emails = email_addresses(pool, user_id).await?;
    let public_email_id: Option<Uuid> = row.try_get("public_email_id")?;
    let public_email = public_email_id
        .and_then(|id| emails.iter().find(|email| email.id == id))
        .map(|email| email.email.clone());
    let social_accounts = social_accounts(pool, user_id).await?;
    let avatar = active_avatar(pool, user_id).await?;
    let fallback_login = fallback_login_from_email(row.try_get::<String, _>("email")?.as_str());

    Ok(PersonalProfileSettings {
        user_id: row.try_get("id")?,
        login: row
            .try_get::<Option<String>, _>("username")?
            .unwrap_or(fallback_login),
        display_name: row
            .try_get::<Option<String>, _>("display_name")?
            .unwrap_or_default(),
        public_email_id,
        public_email,
        emails,
        bio: row.try_get::<Option<String>, _>("bio")?.unwrap_or_default(),
        pronouns: row
            .try_get::<Option<String>, _>("pronouns")?
            .unwrap_or_default(),
        website_url: row
            .try_get::<Option<String>, _>("website_url")?
            .unwrap_or_default(),
        company: row
            .try_get::<Option<String>, _>("company")?
            .unwrap_or_default(),
        location: row
            .try_get::<Option<String>, _>("location")?
            .unwrap_or_default(),
        display_local_time: row.try_get("display_local_time")?,
        time_zone: row.try_get("time_zone")?,
        private_profile: row.try_get("private_profile")?,
        show_private_contribution_count: row.try_get("show_private_contribution_count")?,
        achievements_enabled: row.try_get("achievements_enabled")?,
        preferred_language: row.try_get("preferred_language")?,
        social_accounts,
        avatar,
        updated_at: row.try_get("updated_at")?,
    })
}

pub async fn appearance_settings(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<AppearanceSettings, PersonalSettingsError> {
    ensure_appearance_settings(pool, user_id).await?;
    load_appearance_settings(pool, user_id).await
}

pub async fn update_appearance_settings(
    pool: &PgPool,
    user_id: Uuid,
    input: UpdateAppearanceSettings,
) -> Result<AppearanceSettings, PersonalSettingsError> {
    ensure_appearance_settings(pool, user_id).await?;
    let theme = input.theme.map(AppearanceTheme::as_str);
    let font_size = input.font_size.map(AppearanceFontSize::as_str);

    sqlx::query(
        r#"
        UPDATE user_settings
        SET theme = COALESCE($2, theme),
            font_size = COALESCE($3, font_size)
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .bind(theme)
    .bind(font_size)
    .execute(pool)
    .await?;

    let settings = load_appearance_settings(pool, user_id).await?;
    audit(
        pool,
        user_id,
        "appearance.update",
        json!({
            "theme": settings.theme.as_str(),
            "fontSize": settings.font_size.as_str()
        }),
    )
    .await?;
    Ok(settings)
}

pub async fn update_personal_profile_settings(
    pool: &PgPool,
    user_id: Uuid,
    input: UpdatePersonalProfileSettings,
) -> Result<PersonalProfileSettings, PersonalSettingsError> {
    if let Some(Some(public_email_id)) = input.public_email_id {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM user_email_addresses WHERE id = $1 AND user_id = $2)",
        )
        .bind(public_email_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(PersonalSettingsError::EmailNotFound);
        }
    }

    let display_name = clean(input.display_name, 80)?;
    let bio = clean(input.bio, 280)?;
    let pronouns = clean(input.pronouns, 40)?;
    let website_url = clean(input.website_url, 240)?;
    let company = clean(input.company, 120)?;
    let location = clean(input.location, 120)?;
    let time_zone = clean_required(input.time_zone, 80, "Time zone")?;
    let preferred_language = clean_required(input.preferred_language, 40, "Preferred language")?;
    if let Some(url) = website_url.as_deref() {
        if !(url.is_empty() || url.starts_with("https://") || url.starts_with("http://")) {
            return Err(PersonalSettingsError::Validation(
                "URL must start with http:// or https://".to_owned(),
            ));
        }
    }

    let public_email_for_update = input.public_email_id.flatten();
    sqlx::query(
        r#"
        UPDATE users
        SET display_name = COALESCE($2, display_name),
            public_email_id = COALESCE($3, public_email_id),
            bio = COALESCE($4, bio),
            pronouns = COALESCE($5, pronouns),
            website_url = COALESCE($6, website_url),
            company = COALESCE($7, company),
            location = COALESCE($8, location),
            display_local_time = COALESCE($9, display_local_time),
            time_zone = COALESCE($10, time_zone),
            private_profile = COALESCE($11, private_profile),
            profile_visibility = CASE WHEN COALESCE($11, private_profile) THEN 'private' ELSE 'public' END,
            show_private_contribution_count = COALESCE($12, show_private_contribution_count),
            achievements_enabled = COALESCE($13, achievements_enabled),
            preferred_language = COALESCE($14, preferred_language)
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(display_name)
    .bind(public_email_for_update)
    .bind(bio)
    .bind(pronouns)
    .bind(website_url)
    .bind(company)
    .bind(location)
    .bind(input.display_local_time)
    .bind(time_zone)
    .bind(input.private_profile)
    .bind(input.show_private_contribution_count)
    .bind(input.achievements_enabled)
    .bind(preferred_language)
    .execute(pool)
    .await?;

    if let Some(accounts) = input.social_accounts {
        replace_social_accounts(pool, user_id, accounts).await?;
    }

    audit(
        pool,
        user_id,
        "profile.settings.update",
        json!({ "section": "public_profile" }),
    )
    .await?;
    personal_profile_settings(pool, user_id).await
}

async fn ensure_appearance_settings(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(), PersonalSettingsError> {
    sqlx::query(
        r#"
        INSERT INTO user_settings (user_id)
        VALUES ($1)
        ON CONFLICT (user_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn load_appearance_settings(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<AppearanceSettings, PersonalSettingsError> {
    let row = sqlx::query(
        r#"
        SELECT user_id, theme, font_size, updated_at
        FROM user_settings
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let theme = AppearanceTheme::from_str(row.try_get::<String, _>("theme")?.as_str())?;
    let font_size = AppearanceFontSize::from_str(row.try_get::<String, _>("font_size")?.as_str())?;

    Ok(AppearanceSettings {
        user_id: row.try_get("user_id")?,
        theme,
        font_size,
        updated_at: row.try_get("updated_at")?,
    })
}

pub async fn update_personal_avatar(
    pool: &PgPool,
    user_id: Uuid,
    input: UpdateAvatarInput,
) -> Result<PersonalProfileSettings, PersonalSettingsError> {
    match input.action {
        AvatarAction::Remove => {
            sqlx::query("UPDATE user_avatars SET active = false WHERE user_id = $1")
                .bind(user_id)
                .execute(pool)
                .await?;
            sqlx::query("UPDATE users SET avatar_url = NULL WHERE id = $1")
                .bind(user_id)
                .execute(pool)
                .await?;
            audit(pool, user_id, "profile.avatar.remove", json!({})).await?;
        }
        AvatarAction::Upload => {
            let content_type = input.content_type.unwrap_or_default();
            let byte_size = input.byte_size.unwrap_or_default();
            if !matches!(
                content_type.as_str(),
                "image/png" | "image/jpeg" | "image/webp" | "image/gif"
            ) {
                return Err(PersonalSettingsError::Validation(
                    "Avatar must be a PNG, JPEG, WebP, or GIF image".to_owned(),
                ));
            }
            if !(1..=MAX_AVATAR_BYTES).contains(&byte_size) {
                return Err(PersonalSettingsError::Validation(
                    "Avatar must be smaller than 2 MB".to_owned(),
                ));
            }
            let safe_name = input
                .file_name
                .unwrap_or_else(|| "avatar".to_owned())
                .chars()
                .map(|ch| {
                    if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                        ch
                    } else {
                        '-'
                    }
                })
                .collect::<String>();
            let s3_key = format!("avatars/{user_id}/{}-{safe_name}", Uuid::new_v4().simple());
            let public_url = input
                .preview_url
                .unwrap_or_else(|| format!("s3://opengithub-user-avatars/{s3_key}"));
            sqlx::query("UPDATE user_avatars SET active = false WHERE user_id = $1")
                .bind(user_id)
                .execute(pool)
                .await?;
            sqlx::query(
                r#"
                INSERT INTO user_avatars (user_id, s3_key, public_url, content_type, byte_size, active)
                VALUES ($1, $2, $3, $4, $5, true)
                "#,
            )
            .bind(user_id)
            .bind(&s3_key)
            .bind(&public_url)
            .bind(&content_type)
            .bind(byte_size)
            .execute(pool)
            .await?;
            sqlx::query("UPDATE users SET avatar_url = $2 WHERE id = $1")
                .bind(user_id)
                .bind(&public_url)
                .execute(pool)
                .await?;
            audit(
                pool,
                user_id,
                "profile.avatar.upload",
                json!({ "contentType": content_type, "byteSize": byte_size }),
            )
            .await?;
        }
    }

    personal_profile_settings(pool, user_id).await
}

async fn ensure_primary_email(pool: &PgPool, user_id: Uuid) -> Result<(), PersonalSettingsError> {
    sqlx::query(
        r#"
        INSERT INTO user_email_addresses (user_id, email, is_primary, is_public, verified_at)
        SELECT u.id, u.email, true, true, now()
        FROM users u
        WHERE u.id = $1
          AND NOT EXISTS (
              SELECT 1 FROM user_email_addresses e
              WHERE e.user_id = u.id AND lower(e.email) = lower(u.email)
          )
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        UPDATE users u
        SET public_email_id = e.id
        FROM user_email_addresses e
        WHERE u.id = $1 AND e.user_id = u.id AND e.is_primary = true AND u.public_email_id IS NULL
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn ensure_social_slots(pool: &PgPool, user_id: Uuid) -> Result<(), PersonalSettingsError> {
    for (index, provider) in SOCIAL_PROVIDERS.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO user_social_accounts (user_id, provider, handle_or_url, position)
            VALUES ($1, $2, '', $3)
            ON CONFLICT (user_id, position) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .bind((index + 1) as i32)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn email_addresses(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<UserEmailAddress>, PersonalSettingsError> {
    let rows = sqlx::query(
        r#"
        SELECT id, email, is_primary, is_public, verified_at IS NOT NULL AS verified
        FROM user_email_addresses
        WHERE user_id = $1
        ORDER BY is_primary DESC, email ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(UserEmailAddress {
                id: row.try_get("id")?,
                email: row.try_get("email")?,
                is_primary: row.try_get("is_primary")?,
                is_public: row.try_get("is_public")?,
                verified: row.try_get("verified")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(PersonalSettingsError::Sqlx)
}

async fn social_accounts(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<UserSocialAccount>, PersonalSettingsError> {
    let rows = sqlx::query(
        "SELECT provider, handle_or_url, position FROM user_social_accounts WHERE user_id = $1 ORDER BY position ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(UserSocialAccount {
                provider: row.try_get("provider")?,
                handle_or_url: row.try_get("handle_or_url")?,
                position: row.try_get("position")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(PersonalSettingsError::Sqlx)
}

async fn active_avatar(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<UserAvatar>, PersonalSettingsError> {
    let row = sqlx::query(
        r#"
        SELECT id, public_url, content_type, byte_size, created_at
        FROM user_avatars
        WHERE user_id = $1 AND active = true
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    row.map(|row| {
        Ok(UserAvatar {
            id: row.try_get("id")?,
            url: row.try_get("public_url")?,
            content_type: row.try_get("content_type")?,
            byte_size: row.try_get("byte_size")?,
            created_at: row.try_get("created_at")?,
        })
    })
    .transpose()
    .map_err(PersonalSettingsError::Sqlx)
}

async fn replace_social_accounts(
    pool: &PgPool,
    user_id: Uuid,
    accounts: Vec<UserSocialAccountInput>,
) -> Result<(), PersonalSettingsError> {
    if accounts.len() > 4 {
        return Err(PersonalSettingsError::Validation(
            "At most four social accounts can be saved".to_owned(),
        ));
    }
    for account in accounts {
        if !(1..=4).contains(&account.position) {
            return Err(PersonalSettingsError::Validation(
                "Social account position must be between 1 and 4".to_owned(),
            ));
        }
        let provider = account.provider.trim();
        if provider.is_empty() || provider.len() > 40 {
            return Err(PersonalSettingsError::Validation(
                "Social account provider is invalid".to_owned(),
            ));
        }
        let value = account.handle_or_url.trim();
        if value.len() > 240 {
            return Err(PersonalSettingsError::Validation(
                "Social account URL is too long".to_owned(),
            ));
        }
        sqlx::query(
            r#"
            INSERT INTO user_social_accounts (user_id, provider, handle_or_url, position)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, position)
            DO UPDATE SET provider = EXCLUDED.provider, handle_or_url = EXCLUDED.handle_or_url
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .bind(value)
        .bind(account.position)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn audit(
    pool: &PgPool,
    user_id: Uuid,
    event_type: &str,
    metadata: serde_json::Value,
) -> Result<(), PersonalSettingsError> {
    sqlx::query(
        r#"
        INSERT INTO security_audit_events (actor_user_id, event_type, target_id, metadata)
        VALUES ($1, $2, $1, $3)
        "#,
    )
    .bind(user_id)
    .bind(event_type)
    .bind(metadata)
    .execute(pool)
    .await?;
    Ok(())
}

fn clean(value: Option<String>, max: usize) -> Result<Option<String>, PersonalSettingsError> {
    value
        .map(|value| {
            let trimmed = value.trim().to_owned();
            if trimmed.len() > max {
                Err(PersonalSettingsError::Validation(format!(
                    "Value must be {max} characters or fewer"
                )))
            } else {
                Ok(trimmed)
            }
        })
        .transpose()
}

fn clean_required(
    value: Option<String>,
    max: usize,
    label: &str,
) -> Result<Option<String>, PersonalSettingsError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let trimmed = value.trim().to_owned();
    if trimmed.is_empty() {
        return Err(PersonalSettingsError::Validation(format!(
            "{label} is required"
        )));
    }
    if trimmed.len() > max {
        return Err(PersonalSettingsError::Validation(format!(
            "{label} must be {max} characters or fewer"
        )));
    }
    Ok(Some(trimmed))
}

fn fallback_login_from_email(email: &str) -> String {
    let local_part = email.split('@').next().unwrap_or("user");
    let normalized: String = local_part
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = normalized.trim_matches('-');
    if trimmed.is_empty() {
        "user".to_owned()
    } else {
        trimmed.to_owned()
    }
}
