use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    api_types::{
        database_unavailable, error_response, error_response_with_details, normalize_pagination,
        ErrorEnvelope, RestJson,
    },
    auth::extractor::AuthenticatedUser,
    domain::actions_secrets::{
        create_repository_actions_secret_by_owner_name,
        create_repository_actions_variable_by_owner_name,
        delete_repository_actions_secret_by_owner_name,
        delete_repository_actions_variable_by_owner_name,
        repository_actions_secrets_settings_for_actor_by_owner_name,
        update_repository_actions_secret_by_owner_name,
        update_repository_actions_variable_by_owner_name, ActionsSecretMutation,
        ActionsSecretsError, ActionsVariableMutation,
    },
    domain::discussions::{
        commit_repository_discussion_category_template_by_owner_name,
        create_repository_discussion_by_owner_name,
        create_repository_discussion_category_by_owner_name,
        create_repository_discussion_category_section_by_owner_name,
        create_repository_discussion_comment_by_owner_name,
        create_repository_discussion_reply_by_owner_name,
        delete_repository_discussion_category_by_owner_name,
        delete_repository_discussion_category_section_by_owner_name,
        pin_repository_discussion_by_owner_name,
        preview_repository_discussion_category_template_by_owner_name,
        recategorize_repository_discussion_by_owner_name,
        reorder_repository_discussion_categories_by_owner_name,
        reorder_repository_discussion_category_sections_by_owner_name,
        repository_discussion_category_settings_for_actor_by_owner_name,
        repository_discussion_category_template_for_actor_by_owner_name,
        repository_discussion_creation_for_actor_by_owner_name,
        repository_discussion_detail_for_actor_by_owner_name,
        repository_discussions_for_actor_by_owner_name,
        set_repository_discussion_answer_by_owner_name,
        set_repository_discussion_lock_by_owner_name,
        set_repository_discussion_subscription_by_owner_name,
        set_repository_discussion_vote_by_owner_name,
        toggle_repository_discussion_reaction_by_owner_name,
        unpin_repository_discussion_by_owner_name,
        update_repository_discussion_category_by_owner_name,
        update_repository_discussion_category_section_by_owner_name,
        update_repository_discussion_metadata_by_owner_name,
        update_repository_discussion_pin_by_owner_name,
        update_repository_discussion_state_by_owner_name, CreateDiscussionCategoryRequest,
        CreateDiscussionCategorySectionRequest, CreateDiscussionCommentRequest,
        CreateDiscussionRequest, DeleteDiscussionCategoryRequest, DiscussionAnswerRequest,
        DiscussionCategoryOrderRequest, DiscussionCategoryTemplateCommitRequest,
        DiscussionCategoryTemplatePreviewRequest, DiscussionMetadataRequest,
        DiscussionReactionMutation, DiscussionReactionRequest, DiscussionSectionOrderRequest,
        DiscussionStateRequest, DiscussionSubscriptionRequest, LockDiscussionRequest,
        PinDiscussionRequest, RecategorizeDiscussionRequest, RepositoryDiscussionDetailQuery,
        RepositoryDiscussionsQuery, UpdateDiscussionCategoryRequest,
        UpdateDiscussionCategorySectionRequest, UpdatePinnedDiscussionRequest,
    },
    domain::pages::{
        connect_repository_pages_actions_deployment_by_owner_name,
        recheck_repository_pages_dns_by_owner_name, remove_repository_pages_domain_by_owner_name,
        repository_pages_settings_for_actor_by_owner_name,
        request_repository_pages_deployment_by_owner_name,
        save_repository_pages_domain_by_owner_name, unpublish_repository_pages_by_owner_name,
        update_repository_pages_https_by_owner_name, update_repository_pages_source_by_owner_name,
        PagesActionsDeploymentMutation, PagesDomainMutation, PagesError, PagesHttpsMutation,
        PagesSourceMutation,
    },
    domain::releases::{
        cancel_repository_release_upload_intent_by_owner_name,
        complete_repository_release_upload_intent_by_owner_name,
        create_repository_release_asset_by_owner_name, create_repository_release_by_owner_name,
        create_repository_release_upload_intent_by_owner_name,
        delete_repository_release_asset_by_owner_name, delete_repository_release_by_owner_name,
        generate_repository_release_notes_by_owner_name, publish_repository_release_by_owner_name,
        repository_latest_release_by_owner_name, repository_release_archive_metadata_by_owner_name,
        repository_release_asset_download_by_owner_name,
        repository_release_detail_by_id_by_owner_name,
        repository_release_detail_by_tag_by_owner_name, repository_release_list_by_owner_name,
        repository_release_management_context_by_owner_name, repository_release_tags_by_owner_name,
        toggle_repository_release_reaction_by_owner_name, update_repository_release_by_owner_name,
        GeneratedReleaseNotesRequest, ReleaseAssetMutation, ReleaseMutation,
        ReleaseUploadCancelRequest, ReleaseUploadCompleteRequest, ReleaseUploadIntentRequest,
        ReleasesError,
    },
    domain::repositories::{
        cancel_repository_invitation_by_owner_name, create_repository_branch_rule_by_owner_name,
        create_repository_ruleset_by_owner_name, create_repository_with_bootstrap,
        delete_repository_branch_rule_by_owner_name, delete_repository_ruleset_by_owner_name,
        fork_repository_by_owner_name, grant_repository_team_access_by_owner_name,
        insert_repository_create_feed_event, invite_repository_access_by_owner_name,
        list_repositories_for_user, remove_repository_collaborator_access_by_owner_name,
        remove_repository_team_access_by_owner_name,
        repository_access_settings_for_actor_by_owner_name,
        repository_blame_for_actor_by_owner_name, repository_blob_for_actor_by_owner_name,
        repository_branch_activity_for_actor_by_owner_name,
        repository_branch_settings_for_actor_by_owner_name,
        repository_branches_for_actor_by_owner_name,
        repository_commit_detail_context_for_actor_by_owner_name,
        repository_commit_detail_for_actor_by_owner_name,
        repository_commit_history_for_actor_by_owner_name,
        repository_contributors_for_actor_by_owner_name, repository_creation_options,
        repository_dependencies_for_actor_by_owner_name,
        repository_dependents_for_actor_by_owner_name,
        repository_file_finder_for_actor_by_owner_name, repository_forks_for_actor_by_owner_name,
        repository_name_availability, repository_network_for_actor_by_owner_name,
        repository_overview_for_viewer_by_owner_name,
        repository_path_overview_for_actor_by_owner_name, repository_pulse_for_actor_by_owner_name,
        repository_refs_for_actor_by_owner_name, repository_sbom_export_status,
        repository_settings_for_actor_by_owner_name, repository_traffic_for_actor_by_owner_name,
        repository_watch_settings_by_owner_name, save_repository_fork_defaults_by_owner_name,
        set_repository_star_by_owner_name, set_repository_watch_by_owner_name,
        start_repository_sbom_export, update_repository_branch_rule_by_owner_name,
        update_repository_collaborator_access_by_owner_name,
        update_repository_ruleset_by_owner_name, update_repository_settings_by_owner_name,
        update_repository_team_access_by_owner_name,
        update_repository_watch_settings_by_owner_name, CreateRepository,
        RepositoryAccessInviteRequest, RepositoryAccessRolePatch, RepositoryAccessTeamGrantRequest,
        RepositoryBootstrapRequest, RepositoryBranchRuleMutation, RepositoryBranchesQuery,
        RepositoryCommitDetailContextQuery, RepositoryCommitHistoryQuery,
        RepositoryContributorsQuery, RepositoryDependencyQuery, RepositoryDependentsQuery,
        RepositoryError, RepositoryFileFinderQuery, RepositoryForksQuery, RepositoryOwner,
        RepositoryPathQuery, RepositoryPulseQuery, RepositoryRefsQuery, RepositoryRulesetMutation,
        RepositorySettingsPatch, RepositoryTrafficQuery, RepositoryVisibility,
        RepositoryWatchSettingsPatch,
    },
    domain::repository_security::{
        bulk_update_repository_dependabot_alerts_for_actor_by_owner_name,
        create_or_link_repository_code_scanning_issue_for_actor_by_owner_name,
        create_repository_dependabot_security_update_for_actor_by_owner_name,
        create_repository_security_advisory_for_actor_by_owner_name,
        publish_repository_security_advisory_for_actor_by_owner_name,
        repository_code_scanning_alert_detail_for_actor_by_owner_name,
        repository_code_scanning_alerts_for_actor_by_owner_name,
        repository_dependabot_alert_detail_for_actor_by_owner_name,
        repository_dependabot_alerts_for_actor_by_owner_name,
        repository_secret_scanning_alert_detail_for_actor_by_owner_name,
        repository_secret_scanning_alerts_for_actor_by_owner_name,
        repository_security_advisories_for_actor_by_owner_name,
        repository_security_advisory_detail_for_actor_by_owner_name,
        repository_security_overview_for_actor_by_owner_name,
        repository_security_policy_for_actor_by_owner_name,
        update_repository_code_scanning_alert_for_actor_by_owner_name,
        update_repository_dependabot_alert_for_actor_by_owner_name,
        update_repository_secret_scanning_alert_for_actor_by_owner_name,
        update_repository_security_advisory_for_actor_by_owner_name,
        upload_repository_code_scanning_sarif_for_actor_by_owner_name,
        upsert_repository_security_policy_by_owner_name, CodeScanningAlertMutation,
        CodeScanningAlertsQuery, CodeScanningSarifUpload, DependabotAlertMutation,
        DependabotAlertsQuery, DependabotBulkMutation, RepositorySecurityAdvisoriesQuery,
        RepositorySecurityAdvisoryCreate, RepositorySecurityAdvisoryMutation,
        SecretScanningAlertMutation, SecretScanningAlertsQuery, SecurityPolicyMutation,
    },
    domain::webhooks::{
        create_repository_webhook_by_owner_name, delete_repository_webhook_by_owner_name,
        ping_repository_webhook_by_owner_name, redeliver_repository_webhook_delivery_by_owner_name,
        repository_webhook_delivery_for_actor_by_owner_name,
        repository_webhook_detail_for_actor_by_owner_name,
        repository_webhook_settings_for_actor_by_owner_name,
        update_repository_webhook_by_owner_name, WebhookError, WebhookMutation,
    },
    AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/creation-options", get(creation_options))
        .route("/name-availability", get(name_availability))
        .route("/:owner/:repo/contents/*path", get(contents))
        .route("/:owner/:repo/blobs/*path", get(blob))
        .route("/:owner/:repo/blame/*path", get(blame))
        .route("/:owner/:repo/commits", get(commits))
        .route("/:owner/:repo/commits/:sha", get(commit_detail))
        .route(
            "/:owner/:repo/commits/:sha/context",
            get(commit_detail_context),
        )
        .route("/:owner/:repo/branches", get(branches))
        .route("/:owner/:repo/branches/activity", get(branch_activity))
        .route("/:owner/:repo/pulse", get(pulse))
        .route("/:owner/:repo/graphs/contributors", get(contributors))
        .route("/:owner/:repo/graphs/traffic", get(traffic))
        .route("/:owner/:repo/network/dependencies", get(dependencies))
        .route("/:owner/:repo/network/dependents", get(dependents))
        .route(
            "/:owner/:repo/network/dependencies/sbom",
            post(create_sbom_export),
        )
        .route(
            "/:owner/:repo/network/dependencies/sbom/:export_id",
            get(download_sbom_export),
        )
        .route("/:owner/:repo/network", get(network))
        .route(
            "/:owner/:repo/discussions",
            get(discussions).post(create_discussion),
        )
        .route("/:owner/:repo/discussions/new", get(new_discussion))
        .route(
            "/:owner/:repo/discussions/new/categories/:category_slug",
            get(new_discussion_category),
        )
        .route(
            "/:owner/:repo/discussions/:discussion_number",
            get(discussion_detail),
        )
        .route(
            "/:owner/:repo/discussions/:discussion_number/answer",
            put(mark_discussion_answer).delete(unmark_discussion_answer),
        )
        .route(
            "/:owner/:repo/discussions/:discussion_number/state",
            put(update_discussion_state),
        )
        .route(
            "/:owner/:repo/discussions/:discussion_number/pin",
            put(pin_discussion)
                .patch(update_discussion_pin)
                .delete(unpin_discussion),
        )
        .route(
            "/:owner/:repo/discussions/:discussion_number/lock",
            put(lock_discussion).delete(unlock_discussion),
        )
        .route(
            "/:owner/:repo/discussions/:discussion_number/category",
            patch(recategorize_discussion),
        )
        .route(
            "/:owner/:repo/discussions/:discussion_number/metadata",
            patch(update_discussion_metadata),
        )
        .route(
            "/:owner/:repo/discussions/:discussion_number/comments",
            post(create_discussion_comment),
        )
        .route(
            "/:owner/:repo/discussions/:discussion_number/comments/:comment_id/replies",
            post(create_discussion_reply),
        )
        .route(
            "/:owner/:repo/discussions/:discussion_number/reactions",
            put(react_to_discussion).delete(unreact_to_discussion),
        )
        .route(
            "/:owner/:repo/discussions/:discussion_number/comments/:comment_id/reactions",
            put(react_to_discussion_comment).delete(unreact_to_discussion_comment),
        )
        .route(
            "/:owner/:repo/discussions/:discussion_number/subscription",
            put(subscribe_discussion).delete(unsubscribe_discussion),
        )
        .route(
            "/:owner/:repo/discussions/:discussion_number/vote",
            put(vote_discussion).delete(unvote_discussion),
        )
        .route(
            "/:owner/:repo/discussions/categories/:category_slug",
            get(discussions_category),
        )
        .route("/:owner/:repo/security", get(security_overview))
        .route(
            "/:owner/:repo/security/advisories",
            get(security_advisories).post(create_security_advisory),
        )
        .route(
            "/:owner/:repo/security/advisories/:ghsa_id",
            get(security_advisory_detail).patch(update_security_advisory),
        )
        .route(
            "/:owner/:repo/security/advisories/:ghsa_id/publish",
            post(publish_security_advisory),
        )
        .route(
            "/:owner/:repo/security/code-scanning",
            get(code_scanning_alerts),
        )
        .route(
            "/:owner/:repo/code-scanning/sarifs",
            post(upload_code_scanning_sarif),
        )
        .route(
            "/:owner/:repo/security/code-scanning/:alert_id",
            get(code_scanning_alert_detail).patch(update_code_scanning_alert),
        )
        .route(
            "/:owner/:repo/security/code-scanning/:alert_id/issue",
            post(create_code_scanning_issue),
        )
        .route("/:owner/:repo/security/dependabot", get(dependabot_alerts))
        .route(
            "/:owner/:repo/security/dependabot/bulk",
            post(bulk_update_dependabot_alerts),
        )
        .route(
            "/:owner/:repo/security/dependabot/:alert_id/security-update",
            post(create_dependabot_security_update),
        )
        .route(
            "/:owner/:repo/security/dependabot/:alert_id",
            get(dependabot_alert_detail).patch(update_dependabot_alert),
        )
        .route(
            "/:owner/:repo/security/secret-scanning",
            get(secret_scanning_alerts),
        )
        .route(
            "/:owner/:repo/security/secret-scanning/:alert_id",
            get(secret_scanning_alert_detail).patch(update_secret_scanning_alert),
        )
        .route(
            "/:owner/:repo/security/policy",
            get(security_policy)
                .post(create_security_policy)
                .patch(update_security_policy),
        )
        .route("/:owner/:repo/forks/defaults", put(save_fork_defaults))
        .route("/:owner/:repo/refs", get(refs))
        .route("/:owner/:repo/file-finder", get(file_finder))
        .route("/:owner/:repo/releases", get(releases).post(create_release))
        .route("/:owner/:repo/releases/manage", get(release_manage_new))
        .route(
            "/:owner/:repo/releases/manage/generated-notes",
            post(release_generated_notes),
        )
        .route(
            "/:owner/:repo/releases/manage/upload-intents",
            post(create_release_upload_intent),
        )
        .route(
            "/:owner/:repo/releases/manage/upload-intents/:intent_id/complete",
            post(complete_release_upload_intent),
        )
        .route(
            "/:owner/:repo/releases/manage/upload-intents/:intent_id/cancel",
            post(cancel_release_upload_intent),
        )
        .route(
            "/:owner/:repo/releases/manage/:release_id",
            get(release_manage_edit),
        )
        .route("/:owner/:repo/releases/latest", get(latest_release))
        .route("/:owner/:repo/releases/tags", get(release_tags))
        .route(
            "/:owner/:repo/releases/zipball/*tag",
            get(release_zipball_metadata),
        )
        .route(
            "/:owner/:repo/releases/tarball/*tag",
            get(release_tarball_metadata),
        )
        .route(
            "/:owner/:repo/releases/assets/:asset_id",
            get(release_asset_download),
        )
        .route(
            "/:owner/:repo/releases/:release_id/assets",
            post(create_release_asset),
        )
        .route(
            "/:owner/:repo/releases/:release_id/assets/:asset_id",
            delete(delete_release_asset),
        )
        .route(
            "/:owner/:repo/releases/:release_id/publish",
            post(publish_release),
        )
        .route(
            "/:owner/:repo/releases/:release_id/reactions",
            post(release_reaction),
        )
        .route("/:owner/:repo/releases/tag/*tag", get(release_by_tag))
        .route(
            "/:owner/:repo/releases/:release_id",
            get(release_by_id)
                .patch(update_release)
                .delete(delete_release),
        )
        .route(
            "/:owner/:repo/settings",
            get(settings).patch(update_settings),
        )
        .route(
            "/:owner/:repo/settings/access",
            get(access_settings).post(invite_access),
        )
        .route(
            "/:owner/:repo/settings/access/collaborators/:user_id",
            patch(update_collaborator_access).delete(remove_collaborator_access),
        )
        .route(
            "/:owner/:repo/settings/access/teams",
            post(grant_team_access),
        )
        .route(
            "/:owner/:repo/settings/access/teams/:team_id",
            patch(update_team_access).delete(remove_team_access),
        )
        .route(
            "/:owner/:repo/settings/access/invitations/:invitation_id",
            delete(cancel_invitation),
        )
        .route("/:owner/:repo/settings/branches", get(branch_settings))
        .route(
            "/:owner/:repo/settings/discussions/categories",
            get(discussion_category_settings).post(create_discussion_category),
        )
        .route(
            "/:owner/:repo/settings/discussions/categories/order",
            put(reorder_discussion_categories),
        )
        .route(
            "/:owner/:repo/settings/discussions/categories/:category_id",
            patch(update_discussion_category).delete(delete_discussion_category),
        )
        .route(
            "/:owner/:repo/settings/discussions/categories/:category_id/template",
            get(discussion_category_template).put(commit_discussion_category_template),
        )
        .route(
            "/:owner/:repo/settings/discussions/categories/:category_id/template/preview",
            post(preview_discussion_category_template),
        )
        .route(
            "/:owner/:repo/settings/discussions/sections",
            post(create_discussion_category_section),
        )
        .route(
            "/:owner/:repo/settings/discussions/sections/order",
            put(reorder_discussion_category_sections),
        )
        .route(
            "/:owner/:repo/settings/discussions/sections/:section_id",
            patch(update_discussion_category_section).delete(delete_discussion_category_section),
        )
        .route(
            "/:owner/:repo/settings/branches/rules",
            post(create_branch_rule),
        )
        .route(
            "/:owner/:repo/settings/branches/rules/:rule_id",
            patch(update_branch_rule).delete(delete_branch_rule),
        )
        .route(
            "/:owner/:repo/settings/branches/rulesets",
            post(create_ruleset),
        )
        .route(
            "/:owner/:repo/settings/branches/rulesets/:ruleset_id",
            patch(update_ruleset).delete(delete_ruleset),
        )
        .route(
            "/:owner/:repo/settings/hooks",
            get(webhook_settings).post(create_webhook),
        )
        .route(
            "/:owner/:repo/settings/hooks/:hook_id",
            get(webhook_detail)
                .patch(update_webhook)
                .delete(delete_webhook),
        )
        .route(
            "/:owner/:repo/settings/hooks/:hook_id/ping",
            post(ping_webhook),
        )
        .route(
            "/:owner/:repo/settings/hooks/:hook_id/deliveries/:delivery_id",
            get(webhook_delivery_detail),
        )
        .route(
            "/:owner/:repo/settings/hooks/:hook_id/deliveries/:delivery_id/redeliver",
            post(redeliver_webhook_delivery),
        )
        .route(
            "/:owner/:repo/settings/secrets",
            get(actions_secrets_settings),
        )
        .route(
            "/:owner/:repo/settings/secrets/secrets",
            post(create_actions_secret),
        )
        .route(
            "/:owner/:repo/settings/secrets/secrets/:secret_name",
            patch(update_actions_secret).delete(delete_actions_secret),
        )
        .route(
            "/:owner/:repo/settings/secrets/variables",
            post(create_actions_variable),
        )
        .route(
            "/:owner/:repo/settings/secrets/variables/:variable_name",
            patch(update_actions_variable).delete(delete_actions_variable),
        )
        .route("/:owner/:repo/settings/pages", get(pages_settings))
        .route(
            "/:owner/:repo/settings/pages/source",
            patch(update_pages_source),
        )
        .route(
            "/:owner/:repo/settings/pages/domain",
            post(save_pages_domain).delete(remove_pages_domain),
        )
        .route(
            "/:owner/:repo/settings/pages/domain/recheck",
            post(recheck_pages_dns),
        )
        .route(
            "/:owner/:repo/settings/pages/https",
            patch(update_pages_https),
        )
        .route(
            "/:owner/:repo/settings/pages/deployments",
            post(request_pages_deployment),
        )
        .route(
            "/:owner/:repo/settings/pages/actions-deployments",
            post(connect_pages_actions_deployment),
        )
        .route(
            "/:owner/:repo/settings/pages/unpublish",
            post(unpublish_pages),
        )
        .route("/:owner/:repo/star", put(star).delete(unstar))
        .route(
            "/:owner/:repo/watch",
            get(read_watch)
                .patch(update_watch)
                .put(watch)
                .delete(unwatch),
        )
        .route("/:owner/:repo/forks", get(forks).post(fork))
        .route("/:owner/:repo", get(read))
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
struct CreateRepositoryRequest {
    owner_type: OwnerType,
    owner_id: Uuid,
    name: String,
    description: Option<String>,
    visibility: Option<RepositoryVisibility>,
    default_branch: Option<String>,
    initialize_readme: Option<bool>,
    template_slug: Option<String>,
    gitignore_template_slug: Option<String>,
    license_template_slug: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NameAvailabilityQuery {
    owner_type: OwnerType,
    owner_id: Uuid,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentsQuery {
    #[serde(rename = "ref")]
    ref_name: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
    raw: Option<String>,
    download: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommitsQuery {
    #[serde(rename = "ref")]
    ref_name: Option<String>,
    path: Option<String>,
    author: Option<String>,
    until: Option<DateTime<Utc>>,
    before: Option<DateTime<Utc>>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommitDetailContextParams {
    path: String,
    hunk_id: String,
    #[serde(alias = "context_lines")]
    context_lines: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BranchesQuery {
    tab: Option<String>,
    q: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BranchActivityQuery {
    branch: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PulseQuery {
    period: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContributorsQuery {
    period: Option<String>,
    start: Option<String>,
    end: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ForksQuery {
    period: Option<String>,
    #[serde(alias = "type")]
    repository_type: Option<String>,
    sort: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DependenciesQuery {
    q: Option<String>,
    ecosystem: Option<String>,
    relationship: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DependentsQuery {
    package: Option<String>,
    owner: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ForkDefaultsRequest {
    period: Option<String>,
    #[serde(alias = "type")]
    repository_type: Option<String>,
    sort: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RefsQuery {
    q: Option<String>,
    current_path: Option<String>,
    active_ref: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReleasesQuery {
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseReactionRequest {
    content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileFinderQuery {
    #[serde(rename = "ref")]
    ref_name: Option<String>,
    q: Option<String>,
    page: Option<i64>,
    #[serde(alias = "page_size")]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum OwnerType {
    User,
    Organization,
}

async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let envelope =
        list_repositories_for_user(pool, actor.0.id, pagination.page, pagination.page_size)
            .await
            .map_err(map_repository_error)?;

    Ok(Json(json!(envelope)))
}

async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    RestJson(request): RestJson<CreateRepositoryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let owner = match request.owner_type {
        OwnerType::User => RepositoryOwner::User {
            id: request.owner_id,
        },
        OwnerType::Organization => RepositoryOwner::Organization {
            id: request.owner_id,
        },
    };
    let repository = create_repository_with_bootstrap(
        pool,
        CreateRepository {
            owner,
            name: request.name,
            description: request.description,
            visibility: request.visibility.unwrap_or_default(),
            default_branch: request.default_branch,
            created_by_user_id: actor.0.id,
        },
        RepositoryBootstrapRequest {
            initialize_readme: request.initialize_readme.unwrap_or(false),
            template_slug: request.template_slug,
            gitignore_template_slug: request.gitignore_template_slug,
            license_template_slug: request.license_template_slug,
        },
    )
    .await
    .map_err(map_repository_error)?;
    insert_repository_create_feed_event(pool, &repository, actor.0.id)
        .await
        .map_err(map_repository_error)?;
    let mut body = json!(repository);
    body["href"] = json!(format!("/{}/{}", repository.owner_login, repository.name));

    Ok((StatusCode::CREATED, Json(body)))
}

async fn creation_options(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let options = repository_creation_options(pool, actor.0.id)
        .await
        .map_err(map_repository_error)?;

    Ok(Json(json!(options)))
}

async fn name_availability(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<NameAvailabilityQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let owner = match query.owner_type {
        OwnerType::User => RepositoryOwner::User { id: query.owner_id },
        OwnerType::Organization => RepositoryOwner::Organization { id: query.owner_id },
    };
    let availability = repository_name_availability(pool, actor.0.id, owner, &query.name)
        .await
        .map_err(map_repository_error)?;

    Ok(Json(json!(availability)))
}

async fn read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let repository = repository_overview_for_viewer_by_owner_name(
        pool,
        actor.map(|user| user.id),
        &owner,
        &repo,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(repository)))
}

async fn contents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, path)): Path<(String, String, String)>,
    Query(query): Query<ContentsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let overview = repository_path_overview_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryPathQuery {
            ref_name: query.ref_name.as_deref(),
            path: &path,
            page: pagination.page,
            page_size: pagination.page_size,
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(overview)))
}

async fn blob(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, path)): Path<(String, String, String)>,
    Query(query): Query<ContentsQuery>,
) -> Result<Response, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_blob_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        query.ref_name.as_deref(),
        &path,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    let wants_raw = truthy_query(query.raw.as_deref());
    let wants_download = truthy_query(query.download.as_deref());
    if wants_raw || wants_download {
        let mut response = view.file.content.clone().into_response();
        let headers = response.headers_mut();
        let content_type = if wants_download || view.is_binary {
            "application/octet-stream"
        } else {
            view.mime_type.as_str()
        };
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(content_type)
                .unwrap_or_else(|_| HeaderValue::from_static("text/plain; charset=utf-8")),
        );
        if wants_download {
            let filename = safe_download_filename(&view.path_name);
            headers.insert(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
                    .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
            );
        }
        return Ok(response);
    }

    Ok(Json(json!(view)).into_response())
}

async fn blame(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, path)): Path<(String, String, String)>,
    Query(query): Query<ContentsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_blame_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        query.ref_name.as_deref(),
        &path,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn commits(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<CommitsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let envelope = repository_commit_history_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryCommitHistoryQuery {
            ref_name: query.ref_name.as_deref(),
            path: query.path.as_deref(),
            author: query.author.as_deref(),
            until: query.until.or(query.before),
            page: pagination.page,
            page_size: pagination.page_size,
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(envelope)))
}

async fn commit_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, sha)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view =
        repository_commit_detail_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo, &sha)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(view)))
}

