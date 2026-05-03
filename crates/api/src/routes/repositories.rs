use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
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
    domain::actions_secrets::{
        create_repository_actions_secret_by_owner_name,
        create_repository_actions_variable_by_owner_name,
        delete_repository_actions_secret_by_owner_name,
        delete_repository_actions_variable_by_owner_name,
        repository_actions_secrets_settings_for_actor_by_owner_name,
        update_repository_actions_secret_by_owner_name,
        update_repository_actions_variable_by_owner_name, ActionsSecretMutation,
        ActionsSecretsError, ActionsVariableMutation,
    },
    domain::pages::{
        connect_repository_pages_actions_deployment_by_owner_name,
        recheck_repository_pages_dns_by_owner_name, remove_repository_pages_domain_by_owner_name,
        repository_pages_settings_for_actor_by_owner_name,
        request_repository_pages_deployment_by_owner_name,
        save_repository_pages_domain_by_owner_name, unpublish_repository_pages_by_owner_name,
        update_repository_pages_https_by_owner_name, update_repository_pages_source_by_owner_name,
        PagesActionsDeploymentMutation, PagesDomainMutation, PagesError, PagesHttpsMutation,
        PagesSourceMutation,
    },
    domain::releases::{
        cancel_repository_release_upload_intent_by_owner_name,
        complete_repository_release_upload_intent_by_owner_name,
        create_repository_release_asset_by_owner_name, create_repository_release_by_owner_name,
        create_repository_release_upload_intent_by_owner_name,
        delete_repository_release_asset_by_owner_name, delete_repository_release_by_owner_name,
        generate_repository_release_notes_by_owner_name, publish_repository_release_by_owner_name,
        repository_latest_release_by_owner_name, repository_release_archive_metadata_by_owner_name,
        repository_release_asset_download_by_owner_name,
        repository_release_detail_by_id_by_owner_name,
        repository_release_detail_by_tag_by_owner_name, repository_release_list_by_owner_name,
        repository_release_management_context_by_owner_name, repository_release_tags_by_owner_name,
        toggle_repository_release_reaction_by_owner_name, update_repository_release_by_owner_name,
        GeneratedReleaseNotesRequest, ReleaseAssetMutation, ReleaseMutation,
        ReleaseUploadCancelRequest, ReleaseUploadCompleteRequest, ReleaseUploadIntentRequest,
        ReleasesError,
    },
    domain::repositories::{
        cancel_repository_invitation_by_owner_name, create_repository_branch_rule_by_owner_name,
        create_repository_ruleset_by_owner_name, create_repository_with_bootstrap,
        delete_repository_branch_rule_by_owner_name, delete_repository_ruleset_by_owner_name,
        fork_repository_by_owner_name, grant_repository_team_access_by_owner_name,
        insert_repository_create_feed_event, invite_repository_access_by_owner_name,
        list_repositories_for_user, remove_repository_collaborator_access_by_owner_name,
        remove_repository_team_access_by_owner_name,
        repository_access_settings_for_actor_by_owner_name,
        repository_blame_for_actor_by_owner_name, repository_blob_for_actor_by_owner_name,
        repository_branch_settings_for_actor_by_owner_name,
        repository_commit_history_for_actor_by_owner_name, repository_creation_options,
        repository_file_finder_for_actor_by_owner_name, repository_name_availability,
        repository_overview_for_viewer_by_owner_name,
        repository_path_overview_for_actor_by_owner_name, repository_refs_for_actor_by_owner_name,
        repository_settings_for_actor_by_owner_name, set_repository_star_by_owner_name,
        set_repository_watch_by_owner_name, update_repository_branch_rule_by_owner_name,
        update_repository_collaborator_access_by_owner_name,
        update_repository_ruleset_by_owner_name, update_repository_settings_by_owner_name,
        update_repository_team_access_by_owner_name, CreateRepository,
        RepositoryAccessInviteRequest, RepositoryAccessRolePatch, RepositoryAccessTeamGrantRequest,
        RepositoryBootstrapRequest, RepositoryBranchRuleMutation, RepositoryCommitHistoryQuery,
        RepositoryError, RepositoryFileFinderQuery, RepositoryOwner, RepositoryPathQuery,
        RepositoryRefsQuery, RepositoryRulesetMutation, RepositorySettingsPatch,
        RepositoryVisibility,
    },
    domain::webhooks::{
        create_repository_webhook_by_owner_name, delete_repository_webhook_by_owner_name,
        ping_repository_webhook_by_owner_name, redeliver_repository_webhook_delivery_by_owner_name,
        repository_webhook_delivery_for_actor_by_owner_name,
        repository_webhook_detail_for_actor_by_owner_name,
        repository_webhook_settings_for_actor_by_owner_name,
        update_repository_webhook_by_owner_name, WebhookError, WebhookMutation,
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
        .route("/:owner/:repo/releases", get(releases).post(create_release))
        .route("/:owner/:repo/releases/manage", get(release_manage_new))
        .route(
            "/:owner/:repo/releases/manage/generated-notes",
            post(release_generated_notes),
        )
        .route(
            "/:owner/:repo/releases/manage/upload-intents",
            post(create_release_upload_intent),
        )
        .route(
            "/:owner/:repo/releases/manage/upload-intents/:intent_id/complete",
            post(complete_release_upload_intent),
        )
        .route(
            "/:owner/:repo/releases/manage/upload-intents/:intent_id/cancel",
            post(cancel_release_upload_intent),
        )
        .route(
            "/:owner/:repo/releases/manage/:release_id",
            get(release_manage_edit),
        )
        .route("/:owner/:repo/releases/latest", get(latest_release))
        .route("/:owner/:repo/releases/tags", get(release_tags))
        .route(
            "/:owner/:repo/releases/zipball/*tag",
            get(release_zipball_metadata),
        )
        .route(
            "/:owner/:repo/releases/tarball/*tag",
            get(release_tarball_metadata),
        )
        .route(
            "/:owner/:repo/releases/assets/:asset_id",
            get(release_asset_download),
        )
        .route(
            "/:owner/:repo/releases/:release_id/assets",
            post(create_release_asset),
        )
        .route(
            "/:owner/:repo/releases/:release_id/assets/:asset_id",
            delete(delete_release_asset),
        )
        .route(
            "/:owner/:repo/releases/:release_id/publish",
            post(publish_release),
        )
        .route(
            "/:owner/:repo/releases/:release_id/reactions",
            post(release_reaction),
        )
        .route("/:owner/:repo/releases/tag/*tag", get(release_by_tag))
        .route(
            "/:owner/:repo/releases/:release_id",
            get(release_by_id)
                .patch(update_release)
                .delete(delete_release),
        )
        .route(
            "/:owner/:repo/settings",
            get(settings).patch(update_settings),
        )
        .route(
            "/:owner/:repo/settings/access",
            get(access_settings).post(invite_access),
        )
        .route(
            "/:owner/:repo/settings/access/collaborators/:user_id",
            patch(update_collaborator_access).delete(remove_collaborator_access),
        )
        .route(
            "/:owner/:repo/settings/access/teams",
            post(grant_team_access),
        )
        .route(
            "/:owner/:repo/settings/access/teams/:team_id",
            patch(update_team_access).delete(remove_team_access),
        )
        .route(
            "/:owner/:repo/settings/access/invitations/:invitation_id",
            delete(cancel_invitation),
        )
        .route("/:owner/:repo/settings/branches", get(branch_settings))
        .route(
            "/:owner/:repo/settings/branches/rules",
            post(create_branch_rule),
        )
        .route(
            "/:owner/:repo/settings/branches/rules/:rule_id",
            patch(update_branch_rule).delete(delete_branch_rule),
        )
        .route(
            "/:owner/:repo/settings/branches/rulesets",
            post(create_ruleset),
        )
        .route(
            "/:owner/:repo/settings/branches/rulesets/:ruleset_id",
            patch(update_ruleset).delete(delete_ruleset),
        )
        .route(
            "/:owner/:repo/settings/hooks",
            get(webhook_settings).post(create_webhook),
        )
        .route(
            "/:owner/:repo/settings/hooks/:hook_id",
            get(webhook_detail)
                .patch(update_webhook)
                .delete(delete_webhook),
        )
        .route(
            "/:owner/:repo/settings/hooks/:hook_id/ping",
            post(ping_webhook),
        )
        .route(
            "/:owner/:repo/settings/hooks/:hook_id/deliveries/:delivery_id",
            get(webhook_delivery_detail),
        )
        .route(
            "/:owner/:repo/settings/hooks/:hook_id/deliveries/:delivery_id/redeliver",
            post(redeliver_webhook_delivery),
        )
        .route(
            "/:owner/:repo/settings/secrets",
            get(actions_secrets_settings),
        )
        .route(
            "/:owner/:repo/settings/secrets/secrets",
            post(create_actions_secret),
        )
        .route(
            "/:owner/:repo/settings/secrets/secrets/:secret_name",
            patch(update_actions_secret).delete(delete_actions_secret),
        )
        .route(
            "/:owner/:repo/settings/secrets/variables",
            post(create_actions_variable),
        )
        .route(
            "/:owner/:repo/settings/secrets/variables/:variable_name",
            patch(update_actions_variable).delete(delete_actions_variable),
        )
        .route("/:owner/:repo/settings/pages", get(pages_settings))
        .route(
            "/:owner/:repo/settings/pages/source",
            patch(update_pages_source),
        )
        .route(
            "/:owner/:repo/settings/pages/domain",
            post(save_pages_domain).delete(remove_pages_domain),
        )
        .route(
            "/:owner/:repo/settings/pages/domain/recheck",
            post(recheck_pages_dns),
        )
        .route(
            "/:owner/:repo/settings/pages/https",
            patch(update_pages_https),
        )
        .route(
            "/:owner/:repo/settings/pages/deployments",
            post(request_pages_deployment),
        )
        .route(
            "/:owner/:repo/settings/pages/actions-deployments",
            post(connect_pages_actions_deployment),
        )
        .route(
            "/:owner/:repo/settings/pages/unpublish",
            post(unpublish_pages),
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
struct ReleasesQuery {
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseReactionRequest {
    content: String,
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

async fn releases(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ReleasesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let envelope = repository_release_list_by_owner_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
        pagination.page,
        pagination.page_size,
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(envelope)))
}

async fn create_release(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<ReleaseMutation>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = create_repository_release_by_owner_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
        request,
    )
    .await
    .map_err(map_releases_error)?;

    Ok((StatusCode::CREATED, Json(json!(release))))
}

async fn release_manage_new(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let context = repository_release_management_context_by_owner_name(
        pool,
        &owner,
        &repo,
        None,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(context)))
}

