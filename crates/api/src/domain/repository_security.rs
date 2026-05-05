use chrono::{DateTime, Utc};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::{
    markdown::{render_markdown, RenderMarkdownInput},
    repositories::{
        can_read_repository, can_write_repository, get_repository_by_owner_name,
        replace_repository_snapshot, CreateCommit, Repository, RepositoryError, RepositorySnapshot,
        RepositorySnapshotFile, RepositoryVisibility,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySecurityOverview {
    pub repository: RepositorySecurityRepository,
    pub viewer: SecurityViewer,
    pub policy: SecurityPolicySummary,
    pub features: Vec<SecurityFeatureCard>,
    pub advisories: Vec<RepositorySecurityAdvisorySummary>,
    pub links: SecurityOverviewLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySecurityRepository {
    pub id: Uuid,
    pub owner_login: String,
    pub name: String,
    pub visibility: String,
    pub default_branch: String,
    pub security_href: String,
    pub policy_href: String,
    pub advisories_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityViewer {
    pub permission: String,
    pub can_read: bool,
    pub can_write: bool,
    pub can_edit_policy: bool,
    pub can_view_private_alert_counts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPolicySummary {
    pub exists: bool,
    pub path: Option<String>,
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub blob_oid: Option<String>,
    pub content_sha: Option<String>,
    pub html: Option<String>,
    pub source_href: Option<String>,
    pub raw_href: Option<String>,
    pub history_href: Option<String>,
    pub edit_href: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub empty_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityFeatureCard {
    pub key: String,
    pub label: String,
    pub status: String,
    pub summary: String,
    pub alert_count: Option<i64>,
    pub private_count: Option<i64>,
    pub href: String,
    pub config_href: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySecurityAdvisorySummary {
    pub id: Uuid,
    pub identifier: String,
    pub severity: String,
    pub status: String,
    pub title: String,
    pub summary: String,
    pub package_name: Option<String>,
    pub vulnerable_range: Option<String>,
    pub href: String,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityOverviewLinks {
    pub overview_href: String,
    pub policy_href: String,
    pub advisories_href: String,
    pub dependabot_href: String,
    pub code_scanning_href: String,
    pub secret_scanning_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySecurityPolicyView {
    pub repository: RepositorySecurityRepository,
    pub viewer: SecurityViewer,
    pub policy: SecurityPolicyDocument,
    pub links: SecurityOverviewLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPolicyDocument {
    pub exists: bool,
    pub path: Option<String>,
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub blob_oid: Option<String>,
    pub content_sha: Option<String>,
    pub markdown: Option<String>,
    pub html: Option<String>,
    pub outline: Vec<SecurityPolicyHeading>,
    pub source_href: Option<String>,
    pub raw_href: Option<String>,
    pub history_href: Option<String>,
    pub edit_href: Option<String>,
    pub latest_commit: Option<SecurityPolicyCommit>,
    pub updated_at: Option<DateTime<Utc>>,
    pub empty_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPolicyHeading {
    pub id: String,
    pub level: i32,
    pub text: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPolicyCommit {
    pub oid: String,
    pub short_oid: String,
    pub message: String,
    pub committed_at: DateTime<Utc>,
    pub href: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPolicyMutation {
    pub markdown: String,
    pub commit_message: String,
    pub path: Option<String>,
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub expected_content_sha: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct DependabotAlertsQuery<'a> {
    pub state: Option<&'a str>,
    pub query: Option<&'a str>,
    pub package: Option<&'a str>,
    pub ecosystem: Option<&'a str>,
    pub manifest: Option<&'a str>,
    pub scope: Option<&'a str>,
    pub severity: Option<&'a str>,
    pub sort: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertsView {
    pub repository: RepositorySecurityRepository,
    pub viewer: SecurityViewer,
    pub availability: DependabotAlertsAvailability,
    pub filters: DependabotAlertFilters,
    pub counts: DependabotAlertCounts,
    pub alerts: Vec<DependabotAlertRow>,
    pub packages: Vec<DependabotAlertPackageFilter>,
    pub manifests: Vec<DependabotAlertManifestFilter>,
    pub links: DependabotAlertLinks,
    pub freshness: DependabotAlertFreshness,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertDetail {
    pub repository: RepositorySecurityRepository,
    pub viewer: SecurityViewer,
    pub availability: DependabotAlertsAvailability,
    pub alert: DependabotAlertRow,
    pub advisory: DependabotAdvisoryDetail,
    pub dependency: DependabotDependencyDetail,
    pub timeline: Vec<DependabotAlertTimelineEvent>,
    pub assignee_options: Vec<DependabotAlertAssignmentOption>,
    pub security_update: DependabotSecurityUpdateState,
    pub links: DependabotAlertLinks,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertsAvailability {
    pub enabled: bool,
    pub indexed: bool,
    pub message: String,
    pub disabled_reason: Option<String>,
    pub settings_href: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertFilters {
    pub state: String,
    pub query: Option<String>,
    pub package: Option<String>,
    pub ecosystem: Option<String>,
    pub manifest: Option<String>,
    pub scope: Option<String>,
    pub severity: Option<String>,
    pub sort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertCounts {
    pub open: i64,
    pub closed: i64,
    pub total: i64,
    pub visible: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertPackage {
    pub id: Uuid,
    pub ecosystem: String,
    pub name: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertAdvisorySummary {
    pub id: Uuid,
    pub identifier: String,
    pub severity: String,
    pub title: String,
    pub href: String,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertRow {
    pub id: Uuid,
    pub number: i64,
    pub state: String,
    pub scope: String,
    pub package: DependabotAlertPackage,
    pub advisory: DependabotAlertAdvisorySummary,
    pub manifest_path: String,
    pub manifest_href: String,
    pub lockfile_path: Option<String>,
    pub lockfile_href: Option<String>,
    pub vulnerable_requirements: Option<String>,
    pub current_version: Option<String>,
    pub fixed_version: Option<String>,
    pub relationship: String,
    pub assignees: Vec<DependabotAlertAssignee>,
    pub href: String,
    pub detected_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertAssignee {
    pub id: Uuid,
    pub login: String,
    pub avatar_url: Option<String>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertPackageFilter {
    pub package: DependabotAlertPackage,
    pub open_count: i64,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertManifestFilter {
    pub path: String,
    pub ecosystem: String,
    pub href: String,
    pub open_count: i64,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAdvisoryDetail {
    pub identifier: String,
    pub severity: String,
    pub title: String,
    pub href: String,
    pub vulnerable_range: String,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotDependencyDetail {
    pub package: DependabotAlertPackage,
    pub manifest_path: String,
    pub manifest_href: String,
    pub lockfile_path: Option<String>,
    pub lockfile_href: Option<String>,
    pub current_version: Option<String>,
    pub relationship: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertTimelineEvent {
    pub id: Uuid,
    pub event_type: String,
    pub message: String,
    pub actor: Option<DependabotAlertAssignee>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertAssignmentOption {
    pub id: Uuid,
    pub kind: String,
    pub login: String,
    pub avatar_url: Option<String>,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotSecurityUpdateState {
    pub supported: bool,
    pub status: String,
    pub href: Option<String>,
    pub pull_request_href: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertLinks {
    pub list_href: String,
    pub open_href: String,
    pub closed_href: String,
    pub settings_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertFreshness {
    pub computed_at: DateTime<Utc>,
    pub cadence: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependabotAlertMutation {
    pub action: String,
    pub dismissal_reason: Option<String>,
    pub dismissal_comment: Option<String>,
    pub assignee_ids: Option<Vec<Uuid>>,
}

pub async fn repository_dependabot_alerts_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    query: DependabotAlertsQuery<'_>,
) -> Result<Option<DependabotAlertsView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }

    repository_dependabot_alerts_for_repository(pool, &repository, actor_user_id, query)
        .await
        .map(Some)
}

pub async fn repository_dependabot_alert_detail_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    alert_number: i64,
) -> Result<Option<DependabotAlertDetail>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    if alert_number <= 0 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "alert id must be a positive number".to_owned(),
        ));
    }

    repository_dependabot_alert_detail_for_repository(
        pool,
        &repository,
        actor_user_id,
        alert_number,
    )
    .await
}

pub async fn update_repository_dependabot_alert_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    alert_number: i64,
    mutation: DependabotAlertMutation,
) -> Result<Option<DependabotAlertDetail>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    if !can_write_repository(pool, &repository, actor_user_id).await? {
        return Err(RepositoryError::PermissionDenied);
    }
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }
    if alert_number <= 0 {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "alert id must be a positive number".to_owned(),
        ));
    }

    update_repository_dependabot_alert(pool, &repository, actor_user_id, alert_number, mutation)
        .await?;
    repository_dependabot_alert_detail_for_repository(
        pool,
        &repository,
        actor_user_id,
        alert_number,
    )
    .await
}

pub async fn repository_security_overview_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositorySecurityOverview>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }

    repository_security_overview_for_repository(pool, &repository, actor_user_id)
        .await
        .map(Some)
}

pub async fn repository_security_policy_for_actor_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
) -> Result<Option<RepositorySecurityPolicyView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }

    repository_security_policy_for_repository(pool, &repository, actor_user_id)
        .await
        .map(Some)
}

pub async fn upsert_repository_security_policy_by_owner_name(
    pool: &PgPool,
    actor_user_id: Uuid,
    owner_login: &str,
    name: &str,
    mutation: SecurityPolicyMutation,
) -> Result<Option<RepositorySecurityPolicyView>, RepositoryError> {
    let Some(repository) = get_repository_by_owner_name(pool, owner_login, name).await? else {
        return Ok(None);
    };
    if !can_read_repository(pool, &repository, actor_user_id).await? {
        if repository.visibility == RepositoryVisibility::Private {
            return Ok(None);
        }
        return Err(RepositoryError::PermissionDenied);
    }
    if !can_write_repository(pool, &repository, actor_user_id).await? {
        return Err(RepositoryError::PermissionDenied);
    }
    if repository.is_archived {
        return Err(RepositoryError::ArchivedRepositoryReadOnly);
    }

    write_security_policy(pool, &repository, actor_user_id, mutation).await?;
    repository_security_policy_for_repository(pool, &repository, actor_user_id)
        .await
        .map(Some)
}

async fn repository_security_overview_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<RepositorySecurityOverview, RepositoryError> {
    let can_write = can_write_repository(pool, repository, actor_user_id).await?;
    let permission = viewer_permission(pool, repository, actor_user_id, can_write).await?;
    let links = security_links(repository);

    Ok(RepositorySecurityOverview {
        repository: RepositorySecurityRepository {
            id: repository.id,
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            visibility: repository.visibility.as_str().to_owned(),
            default_branch: repository.default_branch.clone(),
            security_href: links.overview_href.clone(),
            policy_href: links.policy_href.clone(),
            advisories_href: links.advisories_href.clone(),
        },
        viewer: SecurityViewer {
            permission,
            can_read: true,
            can_write,
            can_edit_policy: can_write && !repository.is_archived,
            can_view_private_alert_counts: can_write,
        },
        policy: security_policy_summary(pool, repository, can_write).await?,
        features: security_feature_cards(pool, repository, can_write).await?,
        advisories: published_advisories(pool, repository).await?,
        links,
    })
}

async fn repository_security_policy_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
) -> Result<RepositorySecurityPolicyView, RepositoryError> {
    let can_write = can_write_repository(pool, repository, actor_user_id).await?;
    let permission = viewer_permission(pool, repository, actor_user_id, can_write).await?;
    let links = security_links(repository);

    Ok(RepositorySecurityPolicyView {
        repository: RepositorySecurityRepository {
            id: repository.id,
            owner_login: repository.owner_login.clone(),
            name: repository.name.clone(),
            visibility: repository.visibility.as_str().to_owned(),
            default_branch: repository.default_branch.clone(),
            security_href: links.overview_href.clone(),
            policy_href: links.policy_href.clone(),
            advisories_href: links.advisories_href.clone(),
        },
        viewer: SecurityViewer {
            permission,
            can_read: true,
            can_write,
            can_edit_policy: can_write && !repository.is_archived,
            can_view_private_alert_counts: can_write,
        },
        policy: security_policy_document(pool, repository, can_write).await?,
        links,
    })
}

async fn viewer_permission(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    can_write: bool,
) -> Result<String, RepositoryError> {
    if repository.owner_user_id == Some(actor_user_id) {
        return Ok("owner".to_owned());
    }
    if can_write {
        return Ok("write".to_owned());
    }
    if repository.visibility == RepositoryVisibility::Public {
        return Ok("read".to_owned());
    }
    let role =
        super::repositories::repository_permission_for_user(pool, repository.id, actor_user_id)
            .await?
            .map(|permission| permission.role.as_str().to_owned())
            .unwrap_or_else(|| "read".to_owned());
    Ok(role)
}

async fn security_policy_summary(
    pool: &PgPool,
    repository: &Repository,
    can_write: bool,
) -> Result<SecurityPolicySummary, RepositoryError> {
    if let Some(row) = sqlx::query(
        r#"
        SELECT repository_security_policies.path,
               repository_security_policies.ref_name,
               repository_security_policies.blob_oid,
               repository_security_policies.content_sha,
               repository_security_policies.markdown,
               repository_security_policies.rendered_html,
               repository_security_policies.updated_at,
               commits.oid AS commit_oid,
               commits.message AS commit_message,
               commits.committed_at AS committed_at
        FROM repository_security_policies
        LEFT JOIN commits ON commits.id = repository_security_policies.source_commit_id
        WHERE repository_security_policies.repository_id = $1
          AND repository_security_policies.published = true
        ORDER BY CASE WHEN lower(path) = 'security.md' THEN 0 ELSE 1 END, updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .fetch_optional(pool)
    .await?
    {
        return policy_from_row(repository, row, can_write).await;
    }

    let row = sqlx::query(
        r#"
        SELECT repository_files.path,
               $2::text AS ref_name,
               repository_files.oid AS blob_oid,
               repository_files.content AS markdown,
               commits.committed_at AS updated_at
        FROM repository_files
        JOIN repository_git_refs
          ON repository_git_refs.repository_id = repository_files.repository_id
         AND repository_git_refs.target_commit_id = repository_files.commit_id
        JOIN commits ON commits.id = repository_files.commit_id
        WHERE repository_files.repository_id = $1
          AND repository_git_refs.name IN ($2, 'refs/heads/' || $2)
          AND lower(repository_files.path) IN ('security.md', '.github/security.md', 'docs/security.md')
        ORDER BY CASE lower(repository_files.path)
            WHEN 'security.md' THEN 0
            WHEN '.github/security.md' THEN 1
            ELSE 2
        END
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(&repository.default_branch)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(SecurityPolicySummary {
            exists: false,
            path: None,
            ref_name: None,
            blob_oid: None,
            content_sha: None,
            html: None,
            source_href: None,
            raw_href: None,
            history_href: None,
            edit_href: if can_write {
                Some(format!(
                    "/{}/{}/security/policy/edit",
                    repository.owner_login, repository.name
                ))
            } else {
                None
            },
            updated_at: None,
            empty_state: if can_write {
                "No SECURITY.md policy has been published. Maintainers can start setup.".to_owned()
            } else {
                "No security policy has been published for this repository.".to_owned()
            },
        });
    };

    let path: String = row.get("path");
    let ref_name: String = row.get("ref_name");
    let markdown: String = row.get("markdown");
    let content_sha = markdown_sha(&markdown);
    let rendered = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            markdown: markdown.clone(),
            repository_id: Some(repository.id),
            owner: Some(repository.owner_login.clone()),
            repo: Some(repository.name.clone()),
            ref_name: Some(ref_name.clone()),
            enable_task_toggles: Some(false),
        },
    )
    .await
    .map_err(markdown_error)?;

    Ok(SecurityPolicySummary {
        exists: true,
        path: Some(path.clone()),
        ref_name: Some(ref_name.clone()),
        blob_oid: row.get("blob_oid"),
        content_sha: Some(content_sha),
        html: Some(rendered.html),
        source_href: Some(repository_blob_href(repository, &ref_name, &path)),
        raw_href: Some(repository_raw_href(repository, &ref_name, &path)),
        history_href: Some(repository_history_href(repository, &ref_name, &path)),
        edit_href: can_write.then(|| {
            format!(
                "/{}/{}/security/policy/edit?path={}",
                repository.owner_login,
                repository.name,
                percent_encode_path(&path)
            )
        }),
        updated_at: row.get("updated_at"),
        empty_state: String::new(),
    })
}

async fn security_policy_document(
    pool: &PgPool,
    repository: &Repository,
    can_write: bool,
) -> Result<SecurityPolicyDocument, RepositoryError> {
    if let Some(row) = sqlx::query(
        r#"
        SELECT repository_security_policies.path,
               repository_security_policies.ref_name,
               repository_security_policies.blob_oid,
               repository_security_policies.content_sha,
               repository_security_policies.markdown,
               repository_security_policies.rendered_html,
               repository_security_policies.updated_at,
               commits.oid AS commit_oid,
               commits.message AS commit_message,
               commits.committed_at AS committed_at
        FROM repository_security_policies
        LEFT JOIN commits ON commits.id = repository_security_policies.source_commit_id
        WHERE repository_security_policies.repository_id = $1
          AND repository_security_policies.published = true
        ORDER BY CASE WHEN lower(path) = 'security.md' THEN 0 ELSE 1 END, updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .fetch_optional(pool)
    .await?
    {
        return policy_document_from_row(repository, row, can_write);
    }

    let row = sqlx::query(
        r#"
        SELECT repository_files.path,
               $2::text AS ref_name,
               repository_files.oid AS blob_oid,
               repository_files.content AS markdown,
               commits.oid AS commit_oid,
               commits.message AS commit_message,
               commits.committed_at AS committed_at
        FROM repository_files
        JOIN repository_git_refs
          ON repository_git_refs.repository_id = repository_files.repository_id
         AND repository_git_refs.target_commit_id = repository_files.commit_id
        JOIN commits ON commits.id = repository_files.commit_id
        WHERE repository_files.repository_id = $1
          AND repository_git_refs.name IN ($2, 'refs/heads/' || $2)
          AND lower(repository_files.path) IN ('security.md', '.github/security.md', 'docs/security.md')
        ORDER BY CASE lower(repository_files.path)
            WHEN 'security.md' THEN 0
            WHEN '.github/security.md' THEN 1
            ELSE 2
        END
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(&repository.default_branch)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(SecurityPolicyDocument {
            exists: false,
            path: None,
            ref_name: None,
            blob_oid: None,
            content_sha: None,
            markdown: None,
            html: None,
            outline: Vec::new(),
            source_href: None,
            raw_href: None,
            history_href: None,
            edit_href: if can_write {
                Some(format!(
                    "/{}/{}/security/policy/edit",
                    repository.owner_login, repository.name
                ))
            } else {
                None
            },
            latest_commit: None,
            updated_at: None,
            empty_state: if can_write {
                "No SECURITY.md policy has been published. Maintainers can start setup.".to_owned()
            } else {
                "No security policy has been published for this repository.".to_owned()
            },
        });
    };

    let path: String = row.get("path");
    let ref_name: String = row.get("ref_name");
    let markdown: String = row.get("markdown");
    let rendered = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            markdown: markdown.clone(),
            repository_id: Some(repository.id),
            owner: Some(repository.owner_login.clone()),
            repo: Some(repository.name.clone()),
            ref_name: Some(ref_name.clone()),
            enable_task_toggles: Some(false),
        },
    )
    .await
    .map_err(markdown_error)?;
    let commit_oid: String = row.get("commit_oid");
    let committed_at: DateTime<Utc> = row.get("committed_at");

    Ok(SecurityPolicyDocument {
        exists: true,
        path: Some(path.clone()),
        ref_name: Some(ref_name.clone()),
        blob_oid: row.get("blob_oid"),
        content_sha: Some(rendered.content_sha.clone()),
        markdown: Some(markdown),
        outline: policy_heading_outline(&rendered.html),
        html: Some(rendered.html),
        source_href: Some(repository_blob_href(repository, &ref_name, &path)),
        raw_href: Some(repository_raw_href(repository, &ref_name, &path)),
        history_href: Some(repository_history_href(repository, &ref_name, &path)),
        edit_href: can_write.then(|| {
            format!(
                "/{}/{}/security/policy/edit?path={}",
                repository.owner_login,
                repository.name,
                percent_encode_path(&path)
            )
        }),
        latest_commit: Some(SecurityPolicyCommit {
            short_oid: commit_oid.chars().take(7).collect(),
            href: format!(
                "/{}/{}/commit/{}",
                repository.owner_login,
                repository.name,
                percent_encode_segment(&commit_oid)
            ),
            oid: commit_oid,
            message: row.get("commit_message"),
            committed_at,
        }),
        updated_at: Some(committed_at),
        empty_state: String::new(),
    })
}

async fn policy_from_row(
    repository: &Repository,
    row: sqlx::postgres::PgRow,
    can_write: bool,
) -> Result<SecurityPolicySummary, RepositoryError> {
    let path: String = row.get("path");
    let ref_name: String = row.get("ref_name");
    Ok(SecurityPolicySummary {
        exists: true,
        path: Some(path.clone()),
        ref_name: Some(ref_name.clone()),
        blob_oid: row.get("blob_oid"),
        content_sha: Some(row.get("content_sha")),
        html: Some(row.get("rendered_html")),
        source_href: Some(repository_blob_href(repository, &ref_name, &path)),
        raw_href: Some(repository_raw_href(repository, &ref_name, &path)),
        history_href: Some(repository_history_href(repository, &ref_name, &path)),
        edit_href: can_write.then(|| {
            format!(
                "/{}/{}/security/policy/edit?path={}",
                repository.owner_login,
                repository.name,
                percent_encode_path(&path)
            )
        }),
        updated_at: row.get("updated_at"),
        empty_state: String::new(),
    })
}

fn policy_document_from_row(
    repository: &Repository,
    row: sqlx::postgres::PgRow,
    can_write: bool,
) -> Result<SecurityPolicyDocument, RepositoryError> {
    let path: String = row.get("path");
    let ref_name: String = row.get("ref_name");
    let html: String = row.get("rendered_html");
    Ok(SecurityPolicyDocument {
        exists: true,
        path: Some(path.clone()),
        ref_name: Some(ref_name.clone()),
        blob_oid: row.get("blob_oid"),
        content_sha: Some(row.get("content_sha")),
        markdown: Some(row.get("markdown")),
        outline: policy_heading_outline(&html),
        html: Some(html),
        source_href: Some(repository_blob_href(repository, &ref_name, &path)),
        raw_href: Some(repository_raw_href(repository, &ref_name, &path)),
        history_href: Some(repository_history_href(repository, &ref_name, &path)),
        edit_href: can_write.then(|| {
            format!(
                "/{}/{}/security/policy/edit?path={}",
                repository.owner_login,
                repository.name,
                percent_encode_path(&path)
            )
        }),
        latest_commit: match (
            row.try_get::<Option<String>, _>("commit_oid")
                .ok()
                .flatten(),
            row.try_get::<Option<String>, _>("commit_message")
                .ok()
                .flatten(),
            row.try_get::<Option<DateTime<Utc>>, _>("committed_at")
                .ok()
                .flatten(),
        ) {
            (Some(oid), Some(message), Some(committed_at)) => Some(SecurityPolicyCommit {
                short_oid: oid.chars().take(7).collect(),
                href: format!(
                    "/{}/{}/commit/{}",
                    repository.owner_login,
                    repository.name,
                    percent_encode_segment(&oid)
                ),
                oid,
                message,
                committed_at,
            }),
            _ => None,
        },
        updated_at: row.get("updated_at"),
        empty_state: String::new(),
    })
}

async fn security_feature_cards(
    pool: &PgPool,
    repository: &Repository,
    can_view_counts: bool,
) -> Result<Vec<SecurityFeatureCard>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT feature_key, status, summary, alert_count, private_count, config_href, updated_at
        FROM repository_security_feature_settings
        WHERE repository_id = $1
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;
    let mut cards = default_feature_cards(repository);
    for row in rows {
        let key: String = row.get("feature_key");
        if let Some(card) = cards.iter_mut().find(|card| card.key == key) {
            card.status = row.get("status");
            card.summary = row.get("summary");
            let alert_count = row.get::<i64, _>("alert_count");
            let private_count = row.get::<i64, _>("private_count");
            card.alert_count = can_view_counts.then_some(alert_count);
            card.private_count = can_view_counts.then_some(private_count);
            card.config_href = row.get("config_href");
            card.updated_at = row.get("updated_at");
        }
    }
    Ok(cards)
}

fn default_feature_cards(repository: &Repository) -> Vec<SecurityFeatureCard> {
    [
        (
            "dependabot",
            "Dependabot",
            "Dependency update and vulnerability alert coverage.",
            format!(
                "/{}/{}/security/dependabot",
                repository.owner_login, repository.name
            ),
        ),
        (
            "code_scanning",
            "Code scanning",
            "Static analysis findings from configured workflows.",
            format!(
                "/{}/{}/security/code-scanning",
                repository.owner_login, repository.name
            ),
        ),
        (
            "secret_scanning",
            "Secret scanning",
            "Credential exposure detection for committed content.",
            format!(
                "/{}/{}/security/secret-scanning",
                repository.owner_login, repository.name
            ),
        ),
        (
            "private_vulnerability_reporting",
            "Private vulnerability reporting",
            "Coordinated disclosure intake for repository maintainers.",
            format!(
                "/{}/{}/security/advisories/new",
                repository.owner_login, repository.name
            ),
        ),
    ]
    .into_iter()
    .map(|(key, label, summary, href)| SecurityFeatureCard {
        key: key.to_owned(),
        label: label.to_owned(),
        status: "needs_setup".to_owned(),
        summary: summary.to_owned(),
        alert_count: can_never_count(),
        private_count: can_never_count(),
        href,
        config_href: None,
        updated_at: None,
    })
    .collect()
}

const fn can_never_count() -> Option<i64> {
    None
}

async fn published_advisories(
    pool: &PgPool,
    repository: &Repository,
) -> Result<Vec<RepositorySecurityAdvisorySummary>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT id, advisory_identifier, severity, status, title, summary, package_name,
               vulnerable_range, advisory_href, published_at, updated_at
        FROM repository_security_advisories
        WHERE repository_id = $1 AND status = 'published'
        ORDER BY COALESCE(published_at, updated_at) DESC, advisory_identifier ASC
        LIMIT 10
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(RepositorySecurityAdvisorySummary {
                id: row.get("id"),
                identifier: row.get("advisory_identifier"),
                severity: row.get("severity"),
                status: row.get("status"),
                title: row.get("title"),
                summary: row.get("summary"),
                package_name: row.get("package_name"),
                vulnerable_range: row.get("vulnerable_range"),
                href: row.get("advisory_href"),
                published_at: row.get("published_at"),
                updated_at: row.get("updated_at"),
            })
        })
        .collect()
}

fn security_links(repository: &Repository) -> SecurityOverviewLinks {
    let base = format!("/{}/{}", repository.owner_login, repository.name);
    SecurityOverviewLinks {
        overview_href: format!("{base}/security"),
        policy_href: format!("{base}/security/policy"),
        advisories_href: format!("{base}/security/advisories"),
        dependabot_href: format!("{base}/security/dependabot"),
        code_scanning_href: format!("{base}/security/code-scanning"),
        secret_scanning_href: format!("{base}/security/secret-scanning"),
    }
}

async fn repository_dependabot_alerts_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    query: DependabotAlertsQuery<'_>,
) -> Result<DependabotAlertsView, RepositoryError> {
    let filters = normalize_dependabot_alert_filters(query)?;
    let can_write = can_write_repository(pool, repository, actor_user_id).await?;
    let setting = dependabot_setting(pool, repository).await?;
    let availability = dependabot_availability(repository, setting.as_ref());

    if availability.enabled {
        materialize_dependabot_alerts(pool, repository).await?;
    }

    let mut alerts = if availability.enabled {
        dependabot_alert_rows(pool, repository).await?
    } else {
        Vec::new()
    };
    let all_alerts = alerts.clone();
    apply_dependabot_alert_filters(&mut alerts, &filters);
    sort_dependabot_alerts(&mut alerts, &filters.sort);

    let links = dependabot_links(repository);
    Ok(DependabotAlertsView {
        repository: security_repository(repository, &links),
        viewer: security_viewer(pool, repository, actor_user_id, can_write).await?,
        availability,
        filters,
        counts: dependabot_counts(&all_alerts, alerts.len() as i64),
        alerts,
        packages: dependabot_package_filters(repository, &all_alerts, query.package).await?,
        manifests: dependabot_manifest_filters(repository, &all_alerts, query.manifest).await?,
        links,
        freshness: DependabotAlertFreshness {
            computed_at: Utc::now(),
            cadence: "on repository dependency graph refresh".to_owned(),
        },
    })
}

async fn repository_dependabot_alert_detail_for_repository(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    alert_number: i64,
) -> Result<Option<DependabotAlertDetail>, RepositoryError> {
    let can_write = can_write_repository(pool, repository, actor_user_id).await?;
    let setting = dependabot_setting(pool, repository).await?;
    let availability = dependabot_availability(repository, setting.as_ref());
    if availability.enabled {
        materialize_dependabot_alerts(pool, repository).await?;
    }

    let Some(alert) = dependabot_alert_rows(pool, repository)
        .await?
        .into_iter()
        .find(|alert| alert.number == alert_number)
    else {
        return Ok(None);
    };
    let links = dependabot_links(repository);
    let timeline = dependabot_alert_timeline(pool, alert.id).await?;
    let assignee_options = dependabot_assignment_options(pool, repository, alert.id).await?;
    let advisory = DependabotAdvisoryDetail {
        identifier: alert.advisory.identifier.clone(),
        severity: alert.advisory.severity.clone(),
        title: alert.advisory.title.clone(),
        href: alert.advisory.href.clone(),
        vulnerable_range: alert
            .vulnerable_requirements
            .clone()
            .unwrap_or_else(|| "See advisory".to_owned()),
        published_at: alert.advisory.published_at,
    };
    let dependency = DependabotDependencyDetail {
        package: alert.package.clone(),
        manifest_path: alert.manifest_path.clone(),
        manifest_href: alert.manifest_href.clone(),
        lockfile_path: alert.lockfile_path.clone(),
        lockfile_href: alert.lockfile_href.clone(),
        current_version: alert.current_version.clone(),
        relationship: alert.relationship.clone(),
    };
    let security_update = dependabot_security_update_state(repository, &alert);

    Ok(Some(DependabotAlertDetail {
        repository: security_repository(repository, &links),
        viewer: security_viewer(pool, repository, actor_user_id, can_write).await?,
        availability,
        alert,
        advisory,
        dependency,
        timeline,
        assignee_options,
        security_update,
        links,
    }))
}

async fn update_repository_dependabot_alert(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    alert_number: i64,
    mutation: DependabotAlertMutation,
) -> Result<(), RepositoryError> {
    let setting = dependabot_setting(pool, repository).await?;
    let availability = dependabot_availability(repository, setting.as_ref());
    if !availability.enabled {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "Dependabot alerts are disabled for this repository".to_owned(),
        ));
    }
    materialize_dependabot_alerts(pool, repository).await?;

    let alert = sqlx::query(
        r#"
        SELECT id, state, fixed_version
        FROM dependabot_alerts
        WHERE repository_id = $1 AND number = $2
        "#,
    )
    .bind(repository.id)
    .bind(alert_number)
    .fetch_optional(pool)
    .await?;
    let Some(alert) = alert else {
        return Err(RepositoryError::NotFound);
    };
    let alert_id: Uuid = alert.get("id");
    let state: String = alert.get("state");
    let fixed_version: Option<String> = alert.get("fixed_version");

    match mutation.action.as_str() {
        "dismiss" => {
            if state != "open" {
                return Err(RepositoryError::InvalidDependencyGraphQuery(
                    "only open Dependabot alerts can be dismissed".to_owned(),
                ));
            }
            let reason =
                normalize_dependabot_dismissal_reason(mutation.dismissal_reason.as_deref())?;
            let comment =
                normalize_dependabot_dismissal_comment(mutation.dismissal_comment.as_deref())?;
            sqlx::query(
                r#"
                UPDATE dependabot_alerts
                SET state = 'dismissed',
                    dismissed_reason = $3,
                    dismissed_comment = $4,
                    dismissed_by_user_id = $5,
                    dismissed_at = now(),
                    updated_at = now()
                WHERE repository_id = $1 AND id = $2
                "#,
            )
            .bind(repository.id)
            .bind(alert_id)
            .bind(&reason)
            .bind(&comment)
            .bind(actor_user_id)
            .execute(pool)
            .await?;
            record_dependabot_alert_event(
                pool,
                repository,
                alert_id,
                actor_user_id,
                "dismissed",
                &format!("Dismissed this alert as {reason}."),
                json!({ "reason": reason, "hasComment": comment.is_some() }),
            )
            .await?;
            notify_dependabot_alert_assignees(
                pool,
                repository,
                alert_id,
                "Dependabot alert dismissed",
                "security_alert",
            )
            .await?;
        }
        "reopen" => {
            if fixed_version.is_some() || state == "fixed" {
                return Err(RepositoryError::InvalidDependencyGraphQuery(
                    "fixed Dependabot alerts cannot be reopened".to_owned(),
                ));
            }
            if state != "dismissed" {
                return Err(RepositoryError::InvalidDependencyGraphQuery(
                    "only dismissed Dependabot alerts can be reopened".to_owned(),
                ));
            }
            sqlx::query(
                r#"
                UPDATE dependabot_alerts
                SET state = 'open',
                    dismissed_reason = NULL,
                    dismissed_comment = NULL,
                    dismissed_by_user_id = NULL,
                    dismissed_at = NULL,
                    updated_at = now()
                WHERE repository_id = $1 AND id = $2
                "#,
            )
            .bind(repository.id)
            .bind(alert_id)
            .execute(pool)
            .await?;
            record_dependabot_alert_event(
                pool,
                repository,
                alert_id,
                actor_user_id,
                "reopened",
                "Reopened this Dependabot alert.",
                json!({ "previousState": state }),
            )
            .await?;
            notify_dependabot_alert_assignees(
                pool,
                repository,
                alert_id,
                "Dependabot alert reopened",
                "security_alert",
            )
            .await?;
        }
        "assign" => {
            let assignee_ids = mutation.assignee_ids.unwrap_or_default();
            if assignee_ids.len() > 25 {
                return Err(RepositoryError::InvalidDependencyGraphQuery(
                    "Dependabot alert assignment is limited to 25 users".to_owned(),
                ));
            }
            let options = dependabot_assignment_options(pool, repository, alert_id).await?;
            for assignee_id in &assignee_ids {
                if !options.iter().any(|option| option.id == *assignee_id) {
                    return Err(RepositoryError::InvalidDependencyGraphQuery(
                        "Dependabot alert assignee must have repository access".to_owned(),
                    ));
                }
            }
            sqlx::query("DELETE FROM dependabot_alert_assignees WHERE alert_id = $1")
                .bind(alert_id)
                .execute(pool)
                .await?;
            for assignee_id in &assignee_ids {
                sqlx::query(
                    "INSERT INTO dependabot_alert_assignees (alert_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                )
                .bind(alert_id)
                .bind(assignee_id)
                .execute(pool)
                .await?;
            }
            record_dependabot_alert_event(
                pool,
                repository,
                alert_id,
                actor_user_id,
                "assigned",
                if assignee_ids.is_empty() {
                    "Cleared Dependabot alert assignees."
                } else {
                    "Updated Dependabot alert assignees."
                },
                json!({ "assigneeCount": assignee_ids.len() }),
            )
            .await?;
            notify_dependabot_alert_assignees(
                pool,
                repository,
                alert_id,
                "Dependabot alert assigned",
                "assign",
            )
            .await?;
        }
        _ => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "Dependabot alert action must be dismiss, reopen, or assign".to_owned(),
            ))
        }
    }

    Ok(())
}

async fn security_viewer(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    can_write: bool,
) -> Result<SecurityViewer, RepositoryError> {
    Ok(SecurityViewer {
        permission: viewer_permission(pool, repository, actor_user_id, can_write).await?,
        can_read: true,
        can_write,
        can_edit_policy: can_write && !repository.is_archived,
        can_view_private_alert_counts: can_write,
    })
}

fn security_repository(
    repository: &Repository,
    _links: &DependabotAlertLinks,
) -> RepositorySecurityRepository {
    RepositorySecurityRepository {
        id: repository.id,
        owner_login: repository.owner_login.clone(),
        name: repository.name.clone(),
        visibility: repository.visibility.as_str().to_owned(),
        default_branch: repository.default_branch.clone(),
        security_href: format!("/{}/{}/security", repository.owner_login, repository.name),
        policy_href: format!(
            "/{}/{}/security/policy",
            repository.owner_login, repository.name
        ),
        advisories_href: format!(
            "/{}/{}/security/advisories",
            repository.owner_login, repository.name
        ),
    }
}

async fn dependabot_setting(
    pool: &PgPool,
    repository: &Repository,
) -> Result<Option<sqlx::postgres::PgRow>, RepositoryError> {
    sqlx::query(
        r#"
        SELECT status, summary, config_href
        FROM repository_security_feature_settings
        WHERE repository_id = $1 AND feature_key = 'dependabot'
        "#,
    )
    .bind(repository.id)
    .fetch_optional(pool)
    .await
    .map_err(RepositoryError::from)
}

fn dependabot_availability(
    repository: &Repository,
    setting: Option<&sqlx::postgres::PgRow>,
) -> DependabotAlertsAvailability {
    let status = setting
        .map(|row| row.get::<String, _>("status"))
        .unwrap_or_else(|| "enabled".to_owned());
    let enabled = status == "enabled";
    DependabotAlertsAvailability {
        enabled,
        indexed: enabled,
        message: if enabled {
            "Dependabot alerts are derived from indexed dependency manifests and advisories."
                .to_owned()
        } else {
            setting
                .map(|row| row.get::<String, _>("summary"))
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "Dependabot alerts are disabled for this repository.".to_owned())
        },
        disabled_reason: (!enabled).then_some(status),
        settings_href: setting
            .and_then(|row| row.get::<Option<String>, _>("config_href"))
            .or_else(|| {
                Some(format!(
                    "/{}/{}/settings/security_analysis",
                    repository.owner_login, repository.name
                ))
            }),
    }
}

async fn materialize_dependabot_alerts(
    pool: &PgPool,
    repository: &Repository,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        WITH candidates AS (
            SELECT repository_dependencies.id AS repository_dependency_id,
                   dependency_advisories.id AS dependency_advisory_id,
                   repository_dependencies.relationship,
                   repository_dependencies.package_version,
                   dependency_advisories.advisory_identifier,
                   row_number() OVER (
                       ORDER BY dependency_advisories.severity DESC,
                                lower(dependency_advisories.advisory_identifier),
                                repository_dependencies.id
                   ) AS ordinal
            FROM repository_dependencies
            JOIN dependency_advisories ON dependency_advisories.package_id = repository_dependencies.package_id
            WHERE repository_dependencies.repository_id = $1
        ),
        numbered AS (
            SELECT candidates.*,
                   COALESCE((SELECT max(number) FROM dependabot_alerts WHERE repository_id = $1), 0)
                   + candidates.ordinal AS generated_number
            FROM candidates
        )
        INSERT INTO dependabot_alerts (
            repository_id,
            repository_dependency_id,
            dependency_advisory_id,
            number,
            scope,
            vulnerable_requirements,
            fixed_version
        )
        SELECT $1,
               repository_dependency_id,
               dependency_advisory_id,
               generated_number,
               CASE WHEN relationship = 'direct' THEN 'production' ELSE 'development' END,
               COALESCE(package_version, 'installed version'),
               NULL
        FROM numbered
        ON CONFLICT (repository_id, repository_dependency_id, dependency_advisory_id) DO NOTHING
        "#,
    )
    .bind(repository.id)
    .execute(pool)
    .await?;
    Ok(())
}