async fn commit_detail_context(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, sha)): Path<(String, String, String)>,
    Query(query): Query<CommitDetailContextParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let context = repository_commit_detail_context_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        &sha,
        RepositoryCommitDetailContextQuery {
            path: &query.path,
            hunk_id: &query.hunk_id,
            context_lines: query.context_lines.unwrap_or(30),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(context)))
}

async fn branches(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<BranchesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let view = repository_branches_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryBranchesQuery {
            tab: query.tab.as_deref(),
            query: query.q.as_deref(),
            page: pagination.page,
            page_size: pagination.page_size,
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn branch_activity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<BranchActivityQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_branch_activity_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        &query.branch,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn pulse(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<PulseQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_pulse_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryPulseQuery {
            period: query.period.as_deref(),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn contributors(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ContributorsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_contributors_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryContributorsQuery {
            period: query.period.as_deref(),
            start: query.start.as_deref(),
            end: query.end.as_deref(),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn traffic(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_traffic_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryTrafficQuery,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn dependencies(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<DependenciesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_dependencies_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryDependencyQuery {
            query: query.q.as_deref(),
            ecosystem: query.ecosystem.as_deref(),
            relationship: query.relationship.as_deref(),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn dependents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<DependentsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_dependents_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryDependentsQuery {
            package: query.package.as_deref(),
            owner: query.owner.as_deref(),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn create_sbom_export(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let export = start_repository_sbom_export(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok((StatusCode::CREATED, Json(json!(export))))
}

async fn download_sbom_export(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, export_id)): Path<(String, String, Uuid)>,
) -> Result<Response, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let download = repository_sbom_export_status(pool, actor.0.id, &owner, &repo, export_id)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "SBOM export was not found".to_owned(),
            )
        })?;

    if download.export.status != "ready" {
        return Ok((StatusCode::ACCEPTED, Json(json!(download.export))).into_response());
    }

    let Some(artifact) = download.artifact else {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "SBOM artifact was not found".to_owned(),
        ));
    };
    let body = serde_json::to_vec_pretty(&artifact).map_err(|_| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "sbom_export_failed",
            "SBOM artifact could not be serialized.".to_owned(),
        )
    })?;
    let filename = format!("{owner}-{repo}-sbom.spdx.json");
    let disposition = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
        .map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "sbom_export_failed",
                "SBOM download headers could not be prepared.".to_owned(),
            )
        })?;

    let mut response = body.into_response();
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/spdx+json"),
    );
    response
        .headers_mut()
        .insert(header::CONTENT_DISPOSITION, disposition);
    Ok(response)
}

