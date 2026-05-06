use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    routing::{delete, get, patch, post},
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
            actions_dashboard_for_viewer, actions_job_log_detail_for_viewer,
            actions_run_detail_for_viewer, actions_runner_settings_for_viewer,
            actions_workflow_detail_for_viewer, append_workflow_job_log_chunk, cancel_workflow_run,
            create_actions_runner, create_workflow, create_workflow_run, delete_workflow_run_logs,
            dispatch_workflow_run, get_workflow_for_actor, get_workflow_run_for_actor,
            list_workflow_runs, list_workflows, record_actions_recent_view,
            record_runner_heartbeat, repository_for_actor_by_name,
            repository_for_optional_actor_by_name, rerun_workflow_run, schedule_queued_action_jobs,
            transition_workflow_run, update_actions_log_preferences_for_viewer,
            update_actions_runner_settings, workflow_artifact_download_for_viewer,
            workflow_job_log_download_for_viewer, workflow_job_log_stream_for_viewer,
            workflow_job_logs_for_viewer, workflow_run_log_archive_for_viewer,
            ActionsDashboardQuery, ActionsJobLogDetailQuery, ActionsWorkflowDetailQuery,
            AppendWorkflowJobLogChunk, AutomationError, CreateActionsRunner, CreateWorkflow,
            CreateWorkflowRun, DispatchWorkflowRun, MutateWorkflowRun, RecordActionsRecentView,
            RerunWorkflowRun, RunConclusion, RunStatus, RunnerHeartbeat, TransitionRun,
            UpdateActionsLogPreferences, UpdateActionsRunnerSettings, WorkflowRunRerunMode,
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
            "/api/repos/:owner/:repo/actions/runs/:run_id/rerun",
            post(rerun_workflow_run_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/runs/:run_id/cancel",
            post(cancel_workflow_run_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/runs/:run_id/logs",
            delete(delete_workflow_run_logs_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/runs/:run_id/logs/archive",
            get(download_workflow_run_log_archive_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/runs/:run_id/jobs/:job_id/detail",
            get(read_workflow_job_log_detail_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/log-preferences",
            patch(update_log_preferences_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/jobs/:job_id/logs",
            get(read_workflow_job_logs_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/jobs/:job_id/logs/stream",
            get(stream_workflow_job_logs_route),
        )
        .route(
            "/api/repos/:owner/:repo/actions/jobs/:job_id/logs/download",
            get(download_workflow_job_logs_route),
        )
        .route(
            "/api/internal/actions/jobs/:job_id/logs/chunks",
            post(append_workflow_job_log_chunk_route),
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
            "/api/repos/:owner/:repo/settings/actions/runners",
            get(actions_runner_settings_route)
                .post(create_actions_runner_route)
                .patch(update_actions_runner_settings_route),
        )
        .route(
            "/api/repos/:owner/:repo/settings/actions/runners/heartbeat",
            post(actions_runner_heartbeat_route),
        )
        .route(
            "/api/repos/:owner/:repo/settings/actions/runners/schedule",
            post(schedule_actions_jobs_route),
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
struct LogStreamQuery {
    after: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JobLogDetailQuery {
    q: Option<String>,
    #[serde(rename = "match", alias = "selected_match")]
    selected_match: Option<i64>,
    timestamps: Option<bool>,
    raw: Option<bool>,
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
struct LogPreferencesRequest {
    show_timestamps: bool,
    raw_logs: bool,
    wrap_lines: bool,
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
struct RerunWorkflowRunRequest {
    #[serde(default = "default_rerun_mode")]
    mode: WorkflowRunRerunMode,
    job_id: Option<Uuid>,
}

fn default_rerun_mode() -> WorkflowRunRerunMode {
    WorkflowRunRerunMode::All
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateWorkflowRunRequest {
    status: RunStatus,
    conclusion: Option<RunConclusion>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateRunnerRequest {
    name: String,
    labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateRunnerSettingsRequest {
    concurrency_limit: i32,
    cancel_in_progress: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppendLogChunkRequest {
    runner_token: Option<String>,
    step_id: Option<Uuid>,
    content: String,
    timestamp: Option<chrono::DateTime<chrono::Utc>>,
    finalize: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunnerHeartbeatRequest {
    runner_id: Uuid,
    status: String,
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

async fn actions_runner_settings_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Admin)
            .await
            .map_err(map_automation_error)?;
    let settings = actions_runner_settings_for_viewer(pool, repository_id, actor.0.id)
        .await
        .map_err(map_automation_error)?;
    Ok(Json(json!(settings)))
}

async fn create_actions_runner_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<CreateRunnerRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Admin)
            .await
            .map_err(map_automation_error)?;
    let settings = create_actions_runner(
        pool,
        repository_id,
        actor.0.id,
        CreateActionsRunner {
            name: request.name,
            labels: request.labels,
        },
    )
    .await
    .map_err(map_automation_error)?;
    Ok(Json(json!(settings)))
}

async fn update_actions_runner_settings_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<UpdateRunnerSettingsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Admin)
            .await
            .map_err(map_automation_error)?;
    let settings = update_actions_runner_settings(
        pool,
        repository_id,
        actor.0.id,
        UpdateActionsRunnerSettings {
            concurrency_limit: request.concurrency_limit,
            cancel_in_progress: request.cancel_in_progress,
        },
    )
    .await
    .map_err(map_automation_error)?;
    Ok(Json(json!(settings)))
}

async fn actions_runner_heartbeat_route(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<RunnerHeartbeatRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository = repository_for_optional_actor_by_name(pool, &owner, &repo, None)
        .await
        .map_err(map_automation_error)?;
    let runner = record_runner_heartbeat(
        pool,
        repository.id,
        RunnerHeartbeat {
            runner_id: request.runner_id,
            status: request.status,
        },
    )
    .await
    .map_err(map_automation_error)?;
    Ok(Json(json!(runner)))
}

async fn schedule_actions_jobs_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Admin)
            .await
            .map_err(map_automation_error)?;
    let result = schedule_queued_action_jobs(pool, repository_id, actor.0.id)
        .await
        .map_err(map_automation_error)?;
    Ok(Json(json!(result)))
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

async fn rerun_workflow_run_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, run_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<RerunWorkflowRunRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_automation_error)?;
    let detail = rerun_workflow_run(
        pool,
        RerunWorkflowRun {
            repository_id,
            run_id,
            actor_user_id: actor.0.id,
            mode: request.mode,
            job_id: request.job_id,
        },
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(detail)))
}

async fn cancel_workflow_run_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, run_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_automation_error)?;
    let detail = cancel_workflow_run(
        pool,
        MutateWorkflowRun {
            repository_id,
            run_id,
            actor_user_id: actor.0.id,
        },
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(detail)))
}

async fn delete_workflow_run_logs_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, run_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Write)
            .await
            .map_err(map_automation_error)?;
    let detail = delete_workflow_run_logs(
        pool,
        MutateWorkflowRun {
            repository_id,
            run_id,
            actor_user_id: actor.0.id,
        },
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(detail)))
}

