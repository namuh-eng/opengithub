use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use url::Url;
use uuid::Uuid;

use crate::jobs::{enqueue_job, JobLeaseError};

use super::{
    permissions::RepositoryRole,
    repositories::{
        can_admin_repository, can_read_repository, get_repository_by_owner_name, Repository,
        RepositoryError, RepositoryPolicyLock, RepositorySettingsAuditEvent, RepositoryVisibility,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum PagesError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error("invalid Pages setting: {0}")]
    Invalid(String),
    #[error("Pages domain is already configured")]
    Conflict,
    #[error("Pages site was not found")]
    NotFound,
    #[error("organization policy prevents Pages publishing")]
    PolicyLocked {
        field: String,
        reason: String,
        settings_href: String,
    },
    #[error(transparent)]
    Job(#[from] JobLeaseError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryPagesSettings {
    pub repository_id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub visibility: RepositoryVisibility,
    pub viewer_permission: String,
    pub can_edit: bool,
    pub site: PagesSiteSummary,
    pub available_refs: Vec<PagesBranchRef>,
    pub folder_options: Vec<PagesFolderOption>,
    pub workflow_suggestions: Vec<PagesWorkflowSuggestion>,
    pub deployments: Vec<PagesDeploymentSummary>,
    pub audit_events: Vec<RepositorySettingsAuditEvent>,
    pub policy_lock: Option<RepositoryPolicyLock>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PagesSiteSummary {
    pub id: Uuid,
    pub source: PagesSource,
    pub default_site_url: String,
    pub custom_domain: Option<String>,
    pub domain: PagesDomainState,
    pub https_enforced: bool,
    pub certificate_status: String,
    pub provisioning_status: String,
    pub cloudfront_alias: Option<String>,
    pub latest_deployment_id: Option<Uuid>,
    pub unpublished_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PagesSource {
    pub kind: PagesSourceKind,
    pub branch: Option<String>,
    pub folder: Option<String>,
    pub workflow_id: Option<Uuid>,
    pub workflow_artifact_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PagesSourceKind {
    None,
    Branch,
    Actions,
}

impl PagesSourceKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Branch => "branch",
            Self::Actions => "actions",
        }
    }
}

impl TryFrom<&str> for PagesSourceKind {
    type Error = PagesError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "none" => Ok(Self::None),
            "branch" => Ok(Self::Branch),
            "actions" => Ok(Self::Actions),
            other => Err(PagesError::Invalid(format!(
                "unsupported Pages source `{other}`"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PagesDomainState {
    pub status: String,
    pub challenge: Option<PagesDnsChallenge>,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PagesDnsChallenge {
    pub name: String,
    pub value: String,
    pub record_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PagesBranchRef {
    pub name: String,
    pub target_oid: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PagesFolderOption {
    pub value: String,
    pub label: String,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PagesWorkflowSuggestion {
    pub workflow_id: Uuid,
    pub name: String,
    pub path: String,
    pub artifact_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PagesDeploymentSummary {
    pub id: Uuid,
    pub source: PagesSource,
    pub status: String,
    pub conclusion: Option<String>,
    pub default_url: String,
    pub custom_domain_url: Option<String>,
    pub workflow_run_id: Option<Uuid>,
    pub workflow_artifact_id: Option<Uuid>,
    pub artifact_storage_key: Option<String>,
    pub artifact_manifest: serde_json::Value,
    pub build_log_excerpt: Option<String>,
    pub failure_reason: Option<String>,
    pub queued_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PagesSourceMutation {
    pub kind: PagesSourceKind,
    pub branch: Option<String>,
    pub folder: Option<String>,
    pub workflow_id: Option<Uuid>,
    pub workflow_artifact_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PagesDomainMutation {
    pub domain: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PagesHttpsMutation {
    pub enforced: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PagesActionsDeploymentMutation {
    pub workflow_run_id: Uuid,
    pub workflow_artifact_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PagesDeploymentRequestResult {
    pub settings: RepositoryPagesSettings,
    pub deployment: PagesDeploymentSummary,
}

pub async fn repository_pages_settings_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryPagesSettings>, PagesError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        return Err(PagesError::Repository(RepositoryError::PermissionDenied));
    }
    let can_edit = can_admin_repository(pool, &repository, actor_user_id).await?;
    ensure_pages_site(pool, &repository, actor_user_id).await?;
    repository_pages_settings_for_repository(pool, &repository, actor_user_id, can_edit).await
}

pub async fn update_repository_pages_source_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    mutation: PagesSourceMutation,
) -> Result<Option<RepositoryPagesSettings>, PagesError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_pages_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let source = normalize_source_mutation(pool, &repository, mutation).await?;
    enforce_pages_publishing_policy(pool, &repository, actor_user_id, &source.kind).await?;
    let site = ensure_pages_site(pool, &repository, actor_user_id).await?;
    let before = pages_site_audit_state(pool, site.id).await?;

    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE pages_sites
        SET source_kind = $3,
            source_branch = $4,
            source_folder = $5,
            workflow_id = $6,
            workflow_artifact_name = $7,
            provisioning_status = CASE WHEN $3 = 'none' THEN 'not_configured' ELSE 'queued' END,
            unpublished_at = CASE WHEN $3 = 'none' THEN now() ELSE NULL END,
            updated_by_user_id = $2
        WHERE id = $1
        "#,
    )
    .bind(site.id)
    .bind(actor_user_id)
    .bind(source.kind.as_str())
    .bind(&source.branch)
    .bind(&source.folder)
    .bind(source.workflow_id)
    .bind(&source.workflow_artifact_name)
    .execute(&mut *tx)
    .await?;
    insert_pages_audit_tx(
        &mut tx,
        repository.id,
        actor_user_id,
        "repository.pages.source.update",
        vec!["pages.source".to_owned()],
        before,
        json!({
            "sourceKind": source.kind.as_str(),
            "branch": source.branch,
            "folder": source.folder,
            "workflowId": source.workflow_id,
            "workflowArtifactName": source.workflow_artifact_name,
        }),
    )
    .await?;
    tx.commit().await?;

    if source.kind == PagesSourceKind::Branch {
        create_pages_deployment(
            pool,
            &repository,
            site.id,
            actor_user_id,
            &source,
            None,
            None,
        )
        .await?;
    }
    repository_pages_settings_for_repository(pool, &repository, actor_user_id, true).await
}

pub async fn save_repository_pages_domain_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    mutation: PagesDomainMutation,
) -> Result<Option<RepositoryPagesSettings>, PagesError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_pages_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let domain = normalize_custom_domain(&mutation.domain)?;
    ensure_domain_available(pool, repository.id, &domain).await?;
    let site = ensure_pages_site(pool, &repository, actor_user_id).await?;
    let before = pages_site_audit_state(pool, site.id).await?;
    let challenge_name = format!("_opengithub-pages.{domain}");
    let challenge_value = format!("og-pages-{}", Uuid::new_v4().simple());

    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE pages_sites
        SET custom_domain = $3,
            dns_challenge_name = $4,
            dns_challenge_value = $5,
            dns_status = 'pending',
            certificate_status = 'pending',
            https_enforced = false,
            updated_by_user_id = $2
        WHERE id = $1
        "#,
    )
    .bind(site.id)
    .bind(actor_user_id)
    .bind(&domain)
    .bind(&challenge_name)
    .bind(&challenge_value)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO pages_domain_verifications (site_id, domain, challenge_name, challenge_value, status)
        VALUES ($1, $2, $3, $4, 'pending')
        "#,
    )
    .bind(site.id)
    .bind(&domain)
    .bind(&challenge_name)
    .bind(&challenge_value)
    .execute(&mut *tx)
    .await?;
    insert_pages_audit_tx(
        &mut tx,
        repository.id,
        actor_user_id,
        "repository.pages.domain.save",
        vec!["pages.domain".to_owned()],
        before,
        json!({ "customDomain": domain, "dnsStatus": "pending" }),
    )
    .await?;
    tx.commit().await?;

    repository_pages_settings_for_repository(pool, &repository, actor_user_id, true).await
}

pub async fn remove_repository_pages_domain_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryPagesSettings>, PagesError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_pages_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let site = ensure_pages_site(pool, &repository, actor_user_id).await?;
    let before = pages_site_audit_state(pool, site.id).await?;

    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE pages_sites
        SET custom_domain = NULL,
            dns_challenge_name = NULL,
            dns_challenge_value = NULL,
            dns_status = 'not_configured',
            https_enforced = false,
            certificate_status = 'not_configured',
            cloudfront_alias = NULL,
            updated_by_user_id = $2
        WHERE id = $1
        "#,
    )
    .bind(site.id)
    .bind(actor_user_id)
    .execute(&mut *tx)
    .await?;
    insert_pages_audit_tx(
        &mut tx,
        repository.id,
        actor_user_id,
        "repository.pages.domain.remove",
        vec!["pages.domain".to_owned(), "pages.https".to_owned()],
        before,
        json!({}),
    )
    .await?;
    tx.commit().await?;

    repository_pages_settings_for_repository(pool, &repository, actor_user_id, true).await
}

pub async fn recheck_repository_pages_dns_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryPagesSettings>, PagesError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_pages_admin(pool, &repository, actor_user_id).await?;
    let site = ensure_pages_site(pool, &repository, actor_user_id).await?;
    let before = pages_site_audit_state(pool, site.id).await?;
    let has_domain = sqlx::query_scalar::<_, bool>(
        "SELECT custom_domain IS NOT NULL FROM pages_sites WHERE id = $1",
    )
    .bind(site.id)
    .fetch_one(pool)
    .await?;
    if !has_domain {
        return Err(PagesError::Invalid(
            "custom domain must be configured before DNS can be checked".to_owned(),
        ));
    }
    let verified = std::env::var("PAGES_DNS_VERIFICATION_MODE")
        .ok()
        .is_some_and(|value| value == "verified");
    let status = if verified { "verified" } else { "pending" };
    let certificate_status = if verified { "issued" } else { "pending" };

    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE pages_sites
        SET dns_status = $2,
            certificate_status = $3,
            updated_by_user_id = $4
        WHERE id = $1
        "#,
    )
    .bind(site.id)
    .bind(status)
    .bind(certificate_status)
    .bind(actor_user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        UPDATE pages_domain_verifications
        SET status = $2,
            checked_at = now(),
            last_error = CASE WHEN $2 = 'verified' THEN NULL ELSE 'DNS challenge has not propagated yet' END
        WHERE site_id = $1
          AND id = (
              SELECT id FROM pages_domain_verifications
              WHERE site_id = $1
              ORDER BY created_at DESC
              LIMIT 1
          )
        "#,
    )
    .bind(site.id)
    .bind(status)
    .execute(&mut *tx)
    .await?;
    insert_pages_audit_tx(
        &mut tx,
        repository.id,
        actor_user_id,
        "repository.pages.domain.recheck",
        vec!["pages.domain.dns".to_owned()],
        before,
        json!({ "dnsStatus": status, "certificateStatus": certificate_status }),
    )
    .await?;
    tx.commit().await?;

    repository_pages_settings_for_repository(pool, &repository, actor_user_id, true).await
}

pub async fn update_repository_pages_https_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    mutation: PagesHttpsMutation,
) -> Result<Option<RepositoryPagesSettings>, PagesError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_pages_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let site = ensure_pages_site(pool, &repository, actor_user_id).await?;
    let row = sqlx::query(
        "SELECT custom_domain, dns_status, certificate_status FROM pages_sites WHERE id = $1",
    )
    .bind(site.id)
    .fetch_one(pool)
    .await?;
    if mutation.enforced {
        let custom_domain: Option<String> = row.try_get("custom_domain")?;
        let dns_status: String = row.try_get("dns_status")?;
        let certificate_status: String = row.try_get("certificate_status")?;
        if custom_domain.is_none() || dns_status != "verified" || certificate_status != "issued" {
            return Err(PagesError::Invalid(
                "HTTPS enforcement requires a verified custom domain and issued certificate"
                    .to_owned(),
            ));
        }
    }
    let before = pages_site_audit_state(pool, site.id).await?;
    let mut tx = pool.begin().await?;
    sqlx::query(
        "UPDATE pages_sites SET https_enforced = $2, updated_by_user_id = $3 WHERE id = $1",
    )
    .bind(site.id)
    .bind(mutation.enforced)
    .bind(actor_user_id)
    .execute(&mut *tx)
    .await?;
    insert_pages_audit_tx(
        &mut tx,
        repository.id,
        actor_user_id,
        "repository.pages.https.update",
        vec!["pages.https".to_owned()],
        before,
        json!({ "httpsEnforced": mutation.enforced }),
    )
    .await?;
    tx.commit().await?;

    repository_pages_settings_for_repository(pool, &repository, actor_user_id, true).await
}

