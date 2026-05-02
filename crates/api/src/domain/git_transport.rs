use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tokio::{fs, io::AsyncWriteExt, process::Command};
use uuid::Uuid;

use super::actions::{trigger_workflows_for_push, AutomationError, TriggerWorkflowsForPush};
use super::repositories::{
    can_read_repository, can_write_repository, get_repository_by_owner_name, Repository,
    RepositorySnapshot, RepositorySnapshotFile, RepositoryVisibility,
};

const MAX_UPLOAD_PACK_REQUEST_BYTES: usize = 32 * 1024 * 1024;
const MAX_RECEIVE_PACK_REQUEST_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRepositoryStore {
    pub repository_id: Uuid,
    pub storage_kind: String,
    pub storage_path: String,
    pub last_materialized_commit_id: Option<Uuid>,
    pub last_materialized_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitServiceRequest {
    pub owner: String,
    pub repo: String,
    pub service: String,
    pub actor_user_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitServiceResponse {
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum GitTransportError {
    #[error("database is unavailable")]
    DatabaseUnavailable,
    #[error("repository was not found")]
    NotFound,
    #[error("authentication is required for this repository")]
    AuthenticationRequired,
    #[error("unsupported git service")]
    UnsupportedService,
    #[error("git request is too large")]
    RequestTooLarge,
    #[error("repository has no cloneable refs")]
    EmptyRepository,
    #[error("git storage failed: {0}")]
    Storage(String),
    #[error("git command failed")]
    GitCommand,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl GitTransportError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::DatabaseUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::AuthenticationRequired => StatusCode::UNAUTHORIZED,
            Self::UnsupportedService => StatusCode::BAD_REQUEST,
            Self::RequestTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::EmptyRepository => StatusCode::NOT_FOUND,
            Self::Storage(_) | Self::GitCommand | Self::Sqlx(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::DatabaseUnavailable => "database_unavailable",
            Self::NotFound => "not_found",
            Self::AuthenticationRequired => "authentication_required",
            Self::UnsupportedService => "unsupported_git_service",
            Self::RequestTooLarge => "request_too_large",
            Self::EmptyRepository => "empty_repository",
            Self::Storage(_) | Self::GitCommand | Self::Sqlx(_) => "git_transport_failed",
        }
    }
}

pub async fn advertise_upload_pack(
    pool: &PgPool,
    request: GitServiceRequest,
) -> Result<GitServiceResponse, GitTransportError> {
    ensure_upload_pack(&request.service)?;
    let repository =
        readable_repository(pool, &request.owner, &request.repo, request.actor_user_id).await?;
    let store = materialize_bare_repository(pool, &repository).await?;
    let bare_path = PathBuf::from(store.storage_path);
    let advertisement = git_output(
        None,
        [
            OsStr::new("upload-pack"),
            OsStr::new("--stateless-rpc"),
            OsStr::new("--advertise-refs"),
            bare_path.as_os_str(),
        ],
        None,
    )
    .await?;

    let mut body = Vec::new();
    body.extend_from_slice(b"001e# service=git-upload-pack\n0000");
    body.extend_from_slice(&advertisement);
    Ok(GitServiceResponse {
        content_type: "application/x-git-upload-pack-advertisement",
        body,
    })
}

pub async fn advertise_receive_pack(
    pool: &PgPool,
    request: GitServiceRequest,
) -> Result<GitServiceResponse, GitTransportError> {
    ensure_receive_pack(&request.service)?;
    let actor_user_id = request
        .actor_user_id
        .ok_or(GitTransportError::AuthenticationRequired)?;
    let repository =
        writable_repository(pool, &request.owner, &request.repo, actor_user_id).await?;
    let store = materialize_bare_repository(pool, &repository).await?;
    let bare_path = PathBuf::from(store.storage_path);
    let advertisement = git_output(
        None,
        [
            OsStr::new("receive-pack"),
            OsStr::new("--stateless-rpc"),
            OsStr::new("--advertise-refs"),
            bare_path.as_os_str(),
        ],
        None,
    )
    .await?;

    let mut body = Vec::new();
    body.extend_from_slice(b"001f# service=git-receive-pack\n0000");
    body.extend_from_slice(&advertisement);
    Ok(GitServiceResponse {
        content_type: "application/x-git-receive-pack-advertisement",
        body,
    })
}

pub async fn run_upload_pack(
    pool: &PgPool,
    request: GitServiceRequest,
    body: Vec<u8>,
) -> Result<GitServiceResponse, GitTransportError> {
    ensure_upload_pack(&request.service)?;
    if body.len() > MAX_UPLOAD_PACK_REQUEST_BYTES {
        return Err(GitTransportError::RequestTooLarge);
    }
    let repository =
        readable_repository(pool, &request.owner, &request.repo, request.actor_user_id).await?;
    let store = materialize_bare_repository(pool, &repository).await?;
    let bare_path = PathBuf::from(store.storage_path);
    let result = git_output(
        None,
        [
            OsStr::new("upload-pack"),
            OsStr::new("--stateless-rpc"),
            bare_path.as_os_str(),
        ],
        Some(body),
    )
    .await?;

    Ok(GitServiceResponse {
        content_type: "application/x-git-upload-pack-result",
        body: result,
    })
}

pub async fn run_receive_pack(
    pool: &PgPool,
    request: GitServiceRequest,
    body: Vec<u8>,
) -> Result<GitServiceResponse, GitTransportError> {
    ensure_receive_pack(&request.service)?;
    if body.len() > MAX_RECEIVE_PACK_REQUEST_BYTES {
        return Err(GitTransportError::RequestTooLarge);
    }
    let actor_user_id = request
        .actor_user_id
        .ok_or(GitTransportError::AuthenticationRequired)?;
    let repository =
        writable_repository(pool, &request.owner, &request.repo, actor_user_id).await?;
    let store = materialize_bare_repository(pool, &repository).await?;
    let bare_path = PathBuf::from(store.storage_path);
    let result = git_output(
        None,
        [
            OsStr::new("receive-pack"),
            OsStr::new("--stateless-rpc"),
            bare_path.as_os_str(),
        ],
        Some(body),
    )
    .await?;
    sync_pushed_refs_to_database(pool, &repository, actor_user_id, &bare_path).await?;

    Ok(GitServiceResponse {
        content_type: "application/x-git-receive-pack-result",
        body: result,
    })
}

pub async fn materialize_bare_repository(
    pool: &PgPool,
    repository: &Repository,
) -> Result<GitRepositoryStore, GitTransportError> {
    let target =
        repository_default_branch_target(pool, repository.id, &repository.default_branch).await?;
    let storage_path = repository_storage_path(repository.id);
    let existing = get_storage(pool, repository.id).await?;
    if existing.as_ref().is_some_and(|store| {
        store.last_materialized_commit_id == target.commit_id
            && Path::new(&store.storage_path).exists()
    }) {
        return Ok(existing.expect("checked existing storage"));
    }

    if target.commit_id.is_none() {
        fs::create_dir_all(storage_path.parent().unwrap_or_else(|| Path::new(".")))
            .await
            .map_err(storage_error)?;
        if !storage_path.exists() {
            run_git([
                OsStr::new("init"),
                OsStr::new("--bare"),
                storage_path.as_os_str(),
            ])
            .await?;
            git_output(
                None,
                [
                    OsStr::new("--git-dir"),
                    storage_path.as_os_str(),
                    OsStr::new("symbolic-ref"),
                    OsStr::new("HEAD"),
                    OsStr::new(&format!("refs/heads/{}", repository.default_branch)),
                ],
                None,
            )
            .await?;
        }
        return upsert_storage(pool, repository.id, &storage_path, None).await;
    }

    let files = repository_files_for_commit(
        pool,
        repository.id,
        target.commit_id.expect("target commit"),
    )
    .await?;
    if files.is_empty() {
        return Err(GitTransportError::EmptyRepository);
    }

    let worktree = temp_worktree_path(repository.id);
    let _ = fs::remove_dir_all(&worktree).await;
    fs::create_dir_all(&worktree).await.map_err(storage_error)?;
    let materialized =
        materialize_worktree(repository, &target, &files, &worktree, &storage_path).await;
    let _ = fs::remove_dir_all(&worktree).await;
    materialized?;

    upsert_storage(pool, repository.id, &storage_path, target.commit_id).await
}

pub async fn materialize_bare_repository_by_id(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<GitRepositoryStore, GitTransportError> {
    let repository = crate::domain::repositories::get_repository(pool, repository_id)
        .await
        .map_err(repository_error)?
        .ok_or(GitTransportError::NotFound)?;
    materialize_bare_repository(pool, &repository).await
}

pub async fn sync_pushed_refs_to_database(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    bare_path: &Path,
) -> Result<(), GitTransportError> {
    let refs = pushed_refs(bare_path).await?;
    let mut synced_ref_names = Vec::new();

    for pushed_ref in refs {
        if pushed_ref.kind != "branch" {
            continue;
        }
        let snapshot =
            snapshot_from_commit(bare_path, &pushed_ref.commit_oid, &pushed_ref.short_name).await?;
        let commit = upsert_pushed_snapshot(pool, repository.id, snapshot).await?;
        synced_ref_names.push(pushed_ref.name.clone());
        insert_push_events(
            pool,
            repository,
            actor_user_id,
            &pushed_ref.name,
            &commit.oid,
        )
        .await?;
        trigger_workflows_for_push(
            pool,
            TriggerWorkflowsForPush {
                repository_id: repository.id,
                actor_user_id,
                ref_name: pushed_ref.name.clone(),
                head_sha: commit.oid.clone(),
            },
        )
        .await
        .map_err(actions_error)?;
    }

    prune_deleted_branch_refs(pool, repository.id, &synced_ref_names).await?;
    let default_ref = format!("refs/heads/{}", repository.default_branch);
    let default_commit_id = sqlx::query_scalar::<_, Option<Uuid>>(
        r#"
        SELECT target_commit_id
        FROM repository_git_refs
        WHERE repository_id = $1 AND name = $2
        "#,
    )
    .bind(repository.id)
    .bind(default_ref)
    .fetch_optional(pool)
    .await?
    .flatten();
    upsert_storage(pool, repository.id, bare_path, default_commit_id).await?;
    Ok(())
}

fn actions_error(error: AutomationError) -> GitTransportError {
    match error {
        AutomationError::Sqlx(error) => GitTransportError::Sqlx(error),
        other => GitTransportError::Storage(format!("actions push trigger failed: {other}")),
    }
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

async fn writable_repository(
    pool: &PgPool,
    owner: &str,
    repo: &str,
    actor_user_id: Uuid,
) -> Result<Repository, GitTransportError> {
    let repository = get_repository_by_owner_name(pool, owner, repo)
        .await
        .map_err(repository_error)?
        .ok_or(GitTransportError::NotFound)?;
    if can_write_repository(pool, &repository, actor_user_id)
        .await
        .map_err(repository_error)?
    {
        Ok(repository)
    } else {
        Err(GitTransportError::AuthenticationRequired)
    }
}

async fn materialize_worktree(
    repository: &Repository,
    target: &BranchTarget,
    files: &[SnapshotFile],
    worktree: &Path,
    storage_path: &Path,
) -> Result<(), GitTransportError> {
    git_output(Some(worktree), [OsStr::new("init")], None).await?;
    git_output(
        Some(worktree),
        [
            OsStr::new("config"),
            OsStr::new("user.name"),
            OsStr::new("opengithub"),
        ],
        None,
    )
    .await?;
    git_output(
        Some(worktree),
        [
            OsStr::new("config"),
            OsStr::new("user.email"),
            OsStr::new("noreply@opengithub.namuh.co"),
        ],
        None,
    )
    .await?;
    git_output(
        Some(worktree),
        [
            OsStr::new("checkout"),
            OsStr::new("-B"),
            OsStr::new(&repository.default_branch),
        ],
        None,
    )
    .await?;

    for file in files {
        write_snapshot_file(worktree, file).await?;
    }

    git_output(
        Some(worktree),
        [OsStr::new("add"), OsStr::new("--all")],
        None,
    )
    .await?;
    git_output(
        Some(worktree),
        [
            OsStr::new("commit"),
            OsStr::new("--allow-empty"),
            OsStr::new("-m"),
            OsStr::new(&target.message),
        ],
        None,
    )
    .await?;

    let _ = fs::remove_dir_all(storage_path).await;
    fs::create_dir_all(storage_path.parent().unwrap_or_else(|| Path::new(".")))
        .await
        .map_err(storage_error)?;
    run_git([
        OsStr::new("clone"),
        OsStr::new("--bare"),
        worktree.as_os_str(),
        storage_path.as_os_str(),
    ])
    .await?;
    Ok(())
}

#[derive(Debug)]
struct PushedRef {
    name: String,
    short_name: String,
    kind: String,
    commit_oid: String,
}

async fn pushed_refs(bare_path: &Path) -> Result<Vec<PushedRef>, GitTransportError> {
    let output = git_output(
        Some(bare_path),
        [
            OsStr::new("for-each-ref"),
            OsStr::new("--format=%(refname)%00%(objectname)%00%(objecttype)"),
            OsStr::new("refs/heads"),
            OsStr::new("refs/tags"),
        ],
        None,
    )
    .await?;
    let rendered = String::from_utf8(output).map_err(|_| GitTransportError::GitCommand)?;
    let mut refs = Vec::new();
    for line in rendered.lines() {
        let mut parts = line.split('\0');
        let Some(name) = parts.next() else { continue };
        let Some(object_oid) = parts.next() else {
            continue;
        };
        let Some(object_type) = parts.next() else {
            continue;
        };
        let (kind, short_name) = if let Some(short) = name.strip_prefix("refs/heads/") {
            ("branch", short)
        } else if let Some(short) = name.strip_prefix("refs/tags/") {
            ("tag", short)
        } else {
            continue;
        };
        let commit_oid = if object_type == "commit" {
            object_oid.to_owned()
        } else {
            git_string(
                Some(bare_path),
                vec![
                    OsString::from("rev-parse"),
                    OsString::from(format!("{name}^{{commit}}")),
                ],
            )
            .await?
        };
        refs.push(PushedRef {
            name: name.to_owned(),
            short_name: short_name.to_owned(),
            kind: kind.to_owned(),
            commit_oid,
        });
    }
    Ok(refs)
}

async fn snapshot_from_commit(
    bare_path: &Path,
    commit_oid: &str,
    branch_name: &str,
) -> Result<RepositorySnapshot, GitTransportError> {
    let tree_oid = git_string(
        Some(bare_path),
        vec![
            OsString::from("rev-parse"),
            OsString::from(format!("{commit_oid}^{{tree}}")),
        ],
    )
    .await?;
    let message = git_string(
        Some(bare_path),
        vec![
            OsString::from("log"),
            OsString::from("-1"),
            OsString::from("--format=%B"),
            OsString::from(commit_oid),
        ],
    )
    .await?
    .trim()
    .to_owned();
    let committed_at = git_string(
        Some(bare_path),
        vec![
            OsString::from("log"),
            OsString::from("-1"),
            OsString::from("--format=%cI"),
            OsString::from(commit_oid),
        ],
    )
    .await?;
    let committed_at = DateTime::parse_from_rfc3339(committed_at.trim())
        .map_err(|_| GitTransportError::GitCommand)?
        .with_timezone(&Utc);
    let parent_oids = git_string(
        Some(bare_path),
        vec![
            OsString::from("log"),
            OsString::from("-1"),
            OsString::from("--format=%P"),
            OsString::from(commit_oid),
        ],
    )
    .await?
    .split_whitespace()
    .map(ToOwned::to_owned)
    .collect::<Vec<_>>();

    let files = repository_files_from_commit(bare_path, commit_oid).await?;
    Ok(RepositorySnapshot {
        commit: super::repositories::CreateCommit {
            oid: commit_oid.to_owned(),
            author_user_id: None,
            committer_user_id: None,
            message: if message.is_empty() {
                "Pushed commit".to_owned()
            } else {
                message
            },
            tree_oid: Some(tree_oid),
            parent_oids,
            committed_at,
        },
        branch_name: branch_name.to_owned(),
        files,
    })
}

async fn repository_files_from_commit(
    bare_path: &Path,
    commit_oid: &str,
) -> Result<Vec<RepositorySnapshotFile>, GitTransportError> {
    let output = git_output(
        Some(bare_path),
        [
            OsStr::new("ls-tree"),
            OsStr::new("-r"),
            OsStr::new("-z"),
            OsStr::new("--long"),
            OsStr::new(commit_oid),
        ],
        None,
    )
    .await?;
    let rendered = String::from_utf8_lossy(&output);
    let mut files = Vec::new();
    for entry in rendered.split('\0').filter(|entry| !entry.is_empty()) {
        let Some((metadata, path)) = entry.split_once('\t') else {
            continue;
        };
        let mut metadata_parts = metadata.split_whitespace();
        let _mode = metadata_parts.next();
        let object_type = metadata_parts.next();
        let Some(oid) = metadata_parts.next() else {
            continue;
        };
        let size = metadata_parts
            .next()
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or_default();
        if object_type != Some("blob") {
            continue;
        }
        let content = git_output(
            Some(bare_path),
            [
                OsStr::new("show"),
                OsStr::new(&format!("{commit_oid}:{path}")),
            ],
            None,
        )
        .await?;
        files.push(RepositorySnapshotFile {
            path: safe_repository_file_path(path)?
                .to_string_lossy()
                .replace('\\', "/"),
            content: String::from_utf8_lossy(&content).into_owned(),
            oid: oid.to_owned(),
            byte_size: size,
        });
    }
    Ok(files)
}

async fn upsert_pushed_snapshot(
    pool: &PgPool,
    repository_id: Uuid,
    snapshot: RepositorySnapshot,
) -> Result<super::repositories::Commit, GitTransportError> {
    let mut transaction = pool.begin().await?;
    let row = sqlx::query(
        r#"
        INSERT INTO commits (
            repository_id,
            oid,
            author_user_id,
            committer_user_id,
            message,
            tree_oid,
            parent_oids,
            committed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (repository_id, oid)
        DO UPDATE SET message = EXCLUDED.message,
                      tree_oid = EXCLUDED.tree_oid,
                      parent_oids = EXCLUDED.parent_oids,
                      committed_at = EXCLUDED.committed_at
        RETURNING id, repository_id, oid, author_user_id, committer_user_id, message,
                  tree_oid, parent_oids, committed_at, created_at
        "#,
    )
    .bind(repository_id)
    .bind(&snapshot.commit.oid)
    .bind(snapshot.commit.author_user_id)
    .bind(snapshot.commit.committer_user_id)
    .bind(&snapshot.commit.message)
    .bind(&snapshot.commit.tree_oid)
    .bind(&snapshot.commit.parent_oids)
    .bind(snapshot.commit.committed_at)
    .fetch_one(&mut *transaction)
    .await?;
    let commit = commit_from_row(row);

    if let Some(tree_oid) = snapshot.commit.tree_oid.as_deref() {
        sqlx::query(
            r#"
            INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
            VALUES ($1, $2, 'tree', $3)
            ON CONFLICT (repository_id, oid) DO NOTHING
            "#,
        )
        .bind(repository_id)
        .bind(tree_oid)
        .bind(snapshot.files.len() as i64)
        .execute(&mut *transaction)
        .await?;
    }
    sqlx::query(
        r#"
        INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
        VALUES ($1, $2, 'commit', 0)
        ON CONFLICT (repository_id, oid) DO NOTHING
        "#,
    )
    .bind(repository_id)
    .bind(&commit.oid)
    .execute(&mut *transaction)
    .await?;
    sqlx::query("DELETE FROM repository_files WHERE repository_id = $1 AND commit_id = $2")
        .bind(repository_id)
        .bind(commit.id)
        .execute(&mut *transaction)
        .await?;

    for file in snapshot.files {
        sqlx::query(
            r#"
            INSERT INTO git_objects (repository_id, oid, object_type, byte_size)
            VALUES ($1, $2, 'blob', $3)
            ON CONFLICT (repository_id, oid) DO NOTHING
            "#,
        )
        .bind(repository_id)
        .bind(&file.oid)
        .bind(file.byte_size)
        .execute(&mut *transaction)
        .await?;
        sqlx::query(
            r#"
            INSERT INTO repository_files (repository_id, commit_id, path, content, oid, byte_size)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (repository_id, commit_id, lower(path))
            DO UPDATE SET content = EXCLUDED.content,
                          oid = EXCLUDED.oid,
                          byte_size = EXCLUDED.byte_size
            "#,
        )
        .bind(repository_id)
        .bind(commit.id)
        .bind(&file.path)
        .bind(&file.content)
        .bind(&file.oid)
        .bind(file.byte_size)
        .execute(&mut *transaction)
        .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, $2, 'branch', $3)
        ON CONFLICT (repository_id, name)
        DO UPDATE SET kind = EXCLUDED.kind, target_commit_id = EXCLUDED.target_commit_id
        "#,
    )
    .bind(repository_id)
    .bind(format!("refs/heads/{}", snapshot.branch_name))
    .bind(commit.id)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(commit)
}

async fn prune_deleted_branch_refs(
    pool: &PgPool,
    repository_id: Uuid,
    synced_ref_names: &[String],
) -> Result<(), GitTransportError> {
    sqlx::query(
        r#"
        DELETE FROM repository_git_refs
        WHERE repository_id = $1
          AND name LIKE 'refs/heads/%'
          AND NOT (name = ANY($2::text[]))
        "#,
    )
    .bind(repository_id)
    .bind(synced_ref_names)
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_push_events(
    pool: &PgPool,
    repository: &Repository,
    actor_user_id: Uuid,
    ref_name: &str,
    commit_oid: &str,
) -> Result<(), GitTransportError> {
    let short_ref = ref_name.strip_prefix("refs/heads/").unwrap_or(ref_name);
    sqlx::query(
        r#"
        INSERT INTO feed_events (
            actor_user_id,
            repository_id,
            event_type,
            title,
            excerpt,
            target_href,
            subject_type,
            subject_id,
            metadata
        )
        VALUES ($1, $2, 'push', $3, $4, $5, 'repository', $2, $6)
        "#,
    )
    .bind(actor_user_id)
    .bind(repository.id)
    .bind(format!(
        "Pushed to {}/{}",
        repository.owner_login, repository.name
    ))
    .bind(format!(
        "{short_ref} received commit {}",
        short_oid(commit_oid)
    ))
    .bind(format!(
        "/{}/{}/commits/{}",
        repository.owner_login, repository.name, short_ref
    ))
    .bind(serde_json::json!({
        "ref": ref_name,
        "commitOid": commit_oid,
        "source": "git_receive_pack",
    }))
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO audit_events (actor_user_id, event_type, target_type, target_id, metadata)
        VALUES ($1, 'repository.push', 'repository', $2, $3)
        "#,
    )
    .bind(actor_user_id)
    .bind(repository.id.to_string())
    .bind(serde_json::json!({
        "repository": format!("{}/{}", repository.owner_login, repository.name),
        "ref": ref_name,
        "commitOid": commit_oid,
    }))
    .execute(pool)
    .await?;

    Ok(())
}

fn short_oid(oid: &str) -> &str {
    oid.get(..7).unwrap_or(oid)
}

async fn write_snapshot_file(
    worktree: &Path,
    file: &SnapshotFile,
) -> Result<(), GitTransportError> {
    let relative_path = safe_repository_file_path(&file.path)?;
    let path = worktree.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await.map_err(storage_error)?;
    }
    fs::write(path, file.content.as_bytes())
        .await
        .map_err(storage_error)
}

fn safe_repository_file_path(path: &str) -> Result<PathBuf, GitTransportError> {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return Err(GitTransportError::Storage(
            "empty repository path".to_owned(),
        ));
    }
    let mut safe = PathBuf::new();
    for segment in trimmed.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." || segment.contains('\\') {
            return Err(GitTransportError::Storage(
                "unsafe repository path".to_owned(),
            ));
        }
        safe.push(segment);
    }
    Ok(safe)
}