async fn read_workflow_job_log_detail_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, run_id, job_id)): Path<(String, String, Uuid, Uuid)>,
    Query(query): Query<JobLogDetailQuery>,
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
    let detail = actions_job_log_detail_for_viewer(
        pool,
        repository.id,
        actor.as_ref().map(|user| user.id),
        run_id,
        job_id,
        ActionsJobLogDetailQuery {
            q: query.q,
            selected_match: query.selected_match,
            show_timestamps: query.timestamps,
            raw_logs: query.raw,
            page: pagination.page,
            page_size: pagination.page_size,
        },
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(detail)))
}

async fn update_log_preferences_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<LogPreferencesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let repository_id =
        repository_for_actor_by_name(pool, &owner, &repo, actor.0.id, RepositoryRole::Read)
            .await
            .map_err(map_automation_error)?;
    let options = update_actions_log_preferences_for_viewer(
        pool,
        UpdateActionsLogPreferences {
            repository_id,
            actor_user_id: actor.0.id,
            show_timestamps: request.show_timestamps,
            raw_logs: request.raw_logs,
            wrap_lines: request.wrap_lines,
        },
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(options)))
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

async fn stream_workflow_job_logs_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, job_id)): Path<(String, String, Uuid)>,
    Query(query): Query<LogStreamQuery>,
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
    let stream = workflow_job_log_stream_for_viewer(
        pool,
        repository.id,
        actor.as_ref().map(|user| user.id),
        job_id,
        query.after,
    )
    .await
    .map_err(map_automation_error)?;
    let mut body = String::new();
    for line in stream.lines {
        let payload = json!({
            "lineNumber": line.line_number,
            "timestamp": line.timestamp,
            "content": line.content,
            "anchor": line.anchor,
        });
        body.push_str("event: line\n");
        body.push_str(&format!("id: {}\n", line.line_number));
        body.push_str(&format!("data: {payload}\n\n"));
    }
    let cursor = stream.next_cursor.unwrap_or(0);
    body.push_str("event: cursor\n");
    body.push_str(&format!(
        "data: {}\n\n",
        json!({ "nextCursor": cursor, "finalizedAt": stream.finalized_at })
    ));

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .map_err(|_| database_unavailable())
}

async fn append_workflow_job_log_chunk_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(job_id): Path<Uuid>,
    RestJson(request): RestJson<AppendLogChunkRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let header_token = headers
        .get("x-opengithub-runner-token")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let result = append_workflow_job_log_chunk(
        pool,
        AppendWorkflowJobLogChunk {
            job_id,
            runner_token: request.runner_token.or(header_token).unwrap_or_default(),
            step_id: request.step_id,
            content: request.content,
            timestamp: request.timestamp,
            finalize: request.finalize.unwrap_or(false),
        },
    )
    .await
    .map_err(map_automation_error)?;

    Ok(Json(json!(result)))
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

async fn download_workflow_run_log_archive_route(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, run_id)): Path<(String, String, Uuid)>,
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
    let archive = workflow_run_log_archive_for_viewer(
        pool,
        repository.id,
        actor.as_ref().map(|user| user.id),
        run_id,
    )
    .await
    .map_err(map_automation_error)?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, archive.content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", archive.filename),
        )
        .body(Body::from(archive.body))
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
        AutomationError::WorkflowRunActionUnavailable(_) => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        AutomationError::JobLease(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "automation operation failed",
        ),
        AutomationError::ActionsSecrets(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Actions runtime context could not be resolved",
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
