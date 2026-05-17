use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tokio::{fs, process::Command};
use uuid::Uuid;

use crate::storage::{ObjectStorage, StorageError};

use super::{
    git_transport::{materialize_bare_repository, GitTransportError},
    repositories::{
        can_read_repository, get_repository_by_owner_name, Repository, RepositoryVisibility,
    },
};

const MAX_RAW_BYTES: u64 = 16 * 1024 * 1024;
const MAX_ARCHIVE_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRepositoryFile {
    pub content: Vec<u8>,
    pub content_type: &'static str,
    pub filename: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryArchive {
    pub id: Uuid,
    pub repository_id: Uuid,
    pub ref_name: String,
    pub target_oid: String,
    pub format: String,
    pub storage_key: String,
    pub byte_size: i64,
    pub status: String,
    pub created_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn stream_raw_file(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    ref_name: &str,
    path: &str,
    actor_user_id: Option<Uuid>,
) -> Result<RawRepositoryFile, GitTransportError> {
    let repository = readable_repository(pool, owner, repo, actor_user_id).await?;
    let store = materialize_bare_repository(pool, &repository).await?;
    let bare_path = PathBuf::from(store.storage_path);
    let commit_oid = resolve_ref(&bare_path, &repository, ref_name).await?;
    let safe_path = safe_repository_file_path(path)?;
    let object = format!(
        "{commit_oid}:{}",
        safe_path.to_string_lossy().replace('\\', "/")
    );
    let content = git_output(Some(&bare_path), [OsStr::new("show"), OsStr::new(&object)]).await?;
    if content.len() as u64 > MAX_RAW_BYTES {
        return Err(GitTransportError::RequestTooLarge);
    }
    let filename = safe_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("download")
        .to_owned();

    Ok(RawRepositoryFile {
        content_type: content_type_for_path(&safe_path, is_probably_binary(&content)),
        content,
        filename,
    })
}

pub async fn ensure_repository_archive(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    ref_name: &str,
    actor_user_id: Option<Uuid>,
) -> Result<(RepositoryArchive, Vec<u8>), GitTransportError> {
    let repository = readable_repository(pool, owner, repo, actor_user_id).await?;
    let store = materialize_bare_repository(pool, &repository).await?;
    let bare_path = PathBuf::from(store.storage_path);
    let commit_oid = resolve_ref(&bare_path, &repository, ref_name).await?;
    let storage = git_blob_storage()?;
    let storage_key = archive_storage_key(repository.id, ref_name, &commit_oid);

    if let Some(existing) = archive_row(pool, repository.id, ref_name, &commit_oid).await? {
        if let Ok(bytes) = storage.get(&existing.storage_key).await {
            if bytes.len() as u64 <= MAX_ARCHIVE_BYTES {
                return Ok((existing, bytes));
            }
        }
    }

    let tmp_path = std::env::temp_dir().join(format!(
        "opengithub-archive-{}-{}.zip",
        repository.id,
        Uuid::new_v4()
    ));
    let prefix = format!(
        "{}-{}/",
        repository.name,
        commit_oid.get(..7).unwrap_or(&commit_oid)
    );
    git_status(
        Some(&bare_path),
        [
            OsString::from("archive"),
            OsString::from("--format=zip"),
            OsString::from(format!("--prefix={prefix}")),
            OsString::from("-o"),
            tmp_path.as_os_str().to_os_string(),
            OsString::from(&commit_oid),
        ],
    )
    .await?;
    let metadata = fs::metadata(&tmp_path).await.map_err(storage_error)?;
    if metadata.len() > MAX_ARCHIVE_BYTES {
        let _ = fs::remove_file(&tmp_path).await;
        return Err(GitTransportError::RequestTooLarge);
    }
    let bytes = fs::read(&tmp_path).await.map_err(storage_error)?;
    let _ = fs::remove_file(&tmp_path).await;
    storage
        .put(&storage_key, bytes.clone())
        .await
        .map_err(blob_storage_error)?;
    let archive = upsert_archive(
        pool,
        &repository,
        ref_name,
        &commit_oid,
        &storage_key,
        bytes.len() as i64,
        actor_user_id,
    )
    .await?;
    Ok((archive, bytes))
}

async fn readable_repository(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    actor_user_id: Option<Uuid>,
) -> Result<Repository, GitTransportError> {
    let repository = get_repository_by_owner_name(pool, owner, repo)
        .await
        .map_err(repository_error)?
        .ok_or(GitTransportError::NotFound)?;
    if repository.visibility == RepositoryVisibility::Public {
        return Ok(repository);
    }
    let Some(actor_user_id) = actor_user_id else {
        return Err(GitTransportError::AuthenticationRequired);
    };
    if can_read_repository(pool, &repository, actor_user_id)
        .await
        .map_err(repository_error)?
    {
        Ok(repository)
    } else {
        Err(GitTransportError::AuthenticationRequired)
    }
}

async fn resolve_ref(
    bare_path: &Path,
    repository: &Repository,
    ref_name: &str,
) -> Result<String, GitTransportError> {
    let normalized_ref = ref_name.trim().trim_matches('/');
    if normalized_ref.is_empty()
        || normalized_ref.contains("..")
        || normalized_ref.contains('\\')
        || normalized_ref.starts_with('-')
    {
        return Err(GitTransportError::NotFound);
    }
    let candidates = [
        normalized_ref.to_owned(),
        format!("refs/heads/{normalized_ref}"),
        format!("refs/tags/{normalized_ref}"),
    ];
    for candidate in candidates {
        if let Ok(oid) = git_string(
            Some(bare_path),
            [
                OsString::from("rev-parse"),
                OsString::from(format!("{candidate}^{{commit}}")),
            ],
        )
        .await
        {
            return Ok(oid);
        }
    }
    if normalized_ref == repository.default_branch {
        return Err(GitTransportError::EmptyRepository);
    }
    Err(GitTransportError::NotFound)
}

fn safe_repository_file_path(path: &str) -> Result<PathBuf, GitTransportError> {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return Err(GitTransportError::NotFound);
    }
    let mut safe = PathBuf::new();
    for segment in trimmed.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." || segment.contains('\\') {
            return Err(GitTransportError::NotFound);
        }
        safe.push(segment);
    }
    Ok(safe)
}