fn commit_from_row(row: sqlx::postgres::PgRow) -> super::repositories::Commit {
    super::repositories::Commit {
        id: row.get("id"),
        repository_id: row.get("repository_id"),
        oid: row.get("oid"),
        author_user_id: row.get("author_user_id"),
        committer_user_id: row.get("committer_user_id"),
        message: row.get("message"),
        tree_oid: row.get("tree_oid"),
        parent_oids: row.get("parent_oids"),
        committed_at: row.get("committed_at"),
        created_at: row.get("created_at"),
    }
}

async fn get_storage(
    pool: &PgPool,
    repository_id: Uuid,
) -> Result<Option<GitRepositoryStore>, GitTransportError> {
    let row = sqlx::query(
        r#"
        SELECT repository_id, storage_kind, storage_path, last_materialized_commit_id, last_materialized_at
        FROM repository_git_storage
        WHERE repository_id = $1
        "#,
    )
    .bind(repository_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(storage_from_row))
}

async fn upsert_storage(
    pool: &PgPool,
    repository_id: Uuid,
    storage_path: &Path,
    commit_id: Option<Uuid>,
) -> Result<GitRepositoryStore, GitTransportError> {
    let row = sqlx::query(
        r#"
        INSERT INTO repository_git_storage (
            repository_id,
            storage_kind,
            storage_path,
            last_materialized_commit_id,
            last_materialized_at,
            materialized_by
        )
        VALUES ($1, 'local_bare', $2, $3, now(), 'api')
        ON CONFLICT (repository_id)
        DO UPDATE SET storage_kind = EXCLUDED.storage_kind,
                      storage_path = EXCLUDED.storage_path,
                      last_materialized_commit_id = EXCLUDED.last_materialized_commit_id,
                      last_materialized_at = EXCLUDED.last_materialized_at,
                      materialized_by = EXCLUDED.materialized_by
        RETURNING repository_id, storage_kind, storage_path, last_materialized_commit_id, last_materialized_at
        "#,
    )
    .bind(repository_id)
    .bind(storage_path.to_string_lossy().as_ref())
    .bind(commit_id)
    .fetch_one(pool)
    .await?;

    Ok(storage_from_row(row))
}

