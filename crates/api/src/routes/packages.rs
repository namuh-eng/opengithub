use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    api_types::{
        database_unavailable, error_response, normalize_pagination, ErrorEnvelope, RestJson,
    },
    auth::extractor::AuthenticatedUser,
    domain::{
        actions::{
            create_package, create_package_version, get_package_for_actor, list_package_versions,
            list_packages, repository_for_actor_by_name, CreatePackage, CreatePackageVersion,
            PackageType,
        },
        packages_registry::{
            exchange_registry_token, read_registry_manifest, registry_auth_from_headers,
            registry_challenge, RegistryError,
        },
        permissions::RepositoryRole,
    },
    routes::actions::map_automation_error,
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v2/", get(registry_ping))
        .route("/v2/token", get(registry_token))
        .route(
            "/v2/:namespace/:image/manifests/:reference",
            get(registry_manifest).head(registry_manifest_head),
        )
        .route(
            "/api/repos/:owner/:repo/packages",
            get(list_packages_route).post(create_package_route),
        )
        .route(
            "/api/repos/:owner/:repo/packages/:package_id",
            get(read_package_route),
        )
        .route(
            "/api/repos/:owner/:repo/packages/:package_id/versions",
            get(list_package_versions_route).post(create_package_version_route),
        )
}

async fn registry_token(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(registry_database_unavailable)?;
    let token = exchange_registry_token(pool, &headers)
        .await
        .map_err(|error| registry_error_response(error, None))?;
    Ok(Json(json!(token)))
}

async fn registry_ping(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(registry_database_unavailable)?;
    let auth = registry_auth_from_headers(pool, &headers).await;
    match auth {
        Ok(crate::domain::packages_registry::RegistryAuth::Anonymous) => {
            Err(registry_error_response(RegistryError::Unauthorized, None))
        }
        Ok(crate::domain::packages_registry::RegistryAuth::Token { .. }) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                "docker-distribution-api-version",
                HeaderValue::from_static("registry/2.0"),
            );
            Ok((StatusCode::OK, headers, Json(json!({}))).into_response())
        }
        Err(error) => Err(registry_error_response(error, None)),
    }
}

async fn registry_manifest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((namespace, image, reference)): Path<(String, String, String)>,
) -> Result<Response, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    registry_manifest_response(state, headers, namespace, image, reference, true).await
}

async fn registry_manifest_head(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((namespace, image, reference)): Path<(String, String, String)>,
) -> Result<Response, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    registry_manifest_response(state, headers, namespace, image, reference, false).await
}

async fn registry_manifest_response(
    state: AppState,
    headers: HeaderMap,
    namespace: String,
    image: String,
    reference: String,
    include_body: bool,
) -> Result<Response, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(registry_database_unavailable)?;
    let auth = registry_auth_from_headers(pool, &headers)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok());
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok());
    let manifest = read_registry_manifest(
        pool, &namespace, &image, &reference, accept, &auth, user_agent,
    )
    .await
    .map_err(|error| registry_error_response(error, Some(&namespace)))?;

    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&manifest.media_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/json")),
    );
    response_headers.insert(
        "docker-content-digest",
        HeaderValue::from_str(manifest.digest.as_deref().unwrap_or(&reference))
            .unwrap_or_else(|_| HeaderValue::from_static("")),
    );
    response_headers.insert(
        "docker-distribution-api-version",
        HeaderValue::from_static("registry/2.0"),
    );
    let serialized_manifest = serde_json::to_vec(&manifest.manifest).unwrap_or_else(|_| Vec::new());
    response_headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&serialized_manifest.len().to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("0")),
    );
    if include_body {
        Ok((StatusCode::OK, response_headers, Json(manifest.manifest)).into_response())
    } else {
        Ok((StatusCode::OK, response_headers).into_response())
    }
}

fn registry_database_unavailable() -> (StatusCode, HeaderMap, Json<serde_json::Value>) {
    let headers = HeaderMap::new();
    (
        StatusCode::SERVICE_UNAVAILABLE,
        headers,
        Json(json!({
            "errors": [{
                "code": "UNAVAILABLE",
                "message": "registry database is unavailable"
            }]
        })),
    )
}