fn normalize_dependabot_alert_filters(
    query: DependabotAlertsQuery<'_>,
) -> Result<DependabotAlertFilters, RepositoryError> {
    let state = match query.state.map(str::trim).filter(|value| !value.is_empty()) {
        Some(state @ ("open" | "closed" | "dismissed" | "fixed" | "all")) => state.to_owned(),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported dependabot alert state `{other}`"
            )))
        }
        None => "open".to_owned(),
    };
    let query_text = normalize_optional_filter(query.query, "q", 120)?;
    let package = normalize_optional_filter(query.package, "package", 160)?;
    let manifest = normalize_optional_filter(query.manifest, "manifest", 240)?;
    let ecosystem = match query
        .ecosystem
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(ecosystem @ ("npm" | "cargo" | "pip")) => Some(ecosystem.to_owned()),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported ecosystem `{other}`"
            )))
        }
        None => None,
    };
    let scope = match query.scope.map(str::trim).filter(|value| !value.is_empty()) {
        Some(scope @ ("production" | "development")) => Some(scope.to_owned()),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported scope `{other}`"
            )))
        }
        None => None,
    };
    let severity = match query
        .severity
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(severity @ ("low" | "moderate" | "high" | "critical")) => Some(severity.to_owned()),
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported severity `{other}`"
            )))
        }
        None => None,
    };
    let sort = match query.sort.map(str::trim).filter(|value| !value.is_empty()) {
        Some(sort @ ("most_important" | "recently_detected" | "package" | "manifest")) => {
            sort.to_owned()
        }
        Some(other) => {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "unsupported sort `{other}`"
            )))
        }
        None => "most_important".to_owned(),
    };
    Ok(DependabotAlertFilters {
        state,
        query: query_text,
        package,
        ecosystem,
        manifest,
        scope,
        severity,
        sort,
    })
}

