use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    domain::webhooks::{enqueue_repository_webhook_event, WebhookError},
    jobs::{acquire_job_lease, complete_job_lease, fail_job_lease, JobLeaseError},
};

const PAGES_QUEUE: &str = "pages-build-deploy";
const PAGES_LEASE_SECONDS: i64 = 120;
const MAX_BUILD_LOG_BYTES: usize = 4096;

#[derive(Debug, thiserror::Error)]
pub enum PagesBuildWorkerError {
    #[error("Pages deployment was not found")]
    NotFound,
    #[error("Pages deployment failed: {0}")]
    Build(String),
    #[error(transparent)]
    JobLease(#[from] JobLeaseError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Webhook(#[from] WebhookError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PagesBuildWorkerResult {
    pub deployment_id: Uuid,
    pub status: String,
    pub conclusion: Option<String>,
    pub artifact_count: i64,
    pub storage_key: Option<String>,
    pub failure_reason: Option<String>,
}

struct PagesDeploymentWorkItem {
    id: Uuid,
    repository_id: Uuid,
    site_id: Uuid,
    source_kind: String,
    source_branch: Option<String>,
    source_folder: Option<String>,
    commit_id: Option<Uuid>,
    workflow_run_id: Option<Uuid>,
    workflow_artifact_id: Option<Uuid>,
    status: String,
    requested_by_user_id: Option<Uuid>,
    default_url: String,
    custom_domain_url: Option<String>,
    custom_domain: Option<String>,
    dns_status: String,
}

struct PublishArtifact {
    path: String,
    storage_key: String,
    content_type: Option<String>,
    byte_size: i64,
    checksum: String,
}

pub async fn run_next_pages_build_deployment(
    pool: &PgPool,
    worker_id: &str,
) -> Result<Option<PagesBuildWorkerResult>, PagesBuildWorkerError> {
    let deployment_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT pages_deployments.id
        FROM pages_deployments
        JOIN job_leases ON job_leases.queue = $1
          AND job_leases.lease_key = pages_deployments.id::text
        WHERE pages_deployments.status IN ('queued', 'building')
          AND job_leases.completed_at IS NULL
          AND (job_leases.locked_until IS NULL OR job_leases.locked_until <= now())
        ORDER BY pages_deployments.queued_at ASC
        LIMIT 1
        "#,
    )
    .bind(PAGES_QUEUE)
    .fetch_optional(pool)
    .await?;

    let Some(deployment_id) = deployment_id else {
        return Ok(None);
    };
    run_pages_build_deployment_once(pool, deployment_id, worker_id).await
}

pub async fn run_pages_build_deployment_once(
    pool: &PgPool,
    deployment_id: Uuid,
    worker_id: &str,
) -> Result<Option<PagesBuildWorkerResult>, PagesBuildWorkerError> {
    let Some(lease) = acquire_job_lease(
        pool,
        PAGES_QUEUE,
        &deployment_id.to_string(),
        worker_id,
        PAGES_LEASE_SECONDS,
    )
    .await?
    else {
        return Ok(None);
    };

    let Some(work_item) = load_work_item(pool, deployment_id).await? else {
        fail_job_lease(pool, lease.id, worker_id, "pages_deployment_not_found", 300).await?;
        return Err(PagesBuildWorkerError::NotFound);
    };

    if !matches!(work_item.status.as_str(), "queued" | "building") {
        complete_job_lease(pool, lease.id, worker_id).await?;
        return Ok(Some(result_for_deployment(pool, deployment_id).await?));
    }

    mark_building(pool, deployment_id).await?;

    let publish_result = match work_item.source_kind.as_str() {
        "branch" => publish_branch_source(pool, &work_item).await,
        "actions" => publish_actions_artifact(pool, &work_item).await,
        other => Err(PagesBuildWorkerError::Build(format!(
            "unsupported Pages source `{other}`"
        ))),
    };

    match publish_result {
        Ok(artifacts) => {
            let result = record_success(pool, &work_item, artifacts).await?;
            complete_job_lease(pool, lease.id, worker_id).await?;
            enqueue_page_build_webhooks(pool, &work_item, &result).await?;
            Ok(Some(result))
        }
        Err(error) => {
            let reason = bounded_log(&error.to_string());
            let result = record_failure(pool, &work_item, &reason).await?;
            complete_job_lease(pool, lease.id, worker_id).await?;
            enqueue_page_build_webhooks(pool, &work_item, &result).await?;
            Ok(Some(result))
        }
    }
}