pub async fn request_repository_pages_deployment_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<PagesDeploymentRequestResult>, PagesError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_pages_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let site = ensure_pages_site(pool, &repository, actor_user_id).await?;
    let row = sqlx::query(
        "SELECT source_kind, source_branch, source_folder, workflow_id, workflow_artifact_name FROM pages_sites WHERE id = $1",
    )
    .bind(site.id)
    .fetch_one(pool)
    .await?;
    let source = source_from_row(&row)?;
    if source.kind == PagesSourceKind::None {
        return Err(PagesError::Invalid(
            "Pages source must be configured before deployment".to_owned(),
        ));
    }
    if source.kind == PagesSourceKind::Actions {
        return Err(PagesError::Invalid(
            "Actions source deployments must be linked to a workflow artifact".to_owned(),
        ));
    }
    let deployment = create_pages_deployment(
        pool,
        &repository,
        site.id,
        actor_user_id,
        &source,
        None,
        None,
    )
    .await?;
    let settings = repository_pages_settings_for_repository(pool, &repository, actor_user_id, true)
        .await?
        .ok_or(PagesError::NotFound)?;
    Ok(Some(PagesDeploymentRequestResult {
        settings,
        deployment,
    }))
}

pub async fn connect_repository_pages_actions_deployment_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    mutation: PagesActionsDeploymentMutation,
) -> Result<Option<PagesDeploymentRequestResult>, PagesError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_pages_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let site = ensure_pages_site(pool, &repository, actor_user_id).await?;
    let source = actions_source_for_artifact(pool, repository.id, mutation).await?;
    let deployment = create_pages_deployment(
        pool,
        &repository,
        site.id,
        actor_user_id,
        &source,
        source.workflow_id,
        source.workflow_artifact_name.clone(),
    )
    .await?;
    let settings = repository_pages_settings_for_repository(pool, &repository, actor_user_id, true)
        .await?
        .ok_or(PagesError::NotFound)?;
    Ok(Some(PagesDeploymentRequestResult {
        settings,
        deployment,
    }))
}