fn normalize_optional_filter(
    value: Option<&str>,
    label: &str,
    max_chars: usize,
) -> Result<Option<String>, RepositoryError> {
    let value = value.map(str::trim).filter(|value| !value.is_empty());
    if let Some(value) = value {
        if value.chars().count() > max_chars {
            return Err(RepositoryError::InvalidDependencyGraphQuery(format!(
                "{label} must be {max_chars} characters or fewer"
            )));
        }
        return Ok(Some(value.to_owned()));
    }
    Ok(None)
}

fn normalize_dependabot_dismissal_reason(value: Option<&str>) -> Result<String, RepositoryError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err(RepositoryError::InvalidDependencyGraphQuery(
            "dismissal reason is required".to_owned(),
        ));
    };
    match value {
        "fix_started" | "inaccurate" | "no_bandwidth" | "not_used" | "tolerable_risk" => {
            Ok(value.to_owned())
        }
        other => Err(RepositoryError::InvalidDependencyGraphQuery(format!(
            "unsupported dismissal reason `{other}`"
        ))),
    }
}

fn normalize_dependabot_dismissal_comment(
    value: Option<&str>,
) -> Result<Option<String>, RepositoryError> {
    let value = value.map(str::trim).filter(|value| !value.is_empty());
    if let Some(value) = value {
        if value.chars().count() > 500 {
            return Err(RepositoryError::InvalidDependencyGraphQuery(
                "dismissal comment must be 500 characters or fewer".to_owned(),
            ));
        }
        return Ok(Some(value.to_owned()));
    }
    Ok(None)
}

