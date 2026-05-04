use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub contact_email: String,
    pub ownership_type: OrganizationOwnershipType,
    pub company_name: Option<String>,
    #[serde(default)]
    pub terms_accepted: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OrganizationOwnershipType {
    Personal,
    Business,
}

impl OrganizationOwnershipType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Personal => "personal",
            Self::Business => "business",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationSlugAvailability {
    pub requested_name: String,
    pub normalized_slug: String,
    pub available: bool,
    pub reason: Option<String>,
    pub reserved: bool,
    pub existing_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreatedOrganization {
    pub id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub contact_email: String,
    pub ownership_type: String,
    pub company_name: Option<String>,
    pub terms_of_service_type: String,
    pub role: String,
    pub href: String,
    pub settings_href: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationProfileSettings {
    pub organization: OrganizationSettingsIdentity,
    pub profile: OrganizationProfileSettingsFields,
    pub social_accounts: Vec<OrganizationSocialAccount>,
    pub viewer_state: OrganizationSettingsViewerState,
    pub avatar: OrganizationAvatarSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationSettingsIdentity {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub href: String,
    pub settings_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationProfileSettingsFields {
    pub display_name: String,
    pub description: Option<String>,
    pub website_url: Option<String>,
    pub location: Option<String>,
    pub public_email: Option<String>,
    pub contact_email: Option<String>,
    pub billing_email: Option<String>,
    pub company_name: Option<String>,
    pub ownership_type: String,
    pub profile_visibility: String,
    pub public_members_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationSocialAccount {
    pub provider: String,
    pub value: String,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationSettingsViewerState {
    pub role: String,
    pub can_edit_profile: bool,
    pub can_rename: bool,
    pub can_archive: bool,
    pub can_delete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationAvatarSettings {
    pub avatar_url: Option<String>,
    pub storage_configured: bool,
    pub upload_available: bool,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationProfileSettingsPatch {
    pub display_name: Option<serde_json::Value>,
    pub description: Option<serde_json::Value>,
    pub website_url: Option<serde_json::Value>,
    pub location: Option<serde_json::Value>,
    pub public_email: Option<serde_json::Value>,
    pub contact_email: Option<serde_json::Value>,
    pub billing_email: Option<serde_json::Value>,
    pub company_name: Option<serde_json::Value>,
    pub social_accounts: Option<Vec<OrganizationSocialAccountInput>>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RenameOrganizationRequest {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationSocialAccountInput {
    pub provider: String,
    pub value: String,
}

#[derive(Debug, thiserror::Error)]
pub enum OrganizationSettingsError {
    #[error("organization settings were not found")]
    NotFound,
    #[error("organization settings require owner access")]
    Forbidden,
    #[error("{0}")]
    Validation(String),
    #[error("organization slug is already taken")]
    Conflict,
    #[error("database error")]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum OrganizationCreateError {
    #[error("{0}")]
    Validation(String),
    #[error("organization slug is reserved")]
    ReservedSlug,
    #[error("organization slug is already taken")]
    DuplicateSlug,
    #[error("database error")]
    Sqlx(#[from] sqlx::Error),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationPeopleList {
    #[serde(flatten)]
    pub envelope: ListEnvelope<OrganizationPeopleListItem>,
    pub mode: String,
    pub filters: OrganizationPeopleFilters,
    pub tab_counts: OrganizationTabCounts,
    pub viewer_state: OrganizationViewerState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationPeopleListItem {
    pub id: Uuid,
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub href: String,
    pub role: Option<String>,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationPeopleFilters {
    pub query: Option<String>,
    pub page: i64,
    pub page_size: i64,
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

#[derive(Debug, Clone, Copy)]
pub struct OrganizationPeopleListQuery<'a> {
    pub query: Option<&'a str>,
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

pub fn normalize_organization_slug(name: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;

    for character in name.trim().chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.push(character);
        } else {
            pending_dash = !slug.is_empty();
        }

        if slug.len() >= 39 {
            break;
        }
    }

    slug.trim_matches('-').to_owned()
}

pub fn validate_organization_slug(slug: &str) -> Result<(), String> {
    if slug.is_empty() {
        return Err("Organization name must include at least one letter or number.".to_owned());
    }
    if slug.len() > 39 {
        return Err("Organization slug must be 39 characters or fewer.".to_owned());
    }
    if !slug.chars().all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
    }) {
        return Err(
            "Organization slug may only use lowercase letters, numbers, and hyphens.".to_owned(),
        );
    }
    if slug.starts_with('-') || slug.ends_with('-') || slug.contains("--") {
        return Err("Organization slug cannot start, end, or repeat hyphens.".to_owned());
    }
    Ok(())
}

pub async fn organization_slug_availability(
    pool: &PgPool,
    requested_name: &str,
) -> Result<OrganizationSlugAvailability, OrganizationCreateError> {
    let normalized_slug = normalize_organization_slug(requested_name);
    let mut reason = validate_organization_slug(&normalized_slug).err();
    let mut reserved = false;
    let mut existing_kind = None;

    if reason.is_none() {
        reserved = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM reserved_slugs WHERE lower(slug) = lower($1))",
        )
        .bind(&normalized_slug)
        .fetch_one(pool)
        .await?;
        if reserved {
            reason = Some("This organization slug is reserved.".to_owned());
        }
    }

    if reason.is_none() {
        existing_kind = sqlx::query_scalar::<_, String>(
            r#"
            SELECT existing_kind FROM (
                SELECT 'organization'::text AS existing_kind
                FROM organizations
                WHERE lower(slug) = lower($1)
                UNION ALL
                SELECT 'user'::text AS existing_kind
                FROM users
                WHERE username IS NOT NULL AND lower(username) = lower($1)
            ) existing
            LIMIT 1
            "#,
        )
        .bind(&normalized_slug)
        .fetch_optional(pool)
        .await?;
        if existing_kind.is_some() {
            reason = Some("This organization slug is already taken.".to_owned());
        }
    }

    Ok(OrganizationSlugAvailability {
        requested_name: requested_name.to_owned(),
        normalized_slug,
        available: reason.is_none(),
        reason,
        reserved,
        existing_kind,
    })
}

pub async fn create_organization_from_signup(
    pool: &PgPool,
    actor_user_id: Uuid,
    request: CreateOrganizationRequest,
) -> Result<CreatedOrganization, OrganizationCreateError> {
    let display_name = normalize_display_name(&request.name)?;
    let availability = organization_slug_availability(pool, &request.name).await?;
    validate_organization_slug(&availability.normalized_slug)
        .map_err(OrganizationCreateError::Validation)?;
    if availability.reserved {
        return Err(OrganizationCreateError::ReservedSlug);
    }
    if availability.existing_kind.is_some() {
        return Err(OrganizationCreateError::DuplicateSlug);
    }

    let contact_email = normalize_contact_email(&request.contact_email)?;
    if !request.terms_accepted {
        return Err(OrganizationCreateError::Validation(
            "You must accept the organization terms before creating an organization.".to_owned(),
        ));
    }
    let company_name = normalize_company_name(request.ownership_type, request.company_name)?;
    let ownership_type = request.ownership_type.as_str();
    let terms_of_service_type = "free_organization_terms";
    let mut tx = pool.begin().await?;

    let row = sqlx::query(
        r#"
        INSERT INTO organizations (
            slug, display_name, owner_user_id, contact_email, terms_of_service_type,
            company_name, ownership_type
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, slug, display_name, contact_email, ownership_type, company_name,
                  terms_of_service_type, created_at
        "#,
    )
    .bind(&availability.normalized_slug)
    .bind(&display_name)
    .bind(actor_user_id)
    .bind(&contact_email)
    .bind(terms_of_service_type)
    .bind(&company_name)
    .bind(ownership_type)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_unique_slug_error)?;

    let organization_id = row.get::<Uuid, _>("id");
    sqlx::query(
        r#"
        INSERT INTO organization_memberships (organization_id, user_id, role)
        VALUES ($1, $2, 'owner')
        ON CONFLICT (organization_id, user_id) DO UPDATE SET role = 'owner'
        "#,
    )
    .bind(organization_id)
    .bind(actor_user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO organization_policy_settings (organization_id)
        VALUES ($1)
        ON CONFLICT (organization_id) DO NOTHING
        "#,
    )
    .bind(organization_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO organization_audit_events (
            organization_id, actor_user_id, event_type, metadata
        )
        VALUES ($1, $2, 'organization.create', $3)
        "#,
    )
    .bind(organization_id)
    .bind(actor_user_id)
    .bind(json!({
        "slug": availability.normalized_slug,
        "ownershipType": ownership_type,
        "termsOfServiceType": terms_of_service_type
    }))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let slug = row.get::<String, _>("slug");
    Ok(CreatedOrganization {
        id: organization_id,
        slug: slug.clone(),
        display_name: row.get("display_name"),
        contact_email: row.get("contact_email"),
        ownership_type: row.get("ownership_type"),
        company_name: row.get("company_name"),
        terms_of_service_type: row.get("terms_of_service_type"),
        role: "owner".to_owned(),
        href: format!("/orgs/{slug}"),
        settings_href: format!("/organizations/{slug}/settings/profile"),
        created_at: row.get("created_at"),
    })
}

pub async fn organization_profile_settings(
    pool: &PgPool,
    slug: &str,
    actor_user_id: Uuid,
) -> Result<OrganizationProfileSettings, OrganizationSettingsError> {
    let row = organization_settings_row(pool, slug)
        .await?
        .ok_or(OrganizationSettingsError::NotFound)?;
    ensure_organization_owner(pool, row.id, actor_user_id).await?;
    organization_profile_settings_from_row(pool, row, "owner".to_owned()).await
}

pub async fn update_organization_profile_settings(
    pool: &PgPool,
    slug: &str,
    actor_user_id: Uuid,
    patch: OrganizationProfileSettingsPatch,
) -> Result<OrganizationProfileSettings, OrganizationSettingsError> {
    let row = organization_settings_row(pool, slug)
        .await?
        .ok_or(OrganizationSettingsError::NotFound)?;
    ensure_organization_owner(pool, row.id, actor_user_id).await?;

    let display_name = patch_text_required(
        patch.display_name,
        row.display_name.clone(),
        "Organization display name",
        100,
    )?;
    let description = patch_text_optional(
        patch.description,
        row.description.clone(),
        "Description",
        280,
    )?;
    let website_url =
        patch_url_optional(patch.website_url, row.website_url.clone(), "Website URL")?;
    let location = patch_text_optional(patch.location, row.location.clone(), "Location", 120)?;
    let public_email =
        patch_email_optional(patch.public_email, row.public_email.clone(), "Public email")?;
    let contact_email = patch_email_optional(
        patch.contact_email,
        row.contact_email.clone(),
        "Contact email",
    )?;
    let billing_email = patch_email_optional(
        patch.billing_email,
        row.billing_email.clone(),
        "Billing email",
    )?;
    let company_name =
        patch_text_optional(patch.company_name, row.company_name.clone(), "Company", 120)?;
    let social_accounts = patch
        .social_accounts
        .map(normalize_social_accounts)
        .transpose()?;

    let mut changed_fields = Vec::new();
    if display_name != row.display_name {
        changed_fields.push("displayName");
    }
    if description != row.description {
        changed_fields.push("description");
    }
    if website_url != row.website_url {
        changed_fields.push("websiteUrl");
    }
    if location != row.location {
        changed_fields.push("location");
    }
    if public_email != row.public_email {
        changed_fields.push("publicEmail");
    }
    if contact_email != row.contact_email {
        changed_fields.push("contactEmail");
    }
    if billing_email != row.billing_email {
        changed_fields.push("billingEmail");
    }
    if company_name != row.company_name {
        changed_fields.push("companyName");
    }
    if social_accounts.is_some() {
        changed_fields.push("socialAccounts");
    }

    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE organizations
        SET display_name = $2,
            description = $3,
            website_url = $4,
            location = $5,
            public_email = $6,
            contact_email = $7,
            billing_email = $8,
            company_name = $9
        WHERE id = $1
        "#,
    )
    .bind(row.id)
    .bind(&display_name)
    .bind(&description)
    .bind(&website_url)
    .bind(&location)
    .bind(&public_email)
    .bind(&contact_email)
    .bind(&billing_email)
    .bind(&company_name)
    .execute(&mut *tx)
    .await?;

    if let Some(social_accounts) = &social_accounts {
        sqlx::query("DELETE FROM organization_social_accounts WHERE organization_id = $1")
            .bind(row.id)
            .execute(&mut *tx)
            .await?;
        for account in social_accounts {
            sqlx::query(
                r#"
                INSERT INTO organization_social_accounts (
                    organization_id, provider, value, position
                )
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(row.id)
            .bind(&account.provider)
            .bind(&account.value)
            .bind(account.position)
            .execute(&mut *tx)
            .await?;
        }
    }

    sqlx::query(
        r#"
        INSERT INTO organization_audit_events (
            organization_id, actor_user_id, event_type, metadata
        )
        VALUES ($1, $2, 'organization.profile_settings.update', $3)
        "#,
    )
    .bind(row.id)
    .bind(actor_user_id)
    .bind(json!({
        "slug": row.slug,
        "changedFields": changed_fields,
        "redacted": ["publicEmail", "contactEmail", "billingEmail", "socialAccounts"]
    }))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    organization_profile_settings(pool, slug, actor_user_id).await
}

pub async fn rename_organization(
    pool: &PgPool,
    slug: &str,
    actor_user_id: Uuid,
    request: RenameOrganizationRequest,
) -> Result<OrganizationProfileSettings, OrganizationSettingsError> {
    let row = organization_settings_row(pool, slug)
        .await?
        .ok_or(OrganizationSettingsError::NotFound)?;
    ensure_organization_owner(pool, row.id, actor_user_id).await?;

    let new_slug = normalize_organization_slug(&request.name);
    validate_organization_slug(&new_slug).map_err(OrganizationSettingsError::Validation)?;
    if new_slug.eq_ignore_ascii_case(&row.slug) {
        return Err(OrganizationSettingsError::Validation(
            "Choose a different organization slug before renaming.".to_owned(),
        ));
    }

    let availability = organization_slug_availability(pool, &request.name)
        .await
        .map_err(|error| match error {
            OrganizationCreateError::Validation(message) => {
                OrganizationSettingsError::Validation(message)
            }
            OrganizationCreateError::ReservedSlug => OrganizationSettingsError::Validation(
                "This organization slug is not available.".to_owned(),
            ),
            OrganizationCreateError::DuplicateSlug => OrganizationSettingsError::Conflict,
            OrganizationCreateError::Sqlx(error) => OrganizationSettingsError::Sqlx(error),
        })?;
    if availability.reserved {
        return Err(OrganizationSettingsError::Validation(
            "This organization slug is not available.".to_owned(),
        ));
    }
    if availability.existing_kind.is_some() {
        return Err(OrganizationSettingsError::Conflict);
    }

    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE organizations SET slug = $2 WHERE id = $1")
        .bind(row.id)
        .bind(&new_slug)
        .execute(&mut *tx)
        .await
        .map_err(map_settings_unique_slug_error)?;
    sqlx::query(
        r#"
        INSERT INTO organization_audit_events (
            organization_id, actor_user_id, event_type, metadata
        )
        VALUES ($1, $2, 'organization.rename', $3)
        "#,
    )
    .bind(row.id)
    .bind(actor_user_id)
    .bind(json!({
        "previousSlug": row.slug,
        "newSlug": new_slug,
        "redacted": []
    }))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    organization_profile_settings(pool, &new_slug, actor_user_id).await
}

fn normalize_display_name(name: &str) -> Result<String, OrganizationCreateError> {
    let display_name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if display_name.is_empty() {
        return Err(OrganizationCreateError::Validation(
            "Organization name is required.".to_owned(),
        ));
    }
    if display_name.chars().count() > 100 {
        return Err(OrganizationCreateError::Validation(
            "Organization name must be 100 characters or fewer.".to_owned(),
        ));
    }
    Ok(display_name)
}

fn normalize_contact_email(email: &str) -> Result<String, OrganizationCreateError> {
    let normalized = email.trim().to_ascii_lowercase();
    let valid = normalized.len() <= 254
        && normalized.split('@').count() == 2
        && normalized.split('@').nth(1).is_some_and(|domain| {
            domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
        });
    if !valid {
        return Err(OrganizationCreateError::Validation(
            "Enter a valid contact email address.".to_owned(),
        ));
    }
    Ok(normalized)
}

fn patch_text_required(
    value: Option<serde_json::Value>,
    current: String,
    label: &str,
    max_chars: usize,
) -> Result<String, OrganizationSettingsError> {
    let Some(value) = value else {
        return Ok(current);
    };
    let Some(raw) = value.as_str() else {
        return Err(OrganizationSettingsError::Validation(format!(
            "{label} must be a string."
        )));
    };
    let normalized = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return Err(OrganizationSettingsError::Validation(format!(
            "{label} is required."
        )));
    }
    if normalized.chars().count() > max_chars {
        return Err(OrganizationSettingsError::Validation(format!(
            "{label} must be {max_chars} characters or fewer."
        )));
    }
    Ok(normalized)
}

fn patch_text_optional(
    value: Option<serde_json::Value>,
    current: Option<String>,
    label: &str,
    max_chars: usize,
) -> Result<Option<String>, OrganizationSettingsError> {
    let Some(value) = value else {
        return Ok(current);
    };
    if value.is_null() {
        return Ok(None);
    }
    let Some(raw) = value.as_str() else {
        return Err(OrganizationSettingsError::Validation(format!(
            "{label} must be a string or null."
        )));
    };
    let normalized = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return Ok(None);
    }
    if normalized.chars().count() > max_chars {
        return Err(OrganizationSettingsError::Validation(format!(
            "{label} must be {max_chars} characters or fewer."
        )));
    }
    Ok(Some(normalized))
}

fn patch_email_optional(
    value: Option<serde_json::Value>,
    current: Option<String>,
    label: &str,
) -> Result<Option<String>, OrganizationSettingsError> {
    let normalized = patch_text_optional(value, current, label, 254)?;
    if let Some(email) = &normalized {
        validate_email(email).map_err(OrganizationSettingsError::Validation)?;
        Ok(Some(email.to_ascii_lowercase()))
    } else {
        Ok(None)
    }
}

fn patch_url_optional(
    value: Option<serde_json::Value>,
    current: Option<String>,
    label: &str,
) -> Result<Option<String>, OrganizationSettingsError> {
    let normalized = patch_text_optional(value, current, label, 2048)?;
    if let Some(url) = &normalized {
        let lower = url.to_ascii_lowercase();
        if !(lower.starts_with("https://") || lower.starts_with("http://")) {
            return Err(OrganizationSettingsError::Validation(format!(
                "{label} must start with http:// or https://."
            )));
        }
        if url.contains(char::is_whitespace) {
            return Err(OrganizationSettingsError::Validation(format!(
                "{label} must be a valid URL."
            )));
        }
    }
    Ok(normalized)
}

fn validate_email(email: &str) -> Result<(), String> {
    let valid = email.len() <= 254
        && email.split('@').count() == 2
        && email
            .split('@')
            .next()
            .is_some_and(|local| !local.is_empty())
        && email.split('@').nth(1).is_some_and(|domain| {
            domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
        });
    if valid {
        Ok(())
    } else {
        Err("Enter a valid email address.".to_owned())
    }
}

fn normalize_social_accounts(
    inputs: Vec<OrganizationSocialAccountInput>,
) -> Result<Vec<OrganizationSocialAccount>, OrganizationSettingsError> {
    if inputs.len() > 4 {
        return Err(OrganizationSettingsError::Validation(
            "Organizations can list at most four social accounts.".to_owned(),
        ));
    }

    let mut providers = std::collections::BTreeSet::new();
    let mut accounts = Vec::with_capacity(inputs.len());
    for (index, input) in inputs.into_iter().enumerate() {
        let provider = input.provider.trim().to_ascii_lowercase();
        if !matches!(provider.as_str(), "x" | "mastodon" | "linkedin" | "bluesky") {
            return Err(OrganizationSettingsError::Validation(format!(
                "Unsupported social provider: {provider}."
            )));
        }
        if !providers.insert(provider.clone()) {
            return Err(OrganizationSettingsError::Validation(format!(
                "Duplicate social provider: {provider}."
            )));
        }
        let value = input.value.split_whitespace().collect::<Vec<_>>().join(" ");
        if value.is_empty() {
            return Err(OrganizationSettingsError::Validation(
                "Social account values cannot be blank.".to_owned(),
            ));
        }
        if value.chars().count() > 120 {
            return Err(OrganizationSettingsError::Validation(
                "Social account values must be 120 characters or fewer.".to_owned(),
            ));
        }
        accounts.push(OrganizationSocialAccount {
            provider,
            value,
            position: i32::try_from(index + 1).unwrap_or(4),
        });
    }
    Ok(accounts)
}

fn normalize_company_name(
    ownership_type: OrganizationOwnershipType,
    company_name: Option<String>,
) -> Result<Option<String>, OrganizationCreateError> {
    let normalized = company_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    match ownership_type {
        OrganizationOwnershipType::Personal => Ok(None),
        OrganizationOwnershipType::Business => normalized
            .ok_or_else(|| {
                OrganizationCreateError::Validation(
                    "Company or institution name is required for business organizations."
                        .to_owned(),
                )
            })
            .map(Some),
    }
}

fn map_unique_slug_error(error: sqlx::Error) -> OrganizationCreateError {
    match &error {
        sqlx::Error::Database(database_error) if database_error.is_unique_violation() => {
            OrganizationCreateError::DuplicateSlug
        }
        _ => OrganizationCreateError::Sqlx(error),
    }
}

fn map_settings_unique_slug_error(error: sqlx::Error) -> OrganizationSettingsError {
    match &error {
        sqlx::Error::Database(database_error) if database_error.is_unique_violation() => {
            OrganizationSettingsError::Conflict
        }
        _ => OrganizationSettingsError::Sqlx(error),
    }
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

struct OrganizationSettingsRow {
    id: Uuid,
    slug: String,
    display_name: String,
    description: Option<String>,
    avatar_url: Option<String>,
    website_url: Option<String>,
    location: Option<String>,
    public_email: Option<String>,
    contact_email: Option<String>,
    billing_email: Option<String>,
    company_name: Option<String>,
    ownership_type: String,
    profile_visibility: String,
    public_members_visible: bool,
    avatar_s3_bucket: Option<String>,
    avatar_s3_key: Option<String>,
}

async fn organization_settings_row(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<OrganizationSettingsRow>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, slug, display_name, description, avatar_url, website_url, location,
               public_email, contact_email, billing_email, company_name, ownership_type,
               profile_visibility, public_members_visible, avatar_s3_bucket, avatar_s3_key
        FROM organizations
        WHERE lower(slug) = lower($1)
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| OrganizationSettingsRow {
        id: row.get("id"),
        slug: row.get("slug"),
        display_name: row.get("display_name"),
        description: row.get("description"),
        avatar_url: row.get("avatar_url"),
        website_url: row.get("website_url"),
        location: row.get("location"),
        public_email: row.get("public_email"),
        contact_email: row.get("contact_email"),
        billing_email: row.get("billing_email"),
        company_name: row.get("company_name"),
        ownership_type: row.get("ownership_type"),
        profile_visibility: row.get("profile_visibility"),
        public_members_visible: row.get("public_members_visible"),
        avatar_s3_bucket: row.get("avatar_s3_bucket"),
        avatar_s3_key: row.get("avatar_s3_key"),
    }))
}

async fn ensure_organization_owner(
    pool: &PgPool,
    organization_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), OrganizationSettingsError> {
    let role = viewer_role(pool, organization_id, Some(actor_user_id)).await?;
    if role.as_deref() == Some("owner") {
        Ok(())
    } else {
        Err(OrganizationSettingsError::Forbidden)
    }
}

async fn organization_profile_settings_from_row(
    pool: &PgPool,
    row: OrganizationSettingsRow,
    role: String,
) -> Result<OrganizationProfileSettings, OrganizationSettingsError> {
    let social_accounts = organization_social_accounts(pool, row.id).await?;
    let avatar_storage_configured = row.avatar_s3_bucket.is_some() && row.avatar_s3_key.is_some();
    let slug = row.slug;
    Ok(OrganizationProfileSettings {
        organization: OrganizationSettingsIdentity {
            id: row.id,
            slug: slug.clone(),
            name: row.display_name.clone(),
            href: format!("/orgs/{slug}"),
            settings_href: format!("/organizations/{slug}/settings/profile"),
        },
        profile: OrganizationProfileSettingsFields {
            display_name: row.display_name,
            description: row.description,
            website_url: row.website_url,
            location: row.location,
            public_email: row.public_email,
            contact_email: row.contact_email,
            billing_email: row.billing_email,
            company_name: row.company_name,
            ownership_type: row.ownership_type,
            profile_visibility: row.profile_visibility,
            public_members_visible: row.public_members_visible,
        },
        social_accounts,
        viewer_state: OrganizationSettingsViewerState {
            role,
            can_edit_profile: true,
            can_rename: true,
            can_archive: false,
            can_delete: false,
        },
        avatar: OrganizationAvatarSettings {
            avatar_url: row.avatar_url,
            storage_configured: avatar_storage_configured,
            upload_available: false,
            unavailable_reason: Some(
                "Organization avatar upload will be enabled after the S3 avatar pipeline is wired."
                    .to_owned(),
            ),
        },
    })
}

async fn organization_social_accounts(
    pool: &PgPool,
    organization_id: Uuid,
) -> Result<Vec<OrganizationSocialAccount>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT provider, value, position
        FROM organization_social_accounts
        WHERE organization_id = $1
        ORDER BY position ASC
        "#,
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| OrganizationSocialAccount {
            provider: row.get("provider"),
            value: row.get("value"),
            position: row.get("position"),
        })
        .collect())
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
    let mut filters = normalize_organization_repository_filters(query)?;
    let mut repositories =
        visible_organization_repository_rows(pool, &organization, viewer_user_id, &viewer_state)
            .await?;
    let visible_repository_total = repositories.len() as i64;
    let available_languages = organization_repository_language_options(&repositories);
    let available_types = organization_repository_type_options(&repositories);
    let people_count = visible_people_count(pool, &organization, is_member).await?;
    let packages = packages_count(pool, organization.id, is_member).await?;

    canonicalize_organization_repository_language(&mut filters, &available_languages);
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
            repositories: visible_repository_total,
            projects: 0,
            packages,
            people: people_count,
            sponsoring: 0,
        },
        viewer_state,
    })
}

