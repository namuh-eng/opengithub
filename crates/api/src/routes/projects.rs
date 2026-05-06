use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope},
    auth::extractor::AuthenticatedUser,
    domain::projects::{
        add_project_item_for_actor, archive_project_item_for_actor,
        bulk_add_project_items_for_actor, close_project_for_actor,
        convert_project_draft_to_issue_for_actor, copy_project_for_actor,
        create_project_access_grant_for_actor, create_project_field_for_actor,
        create_project_field_option_for_actor, create_project_insights_chart_for_actor,
        create_project_item_comment_for_actor, create_project_iteration_break_for_actor,
        create_project_iteration_for_actor, create_project_status_update_for_actor,
        delete_project_access_grant_for_actor, delete_project_field_for_actor,
        delete_project_field_option_for_actor, delete_project_for_actor,
        delete_project_insights_chart_for_actor, delete_project_item_comment_for_actor,
        delete_project_iteration_break_for_actor, invoke_project_automation_for_actor,
        link_project_repository_for_actor, organization_projects,
        project_conversion_targets_for_actor, project_field_settings, project_insights,
        project_item_detail, project_items_archived, project_settings, project_workflow_settings,
        project_workspace, remove_project_item_for_actor, reopen_project_for_actor,
        reorder_project_field_options_for_actor, repository_projects,
        restore_project_item_for_actor, unlink_project_repository_for_actor,
        update_project_access_grant_for_actor, update_project_draft_item_for_actor,
        update_project_field_for_actor, update_project_field_option_for_actor,
        update_project_insights_chart_for_actor, update_project_item_comment_for_actor,
        update_project_item_field_for_actor, update_project_item_position_for_actor,
        update_project_iteration_for_actor, update_project_iteration_settings_for_actor,
        update_project_roadmap_settings_for_actor, update_project_settings_for_actor,
        update_project_template_for_actor, update_project_view_layout_for_actor,
        update_project_view_state_for_actor, update_project_workflow_for_actor, user_projects,
        CopiedProject, CopyProjectRequest, ProjectAccessGrantCreateRequest,
        ProjectAccessGrantDeleteRequest, ProjectAccessGrantUpdateRequest, ProjectArchivedItem,
        ProjectAutomationInvocationRequest, ProjectAutomationInvocationResponse,
        ProjectConversionTargets, ProjectDeleteResponse, ProjectDraftConvertRequest,
        ProjectDraftUpdateRequest, ProjectFieldCreateRequest, ProjectFieldDeleteRequest,
        ProjectFieldOptionCreateRequest, ProjectFieldOptionReorderRequest,
        ProjectFieldOptionUpdateRequest, ProjectFieldSettings, ProjectFieldUpdateRequest,
        ProjectInsights, ProjectInsightsChartMutationRequest, ProjectInsightsQuery,
        ProjectItemAddRequest, ProjectItemCommentCreateRequest, ProjectItemCommentUpdateRequest,
        ProjectItemDetail, ProjectItemFieldValueRequest, ProjectItemPositionRequest,
        ProjectItemsArchivedQuery, ProjectItemsBulkAddRequest, ProjectIterationBreakCreateRequest,
        ProjectIterationCreateRequest, ProjectIterationSettingsRequest,
        ProjectIterationUpdateRequest, ProjectLifecycleRequest, ProjectList, ProjectListQuery,
        ProjectRepositoryLinkRequest, ProjectRoadmapSettingsRequest, ProjectSettings,
        ProjectSettingsUpdateRequest, ProjectStatusUpdateRequest, ProjectTemplateUpdateRequest,
        ProjectViewLayoutRequest, ProjectViewStateRequest, ProjectWorkflowSettings,
        ProjectWorkflowUpdateRequest, ProjectWorkspace, ProjectWorkspaceQuery, ProjectsError,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/users/:username/projects", get(user_projects_route))
        .route("/api/orgs/:org/projects", get(organization_projects_route))
        .route(
            "/api/projects/:project_id/workspace",
            get(project_workspace_route),
        )
        .route(
            "/api/projects/:project_id/insights",
            get(project_insights_route),
        )
        .route(
            "/api/projects/:project_id/charts",
            post(create_project_insights_chart_route),
        )
        .route(
            "/api/projects/:project_id/charts/:chart_id",
            patch(update_project_insights_chart_route).delete(delete_project_insights_chart_route),
        )
        .route(
            "/api/projects/:project_id/settings",
            get(project_settings_route).patch(update_project_settings_route),
        )
        .route(
            "/api/projects/:project_id/settings/access",
            get(project_settings_route),
        )
        .route(
            "/api/projects/:project_id/access-grants",
            post(create_project_access_grant_route),
        )
        .route(
            "/api/projects/:project_id/access-grants/:grant_id",
            patch(update_project_access_grant_route).delete(delete_project_access_grant_route),
        )
        .route(
            "/api/projects/:project_id/repositories/:repository_id",
            post(link_project_repository_route).delete(unlink_project_repository_route),
        )
        .route(
            "/api/projects/:project_id/status-updates",
            post(create_project_status_update_route),
        )
        .route(
            "/api/projects/:project_id/template",
            patch(update_project_template_route),
        )
        .route("/api/projects/:project_id/close", post(close_project_route))
        .route(
            "/api/projects/:project_id/reopen",
            post(reopen_project_route),
        )
        .route("/api/projects/:project_id", delete(delete_project_route))
        .route(
            "/api/projects/:project_id/settings/fields",
            get(project_field_settings_route).post(create_project_field_route),
        )
        .route(
            "/api/projects/:project_id/workflows",
            get(project_workflow_settings_route),
        )
        .route(
            "/api/projects/:project_id/workflows/:workflow_id",
            patch(update_project_workflow_route),
        )
        .route(
            "/api/projects/:project_id/automation/invocations",
            post(invoke_project_automation_route),
        )
        .route(
            "/api/projects/:project_id/fields/:field_id",
            patch(update_project_field_route).delete(delete_project_field_route),
        )
        .route(
            "/api/projects/:project_id/fields/:field_id/options",
            post(create_project_field_option_route),
        )
        .route(
            "/api/projects/:project_id/fields/:field_id/options/reorder",
            patch(reorder_project_field_options_route),
        )
        .route(
            "/api/projects/:project_id/fields/:field_id/options/:option_id",
            patch(update_project_field_option_route).delete(delete_project_field_option_route),
        )
        .route(
            "/api/projects/:project_id/fields/:field_id/iterations/settings",
            patch(update_project_iteration_settings_route),
        )
        .route(
            "/api/projects/:project_id/fields/:field_id/iterations",
            post(create_project_iteration_route),
        )
        .route(
            "/api/projects/:project_id/fields/:field_id/iterations/:iteration_id",
            patch(update_project_iteration_route),
        )
        .route(
            "/api/projects/:project_id/fields/:field_id/iteration-breaks",
            post(create_project_iteration_break_route),
        )
        .route(
            "/api/projects/:project_id/fields/:field_id/iteration-breaks/:break_id",
            delete(delete_project_iteration_break_route),
        )
        .route(
            "/api/projects/:project_id/views/:view_id/state",
            patch(update_project_view_state_route),
        )
        .route(
            "/api/projects/:project_id/views/:view_id/layout",
            patch(update_project_view_layout_route),
        )
        .route(
            "/api/projects/:project_id/views/:view_id/roadmap-settings",
            patch(update_project_roadmap_settings_route),
        )
        .route(
            "/api/projects/:project_id/items/:item_id/fields/:field_id",
            patch(update_project_item_field_route),
        )
        .route(
            "/api/projects/:project_id/items/:item_id/draft",
            patch(update_project_draft_item_route),
        )
        .route(
            "/api/projects/:project_id/items/:item_id/comments",
            post(create_project_item_comment_route),
        )
        .route(
            "/api/projects/:project_id/items/:item_id/comments/:comment_id",
            patch(update_project_item_comment_route).delete(delete_project_item_comment_route),
        )
        .route(
            "/api/projects/:project_id/conversion-targets",
            get(project_conversion_targets_route),
        )
        .route(
            "/api/projects/:project_id/items/:item_id/convert-to-issue",
            post(convert_project_draft_to_issue_route),
        )
        .route(
            "/api/projects/:project_id/items/:item_id/archive",
            patch(archive_project_item_route),
        )
        .route(
            "/api/projects/:project_id/items/:item_id/restore",
            patch(restore_project_item_route),
        )
        .route(
            "/api/projects/:project_id/items/archived",
            get(project_items_archived_route),
        )
        .route(
            "/api/projects/:project_id/items/:item_id",
            get(project_item_detail_route).delete(remove_project_item_route),
        )
        .route(
            "/api/projects/:project_id/items",
            post(add_project_item_route),
        )
        .route(
            "/api/projects/:project_id/items/bulk",
            post(bulk_add_project_items_route),
        )
        .route(
            "/api/projects/:project_id/items/:item_id/position",
            patch(update_project_item_position_route),
        )
        .route("/api/projects/:project_id/copies", post(copy_project_route))
        .route(
            "/api/repos/:owner/:repo/projects",
            get(repository_projects_route),
        )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectsQuery {
    q: Option<String>,
    state: Option<String>,
    tab: Option<String>,
    sort: Option<String>,
    page: Option<i64>,
    #[serde(rename = "pageSize")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectWorkspaceRouteQuery {
    view: Option<String>,
    q: Option<String>,
    sort: Option<String>,
    group: Option<String>,
    slice: Option<String>,
    page: Option<i64>,
    #[serde(rename = "pageSize")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectItemsArchivedRouteQuery {
    item_type: Option<String>,
    q: Option<String>,
    page: Option<i64>,
    #[serde(rename = "pageSize")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectInsightsRouteQuery {
    chart: Option<String>,
    range: Option<String>,
    start: Option<chrono::NaiveDate>,
    end: Option<chrono::NaiveDate>,
    filter: Option<String>,
    table: Option<bool>,
}

impl ProjectWorkspaceRouteQuery {
    fn as_domain_query(&self) -> ProjectWorkspaceQuery<'_> {
        ProjectWorkspaceQuery {
            view: self.view.as_deref(),
            query: self.q.as_deref(),
            sort: self.sort.as_deref(),
            group: self.group.as_deref(),
            slice: self.slice.as_deref(),
            page: self.page,
            page_size: self.page_size,
        }
    }
}

