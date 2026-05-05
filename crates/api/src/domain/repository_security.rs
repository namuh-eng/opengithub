use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::{
    markdown::{render_markdown, RenderMarkdownInput},
    repositories::{
        can_read_repository, can_write_repository, get_repository_by_owner_name, Repository,
        RepositoryError, RepositoryVisibility,
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
        SELECT path, ref_name, blob_oid, content_sha, markdown, rendered_html, updated_at
        FROM repository_security_policies
        WHERE repository_id = $1 AND published = true
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