async fn dependabot_alert_rows(
    pool: &PgPool,
    repository: &Repository,
) -> Result<Vec<DependabotAlertRow>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT dependabot_alerts.id,
               dependabot_alerts.number,
               dependabot_alerts.state,
               dependabot_alerts.scope,
               dependabot_alerts.vulnerable_requirements,
               dependabot_alerts.fixed_version,
               dependabot_alerts.created_at,
               dependabot_alerts.updated_at,
               repository_dependencies.package_version,
               repository_dependencies.relationship,
               repository_dependencies.lockfile_path,
               dependency_manifests.path AS manifest_path,
               dependency_packages.id AS package_id,
               dependency_packages.ecosystem,
               dependency_packages.name,
               dependency_packages.package_href,
               dependency_advisories.id AS advisory_id,
               dependency_advisories.advisory_identifier,
               dependency_advisories.severity,
               dependency_advisories.title,
               dependency_advisories.advisory_href,
               dependency_advisories.published_at
        FROM dependabot_alerts
        JOIN repository_dependencies ON repository_dependencies.id = dependabot_alerts.repository_dependency_id
        JOIN dependency_manifests ON dependency_manifests.id = repository_dependencies.manifest_id
        JOIN dependency_packages ON dependency_packages.id = repository_dependencies.package_id
        JOIN dependency_advisories ON dependency_advisories.id = dependabot_alerts.dependency_advisory_id
        WHERE dependabot_alerts.repository_id = $1
        ORDER BY dependabot_alerts.number ASC
        LIMIT 250
        "#,
    )
    .bind(repository.id)
    .fetch_all(pool)
    .await?;

    let mut alerts = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let ecosystem: String = row.get("ecosystem");
        let package_name: String = row.get("name");
        let manifest_path: String = row.get("manifest_path");
        let lockfile_path: Option<String> = row.get("lockfile_path");
        let package_href = row
            .get::<Option<String>, _>("package_href")
            .unwrap_or_else(|| package_href(&ecosystem, &package_name));
        alerts.push(DependabotAlertRow {
            id,
            number: row.get("number"),
            state: row.get("state"),
            scope: row.get("scope"),
            package: DependabotAlertPackage {
                id: row.get("package_id"),
                ecosystem: ecosystem.clone(),
                name: package_name.clone(),
                href: package_href,
            },
            advisory: DependabotAlertAdvisorySummary {
                id: row.get("advisory_id"),
                identifier: row.get("advisory_identifier"),
                severity: row.get("severity"),
                title: row.get("title"),
                href: row.get("advisory_href"),
                published_at: row.get("published_at"),
            },
            manifest_path: manifest_path.clone(),
            manifest_href: repository_blob_href(
                repository,
                &repository.default_branch,
                &manifest_path,
            ),
            lockfile_path: lockfile_path.clone(),
            lockfile_href: lockfile_path
                .as_deref()
                .map(|path| repository_blob_href(repository, &repository.default_branch, path)),
            vulnerable_requirements: row.get("vulnerable_requirements"),
            current_version: row.get("package_version"),
            fixed_version: row.get("fixed_version"),
            relationship: row.get("relationship"),
            assignees: dependabot_alert_assignees(pool, id).await?,
            href: format!(
                "/{}/{}/security/dependabot/{}",
                repository.owner_login,
                repository.name,
                row.get::<i64, _>("number")
            ),
            detected_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });
    }
    Ok(alerts)
}