impl ProjectItemsArchivedRouteQuery {
    fn as_domain_query(&self) -> ProjectItemsArchivedQuery<'_> {
        ProjectItemsArchivedQuery {
            item_type: self.item_type.as_deref(),
            query: self.q.as_deref(),
            page: self.page,
            page_size: self.page_size,
        }
    }
}

impl ProjectInsightsRouteQuery {
    fn as_domain_query(&self) -> ProjectInsightsQuery<'_> {
        ProjectInsightsQuery {
            chart: self.chart.as_deref(),
            range: self.range.as_deref(),
            start: self.start,
            end: self.end,
            filter: self.filter.as_deref(),
            table: self.table,
        }
    }
}

impl ProjectsQuery {
    fn as_domain_query(&self) -> ProjectListQuery<'_> {
        ProjectListQuery {
            query: self.q.as_deref(),
            state: self.state.as_deref(),
            tab: self.tab.as_deref(),
            sort: self.sort.as_deref(),
            page: self.page,
            page_size: self.page_size,
        }
    }
}

async fn user_projects_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Query(query): Query<ProjectsQuery>,
) -> Result<Json<ProjectList>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let list = user_projects(
        pool,
        &username,
        actor.map(|user| user.id),
        query.as_domain_query(),
    )
    .await
    .map_err(map_projects_error)?;
    Ok(Json(list))
}

