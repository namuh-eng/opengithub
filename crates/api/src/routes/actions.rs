use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    api_types::{
        database_unavailable, error_response, normalize_pagination, ErrorEnvelope, RestJson,
    },
    auth::extractor::AuthenticatedUser,
    domain::{
        actions::{
            actions_dashboard_for_viewer, actions_run_detail_for_viewer,
            actions_workflow_detail_for_viewer, create_workflow, create_workflow_run,
            dispatch_workflow_run, get_workflow_for_actor, get_workflow_run_for_actor,
            list_workflow_runs, list_workflows, record_actions_recent_view,
            repository_for_actor_by_name, repository_for_optional_actor_by_name,
            transition_workflow_run, workflow_artifact_download_for_viewer,
            workflow_job_log_download_for_viewer, workflow_job_logs_for_viewer,
            ActionsDashboardQuery, ActionsWorkflowDetailQuery, AutomationError, CreateWorkflow,
            CreateWorkflowRun, DispatchWorkflowRun, RecordActionsRecentView, RunConclusion,
            RunStatus, TransitionRun,
        },
        permissions::RepositoryRole,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/repos/:owner/:repo/actions/dashboard",
            get(actions_dashboard_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/workflows",
            get(list_workflows_route).post(create_workflow_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/workflows/:workflow_path/dashboard",
            get(actions_workflow_detail_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/workflows/:workflow_path/dispatches",
            post(dispatch_workflow_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/workflows/:workflow_id",
            get(read_workflow_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/workflows/:workflow_id/runs",
            get(list_workflow_runs_route).post(create_workflow_run_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/runs",
            get(list_all_runs_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/runs/:run_id/detail",
            get(read_workflow_run_detail_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/jobs/:job_id/logs",
            get(read_workflow_job_logs_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/jobs/:job_id/logs/download",
            get(download_workflow_job_logs_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/artifacts/:artifact_id/download",
            get(download_workflow_artifact_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/recent-view",
            axum::routing::post(record_recent_view_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/runs/:run_id",
            get(read_workflow_run_route).patch(update_workflow_run_route),
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
struct DashboardQuery {
    q: Option<String>,
    workflow: Option<String>,
    event: Option<String>,
    status: Option<String>,
    branch: Option<String>,
    actor: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogQuery {
    q: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecentViewRequest {
    q: Option<String>,
    workflow: Option<String>,
    event: Option<String>,
    status: Option<String>,
    branch: Option<String>,
    actor: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateWorkflowRequest {
    name: String,
    path: String,
    trigger_events: Option<Vec<String>>,
    dispatch_enabled: Option<bool>,
    dispatch_inputs: Option<Vec<crate::domain::actions::WorkflowDispatchInput>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateWorkflowRunRequest {
    head_branch: String,
    head_sha: Option<String>,
    event: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DispatchWorkflowRequest {
    #[serde(rename = "ref")]
    ref_name: String,
    #[serde(default)]
    inputs: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateWorkflowRunRequest {
    status: RunStatus,
    conclusion: Option<RunConclusion>,
}

async fn actions_dashboard_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<DashboardQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repository = repository_for_optional_actor_by_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_automation_error)?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let dashboard = actions_dashboard_for_viewer(
        pool,
        repository.id,
        actor.as_ref().map(|user| user.id),
        ActionsDashboardQuery {
            q: query.q,
            workflow: query.workflow,
            event: query.event,
            status: query.status,
            branch: query.branch,
            actor: query.actor,
            page: pagination.page,
            page_size: pagination.page_size,
        },
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(dashboard)))
}

async fn actions_workflow_detail_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, workflow_path)): Path<(String, String, String)>,
    Query(query): Query<DashboardQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repository = repository_for_optional_actor_by_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_automation_error)?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let detail = actions_workflow_detail_for_viewer(
        pool,
        repository.id,
        actor.as_ref().map(|user| user.id),
        &workflow_path,
        ActionsWorkflowDetailQuery {
            q: query.q,
            event: query.event,
            status: query.status,
            branch: query.branch,
            actor: query.actor,
            page: pagination.page,
            page_size: pagination.page_size,
        },
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(detail)))
}

async fn list_workflows_route(
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
    let envelope = list_workflows(
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

async fn create_workflow_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<CreateWorkflowRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_automation_error)?;
    if request.name.trim().is_empty() || request.path.trim().is_empty() {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "workflow name and path are required",
        ));
    }
    let workflow = create_workflow(
        pool,
        CreateWorkflow {
            repository_id,
            actor_user_id: actor.0.id,
            name: request.name,
            path: request.path,
            trigger_events: request.trigger_events.unwrap_or_default(),
        },
    )
    .await
    .map_err(map_automation_error)?;
    if request.dispatch_enabled.is_some() || request.dispatch_inputs.is_some() {
        sqlx::query(
            r#"
            UPDATE actions_workflows
            SET dispatch_enabled = COALESCE($2, dispatch_enabled),
                dispatch_inputs = COALESCE($3, dispatch_inputs),
                source_branch = COALESCE(source_branch, (
                    SELECT default_branch FROM repositories WHERE id = actions_workflows.repository_id
                ))
            WHERE id = $1
            "#,
        )
        .bind(workflow.id)
        .bind(request.dispatch_enabled)
        .bind(
            request
                .dispatch_inputs
                .map(|inputs| json!(inputs)),
        )
        .execute(pool)
        .await
        .map_err(AutomationError::Sqlx)
        .map_err(map_automation_error)?;
    }

    Ok((StatusCode::CREATED, Json(json!(workflow))))
}

async fn read_workflow_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, workflow_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_automation_error)?;
    let workflow = get_workflow_for_actor(pool, repository_id, workflow_id, actor.0.id)
        .await
        .map_err(map_automation_error)?;

    Ok(Json(json!(workflow)))
}

async fn list_workflow_runs_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, workflow_id)): Path<(String, String, Uuid)>,
    Query(query): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    list_runs_for_workflow(&state, &headers, owner, repo, Some(workflow_id), query).await
}

async fn list_all_runs_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    list_runs_for_workflow(&state, &headers, owner, repo, None, query).await
}

async fn record_recent_view_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<RecentViewRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_automation_error)?;
    let recent_view = record_actions_recent_view(
        pool,
        RecordActionsRecentView {
            repository_id,
            actor_user_id: actor.0.id,
            workflow: request.workflow,
            q: request.q,
            event: request.event,
            status: request.status,
            branch: request.branch,
            actor: request.actor,
        },
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(recent_view)))
}