pub async fn unpublish_repository_pages_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositoryPagesSettings>, PagesError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    require_pages_admin(pool, &repository, actor_user_id).await?;
    ensure_repository_mutable(&repository)?;
    let site = ensure_pages_site(pool, &repository, actor_user_id).await?;
    let before = pages_site_audit_state(pool, site.id).await?;

    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE pages_sites
        SET source_kind = 'none',
            source_branch = NULL,
            source_folder = NULL,
            workflow_id = NULL,
            workflow_artifact_name = NULL,
            provisioning_status = 'unpublished',
            https_enforced = false,
            cloudfront_distribution_id = NULL,
            cloudfront_alias = NULL,
            s3_artifact_prefix = NULL,
            unpublished_at = now(),
            updated_by_user_id = $2
        WHERE id = $1
        "#,
    )
    .bind(site.id)
    .bind(actor_user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO pages_deployments (
            repository_id, site_id, source_kind, status, conclusion, default_url,
            custom_domain_url, requested_by_user_id, completed_at
        )
        SELECT repository_id, id, 'branch', 'unpublished', 'success', default_site_url,
               CASE WHEN custom_domain IS NULL THEN NULL ELSE 'https://' || custom_domain END,
               $2, now()
        FROM pages_sites
        WHERE id = $1
        "#,
    )
    .bind(site.id)
    .bind(actor_user_id)
    .execute(&mut *tx)
    .await?;
    insert_pages_audit_tx(
        &mut tx,
        repository.id,
        actor_user_id,
        "repository.pages.unpublish",
        vec!["pages.source".to_owned(), "pages.publication".to_owned()],
        before,
        json!({ "provisioningStatus": "unpublished" }),
    )
    .await?;
    tx.commit().await?;

    repository_pages_settings_for_repository(pool, &repository, actor_user_id, true).await
}

