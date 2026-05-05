use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    api_types::{database_unavailable, error_response, error_response_with_details, ErrorEnvelope},
    auth::extractor::AuthenticatedUser,
    domain::{
        repositories::{CreateRepository, RepositoryError, RepositoryOwner, RepositoryVisibility},
        repository_imports::{
            create_repository_import, get_repository_import_for_actor, CreateRepositoryImport,
            RepositoryImportError,
        },
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/repos/imports", post(create))
        .route("/api/repos/imports/:id", get(read))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateRepositoryImportRequest {
    source_url: String,
    source_username: Option<String>,
    source_token: Option<String>,
    source_password: Option<String>,
    owner_type: OwnerType,
    owner_id: Uuid,
    name: String,
    description: Option<String>,
    visibility: Option<RepositoryVisibility>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum OwnerType {
    User,
    Organization,
}

async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateRepositoryImportRequest>,
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

    let import = create_repository_import(
        pool,
        CreateRepositoryImport {
            repository: CreateRepository {
                owner,
                name: request.name,
                description: request.description,
                visibility: request.visibility.unwrap_or_default(),
                default_branch: Some("main".to_owned()),
                created_by_user_id: actor.0.id,
            },
            source_url: request.source_url,
            source_username: request.source_username,
            source_token: request.source_token,
            source_password: request.source_password,
        },
    )
    .await
    .map_err(map_import_error)?;

    Ok((StatusCode::CREATED, Json(json!(import))))
}

async fn read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(import_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let import = get_repository_import_for_actor(pool, import_id, actor.0.id)
        .await
        .map_err(map_import_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "import was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(import)))
}

fn map_import_error(error: RepositoryImportError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        RepositoryImportError::InvalidSourceUrl(_) | RepositoryImportError::BlockedSourceHost => {
            error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_failed",
                error.to_string(),
            )
        }
        RepositoryImportError::PermissionDenied => {
            error_response(StatusCode::FORBIDDEN, "forbidden", error.to_string())
        }
        RepositoryImportError::NotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        RepositoryImportError::Repository(repository_error) => {
            map_repository_error(repository_error)
        }
        RepositoryImportError::Sqlx(sqlx::Error::Database(database_error))
            if database_error.is_unique_violation() =>
        {
            error_response(
                StatusCode::CONFLICT,
                "conflict",
                database_error.message().to_owned(),
            )
        }
        RepositoryImportError::Sqlx(_)
        | RepositoryImportError::JobLease(_)
        | RepositoryImportError::InvalidStatus(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "repository import operation failed".to_owned(),
        ),
    }
}

fn map_repository_error(error: RepositoryError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        RepositoryError::OwnerPermissionDenied
        | RepositoryError::PermissionDenied
        | RepositoryError::TrafficAccessDenied => {
            error_response(StatusCode::FORBIDDEN, "forbidden", error.to_string())
        }
        RepositoryError::OrganizationRepositoryCreationPolicy {
            visibility,
            reason,
            settings_href,
        } => error_response_with_details(
            StatusCode::FORBIDDEN,
            "policy_locked",
            reason.clone(),
            json!({
                "visibility": visibility,
                "reason": reason,
                "settingsHref": settings_href,
            }),
        ),
        RepositoryError::OrganizationPolicyLocked {
            field,
            reason,
            settings_href,
        } => error_response_with_details(
            StatusCode::FORBIDDEN,
            "policy_locked",
            reason.clone(),
            json!({
                "field": field,
                "reason": reason,
                "settingsHref": settings_href,
            }),
        ),
        RepositoryError::OwnerNotFound
        | RepositoryError::NotFound
        | RepositoryError::AccessTargetNotFound
        | RepositoryError::PathNotFound
        | RepositoryError::RefNotFound
        | RepositoryError::PathNotFoundWithRecovery { .. }
        | RepositoryError::RefNotFoundWithRecovery { .. } => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        RepositoryError::InvalidVisibility(_)
        | RepositoryError::InvalidName(_)
        | RepositoryError::InvalidDescription(_)
        | RepositoryError::InvalidMergeMethod(_)
        | RepositoryError::InvalidWatchLevel(_)
        | RepositoryError::InvalidWatchEvent(_)
        | RepositoryError::InvalidAccessRole(_)
        | RepositoryError::InvalidBranchPolicy(_)
        | RepositoryError::InvalidSecurityPolicy(_)
        | RepositoryError::InvalidBranchDirectoryQuery(_)
        | RepositoryError::InvalidPulseQuery(_)
        | RepositoryError::InvalidContributorsQuery(_)
        | RepositoryError::InvalidForksQuery(_)
        | RepositoryError::InvalidDependencyGraphQuery(_)
        | RepositoryError::InvalidDiffContext(_)
        | RepositoryError::MergeMethodRequired
        | RepositoryError::DefaultMergeMethodDisabled
        | RepositoryError::ArchivedRepositoryReadOnly
        | RepositoryError::UnknownTemplate(_)
        | RepositoryError::UnknownGitignoreTemplate(_)
        | RepositoryError::UnknownLicenseTemplate(_)
        | RepositoryError::DependencyGraphUnavailable(_)
        | RepositoryError::TeamAccessUnsupported => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        RepositoryError::ForkAlreadyExists => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::AccessGrantConflict => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::BranchPolicyConflict => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::SecurityPolicyConflict => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::LastAdminAccess => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::BranchPolicyNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        RepositoryError::DefaultBranchNotFound(_) => {
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
