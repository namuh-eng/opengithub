use axum::{
    body::{to_bytes, Body},
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, patch, post},
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
            append_blob_upload, cancel_blob_upload, complete_blob_upload, delete_registry_manifest,
            exchange_registry_token, list_registry_tags, put_registry_manifest, read_registry_blob,
            read_registry_manifest, registry_auth_from_headers, registry_challenge,
            start_blob_upload, RegistryError, RegistryManifestPutRequest,
            RegistryManifestReadRequest,
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
            get(registry_manifest)
                .head(registry_manifest_head)
                .put(registry_manifest_put)
                .delete(registry_manifest_delete),
        )
        .route(
            "/v2/:namespace/:image/blobs/uploads/",
            post(registry_blob_upload_start),
        )
        .route(
            "/v2/:namespace/:image/blobs/uploads/:upload_id",
            patch(registry_blob_upload_patch)
                .put(registry_blob_upload_complete)
                .delete(registry_blob_upload_cancel),
        )
        .route(
            "/v2/:namespace/:image/blobs/:digest",
            get(registry_blob).head(registry_blob_head),
        )
        .route("/v2/:namespace/:image/tags/list", get(registry_tags_list))
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
        Ok(crate::domain::packages_registry::RegistryAuth::Token { .. })
        | Ok(crate::domain::packages_registry::RegistryAuth::Workflow { .. }) => {
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

async fn registry_blob_upload_start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((namespace, image)): Path<(String, String)>,
) -> Result<Response, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(registry_database_unavailable)?;
    let auth = registry_auth_from_headers(pool, &headers)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    let upload = start_blob_upload(pool, &namespace, &image, &auth)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    let mut response_headers = registry_success_headers();
    response_headers.insert(
        header::LOCATION,
        HeaderValue::from_str(&upload.location).unwrap_or_else(|_| HeaderValue::from_static("/")),
    );
    response_headers.insert(
        "docker-upload-uuid",
        HeaderValue::from_str(&upload.upload_id.to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("")),
    );
    response_headers.insert(
        header::RANGE,
        HeaderValue::from_str(&upload.range).unwrap_or_else(|_| HeaderValue::from_static("0-0")),
    );
    Ok((StatusCode::ACCEPTED, response_headers, Json(json!({}))).into_response())
}

async fn registry_blob_upload_patch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((namespace, image, upload_id)): Path<(String, String, Uuid)>,
    body: Body,
) -> Result<Response, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(registry_database_unavailable)?;
    let auth = registry_auth_from_headers(pool, &headers)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    let bytes = to_bytes(body, usize::MAX).await.map_err(|_| {
        registry_error_response(
            RegistryError::InvalidManifest("request body could not be read".to_owned()),
            Some(&namespace),
        )
    })?;
    let upload = append_blob_upload(pool, &namespace, &image, upload_id, &bytes, &auth)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    Ok(upload_progress_response(StatusCode::ACCEPTED, upload))
}

async fn registry_blob_upload_complete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<UploadCompleteQuery>,
    Path((namespace, image, upload_id)): Path<(String, String, Uuid)>,
    body: Body,
) -> Result<Response, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(registry_database_unavailable)?;
    let auth = registry_auth_from_headers(pool, &headers)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    let bytes = to_bytes(body, usize::MAX).await.map_err(|_| {
        registry_error_response(
            RegistryError::InvalidManifest("request body could not be read".to_owned()),
            Some(&namespace),
        )
    })?;
    let upload = complete_blob_upload(
        pool,
        &namespace,
        &image,
        upload_id,
        &query.digest,
        &bytes,
        &auth,
    )
    .await
    .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    Ok(upload_progress_response(StatusCode::CREATED, upload))
}

async fn registry_blob_upload_cancel(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((namespace, image, upload_id)): Path<(String, String, Uuid)>,
) -> Result<Response, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(registry_database_unavailable)?;
    let auth = registry_auth_from_headers(pool, &headers)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    cancel_blob_upload(pool, &namespace, &image, upload_id, &auth)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    Ok((StatusCode::NO_CONTENT, registry_success_headers()).into_response())
}

async fn registry_manifest_put(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((namespace, image, reference)): Path<(String, String, String)>,
    body: Body,
) -> Result<Response, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(registry_database_unavailable)?;
    let auth = registry_auth_from_headers(pool, &headers)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok());
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok());
    let bytes = to_bytes(body, usize::MAX).await.map_err(|_| {
        registry_error_response(
            RegistryError::InvalidManifest("request body could not be read".to_owned()),
            Some(&namespace),
        )
    })?;
    let manifest: Value = serde_json::from_slice(&bytes).map_err(|_| {
        registry_error_response(
            RegistryError::InvalidManifest("manifest body must be valid JSON".to_owned()),
            Some(&namespace),
        )
    })?;
    let written = put_registry_manifest(
        pool,
        RegistryManifestPutRequest {
            namespace: &namespace,
            image: &image,
            reference: &reference,
            manifest,
            content_type,
            auth: &auth,
            user_agent,
        },
    )
    .await
    .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    let mut response_headers = registry_success_headers();
    response_headers.insert(
        header::LOCATION,
        HeaderValue::from_str(&format!("/v2/{namespace}/{image}/manifests/{reference}"))
            .unwrap_or_else(|_| HeaderValue::from_static("/")),
    );
    response_headers.insert(
        "docker-content-digest",
        HeaderValue::from_str(written.digest.as_deref().unwrap_or(""))
            .unwrap_or_else(|_| HeaderValue::from_static("")),
    );
    Ok((StatusCode::CREATED, response_headers, Json(json!({}))).into_response())
}