async fn network(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_network_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(view)))
}

async fn security_overview(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view =
        repository_security_overview_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(view)))
}

async fn security_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_security_policy_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(view)))
}

#[derive(Debug, Deserialize)]
struct DependabotAlertsQueryParams {
    state: Option<String>,
    q: Option<String>,
    package: Option<String>,
    ecosystem: Option<String>,
    manifest: Option<String>,
    scope: Option<String>,
    severity: Option<String>,
    sort: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodeScanningAlertsQueryParams {
    state: Option<String>,
    q: Option<String>,
    severity: Option<String>,
    #[serde(rename = "security_severity")]
    security_severity: Option<String>,
    tool: Option<String>,
    branch: Option<String>,
    #[serde(rename = "ref")]
    ref_name: Option<String>,
    tag: Option<String>,
    application_code: Option<String>,
    sort: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SecretScanningAlertsQueryParams {
    state: Option<String>,
    q: Option<String>,
    provider: Option<String>,
    secret_type: Option<String>,
    validity: Option<String>,
    resolution: Option<String>,
    bypassed: Option<String>,
    team: Option<String>,
    topic: Option<String>,
    sort: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SecurityAdvisoriesQueryParams {
    state: Option<String>,
    q: Option<String>,
    severity: Option<String>,
    sort: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct DiscussionsQueryParams {
    q: Option<String>,
    label: Option<String>,
    state: Option<String>,
    answered: Option<String>,
    locked: Option<String>,
    pinned: Option<String>,
    sort: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct DiscussionDetailQueryParams {
    sort: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct NewDiscussionQueryParams {
    category: Option<String>,
    title: Option<String>,
}

struct DiscussionReactionTarget {
    owner: String,
    repo: String,
    discussion_number: i64,
    comment_id: Option<Uuid>,
}

async fn discussions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<DiscussionsQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    repository_discussions_response(state, headers, owner, repo, None, query).await
}

async fn new_discussion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<NewDiscussionQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    repository_new_discussion_response(
        state,
        headers,
        owner,
        repo,
        query.category.as_deref(),
        query.title.as_deref(),
    )
    .await
}

async fn new_discussion_category(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, category_slug)): Path<(String, String, String)>,
    Query(query): Query<NewDiscussionQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    repository_new_discussion_response(
        state,
        headers,
        owner,
        repo,
        Some(category_slug.as_str()),
        query.title.as_deref(),
    )
    .await
}

async fn repository_new_discussion_response(
    state: AppState,
    headers: HeaderMap,
    owner: String,
    repo: String,
    category_slug: Option<&str>,
    title_query: Option<&str>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_discussion_creation_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        category_slug,
        title_query,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion creation metadata was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(view)))
}

