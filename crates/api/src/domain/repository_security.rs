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