async fn release_manage_edit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let context = repository_release_management_context_by_owner_name(
        pool,
        &owner,
        &repo,
        Some(release_id),
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(context)))
}

async fn release_generated_notes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<GeneratedReleaseNotesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let preview = generate_repository_release_notes_by_owner_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
        request,
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(preview)))
}

async fn create_release_upload_intent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<ReleaseUploadIntentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let intent = create_repository_release_upload_intent_by_owner_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
        request,
    )
    .await
    .map_err(map_releases_error)?;

    Ok((StatusCode::CREATED, Json(json!(intent))))
}

async fn complete_release_upload_intent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, intent_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<ReleaseUploadCompleteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = complete_repository_release_upload_intent_by_owner_name(
        pool,
        &owner,
        &repo,
        intent_id,
        actor.as_ref().map(|user| user.id),
        request,
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(release)))
}

async fn cancel_release_upload_intent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, intent_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<ReleaseUploadCancelRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let intent = cancel_repository_release_upload_intent_by_owner_name(
        pool,
        &owner,
        &repo,
        intent_id,
        actor.as_ref().map(|user| user.id),
        request,
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(intent)))
}

async fn latest_release(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = repository_latest_release_by_owner_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(release)))
}

async fn update_release(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<ReleaseMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = update_repository_release_by_owner_name(
        pool,
        &owner,
        &repo,
        release_id,
        actor.as_ref().map(|user| user.id),
        request,
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(release)))
}