async fn organization_projects_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(org): Path<String>,
    Query(query): Query<ProjectsQuery>,
) -> Result<Json<ProjectList>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let list = organization_projects(
        pool,
        &org,
        actor.map(|user| user.id),
        query.as_domain_query(),
    )
    .await
    .map_err(map_projects_error)?;
    Ok(Json(list))
}

async fn repository_projects_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ProjectsQuery>,
) -> Result<Json<ProjectList>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let list = repository_projects(
        pool,
        &owner,
        &repo,
        actor.map(|user| user.id),
        query.as_domain_query(),
    )
    .await
    .map_err(map_projects_error)?;
    Ok(Json(list))
}

async fn copy_project_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<CopyProjectRequest>,
) -> Result<(StatusCode, Json<CopiedProject>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let copied = copy_project_for_actor(pool, project_id, actor.id, request)
        .await
        .map_err(map_projects_error)?;
    Ok((StatusCode::CREATED, Json(copied)))
}

async fn project_workspace_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ProjectWorkspaceRouteQuery>,
) -> Result<Json<ProjectWorkspace>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let workspace = project_workspace(
        pool,
        project_id,
        actor.map(|user| user.id),
        query.as_domain_query(),
    )
    .await
    .map_err(map_projects_error)?;
    Ok(Json(workspace))
}

async fn project_insights_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ProjectInsightsRouteQuery>,
) -> Result<Json<ProjectInsights>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let insights = project_insights(
        pool,
        project_id,
        actor.map(|user| user.id),
        query.as_domain_query(),
    )
    .await
    .map_err(map_project_insights_error)?;
    Ok(Json(insights))
}