async fn discussions_category(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, category_slug)): Path<(String, String, String)>,
    Query(query): Query<DiscussionsQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    repository_discussions_response(state, headers, owner, repo, Some(category_slug), query).await
}

async fn discussion_category_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = repository_discussion_category_settings_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(settings)))
}

async fn create_discussion_category(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<CreateDiscussionCategoryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = create_repository_discussion_category_by_owner_name(
        pool, actor.0.id, &owner, &repo, request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(settings)))
}

async fn update_discussion_category(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, category_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<UpdateDiscussionCategoryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_discussion_category_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        category_id,
        request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(settings)))
}

async fn create_discussion_category_section(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<CreateDiscussionCategorySectionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = create_repository_discussion_category_section_by_owner_name(
        pool, actor.0.id, &owner, &repo, request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(settings)))
}

async fn update_discussion_category_section(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, section_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<UpdateDiscussionCategorySectionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_discussion_category_section_by_owner_name(
        pool, actor.0.id, &owner, &repo, section_id, request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(settings)))
}

async fn delete_discussion_category_section(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, section_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = delete_repository_discussion_category_section_by_owner_name(
        pool, actor.0.id, &owner, &repo, section_id,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(settings)))
}

async fn reorder_discussion_categories(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<DiscussionCategoryOrderRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = reorder_repository_discussion_categories_by_owner_name(
        pool, actor.0.id, &owner, &repo, request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(settings)))
}

async fn reorder_discussion_category_sections(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<DiscussionSectionOrderRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = reorder_repository_discussion_category_sections_by_owner_name(
        pool, actor.0.id, &owner, &repo, request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(settings)))
}

async fn delete_discussion_category(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, category_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<DeleteDiscussionCategoryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = delete_repository_discussion_category_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        category_id,
        request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(settings)))
}