async fn publish_release(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = publish_repository_release_by_owner_name(
        pool,
        &owner,
        &repo,
        release_id,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(release)))
}

async fn delete_release(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<ReleaseMutation>,
) -> Result<StatusCode, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    delete_repository_release_by_owner_name(
        pool,
        &owner,
        &repo,
        release_id,
        actor.as_ref().map(|user| user.id),
        request.delete_tag.unwrap_or(false),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn release_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = repository_release_detail_by_id_by_owner_name(
        pool,
        &owner,
        &repo,
        release_id,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(release)))
}

async fn release_by_tag(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, tag)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = repository_release_detail_by_tag_by_owner_name(
        pool,
        &owner,
        &repo,
        &tag,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(release)))
}

async fn release_tags(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ReleasesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let envelope = repository_release_tags_by_owner_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
        pagination.page,
        pagination.page_size,
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(envelope)))
}

async fn release_zipball_metadata(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, tag)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    release_archive_metadata(state, headers, owner, repo, tag, "zipball").await
}

async fn release_tarball_metadata(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, tag)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    release_archive_metadata(state, headers, owner, repo, tag, "tarball").await
}

async fn release_archive_metadata(
    state: AppState,
    headers: HeaderMap,
    owner: String,
    repo: String,
    tag: String,
    format: &str,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let metadata = repository_release_archive_metadata_by_owner_name(
        pool,
        &owner,
        &repo,
        &tag,
        format,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(metadata)))
}