#[derive(Debug)]
struct BranchTarget {
    commit_id: Option<Uuid>,
    message: String,
}

async fn repository_default_branch_target(
    pool: &PgPool,
    repository_id: Uuid,
    default_branch: &str,
) -> Result<BranchTarget, GitTransportError> {
    let row = sqlx::query(
        r#"
        SELECT commits.id AS commit_id, commits.message
        FROM repository_git_refs
        LEFT JOIN commits ON commits.id = repository_git_refs.target_commit_id
        WHERE repository_git_refs.repository_id = $1
          AND repository_git_refs.name = $2
        "#,
    )
    .bind(repository_id)
    .bind(format!("refs/heads/{default_branch}"))
    .fetch_optional(pool)
    .await?;

    Ok(match row {
        Some(row) => BranchTarget {
            commit_id: row.get("commit_id"),
            message: row
                .get::<Option<String>, _>("message")
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| "Materialize repository snapshot".to_owned()),
        },
        None => BranchTarget {
            commit_id: None,
            message: "Materialize empty repository".to_owned(),
        },
    })
}

#[derive(Debug)]
struct SnapshotFile {
    path: String,
    content: String,
}

async fn repository_files_for_commit(
    pool: &PgPool,
    repository_id: Uuid,
    commit_id: Uuid,
) -> Result<Vec<SnapshotFile>, GitTransportError> {
    let rows = sqlx::query(
        r#"
        SELECT path, content
        FROM repository_files
        WHERE repository_id = $1 AND commit_id = $2
        ORDER BY lower(path) ASC
        "#,
    )
    .bind(repository_id)
    .bind(commit_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| SnapshotFile {
            path: row.get("path"),
            content: row.get("content"),
        })
        .collect())
}