async fn repository_pages_settings_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    can_edit: bool,
) -> Result<Option<RepositoryPagesSettings>, PagesError> {
    let Some(row) = sqlx::query(
        r#"
        SELECT pages_sites.id,
               pages_sites.source_kind,
               pages_sites.source_branch,
               pages_sites.source_folder,
               pages_sites.workflow_id,
               pages_sites.workflow_artifact_name,
               pages_sites.default_site_url,
               pages_sites.custom_domain,
               pages_sites.dns_challenge_name,
               pages_sites.dns_challenge_value,
               pages_sites.dns_status,
               pages_sites.https_enforced,
               pages_sites.certificate_status,
               pages_sites.provisioning_status,
               pages_sites.cloudfront_alias,
               pages_sites.last_deployment_id,
               pages_sites.unpublished_at,
               pages_sites.updated_at,
               repositories.visibility,
               repository_permissions.role AS viewer_permission
        FROM pages_sites
        JOIN repositories ON repositories.id = pages_sites.repository_id
        LEFT JOIN repository_permissions
          ON repository_permissions.repository_id = repositories.id
         AND repository_permissions.user_id = $2
        WHERE pages_sites.repository_id = $1
        "#,
    )
    .bind(repository.id)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let viewer_permission =
        viewer_permission(repository, actor_user_id, row.try_get("viewer_permission")?);
    let site = pages_site_from_row(&row, can_edit)?;
    let latest_commit_id =
        branch_commit_id(pool, repository.id, site.source.branch.as_deref()).await?;
    let available_refs = pages_branch_refs(pool, repository.id).await?;
    let folder_options = pages_folder_options(pool, repository.id, latest_commit_id).await?;
    let workflow_suggestions = pages_workflow_suggestions(pool, repository.id).await?;
    let deployments = pages_deployments(pool, repository.id).await?;
    let audit_events = pages_audit_events(pool, repository.id).await?;
    let policy_lock = pages_policy_lock(pool, repository, actor_user_id).await?;

    Ok(Some(RepositoryPagesSettings {
        repository_id: repository.id,
        owner_login: repository.owner_login.clone(),
        name: repository.name.clone(),
        visibility: RepositoryVisibility::try_from(
            row.try_get::<String, _>("visibility")?.as_str(),
        )?,
        viewer_permission,
        can_edit,
        site,
        available_refs,
        folder_options,
        workflow_suggestions,
        deployments,
        audit_events,
        policy_lock,
    }))
}