async fn registry_manifest_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((namespace, image, reference)): Path<(String, String, String)>,
) -> Result<Response, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(registry_database_unavailable)?;
    let auth = registry_auth_from_headers(pool, &headers)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok());
    let deleted = delete_registry_manifest(pool, &namespace, &image, &reference, &auth, user_agent)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    let mut response_headers = registry_success_headers();
    if let Some(digest) = deleted.digest {
        response_headers.insert(
            "docker-content-digest",
            HeaderValue::from_str(&digest).unwrap_or_else(|_| HeaderValue::from_static("")),
        );
    }
    Ok((StatusCode::ACCEPTED, response_headers).into_response())
}

async fn registry_blob(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((namespace, image, digest)): Path<(String, String, String)>,
) -> Result<Response, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    registry_blob_response(state, headers, namespace, image, digest, true).await
}

async fn registry_blob_head(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((namespace, image, digest)): Path<(String, String, String)>,
) -> Result<Response, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    registry_blob_response(state, headers, namespace, image, digest, false).await
}

async fn registry_blob_response(
    state: AppState,
    headers: HeaderMap,
    namespace: String,
    image: String,
    digest: String,
    include_body: bool,
) -> Result<Response, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(registry_database_unavailable)?;
    let auth = registry_auth_from_headers(pool, &headers)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok());
    let blob = read_registry_blob(pool, &namespace, &image, &digest, &auth, user_agent)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    let mut response_headers = registry_success_headers();
    response_headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&blob.media_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    response_headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&blob.bytes.len().to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("0")),
    );
    response_headers.insert(
        "docker-content-digest",
        HeaderValue::from_str(&blob.digest).unwrap_or_else(|_| HeaderValue::from_static("")),
    );
    if include_body {
        Ok((StatusCode::OK, response_headers, blob.bytes).into_response())
    } else {
        Ok((StatusCode::OK, response_headers).into_response())
    }
}

async fn registry_tags_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((namespace, image)): Path<(String, String)>,
) -> Result<Response, (StatusCode, HeaderMap, Json<serde_json::Value>)> {
    let pool = state
        .db
        .as_ref()
        .ok_or_else(registry_database_unavailable)?;
    let auth = registry_auth_from_headers(pool, &headers)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    let tags = list_registry_tags(pool, &namespace, &image, &auth)
        .await
        .map_err(|error| registry_error_response(error, Some(&namespace)))?;
    let mut headers = registry_success_headers();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    Ok((StatusCode::OK, headers, Json(json!(tags))).into_response())
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
        pool,
        RegistryManifestReadRequest {
            namespace: &namespace,
            image: &image,
            reference: &reference,
            accept,
            auth: &auth,
            user_agent,
            record_transfer: include_body,
        },
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

fn registry_success_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "docker-distribution-api-version",
        HeaderValue::from_static("registry/2.0"),
    );
    headers
}

fn upload_progress_response(
    status: StatusCode,
    upload: crate::domain::packages_registry::RegistryUploadProgress,
) -> Response {
    let mut response_headers = registry_success_headers();
    response_headers.insert(
        header::LOCATION,
        HeaderValue::from_str(&upload.location).unwrap_or_else(|_| HeaderValue::from_static("/")),
    );
    response_headers.insert(
        "docker-upload-uuid",
        HeaderValue::from_str(&upload.upload_id.to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("")),
    );
    response_headers.insert(
        header::RANGE,
        HeaderValue::from_str(&upload.range).unwrap_or_else(|_| HeaderValue::from_static("0-0")),
    );
    if let Some(digest) = upload.digest {
        response_headers.insert(
            "docker-content-digest",
            HeaderValue::from_str(&digest).unwrap_or_else(|_| HeaderValue::from_static("")),
        );
    }
    (status, response_headers, Json(json!({}))).into_response()
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
        RegistryError::InsufficientWriteScope => (
            StatusCode::FORBIDDEN,
            "DENIED",
            "token is missing packages:write scope",
        ),
        RegistryError::NotFound => (
            StatusCode::NOT_FOUND,
            "MANIFEST_UNKNOWN",
            "manifest unknown",
        ),
        RegistryError::BlobNotFound => (StatusCode::NOT_FOUND, "BLOB_UNKNOWN", "blob unknown"),
        RegistryError::UploadNotFound => (
            StatusCode::NOT_FOUND,
            "BLOB_UPLOAD_UNKNOWN",
            "blob upload unknown",
        ),
        RegistryError::DigestMismatch => (
            StatusCode::BAD_REQUEST,
            "DIGEST_INVALID",
            "upload digest does not match content",
        ),
        RegistryError::InvalidReference(message) => {
            return (
                StatusCode::BAD_REQUEST,
                headers,
                Json(json!({ "errors": [{ "code": "NAME_INVALID", "message": message }] })),
            );
        }
        RegistryError::InvalidManifest(message) => {
            return (
                StatusCode::BAD_REQUEST,
                headers,
                Json(json!({ "errors": [{ "code": "MANIFEST_INVALID", "message": message }] })),
            );
        }
        RegistryError::NotAcceptable => (
            StatusCode::NOT_ACCEPTABLE,
            "MANIFEST_INVALID",
            "requested manifest media type is not acceptable",
        ),
        RegistryError::Storage(_) | RegistryError::Sqlx(_) | RegistryError::Webhook(_) => (
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
struct UploadCompleteQuery {
    digest: String,
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