pub async fn organization_people(
    pool: &PgPool,
    slug: &str,
    viewer_user_id: Option<Uuid>,
    query: OrganizationPeopleListQuery<'_>,
) -> Result<OrganizationPeopleList, OrganizationProfileError> {
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
    let filters = normalize_organization_people_filters(query);
    let visible_repository_ids =
        visible_repository_ids(pool, organization.id, viewer_user_id, is_member).await?;
    let people_count = visible_people_count(pool, &organization, is_member).await?;
    let packages = packages_count(pool, organization.id, is_member).await?;
    let mut people = visible_organization_people_rows(pool, &organization, is_member).await?;

    if let Some(query) = &filters.query {
        let needle = query.to_ascii_lowercase();
        people.retain(|person| {
            person.login.to_ascii_lowercase().contains(&needle)
                || person
                    .name
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&needle)
        });
    }

    let total = people.len() as i64;
    let offset = ((filters.page - 1) * filters.page_size) as usize;
    let limit = filters.page_size as usize;
    let items = people.into_iter().skip(offset).take(limit).collect();

    Ok(OrganizationPeopleList {
        envelope: ListEnvelope {
            items,
            total,
            page: filters.page,
            page_size: filters.page_size,
        },
        mode: "people".to_owned(),
        filters,
        tab_counts: OrganizationTabCounts {
            repositories: visible_repository_ids.len() as i64,
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

fn normalize_organization_people_filters(
    query: OrganizationPeopleListQuery<'_>,
) -> OrganizationPeopleFilters {
    let pagination = normalize_pagination(query.page, query.page_size);
    let normalized_query = query.query.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.chars().take(120).collect::<String>())
    });

    OrganizationPeopleFilters {
        query: normalized_query,
        page: pagination.page,
        page_size: pagination.page_size,
    }
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

async fn visible_organization_people_rows(
    pool: &PgPool,
    organization: &OrganizationRow,
    is_member: bool,
) -> Result<Vec<OrganizationPeopleListItem>, sqlx::Error> {
    if !is_member && !organization.public_members_visible {
        return Ok(Vec::new());
    }

    let rows = sqlx::query(
        r#"
        SELECT users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.display_name,
               users.avatar_url,
               organization_memberships.role,
               organization_memberships.created_at
        FROM organization_memberships
        JOIN users ON users.id = organization_memberships.user_id
        WHERE organization_memberships.organization_id = $1
        ORDER BY
            CASE organization_memberships.role
                WHEN 'owner' THEN 0
                WHEN 'admin' THEN 1
                ELSE 2
            END ASC,
            lower(COALESCE(NULLIF(users.display_name, ''), NULLIF(users.username, ''), users.email)) ASC,
            lower(COALESCE(NULLIF(users.username, ''), users.email)) ASC
        "#,
    )
    .bind(organization.id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let login: String = row.get("login");
            OrganizationPeopleListItem {
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
                joined_at: row.get("created_at"),
            }
        })
        .collect())
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

fn canonicalize_organization_repository_language(
    filters: &mut OrganizationRepositoryFilters,
    available_languages: &[OrganizationRepositoryFilterOption],
) {
    let Some(language) = filters.language.as_deref() else {
        return;
    };
    let Some(option) = available_languages
        .iter()
        .find(|option| option.value.eq_ignore_ascii_case(language))
    else {
        return;
    };
    filters.language = Some(option.value.clone());
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