async fn discussion_category_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, category_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let template = repository_discussion_category_template_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        category_id,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "discussion category was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(template)))
}

async fn preview_discussion_category_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, category_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<DiscussionCategoryTemplatePreviewRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let form = preview_repository_discussion_category_template_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        category_id,
        request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "discussion category was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(form)))
}

async fn commit_discussion_category_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, category_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<DiscussionCategoryTemplateCommitRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let response = commit_repository_discussion_category_template_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        category_id,
        request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "discussion category was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(response)))
}

async fn discussion_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
    Query(query): Query<DiscussionDetailQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail = repository_discussion_detail_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        discussion_number,
        RepositoryDiscussionDetailQuery {
            sort: query.sort.as_deref(),
            page: query.page,
            page_size: query.page_size,
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(detail)))
}

async fn create_discussion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Json(request): Json<CreateDiscussionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let response =
        create_repository_discussion_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository discussions were not found".to_owned(),
                )
            })?;
    Ok(Json(json!(response)))
}

async fn create_discussion_comment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
    Json(request): Json<CreateDiscussionCommentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail = create_repository_discussion_comment_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        discussion_number,
        request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(detail)))
}

async fn mark_discussion_answer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
    Json(request): Json<DiscussionAnswerRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_discussion_answer(
        state,
        headers,
        owner,
        repo,
        discussion_number,
        request,
        true,
    )
    .await
}

async fn unmark_discussion_answer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
    Json(request): Json<DiscussionAnswerRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_discussion_answer(
        state,
        headers,
        owner,
        repo,
        discussion_number,
        request,
        false,
    )
    .await
}

async fn set_discussion_answer(
    state: AppState,
    headers: HeaderMap,
    owner: String,
    repo: String,
    discussion_number: i64,
    request: DiscussionAnswerRequest,
    marked: bool,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail = set_repository_discussion_answer_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        discussion_number,
        request,
        marked,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(detail)))
}

async fn update_discussion_state(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
    Json(request): Json<DiscussionStateRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail = update_repository_discussion_state_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        discussion_number,
        request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(detail)))
}

async fn pin_discussion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
    Json(request): Json<PinDiscussionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail = pin_repository_discussion_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        discussion_number,
        request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(detail)))
}

async fn update_discussion_pin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
    Json(request): Json<UpdatePinnedDiscussionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail = update_repository_discussion_pin_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        discussion_number,
        request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(detail)))
}

async fn unpin_discussion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail = unpin_repository_discussion_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        discussion_number,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(detail)))
}

async fn lock_discussion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
    Json(request): Json<LockDiscussionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail = set_repository_discussion_lock_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        discussion_number,
        true,
        request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(detail)))
}

async fn unlock_discussion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail = set_repository_discussion_lock_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        discussion_number,
        false,
        LockDiscussionRequest {
            allow_reactions: Some(true),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(detail)))
}

async fn recategorize_discussion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
    Json(request): Json<RecategorizeDiscussionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail = recategorize_repository_discussion_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        discussion_number,
        request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(detail)))
}

async fn update_discussion_metadata(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
    Json(request): Json<DiscussionMetadataRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail = update_repository_discussion_metadata_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        discussion_number,
        request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(detail)))
}

async fn create_discussion_reply(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number, comment_id)): Path<(String, String, i64, Uuid)>,
    Json(request): Json<CreateDiscussionCommentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail = create_repository_discussion_reply_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        discussion_number,
        comment_id,
        request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(detail)))
}

async fn react_to_discussion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
    Json(request): Json<DiscussionReactionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_discussion_reaction(
        state,
        headers,
        DiscussionReactionTarget {
            owner,
            repo,
            discussion_number,
            comment_id: None,
        },
        request,
        true,
    )
    .await
}

async fn unreact_to_discussion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
    Json(request): Json<DiscussionReactionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_discussion_reaction(
        state,
        headers,
        DiscussionReactionTarget {
            owner,
            repo,
            discussion_number,
            comment_id: None,
        },
        request,
        false,
    )
    .await
}

async fn react_to_discussion_comment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number, comment_id)): Path<(String, String, i64, Uuid)>,
    Json(request): Json<DiscussionReactionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_discussion_reaction(
        state,
        headers,
        DiscussionReactionTarget {
            owner,
            repo,
            discussion_number,
            comment_id: Some(comment_id),
        },
        request,
        true,
    )
    .await
}

async fn unreact_to_discussion_comment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number, comment_id)): Path<(String, String, i64, Uuid)>,
    Json(request): Json<DiscussionReactionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_discussion_reaction(
        state,
        headers,
        DiscussionReactionTarget {
            owner,
            repo,
            discussion_number,
            comment_id: Some(comment_id),
        },
        request,
        false,
    )
    .await
}

async fn set_discussion_reaction(
    state: AppState,
    headers: HeaderMap,
    target: DiscussionReactionTarget,
    request: DiscussionReactionRequest,
    reacted: bool,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let reactions = toggle_repository_discussion_reaction_by_owner_name(
        pool,
        actor.0.id,
        &target.owner,
        &target.repo,
        target.discussion_number,
        target.comment_id,
        DiscussionReactionMutation {
            content: &request.content,
            reacted,
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(reactions)))
}

async fn subscribe_discussion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_discussion_subscription(state, headers, owner, repo, discussion_number, true).await
}

async fn unsubscribe_discussion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_discussion_subscription(state, headers, owner, repo, discussion_number, false).await
}

async fn set_discussion_subscription(
    state: AppState,
    headers: HeaderMap,
    owner: String,
    repo: String,
    discussion_number: i64,
    subscribed: bool,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let subscription = set_repository_discussion_subscription_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        discussion_number,
        DiscussionSubscriptionRequest { subscribed },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;
    Ok(Json(json!(subscription)))
}

async fn repository_discussions_response(
    state: AppState,
    headers: HeaderMap,
    owner: String,
    repo: String,
    category_slug: Option<String>,
    query: DiscussionsQueryParams,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_discussions_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        category_slug.as_deref(),
        RepositoryDiscussionsQuery {
            q: query.q.as_deref(),
            label: query.label.as_deref(),
            state: query.state.as_deref(),
            answered: query.answered.as_deref(),
            locked: query.locked.as_deref(),
            pinned: query.pinned.as_deref(),
            sort: query.sort.as_deref(),
            page: query.page,
            page_size: query.page_size,
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussions were not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn vote_discussion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_discussion_vote(state, headers, owner, repo, discussion_number, true).await
}

async fn unvote_discussion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, discussion_number)): Path<(String, String, i64)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_discussion_vote(state, headers, owner, repo, discussion_number, false).await
}

async fn set_discussion_vote(
    state: AppState,
    headers: HeaderMap,
    owner: String,
    repo: String,
    discussion_number: i64,
    voted: bool,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let vote = set_repository_discussion_vote_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        discussion_number,
        voted,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository discussion was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(vote)))
}

async fn security_advisories(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<SecurityAdvisoriesQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_security_advisories_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositorySecurityAdvisoriesQuery {
            state: query.state.as_deref(),
            severity: query.severity.as_deref(),
            query: query.q.as_deref(),
            sort: query.sort.as_deref(),
            page: query.page,
            page_size: query.page_size,
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn security_advisory_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, ghsa_id)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_security_advisory_detail_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo, &ghsa_id,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "security advisory was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn create_security_advisory(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<RepositorySecurityAdvisoryCreate>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = create_repository_security_advisory_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo, request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok((StatusCode::CREATED, Json(json!(view))))
}

async fn update_security_advisory(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, ghsa_id)): Path<(String, String, String)>,
    RestJson(request): RestJson<RepositorySecurityAdvisoryMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = update_repository_security_advisory_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo, &ghsa_id, request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "security advisory was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn publish_security_advisory(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, ghsa_id)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = publish_repository_security_advisory_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo, &ghsa_id,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "security advisory was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn code_scanning_alerts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<CodeScanningAlertsQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_code_scanning_alerts_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        CodeScanningAlertsQuery {
            state: query.state.as_deref(),
            query: query.q.as_deref(),
            severity: query.severity.as_deref(),
            security_severity: query.security_severity.as_deref(),
            tool: query.tool.as_deref(),
            branch: query.branch.as_deref(),
            ref_name: query.ref_name.as_deref(),
            tag: query.tag.as_deref(),
            application_code: query.application_code.as_deref(),
            sort: query.sort.as_deref(),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn code_scanning_alert_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, alert_id)): Path<(String, String, i64)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_code_scanning_alert_detail_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo, alert_id,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "code scanning alert was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn secret_scanning_alerts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<SecretScanningAlertsQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_secret_scanning_alerts_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        SecretScanningAlertsQuery {
            state: query.state.as_deref(),
            query: query.q.as_deref(),
            provider: query.provider.as_deref(),
            secret_type: query.secret_type.as_deref(),
            validity: query.validity.as_deref(),
            resolution: query.resolution.as_deref(),
            bypassed: query.bypassed.as_deref(),
            team: query.team.as_deref(),
            topic: query.topic.as_deref(),
            sort: query.sort.as_deref(),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn secret_scanning_alert_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, alert_id)): Path<(String, String, i64)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_secret_scanning_alert_detail_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo, alert_id,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "secret scanning alert was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn update_secret_scanning_alert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, alert_id)): Path<(String, String, i64)>,
    RestJson(request): RestJson<SecretScanningAlertMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = update_repository_secret_scanning_alert_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo, alert_id, request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "secret scanning alert was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn update_code_scanning_alert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, alert_id)): Path<(String, String, i64)>,
    RestJson(request): RestJson<CodeScanningAlertMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = update_repository_code_scanning_alert_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo, alert_id, request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "code scanning alert was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn create_code_scanning_issue(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, alert_id)): Path<(String, String, i64)>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = create_or_link_repository_code_scanning_issue_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo, alert_id,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "code scanning alert was not found".to_owned(),
        )
    })?;

    Ok((StatusCode::CREATED, Json(json!(view))))
}