async fn dependabot_alert_assignees(
    pool: &PgPool,
    alert_id: Uuid,
) -> Result<Vec<DependabotAlertAssignee>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.avatar_url
        FROM dependabot_alert_assignees
        JOIN users ON users.id = dependabot_alert_assignees.user_id
        WHERE dependabot_alert_assignees.alert_id = $1
        ORDER BY lower(COALESCE(NULLIF(users.username, ''), users.email)) ASC
        "#,
    )
    .bind(alert_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let login: String = row.get("login");
            DependabotAlertAssignee {
                id: row.get("id"),
                href: format!("/{login}"),
                login,
                avatar_url: row.get("avatar_url"),
            }
        })
        .collect())
}

fn apply_dependabot_alert_filters(
    alerts: &mut Vec<DependabotAlertRow>,
    filters: &DependabotAlertFilters,
) {
    alerts.retain(|alert| match filters.state.as_str() {
        "open" => alert.state == "open",
        "closed" => alert.state == "dismissed" || alert.state == "fixed",
        "dismissed" => alert.state == "dismissed",
        "fixed" => alert.state == "fixed",
        "all" => true,
        _ => true,
    });
    if let Some(query) = filters.query.as_deref() {
        let needle = query.to_lowercase();
        alerts.retain(|alert| {
            alert.package.name.to_lowercase().contains(&needle)
                || alert.advisory.title.to_lowercase().contains(&needle)
                || alert.advisory.identifier.to_lowercase().contains(&needle)
                || alert.manifest_path.to_lowercase().contains(&needle)
        });
    }
    if let Some(package) = filters.package.as_deref() {
        alerts.retain(|alert| {
            alert.package.name.eq_ignore_ascii_case(package)
                || format!("{}:{}", alert.package.ecosystem, alert.package.name)
                    .eq_ignore_ascii_case(package)
        });
    }
    if let Some(ecosystem) = filters.ecosystem.as_deref() {
        alerts.retain(|alert| alert.package.ecosystem == ecosystem);
    }
    if let Some(manifest) = filters.manifest.as_deref() {
        alerts.retain(|alert| alert.manifest_path.eq_ignore_ascii_case(manifest));
    }
    if let Some(scope) = filters.scope.as_deref() {
        alerts.retain(|alert| alert.scope == scope);
    }
    if let Some(severity) = filters.severity.as_deref() {
        alerts.retain(|alert| alert.advisory.severity == severity);
    }
}

