use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub database: ComponentHealth,
}

#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub async fn health(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let database = database_health(&state, "degraded").await;
    let status = if database.status == "ok" {
        "ok"
    } else {
        "degraded"
    };

    (StatusCode::OK, Json(HealthResponse { status, database }))
}

pub async fn ready(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let database = database_health(&state, "unavailable").await;
    let status = if database.status == "ok" {
        "ok"
    } else {
        "unavailable"
    };
    let status_code = if status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(HealthResponse { status, database }))
}

async fn database_health(state: &AppState, unhealthy_status: &'static str) -> ComponentHealth {
    match &state.db {
        Some(pool) => match sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(pool)
            .await
        {
            Ok(_) => ComponentHealth {
                status: "ok",
                message: None,
            },
            Err(error) => ComponentHealth {
                status: unhealthy_status,
                message: Some(error.to_string()),
            },
        },
        None => ComponentHealth {
            status: unhealthy_status,
            message: Some("DATABASE_URL is not configured".to_owned()),
        },
    }
}
