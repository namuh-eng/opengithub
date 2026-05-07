use axum::{extract::State, http::StatusCode, routing::post, Json, Router};

use crate::{
    api_types::{error_response, ErrorEnvelope, RestJson},
    domain::highlight::{highlight_code, HighlightCodeInput, HighlightError},
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/highlight/render", post(render))
}

async fn render(
    State(state): State<AppState>,
    RestJson(request): RestJson<HighlightCodeInput>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let highlighted = highlight_code(state.db.as_ref(), request)
        .await
        .map_err(map_highlight_error)?;

    Ok(Json(serde_json::json!(highlighted)))
}

fn map_highlight_error(error: HighlightError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        HighlightError::TooLarge => error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            "source_too_large",
            error.to_string(),
        ),
        HighlightError::EmptySource => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        HighlightError::Sqlx(_) | HighlightError::Json(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "syntax highlighting failed",
        ),
    }
}