async fn create_project_insights_chart_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<ProjectInsightsChartMutationRequest>,
) -> Result<(StatusCode, Json<ProjectInsights>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let insights = create_project_insights_chart_for_actor(pool, project_id, actor.id, request)
        .await
        .map_err(map_project_insights_error)?;
    Ok((StatusCode::CREATED, Json(insights)))
}

async fn update_project_insights_chart_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, chart_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectInsightsChartMutationRequest>,
) -> Result<Json<ProjectInsights>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let insights =
        update_project_insights_chart_for_actor(pool, project_id, chart_id, actor.id, request)
            .await
            .map_err(map_project_insights_error)?;
    Ok(Json(insights))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectInsightsChartDeleteRequest {
    expected_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn delete_project_insights_chart_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, chart_id)): Path<(Uuid, Uuid)>,
    body: Option<Json<ProjectInsightsChartDeleteRequest>>,
) -> Result<Json<ProjectInsights>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let insights = delete_project_insights_chart_for_actor(
        pool,
        project_id,
        chart_id,
        actor.id,
        body.and_then(|Json(request)| request.expected_updated_at),
    )
    .await
    .map_err(map_project_insights_error)?;
    Ok(Json(insights))
}

async fn project_item_detail_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ProjectItemDetail>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let detail = project_item_detail(pool, project_id, item_id, actor.map(|user| user.id))
        .await
        .map_err(map_projects_error)?;
    Ok(Json(detail))
}

async fn project_items_archived_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ProjectItemsArchivedRouteQuery>,
) -> Result<
    Json<crate::api_types::ListEnvelope<ProjectArchivedItem>>,
    (StatusCode, Json<ErrorEnvelope>),
> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let archived = project_items_archived(
        pool,
        project_id,
        actor.map(|user| user.id),
        query.as_domain_query(),
    )
    .await
    .map_err(map_projects_error)?;
    Ok(Json(archived))
}

async fn project_field_settings_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectFieldSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let settings = project_field_settings(pool, project_id, actor.map(|user| user.id))
        .await
        .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn project_settings_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let settings = project_settings(pool, project_id, actor.map(|user| user.id))
        .await
        .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn update_project_settings_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<ProjectSettingsUpdateRequest>,
) -> Result<Json<ProjectSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings = update_project_settings_for_actor(pool, project_id, actor.id, request)
        .await
        .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn link_project_repository_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, repository_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectRepositoryLinkRequest>,
) -> Result<Json<ProjectSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings =
        link_project_repository_for_actor(pool, project_id, repository_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn unlink_project_repository_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, repository_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectRepositoryLinkRequest>,
) -> Result<Json<ProjectSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings =
        unlink_project_repository_for_actor(pool, project_id, repository_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn create_project_status_update_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<ProjectStatusUpdateRequest>,
) -> Result<Json<ProjectSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings = create_project_status_update_for_actor(pool, project_id, actor.id, request)
        .await
        .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn update_project_template_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<ProjectTemplateUpdateRequest>,
) -> Result<Json<ProjectSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings = update_project_template_for_actor(pool, project_id, actor.id, request)
        .await
        .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn create_project_access_grant_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<ProjectAccessGrantCreateRequest>,
) -> Result<Json<ProjectSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings = create_project_access_grant_for_actor(pool, project_id, actor.id, request)
        .await
        .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn update_project_access_grant_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, grant_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectAccessGrantUpdateRequest>,
) -> Result<Json<ProjectSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings =
        update_project_access_grant_for_actor(pool, project_id, grant_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn delete_project_access_grant_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, grant_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectAccessGrantDeleteRequest>,
) -> Result<Json<ProjectSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings =
        delete_project_access_grant_for_actor(pool, project_id, grant_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn close_project_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<ProjectLifecycleRequest>,
) -> Result<Json<ProjectSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings = close_project_for_actor(pool, project_id, actor.id, request)
        .await
        .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn reopen_project_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<ProjectLifecycleRequest>,
) -> Result<Json<ProjectSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings = reopen_project_for_actor(pool, project_id, actor.id, request)
        .await
        .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn delete_project_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<ProjectLifecycleRequest>,
) -> Result<Json<ProjectDeleteResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let response = delete_project_for_actor(pool, project_id, actor.id, request)
        .await
        .map_err(map_projects_error)?;
    Ok(Json(response))
}

