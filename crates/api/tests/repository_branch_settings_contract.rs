use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
        branch_policies::{evaluate_branch_policy, BranchPolicyOperation},
        identity::{upsert_session, upsert_user_by_email, User},
        permissions::RepositoryRole,
        repositories::{
            create_repository, grant_repository_permission, CreateRepository, RepositoryOwner,
            RepositoryVisibility,
        },
    },
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use url::Url;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

async fn database_pool() -> Option<PgPool> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|value| !value.trim().is_empty())?;

    let pool = opengithub_api::db::test_pool_options()
        .connect(&database_url)
        .await
        .ok()?;
    MIGRATOR.run(&pool).await.ok()?;
    Some(pool)
}

#[tokio::test]
async fn repository_branch_settings_cover_rules_rulesets_privacy_and_audit_events() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository branch settings scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("branches{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let reader = create_user(&pool, &format!("{marker}-reader")).await;
    let outside = create_user(&pool, &format!("{marker}-outside")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outside_cookie = cookie_header(&pool, &config, &outside).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-repo"),
            description: Some("Branch policy surface".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(&pool, repo.id, reader.id, RepositoryRole::Read, "direct")
        .await
        .expect("reader grant should persist");
    insert_branch(&pool, repo.id, owner.id, "main").await;
    insert_branch(&pool, repo.id, owner.id, "release/2026").await;

    let uri = format!("/api/repos/{}/{}/settings/branches", owner.email, repo.name);
    let (anonymous_status, _, anonymous_body) =
        send_json(app.clone(), Method::GET, &uri, None, None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (create_status, _, create_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/rules"),
        Some(&owner_cookie),
        Some(json!({
            "pattern": "main",
            "description": "Protect mainline",
            "requiredApprovingReviewCount": 2,
            "requiresUpToDateBranch": true,
            "requiredStatusChecks": ["ci/test", "lint"],
            "requiresConversationResolution": true,
            "requiresSignedCommits": true,
            "restrictsPushes": true,
            "bypassActors": [{ "actorType": "user", "actorId": owner.id, "label": "owner" }]
        })),
    )
    .await;
    assert_eq!(create_status, StatusCode::OK);
    assert_eq!(create_body["defaultBranchSummary"]["protected"], true);
    assert_eq!(create_body["rules"][0]["pattern"], "main");
    assert_eq!(
        create_body["rules"][0]["requirements"]["requiredApprovingReviewCount"],
        2
    );
    assert!(create_body["statusCheckSuggestions"]
        .as_array()
        .expect("suggestions should be present")
        .iter()
        .any(|value| value == "ci/test"));
    let rule_id = create_body["rules"][0]["id"]
        .as_str()
        .expect("rule id should be returned")
        .to_owned();

    let (reader_status, _, reader_body) =
        send_json(app.clone(), Method::GET, &uri, Some(&reader_cookie), None).await;
    assert_eq!(reader_status, StatusCode::OK);
    assert_eq!(reader_body["canEdit"], false);
    assert_eq!(reader_body["rules"][0]["canEdit"], false);
    assert_eq!(reader_body["rules"][0]["enforcement"], "active");

    let (duplicate_status, _, duplicate_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/rules"),
        Some(&owner_cookie),
        Some(json!({ "pattern": "refs/heads/main" })),
    )
    .await;
    assert_eq!(duplicate_status, StatusCode::CONFLICT);
    assert_eq!(duplicate_body["error"]["code"], "conflict");

    let (invalid_status, _, invalid_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/rules"),
        Some(&owner_cookie),
        Some(json!({ "pattern": "release/*", "requiredApprovingReviewCount": -1 })),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    let (ruleset_status, _, ruleset_body) = send_json(
        app.clone(),
        Method::POST,
        &format!("{uri}/rulesets"),
        Some(&owner_cookie),
        Some(json!({
            "name": "release readiness",
            "enforcement": "evaluate",
            "patterns": ["release/*"],
            "requiredStatusChecks": ["deploy/staging"],
            "requiresLinearHistory": true
        })),
    )
    .await;
    assert_eq!(ruleset_status, StatusCode::OK);
    assert_eq!(ruleset_body["rulesets"][0]["enforcement"], "evaluate");
    assert_eq!(
        ruleset_body["rulesets"][0]["matchingBranches"][0],
        "release/2026"
    );
    let ruleset_id = ruleset_body["rulesets"][0]["id"]
        .as_str()
        .expect("ruleset id should be returned")
        .to_owned();

    let (update_ruleset_status, _, update_ruleset_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/rulesets/{ruleset_id}"),
        Some(&owner_cookie),
        Some(json!({
            "name": "release readiness",
            "enforcement": "active",
            "patterns": ["release/*"],
            "requiredStatusChecks": ["deploy/staging", "security/review"],
            "requiresSignedCommits": true,
            "bypassActors": [{ "actorType": "user", "actorId": owner.id, "label": "owner" }]
        })),
    )
    .await;
    assert_eq!(update_ruleset_status, StatusCode::OK);
    assert_eq!(update_ruleset_body["rulesets"][0]["enforcement"], "active");
    assert_eq!(
        update_ruleset_body["rulesets"][0]["requirements"]["requiredStatusChecks"][1],
        "security/review"
    );
    assert_eq!(
        update_ruleset_body["rulesets"][0]["bypassActors"][0]["label"],
        "owner"
    );

    let (update_status, _, update_body) = send_json(
        app.clone(),
        Method::PATCH,
        &format!("{uri}/rules/{rule_id}"),
        Some(&owner_cookie),
        Some(json!({
            "pattern": "main",
            "enforcement": "active",
            "requiredApprovingReviewCount": 1,
            "requiredStatusChecks": ["ci/test"]
        })),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK);
    assert_eq!(
        update_body["rules"][0]["requirements"]["requiredApprovingReviewCount"],
        1
    );

    let private_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-private"),
            description: Some("Private branch policy".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repository should create");
    let private_uri = format!(
        "/api/repos/{}/{}/settings/branches",
        owner.email, private_repo.name
    );
    let (private_status, _, private_body) = send_json(
        app.clone(),
        Method::GET,
        &private_uri,
        Some(&outside_cookie),
        None,
    )
    .await;
    assert_eq!(private_status, StatusCode::FORBIDDEN);
    assert_eq!(private_body["error"]["code"], "forbidden");
    assert!(!private_body.to_string().contains("Private branch policy"));

    let (delete_ruleset_status, _, delete_ruleset_body) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("{uri}/rulesets/{ruleset_id}"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(delete_ruleset_status, StatusCode::OK);
    assert!(delete_ruleset_body["rulesets"]
        .as_array()
        .expect("rulesets should be present")
        .is_empty());

    let (delete_rule_status, _, delete_rule_body) = send_json(
        app.clone(),
        Method::DELETE,
        &format!("{uri}/rules/{rule_id}"),
        Some(&owner_cookie),
        None,
    )
    .await;
    assert_eq!(delete_rule_status, StatusCode::OK);
    assert!(delete_rule_body["rules"]
        .as_array()
        .expect("rules should be present")
        .is_empty());

    let audit_count = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM repository_settings_audit_events WHERE repository_id = $1 AND event_type LIKE 'repository.branch_rule.%' OR repository_id = $1 AND event_type LIKE 'repository.ruleset.%'",
    )
    .bind(repo.id)
    .fetch_one(&pool)
    .await
    .expect("audit events should load");
    assert!(audit_count >= 4);
}

#[tokio::test]
async fn branch_policy_push_enforcement_uses_most_restrictive_matching_source() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping branch policy enforcement scenario; set TEST_DATABASE_URL");
        return;
    };

    let marker = format!("restrictive{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let actor = create_user(&pool, &format!("{marker}-actor")).await;
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{marker}-repo"),
            description: Some("Most restrictive branch policy".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    insert_branch(&pool, repo.id, owner.id, "main").await;

    sqlx::query(
        r#"
        INSERT INTO repository_branch_protection_rules (
            repository_id, pattern, allows_force_pushes, allows_deletions
        )
        VALUES ($1, 'main', true, true)
        "#,
    )
    .bind(repo.id)
    .execute(&pool)
    .await
    .expect("permissive branch rule should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_rulesets (
            repository_id, name, enforcement, patterns, allows_force_pushes, allows_deletions
        )
        VALUES ($1, 'main safety', 'active', ARRAY['main'], false, false)
        "#,
    )
    .bind(repo.id)
    .execute(&pool)
    .await
    .expect("restrictive ruleset should insert");

    let force_summary = evaluate_branch_policy(
        &pool,
        repo.id,
        "main",
        Some(actor.id),
        BranchPolicyOperation::Push {
            force: true,
            deletion: false,
            creation: false,
        },
    )
    .await
    .expect("force push policy should evaluate");
    assert!(force_summary.protected);
    assert_eq!(force_summary.active_rule_count, 1);
    assert_eq!(force_summary.active_ruleset_count, 1);
    assert!(!force_summary.allows_force_pushes);
    assert!(force_summary
        .blocking_reasons
        .iter()
        .any(|reason| reason == "force pushes are blocked by branch protection"));

    let deletion_summary = evaluate_branch_policy(
        &pool,
        repo.id,
        "main",
        Some(actor.id),
        BranchPolicyOperation::Push {
            force: false,
            deletion: true,
            creation: false,
        },
    )
    .await
    .expect("deletion policy should evaluate");
    assert!(!deletion_summary.allows_deletions);
    assert!(deletion_summary
        .blocking_reasons
        .iter()
        .any(|reason| reason == "branch deletion is blocked by branch protection"));
}

fn app_config() -> AppConfig {
    AppConfig {
        app_url: Url::parse("http://localhost:3015").expect("app URL"),
        api_url: Url::parse("http://localhost:3016").expect("api URL"),
        auth: Some(AuthConfig {
            google_client_id: "google-client-id.apps.googleusercontent.com".to_owned(),
            google_client_secret: "google-client-secret".to_owned(),
            session_secret: "test-session-secret-with-enough-entropy".to_owned(),
        }),
        session_cookie_name: "__Host-session".to_owned(),
        session_cookie_secure: false,
    }
}

async fn create_user(pool: &PgPool, login: &str) -> User {
    let user = upsert_user_by_email(
        pool,
        &format!("{login}-{}@opengithub.local", Uuid::new_v4()),
        Some(&format!("{login} display")),
        Some("https://images.opengithub.local/avatar.png"),
    )
    .await
    .expect("user should upsert");
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(login)
        .bind(user.id)
        .execute(pool)
        .await
        .expect("username should update");
    user
}

async fn cookie_header(pool: &PgPool, config: &AppConfig, user: &User) -> String {
    let session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(1);
    upsert_session(
        pool,
        &session_id,
        Some(user.id),
        json!({ "provider": "google" }),
        expires_at,
    )
    .await
    .expect("session should persist");
    let set_cookie = session::set_cookie_header(config, &session_id, expires_at)
        .expect("signed cookie should be created");
    let cookie_value =
        session::cookie_value_from_set_cookie(&set_cookie).expect("cookie value should exist");
    format!("{}={cookie_value}", config.session_cookie_name)
}

async fn send_json(
    app: axum::Router,
    method: Method,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request_body = if let Some(value) = body {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
        Body::from(serde_json::to_vec(&value).expect("body should serialize"))
    } else {
        Body::empty()
    };
    let response = app
        .oneshot(builder.body(request_body).expect("request should build"))
        .await
        .expect("request should run");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let value = serde_json::from_slice(&bytes).expect("response should be JSON");
    (status, headers, value)
}

async fn insert_branch(pool: &PgPool, repository_id: Uuid, user_id: Uuid, branch: &str) {
    let commit_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO commits (repository_id, oid, author_user_id, committer_user_id, message)
        VALUES ($1, $2, $3, $3, $4)
        RETURNING id
        "#,
    )
    .bind(repository_id)
    .bind(format!(
        "{}{}",
        branch.replace('/', ""),
        Uuid::new_v4().simple()
    ))
    .bind(user_id)
    .bind(format!("Seed {branch}"))
    .fetch_one(pool)
    .await
    .expect("commit should insert");

    sqlx::query(
        r#"
        INSERT INTO repository_git_refs (repository_id, name, kind, target_commit_id)
        VALUES ($1, $2, 'branch', $3)
        ON CONFLICT (repository_id, name)
        DO UPDATE SET target_commit_id = EXCLUDED.target_commit_id
        "#,
    )
    .bind(repository_id)
    .bind(format!("refs/heads/{branch}"))
    .bind(commit_id)
    .execute(pool)
    .await
    .expect("branch ref should insert");
}