async fn archive_row(
    pool: &PgPool,
    repository_id: Uuid,
    ref_name: &str,
    target_oid: &str,
) -> Result<Option<RepositoryArchive>, GitTransportError> {
    let row = sqlx::query(
        r#"
        SELECT id, repository_id, ref_name, target_oid, format, storage_key, byte_size,
               status, created_by_user_id, created_at, updated_at
        FROM repository_archives
        WHERE repository_id = $1 AND ref_name = $2 AND target_oid = $3 AND format = 'zip'
        "#,
    )
    .bind(repository_id)
    .bind(ref_name)
    .bind(target_oid)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(archive_from_row))
}

async fn upsert_archive(
    pool: &PgPool,
    repository: &Repository,
    ref_name: &str,
    target_oid: &str,
    storage_key: &str,
    byte_size: i64,
    actor_user_id: Option<Uuid>,
) -> Result<RepositoryArchive, GitTransportError> {
    let row = sqlx::query(
        r#"
        INSERT INTO repository_archives (
            repository_id, ref_name, target_oid, format, storage_key, byte_size,
            status, created_by_user_id
        )
        VALUES ($1, $2, $3, 'zip', $4, $5, 'ready', $6)
        ON CONFLICT (repository_id, ref_name, target_oid, format)
        DO UPDATE SET storage_key = EXCLUDED.storage_key,
                      byte_size = EXCLUDED.byte_size,
                      status = 'ready',
                      created_by_user_id = COALESCE(repository_archives.created_by_user_id, EXCLUDED.created_by_user_id)
        RETURNING id, repository_id, ref_name, target_oid, format, storage_key, byte_size,
                  status, created_by_user_id, created_at, updated_at
        "#,
    )
    .bind(repository.id)
    .bind(ref_name)
    .bind(target_oid)
    .bind(storage_key)
    .bind(byte_size)
    .bind(actor_user_id)
    .fetch_one(pool)
    .await?;
    Ok(archive_from_row(row))
}

fn archive_from_row(row: sqlx::postgres::PgRow) -> RepositoryArchive {
    RepositoryArchive {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        ref_name: row.get("ref_name"),
        target_oid: row.get("target_oid"),
        format: row.get("format"),
        storage_key: row.get("storage_key"),
        byte_size: row.get("byte_size"),
        status: row.get("status"),
        created_by_user_id: row.get("created_by_user_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn archive_storage_key(repository_id: Uuid, ref_name: &str, target_oid: &str) -> String {
    format!(
        "archives/{}/{}-{}.zip",
        repository_id,
        sanitize_storage_segment(ref_name),
        target_oid.get(..12).unwrap_or(target_oid)
    )
}

fn git_blob_storage() -> Result<ObjectStorage, GitTransportError> {
    ObjectStorage::from_env_with_local(git_storage_root()).map_err(blob_storage_error)
}

fn blob_storage_error(error: StorageError) -> GitTransportError {
    GitTransportError::Storage(error.to_string())
}

fn git_storage_root() -> PathBuf {
    std::env::var("OPENGITHUB_GIT_STORAGE_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("opengithub-git-storage"))
}

fn sanitize_storage_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn content_type_for_path(path: &Path, is_binary: bool) -> &'static str {
    if is_binary {
        return "application/octet-stream";
    }
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("md") | Some("markdown") => "text/markdown; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") | Some("mjs") | Some("ts") | Some("tsx") => "text/plain; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("txt") | Some("rs") | Some("toml") | Some("yaml") | Some("yml") => {
            "text/plain; charset=utf-8"
        }
        _ => "text/plain; charset=utf-8",
    }
}

fn is_probably_binary(content: &[u8]) -> bool {
    content.iter().take(8192).any(|byte| *byte == 0)
}

async fn git_output<I, S>(current_dir: Option<&Path>, args: I) -> Result<Vec<u8>, GitTransportError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = git_command(current_dir, args).await?;
    Ok(output.stdout)
}

async fn git_string<I, S>(current_dir: Option<&Path>, args: I) -> Result<String, GitTransportError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = git_output(current_dir, args).await?;
    String::from_utf8(output)
        .map(|value| value.trim().to_owned())
        .map_err(|_| GitTransportError::GitCommand)
}

async fn git_status<I, S>(current_dir: Option<&Path>, args: I) -> Result<(), GitTransportError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    git_command(current_dir, args).await.map(|_| ())
}

async fn git_command<I, S>(
    current_dir: Option<&Path>,
    args: I,
) -> Result<std::process::Output, GitTransportError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new("git");
    if let Some(current_dir) = current_dir {
        command.current_dir(current_dir);
    }
    command.args(args);
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    let output = command
        .output()
        .await
        .map_err(|_| GitTransportError::GitCommand)?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(GitTransportError::NotFound)
    }
}

fn storage_error(error: std::io::Error) -> GitTransportError {
    GitTransportError::Storage(error.to_string())
}

fn repository_error(error: super::repositories::RepositoryError) -> GitTransportError {
    match error {
        super::repositories::RepositoryError::Sqlx(error) => GitTransportError::Sqlx(error),
        _ => GitTransportError::GitCommand,
    }
}