async fn list_runs_for_workflow(
    state: &AppState,
    headers: &HeaderMap,
    owner: String,
    repo: String,
    workflow_id: Option<Uuid>,
    query: ListQuery,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(state, headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_automation_error)?;
    if let Some(workflow_id) = workflow_id {
        get_workflow_for_actor(pool, repository_id, workflow_id, actor.0.id)
            .await
            .map_err(map_automation_error)?;
    }
    let pagination = normalize_pagination(query.page, query.page_size);
    let envelope = list_workflow_runs(
        pool,
        repository_id,
        workflow_id,
        actor.0.id,
        pagination.page,
        pagination.page_size,
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(envelope)))
}

async fn create_workflow_run_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, workflow_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<CreateWorkflowRunRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_automation_error)?;
    get_workflow_for_actor(pool, repository_id, workflow_id, actor.0.id)
        .await
        .map_err(map_automation_error)?;
    if request.head_branch.trim().is_empty() || request.event.trim().is_empty() {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "headBranch and event are required",
        ));
    }
    let run = create_workflow_run(
        pool,
        CreateWorkflowRun {
            workflow_id,
            actor_user_id: Some(actor.0.id),
            head_branch: request.head_branch,
            head_sha: request.head_sha,
            event: request.event,
        },
    )
    .await
    .map_err(map_automation_error)?;

    Ok((StatusCode::CREATED, Json(json!(run))))
}

