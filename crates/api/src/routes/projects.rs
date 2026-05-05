use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope},
    auth::extractor::AuthenticatedUser,
    domain::projects::{
        copy_project_for_actor, organization_projects, project_workspace, repository_projects,
        update_project_item_field_for_actor, update_project_view_state_for_actor, user_projects,
        CopiedProject, CopyProjectRequest, ProjectItemFieldValueRequest, ProjectList,
        ProjectListQuery, ProjectViewStateRequest, ProjectWorkspace, ProjectWorkspaceQuery,
        ProjectsError,
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
            "/api/projects/:project_id/views/:view_id/state",
            patch(update_project_view_state_route),
        )
        .route(
            "/api/projects/:project_id/items/:item_id/fields/:field_id",
            patch(update_project_item_field_route),
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