async fn upload_code_scanning_sarif(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    body: Bytes,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    const MAX_SARIF_UPLOAD_BYTES: usize = 2 * 1024 * 1024;
    if body.len() > MAX_SARIF_UPLOAD_BYTES {
        return Err(error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            "payload_too_large",
            "SARIF upload must be 2 MiB or smaller.".to_owned(),
        ));
    }
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let upload: CodeScanningSarifUpload = serde_json::from_slice(&body).map_err(|_| {
        error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "SARIF upload must be valid JSON with a sarif object.".to_owned(),
        )
    })?;
    let view = upload_repository_code_scanning_sarif_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo, upload, &body,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok((StatusCode::ACCEPTED, Json(json!(view))))
}

async fn dependabot_alerts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<DependabotAlertsQueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_dependabot_alerts_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        DependabotAlertsQuery {
            state: query.state.as_deref(),
            query: query.q.as_deref(),
            package: query.package.as_deref(),
            ecosystem: query.ecosystem.as_deref(),
            manifest: query.manifest.as_deref(),
            scope: query.scope.as_deref(),
            severity: query.severity.as_deref(),
            sort: query.sort.as_deref(),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn dependabot_alert_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, alert_id)): Path<(String, String, i64)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_dependabot_alert_detail_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo, alert_id,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "dependabot alert was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn update_dependabot_alert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, alert_id)): Path<(String, String, i64)>,
    RestJson(request): RestJson<DependabotAlertMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = update_repository_dependabot_alert_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo, alert_id, request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "dependabot alert was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn bulk_update_dependabot_alerts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<DependabotBulkMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let result = bulk_update_repository_dependabot_alerts_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo, request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(result)))
}

async fn create_dependabot_security_update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, alert_id)): Path<(String, String, i64)>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let result = create_repository_dependabot_security_update_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo, alert_id,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "dependabot alert was not found".to_owned(),
        )
    })?;

    Ok((StatusCode::CREATED, Json(json!(result))))
}

async fn create_security_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<SecurityPolicyMutation>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    mutate_security_policy(state, headers, owner, repo, request, StatusCode::CREATED).await
}

async fn update_security_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<SecurityPolicyMutation>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    mutate_security_policy(state, headers, owner, repo, request, StatusCode::OK).await
}

async fn mutate_security_policy(
    state: AppState,
    headers: HeaderMap,
    owner: String,
    repo: String,
    request: SecurityPolicyMutation,
    success_status: StatusCode,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view =
        upsert_repository_security_policy_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok((success_status, Json(json!(view))))
}

async fn forks(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ForksQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = repository_forks_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryForksQuery {
            period: query.period.as_deref(),
            repository_type: query.repository_type.as_deref(),
            sort: query.sort.as_deref(),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn save_fork_defaults(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Json(request): Json<ForkDefaultsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let view = save_repository_fork_defaults_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryForksQuery {
            period: request.period.as_deref(),
            repository_type: request.repository_type.as_deref(),
            sort: request.sort.as_deref(),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(view)))
}

async fn refs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<RefsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let envelope = repository_refs_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryRefsQuery {
            query: query.q.as_deref(),
            current_path: query.current_path.as_deref(),
            active_ref: query.active_ref.as_deref(),
            page: query.page.unwrap_or(1).max(1),
            page_size: query.page_size.unwrap_or(100).clamp(1, 100),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(envelope)))
}

async fn releases(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ReleasesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let envelope = repository_release_list_by_owner_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
        pagination.page,
        pagination.page_size,
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(envelope)))
}

async fn create_release(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<ReleaseMutation>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = create_repository_release_by_owner_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
        request,
    )
    .await
    .map_err(map_releases_error)?;

    Ok((StatusCode::CREATED, Json(json!(release))))
}

async fn release_manage_new(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let context = repository_release_management_context_by_owner_name(
        pool,
        &owner,
        &repo,
        None,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(context)))
}

async fn release_manage_edit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let context = repository_release_management_context_by_owner_name(
        pool,
        &owner,
        &repo,
        Some(release_id),
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(context)))
}

async fn release_generated_notes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<GeneratedReleaseNotesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let preview = generate_repository_release_notes_by_owner_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
        request,
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(preview)))
}

async fn create_release_upload_intent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<ReleaseUploadIntentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let intent = create_repository_release_upload_intent_by_owner_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
        request,
    )
    .await
    .map_err(map_releases_error)?;

    Ok((StatusCode::CREATED, Json(json!(intent))))
}

async fn complete_release_upload_intent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, intent_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<ReleaseUploadCompleteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = complete_repository_release_upload_intent_by_owner_name(
        pool,
        &owner,
        &repo,
        intent_id,
        actor.as_ref().map(|user| user.id),
        request,
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(release)))
}

async fn cancel_release_upload_intent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, intent_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<ReleaseUploadCancelRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let intent = cancel_repository_release_upload_intent_by_owner_name(
        pool,
        &owner,
        &repo,
        intent_id,
        actor.as_ref().map(|user| user.id),
        request,
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(intent)))
}

async fn latest_release(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = repository_latest_release_by_owner_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(release)))
}

async fn update_release(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<ReleaseMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = update_repository_release_by_owner_name(
        pool,
        &owner,
        &repo,
        release_id,
        actor.as_ref().map(|user| user.id),
        request,
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(release)))
}

async fn publish_release(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = publish_repository_release_by_owner_name(
        pool,
        &owner,
        &repo,
        release_id,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(release)))
}

async fn delete_release(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<ReleaseMutation>,
) -> Result<StatusCode, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    delete_repository_release_by_owner_name(
        pool,
        &owner,
        &repo,
        release_id,
        actor.as_ref().map(|user| user.id),
        request.delete_tag.unwrap_or(false),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn release_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = repository_release_detail_by_id_by_owner_name(
        pool,
        &owner,
        &repo,
        release_id,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(release)))
}

async fn release_by_tag(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, tag)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = repository_release_detail_by_tag_by_owner_name(
        pool,
        &owner,
        &repo,
        &tag,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(release)))
}

async fn release_tags(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<ReleasesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let pagination = normalize_pagination(query.page, query.page_size);
    let envelope = repository_release_tags_by_owner_name(
        pool,
        &owner,
        &repo,
        actor.as_ref().map(|user| user.id),
        pagination.page,
        pagination.page_size,
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(envelope)))
}

async fn release_zipball_metadata(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, tag)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    release_archive_metadata(state, headers, owner, repo, tag, "zipball").await
}

async fn release_tarball_metadata(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, tag)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    release_archive_metadata(state, headers, owner, repo, tag, "tarball").await
}

async fn release_archive_metadata(
    state: AppState,
    headers: HeaderMap,
    owner: String,
    repo: String,
    tag: String,
    format: &str,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let metadata = repository_release_archive_metadata_by_owner_name(
        pool,
        &owner,
        &repo,
        &tag,
        format,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(metadata)))
}

async fn release_asset_download(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, asset_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let metadata = repository_release_asset_download_by_owner_name(
        pool,
        &owner,
        &repo,
        asset_id,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(metadata)))
}

async fn create_release_asset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<ReleaseAssetMutation>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = create_repository_release_asset_by_owner_name(
        pool,
        &owner,
        &repo,
        release_id,
        actor.as_ref().map(|user| user.id),
        request,
    )
    .await
    .map_err(map_releases_error)?;

    Ok((StatusCode::CREATED, Json(json!(release))))
}

async fn delete_release_asset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id, asset_id)): Path<(String, String, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let release = delete_repository_release_asset_by_owner_name(
        pool,
        &owner,
        &repo,
        release_id,
        asset_id,
        actor.as_ref().map(|user| user.id),
    )
    .await
    .map_err(map_releases_error)?;

    Ok(Json(json!(release)))
}