async fn dispatch_workflow_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, workflow_path)): Path<(String, String, String)>,
    RestJson(request): RestJson<DispatchWorkflowRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_automation_error)?;
    let run = dispatch_workflow_run(
        pool,
        DispatchWorkflowRun {
            repository_id,
            workflow_path,
            actor_user_id: actor.0.id,
            ref_name: request.ref_name,
            inputs: request.inputs,
        },
    )
    .await
    .map_err(map_automation_error)?;

    Ok((StatusCode::CREATED, Json(json!(run))))
}

async fn read_workflow_run_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, run_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_automation_error)?;
    let run = get_workflow_run_for_actor(pool, repository_id, run_id, actor.0.id)
        .await
        .map_err(map_automation_error)?;

    Ok(Json(json!(run)))
}

async fn read_workflow_run_detail_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, run_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repository = repository_for_optional_actor_by_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_automation_error)?;
    let detail = actions_run_detail_for_viewer(
        pool,
        repository.id,
        actor.as_ref().map(|user| user.id),
        run_id,
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(detail)))
}

async fn read_workflow_job_logs_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, job_id)): Path<(String, String, Uuid)>,
    Query(query): Query<LogQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repository = repository_for_optional_actor_by_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_automation_error)?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let logs = workflow_job_logs_for_viewer(
        pool,
        repository.id,
        actor.as_ref().map(|user| user.id),
        job_id,
        query.q,
        pagination.page,
        pagination.page_size,
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(logs)))
}

async fn download_workflow_job_logs_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, job_id)): Path<(String, String, Uuid)>,
) -> Result<Response<Body>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repository = repository_for_optional_actor_by_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_automation_error)?;
    let (filename, body) = workflow_job_log_download_for_viewer(
        pool,
        repository.id,
        actor.as_ref().map(|user| user.id),
        job_id,
    )
    .await
    .map_err(map_automation_error)?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(Body::from(body))
        .map_err(|_| database_unavailable())
}

async fn download_workflow_artifact_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, artifact_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repository = repository_for_optional_actor_by_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_automation_error)?;
    let download = workflow_artifact_download_for_viewer(
        pool,
        repository.id,
        actor.as_ref().map(|user| user.id),
        artifact_id,
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(download)))
}

async fn update_workflow_run_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, run_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<UpdateWorkflowRunRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_automation_error)?;
    get_workflow_run_for_actor(pool, repository_id, run_id, actor.0.id)
        .await
        .map_err(map_automation_error)?;
    let run = transition_workflow_run(
        pool,
        run_id,
        TransitionRun {
            status: request.status,
            conclusion: request.conclusion,
        },
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(run)))
}

pub(crate) fn map_automation_error(error: AutomationError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        AutomationError::RepositoryAccessDenied => error_response(
            StatusCode::FORBIDDEN,
            "forbidden",
            "user does not have repository access",
        ),
        AutomationError::RepositoryNotFound
        | AutomationError::WorkflowNotFound
        | AutomationError::WorkflowRunNotFound
        | AutomationError::WorkflowJobNotFound
        | AutomationError::WorkflowArtifactNotFound
        | AutomationError::PackageNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        AutomationError::WorkflowLogsUnavailable | AutomationError::WorkflowArtifactUnavailable => {
            error_response(StatusCode::GONE, "gone", error.to_string())
        }
        AutomationError::InvalidWorkflowState(_)
        | AutomationError::InvalidRunStatus(_)
        | AutomationError::InvalidRunConclusion(_)
        | AutomationError::InvalidPackageType(_)
        | AutomationError::InvalidActionsFilter(_)
        | AutomationError::WorkflowDispatchDisabled(_)
        | AutomationError::InvalidWorkflowDispatch(_) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        AutomationError::JobLease(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "automation operation failed",
        ),
        AutomationError::Sqlx(sqlx::Error::Database(database_error))
            if database_error.is_unique_violation() =>
        {
            error_response(
                StatusCode::CONFLICT,
                "conflict",
                database_error.message().to_owned(),
            )
        }
        AutomationError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "automation operation failed",
        ),
    }
}