async fn release_asset_download(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, asset_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let metadata = repository_release_asset_download_by_owner_name(
        pool,
        &owner,
        &repo,
        asset_id,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(metadata)))
}

async fn create_release_asset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<ReleaseAssetMutation>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = create_repository_release_asset_by_owner_name(
        pool,
        &owner,
        &repo,
        release_id,
        actor.as_ref().map(|user| user.id),
        request,
    )
    .await
    .map_err(map_releases_error)?;

    Ok((StatusCode::CREATED, Json(json!(release))))
}

async fn delete_release_asset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id, asset_id)): Path<(String, String, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = delete_repository_release_asset_by_owner_name(
        pool,
        &owner,
        &repo,
        release_id,
        asset_id,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(release)))
}

async fn release_reaction(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<ReleaseReactionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let reactions = toggle_repository_release_reaction_by_owner_name(
        pool,
        &owner,
        &repo,
        release_id,
        actor.as_ref().map(|user| user.id),
        &request.content,
    )
    .await
    .map_err(map_releases_error)?;

    Ok((StatusCode::CREATED, Json(json!(reactions))))
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
    let settings = repository_settings_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo)
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
    RestJson(patch): RestJson<RepositorySettingsPatch>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_settings_by_owner_name(pool, actor.0.id, &owner, &repo, patch)
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

async fn access_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        repository_access_settings_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo)
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

async fn invite_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<RepositoryAccessInviteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = invite_repository_access_by_owner_name(pool, actor.0.id, &owner, &repo, request)
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

async fn grant_team_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<RepositoryAccessTeamGrantRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        grant_repository_team_access_by_owner_name(pool, actor.0.id, &owner, &repo, request)
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

async fn update_team_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, team_id)): Path<(String, String, Uuid)>,
    RestJson(patch): RestJson<RepositoryAccessRolePatch>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_team_access_by_owner_name(
        pool, actor.0.id, &owner, &repo, team_id, patch,
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

    Ok(Json(json!(settings)))
}

async fn remove_team_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, team_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        remove_repository_team_access_by_owner_name(pool, actor.0.id, &owner, &repo, team_id)
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

async fn update_collaborator_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, user_id)): Path<(String, String, Uuid)>,
    RestJson(patch): RestJson<RepositoryAccessRolePatch>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_collaborator_access_by_owner_name(
        pool, actor.0.id, &owner, &repo, user_id, patch,
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

    Ok(Json(json!(settings)))
}

async fn remove_collaborator_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, user_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = remove_repository_collaborator_access_by_owner_name(
        pool, actor.0.id, &owner, &repo, user_id,
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

    Ok(Json(json!(settings)))
}

async fn cancel_invitation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, invitation_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        cancel_repository_invitation_by_owner_name(pool, actor.0.id, &owner, &repo, invitation_id)
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

async fn branch_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        repository_branch_settings_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo)
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

async fn create_branch_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<RepositoryBranchRuleMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        create_repository_branch_rule_by_owner_name(pool, actor.0.id, &owner, &repo, request)
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