async fn release_reaction(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, release_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<ReleaseReactionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let actor = AuthenticatedUser::optional_from_headers(&state, &headers).await?;
    let reactions = toggle_repository_release_reaction_by_owner_name(
        pool,
        &owner,
        &repo,
        release_id,
        actor.as_ref().map(|user| user.id),
        &request.content,
    )
    .await
    .map_err(map_releases_error)?;

    Ok((StatusCode::CREATED, Json(json!(reactions))))
}

async fn file_finder(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<FileFinderQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let envelope = repository_file_finder_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        RepositoryFileFinderQuery {
            ref_name: query.ref_name.as_deref(),
            query: query.q.as_deref(),
            page: query.page.unwrap_or(1).max(1),
            page_size: query.page_size.unwrap_or(20).clamp(1, 100),
        },
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(envelope)))
}

async fn settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = repository_settings_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(settings)))
}

async fn update_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(patch): RestJson<RepositorySettingsPatch>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_settings_by_owner_name(pool, actor.0.id, &owner, &repo, patch)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(settings)))
}

async fn access_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        repository_access_settings_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn invite_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<RepositoryAccessInviteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = invite_repository_access_by_owner_name(pool, actor.0.id, &owner, &repo, request)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(settings)))
}

async fn grant_team_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<RepositoryAccessTeamGrantRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        grant_repository_team_access_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn update_team_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, team_id)): Path<(String, String, Uuid)>,
    RestJson(patch): RestJson<RepositoryAccessRolePatch>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_team_access_by_owner_name(
        pool, actor.0.id, &owner, &repo, team_id, patch,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn remove_team_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, team_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        remove_repository_team_access_by_owner_name(pool, actor.0.id, &owner, &repo, team_id)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn update_collaborator_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, user_id)): Path<(String, String, Uuid)>,
    RestJson(patch): RestJson<RepositoryAccessRolePatch>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_collaborator_access_by_owner_name(
        pool, actor.0.id, &owner, &repo, user_id, patch,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn remove_collaborator_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, user_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = remove_repository_collaborator_access_by_owner_name(
        pool, actor.0.id, &owner, &repo, user_id,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn cancel_invitation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, invitation_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        cancel_repository_invitation_by_owner_name(pool, actor.0.id, &owner, &repo, invitation_id)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn branch_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        repository_branch_settings_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn create_branch_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<RepositoryBranchRuleMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        create_repository_branch_rule_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn update_branch_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, rule_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<RepositoryBranchRuleMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_branch_rule_by_owner_name(
        pool, actor.0.id, &owner, &repo, rule_id, request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn delete_branch_rule(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, rule_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        delete_repository_branch_rule_by_owner_name(pool, actor.0.id, &owner, &repo, rule_id)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn create_ruleset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<RepositoryRulesetMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        create_repository_ruleset_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn update_ruleset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, ruleset_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<RepositoryRulesetMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_ruleset_by_owner_name(
        pool, actor.0.id, &owner, &repo, ruleset_id, request,
    )
    .await
    .map_err(map_repository_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn delete_ruleset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, ruleset_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        delete_repository_ruleset_by_owner_name(pool, actor.0.id, &owner, &repo, ruleset_id)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn webhook_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        repository_webhook_settings_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo)
            .await
            .map_err(map_webhook_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn webhook_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, hook_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail =
        repository_webhook_detail_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo, hook_id)
            .await
            .map_err(map_webhook_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(detail)))
}

async fn webhook_delivery_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, hook_id, delivery_id)): Path<(String, String, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let detail = repository_webhook_delivery_for_actor_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        hook_id,
        delivery_id,
    )
    .await
    .map_err(map_webhook_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(detail)))
}

async fn create_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<WebhookMutation>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let result = create_repository_webhook_by_owner_name(pool, actor.0.id, &owner, &repo, request)
        .await
        .map_err(map_webhook_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok((StatusCode::CREATED, Json(json!(result))))
}

async fn update_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, hook_id)): Path<(String, String, Uuid)>,
    RestJson(request): RestJson<WebhookMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        update_repository_webhook_by_owner_name(pool, actor.0.id, &owner, &repo, hook_id, request)
            .await
            .map_err(map_webhook_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn delete_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, hook_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        delete_repository_webhook_by_owner_name(pool, actor.0.id, &owner, &repo, hook_id)
            .await
            .map_err(map_webhook_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn ping_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, hook_id)): Path<(String, String, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let result = ping_repository_webhook_by_owner_name(pool, actor.0.id, &owner, &repo, hook_id)
        .await
        .map_err(map_webhook_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(result)))
}

async fn redeliver_webhook_delivery(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, hook_id, delivery_id)): Path<(String, String, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let result = redeliver_repository_webhook_delivery_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        hook_id,
        delivery_id,
    )
    .await
    .map_err(map_webhook_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(result)))
}

async fn actions_secrets_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = repository_actions_secrets_settings_for_actor_by_owner_name(
        pool, actor.0.id, &owner, &repo,
    )
    .await
    .map_err(map_actions_secrets_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn create_actions_secret(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<ActionsSecretMutation>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        create_repository_actions_secret_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_actions_secrets_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok((StatusCode::CREATED, Json(json!(settings))))
}

async fn update_actions_secret(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, secret_name)): Path<(String, String, String)>,
    RestJson(request): RestJson<ActionsSecretMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_actions_secret_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        &secret_name,
        request,
    )
    .await
    .map_err(map_actions_secrets_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn delete_actions_secret(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, secret_name)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = delete_repository_actions_secret_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        &secret_name,
    )
    .await
    .map_err(map_actions_secrets_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn create_actions_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<ActionsVariableMutation>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        create_repository_actions_variable_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_actions_secrets_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok((StatusCode::CREATED, Json(json!(settings))))
}

async fn update_actions_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, variable_name)): Path<(String, String, String)>,
    RestJson(request): RestJson<ActionsVariableMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = update_repository_actions_variable_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        &variable_name,
        request,
    )
    .await
    .map_err(map_actions_secrets_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn delete_actions_variable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo, variable_name)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = delete_repository_actions_variable_by_owner_name(
        pool,
        actor.0.id,
        &owner,
        &repo,
        &variable_name,
    )
    .await
    .map_err(map_actions_secrets_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(settings)))
}

async fn pages_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        repository_pages_settings_for_actor_by_owner_name(pool, actor.0.id, &owner, &repo)
            .await
            .map_err(map_pages_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn update_pages_source(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<PagesSourceMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        update_repository_pages_source_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_pages_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn save_pages_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<PagesDomainMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        save_repository_pages_domain_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_pages_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn remove_pages_domain(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = remove_repository_pages_domain_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_pages_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(settings)))
}

async fn recheck_pages_dns(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = recheck_repository_pages_dns_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_pages_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(settings)))
}

async fn update_pages_https(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<PagesHttpsMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        update_repository_pages_https_by_owner_name(pool, actor.0.id, &owner, &repo, request)
            .await
            .map_err(map_pages_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn request_pages_deployment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let result = request_repository_pages_deployment_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_pages_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(result)))
}

async fn connect_pages_actions_deployment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    RestJson(request): RestJson<PagesActionsDeploymentMutation>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let result = connect_repository_pages_actions_deployment_by_owner_name(
        pool, actor.0.id, &owner, &repo, request,
    )
    .await
    .map_err(map_pages_error)?
    .ok_or_else(|| {
        error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "repository was not found".to_owned(),
        )
    })?;

    Ok(Json(json!(result)))
}

async fn unpublish_pages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = unpublish_repository_pages_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_pages_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(settings)))
}

async fn star(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_star(state, headers, owner, repo, true).await
}

async fn unstar(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_star(state, headers, owner, repo, false).await
}

async fn set_star(
    state: AppState,
    headers: HeaderMap,
    owner: String,
    repo: String,
    starred: bool,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let social = set_repository_star_by_owner_name(pool, actor.0.id, &owner, &repo, starred)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(social)))
}

async fn watch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_watch(state, headers, owner, repo, true).await
}

async fn unwatch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    set_watch(state, headers, owner, repo, false).await
}

async fn set_watch(
    state: AppState,
    headers: HeaderMap,
    owner: String,
    repo: String,
    watching: bool,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let social = set_repository_watch_by_owner_name(pool, actor.0.id, &owner, &repo, watching)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(social)))
}

async fn read_watch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings = repository_watch_settings_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok(Json(json!(settings)))
}

async fn update_watch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
    Json(patch): Json<RepositoryWatchSettingsPatch>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let settings =
        update_repository_watch_settings_by_owner_name(pool, actor.0.id, &owner, &repo, patch)
            .await
            .map_err(map_repository_error)?
            .ok_or_else(|| {
                error_response(
                    StatusCode::NOT_FOUND,
                    "not_found",
                    "repository was not found".to_owned(),
                )
            })?;

    Ok(Json(json!(settings)))
}