fn sort_dependabot_alerts(alerts: &mut [DependabotAlertRow], sort: &str) {
    alerts.sort_by(|left, right| match sort {
        "recently_detected" => right
            .detected_at
            .cmp(&left.detected_at)
            .then(left.number.cmp(&right.number)),
        "package" => left
            .package
            .name
            .to_lowercase()
            .cmp(&right.package.name.to_lowercase()),
        "manifest" => left
            .manifest_path
            .to_lowercase()
            .cmp(&right.manifest_path.to_lowercase()),
        _ => severity_rank(&left.advisory.severity)
            .cmp(&severity_rank(&right.advisory.severity))
            .then(left.number.cmp(&right.number)),
    });
}

fn severity_rank(severity: &str) -> i32 {
    match severity {
        "critical" => 0,
        "high" => 1,
        "moderate" => 2,
        "low" => 3,
        _ => 4,
    }
}

fn dependabot_counts(alerts: &[DependabotAlertRow], visible: i64) -> DependabotAlertCounts {
    let open = alerts.iter().filter(|alert| alert.state == "open").count() as i64;
    let closed = alerts
        .iter()
        .filter(|alert| alert.state == "dismissed" || alert.state == "fixed")
        .count() as i64;
    DependabotAlertCounts {
        open,
        closed,
        total: alerts.len() as i64,
        visible,
    }
}

async fn dependabot_package_filters(
    _repository: &Repository,
    alerts: &[DependabotAlertRow],
    selected: Option<&str>,
) -> Result<Vec<DependabotAlertPackageFilter>, RepositoryError> {
    let mut packages = Vec::<DependabotAlertPackageFilter>::new();
    for alert in alerts.iter().filter(|alert| alert.state == "open") {
        if let Some(existing) = packages
            .iter_mut()
            .find(|entry| entry.package.id == alert.package.id)
        {
            existing.open_count += 1;
        } else {
            packages.push(DependabotAlertPackageFilter {
                package: alert.package.clone(),
                open_count: 1,
                selected: selected
                    .map(|value| {
                        value.eq_ignore_ascii_case(&alert.package.name)
                            || value.eq_ignore_ascii_case(&format!(
                                "{}:{}",
                                alert.package.ecosystem, alert.package.name
                            ))
                    })
                    .unwrap_or(false),
            });
        }
    }
    packages.sort_by(|left, right| {
        right
            .open_count
            .cmp(&left.open_count)
            .then(left.package.name.cmp(&right.package.name))
    });
    Ok(packages)
}

async fn dependabot_manifest_filters(
    repository: &Repository,
    alerts: &[DependabotAlertRow],
    selected: Option<&str>,
) -> Result<Vec<DependabotAlertManifestFilter>, RepositoryError> {
    let mut manifests = Vec::<DependabotAlertManifestFilter>::new();
    for alert in alerts.iter().filter(|alert| alert.state == "open") {
        if let Some(existing) = manifests
            .iter_mut()
            .find(|entry| entry.path == alert.manifest_path)
        {
            existing.open_count += 1;
        } else {
            manifests.push(DependabotAlertManifestFilter {
                path: alert.manifest_path.clone(),
                ecosystem: alert.package.ecosystem.clone(),
                href: repository_blob_href(
                    repository,
                    &repository.default_branch,
                    &alert.manifest_path,
                ),
                open_count: 1,
                selected: selected
                    .map(|value| value.eq_ignore_ascii_case(&alert.manifest_path))
                    .unwrap_or(false),
            });
        }
    }
    manifests.sort_by(|left, right| left.path.to_lowercase().cmp(&right.path.to_lowercase()));
    Ok(manifests)
}

