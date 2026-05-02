use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    api_types::{
        database_unavailable, error_response, error_response_with_details, normalize_pagination,
        ErrorEnvelope, RestJson,
    },
    auth::extractor::AuthenticatedUser,
    domain::repositories::{
        create_repository_with_bootstrap, fork_repository_by_owner_name,
        insert_repository_create_feed_event, list_repositories_for_user,
        repository_blame_for_actor_by_owner_name, repository_blob_for_actor_by_owner_name,
        repository_commit_history_for_actor_by_owner_name, repository_creation_options,
        repository_file_finder_for_actor_by_owner_name, repository_name_availability,
        repository_overview_for_viewer_by_owner_name,
        repository_path_overview_for_actor_by_owner_name, repository_refs_for_actor_by_owner_name,
        repository_settings_for_admin_by_owner_name, set_repository_star_by_owner_name,
        set_repository_watch_by_owner_name, update_repository_settings_by_owner_name,
        CreateRepository, RepositoryBootstrapRequest, RepositoryCommitHistoryQuery,
        RepositoryError, RepositoryFileFinderQuery, RepositoryOwner, RepositoryPathQuery,
        RepositoryRefsQuery, RepositoryVisibility, UpdateRepositorySettings,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/creation-options", get(creation_options))
        .route("/name-availability", get(name_availability))
        .route("/:owner/:repo/contents/*path", get(contents))
        .route("/:owner/:repo/blobs/*path", get(blob))
        .route("/:owner/:repo/blame/*path", get(blame))
        .route("/:owner/:repo/commits", get(commits))
        .route("/:owner/:repo/refs", get(refs))
        .route("/:owner/:repo/file-finder", get(file_finder))
        .route(
            "/:owner/:repo/settings",
            get(settings).patch(update_settings),
        )
        .route("/:owner/:repo/star", put(star).delete(unstar))
        .route("/:owner/:repo/watch", put(watch).delete(unwatch))
        .route("/:owner/:repo/forks", post(fork))
        .route("/:owner/:repo", get(read))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQuery {
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateRepositoryRequest {
    owner_type: OwnerType,
    owner_id: Uuid,
    name: String,
    description: Option<String>,
    visibility: Option<RepositoryVisibility>,
    default_branch: Option<String>,
    initialize_readme: Option<bool>,
    template_slug: Option<String>,
    gitignore_template_slug: Option<String>,
    license_template_slug: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NameAvailabilityQuery {
    owner_type: OwnerType,
    owner_id: Uuid,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentsQuery {
    #[serde(rename = "ref")]
    ref_name: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
    raw: Option<String>,
    download: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommitsQuery {
    #[serde(rename = "ref")]
    ref_name: Option<String>,
    path: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RefsQuery {
    q: Option<String>,
    current_path: Option<String>,
    active_ref: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileFinderQuery {
    #[serde(rename = "ref")]
    ref_name: Option<String>,
    q: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum OwnerType {
    User,
    Organization,
}

async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let envelope =
        list_repositories_for_user(pool, actor.0.id, pagination.page, pagination.page_size)
            .await
            .map_err(map_repository_error)?;

    Ok(Json(json!(envelope)))
}

async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    RestJson(request): RestJson<CreateRepositoryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let owner = match request.owner_type {
        OwnerType::User => RepositoryOwner::User {
            id: request.owner_id,
        },
        OwnerType::Organization => RepositoryOwner::Organization {
            id: request.owner_id,
        },
    };
    let repository = create_repository_with_bootstrap(
        pool,
        CreateRepository {
            owner,
            name: request.name,
            description: request.description,
            visibility: request.visibility.unwrap_or_default(),
            default_branch: request.default_branch,
            created_by_user_id: actor.0.id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: request.initialize_readme.unwrap_or(false),
            template_slug: request.template_slug,
            gitignore_template_slug: request.gitignore_template_slug,
            license_template_slug: request.license_template_slug,
        },
    )
    .await
    .map_err(map_repository_error)?;
    insert_repository_create_feed_event(pool, &repository, actor.0.id)
        .await
        .map_err(map_repository_error)?;
    let mut body = json!(repository);
    body["href"] = json!(format!("/{}/{}", repository.owner_login, repository.name));

    Ok((StatusCode::CREATED, Json(body)))
}

async fn creation_options(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let options = repository_creation_options(pool, actor.0.id)
        .await
        .map_err(map_repository_error)?;

    Ok(Json(json!(options)))
}

async fn name_availability(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NameAvailabilityQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let owner = match query.owner_type {
        OwnerType::User => RepositoryOwner::User { id: query.owner_id },
        OwnerType::Organization => RepositoryOwner::Organization { id: query.owner_id },
    };
    let availability = repository_name_availability(pool, actor.0.id, owner, &query.name)
        .await
        .map_err(map_repository_error)?;

    Ok(Json(json!(availability)))
}

async fn read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repository = repository_overview_for_viewer_by_owner_name(
        pool,
        actor.map(|user| user.id),
        &owner,
        &repo,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(repository)))
}

async fn contents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, path)): Path<(String, String, String)>,
    Query(query): Query<ContentsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let overview = repository_path_overview_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryPathQuery {
            ref_name: query.ref_name.as_deref(),
            path: &path,
            page: pagination.page,
            page_size: pagination.page_size,
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(overview)))
}

