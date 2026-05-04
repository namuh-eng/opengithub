use axum::{
    extract::{RawQuery, State},
    http::{HeaderMap, StatusCode},
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use crate::{
    api_types::{database_unavailable, error_response, error_response_with_details, ErrorEnvelope},
    auth::extractor::AuthenticatedUser,
    domain::{
        dashboard::{
            dashboard_summary, reset_dashboard_feed_preferences, save_dashboard_feed_preferences,
            DashboardError, DashboardFeedEventType, DashboardFeedPreferences, DashboardFeedTab,
            DashboardSummary,
        },
        onboarding::OnboardingError,
        repositories::RepositoryError,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/dashboard", get(read)).route(
        "/api/dashboard/feed-preferences",
        put(update_feed_preferences).delete(reset_feed_preferences),
    )
}

#[derive(Debug, Default)]
struct DashboardQuery {
    page: Option<i64>,
    page_size: Option<i64>,
    repository_filter: Option<String>,
    feed_tab: Option<DashboardFeedTab>,
    event_types: Option<Vec<DashboardFeedEventType>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateFeedPreferencesRequest {
    feed_tab: DashboardFeedTabInput,
    #[serde(default)]
    event_types: Vec<DashboardFeedEventTypeInput>,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct DashboardFeedTabInput(String);

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct DashboardFeedEventTypeInput(String);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResetFeedPreferencesResponse {
    feed_preferences: DashboardFeedPreferences,
}

async fn read(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<DashboardSummary>, (StatusCode, Json<ErrorEnvelope>)> {
    let query = parse_dashboard_query(raw_query.as_deref()).map_err(map_dashboard_error)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let summary = dashboard_summary(
        pool,
        actor.into_auth_user(),
        query.page.unwrap_or(1),
        query.page_size.unwrap_or(10),
        query.repository_filter.as_deref(),
        query.feed_tab,
        query.event_types.as_deref(),
    )
    .await
    .map_err(map_dashboard_error)?;

    Ok(Json(summary))
}

async fn update_feed_preferences(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<UpdateFeedPreferencesRequest>,
) -> Result<Json<DashboardFeedPreferences>, (StatusCode, Json<ErrorEnvelope>)> {
    let feed_tab =
        DashboardFeedTab::try_from(request.feed_tab.0.as_str()).map_err(map_dashboard_error)?;
    let event_types =
        normalize_event_type_inputs(request.event_types).map_err(map_dashboard_error)?;
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let preferences = save_dashboard_feed_preferences(pool, actor.0.id, feed_tab, &event_types)
        .await
        .map_err(map_dashboard_error)?;

    Ok(Json(preferences))
}

async fn reset_feed_preferences(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ResetFeedPreferencesResponse>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let feed_preferences = reset_dashboard_feed_preferences(pool, actor.0.id)
        .await
        .map_err(map_dashboard_error)?;

    Ok(Json(ResetFeedPreferencesResponse { feed_preferences }))
}

fn parse_dashboard_query(raw_query: Option<&str>) -> Result<DashboardQuery, DashboardError> {
    let Some(raw_query) = raw_query else {
        return Ok(DashboardQuery::default());
    };

    let mut query = DashboardQuery::default();
    let mut first_values: HashMap<String, String> = HashMap::new();
    let mut event_type_values = Vec::new();

    for (key, value) in url::form_urlencoded::parse(raw_query.as_bytes()) {
        let key = key.into_owned();
        let value = value.into_owned();
        if key == "eventType" || key == "event_type" {
            event_type_values.extend(
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned),
            );
        } else {
            first_values.entry(key).or_insert(value);
        }
    }

    query.page = first_values
        .get("page")
        .and_then(|value| value.parse::<i64>().ok());
    query.page_size = first_values
        .get("pageSize")
        .or_else(|| first_values.get("page_size"))
        .and_then(|value| value.parse::<i64>().ok());
    query.repository_filter = first_values
        .get("repositoryFilter")
        .or_else(|| first_values.get("repository_filter"))
        .cloned();
    if let Some(feed_tab) = first_values
        .get("feedTab")
        .or_else(|| first_values.get("feed_tab"))
        .map(String::as_str)
    {
        query.feed_tab = Some(DashboardFeedTab::try_from(feed_tab)?);
    }

    let mut parsed_event_types = Vec::new();
    for value in event_type_values {
        let event_type = DashboardFeedEventType::try_from(value.as_str())?;
        if !parsed_event_types.contains(&event_type) {
            parsed_event_types.push(event_type);
        }
    }
    if !parsed_event_types.is_empty() {
        query.event_types = Some(parsed_event_types);
    }

    Ok(query)
}

fn normalize_event_type_inputs(
    inputs: Vec<DashboardFeedEventTypeInput>,
) -> Result<Vec<DashboardFeedEventType>, DashboardError> {
    let mut event_types = Vec::new();
    for input in inputs {
        let event_type = DashboardFeedEventType::try_from(input.0.as_str())?;
        if !event_types.contains(&event_type) {
            event_types.push(event_type);
        }
    }
    Ok(event_types)
}

fn map_dashboard_error(error: DashboardError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        DashboardError::InvalidFeedTab(_) | DashboardError::InvalidFeedEventType(_) => {
            error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation_failed",
                error.to_string(),
            )
        }
        DashboardError::Repositories(RepositoryError::InvalidVisibility(_))
        | DashboardError::Repositories(RepositoryError::InvalidName(_))
        | DashboardError::Repositories(RepositoryError::InvalidDescription(_))
        | DashboardError::Repositories(RepositoryError::InvalidMergeMethod(_))
        | DashboardError::Repositories(RepositoryError::InvalidWatchLevel(_))
        | DashboardError::Repositories(RepositoryError::InvalidWatchEvent(_))
        | DashboardError::Repositories(RepositoryError::InvalidAccessRole(_))
        | DashboardError::Repositories(RepositoryError::InvalidBranchPolicy(_))
        | DashboardError::Repositories(RepositoryError::InvalidBranchDirectoryQuery(_))
        | DashboardError::Repositories(RepositoryError::InvalidPulseQuery(_))
        | DashboardError::Repositories(RepositoryError::InvalidContributorsQuery(_))
        | DashboardError::Repositories(RepositoryError::InvalidForksQuery(_))
        | DashboardError::Repositories(RepositoryError::InvalidDiffContext(_))
        | DashboardError::Repositories(RepositoryError::MergeMethodRequired)
        | DashboardError::Repositories(RepositoryError::DefaultMergeMethodDisabled)
        | DashboardError::Repositories(RepositoryError::ArchivedRepositoryReadOnly)
        | DashboardError::Repositories(RepositoryError::UnknownTemplate(_))
        | DashboardError::Repositories(RepositoryError::UnknownGitignoreTemplate(_))
        | DashboardError::Repositories(RepositoryError::UnknownLicenseTemplate(_))
        | DashboardError::Repositories(RepositoryError::TeamAccessUnsupported)
        | DashboardError::Onboarding(OnboardingError::BlankHintKey) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        DashboardError::Repositories(RepositoryError::OwnerPermissionDenied)
        | DashboardError::Repositories(RepositoryError::PermissionDenied)
        | DashboardError::Repositories(RepositoryError::TrafficAccessDenied) => {
            error_response(StatusCode::FORBIDDEN, "forbidden", error.to_string())
        }
        DashboardError::Repositories(RepositoryError::OrganizationRepositoryCreationPolicy {
            visibility,
            reason,
            settings_href,
        }) => error_response_with_details(
            StatusCode::FORBIDDEN,
            "policy_locked",
            reason.clone(),
            json!({
                "visibility": visibility,
                "reason": reason,
                "settingsHref": settings_href,
            }),
        ),
        DashboardError::Repositories(RepositoryError::OrganizationPolicyLocked {
            field,
            reason,
            settings_href,
        }) => error_response_with_details(
            StatusCode::FORBIDDEN,
            "policy_locked",
            reason.clone(),
            json!({
                "field": field,
                "reason": reason,
                "settingsHref": settings_href,
            }),
        ),
        DashboardError::Repositories(RepositoryError::ForkAlreadyExists) => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        DashboardError::Repositories(RepositoryError::AccessGrantConflict) => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        DashboardError::Repositories(RepositoryError::BranchPolicyConflict) => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        DashboardError::Repositories(RepositoryError::LastAdminAccess) => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        DashboardError::Repositories(RepositoryError::DefaultBranchNotFound(_)) => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        DashboardError::Repositories(RepositoryError::OwnerNotFound)
        | DashboardError::Repositories(RepositoryError::NotFound)
        | DashboardError::Repositories(RepositoryError::AccessTargetNotFound)
        | DashboardError::Repositories(RepositoryError::BranchPolicyNotFound)
        | DashboardError::Repositories(RepositoryError::PathNotFound)
        | DashboardError::Repositories(RepositoryError::RefNotFound)
        | DashboardError::Repositories(RepositoryError::PathNotFoundWithRecovery { .. })
        | DashboardError::Repositories(RepositoryError::RefNotFoundWithRecovery { .. }) => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        DashboardError::Repositories(RepositoryError::GitStorageFailed)
        | DashboardError::Repositories(RepositoryError::Sqlx(_))
        | DashboardError::Onboarding(OnboardingError::Sqlx(_))
        | DashboardError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "dashboard summary operation failed",
        ),
    }
}