async fn git_output<I, S>(
    current_dir: Option<&Path>,
    args: I,
    stdin: Option<Vec<u8>>,
) -> Result<Vec<u8>, GitTransportError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new("git");
    if let Some(current_dir) = current_dir {
        command.current_dir(current_dir);
    }
    command.args(args);
    if stdin.is_some() {
        command.stdin(std::process::Stdio::piped());
    }
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    let mut child = command.spawn().map_err(|_| GitTransportError::GitCommand)?;
    if let Some(stdin) = stdin {
        let mut child_stdin = child.stdin.take().ok_or(GitTransportError::GitCommand)?;
        child_stdin
            .write_all(&stdin)
            .await
            .map_err(|_| GitTransportError::GitCommand)?;
        drop(child_stdin);
    }
    let output = child
        .wait_with_output()
        .await
        .map_err(|_| GitTransportError::GitCommand)?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(GitTransportError::GitCommand)
    }
}

async fn git_string<I, S>(current_dir: Option<&Path>, args: I) -> Result<String, GitTransportError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = git_output(current_dir, args, None).await?;
    String::from_utf8(output)
        .map(|value| value.trim().to_owned())
        .map_err(|_| GitTransportError::GitCommand)
}

async fn run_git<I, S>(args: I) -> Result<(), GitTransportError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    git_output(None, args, None).await.map(|_| ())
}

