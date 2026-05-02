pub mod api_types;
pub mod auth;
pub mod config;
pub mod db;
pub mod domain;
pub mod jobs;
pub mod middleware;
pub mod routes;

use axum::{middleware as axum_middleware, routing::get, Json, Router};
use config::AppConfig;
use db::DbPool;
use serde_json::json;

#[derive(Clone)]
pub struct AppState {
    pub db: Option<DbPool>,
    pub config: AppConfig,
}

pub fn build_app(db: Option<DbPool>) -> Router {
    build_app_with_config(
        db,
        AppConfig::from_env().unwrap_or_else(|error| {
            tracing::warn!(%error, "starting with local fallback application config");
            AppConfig::local_development()
        }),
    )
}

pub fn build_app_with_config(db: Option<DbPool>, config: AppConfig) -> Router {
    let state = AppState { db, config };

    Router::new()
        .route("/", get(root))
        .route("/health", get(routes::health::health))
        .merge(routes::git::router())
        .merge(routes::auth::router())
        .merge(routes::app_shell::router())
        .merge(routes::users::router())
        .merge(routes::organizations::router())
        .merge(routes::repository_imports::router())
        .nest("/api/repos", routes::repositories::router())
        .merge(routes::issues::router())
        .merge(routes::pulls::router())
        .merge(routes::actions::router())
        .merge(routes::packages::router())
        .merge(routes::search::router())
        .merge(routes::markdown::router())
        .merge(routes::highlight::router())
        .merge(routes::dashboard::router())
        .merge(routes::onboarding::router())
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::request_log::log_request,
        ))
        .with_state(state)
}

async fn root() -> Json<serde_json::Value> {
    Json(json!({ "service": "opengithub-api", "status": "ok" }))
}