async fn require_pages_admin(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<(), PagesError> {
    if can_admin_repository(pool, repository, actor_user_id).await? {
        Ok(())
    } else {
        Err(PagesError::Repository(RepositoryError::PermissionDenied))
    }
}

fn ensure_repository_mutable(repository: &Repository) -> Result<(), PagesError> {
    if repository.is_archived {
        Err(PagesError::Repository(
            RepositoryError::ArchivedRepositoryReadOnly,
        ))
    } else {
        Ok(())
    }
}

async fn enforce_pages_publishing_policy(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    source_kind: &PagesSourceKind,
) -> Result<(), PagesError> {
    if source_kind == &PagesSourceKind::None {
        return Ok(());
    }
    if let Some(lock) = pages_policy_lock(pool, repository, actor_user_id).await? {
        return Err(PagesError::PolicyLocked {
            field: lock.field,
            reason: lock.reason,
            settings_href: lock.settings_href,
        });
    }
    Ok(())
}

async fn pages_policy_lock(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<Option<RepositoryPolicyLock>, PagesError> {
    let Some(organization_id) = repository.owner_organization_id else {
        return Ok(None);
    };
    let role = sqlx::query_scalar::<_, String>(
        r#"
        SELECT role
        FROM organization_memberships
        WHERE organization_id = $1 AND user_id = $2
        "#,
    )
    .bind(organization_id)
    .bind(actor_user_id)
    .fetch_optional(pool)
    .await?;
    if role
        .as_deref()
        .is_some_and(|role| matches!(role, "owner" | "admin"))
    {
        return Ok(None);
    }

    let Some(row) = sqlx::query(
        r#"
        SELECT organizations.slug,
               COALESCE(organization_policy_settings.pages_public_publishing, true) AS pages_public_publishing,
               COALESCE(organization_policy_settings.pages_private_publishing, true) AS pages_private_publishing
        FROM organizations
        LEFT JOIN organization_policy_settings
          ON organization_policy_settings.organization_id = organizations.id
        WHERE organizations.id = $1
        "#,
    )
    .bind(organization_id)
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };

    let denied = match repository.visibility {
        RepositoryVisibility::Public => !row.try_get::<bool, _>("pages_public_publishing")?,
        RepositoryVisibility::Private | RepositoryVisibility::Internal => {
            !row.try_get::<bool, _>("pages_private_publishing")?
        }
    };
    if !denied {
        return Ok(None);
    }
    let slug: String = row.try_get("slug")?;
    let field = if repository.visibility == RepositoryVisibility::Public {
        "pagesPublicPublishing"
    } else {
        "pagesPrivatePublishing"
    };
    Ok(Some(RepositoryPolicyLock {
        field: field.to_owned(),
        reason: format!(
            "Organization policy prevents Pages publishing for {} repositories.",
            repository.visibility.as_str()
        ),
        settings_href: format!("/organizations/{slug}/settings/member_privileges"),
    }))
}

async fn ensure_pages_site(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<PagesSiteRow, PagesError> {
    let default_url = default_pages_url(repository);
    let row = sqlx::query(
        r#"
        INSERT INTO pages_sites (repository_id, default_site_url, created_by_user_id, updated_by_user_id)
        VALUES ($1, $2, $3, $3)
        ON CONFLICT (repository_id) DO UPDATE SET default_site_url = EXCLUDED.default_site_url
        RETURNING id, repository_id
        "#,
    )
    .bind(repository.id)
    .bind(default_url)
    .bind(actor_user_id)
    .fetch_one(pool)
    .await?;
    Ok(PagesSiteRow {
        id: row.try_get("id")?,
    })
}

async fn normalize_source_mutation(
    pool: &PgPool,
    repository: &Repository,
    mutation: PagesSourceMutation,
) -> Result<PagesSource, PagesError> {
    match mutation.kind {
        PagesSourceKind::None => Ok(PagesSource {
            kind: PagesSourceKind::None,
            branch: None,
            folder: None,
            workflow_id: None,
            workflow_artifact_name: None,
        }),
        PagesSourceKind::Branch => {
            let branch = mutation
                .branch
                .as_deref()
                .map(normalize_branch_name)
                .transpose()?
                .ok_or_else(|| PagesError::Invalid("branch source requires a branch".to_owned()))?;
            let folder = normalize_pages_folder(mutation.folder.as_deref())?;
            let commit_id = branch_commit_id(pool, repository.id, Some(&branch))
                .await?
                .ok_or_else(|| PagesError::Invalid(format!("branch `{branch}` was not found")))?;
            if folder == "/docs"
                && !folder_exists_at_commit(pool, repository.id, commit_id, "docs").await?
            {
                return Err(PagesError::Invalid(
                    "selected branch does not contain a /docs folder".to_owned(),
                ));
            }
            Ok(PagesSource {
                kind: PagesSourceKind::Branch,
                branch: Some(branch),
                folder: Some(folder),
                workflow_id: None,
                workflow_artifact_name: None,
            })
        }
        PagesSourceKind::Actions => {
            let workflow_id = mutation.workflow_id.ok_or_else(|| {
                PagesError::Invalid("Actions source requires a workflow".to_owned())
            })?;
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS (SELECT 1 FROM actions_workflows WHERE repository_id = $1 AND id = $2)",
            )
            .bind(repository.id)
            .bind(workflow_id)
            .fetch_one(pool)
            .await?;
            if !exists {
                return Err(PagesError::Invalid(
                    "selected workflow does not belong to this repository".to_owned(),
                ));
            }
            let artifact = mutation
                .workflow_artifact_name
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("github-pages")
                .to_owned();
            Ok(PagesSource {
                kind: PagesSourceKind::Actions,
                branch: None,
                folder: None,
                workflow_id: Some(workflow_id),
                workflow_artifact_name: Some(artifact),
            })
        }
    }
}

async fn actions_source_for_artifact(
    pool: &PgPool,
    repository_id: Uuid,
    mutation: PagesActionsDeploymentMutation,
) -> Result<PagesSource, PagesError> {
    let row = sqlx::query(
        r#"
        SELECT workflow_runs.workflow_id, workflow_artifacts.name
        FROM workflow_artifacts
        JOIN workflow_runs ON workflow_runs.id = workflow_artifacts.run_id
        WHERE workflow_runs.repository_id = $1
          AND workflow_runs.id = $2
          AND workflow_artifacts.id = $3
          AND workflow_runs.status = 'completed'
          AND workflow_runs.conclusion = 'success'
          AND workflow_artifacts.expired_at IS NULL
        "#,
    )
    .bind(repository_id)
    .bind(mutation.workflow_run_id)
    .bind(mutation.workflow_artifact_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        PagesError::Invalid("Pages artifact was not found or is not deployable".to_owned())
    })?;
    Ok(PagesSource {
        kind: PagesSourceKind::Actions,
        branch: None,
        folder: None,
        workflow_id: row.try_get("workflow_id")?,
        workflow_artifact_name: row.try_get("name")?,
    })
}