async fn dependabot_alert_timeline(
    pool: &PgPool,
    alert_id: Uuid,
) -> Result<Vec<DependabotAlertTimelineEvent>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT security_alert_events.id,
               security_alert_events.event_type,
               security_alert_events.message,
               security_alert_events.created_at,
               users.id AS actor_id,
               COALESCE(NULLIF(users.username, ''), users.email) AS actor_login,
               users.avatar_url AS actor_avatar_url
        FROM security_alert_events
        LEFT JOIN users ON users.id = security_alert_events.actor_user_id
        WHERE security_alert_events.alert_id = $1
        ORDER BY security_alert_events.created_at ASC
        "#,
    )
    .bind(alert_id)
    .fetch_all(pool)
    .await?;

    let mut events = Vec::new();
    for row in rows {
        let actor = match (
            row.try_get::<Option<Uuid>, _>("actor_id").ok().flatten(),
            row.try_get::<Option<String>, _>("actor_login")
                .ok()
                .flatten(),
        ) {
            (Some(id), Some(login)) => Some(DependabotAlertAssignee {
                id,
                href: format!("/{login}"),
                login,
                avatar_url: row.get("actor_avatar_url"),
            }),
            _ => None,
        };
        events.push(DependabotAlertTimelineEvent {
            id: row.get("id"),
            event_type: row.get("event_type"),
            message: row.get("message"),
            actor,
            created_at: row.get("created_at"),
        });
    }
    if events.is_empty() {
        events.push(DependabotAlertTimelineEvent {
            id: alert_id,
            event_type: "created".to_owned(),
            message: "Dependabot opened this alert from the dependency graph.".to_owned(),
            actor: None,
            created_at: Utc::now(),
        });
    }
    Ok(events)
}

async fn record_dependabot_alert_event(
    pool: &PgPool,
    repository: &Repository,
    alert_id: Uuid,
    actor_user_id: Uuid,
    event_type: &str,
    message: &str,
    metadata: serde_json::Value,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO security_alert_events (
            repository_id, alert_id, actor_user_id, event_type, message, metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(repository.id)
    .bind(alert_id)
    .bind(actor_user_id)
    .bind(event_type)
    .bind(message)
    .bind(metadata.clone())
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO security_audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'repository.dependabot_alert.update', 'repository', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(repository.id)
    .bind(json!({
        "repositoryId": repository.id,
        "alertId": alert_id,
        "alertEvent": event_type,
        "metadata": metadata,
    }))
    .execute(pool)
    .await?;

    Ok(())
}

async fn notify_dependabot_alert_assignees(
    pool: &PgPool,
    repository: &Repository,
    alert_id: Uuid,
    title: &str,
    reason: &str,
) -> Result<(), RepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO notifications (
            user_id, repository_id, subject_type, subject_id, title, reason
        )
        SELECT dependabot_alert_assignees.user_id,
               $2,
               'dependabot_alert',
               $1,
               $3,
               $4
        FROM dependabot_alert_assignees
        WHERE dependabot_alert_assignees.alert_id = $1
        "#,
    )
    .bind(alert_id)
    .bind(repository.id)
    .bind(title)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

async fn dependabot_assignment_options(
    pool: &PgPool,
    repository: &Repository,
    alert_id: Uuid,
) -> Result<Vec<DependabotAlertAssignmentOption>, RepositoryError> {
    let rows = sqlx::query(
        r#"
        SELECT users.id,
               COALESCE(NULLIF(users.username, ''), users.email) AS login,
               users.avatar_url,
               EXISTS (
                   SELECT 1 FROM dependabot_alert_assignees
                   WHERE alert_id = $2 AND user_id = users.id
               ) AS selected
        FROM users
        WHERE users.id = $3
           OR EXISTS (
               SELECT 1 FROM repository_permissions
               WHERE repository_permissions.repository_id = $1
                 AND repository_permissions.user_id = users.id
           )
        ORDER BY lower(COALESCE(NULLIF(users.username, ''), users.email)) ASC
        LIMIT 25
        "#,
    )
    .bind(repository.id)
    .bind(alert_id)
    .bind(repository.created_by_user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| DependabotAlertAssignmentOption {
            id: row.get("id"),
            kind: "user".to_owned(),
            login: row.get("login"),
            avatar_url: row.get("avatar_url"),
            selected: row.get("selected"),
        })
        .collect())
}

fn dependabot_security_update_state(
    repository: &Repository,
    alert: &DependabotAlertRow,
) -> DependabotSecurityUpdateState {
    let supported = alert.state == "open"
        && matches!(alert.package.ecosystem.as_str(), "npm" | "cargo" | "pip");
    DependabotSecurityUpdateState {
        supported,
        status: if supported {
            "available"
        } else {
            "unsupported"
        }
        .to_owned(),
        href: supported.then(|| {
            format!(
                "/api/repos/{}/{}/security/dependabot/{}/security-update",
                percent_encode_segment(&repository.owner_login),
                percent_encode_segment(&repository.name),
                alert.number
            )
        }),
        pull_request_href: None,
        message: if supported {
            "A security update pull request can be prepared for this manifest.".to_owned()
        } else {
            "Security update pull requests are unavailable for this alert state or ecosystem."
                .to_owned()
        },
    }
}

fn dependabot_links(repository: &Repository) -> DependabotAlertLinks {
    let base = format!(
        "/{}/{}/security/dependabot",
        repository.owner_login, repository.name
    );
    DependabotAlertLinks {
        list_href: base.clone(),
        open_href: format!("{base}?state=open"),
        closed_href: format!("{base}?state=closed"),
        settings_href: format!(
            "/{}/{}/settings/security_analysis",
            repository.owner_login, repository.name
        ),
    }
}

fn package_href(ecosystem: &str, name: &str) -> String {
    match ecosystem {
        "npm" => format!(
            "https://www.npmjs.com/package/{}",
            percent_encode_segment(name)
        ),
        "cargo" => format!("https://crates.io/crates/{}", percent_encode_segment(name)),
        "pip" => format!("https://pypi.org/project/{}", percent_encode_segment(name)),
        _ => format!(
            "/packages/{}/{}",
            percent_encode_segment(ecosystem),
            percent_encode_segment(name)
        ),
    }
}

