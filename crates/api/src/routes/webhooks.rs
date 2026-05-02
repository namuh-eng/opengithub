use axum::{extract::{Path, State}, http::{HeaderMap, StatusCode}, routing::{get, patch, post}, Json, Router};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{api_types::{database_unavailable, error_response, ErrorEnvelope, RestJson}, auth::extractor::AuthenticatedUser, domain::webhooks::{create_scoped_webhook, dispatch_event, list_webhooks, organization_scope_by_slug, redeliver, repository_scope_by_owner_name, update_webhook, CreateScopedWebhook, DispatchWebhookEvent, UpdateWebhook, WebhookError, WebhookScopeType, SUPPORTED_EVENTS}, AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/repos/:owner/:repo/hooks", get(repo_list).post(repo_create))
        .route("/api/repos/:owner/:repo/hooks/events", post(repo_dispatch))
        .route("/api/repos/:owner/:repo/hooks/:hook_id", patch(repo_update))
        .route("/api/repos/:owner/:repo/hooks/:hook_id/redeliveries", post(repo_redeliver))
        .route("/api/orgs/:org/hooks", get(org_list).post(org_create))
        .route("/api/orgs/:org/hooks/events", post(org_dispatch))
        .route("/api/orgs/:org/hooks/:hook_id", patch(org_update))
        .route("/api/orgs/:org/hooks/:hook_id/redeliveries", post(org_redeliver))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateWebhookRequest { url: String, content_type: Option<String>, secret: Option<String>, events: Vec<String>, active: Option<bool>, ssl_verify: Option<bool> }
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateWebhookRequest { active: Option<bool> }
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RedeliveryRequest { delivery_id: Option<Uuid> }
#[derive(Debug, Deserialize)]
struct DispatchRequest { event: String, payload: serde_json::Value }

async fn repo_list(State(state): State<AppState>, headers: HeaderMap, Path((owner, repo)): Path<(String, String)>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let (actor, pool) = actor_pool(&state, &headers).await?;
    let repo_id = repository_scope_by_owner_name(pool, &owner, &repo).await.map_err(map_webhook_error)?.ok_or_else(not_found)?;
    let hooks = list_webhooks(pool, WebhookScopeType::Repository, repo_id, actor.0.id).await.map_err(map_webhook_error)?;
    Ok(Json(json!({"supportedEvents": SUPPORTED_EVENTS, "hooks": hooks})))
}

async fn repo_create(State(state): State<AppState>, headers: HeaderMap, Path((owner, repo)): Path<(String, String)>, RestJson(input): RestJson<CreateWebhookRequest>) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let (actor, pool) = actor_pool(&state, &headers).await?;
    let repo_id = repository_scope_by_owner_name(pool, &owner, &repo).await.map_err(map_webhook_error)?.ok_or_else(not_found)?;
    let hook = create_scoped_webhook(pool, CreateScopedWebhook { scope_type: WebhookScopeType::Repository, scope_id: repo_id, actor_user_id: actor.0.id, url: input.url, content_type: input.content_type.unwrap_or_else(|| "json".to_owned()), secret: input.secret, events: input.events, active: input.active.unwrap_or(true), ssl_verify: input.ssl_verify.unwrap_or(true) }).await.map_err(map_webhook_error)?;
    Ok((StatusCode::CREATED, Json(json!(hook))))
}

async fn repo_update(State(state): State<AppState>, headers: HeaderMap, Path((_owner, _repo, hook_id)): Path<(String, String, Uuid)>, RestJson(input): RestJson<UpdateWebhookRequest>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    update_any(state, headers, hook_id, input).await
}
async fn repo_redeliver(State(state): State<AppState>, headers: HeaderMap, Path((_owner, _repo, hook_id)): Path<(String, String, Uuid)>, RestJson(input): RestJson<RedeliveryRequest>) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    redeliver_any(state, headers, hook_id, input).await
}
async fn repo_dispatch(State(state): State<AppState>, headers: HeaderMap, Path((owner, repo)): Path<(String, String)>, RestJson(input): RestJson<DispatchRequest>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let (_actor, pool) = actor_pool(&state, &headers).await?;
    let repo_id = repository_scope_by_owner_name(pool, &owner, &repo).await.map_err(map_webhook_error)?.ok_or_else(not_found)?;
    let deliveries = dispatch_event(pool, DispatchWebhookEvent { scope_type: WebhookScopeType::Repository, scope_id: repo_id, event: input.event, payload: input.payload }).await.map_err(map_webhook_error)?;
    Ok(Json(json!({"deliveries": deliveries})))
}

