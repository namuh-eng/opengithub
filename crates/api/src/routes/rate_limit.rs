use axum::{extract::State, http::HeaderMap, routing::get, Json, Router};
use chrono::Utc;
use serde_json::json;

use crate::{
    domain::rate_limits::{rate_limit_status, subject_from_headers},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/rate_limit", get(read))
        .route("/api/rate_limit", get(read))
}

async fn read(State(state): State<AppState>, headers: HeaderMap) -> Json<serde_json::Value> {
    let subject = subject_from_headers(state.db.as_ref(), &state.config, &headers).await;
    let status = match rate_limit_status(state.db.as_ref(), &subject, Utc::now()).await {
        Ok(status) => status,
        Err(error) => {
            tracing::warn!(%error, "failed to read rate limit buckets");
            rate_limit_status(None, &subject, Utc::now())
                .await
                .expect("fallback rate limit status should not query")
        }
    };

    Json(json!(status))
}