async fn create_pages_deployment(
    pool: &PgPool,
    repository: &Repository,
    site_id: Uuid,
    actor_user_id: Uuid,
    source: &PagesSource,
    workflow_run_id: Option<Uuid>,
    workflow_artifact_name: Option<String>,
) -> Result<PagesDeploymentSummary, PagesError> {
    let commit_id = if source.kind == PagesSourceKind::Branch {
        branch_commit_id(pool, repository.id, source.branch.as_deref()).await?
    } else {
        None
    };
    let workflow_artifact_id = if let (Some(run_id), Some(artifact_name)) =
        (workflow_run_id, workflow_artifact_name.as_deref())
    {
        sqlx::query_scalar::<_, Option<Uuid>>(
            "SELECT id FROM workflow_artifacts WHERE run_id = $1 AND lower(name) = lower($2)",
        )
        .bind(run_id)
        .bind(artifact_name)
        .fetch_one(pool)
        .await?
    } else {
        None
    };
    let site = pages_site_audit_state(pool, site_id).await?;
    let custom_domain_url = site
        .get("customDomain")
        .and_then(|value| value.as_str())
        .map(|domain| format!("https://{domain}"));
    let default_url = default_pages_url(repository);
    let mut tx = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO pages_deployments (
            repository_id, site_id, source_kind, source_branch, source_folder, commit_id,
            workflow_run_id, workflow_artifact_id, default_url, custom_domain_url,
            requested_by_user_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, source_kind, source_branch, source_folder, status, conclusion,
                  default_url, custom_domain_url, workflow_run_id, workflow_artifact_id,
                  artifact_storage_key, artifact_manifest, build_log_excerpt, failure_reason,
                  queued_at, completed_at, created_at
        "#,
    )
    .bind(repository.id)
    .bind(site_id)
    .bind(source.kind.as_str())
    .bind(&source.branch)
    .bind(&source.folder)
    .bind(commit_id)
    .bind(workflow_run_id)
    .bind(workflow_artifact_id)
    .bind(&default_url)
    .bind(&custom_domain_url)
    .bind(actor_user_id)
    .fetch_one(&mut *tx)
    .await?;
    let deployment_id: Uuid = row.try_get("id")?;
    sqlx::query("UPDATE pages_sites SET last_deployment_id = $2, provisioning_status = 'queued' WHERE id = $1")
        .bind(site_id)
        .bind(deployment_id)
        .execute(&mut *tx)
        .await?;
    insert_pages_audit_tx(
        &mut tx,
        repository.id,
        actor_user_id,
        "repository.pages.deploy.request",
        vec!["pages.deployments".to_owned()],
        json!({}),
        json!({ "deploymentId": deployment_id, "sourceKind": source.kind.as_str() }),
    )
    .await?;
    tx.commit().await?;
    enqueue_job(
        pool,
        "pages-build-deploy",
        &deployment_id.to_string(),
        json!({
            "repositoryId": repository.id,
            "siteId": site_id,
            "deploymentId": deployment_id,
            "source": source,
        }),
    )
    .await?;
    deployment_from_row(row)
}

async fn branch_commit_id(
    pool: &PgPool,
    repository_id: Uuid,
    branch: Option<&str>,
) -> Result<Option<Uuid>, PagesError> {
    let Some(branch) = branch else {
        return Ok(None);
    };
    let qualified = format!("refs/heads/{}", normalize_branch_name(branch)?);
    let id = sqlx::query_scalar::<_, Option<Uuid>>(
        r#"
        SELECT target_commit_id
        FROM repository_git_refs
        WHERE repository_id = $1
          AND kind = 'branch'
          AND (name = $2 OR name = $3)
        LIMIT 1
        "#,
    )
    .bind(repository_id)
    .bind(&qualified)
    .bind(branch)
    .fetch_optional(pool)
    .await?
    .flatten();
    Ok(id)
}

async fn folder_exists_at_commit(
    pool: &PgPool,
    repository_id: Uuid,
    commit_id: Uuid,
    folder: &str,
) -> Result<bool, PagesError> {
    let prefix = format!("{folder}/%");
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM repository_files
            WHERE repository_id = $1
              AND commit_id = $2
              AND (path = $3 OR path LIKE $4)
        )
        "#,
    )
    .bind(repository_id)
    .bind(commit_id)
    .bind(folder)
    .bind(prefix)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}

async fn pages_branch_refs(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<PagesBranchRef>, PagesError> {
    let rows = sqlx::query(
        r#"
        SELECT repository_git_refs.name, commits.oid AS target_oid, repository_git_refs.updated_at
        FROM repository_git_refs
        LEFT JOIN commits ON commits.id = repository_git_refs.target_commit_id
        WHERE repository_git_refs.repository_id = $1 AND repository_git_refs.kind = 'branch'
        ORDER BY lower(repository_git_refs.name)
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let name: String = row.try_get("name")?;
            Ok(PagesBranchRef {
                name: name.strip_prefix("refs/heads/").unwrap_or(&name).to_owned(),
                target_oid: row.try_get("target_oid")?,
                updated_at: row.try_get("updated_at")?,
            })
        })
        .collect()
}

