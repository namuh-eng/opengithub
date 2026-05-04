use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope, RestJson},
    auth::extractor::AuthenticatedUser,
    domain::{
        organizations::{
            create_organization_from_signup, organization_people, organization_profile_settings,
            organization_repositories, organization_slug_availability, public_organization_profile,
            rename_organization, update_organization_profile_settings, CreateOrganizationRequest,
            CreatedOrganization, OrganizationCreateError, OrganizationPeopleList,
            OrganizationPeopleListQuery, OrganizationProfileError, OrganizationProfileSettings,
            OrganizationProfileSettingsPatch, OrganizationRepositoryList,
            OrganizationRepositoryListQuery, OrganizationSettingsError,
            OrganizationSlugAvailability, PublicOrganizationProfile, RenameOrganizationRequest,
        },
        packages::{
            mutate_package_settings, owner_packages, package_detail, package_settings,
            record_package_download_metadata, OwnerPackageList, OwnerPackageListQuery,
            PackageDetail, PackageDetailError, PackageDetailQuery, PackageDownloadMetadata,
            PackageListError, PackageOwnerKind, PackageSettings, PackageSettingsMutation,
        },
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/organizations/slug-availability",
            get(slug_availability),
        )
        .route("/api/organizations", post(create_organization))
        .route("/api/orgs/:org/profile", get(public_profile))
        .route(
            "/api/orgs/:org/settings/profile",
            get(get_profile_settings).patch(patch_profile_settings),
        )
        .route(
            "/api/orgs/:org/settings/profile/rename",
            post(rename_profile_settings),
        )
        .route("/api/orgs/:org/repositories", get(public_repositories))
        .route("/api/orgs/:org/people", get(public_people))
        .route("/api/orgs/:org/packages", get(public_packages))
        .route(
            "/api/orgs/:org/packages/:package_type/:package_name",
            get(public_package_detail),
        )
        .route(
            "/api/orgs/:org/packages/:package_type/:package_name/settings",
            get(public_package_settings).patch(update_package_settings),
        )
        .route(
            "/api/orgs/:org/packages/:package_type/:package_name/download",
            get(public_package_download_metadata),
        )
}

async fn slug_availability(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<OrganizationSlugAvailabilityQuery>,
) -> Result<Json<OrganizationSlugAvailability>, (StatusCode, Json<ErrorEnvelope>)> {
    AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let availability = organization_slug_availability(pool, &query.name)
        .await
        .map_err(map_organization_create_error)?;

    Ok(Json(availability))
}

async fn create_organization(
    State(state): State<AppState>,
    headers: HeaderMap,
    RestJson(request): RestJson<CreateOrganizationRequest>,
) -> Result<(StatusCode, Json<CreatedOrganization>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let organization = create_organization_from_signup(pool, actor.0.id, request)
        .await
        .map_err(map_organization_create_error)?;

    Ok((StatusCode::CREATED, Json(organization)))
}

async fn public_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org): Path<String>,
) -> Result<Json<PublicOrganizationProfile>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let profile = public_organization_profile(pool, &org, actor.map(|user| user.id))
        .await
        .map_err(map_organization_profile_error)?;

    Ok(Json(profile))
}

async fn get_profile_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org): Path<String>,
) -> Result<Json<OrganizationProfileSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = organization_profile_settings(pool, &org, actor.0.id)
        .await
        .map_err(map_organization_settings_error)?;

    Ok(Json(settings))
}

async fn patch_profile_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org): Path<String>,
    RestJson(request): RestJson<OrganizationProfileSettingsPatch>,
) -> Result<Json<OrganizationProfileSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_organization_profile_settings(pool, &org, actor.0.id, request)
        .await
        .map_err(map_organization_settings_error)?;

    Ok(Json(settings))
}

async fn rename_profile_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org): Path<String>,
    RestJson(request): RestJson<RenameOrganizationRequest>,
) -> Result<Json<OrganizationProfileSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = rename_organization(pool, &org, actor.0.id, request)
        .await
        .map_err(map_organization_settings_error)?;

    Ok(Json(settings))
}

async fn public_repositories(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org): Path<String>,
    Query(query): Query<OrganizationRepositoriesQuery>,
) -> Result<Json<OrganizationRepositoryList>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repositories = organization_repositories(
        pool,
        &org,
        actor.map(|user| user.id),
        OrganizationRepositoryListQuery {
            query: query.q.as_deref(),
            repository_type: query.repository_type.as_deref(),
            language: query.language.as_deref(),
            sort: query.sort.as_deref(),
            density: query.density.as_deref(),
            page: query.page,
            page_size: query.page_size,
        },
    )
    .await
    .map_err(map_organization_profile_error)?;

    Ok(Json(repositories))
}

async fn public_people(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org): Path<String>,
    Query(query): Query<OrganizationPeopleQuery>,
) -> Result<Json<OrganizationPeopleList>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let people = organization_people(
        pool,
        &org,
        actor.map(|user| user.id),
        OrganizationPeopleListQuery {
            query: query.q.as_deref(),
            page: query.page,
            page_size: query.page_size,
        },
    )
    .await
    .map_err(map_organization_profile_error)?;

    Ok(Json(people))
}