fn markdown_sha(markdown: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(markdown.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn markdown_error(error: super::markdown::MarkdownError) -> RepositoryError {
    RepositoryError::Sqlx(sqlx::Error::Protocol(error.to_string()))
}

async fn write_security_policy(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    mutation: SecurityPolicyMutation,
) -> Result<(), RepositoryError> {
    let path = normalize_policy_path(mutation.path.as_deref())?;
    let ref_name = normalize_policy_ref(repository, mutation.ref_name.as_deref())?;
    let markdown = normalize_policy_markdown(&mutation.markdown)?;
    let commit_message = normalize_policy_commit_message(&mutation.commit_message)?;
    let rendered = render_markdown(
        Some(pool),
        RenderMarkdownInput {
            markdown: markdown.clone(),
            repository_id: Some(repository.id),
            owner: Some(repository.owner_login.clone()),
            repo: Some(repository.name.clone()),
            ref_name: Some(ref_name.clone()),
            enable_task_toggles: Some(false),
        },
    )
    .await
    .map_err(markdown_error)?;
    let content_sha = markdown_sha(&markdown);

    let current_ref = current_branch_commit(pool, repository.id, &ref_name).await?;
    let existing_policy = current_security_policy_file(pool, repository, &ref_name).await?;
    if let Some(expected) = mutation
        .expected_content_sha
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let current_sha = existing_policy
            .as_ref()
            .map(|file| markdown_sha(&file.content))
            .unwrap_or_default();
        if expected != current_sha {
            return Err(RepositoryError::SecurityPolicyConflict);
        }
    }

    let mut files =
        current_branch_files(pool, repository.id, current_ref.as_ref().map(|r| r.id)).await?;
    if let Some(file) = files
        .iter_mut()
        .find(|file| file.path.eq_ignore_ascii_case(&path))
    {
        file.content = markdown.clone();
        file.oid = deterministic_content_oid("blob", &markdown);
        file.byte_size = markdown.len() as i64;
    } else {
        files.push(RepositorySnapshotFile {
            path: path.clone(),
            content: markdown.clone(),
            oid: deterministic_content_oid("blob", &markdown),
            byte_size: markdown.len() as i64,
        });
    }
    files.sort_by(|left, right| left.path.to_lowercase().cmp(&right.path.to_lowercase()));

    let tree_oid = deterministic_content_oid(
        "tree",
        &files
            .iter()
            .map(|file| format!("{}:{}:{}", file.path, file.oid, file.byte_size))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    let parent_oids = current_ref
        .as_ref()
        .map(|commit| vec![commit.oid.clone()])
        .unwrap_or_default();
    let commit_oid = deterministic_content_oid(
        "commit",
        &format!(
            "{}:{}:{}:{}:{}",
            repository.id, ref_name, tree_oid, commit_message, content_sha
        ),
    );
    let commit = replace_repository_snapshot(
        pool,
        repository.id,
        RepositorySnapshot {
            commit: CreateCommit {
                oid: commit_oid.clone(),
                author_user_id: Some(actor_user_id),
                committer_user_id: Some(actor_user_id),
                message: commit_message.clone(),
                tree_oid: Some(tree_oid),
                parent_oids,
                committed_at: Utc::now(),
            },
            branch_name: ref_name.clone(),
            files,
        },
    )
    .await?;

    sqlx::query(
        r#"
        INSERT INTO repository_security_policies (
            repository_id, path, ref_name, source_commit_id, blob_oid, content_sha,
            markdown, rendered_html, published, updated_by_user_id, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, now())
        ON CONFLICT (repository_id, lower(path))
        DO UPDATE SET ref_name = EXCLUDED.ref_name,
                      source_commit_id = EXCLUDED.source_commit_id,
                      blob_oid = EXCLUDED.blob_oid,
                      content_sha = EXCLUDED.content_sha,
                      markdown = EXCLUDED.markdown,
                      rendered_html = EXCLUDED.rendered_html,
                      published = true,
                      updated_by_user_id = EXCLUDED.updated_by_user_id,
                      updated_at = now()
        "#,
    )
    .bind(repository.id)
    .bind(&path)
    .bind(&ref_name)
    .bind(commit.id)
    .bind(deterministic_content_oid("blob", &markdown))
    .bind(&content_sha)
    .bind(&markdown)
    .bind(&rendered.html)
    .bind(actor_user_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO security_audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'repository.security_policy.upsert', 'repository', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(repository.id)
    .bind(json!({
        "repositoryId": repository.id,
        "path": path,
        "ref": ref_name,
        "commitOid": commit.oid,
        "contentSha": content_sha,
    }))
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(Debug)]
struct CurrentPolicyFile {
    content: String,
}

#[derive(Debug)]
struct CurrentBranchCommit {
    id: Uuid,
    oid: String,
}

async fn current_branch_commit(
    pool: &PgPool,
    repository_id: Uuid,
    ref_name: &str,
) -> Result<Option<CurrentBranchCommit>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT commits.id, commits.oid
        FROM repository_git_refs
        JOIN commits ON commits.id = repository_git_refs.target_commit_id
        WHERE repository_git_refs.repository_id = $1
          AND repository_git_refs.name IN ($2, 'refs/heads/' || $2)
        ORDER BY CASE WHEN repository_git_refs.name = 'refs/heads/' || $2 THEN 0 ELSE 1 END
        LIMIT 1
        "#,
    )
    .bind(repository_id)
    .bind(ref_name)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| CurrentBranchCommit {
        id: row.get("id"),
        oid: row.get("oid"),
    }))
}

async fn current_branch_files(
    pool: &PgPool,
    repository_id: Uuid,
    commit_id: Option<Uuid>,
) -> Result<Vec<RepositorySnapshotFile>, RepositoryError> {
    let Some(commit_id) = commit_id else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query(
        r#"
        SELECT path, content, oid, byte_size
        FROM repository_files
        WHERE repository_id = $1 AND commit_id = $2
        ORDER BY lower(path)
        "#,
    )
    .bind(repository_id)
    .bind(commit_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| RepositorySnapshotFile {
            path: row.get("path"),
            content: row.get("content"),
            oid: row.get("oid"),
            byte_size: row.get("byte_size"),
        })
        .collect())
}

async fn current_security_policy_file(
    pool: &PgPool,
    repository: &Repository,
    ref_name: &str,
) -> Result<Option<CurrentPolicyFile>, RepositoryError> {
    let row = sqlx::query(
        r#"
        SELECT repository_files.content
        FROM repository_files
        JOIN repository_git_refs
          ON repository_git_refs.repository_id = repository_files.repository_id
         AND repository_git_refs.target_commit_id = repository_files.commit_id
        WHERE repository_files.repository_id = $1
          AND repository_git_refs.name IN ($2, 'refs/heads/' || $2)
          AND lower(repository_files.path) IN ('security.md', '.github/security.md', 'docs/security.md')
        ORDER BY CASE lower(repository_files.path)
            WHEN 'security.md' THEN 0
            WHEN '.github/security.md' THEN 1
            ELSE 2
        END
        LIMIT 1
        "#,
    )
    .bind(repository.id)
    .bind(ref_name)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| CurrentPolicyFile {
        content: row.get("content"),
    }))
}

fn normalize_policy_path(path: Option<&str>) -> Result<String, RepositoryError> {
    let path = path.unwrap_or("SECURITY.md").trim();
    let normalized = if path.is_empty() { "SECURITY.md" } else { path };
    match normalized.to_ascii_lowercase().as_str() {
        "security.md" | ".github/security.md" | "docs/security.md" => Ok(normalized.to_owned()),
        _ => Err(RepositoryError::InvalidSecurityPolicy(
            "policy path must be SECURITY.md, .github/SECURITY.md, or docs/SECURITY.md".to_owned(),
        )),
    }
}

fn normalize_policy_ref(
    repository: &Repository,
    ref_name: Option<&str>,
) -> Result<String, RepositoryError> {
    let ref_name = ref_name.unwrap_or(&repository.default_branch).trim();
    let ref_name = ref_name.strip_prefix("refs/heads/").unwrap_or(ref_name);
    if ref_name.is_empty() || ref_name.contains("..") || ref_name.starts_with('/') {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "policy branch is invalid".to_owned(),
        ));
    }
    Ok(ref_name.to_owned())
}

fn normalize_policy_markdown(markdown: &str) -> Result<String, RepositoryError> {
    let markdown = markdown.trim();
    if markdown.is_empty() {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "policy markdown must not be empty".to_owned(),
        ));
    }
    if markdown.len() > 128 * 1024 {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "policy markdown must be 128 KiB or smaller".to_owned(),
        ));
    }
    Ok(markdown.to_owned())
}

fn normalize_policy_commit_message(message: &str) -> Result<String, RepositoryError> {
    let message = message.trim();
    if message.is_empty() {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "commit message must not be empty".to_owned(),
        ));
    }
    if message.len() > 240 {
        return Err(RepositoryError::InvalidSecurityPolicy(
            "commit message must be 240 characters or fewer".to_owned(),
        ));
    }
    Ok(message.to_owned())
}

fn deterministic_content_oid(kind: &str, content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update([0]);
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn policy_heading_outline(html: &str) -> Vec<SecurityPolicyHeading> {
    Regex::new(r#"<h([1-6]) id="([^"]+)">(.*?)</h[1-6]>"#)
        .expect("heading outline regex")
        .captures_iter(html)
        .map(|captures| {
            let level = captures[1].parse::<i32>().unwrap_or(1);
            let id = captures[2].to_owned();
            let text = strip_tags(&captures[3])
                .trim()
                .trim_start_matches('#')
                .trim()
                .to_owned();
            SecurityPolicyHeading {
                href: format!("#{id}"),
                id,
                level,
                text,
            }
        })
        .collect()
}

fn strip_tags(value: &str) -> String {
    Regex::new(r"<[^>]+>")
        .expect("tag regex")
        .replace_all(value, |captures: &Captures<'_>| {
            if captures[0].starts_with("</") {
                " "
            } else {
                ""
            }
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn repository_blob_href(repository: &Repository, ref_name: &str, path: &str) -> String {
    format!(
        "/{}/{}/blob/{}/{}",
        repository.owner_login,
        repository.name,
        percent_encode_segment(ref_name),
        percent_encode_path(path)
    )
}

fn repository_raw_href(repository: &Repository, ref_name: &str, path: &str) -> String {
    format!(
        "/{}/{}/raw/{}/{}",
        repository.owner_login,
        repository.name,
        percent_encode_segment(ref_name),
        percent_encode_path(path)
    )
}

fn repository_history_href(repository: &Repository, ref_name: &str, path: &str) -> String {
    format!(
        "/{}/{}/commits/{}/{}",
        repository.owner_login,
        repository.name,
        percent_encode_segment(ref_name),
        percent_encode_path(path)
    )
}

fn percent_encode_segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

fn percent_encode_path(path: &str) -> String {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .map(percent_encode_segment)
        .collect::<Vec<_>>()
        .join("/")
}