fn ensure_upload_pack(service: &str) -> Result<(), GitTransportError> {
    if service == "git-upload-pack" {
        Ok(())
    } else {
        Err(GitTransportError::UnsupportedService)
    }
}

fn ensure_receive_pack(service: &str) -> Result<(), GitTransportError> {
    if service == "git-receive-pack" {
        Ok(())
    } else {
        Err(GitTransportError::UnsupportedService)
    }
}

fn repository_storage_path(repository_id: Uuid) -> PathBuf {
    git_storage_root()
        .join("repositories")
        .join(format!("{repository_id}.git"))
}

fn git_storage_root() -> PathBuf {
    std::env::var("OPENGITHUB_GIT_STORAGE_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("opengithub-git-storage"))
}

fn temp_worktree_path(repository_id: Uuid) -> PathBuf {
    std::env::temp_dir().join(format!(
        "opengithub-materialize-{repository_id}-{}",
        Uuid::new_v4()
    ))
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

fn storage_from_row(row: sqlx::postgres::PgRow) -> GitRepositoryStore {
    GitRepositoryStore {
        repository_id: row.get("repository_id"),
        storage_kind: row.get("storage_kind"),
        storage_path: row.get("storage_path"),
        last_materialized_commit_id: row.get("last_materialized_commit_id"),
        last_materialized_at: row.get("last_materialized_at"),
    }
}
