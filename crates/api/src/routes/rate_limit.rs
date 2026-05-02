use axum::{extract::State, http::HeaderMap, routing::get, Json, Router};

use crate::{
    middleware::rate_limit::{current_rate_limits, identity_from_headers},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/rate_limit", get(rate_limit))
}

async fn rate_limit(State(state): State<AppState>, headers: HeaderMap) -> Json<serde_json::Value> {
    let identity = identity_from_headers(&headers);
    Json(current_rate_limits(state.db.as_ref(), &identity).await)
}