fn registry_error_response(
    error: RegistryError,
    namespace: Option<&str>,
) -> (StatusCode, HeaderMap, Json<serde_json::Value>) {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    let (status, code, message) = match error {
        RegistryError::Unauthorized => {
            headers.insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_str(&registry_challenge(namespace.map(|namespace| {
                    format!("repository:{namespace}:pull")
                }).as_deref()))
                .unwrap_or_else(|_| HeaderValue::from_static("Bearer realm=\"http://localhost:3016/v2/token\",service=\"opengithub-registry\"")),
            );
            (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "authentication required",
            )
        }
        RegistryError::InvalidToken => {
            headers.insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static("Bearer realm=\"http://localhost:3016/v2/token\",service=\"opengithub-registry\""),
            );
            (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "invalid registry token",
            )
        }
        RegistryError::InsufficientScope => (
            StatusCode::FORBIDDEN,
            "DENIED",
            "token is missing packages:read scope",
        ),
        RegistryError::NotFound => (
            StatusCode::NOT_FOUND,
            "MANIFEST_UNKNOWN",
            "manifest unknown",
        ),
        RegistryError::InvalidReference(message) => {
            return (
                StatusCode::BAD_REQUEST,
                headers,
                Json(json!({ "errors": [{ "code": "NAME_INVALID", "message": message }] })),
            );
        }
        RegistryError::NotAcceptable => (
            StatusCode::NOT_ACCEPTABLE,
            "MANIFEST_INVALID",
            "requested manifest media type is not acceptable",
        ),
        RegistryError::Sqlx(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "UNKNOWN",
            "registry request failed",
        ),
    };
    (
        status,
        headers,
        Json(json!({ "errors": [{ "code": code, "message": message }] })),
    )
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
struct CreatePackageRequest {
    name: String,
    package_type: PackageType,
    visibility: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreatePackageVersionRequest {
    version: String,
    manifest: Option<Value>,
    blob_key: Option<String>,
    size_bytes: Option<i64>,
}

async fn list_packages_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_automation_error)?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let envelope = list_packages(
        pool,
        repository_id,
        actor.0.id,
        pagination.page,
        pagination.page_size,
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(envelope)))
}

async fn create_package_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<CreatePackageRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_automation_error)?;
    if request.name.trim().is_empty() {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "package name is required",
        ));
    }
    let visibility = request.visibility.unwrap_or_else(|| "private".to_owned());
    if !matches!(visibility.as_str(), "public" | "private" | "internal") {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "package visibility must be public, private, or internal",
        ));
    }
    let package = create_package(
        pool,
        CreatePackage {
            repository_id,
            actor_user_id: actor.0.id,
            name: request.name,
            package_type: request.package_type,
            visibility,
        },
    )
    .await
    .map_err(map_automation_error)?;

    Ok((StatusCode::CREATED, Json(json!(package))))
}

async fn read_package_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, package_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_automation_error)?;
    let package = get_package_for_actor(pool, repository_id, package_id, actor.0.id)
        .await
        .map_err(map_automation_error)?;

    Ok(Json(json!(package)))
}

async fn list_package_versions_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, package_id)): Path<(String, String, Uuid)>,
    Query(query): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_automation_error)?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let envelope = list_package_versions(
        pool,
        repository_id,
        package_id,
        actor.0.id,
        pagination.page,
        pagination.page_size,
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(envelope)))
}

async fn create_package_version_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, package_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<CreatePackageVersionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_automation_error)?;
    get_package_for_actor(pool, repository_id, package_id, actor.0.id)
        .await
        .map_err(map_automation_error)?;
    if request.version.trim().is_empty() {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "package version is required",
        ));
    }
    let version = create_package_version(
        pool,
        CreatePackageVersion {
            package_id,
            actor_user_id: actor.0.id,
            version: request.version,
            manifest: request.manifest.unwrap_or_else(|| json!({})),
            blob_key: request.blob_key,
            size_bytes: request.size_bytes,
        },
    )
    .await
    .map_err(map_automation_error)?;

    Ok((StatusCode::CREATED, Json(json!(version))))
}
