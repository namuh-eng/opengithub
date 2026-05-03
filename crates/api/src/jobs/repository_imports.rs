use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tokio::process::Command;
use uuid::Uuid;

use crate::{
    domain::{
        notifications::{create_notification, CreateNotification},
        repositories::{
            replace_repository_snapshot, CreateCommit, RepositorySnapshot, RepositorySnapshotFile,
        },
        repository_imports::{
            get_repository_import_work_item, mark_repository_import_failed,
            mark_repository_import_imported, mark_repository_import_importing,
            validate_import_source_url, RepositoryImportStatus,
        },
    },
    jobs::{acquire_job_lease, complete_job_lease, fail_job_lease, JobLeaseError},
};

const IMPORT_QUEUE: &str = "repository_import";
const EMAIL_QUEUE: &str = "email_delivery";
const IMPORT_LEASE_SECONDS: i64 = 300;
const MAX_IMPORTED_FILES: usize = 1_000;
const MAX_IMPORTED_FILE_BYTES: usize = 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryImportWorkerError {
    #[error("repository import was not found")]
    NotFound,
    #[error("repository import failed: {0}")]
    Import(#[from] RepositoryImportProcessError),
    #[error(transparent)]
    RepositoryImport(#[from] crate::domain::repository_imports::RepositoryImportError),
    #[error(transparent)]
    Repository(#[from] crate::domain::repositories::RepositoryError),
    #[error(transparent)]
    JobLease(#[from] JobLeaseError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryImportProcessError {
    #[error("source URL is not supported")]
    UnsupportedSource,
    #[error("source URL points to a blocked host")]
    BlockedSource,
    #[error("source repository could not be reached")]
    UnreachableSource,
    #[error("source is not a supported Git repository")]
    UnsupportedRepository,
    #[error("source repository has no commits")]
    EmptyRepository,
    #[error("imported repository is too large for the MVP importer")]
    RepositoryTooLarge,
    #[error("git command failed: {0}")]
    Git(String),
    #[error("temporary import path failed: {0}")]
    Temp(String),
}

impl RepositoryImportProcessError {
    fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedSource => "unsupported_source",
            Self::BlockedSource => "private_network_source",
            Self::UnreachableSource => "unreachable_source",
            Self::UnsupportedRepository => "unsupported_repository",
            Self::EmptyRepository => "empty_repository",
            Self::RepositoryTooLarge => "repository_too_large",
            Self::Git(_) | Self::Temp(_) => "import_failed",
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::Git(_) | Self::Temp(_) => "The repository import failed.".to_owned(),
            other => other.to_string(),
        }
    }
}

pub async fn run_next_repository_import(
    pool: &PgPool,
    worker_id: &str,
) -> Result<Option<RepositoryImportStatus>, RepositoryImportWorkerError> {
    let import_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT repository_imports.id
        FROM repository_imports
        JOIN job_leases
          ON job_leases.id = repository_imports.job_lease_id
        WHERE repository_imports.status = 'queued'
          AND job_leases.queue = $1
          AND job_leases.completed_at IS NULL
          AND (job_leases.locked_until IS NULL OR job_leases.locked_until <= now())
        ORDER BY repository_imports.created_at ASC
        LIMIT 1
        "#,
    )
    .bind(IMPORT_QUEUE)
    .fetch_optional(pool)
    .await?;

    let Some(import_id) = import_id else {
        return Ok(None);
    };
    run_repository_import_once(pool, import_id, worker_id).await
}

pub async fn run_repository_import_once(
    pool: &PgPool,
    import_id: Uuid,
    worker_id: &str,
) -> Result<Option<RepositoryImportStatus>, RepositoryImportWorkerError> {
    let Some(work_item) = get_repository_import_work_item(pool, import_id).await? else {
        return Err(RepositoryImportWorkerError::NotFound);
    };
    if work_item.import.status != RepositoryImportStatus::Queued {
        return Ok(Some(work_item.import.status));
    }

    let Some(lease_key) = work_item.import.job_lease_id.map(|_| import_id.to_string()) else {
        return Err(RepositoryImportWorkerError::JobLease(
            JobLeaseError::NotFound,
        ));
    };
    let Some(lease) = acquire_job_lease(
        pool,
        IMPORT_QUEUE,
        &lease_key,
        worker_id,
        IMPORT_LEASE_SECONDS,
    )
    .await?
    else {
        return Ok(None);
    };

    mark_repository_import_importing(
        pool,
        import_id,
        "Fetching source repository and indexing default branch.",
    )
    .await?;

    match import_source_snapshot(&work_item.import.source.url).await {
        Ok(snapshot) => {
            replace_repository_snapshot(pool, work_item.repository.id, snapshot).await?;
            mark_repository_import_imported(
                pool,
                import_id,
                "Repository import completed. The default branch is ready.",
            )
            .await?;
            create_terminal_side_effects(pool, &work_item, RepositoryImportStatus::Imported, None)
                .await?;
            complete_job_lease(pool, lease.id, worker_id).await?;
            Ok(Some(RepositoryImportStatus::Imported))
        }
        Err(error) => {
            mark_repository_import_failed(pool, import_id, error.code(), &error.user_message())
                .await?;
            create_terminal_side_effects(
                pool,
                &work_item,
                RepositoryImportStatus::Failed,
                Some((error.code(), error.user_message())),
            )
            .await?;
            fail_job_lease(pool, lease.id, worker_id, error.code(), 300).await?;
            Ok(Some(RepositoryImportStatus::Failed))
        }
    }
}

async fn create_terminal_side_effects(
    pool: &PgPool,
    work_item: &crate::domain::repository_imports::RepositoryImportWorkItem,
    status: RepositoryImportStatus,
    failure: Option<(&str, String)>,
) -> Result<(), RepositoryImportWorkerError> {
    let repository_name = work_item.import.repository_href.trim_start_matches('/');
    let (title, reason, email_subject, email_body) = match status {
        RepositoryImportStatus::Imported => (
            format!("Import completed for {repository_name}"),
            "import_completed",
            format!("Repository import completed: {repository_name}"),
            format!(
                "The repository import for {repository_name} completed. Open it at {}.",
                work_item.import.repository_href
            ),
        ),
        RepositoryImportStatus::Failed => {
            let message = failure
                .as_ref()
                .map(|(_, message)| message.as_str())
                .unwrap_or("The repository import failed.");
            (
                format!("Import failed for {repository_name}"),
                "import_failed",
                format!("Repository import failed: {repository_name}"),
                format!("The repository import for {repository_name} failed: {message}"),
            )
        }
        RepositoryImportStatus::Queued | RepositoryImportStatus::Importing => return Ok(()),
    };

    create_notification(
        pool,
        CreateNotification {
            user_id: work_item.import.requested_by_user_id,
            repository_id: Some(work_item.repository.id),
            subject_type: "repository_import".to_owned(),
            subject_id: Some(work_item.import.id),
            title,
            reason: reason.to_owned(),
        },
    )
    .await
    .map_err(|error| match error {
        crate::domain::notifications::NotificationError::Sqlx(error) => {
            RepositoryImportWorkerError::Sqlx(error)
        }
        crate::domain::notifications::NotificationError::NotFound => {
            RepositoryImportWorkerError::NotFound
        }
        crate::domain::notifications::NotificationError::Validation(_) => {
            RepositoryImportWorkerError::NotFound
        }
    })?;

    let email_key = format!(
        "repository_import:{}:{}",
        work_item.import.id,
        status.as_str()
    );
    crate::jobs::enqueue_job(
        pool,
        EMAIL_QUEUE,
        &email_key,
        serde_json::json!({
            "kind": "repository_import",
            "importId": work_item.import.id,
            "repositoryId": work_item.repository.id,
            "userId": work_item.import.requested_by_user_id,
            "status": status,
            "subject": email_subject,
            "body": email_body,
            "repositoryHref": work_item.import.repository_href,
            "errorCode": failure.as_ref().map(|(code, _)| *code),
        }),
    )
    .await?;

    Ok(())
}

async fn import_source_snapshot(
    source_url: &str,
) -> Result<RepositorySnapshot, RepositoryImportProcessError> {
    validate_worker_source(source_url)?;
    let checkout_dir = temp_checkout_dir()?;
    let clone_result = run_git([
        OsStr::new("clone"),
        OsStr::new("--no-tags"),
        OsStr::new("--depth"),
        OsStr::new("1"),
        OsStr::new("--"),
        OsStr::new(source_url),
        checkout_dir.as_os_str(),
    ])
    .await;

    if let Err(error) = clone_result {
        let _ = std::fs::remove_dir_all(&checkout_dir);
        return Err(error);
    }

    let snapshot = snapshot_checkout(&checkout_dir).await;
    let _ = std::fs::remove_dir_all(&checkout_dir);
    snapshot
}

fn validate_worker_source(source_url: &str) -> Result<(), RepositoryImportProcessError> {
    if source_url.starts_with("file://") {
        let file_imports_enabled = std::env::var("OPENGITHUB_ALLOW_FILE_IMPORTS")
            .ok()
            .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"));
        return file_imports_enabled
            .then_some(())
            .ok_or(RepositoryImportProcessError::UnsupportedSource);
    }

    match validate_import_source_url(source_url) {
        Ok(_) => Ok(()),
        Err(crate::domain::repository_imports::RepositoryImportError::BlockedSourceHost) => {
            Err(RepositoryImportProcessError::BlockedSource)
        }
        Err(_) => Err(RepositoryImportProcessError::UnsupportedSource),
    }
}

async fn snapshot_checkout(
    path: &Path,
) -> Result<RepositorySnapshot, RepositoryImportProcessError> {
    let head_oid = git_output(path, ["rev-parse", "HEAD"]).await?;
    if head_oid.trim().is_empty() {
        return Err(RepositoryImportProcessError::EmptyRepository);
    }
    let tree_oid = git_output(path, ["rev-parse", "HEAD^{tree}"]).await?;
    let branch_name = git_output(path, ["branch", "--show-current"])
        .await
        .unwrap_or_else(|_| "main".to_owned())
        .trim()
        .to_owned();
    let branch_name = if branch_name.is_empty() {
        "main".to_owned()
    } else {
        branch_name
    };
    let message = git_output(path, ["log", "-1", "--format=%B"]).await?;
    let committed_at = git_output(path, ["log", "-1", "--format=%cI"]).await?;
    let parent_output = git_output(path, ["log", "-1", "--format=%P"]).await?;
    let files = snapshot_files(path).await?;

    Ok(RepositorySnapshot {
        commit: CreateCommit {
            oid: head_oid.trim().to_owned(),
            author_user_id: None,
            committer_user_id: None,
            message: message.trim().to_owned(),
            tree_oid: Some(tree_oid.trim().to_owned()),
            parent_oids: parent_output
                .split_whitespace()
                .map(str::to_owned)
                .collect(),
            committed_at: DateTime::parse_from_rfc3339(committed_at.trim())
                .map(|value| value.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        },
        branch_name,
        files,
    })
}

async fn snapshot_files(
    path: &Path,
) -> Result<Vec<RepositorySnapshotFile>, RepositoryImportProcessError> {
    let output = Command::new("git")
        .current_dir(path)
        .args(["ls-tree", "-r", "-z", "--full-tree", "HEAD"])
        .output()
        .await
        .map_err(|_| RepositoryImportProcessError::UnsupportedRepository)?;
    if !output.status.success() {
        return Err(RepositoryImportProcessError::UnsupportedRepository);
    }

    let mut files = Vec::new();
    for entry in output.stdout.split(|byte| *byte == 0) {
        if entry.is_empty() {
            continue;
        }
        if files.len() >= MAX_IMPORTED_FILES {
            return Err(RepositoryImportProcessError::RepositoryTooLarge);
        }
        let entry = String::from_utf8_lossy(entry);
        let Some((metadata, file_path)) = entry.split_once('\t') else {
            continue;
        };
        let mut metadata_parts = metadata.split_whitespace();
        let _mode = metadata_parts.next();
        let object_type = metadata_parts.next();
        let oid = metadata_parts.next().unwrap_or_default().to_owned();
        if object_type != Some("blob") || oid.is_empty() {
            continue;
        }

        let bytes = git_bytes(path, ["show", &format!("HEAD:{file_path}")]).await?;
        if bytes.len() > MAX_IMPORTED_FILE_BYTES {
            return Err(RepositoryImportProcessError::RepositoryTooLarge);
        }
        files.push(RepositorySnapshotFile {
            path: file_path.to_owned(),
            content: String::from_utf8_lossy(&bytes).into_owned(),
            oid,
            byte_size: bytes.len() as i64,
        });
    }
    files.sort_by(|left, right| left.path.to_lowercase().cmp(&right.path.to_lowercase()));
    Ok(files)
}

fn temp_checkout_dir() -> Result<PathBuf, RepositoryImportProcessError> {
    let dir = std::env::temp_dir().join(format!("opengithub-import-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir)
        .map_err(|error| RepositoryImportProcessError::Temp(error.to_string()))?;
    Ok(dir)
}

async fn git_output<const N: usize>(
    current_dir: &Path,
    args: [&str; N],
) -> Result<String, RepositoryImportProcessError> {
    let bytes = git_bytes(current_dir, args).await?;
    String::from_utf8(bytes).map_err(|_| RepositoryImportProcessError::UnsupportedRepository)
}

async fn git_bytes<const N: usize>(
    current_dir: &Path,
    args: [&str; N],
) -> Result<Vec<u8>, RepositoryImportProcessError> {
    let output = Command::new("git")
        .current_dir(current_dir)
        .args(args)
        .output()
        .await
        .map_err(|_| RepositoryImportProcessError::UnsupportedRepository)?;
    if output.status.success() {
        return Ok(output.stdout);
    }
    Err(classify_git_failure(&output.stderr))
}

async fn run_git<I, S>(args: I) -> Result<(), RepositoryImportProcessError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git")
        .args(args)
        .output()
        .await
        .map_err(|_| RepositoryImportProcessError::UnsupportedRepository)?;
    if output.status.success() {
        return Ok(());
    }
    Err(classify_git_failure(&output.stderr))
}

fn classify_git_failure(stderr: &[u8]) -> RepositoryImportProcessError {
    let message = String::from_utf8_lossy(stderr).to_string();
    let lower = message.to_ascii_lowercase();
    if lower.contains("repository not found")
        || lower.contains("could not resolve host")
        || lower.contains("failed to connect")
        || lower.contains("connection refused")
        || lower.contains("authentication failed")
    {
        RepositoryImportProcessError::UnreachableSource
    } else if lower.contains("does not appear to be a git repository")
        || lower.contains("not a git repository")
        || lower.contains("remote head refers to nonexistent ref")
    {
        RepositoryImportProcessError::UnsupportedRepository
    } else {
        RepositoryImportProcessError::Git(message)
    }
}