async fn project_workflow_settings_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectWorkflowSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let settings = project_workflow_settings(pool, project_id, actor.as_ref().map(|user| user.id))
        .await
        .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn update_project_workflow_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, workflow_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectWorkflowUpdateRequest>,
) -> Result<Json<ProjectWorkflowSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings =
        update_project_workflow_for_actor(pool, project_id, workflow_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn invoke_project_automation_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<ProjectAutomationInvocationRequest>,
) -> Result<Json<ProjectAutomationInvocationResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let response = invoke_project_automation_for_actor(pool, project_id, actor.id, request)
        .await
        .map_err(map_projects_error)?;
    Ok(Json(response))
}

async fn create_project_field_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<ProjectFieldCreateRequest>,
) -> Result<(StatusCode, Json<ProjectFieldSettings>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings = create_project_field_for_actor(pool, project_id, actor.id, request)
        .await
        .map_err(map_projects_error)?;
    Ok((StatusCode::CREATED, Json(settings)))
}

async fn update_project_field_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, field_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectFieldUpdateRequest>,
) -> Result<Json<ProjectFieldSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings = update_project_field_for_actor(pool, project_id, field_id, actor.id, request)
        .await
        .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn delete_project_field_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, field_id)): Path<(Uuid, Uuid)>,
    request: Option<Json<ProjectFieldDeleteRequest>>,
) -> Result<Json<ProjectFieldSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings = delete_project_field_for_actor(
        pool,
        project_id,
        field_id,
        actor.id,
        request
            .map(|Json(body)| body)
            .unwrap_or(ProjectFieldDeleteRequest {
                expected_updated_at: None,
            }),
    )
    .await
    .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn create_project_field_option_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, field_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectFieldOptionCreateRequest>,
) -> Result<(StatusCode, Json<ProjectFieldSettings>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings =
        create_project_field_option_for_actor(pool, project_id, field_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok((StatusCode::CREATED, Json(settings)))
}

async fn update_project_field_option_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, field_id, option_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<ProjectFieldOptionUpdateRequest>,
) -> Result<Json<ProjectFieldSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings = update_project_field_option_for_actor(
        pool, project_id, field_id, option_id, actor.id, request,
    )
    .await
    .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn reorder_project_field_options_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, field_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectFieldOptionReorderRequest>,
) -> Result<Json<ProjectFieldSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings =
        reorder_project_field_options_for_actor(pool, project_id, field_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn delete_project_field_option_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, field_id, option_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<ProjectFieldSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings =
        delete_project_field_option_for_actor(pool, project_id, field_id, option_id, actor.id)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn update_project_iteration_settings_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, field_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectIterationSettingsRequest>,
) -> Result<Json<ProjectFieldSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings =
        update_project_iteration_settings_for_actor(pool, project_id, field_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn create_project_iteration_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, field_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectIterationCreateRequest>,
) -> Result<(StatusCode, Json<ProjectFieldSettings>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings =
        create_project_iteration_for_actor(pool, project_id, field_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok((StatusCode::CREATED, Json(settings)))
}

async fn update_project_iteration_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, field_id, iteration_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<ProjectIterationUpdateRequest>,
) -> Result<Json<ProjectFieldSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings = update_project_iteration_for_actor(
        pool,
        project_id,
        field_id,
        iteration_id,
        actor.id,
        request,
    )
    .await
    .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn create_project_iteration_break_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, field_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectIterationBreakCreateRequest>,
) -> Result<(StatusCode, Json<ProjectFieldSettings>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings =
        create_project_iteration_break_for_actor(pool, project_id, field_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok((StatusCode::CREATED, Json(settings)))
}

async fn delete_project_iteration_break_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, field_id, break_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<ProjectFieldSettings>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let settings =
        delete_project_iteration_break_for_actor(pool, project_id, field_id, break_id, actor.id)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(settings))
}

async fn update_project_view_state_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, view_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectViewStateRequest>,
) -> Result<Json<ProjectWorkspace>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let workspace =
        update_project_view_state_for_actor(pool, project_id, view_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(workspace))
}

async fn update_project_view_layout_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, view_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectViewLayoutRequest>,
) -> Result<Json<ProjectWorkspace>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let workspace =
        update_project_view_layout_for_actor(pool, project_id, view_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(workspace))
}

