use axum::{extract::State, http::StatusCode, routing::post, Json, Router};

use crate::{
    api_types::{error_response, ErrorEnvelope, RestJson},
    domain::markdown::{
        render_markdown, toggle_task, MarkdownError, RenderMarkdownInput, ToggleTaskInput,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/markdown/render", post(render))
        .route("/api/markdown/task-toggle", post(toggle))
}

async fn render(
    State(state): State<AppState>,
    RestJson(request): RestJson<RenderMarkdownInput>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let rendered = render_markdown(state.db.as_ref(), request)
        .await
        .map_err(map_markdown_error)?;

    Ok(Json(serde_json::json!(rendered)))
}

async fn toggle(
    State(state): State<AppState>,
    RestJson(request): RestJson<ToggleTaskInput>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let output = toggle_task(state.db.as_ref(), request)
        .await
        .map_err(map_markdown_error)?;

    Ok(Json(serde_json::json!(output)))
}

fn map_markdown_error(error: MarkdownError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        MarkdownError::TooLarge => error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            "markdown_too_large",
            error.to_string(),
        ),
        MarkdownError::TaskNotFound => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        MarkdownError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "markdown rendering failed",
        ),
    }
}