async fn update_branch_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, rule_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<RepositoryBranchRuleMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_branch_rule_by_owner_name(
        pool, actor.0.id, &owner, &repo, rule_id, request,
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

    Ok(Json(json!(settings)))
}

async fn delete_branch_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, rule_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        delete_repository_branch_rule_by_owner_name(pool, actor.0.id, &owner, &repo, rule_id)
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

async fn create_ruleset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<RepositoryRulesetMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        create_repository_ruleset_by_owner_name(pool, actor.0.id, &owner, &repo, request)
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

async fn update_ruleset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, ruleset_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<RepositoryRulesetMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_ruleset_by_owner_name(
        pool, actor.0.id, &owner, &repo, ruleset_id, request,
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

    Ok(Json(json!(settings)))
}

async fn delete_ruleset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, ruleset_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        delete_repository_ruleset_by_owner_name(pool, actor.0.id, &owner, &repo, ruleset_id)
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

async fn webhook_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        repository_webhook_settings_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo)
            .await
            .map_err(map_webhook_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn webhook_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, hook_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail =
        repository_webhook_detail_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo, hook_id)
            .await
            .map_err(map_webhook_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(detail)))
}

async fn webhook_delivery_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, hook_id, delivery_id)): Path<(String, String, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail = repository_webhook_delivery_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        hook_id,
        delivery_id,
    )
    .await
    .map_err(map_webhook_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(detail)))
}

async fn create_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<WebhookMutation>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let result = create_repository_webhook_by_owner_name(pool, actor.0.id, &owner, &repo, request)
        .await
        .map_err(map_webhook_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok((StatusCode::CREATED, Json(json!(result))))
}

async fn update_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, hook_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<WebhookMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        update_repository_webhook_by_owner_name(pool, actor.0.id, &owner, &repo, hook_id, request)
            .await
            .map_err(map_webhook_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn delete_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, hook_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        delete_repository_webhook_by_owner_name(pool, actor.0.id, &owner, &repo, hook_id)
            .await
            .map_err(map_webhook_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn ping_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, hook_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let result = ping_repository_webhook_by_owner_name(pool, actor.0.id, &owner, &repo, hook_id)
        .await
        .map_err(map_webhook_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(result)))
}

async fn redeliver_webhook_delivery(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, hook_id, delivery_id)): Path<(String, String, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let result = redeliver_repository_webhook_delivery_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        hook_id,
        delivery_id,
    )
    .await
    .map_err(map_webhook_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(result)))
}

async fn actions_secrets_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = repository_actions_secrets_settings_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo,
    )
    .await
    .map_err(map_actions_secrets_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn create_actions_secret(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<ActionsSecretMutation>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        create_repository_actions_secret_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_actions_secrets_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok((StatusCode::CREATED, Json(json!(settings))))
}

async fn update_actions_secret(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, secret_name)): Path<(String, String, String)>,
    RestJson(request): RestJson<ActionsSecretMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_actions_secret_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        &secret_name,
        request,
    )
    .await
    .map_err(map_actions_secrets_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn delete_actions_secret(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, secret_name)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = delete_repository_actions_secret_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        &secret_name,
    )
    .await
    .map_err(map_actions_secrets_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn create_actions_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<ActionsVariableMutation>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        create_repository_actions_variable_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_actions_secrets_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok((StatusCode::CREATED, Json(json!(settings))))
}

async fn update_actions_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, variable_name)): Path<(String, String, String)>,
    RestJson(request): RestJson<ActionsVariableMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_actions_variable_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        &variable_name,
        request,
    )
    .await
    .map_err(map_actions_secrets_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn delete_actions_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, variable_name)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = delete_repository_actions_variable_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        &variable_name,
    )
    .await
    .map_err(map_actions_secrets_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn pages_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        repository_pages_settings_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo)
            .await
            .map_err(map_pages_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn update_pages_source(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<PagesSourceMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        update_repository_pages_source_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_pages_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn save_pages_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<PagesDomainMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        save_repository_pages_domain_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_pages_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn remove_pages_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = remove_repository_pages_domain_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_pages_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(settings)))
}