async fn blob(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, path)): Path<(String, String, String)>,
    Query(query): Query<ContentsQuery>,
) -> Result<Response, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_blob_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        query.ref_name.as_deref(),
        &path,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    let wants_raw = truthy_query(query.raw.as_deref());
    let wants_download = truthy_query(query.download.as_deref());
    if wants_raw || wants_download {
        let mut response = view.file.content.clone().into_response();
        let headers = response.headers_mut();
        let content_type = if wants_download || view.is_binary {
            "application/octet-stream"
        } else {
            view.mime_type.as_str()
        };
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(content_type)
                .unwrap_or_else(|_| HeaderValue::from_static("text/plain; charset=utf-8")),
        );
        if wants_download {
            let filename = safe_download_filename(&view.path_name);
            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
                    .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
            );
        }
        return Ok(response);
    }

    Ok(Json(json!(view)).into_response())
}

async fn blame(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, path)): Path<(String, String, String)>,
    Query(query): Query<ContentsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_blame_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        query.ref_name.as_deref(),
        &path,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn commits(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<CommitsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let envelope = repository_commit_history_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryCommitHistoryQuery {
            ref_name: query.ref_name.as_deref(),
            path: query.path.as_deref(),
            page: pagination.page,
            page_size: pagination.page_size,
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(envelope)))
}

async fn refs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<RefsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let envelope = repository_refs_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryRefsQuery {
            query: query.q.as_deref(),
            current_path: query.current_path.as_deref(),
            active_ref: query.active_ref.as_deref(),
            page: query.page.unwrap_or(1).max(1),
            page_size: query.page_size.unwrap_or(100).clamp(1, 100),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(envelope)))
}

async fn file_finder(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<FileFinderQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let envelope = repository_file_finder_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryFileFinderQuery {
            ref_name: query.ref_name.as_deref(),
            query: query.q.as_deref(),
            page: query.page.unwrap_or(1).max(1),
            page_size: query.page_size.unwrap_or(20).clamp(1, 100),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(envelope)))
}

async fn settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = repository_settings_for_admin_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(settings)))
}

async fn update_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<UpdateRepositorySettings>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        update_repository_settings_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn star(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_star(state, headers, owner, repo, true).await
}

async fn unstar(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_star(state, headers, owner, repo, false).await
}

async fn set_star(
    state: AppState,
    headers: HeaderMap,
    owner: String,
    repo: String,
    starred: bool,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let social = set_repository_star_by_owner_name(pool, actor.0.id, &owner, &repo, starred)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(social)))
}

async fn watch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_watch(state, headers, owner, repo, true).await
}

async fn unwatch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_watch(state, headers, owner, repo, false).await
}

async fn set_watch(
    state: AppState,
    headers: HeaderMap,
    owner: String,
    repo: String,
    watching: bool,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let social = set_repository_watch_by_owner_name(pool, actor.0.id, &owner, &repo, watching)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(social)))
}

async fn fork(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let fork = fork_repository_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok((StatusCode::CREATED, Json(json!(fork))))
}

fn map_repository_error(error: RepositoryError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        RepositoryError::OwnerPermissionDenied | RepositoryError::PermissionDenied => {
            error_response(StatusCode::FORBIDDEN, "forbidden", error.to_string())
        }
        RepositoryError::OwnerNotFound
        | RepositoryError::NotFound
        | RepositoryError::PathNotFound
        | RepositoryError::RefNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        RepositoryError::RefNotFoundWithRecovery {
            ref_name,
            recovery_href,
            default_branch_href,
        } => error_response_with_details(
            StatusCode::NOT_FOUND,
            "ref_not_found",
            format!("repository ref `{ref_name}` was not found"),
            json!({
                "refName": ref_name,
                "recoveryHref": recovery_href,
                "defaultBranchHref": default_branch_href,
            }),
        ),
        RepositoryError::PathNotFoundWithRecovery {
            path,
            recovery_href,
            default_branch_href,
        } => error_response_with_details(
            StatusCode::NOT_FOUND,
            "path_not_found",
            format!("repository path `{path}` was not found"),
            json!({
                "path": path,
                "recoveryHref": recovery_href,
                "defaultBranchHref": default_branch_href,
            }),
        ),
        RepositoryError::InvalidVisibility(_)
        | RepositoryError::InvalidName(_)
        | RepositoryError::InvalidDescription(_)
        | RepositoryError::NoMergeMethodEnabled
        | RepositoryError::UnknownTemplate(_)
        | RepositoryError::UnknownGitignoreTemplate(_)
        | RepositoryError::UnknownLicenseTemplate(_) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        RepositoryError::ForkAlreadyExists => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::GitStorageFailed => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "git_storage_failed",
            "repository git storage failed".to_owned(),
        ),
        RepositoryError::Sqlx(sqlx::Error::Database(database_error))
            if database_error.is_unique_violation() =>
        {
            error_response(
                StatusCode::CONFLICT,
                "conflict",
                database_error.message().to_owned(),
            )
        }
        RepositoryError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "repository operation failed".to_owned(),
        ),
    }
}

fn safe_download_filename(path_name: &str) -> String {
    let sanitized = path_name
        .chars()
        .map(|character| match character {
            '"' | '\\' | '/' | '\r' | '\n' | '\t' => '_',
            character if character.is_control() => '_',
            character => character,
        })
        .collect::<String>();
    let trimmed = sanitized.trim_matches('.').trim();
    if trimmed.is_empty() {
        "download".to_owned()
    } else {
        trimmed.chars().take(120).collect()
    }
}

fn truthy_query(value: Option<&str>) -> bool {
    matches!(
        value.map(str::to_ascii_lowercase).as_deref(),
        Some("1" | "true" | "yes" | "on" | "")
    )
}