async fn org_list(State(state): State<AppState>, headers: HeaderMap, Path(org): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let (actor, pool) = actor_pool(&state, &headers).await?;
    let org_id = organization_scope_by_slug(pool, &org).await.map_err(map_webhook_error)?.ok_or_else(not_found)?;
    let hooks = list_webhooks(pool, WebhookScopeType::Organization, org_id, actor.0.id).await.map_err(map_webhook_error)?;
    Ok(Json(json!({"supportedEvents": SUPPORTED_EVENTS, "hooks": hooks})))
}
async fn org_create(State(state): State<AppState>, headers: HeaderMap, Path(org): Path<String>, RestJson(input): RestJson<CreateWebhookRequest>) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let (actor, pool) = actor_pool(&state, &headers).await?;
    let org_id = organization_scope_by_slug(pool, &org).await.map_err(map_webhook_error)?.ok_or_else(not_found)?;
    let hook = create_scoped_webhook(pool, CreateScopedWebhook { scope_type: WebhookScopeType::Organization, scope_id: org_id, actor_user_id: actor.0.id, url: input.url, content_type: input.content_type.unwrap_or_else(|| "json".to_owned()), secret: input.secret, events: input.events, active: input.active.unwrap_or(true), ssl_verify: input.ssl_verify.unwrap_or(true) }).await.map_err(map_webhook_error)?;
    Ok((StatusCode::CREATED, Json(json!(hook))))
}
async fn org_update(State(state): State<AppState>, headers: HeaderMap, Path((_org, hook_id)): Path<(String, Uuid)>, RestJson(input): RestJson<UpdateWebhookRequest>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> { update_any(state, headers, hook_id, input).await }
async fn org_redeliver(State(state): State<AppState>, headers: HeaderMap, Path((_org, hook_id)): Path<(String, Uuid)>, RestJson(input): RestJson<RedeliveryRequest>) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> { redeliver_any(state, headers, hook_id, input).await }
async fn org_dispatch(State(state): State<AppState>, headers: HeaderMap, Path(org): Path<String>, RestJson(input): RestJson<DispatchRequest>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let (_actor, pool) = actor_pool(&state, &headers).await?;
    let org_id = organization_scope_by_slug(pool, &org).await.map_err(map_webhook_error)?.ok_or_else(not_found)?;
    let deliveries = dispatch_event(pool, DispatchWebhookEvent { scope_type: WebhookScopeType::Organization, scope_id: org_id, event: input.event, payload: input.payload }).await.map_err(map_webhook_error)?;
    Ok(Json(json!({"deliveries": deliveries})))
}

async fn update_any(state: AppState, headers: HeaderMap, hook_id: Uuid, input: UpdateWebhookRequest) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let (actor, pool) = actor_pool(&state, &headers).await?;
    let hook = update_webhook(pool, hook_id, actor.0.id, UpdateWebhook { active: input.active }).await.map_err(map_webhook_error)?;
    Ok(Json(json!(hook)))
}
async fn redeliver_any(state: AppState, headers: HeaderMap, hook_id: Uuid, input: RedeliveryRequest) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let (actor, pool) = actor_pool(&state, &headers).await?;
    let delivery = redeliver(pool, hook_id, actor.0.id, input.delivery_id).await.map_err(map_webhook_error)?;
    Ok((StatusCode::ACCEPTED, Json(json!(delivery))))
}
async fn actor_pool<'a>(state: &'a AppState, headers: &HeaderMap) -> Result<(AuthenticatedUser, &'a sqlx::PgPool), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(state, headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    Ok((actor, pool))
}
fn not_found() -> (StatusCode, Json<ErrorEnvelope>) { error_response(StatusCode::NOT_FOUND, "not_found", "webhook scope was not found") }
fn map_webhook_error(error: WebhookError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        WebhookError::AccessDenied => error_response(StatusCode::FORBIDDEN, "forbidden", "webhook settings require owner or admin access"),
        WebhookError::WebhookNotFound => error_response(StatusCode::NOT_FOUND, "not_found", "webhook was not found"),
        WebhookError::InvalidUrl => error_response(StatusCode::UNPROCESSABLE_ENTITY, "validation_failed", "payload URL must start with http:// or https://"),
        WebhookError::UnsupportedEvent(event) => error_response(StatusCode::UNPROCESSABLE_ENTITY, "validation_failed", format!("unsupported webhook event: {event}")),
        WebhookError::InvalidScope(_) | WebhookError::InvalidDeliveryStatus(_) => error_response(StatusCode::UNPROCESSABLE_ENTITY, "validation_failed", "webhook payload is invalid"),
        WebhookError::Sqlx(_) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "webhook operation failed"),
    }
}