async fn recheck_pages_dns(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = recheck_repository_pages_dns_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_pages_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(settings)))
}

async fn update_pages_https(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<PagesHttpsMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        update_repository_pages_https_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_pages_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn request_pages_deployment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let result = request_repository_pages_deployment_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_pages_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(result)))
}

async fn connect_pages_actions_deployment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<PagesActionsDeploymentMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let result = connect_repository_pages_actions_deployment_by_owner_name(
        pool, actor.0.id, &owner, &repo, request,
    )
    .await
    .map_err(map_pages_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(result)))
}

async fn unpublish_pages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = unpublish_repository_pages_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_pages_error)?
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
        | RepositoryError::InvalidMergeMethod(_)
        | RepositoryError::InvalidAccessRole(_)
        | RepositoryError::InvalidBranchPolicy(_)
        | RepositoryError::MergeMethodRequired
        | RepositoryError::DefaultMergeMethodDisabled
        | RepositoryError::ArchivedRepositoryReadOnly
        | RepositoryError::UnknownTemplate(_)
        | RepositoryError::UnknownGitignoreTemplate(_)
        | RepositoryError::UnknownLicenseTemplate(_) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        RepositoryError::DefaultBranchNotFound(_) => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::ForkAlreadyExists => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::AccessTargetNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        RepositoryError::AccessGrantConflict => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::LastAdminAccess => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::BranchPolicyConflict => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::BranchPolicyNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        RepositoryError::TeamAccessUnsupported => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
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
                "repository already exists for this owner".to_owned(),
            )
        }
        RepositoryError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "repository operation failed".to_owned(),
        ),
    }
}

fn map_webhook_error(error: WebhookError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        WebhookError::RepositoryNotFound
        | WebhookError::WebhookNotFound
        | WebhookError::DeliveryNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        WebhookError::RepositoryAccessDenied => {
            error_response(StatusCode::FORBIDDEN, "forbidden", error.to_string())
        }
        WebhookError::InvalidWebhook(_)
        | WebhookError::InvalidDeliveryStatus(_)
        | WebhookError::DeliveryQueue(_) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        WebhookError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "webhook operation failed".to_owned(),
        ),
    }
}

fn map_actions_secrets_error(error: ActionsSecretsError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        ActionsSecretsError::Repository(error) => map_repository_error(error),
        ActionsSecretsError::Invalid(_) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        ActionsSecretsError::Conflict => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        ActionsSecretsError::NotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        ActionsSecretsError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Actions secrets operation failed".to_owned(),
        ),
    }
}

fn map_pages_error(error: PagesError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        PagesError::Repository(error) => map_repository_error(error),
        PagesError::Invalid(_) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        PagesError::Conflict => error_response(StatusCode::CONFLICT, "conflict", error.to_string()),
        PagesError::NotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        PagesError::Job(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "job_enqueue_failed",
            "Pages deployment could not be queued".to_owned(),
        ),
        PagesError::Sqlx(sqlx::Error::Database(ref database_error))
            if database_error.is_unique_violation() =>
        {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        PagesError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Pages settings operation failed".to_owned(),
        ),
    }
}

fn map_releases_error(error: ReleasesError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        ReleasesError::Repository(RepositoryError::PermissionDenied) => error_response(
            StatusCode::FORBIDDEN,
            "forbidden",
            "user does not have repository access".to_owned(),
        ),
        ReleasesError::Repository(error) => map_repository_error(error),
        ReleasesError::NotFound | ReleasesError::TagNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        ReleasesError::UnsupportedArchiveFormat => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        ReleasesError::UnsupportedReaction => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        ReleasesError::Validation(_) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        ReleasesError::Conflict(_) => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        ReleasesError::ArchivedRepository | ReleasesError::ImmutableRelease => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        ReleasesError::AuthenticationRequired => {
            error_response(StatusCode::UNAUTHORIZED, "unauthorized", error.to_string())
        }
        ReleasesError::Markdown => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "release notes could not be rendered".to_owned(),
        ),
        ReleasesError::Webhook(_) | ReleasesError::Job(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "release side effect queueing failed".to_owned(),
        ),
        ReleasesError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "release operation failed".to_owned(),
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