async fn fork(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorEnvelope>)> {
    let actor = AuthenticatedUser::from_headers(&state, &headers).await?;
    let pool = state.db.as_ref().ok_or_else(database_unavailable)?;
    let fork = fork_repository_by_owner_name(pool, actor.0.id, &owner, &repo)
        .await
        .map_err(map_repository_error)?
        .ok_or_else(|| {
            error_response(
                StatusCode::NOT_FOUND,
                "not_found",
                "repository was not found".to_owned(),
            )
        })?;

    Ok((StatusCode::CREATED, Json(json!(fork))))
}

fn map_repository_error(error: RepositoryError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        RepositoryError::OwnerPermissionDenied | RepositoryError::PermissionDenied => {
            error_response(StatusCode::FORBIDDEN, "forbidden", error.to_string())
        }
        RepositoryError::TrafficAccessDenied => error_response_with_details(
            StatusCode::FORBIDDEN,
            "traffic_access_required",
            "Repository traffic is available to users with push access.".to_owned(),
            json!({
                "requiredPermission": "write",
                "countsVisible": false,
            }),
        ),
        RepositoryError::DependencyGraphUnavailable(reason) => error_response_with_details(
            StatusCode::UNPROCESSABLE_ENTITY,
            "dependency_graph_unavailable",
            "Repository dependency graph is unavailable.".to_owned(),
            json!({ "reason": reason }),
        ),
        RepositoryError::OrganizationRepositoryCreationPolicy {
            visibility,
            reason,
            settings_href,
        } => error_response_with_details(
            StatusCode::FORBIDDEN,
            "policy_locked",
            reason.clone(),
            json!({
                "visibility": visibility,
                "reason": reason,
                "settingsHref": settings_href,
            }),
        ),
        RepositoryError::OrganizationPolicyLocked {
            field,
            reason,
            settings_href,
        } => error_response_with_details(
            StatusCode::FORBIDDEN,
            "policy_locked",
            reason.clone(),
            json!({
                "field": field,
                "reason": reason,
                "settingsHref": settings_href,
            }),
        ),
        RepositoryError::OwnerNotFound
        | RepositoryError::NotFound
        | RepositoryError::PathNotFound
        | RepositoryError::RefNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        RepositoryError::RefNotFoundWithRecovery {
            ref_name,
            recovery_href,
            default_branch_href,
        } => error_response_with_details(
            StatusCode::NOT_FOUND,
            "ref_not_found",
            format!("repository ref `{ref_name}` was not found"),
            json!({
                "refName": ref_name,
                "recoveryHref": recovery_href,
                "defaultBranchHref": default_branch_href,
            }),
        ),
        RepositoryError::PathNotFoundWithRecovery {
            path,
            recovery_href,
            default_branch_href,
        } => error_response_with_details(
            StatusCode::NOT_FOUND,
            "path_not_found",
            format!("repository path `{path}` was not found"),
            json!({
                "path": path,
                "recoveryHref": recovery_href,
                "defaultBranchHref": default_branch_href,
            }),
        ),
        RepositoryError::InvalidVisibility(_)
        | RepositoryError::InvalidName(_)
        | RepositoryError::InvalidDescription(_)
        | RepositoryError::InvalidMergeMethod(_)
        | RepositoryError::InvalidWatchLevel(_)
        | RepositoryError::InvalidWatchEvent(_)
        | RepositoryError::InvalidAccessRole(_)
        | RepositoryError::InvalidBranchPolicy(_)
        | RepositoryError::InvalidSecurityPolicy(_)
        | RepositoryError::InvalidBranchDirectoryQuery(_)
        | RepositoryError::InvalidPulseQuery(_)
        | RepositoryError::InvalidContributorsQuery(_)
        | RepositoryError::InvalidForksQuery(_)
        | RepositoryError::InvalidDependencyGraphQuery(_)
        | RepositoryError::InvalidDiffContext(_)
        | RepositoryError::MergeMethodRequired
        | RepositoryError::DefaultMergeMethodDisabled
        | RepositoryError::ArchivedRepositoryReadOnly
        | RepositoryError::UnknownTemplate(_)
        | RepositoryError::UnknownGitignoreTemplate(_)
        | RepositoryError::UnknownLicenseTemplate(_) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        RepositoryError::DefaultBranchNotFound(_) => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::ForkAlreadyExists => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::AccessTargetNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        RepositoryError::AccessGrantConflict => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::LastAdminAccess => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::BranchPolicyConflict => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::SecurityPolicyConflict => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        RepositoryError::BranchPolicyNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        RepositoryError::TeamAccessUnsupported => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        RepositoryError::GitStorageFailed => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "git_storage_failed",
            "repository git storage failed".to_owned(),
        ),
        RepositoryError::Sqlx(sqlx::Error::Database(database_error))
            if database_error.is_unique_violation() =>
        {
            error_response(
                StatusCode::CONFLICT,
                "conflict",
                "repository already exists for this owner".to_owned(),
            )
        }
        RepositoryError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "repository operation failed".to_owned(),
        ),
    }
}

fn map_webhook_error(error: WebhookError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        WebhookError::RepositoryNotFound
        | WebhookError::WebhookNotFound
        | WebhookError::DeliveryNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        WebhookError::RepositoryAccessDenied => {
            error_response(StatusCode::FORBIDDEN, "forbidden", error.to_string())
        }
        WebhookError::InvalidWebhook(_)
        | WebhookError::InvalidDeliveryStatus(_)
        | WebhookError::DeliveryQueue(_) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        WebhookError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "webhook operation failed".to_owned(),
        ),
    }
}

fn map_actions_secrets_error(error: ActionsSecretsError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        ActionsSecretsError::Repository(error) => map_repository_error(error),
        ActionsSecretsError::Invalid(_) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        ActionsSecretsError::Conflict => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        ActionsSecretsError::NotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        ActionsSecretsError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Actions secrets operation failed".to_owned(),
        ),
    }
}

fn map_pages_error(error: PagesError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        PagesError::Repository(error) => map_repository_error(error),
        PagesError::Invalid(_) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        PagesError::Conflict => error_response(StatusCode::CONFLICT, "conflict", error.to_string()),
        PagesError::NotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        PagesError::PolicyLocked {
            field,
            reason,
            settings_href,
        } => error_response_with_details(
            StatusCode::FORBIDDEN,
            "policy_locked",
            reason.clone(),
            json!({
                "field": field,
                "reason": reason,
                "settingsHref": settings_href,
            }),
        ),
        PagesError::Job(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "job_enqueue_failed",
            "Pages deployment could not be queued".to_owned(),
        ),
        PagesError::Sqlx(sqlx::Error::Database(ref database_error))
            if database_error.is_unique_violation() =>
        {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        PagesError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "Pages settings operation failed".to_owned(),
        ),
    }
}

fn map_releases_error(error: ReleasesError) -> (StatusCode, Json<ErrorEnvelope>) {
    match error {
        ReleasesError::Repository(RepositoryError::PermissionDenied) => error_response(
            StatusCode::FORBIDDEN,
            "forbidden",
            "user does not have repository access".to_owned(),
        ),
        ReleasesError::Repository(error) => map_repository_error(error),
        ReleasesError::NotFound | ReleasesError::TagNotFound => {
            error_response(StatusCode::NOT_FOUND, "not_found", error.to_string())
        }
        ReleasesError::UnsupportedArchiveFormat => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        ReleasesError::UnsupportedReaction => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        ReleasesError::Validation(_) => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            error.to_string(),
        ),
        ReleasesError::Conflict(_) => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        ReleasesError::ArchivedRepository | ReleasesError::ImmutableRelease => {
            error_response(StatusCode::CONFLICT, "conflict", error.to_string())
        }
        ReleasesError::AuthenticationRequired => {
            error_response(StatusCode::UNAUTHORIZED, "unauthorized", error.to_string())
        }
        ReleasesError::Markdown => error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_failed",
            "release notes could not be rendered".to_owned(),
        ),
        ReleasesError::Webhook(_) | ReleasesError::Job(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "release side effect queueing failed".to_owned(),
        ),
        ReleasesError::Sqlx(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "release operation failed".to_owned(),
        ),
    }
}

fn safe_download_filename(path_name: &str) -> String {
    let sanitized = path_name
        .chars()
        .map(|character| match character {
            '"' | '\\' | '/' | '\r' | '\n' | '\t' => '_',
            character if character.is_control() => '_',
            character => character,
        })
        .collect::<String>();
    let trimmed = sanitized.trim_matches('.').trim();
    if trimmed.is_empty() {
        "download".to_owned()
    } else {
        trimmed.chars().take(120).collect()
    }
}

fn truthy_query(value: Option<&str>) -> bool {
    matches!(
        value.map(str::to_ascii_lowercase).as_deref(),
        Some("1" | "true" | "yes" | "on" | "")
    )
}