async fn load_work_item(
    pool: &PgPool,
    deployment_id: Uuid,
) -> Result<Option<PagesDeploymentWorkItem>, PagesBuildWorkerError> {
    let row = sqlx::query(
        r#"
        SELECT pages_deployments.id,
               pages_deployments.repository_id,
               pages_deployments.site_id,
               pages_deployments.source_kind,
               pages_deployments.source_branch,
               pages_deployments.source_folder,
               pages_deployments.commit_id,
               pages_deployments.workflow_run_id,
               pages_deployments.workflow_artifact_id,
               pages_deployments.status,
               pages_deployments.requested_by_user_id,
               pages_deployments.default_url,
               pages_deployments.custom_domain_url,
               pages_sites.custom_domain,
               pages_sites.dns_status
        FROM pages_deployments
        JOIN pages_sites ON pages_sites.id = pages_deployments.site_id
        WHERE pages_deployments.id = $1
        "#,
    )
    .bind(deployment_id)
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        Ok(PagesDeploymentWorkItem {
            id: row.try_get("id")?,
            repository_id: row.try_get("repository_id")?,
            site_id: row.try_get("site_id")?,
            source_kind: row.try_get("source_kind")?,
            source_branch: row.try_get("source_branch")?,
            source_folder: row.try_get("source_folder")?,
            commit_id: row.try_get("commit_id")?,
            workflow_run_id: row.try_get("workflow_run_id")?,
            workflow_artifact_id: row.try_get("workflow_artifact_id")?,
            status: row.try_get("status")?,
            requested_by_user_id: row.try_get("requested_by_user_id")?,
            default_url: row.try_get("default_url")?,
            custom_domain_url: row.try_get("custom_domain_url")?,
            custom_domain: row.try_get("custom_domain")?,
            dns_status: row.try_get("dns_status")?,
        })
    })
    .transpose()
}

async fn publish_branch_source(
    pool: &PgPool,
    work_item: &PagesDeploymentWorkItem,
) -> Result<Vec<PublishArtifact>, PagesBuildWorkerError> {
    let commit_id = work_item.commit_id.ok_or_else(|| {
        PagesBuildWorkerError::Build("branch deployment has no commit".to_owned())
    })?;
    let folder = work_item.source_folder.as_deref().unwrap_or("/");
    let rows = sqlx::query(
        r#"
        SELECT path, content, oid, byte_size
        FROM repository_files
        WHERE repository_id = $1 AND commit_id = $2
        ORDER BY lower(path)
        "#,
    )
    .bind(work_item.repository_id)
    .bind(commit_id)
    .fetch_all(pool)
    .await?;

    let mut artifacts = Vec::new();
    for row in rows {
        let source_path: String = row.try_get("path")?;
        let Some(public_path) = public_path_for_folder(&source_path, folder) else {
            continue;
        };
        if should_skip_static_file(&public_path) {
            continue;
        }
        let content: String = row.try_get("content")?;
        let byte_size: i64 = row.try_get("byte_size")?;
        artifacts.push(PublishArtifact {
            path: public_path.clone(),
            storage_key: storage_key(work_item.repository_id, work_item.id, &public_path),
            content_type: content_type_for_path(&public_path).map(str::to_owned),
            byte_size,
            checksum: checksum(&content),
        });
    }

    if artifacts.is_empty() {
        return Err(PagesBuildWorkerError::Build(
            "source folder did not contain publishable files".to_owned(),
        ));
    }
    if !artifacts
        .iter()
        .any(|artifact| artifact.path.eq_ignore_ascii_case("index.html"))
    {
        return Err(PagesBuildWorkerError::Build(
            "source folder must include index.html".to_owned(),
        ));
    }
    Ok(artifacts)
}

async fn publish_actions_artifact(
    pool: &PgPool,
    work_item: &PagesDeploymentWorkItem,
) -> Result<Vec<PublishArtifact>, PagesBuildWorkerError> {
    let artifact_id = work_item.workflow_artifact_id.ok_or_else(|| {
        PagesBuildWorkerError::Build("Actions deployment has no artifact".to_owned())
    })?;
    let row = sqlx::query(
        r#"
        SELECT name, digest, size_bytes, storage_key
        FROM workflow_artifacts
        WHERE id = $1 AND expired_at IS NULL
        "#,
    )
    .bind(artifact_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        PagesBuildWorkerError::Build("Pages artifact is missing or expired".to_owned())
    })?;
    let storage_key: Option<String> = row.try_get("storage_key")?;
    let storage_key = storage_key.ok_or_else(|| {
        PagesBuildWorkerError::Build("Pages artifact does not have storage metadata".to_owned())
    })?;
    let artifact_name: String = row.try_get("name")?;
    let digest: Option<String> = row.try_get("digest")?;
    Ok(vec![PublishArtifact {
        path: format!("{}.zip", artifact_name.trim().trim_end_matches(".zip")),
        storage_key: format!(
            "pages/{}/{}/actions/{}",
            work_item.repository_id, work_item.id, storage_key
        ),
        content_type: Some("application/zip".to_owned()),
        byte_size: row.try_get("size_bytes")?,
        checksum: digest.unwrap_or_else(|| checksum(&storage_key)),
    }])
}

