use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, Response, StatusCode},
    routing::{get, post, put},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api_types::{database_unavailable, error_response, ErrorEnvelope, RestJson},
    auth::extractor::AuthenticatedUser,
    domain::gists::{
        create_gist, fork_gist, get_gist, gist_revisions, list_gists, star_gist, unstar_gist,
        update_gist, GistError, GistList, GistListQuery, GistMutation,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/gists", get(list).post(create))
        .route("/api/gists/public", get(list_public))
        .route("/api/gists/:gist_id", get(detail).patch(update))
        .route("/api/gists/:gist_id/revisions", get(revisions))
        .route("/api/gists/:gist_id/star", put(star).delete(unstar))
        .route("/api/gists/:gist_id/forks", post(fork))
        .route("/api/gists/:gist_id/embed.js", get(embed))
        .route("/api/users/:username/gists", get(user_gists))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub scope: Option<String>,
}

async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<GistList>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let list = list_gists(
        pool,
        actor.map(|user| user.id),
        GistListQuery {
            username: None,
            scope: query.scope.as_deref(),
            page: query.page,
            page_size: query.page_size,
        },
        &state.config.app_url,
    )
    .await
    .map_err(map_gist_error)?;
    Ok(Json(list))
}

async fn list_public(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<GistList>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let list = list_gists(
        pool,
        None,
        GistListQuery {
            username: None,
            scope: Some("public"),
            page: query.page,
            page_size: query.page_size,
        },
        &state.config.app_url,
    )
    .await
    .map_err(map_gist_error)?;
    Ok(Json(list))
}

async fn user_gists(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Query(query): Query<ListQuery>,
) -> Result<Json<GistList>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let list = list_gists(
        pool,
        actor.map(|user| user.id),
        GistListQuery {
            username: Some(&username),
            scope: Some("user"),
            page: query.page,
            page_size: query.page_size,
        },
        &state.config.app_url,
    )
    .await
    .map_err(map_gist_error)?;
    Ok(Json(list))
}

async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    RestJson(input): RestJson<GistMutation>,
) -> Result<(StatusCode, Json<crate::domain::gists::GistDetail>), (StatusCode, Json<ErrorEnvelope>)>
{
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let gist = create_gist(pool, actor.id, input, &state.config.app_url)
        .await
        .map_err(map_gist_error)?;
    Ok((StatusCode::CREATED, Json(gist)))
}

async fn detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(gist_id): Path<Uuid>,
) -> Result<Json<crate::domain::gists::GistDetail>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let gist = get_gist(
        pool,
        gist_id,
        actor.map(|user| user.id),
        &state.config.app_url,
    )
    .await
    .map_err(map_gist_error)?;
    Ok(Json(gist))
}

async fn update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(gist_id): Path<Uuid>,
    RestJson(input): RestJson<GistMutation>,
) -> Result<Json<crate::domain::gists::GistDetail>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    let gist = update_gist(pool, actor.id, gist_id, input, &state.config.app_url)
        .await
        .map_err(map_gist_error)?;
    Ok(Json(gist))
}

async fn revisions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(gist_id): Path<Uuid>,
) -> Result<Json<crate::domain::gists::GistRevisionList>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let revisions = gist_revisions(
        pool,
        gist_id,
        actor.map(|user| user.id),
        &state.config.app_url,
    )
    .await
    .map_err(map_gist_error)?;
    Ok(Json(revisions))
}

async fn star(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(gist_id): Path<Uuid>,
) -> Result<Json<crate::domain::gists::GistDetail>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    Ok(Json(
        star_gist(pool, actor.id, gist_id, &state.config.app_url)
            .await
            .map_err(map_gist_error)?,
    ))
}

async fn unstar(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(gist_id): Path<Uuid>,
) -> Result<Json<crate::domain::gists::GistDetail>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    Ok(Json(
        unstar_gist(pool, actor.id, gist_id, &state.config.app_url)
            .await
            .map_err(map_gist_error)?,
    ))
}

async fn fork(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(gist_id): Path<Uuid>,
) -> Result<(StatusCode, Json<crate::domain::gists::GistDetail>), (StatusCode, Json<ErrorEnvelope>)>
{
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?.0;
    Ok((
        StatusCode::CREATED,
        Json(
            fork_gist(pool, actor.id, gist_id, &state.config.app_url)
                .await
                .map_err(map_gist_error)?,
        ),
    ))
}

async fn embed(
    State(state): State<AppState>,
    Path(gist_id): Path<Uuid>,
) -> Result<Response<Body>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let gist = get_gist(pool, gist_id, None, &state.config.app_url)
        .await
        .map_err(map_gist_error)?;
    let mut html = String::from("<div class=\"opengithub-gist\">");
    if let Some(description) = gist.summary.description.as_deref() {
        html.push_str(&format!("<p>{}</p>", html_escape(description)));
    }
    for file in &gist.summary.files {
        html.push_str(&format!(
            "<section><header>{}</header><pre><code>{}</code></pre></section>",
            html_escape(&file.filename),
            html_escape(&file.content)
        ));
    }
    html.push_str("</div>");
    let body = format!(
        "document.currentScript.insertAdjacentHTML('beforebegin', {});",
        serde_json::to_string(&html).unwrap_or_else(|_| "\"\"".to_owned())
    );
    let mut response = Response::new(Body::from(body));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/javascript; charset=utf-8"),
    );
    Ok(response)
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn map_gist_error(error: GistError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        GistError::NotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        GistError::Forbidden => {
            error_response(StatusCode::FORBIDDEN, "forbidden", error.to_string())
        }
        GistError::Validation(message) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            message,
        ),
        GistError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "gist_error",
            "gist data could not be loaded",
        ),
    }
}
