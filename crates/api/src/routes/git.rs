use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::Engine as _;
use serde::Deserialize;

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope},
    auth::session,
    domain::git_archives::{ensure_repository_archive, stream_raw_file},
    domain::git_transport::{
        advertise_receive_pack, advertise_upload_pack, run_receive_pack, run_upload_pack,
        GitServiceRequest, GitTransportError,
    },
    domain::repositories::get_repository_by_owner_name,
    domain::tokens::verify_personal_access_token,
    AppState,
};

type GitRouteError = (
    StatusCode,
    [(HeaderName, HeaderValue); 1],
    Json<ErrorEnvelope>,
);

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:owner/:repo_git/info/refs", get(info_refs))
        .route("/:owner/:repo_git/git-upload-pack", post(upload_pack))
        .route("/:owner/:repo_git/git-receive-pack", post(receive_pack))
        .route("/:owner/:repo/raw/:ref_name/*path", get(raw_file))
        .route("/:owner/:repo/download/:ref_name/*path", get(download_file))
        .route(
            "/:owner/:repo/archive/refs/heads/*branch_zip",
            get(branch_archive),
        )
}

#[derive(Debug, Deserialize)]
struct InfoRefsQuery {
    service: Option<String>,
}

async fn info_refs(
    State(state): State<AppState>,
    Path((owner, repo_git)): Path<(String, String)>,
    Query(query): Query<InfoRefsQuery>,
    headers: HeaderMap,
) -> Result<Response, GitRouteError> {
    let repo = repository_name_from_git_path(&repo_git).ok_or_else(repository_not_found)?;
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| with_git_auth_challenge(database_unavailable()))?;
    let service = query.service.unwrap_or_default();
    let requires_write = service == "git-receive-pack";
    let actor_user_id =
        git_actor_user_id(pool, &state, &headers, &owner, repo, requires_write).await;
    let request = GitServiceRequest {
        owner,
        repo: repo.to_owned(),
        service,
        actor_user_id,
    };
    let response = if request.service == "git-receive-pack" {
        advertise_receive_pack(pool, request).await
    } else {
        advertise_upload_pack(pool, request).await
    }
    .map_err(map_git_error)?;
    Ok(git_response(response.content_type, response.body))
}

async fn upload_pack(
    State(state): State<AppState>,
    Path((owner, repo_git)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, GitRouteError> {
    let repo = repository_name_from_git_path(&repo_git).ok_or_else(repository_not_found)?;
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| with_git_auth_challenge(database_unavailable()))?;
    let actor_user_id = git_actor_user_id(pool, &state, &headers, &owner, repo, false).await;
    let response = run_upload_pack(
        pool,
        GitServiceRequest {
            owner,
            repo: repo.to_owned(),
            service: "git-upload-pack".to_owned(),
            actor_user_id,
        },
        body.to_vec(),
    )
    .await
    .map_err(map_git_error)?;
    Ok(git_response(response.content_type, response.body))
}

async fn receive_pack(
    State(state): State<AppState>,
    Path((owner, repo_git)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, GitRouteError> {
    let repo = repository_name_from_git_path(&repo_git).ok_or_else(repository_not_found)?;
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| with_git_auth_challenge(database_unavailable()))?;
    let actor_user_id = git_actor_user_id(pool, &state, &headers, &owner, repo, true).await;
    let response = run_receive_pack(
        pool,
        GitServiceRequest {
            owner,
            repo: repo.to_owned(),
            service: "git-receive-pack".to_owned(),
            actor_user_id,
        },
        body.to_vec(),
    )
    .await
    .map_err(map_git_error)?;
    Ok(git_response(response.content_type, response.body))
}

async fn raw_file(
    State(state): State<AppState>,
    Path((owner, repo, ref_name, path)): Path<(String, String, String, String)>,
    headers: HeaderMap,
) -> Result<Response, GitRouteError> {
    repository_bytes(state, headers, owner, repo, ref_name, path, false).await
}

async fn download_file(
    State(state): State<AppState>,
    Path((owner, repo, ref_name, path)): Path<(String, String, String, String)>,
    headers: HeaderMap,
) -> Result<Response, GitRouteError> {
    repository_bytes(state, headers, owner, repo, ref_name, path, true).await
}

async fn repository_bytes(
    state: AppState,
    headers: HeaderMap,
    owner: String,
    repo: String,
    ref_name: String,
    path: String,
    download: bool,
) -> Result<Response, GitRouteError> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| with_git_auth_challenge(database_unavailable()))?;
    let actor_user_id = git_actor_user_id(pool, &state, &headers, &owner, &repo, false).await;
    let raw = stream_raw_file(pool, &owner, &repo, &ref_name, &path, actor_user_id)
        .await
        .map_err(map_git_error)?;
    let mut response = raw.content.into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(if download {
            "application/octet-stream"
        } else {
            raw.content_type
        }),
    );
    if download {
        headers.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!(
                "attachment; filename=\"{}\"",
                safe_download_filename(&raw.filename)
            ))
            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
        );
    }
    Ok(response)
}