async fn mark_building(pool: &PgPool, deployment_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE pages_deployments
        SET status = 'building', started_at = COALESCE(started_at, now())
        WHERE id = $1 AND status = 'queued'
        "#,
    )
    .bind(deployment_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn record_success(
    pool: &PgPool,
    work_item: &PagesDeploymentWorkItem,
    artifacts: Vec<PublishArtifact>,
) -> Result<PagesBuildWorkerResult, PagesBuildWorkerError> {
    let storage_prefix = format!("pages/{}/{}", work_item.repository_id, work_item.id);
    let artifact_count = artifacts.len() as i64;
    let total_bytes = artifacts
        .iter()
        .map(|artifact| artifact.byte_size)
        .sum::<i64>();
    let storage_mode = pages_storage_mode();
    let cloudfront_alias = confirmed_alias(work_item);
    let manifest = json!({
        "storageMode": storage_mode,
        "storagePrefix": storage_prefix,
        "sourceKind": work_item.source_kind,
        "sourceBranch": work_item.source_branch,
        "sourceFolder": work_item.source_folder,
        "workflowRunId": work_item.workflow_run_id,
        "workflowArtifactId": work_item.workflow_artifact_id,
        "artifactCount": artifact_count,
        "totalBytes": total_bytes,
        "files": artifacts.iter().map(|artifact| json!({
            "path": artifact.path,
            "byteSize": artifact.byte_size,
            "checksum": artifact.checksum,
            "contentType": artifact.content_type,
        })).collect::<Vec<_>>(),
        "cloud": {
            "s3": storage_mode == "s3",
            "cloudfrontAlias": cloudfront_alias,
            "customDomainReady": work_item.custom_domain.is_some() && work_item.dns_status == "verified",
        }
    });
    let build_log = bounded_log(&format!(
        "Published {artifact_count} Pages artifact(s) to {storage_prefix} using {storage_mode} storage metadata."
    ));

    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM pages_build_artifacts WHERE deployment_id = $1")
        .bind(work_item.id)
        .execute(&mut *tx)
        .await?;
    for artifact in &artifacts {
        sqlx::query(
            r#"
            INSERT INTO pages_build_artifacts (
                deployment_id, path, storage_key, content_type, byte_size, checksum
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(work_item.id)
        .bind(&artifact.path)
        .bind(&artifact.storage_key)
        .bind(&artifact.content_type)
        .bind(artifact.byte_size)
        .bind(&artifact.checksum)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(
        r#"
        UPDATE pages_deployments
        SET status = 'deployed',
            conclusion = 'success',
            artifact_storage_key = $2,
            artifact_manifest = $3,
            build_log_excerpt = $4,
            failure_reason = NULL,
            completed_at = now()
        WHERE id = $1
        "#,
    )
    .bind(work_item.id)
    .bind(&storage_prefix)
    .bind(&manifest)
    .bind(&build_log)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        UPDATE pages_sites
        SET provisioning_status = 'ready',
            s3_artifact_prefix = $2,
            cloudfront_alias = $3,
            last_deployment_id = $4,
            unpublished_at = NULL
        WHERE id = $1
        "#,
    )
    .bind(work_item.site_id)
    .bind(&storage_prefix)
    .bind(&cloudfront_alias)
    .bind(work_item.id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(PagesBuildWorkerResult {
        deployment_id: work_item.id,
        status: "deployed".to_owned(),
        conclusion: Some("success".to_owned()),
        artifact_count,
        storage_key: Some(storage_prefix),
        failure_reason: None,
    })
}

async fn record_failure(
    pool: &PgPool,
    work_item: &PagesDeploymentWorkItem,
    reason: &str,
) -> Result<PagesBuildWorkerResult, PagesBuildWorkerError> {
    let manifest = json!({
        "sourceKind": work_item.source_kind,
        "sourceBranch": work_item.source_branch,
        "sourceFolder": work_item.source_folder,
        "workflowRunId": work_item.workflow_run_id,
        "workflowArtifactId": work_item.workflow_artifact_id,
        "failureReason": reason,
    });
    let mut tx = pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE pages_deployments
        SET status = 'failed',
            conclusion = 'failure',
            artifact_manifest = $2,
            build_log_excerpt = $3,
            failure_reason = $3,
            completed_at = now()
        WHERE id = $1
        "#,
    )
    .bind(work_item.id)
    .bind(&manifest)
    .bind(reason)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        UPDATE pages_sites
        SET provisioning_status = 'failed',
            last_deployment_id = $2
        WHERE id = $1
        "#,
    )
    .bind(work_item.site_id)
    .bind(work_item.id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(PagesBuildWorkerResult {
        deployment_id: work_item.id,
        status: "failed".to_owned(),
        conclusion: Some("failure".to_owned()),
        artifact_count: 0,
        storage_key: None,
        failure_reason: Some(reason.to_owned()),
    })
}

async fn result_for_deployment(
    pool: &PgPool,
    deployment_id: Uuid,
) -> Result<PagesBuildWorkerResult, PagesBuildWorkerError> {
    let row = sqlx::query(
        r#"
        SELECT status, conclusion, artifact_storage_key, failure_reason,
               (SELECT count(*) FROM pages_build_artifacts WHERE deployment_id = $1) AS artifact_count
        FROM pages_deployments
        WHERE id = $1
        "#,
    )
    .bind(deployment_id)
    .fetch_optional(pool)
    .await?
    .ok_or(PagesBuildWorkerError::NotFound)?;
    Ok(PagesBuildWorkerResult {
        deployment_id,
        status: row.try_get("status")?,
        conclusion: row.try_get("conclusion")?,
        artifact_count: row.try_get("artifact_count")?,
        storage_key: row.try_get("artifact_storage_key")?,
        failure_reason: row.try_get("failure_reason")?,
    })
}

async fn enqueue_page_build_webhooks(
    pool: &PgPool,
    work_item: &PagesDeploymentWorkItem,
    result: &PagesBuildWorkerResult,
) -> Result<(), PagesBuildWorkerError> {
    let payload = json!({
        "deploymentId": result.deployment_id,
        "status": result.status,
        "conclusion": result.conclusion,
        "artifactCount": result.artifact_count,
        "defaultUrl": work_item.default_url,
        "customDomainUrl": work_item.custom_domain_url,
        "failureReason": result.failure_reason,
        "requestedByUserId": work_item.requested_by_user_id,
    });
    enqueue_repository_webhook_event(pool, work_item.repository_id, "page_build", payload).await?;
    Ok(())
}

fn public_path_for_folder(source_path: &str, folder: &str) -> Option<String> {
    let trimmed = source_path.trim_matches('/');
    match folder {
        "/" => Some(trimmed.to_owned()),
        "/docs" => trimmed
            .strip_prefix("docs/")
            .map(str::to_owned)
            .filter(|path| !path.is_empty()),
        _ => None,
    }
}

fn should_skip_static_file(path: &str) -> bool {
    path.is_empty() || path.starts_with(".git/") || path.starts_with(".github/")
}

fn storage_key(repository_id: Uuid, deployment_id: Uuid, path: &str) -> String {
    let clean = path
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != "." && *segment != "..")
        .collect::<Vec<_>>()
        .join("/");
    format!("pages/{repository_id}/{deployment_id}/{clean}")
}

fn content_type_for_path(path: &str) -> Option<&'static str> {
    match path.rsplit('.').next().unwrap_or_default() {
        "html" | "htm" => Some("text/html; charset=utf-8"),
        "css" => Some("text/css; charset=utf-8"),
        "js" => Some("text/javascript; charset=utf-8"),
        "json" => Some("application/json"),
        "svg" => Some("image/svg+xml"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "txt" => Some("text/plain; charset=utf-8"),
        _ => None,
    }
}

fn checksum(content: &str) -> String {
    let digest = Sha256::digest(content.as_bytes());
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn bounded_log(value: &str) -> String {
    if value.len() <= MAX_BUILD_LOG_BYTES {
        return value.to_owned();
    }
    value.chars().take(MAX_BUILD_LOG_BYTES).collect()
}

fn pages_storage_mode() -> String {
    std::env::var("PAGES_STORAGE_MODE")
        .ok()
        .filter(|value| value == "s3")
        .unwrap_or_else(|| "local_metadata".to_owned())
}

fn confirmed_alias(work_item: &PagesDeploymentWorkItem) -> Option<String> {
    if work_item.custom_domain.is_some() && work_item.dns_status == "verified" {
        work_item.custom_domain.clone()
    } else {
        None
    }
}