async fn pages_folder_options(
    pool: &PgPool,
    repository_id: Uuid,
    commit_id: Option<Uuid>,
) -> Result<Vec<PagesFolderOption>, PagesError> {
    let docs_exists = if let Some(commit_id) = commit_id {
        folder_exists_at_commit(pool, repository_id, commit_id, "docs").await?
    } else {
        false
    };
    Ok(vec![
        PagesFolderOption {
            value: "/".to_owned(),
            label: "/ (root)".to_owned(),
            exists: true,
        },
        PagesFolderOption {
            value: "/docs".to_owned(),
            label: "/docs".to_owned(),
            exists: docs_exists,
        },
    ])
}

async fn pages_workflow_suggestions(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<PagesWorkflowSuggestion>, PagesError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, path
        FROM actions_workflows
        WHERE repository_id = $1 AND state = 'active'
        ORDER BY lower(name), lower(path)
        LIMIT 10
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let path: String = row.try_get("path")?;
            Ok(PagesWorkflowSuggestion {
                workflow_id: row.try_get("id")?,
                name: row.try_get("name")?,
                artifact_hint: if path.contains("pages") {
                    "github-pages".to_owned()
                } else {
                    "Upload a github-pages artifact before deployment".to_owned()
                },
                path,
            })
        })
        .collect()
}