async fn branch_archive(
    State(state): State<AppState>,
    Path((owner, repo, branch_zip)): Path<(String, String, String)>,
    headers: HeaderMap,
) -> Result<Response, GitRouteError> {
    let branch = branch_zip
        .strip_suffix(".zip")
        .filter(|value| !value.is_empty())
        .ok_or_else(repository_not_found)?;
    let pool = state
        .db
        .as_ref()
        .ok_or_else(|| with_git_auth_challenge(database_unavailable()))?;
    let actor_user_id = git_actor_user_id(pool, &state, &headers, &owner, &repo, false).await;
    let (archive, bytes) = ensure_repository_archive(pool, &owner, &repo, branch, actor_user_id)
        .await
        .map_err(map_git_error)?;
    let mut response = bytes.into_response();
    let filename = format!("{}-{}.zip", repo, branch.replace('/', "-"));
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/zip"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename=\"{}\"",
            safe_download_filename(&filename)
        ))
        .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&archive.byte_size.to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("0")),
    );
    Ok(response)
}

async fn git_actor_user_id(
    pool: &sqlx::PgPool,
    state: &AppState,
    headers: &HeaderMap,
    owner: &str,
    repo: &str,
    requires_write: bool,
) -> Option<uuid::Uuid> {
    if let Some(token) = git_token_from_headers(headers) {
        if let Ok(verified) = verify_personal_access_token(pool, &token).await {
            let repository = get_repository_by_owner_name(pool, owner, repo)
                .await
                .ok()
                .flatten();
            if ((requires_write && verified.allows_repo_write())
                || (!requires_write && verified.allows_repo_read()))
                && repository.is_some_and(|repository| verified.allows_repository(repository.id))
            {
                return Some(verified.user_id);
            }
        }
    }

    session::current_user_from_headers(pool, &state.config, headers)
        .await
        .ok()
        .flatten()
        .map(|user| user.id)
}

fn git_token_from_headers(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?.trim();
    if let Some(token) = value.strip_prefix("Bearer ") {
        return non_empty_token(token);
    }
    let encoded = value.strip_prefix("Basic ")?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded.as_bytes())
        .ok()?;
    let decoded = String::from_utf8(decoded).ok()?;
    let (_username, password) = decoded.split_once(':')?;
    non_empty_token(password)
}

fn non_empty_token(value: &str) -> Option<String> {
    let token = value.trim();
    (!token.is_empty()).then(|| token.to_owned())
}

fn safe_download_filename(path_name: &str) -> String {
    let sanitized: String = path_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.trim_matches('_').is_empty() {
        "download".to_owned()
    } else {
        sanitized
    }
}

fn git_response(content_type: &'static str, body: Vec<u8>) -> Response {
    let mut response = body.into_response();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    response
}

fn repository_name_from_git_path(repo_git: &str) -> Option<&str> {
    repo_git
        .strip_suffix(".git")
        .filter(|repo| !repo.is_empty())
}

fn repository_not_found() -> GitRouteError {
    with_git_auth_challenge(error_response(
        StatusCode::NOT_FOUND,
        "not_found",
        "repository was not found".to_owned(),
    ))
}

fn map_git_error(error: GitTransportError) -> GitRouteError {
    let status = error.status_code();
    with_git_auth_challenge(error_response(
        status,
        error.code(),
        match error {
            GitTransportError::Sqlx(_)
            | GitTransportError::Storage(_)
            | GitTransportError::GitCommand => "git transport failed".to_owned(),
            GitTransportError::DatabaseUnavailable => "database is unavailable".to_owned(),
            GitTransportError::NotFound => "repository was not found".to_owned(),
            GitTransportError::AuthenticationRequired => {
                "authentication is required for this repository".to_owned()
            }
            GitTransportError::UnsupportedService => "unsupported git service".to_owned(),
            GitTransportError::RequestTooLarge => "git request is too large".to_owned(),
            GitTransportError::EmptyRepository => "repository has no cloneable refs".to_owned(),
            GitTransportError::BranchPolicyBlocked(message) => message,
        },
    ))
}

fn with_git_auth_challenge((status, body): (StatusCode, Json<ErrorEnvelope>)) -> GitRouteError {
    (
        status,
        [(
            header::WWW_AUTHENTICATE,
            HeaderValue::from_static(r#"Basic realm="opengithub Git""#),
        )],
        body,
    )
}