async fn update_project_roadmap_settings_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, view_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectRoadmapSettingsRequest>,
) -> Result<Json<ProjectWorkspace>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let workspace =
        update_project_roadmap_settings_for_actor(pool, project_id, view_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(workspace))
}

async fn update_project_item_field_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, item_id, field_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<ProjectItemFieldValueRequest>,
) -> Result<Json<ProjectWorkspace>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let workspace =
        update_project_item_field_for_actor(pool, project_id, item_id, field_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(workspace))
}

async fn add_project_item_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<ProjectItemAddRequest>,
) -> Result<(StatusCode, Json<ProjectWorkspace>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let workspace = add_project_item_for_actor(pool, project_id, actor.id, request)
        .await
        .map_err(map_projects_error)?;
    Ok((StatusCode::CREATED, Json(workspace)))
}

async fn bulk_add_project_items_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
    Json(request): Json<ProjectItemsBulkAddRequest>,
) -> Result<(StatusCode, Json<ProjectWorkspace>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let workspace = bulk_add_project_items_for_actor(pool, project_id, actor.id, request)
        .await
        .map_err(map_projects_error)?;
    Ok((StatusCode::CREATED, Json(workspace)))
}

async fn update_project_item_position_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, item_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectItemPositionRequest>,
) -> Result<Json<ProjectWorkspace>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let workspace =
        update_project_item_position_for_actor(pool, project_id, item_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(workspace))
}

async fn update_project_draft_item_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, item_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectDraftUpdateRequest>,
) -> Result<Json<ProjectItemDetail>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let detail = update_project_draft_item_for_actor(pool, project_id, item_id, actor.id, request)
        .await
        .map_err(map_projects_error)?;
    Ok(Json(detail))
}

async fn create_project_item_comment_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, item_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectItemCommentCreateRequest>,
) -> Result<Json<ProjectItemDetail>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let detail =
        create_project_item_comment_for_actor(pool, project_id, item_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(detail))
}

async fn update_project_item_comment_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, item_id, comment_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<ProjectItemCommentUpdateRequest>,
) -> Result<Json<ProjectItemDetail>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let detail = update_project_item_comment_for_actor(
        pool, project_id, item_id, comment_id, actor.id, request,
    )
    .await
    .map_err(map_projects_error)?;
    Ok(Json(detail))
}

async fn delete_project_item_comment_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, item_id, comment_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<ProjectItemDetail>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let detail =
        delete_project_item_comment_for_actor(pool, project_id, item_id, comment_id, actor.id)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(detail))
}

async fn project_conversion_targets_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectConversionTargets>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let targets = project_conversion_targets_for_actor(pool, project_id, actor.id)
        .await
        .map_err(map_projects_error)?;
    Ok(Json(targets))
}

async fn convert_project_draft_to_issue_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, item_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ProjectDraftConvertRequest>,
) -> Result<Json<ProjectItemDetail>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let detail =
        convert_project_draft_to_issue_for_actor(pool, project_id, item_id, actor.id, request)
            .await
            .map_err(map_projects_error)?;
    Ok(Json(detail))
}

async fn archive_project_item_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ProjectItemDetail>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let detail = archive_project_item_for_actor(pool, project_id, item_id, actor.id)
        .await
        .map_err(map_projects_error)?;
    Ok(Json(detail))
}

async fn restore_project_item_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ProjectItemDetail>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let detail = restore_project_item_for_actor(pool, project_id, item_id, actor.id)
        .await
        .map_err(map_projects_error)?;
    Ok(Json(detail))
}

async fn remove_project_item_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((project_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ProjectWorkspace>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let workspace = remove_project_item_for_actor(pool, project_id, item_id, actor.id)
        .await
        .map_err(map_projects_error)?;
    Ok(Json(workspace))
}

fn map_projects_error(error: ProjectsError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        ProjectsError::NotFound => error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "Project list was not found",
        ),
        ProjectsError::Forbidden => error_response(
            StatusCode::FORBIDDEN,
            "forbidden",
            "You do not have access to this project list",
        ),
        ProjectsError::InvalidFilter(message) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            message,
        ),
        ProjectsError::Validation(message) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            message,
        ),
        ProjectsError::Sqlx(_) | ProjectsError::Repository(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Project list could not be loaded",
        ),
    }
}

fn map_project_insights_error(error: ProjectsError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        ProjectsError::InvalidFilter(message) => {
            error_response(StatusCode::UNPROCESSABLE_ENTITY, "invalid_filter", message)
        }
        other => map_projects_error(other),
    }
}