async fn pages_deployments(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<PagesDeploymentSummary>, PagesError> {
    let rows = sqlx::query(
        r#"
        SELECT id, source_kind, source_branch, source_folder, workflow_run_id,
               workflow_artifact_id, status, conclusion, default_url, custom_domain_url,
               artifact_storage_key, artifact_manifest, build_log_excerpt, failure_reason,
               queued_at, completed_at, created_at
        FROM pages_deployments
        WHERE repository_id = $1
        ORDER BY created_at DESC
        LIMIT 20
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(deployment_from_row).collect()
}

async fn pages_audit_events(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Vec<RepositorySettingsAuditEvent>, PagesError> {
    let rows = sqlx::query(
        r#"
        SELECT id, event_type, changed_fields, actor_user_id, created_at
        FROM repository_settings_audit_events
        WHERE repository_id = $1 AND event_type LIKE 'repository.pages.%'
        ORDER BY created_at DESC
        LIMIT 20
        "#,
    )
    .bind(repository_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(RepositorySettingsAuditEvent {
                id: row.try_get("id")?,
                event_type: row.try_get("event_type")?,
                changed_fields: row.try_get("changed_fields")?,
                actor_user_id: row.try_get("actor_user_id")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}

async fn pages_site_audit_state(
    pool: &PgPool,
    site_id: Uuid,
) -> Result<serde_json::Value, PagesError> {
    let row = sqlx::query(
        r#"
        SELECT source_kind, source_branch, source_folder, workflow_id, workflow_artifact_name,
               custom_domain, dns_status, https_enforced, certificate_status, provisioning_status
        FROM pages_sites
        WHERE id = $1
        "#,
    )
    .bind(site_id)
    .fetch_one(pool)
    .await?;
    Ok(json!({
        "sourceKind": row.try_get::<String, _>("source_kind")?,
        "sourceBranch": row.try_get::<Option<String>, _>("source_branch")?,
        "sourceFolder": row.try_get::<Option<String>, _>("source_folder")?,
        "workflowId": row.try_get::<Option<Uuid>, _>("workflow_id")?,
        "workflowArtifactName": row.try_get::<Option<String>, _>("workflow_artifact_name")?,
        "customDomain": row.try_get::<Option<String>, _>("custom_domain")?,
        "dnsStatus": row.try_get::<String, _>("dns_status")?,
        "httpsEnforced": row.try_get::<bool, _>("https_enforced")?,
        "certificateStatus": row.try_get::<String, _>("certificate_status")?,
        "provisioningStatus": row.try_get::<String, _>("provisioning_status")?,
    }))
}

async fn insert_pages_audit_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    repository_id: Uuid,
    actor_user_id: Uuid,
    event_type: &str,
    changed_fields: Vec<String>,
    before_state: serde_json::Value,
    after_state: serde_json::Value,
) -> Result<(), PagesError> {
    sqlx::query(
        r#"
        INSERT INTO repository_settings_audit_events (
            repository_id, actor_user_id, event_type, changed_fields, before_state, after_state
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(repository_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(changed_fields)
    .bind(before_state)
    .bind(after_state)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn pages_site_from_row(
    row: &sqlx::postgres::PgRow,
    include_sensitive: bool,
) -> Result<PagesSiteSummary, PagesError> {
    let dns_status: String = row.try_get("dns_status")?;
    let challenge = if include_sensitive {
        let name: Option<String> = row.try_get("dns_challenge_name")?;
        let value: Option<String> = row.try_get("dns_challenge_value")?;
        name.zip(value).map(|(name, value)| PagesDnsChallenge {
            name,
            value,
            record_type: "TXT".to_owned(),
        })
    } else {
        None
    };
    let warning = match dns_status.as_str() {
        "pending" => {
            Some("DNS verification is waiting for the TXT challenge to propagate.".to_owned())
        }
        "misconfigured" => Some("DNS records do not match the expected Pages target.".to_owned()),
        _ => None,
    };
    Ok(PagesSiteSummary {
        id: row.try_get("id")?,
        source: source_from_row(row)?,
        default_site_url: row.try_get("default_site_url")?,
        custom_domain: row.try_get("custom_domain")?,
        domain: PagesDomainState {
            status: dns_status,
            challenge,
            last_checked_at: None,
            warning,
        },
        https_enforced: row.try_get("https_enforced")?,
        certificate_status: row.try_get("certificate_status")?,
        provisioning_status: row.try_get("provisioning_status")?,
        cloudfront_alias: if include_sensitive {
            row.try_get("cloudfront_alias")?
        } else {
            None
        },
        latest_deployment_id: row.try_get("last_deployment_id")?,
        unpublished_at: row.try_get("unpublished_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn source_from_row(row: &sqlx::postgres::PgRow) -> Result<PagesSource, PagesError> {
    Ok(PagesSource {
        kind: PagesSourceKind::try_from(row.try_get::<String, _>("source_kind")?.as_str())?,
        branch: row.try_get("source_branch")?,
        folder: row.try_get("source_folder")?,
        workflow_id: row.try_get("workflow_id")?,
        workflow_artifact_name: row.try_get("workflow_artifact_name")?,
    })
}

fn deployment_from_row(row: sqlx::postgres::PgRow) -> Result<PagesDeploymentSummary, PagesError> {
    Ok(PagesDeploymentSummary {
        id: row.try_get("id")?,
        source: PagesSource {
            kind: PagesSourceKind::try_from(row.try_get::<String, _>("source_kind")?.as_str())?,
            branch: row.try_get("source_branch")?,
            folder: row.try_get("source_folder")?,
            workflow_id: None,
            workflow_artifact_name: None,
        },
        status: row.try_get("status")?,
        conclusion: row.try_get("conclusion")?,
        default_url: row.try_get("default_url")?,
        custom_domain_url: row.try_get("custom_domain_url")?,
        workflow_run_id: row.try_get("workflow_run_id")?,
        workflow_artifact_id: row.try_get("workflow_artifact_id")?,
        artifact_storage_key: row.try_get("artifact_storage_key")?,
        artifact_manifest: row.try_get("artifact_manifest")?,
        build_log_excerpt: row.try_get("build_log_excerpt")?,
        failure_reason: row.try_get("failure_reason")?,
        queued_at: row.try_get("queued_at")?,
        completed_at: row.try_get("completed_at")?,
        created_at: row.try_get("created_at")?,
    })
}

fn normalize_branch_name(value: &str) -> Result<String, PagesError> {
    let trimmed = value
        .trim()
        .strip_prefix("refs/heads/")
        .unwrap_or(value.trim())
        .trim_matches('/');
    if trimmed.is_empty() || trimmed.contains("..") || trimmed.contains('\\') {
        return Err(PagesError::Invalid("branch name is invalid".to_owned()));
    }
    Ok(trimmed.to_owned())
}

fn normalize_pages_folder(value: Option<&str>) -> Result<String, PagesError> {
    match value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("/")
    {
        "/" | "/root" | "root" => Ok("/".to_owned()),
        "docs" | "/docs" => Ok("/docs".to_owned()),
        other => Err(PagesError::Invalid(format!(
            "unsupported Pages folder `{other}`"
        ))),
    }
}

fn normalize_custom_domain(value: &str) -> Result<String, PagesError> {
    let trimmed = value
        .trim()
        .trim_end_matches('.')
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_ascii_lowercase();
    if trimmed.is_empty()
        || trimmed.len() > 253
        || trimmed.contains('/')
        || trimmed.contains('*')
        || !trimmed.contains('.')
    {
        return Err(PagesError::Invalid("custom domain is invalid".to_owned()));
    }
    Url::parse(&format!("https://{trimmed}"))
        .map_err(|_| PagesError::Invalid("custom domain is invalid".to_owned()))?;
    Ok(trimmed)
}

async fn ensure_domain_available(
    pool: &PgPool,
    repository_id: Uuid,
    domain: &str,
) -> Result<(), PagesError> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM pages_sites
            WHERE lower(custom_domain) = lower($1)
              AND repository_id <> $2
              AND unpublished_at IS NULL
        )
        "#,
    )
    .bind(domain)
    .bind(repository_id)
    .fetch_one(pool)
    .await?;
    if exists {
        Err(PagesError::Conflict)
    } else {
        Ok(())
    }
}

fn default_pages_url(repository: &Repository) -> String {
    format!(
        "https://{}.opengithub-pages.namuh.co/{}",
        repository.owner_login, repository.name
    )
}

fn viewer_permission(repository: &Repository, actor_user_id: Uuid, role: Option<String>) -> String {
    role.unwrap_or_else(|| {
        if repository.owner_user_id == Some(actor_user_id) {
            RepositoryRole::Owner.as_str().to_owned()
        } else {
            RepositoryRole::Read.as_str().to_owned()
        }
    })
}

#[derive(Debug, Clone, Copy)]
struct PagesSiteRow {
    id: Uuid,
}