async fn public_packages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org): Path<String>,
    Query(query): Query<OwnerPackagesQuery>,
) -> Result<Json<OwnerPackageList>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let packages = owner_packages(
        pool,
        &org,
        PackageOwnerKind::Organization,
        actor.map(|user| user.id),
        OwnerPackageListQuery {
            query: query.q.as_deref(),
            package_type: query.package_type.as_deref(),
            visibility: query.visibility.as_deref(),
            sort: query.sort.as_deref(),
            artifact_tab: query.artifact_tab.as_deref(),
            page: query.page,
            page_size: query.page_size,
        },
    )
    .await
    .map_err(map_package_list_error)?;

    Ok(Json(packages))
}

async fn public_package_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((org, package_type, package_name)): Path<(String, String, String)>,
    Query(query): Query<PackageDetailRouteQuery>,
) -> Result<Json<PackageDetail>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let package = package_detail(
        pool,
        &org,
        PackageOwnerKind::Organization,
        &package_type,
        &package_name,
        actor.map(|user| user.id),
        PackageDetailQuery {
            version: query.version.as_deref(),
        },
    )
    .await
    .map_err(map_package_detail_error)?;

    Ok(Json(package))
}

async fn public_package_download_metadata(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((org, package_type, package_name)): Path<(String, String, String)>,
    Query(query): Query<PackageDetailRouteQuery>,
) -> Result<Json<PackageDownloadMetadata>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let metadata = record_package_download_metadata(
        pool,
        &org,
        PackageOwnerKind::Organization,
        &package_type,
        &package_name,
        actor.map(|user| user.id),
        PackageDetailQuery {
            version: query.version.as_deref(),
        },
    )
    .await
    .map_err(map_package_detail_error)?;

    Ok(Json(metadata))
}

async fn public_package_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((org, package_type, package_name)): Path<(String, String, String)>,
) -> Result<Json<PackageSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let settings = package_settings(
        pool,
        &org,
        PackageOwnerKind::Organization,
        &package_type,
        &package_name,
        actor.map(|user| user.id),
    )
    .await
    .map_err(map_package_detail_error)?;

    Ok(Json(settings))
}

async fn update_package_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((org, package_type, package_name)): Path<(String, String, String)>,
    Json(request): Json<PackageSettingsMutation>,
) -> Result<Json<PackageSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let settings = mutate_package_settings(
        pool,
        &org,
        PackageOwnerKind::Organization,
        &package_type,
        &package_name,
        actor.0.id,
        request,
    )
    .await
    .map_err(map_package_detail_error)?;

    Ok(Json(settings))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrganizationRepositoriesQuery {
    q: Option<String>,
    #[serde(rename = "type")]
    repository_type: Option<String>,
    language: Option<String>,
    sort: Option<String>,
    density: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrganizationSlugAvailabilityQuery {
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrganizationPeopleQuery {
    q: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OwnerPackagesQuery {
    q: Option<String>,
    #[serde(rename = "type")]
    package_type: Option<String>,
    visibility: Option<String>,
    sort: Option<String>,
    #[serde(alias = "artifact_tab")]
    artifact_tab: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackageDetailRouteQuery {
    version: Option<String>,
}

fn map_package_list_error(error: PackageListError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        PackageListError::NotFound => error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "package owner was not found",
        ),
        PackageListError::InvalidFilter(message) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            message,
        ),
        PackageListError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "packages could not be loaded",
        ),
    }
}

fn map_package_detail_error(error: PackageDetailError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        PackageDetailError::NotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", "package was not found")
        }
        PackageDetailError::Forbidden => error_response(
            StatusCode::FORBIDDEN,
            "forbidden",
            "package settings require admin access",
        ),
        PackageDetailError::InvalidSelection(message) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            message,
        ),
        PackageDetailError::Markdown(_) | PackageDetailError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "package could not be loaded",
        ),
    }
}

fn map_organization_profile_error(
    error: OrganizationProfileError,
) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        OrganizationProfileError::NotFound => error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "organization profile was not found",
        ),
        OrganizationProfileError::InvalidRepositoryFilter(message) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            message,
        ),
        OrganizationProfileError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "organization profile could not be loaded",
        ),
    }
}

fn map_organization_settings_error(
    error: OrganizationSettingsError,
) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        OrganizationSettingsError::NotFound => error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "organization settings were not found",
        ),
        OrganizationSettingsError::Forbidden => error_response(
            StatusCode::FORBIDDEN,
            "forbidden",
            "organization settings require owner access",
        ),
        OrganizationSettingsError::Validation(message) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            message,
        ),
        OrganizationSettingsError::Conflict => error_response(
            StatusCode::CONFLICT,
            "conflict",
            "organization slug is already taken",
        ),
        OrganizationSettingsError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "organization settings could not be loaded",
        ),
    }
}

fn map_organization_create_error(
    error: OrganizationCreateError,
) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        OrganizationCreateError::Validation(message) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            message,
        ),
        OrganizationCreateError::ReservedSlug => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "organization slug is reserved",
        ),
        OrganizationCreateError::DuplicateSlug => error_response(
            StatusCode::CONFLICT,
            "conflict",
            "organization slug is already taken",
        ),
        OrganizationCreateError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "organization could not be created",
        ),
    }
}
