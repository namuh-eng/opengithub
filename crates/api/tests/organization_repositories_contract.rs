use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
        identity::{upsert_session, upsert_user_by_email, User},
        repositories::{
            can_admin_repository, can_read_repository, can_write_repository, create_organization,
            create_repository, grant_repository_permission, repository_permission_for_user,
            CreateOrganization, CreateRepository, RepositoryOwner, RepositoryVisibility,
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

async fn get_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(Method::GET).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
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
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    let response = app
        .oneshot(
            builder
                .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
                .expect("request should build"),
        )
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

fn assert_json(headers: &HeaderMap) {
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
}

fn repo_names(body: &Value) -> Vec<String> {
    body["items"]
        .as_array()
        .expect("items should be an array")
        .iter()
        .map(|item| {
            item["name"]
                .as_str()
                .expect("name should be string")
                .to_owned()
        })
        .collect()
}

fn option_count(body: &Value, field: &str, value: &str) -> i64 {
    body[field]
        .as_array()
        .expect("filter options should be array")
        .iter()
        .find(|option| option["value"] == value)
        .and_then(|option| option["count"].as_i64())
        .expect("filter option should exist")
}

#[tokio::test]
async fn organization_repository_creation_policy_and_base_permissions_are_enforced() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization repository policy scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orgpolicy{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let member = create_user(&pool, &format!("{marker}-member")).await;
    let direct_admin = create_user(&pool, &format!("{marker}-admin")).await;
    let outsider = create_user(&pool, &format!("{marker}-outsider")).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);

    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Policy Organization".to_owned(),
            description: None,
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    sqlx::query(
        "INSERT INTO organization_memberships (organization_id, user_id, role) VALUES ($1, $2, 'member'), ($1, $3, 'member')",
    )
    .bind(org.id)
    .bind(member.id)
    .bind(direct_admin.id)
    .execute(&pool)
    .await
    .expect("members should insert");
    sqlx::query(
        r#"
        INSERT INTO organization_policy_settings (
            organization_id,
            base_repository_permission,
            members_can_create_public_repositories,
            members_can_create_private_repositories,
            members_can_create_internal_repositories
        )
        VALUES ($1, 'write', true, false, false)
        ON CONFLICT (organization_id) DO UPDATE
        SET base_repository_permission = 'write',
            members_can_create_public_repositories = true,
            members_can_create_private_repositories = false,
            members_can_create_internal_repositories = false
        "#,
    )
    .bind(org.id)
    .execute(&pool)
    .await
    .expect("policy should upsert");

    let (options_status, _, options_body) = get_json(
        app.clone(),
        "/api/repos/creation-options",
        Some(&member_cookie),
    )
    .await;
    assert_eq!(options_status, StatusCode::OK);
    let org_owner = options_body["owners"]
        .as_array()
        .expect("owners should be present")
        .iter()
        .find(|owner| owner["login"] == marker)
        .expect("member organization should be available as an owner");
    assert!(org_owner["visibilityOptions"]
        .as_array()
        .expect("visibility options should be present")
        .iter()
        .any(|option| option["visibility"] == "public" && option["enabled"] == true));
    assert!(org_owner["visibilityOptions"]
        .as_array()
        .expect("visibility options should be present")
        .iter()
        .any(|option| option["visibility"] == "private"
            && option["enabled"] == false
            && option["reason"]
                .as_str()
                .is_some_and(|reason| reason.contains("Organization policy"))));

    let (denied_status, _, denied_body) = send_json(
        app.clone(),
        Method::POST,
        "/api/repos",
        Some(&member_cookie),
        Some(json!({
            "ownerType": "organization",
            "ownerId": org.id,
            "name": format!("{marker}-blocked-private"),
            "visibility": "private",
            "defaultBranch": "main"
        })),
    )
    .await;
    assert_eq!(denied_status, StatusCode::FORBIDDEN);
    assert_eq!(denied_body["error"]["code"], "policy_locked");
    assert_eq!(denied_body["details"]["visibility"], "private");
    assert!(!denied_body.to_string().contains(&owner.email));

    let allowed_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("{marker}-public"),
            description: Some("Base permission contract".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: member.id,
        },
    )
    .await
    .expect("member should create an allowed public repository");

    assert!(can_read_repository(&pool, &allowed_repo, member.id)
        .await
        .expect("read check should work"));
    assert!(can_write_repository(&pool, &allowed_repo, member.id)
        .await
        .expect("write check should work"));
    assert!(!can_admin_repository(&pool, &allowed_repo, member.id)
        .await
        .expect("admin check should work"));
    let member_permission = repository_permission_for_user(&pool, allowed_repo.id, member.id)
        .await
        .expect("permission should load")
        .expect("base permission should be present");
    assert_eq!(member_permission.role.as_str(), "write");
    assert_eq!(member_permission.source, "organization");
    grant_repository_permission(
        &pool,
        allowed_repo.id,
        direct_admin.id,
        opengithub_api::domain::permissions::RepositoryRole::Admin,
        "direct",
    )
    .await
    .expect("direct grant should persist");
    assert!(can_admin_repository(&pool, &allowed_repo, direct_admin.id)
        .await
        .expect("direct admin should win over base permission"));
    assert!(!can_write_repository(&pool, &allowed_repo, outsider.id)
        .await
        .expect("outsider should not inherit base permission"));
}

#[tokio::test]
async fn organization_repositories_filter_sort_and_redact_by_visibility() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization repositories scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orgrepo{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let member = create_user(&pool, &format!("{marker}-member")).await;
    let collaborator = create_user(&pool, &format!("{marker}-collab")).await;
    let outsider = create_user(&pool, &format!("{marker}-outsider")).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let collaborator_cookie = cookie_header(&pool, &config, &collaborator).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Repository Guild".to_owned(),
            description: Some("Repository list contract".to_owned()),
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    sqlx::query(
        "INSERT INTO organization_memberships (organization_id, user_id, role) VALUES ($1, $2, 'member')",
    )
    .bind(org.id)
    .bind(member.id)
    .execute(&pool)
    .await
    .expect("member should insert");

    let alpha = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("{marker}-alpha"),
            description: Some("Rust source with api topic".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("alpha repo should create");
    let beta_private = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("{marker}-private"),
            description: Some("Private TypeScript template".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("trunk".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repo should create");
    let gamma_internal = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("{marker}-internal"),
            description: Some("Internal Go service".to_owned()),
            visibility: RepositoryVisibility::Internal,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("internal repo should create");
    let upstream = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: outsider.id },
            name: format!("{marker}-upstream"),
            description: Some("Upstream source".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: outsider.id,
        },
    )
    .await
    .expect("upstream repo should create");
    let fork = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("{marker}-fork"),
            description: Some("Forked public worker".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("fork repo should create");

    sqlx::query(
        "UPDATE repositories SET license_template_slug = 'mit', updated_at = now() - INTERVAL '1 hour' WHERE id = $1",
    )
    .bind(alpha.id)
    .execute(&pool)
    .await
    .expect("alpha metadata should update");
    sqlx::query(
        "UPDATE repositories SET is_template = true, is_archived = true, created_by_user_id = $2, updated_at = now() - INTERVAL '2 hours' WHERE id = $1",
    )
    .bind(beta_private.id)
    .bind(collaborator.id)
    .execute(&pool)
    .await
    .expect("private metadata should update");
    sqlx::query(
        "UPDATE repositories SET created_by_user_id = $2, updated_at = now() - INTERVAL '3 hours' WHERE id = $1",
    )
    .bind(gamma_internal.id)
    .bind(member.id)
    .execute(&pool)
    .await
    .expect("internal metadata should update");
    sqlx::query("UPDATE repositories SET updated_at = now() - INTERVAL '4 hours' WHERE id = $1")
        .bind(fork.id)
        .execute(&pool)
        .await
        .expect("fork metadata should update");
    sqlx::query(
        "INSERT INTO repository_forks (source_repository_id, fork_repository_id, forked_by_user_id) VALUES ($1, $2, $3)",
    )
    .bind(upstream.id)
    .bind(fork.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("fork edge should insert");
    sqlx::query(
        "INSERT INTO repository_languages (repository_id, language, color, byte_count) VALUES ($1, 'Rust', '#b7410e', 900), ($2, 'TypeScript', '#8c5a3c', 500), ($3, 'Go', '#6f8f72', 200), ($4, 'Rust', '#b7410e', 100)",
    )
    .bind(alpha.id)
    .bind(beta_private.id)
    .bind(gamma_internal.id)
    .bind(fork.id)
    .execute(&pool)
    .await
    .expect("languages should insert");
    sqlx::query(
        "INSERT INTO repository_topics (repository_id, topic) VALUES ($1, 'api'), ($2, 'template'), ($3, 'internal'), ($4, 'worker')",
    )
    .bind(alpha.id)
    .bind(beta_private.id)
    .bind(gamma_internal.id)
    .bind(fork.id)
    .execute(&pool)
    .await
    .expect("topics should insert");
    sqlx::query(
        "INSERT INTO repository_stars (user_id, repository_id) VALUES ($1, $2), ($3, $2), ($1, $4)",
    )
    .bind(member.id)
    .bind(alpha.id)
    .bind(outsider.id)
    .bind(fork.id)
    .execute(&pool)
    .await
    .expect("stars should insert");
    sqlx::query(
        "INSERT INTO repository_permissions (repository_id, user_id, role, source) VALUES ($1, $2, 'admin', 'direct')",
    )
    .bind(beta_private.id)
    .bind(collaborator.id)
    .execute(&pool)
    .await
    .expect("repository permission should insert");

    let (status, headers, anonymous) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/repositories?sort=name-asc&page=0&pageSize=500"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(anonymous["page"], 1);
    assert_eq!(anonymous["pageSize"], 100);
    assert_eq!(anonymous["total"], 2);
    assert_eq!(
        repo_names(&anonymous),
        vec![format!("{marker}-alpha"), format!("{marker}-fork")]
    );
    assert_eq!(option_count(&anonymous, "availableTypes", "contributed"), 0);
    assert_eq!(option_count(&anonymous, "availableTypes", "admin"), 0);
    assert_eq!(option_count(&anonymous, "availableTypes", "public"), 2);
    assert_eq!(option_count(&anonymous, "availableLanguages", "Rust"), 2);
    assert_eq!(anonymous["viewerState"]["authenticated"], false);
    assert!(anonymous["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["visibility"] == "public"));

    let (status, _, member_view) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/repositories?sort=name-asc"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(member_view["total"], 4);
    assert_eq!(
        repo_names(&member_view),
        vec![
            format!("{marker}-alpha"),
            format!("{marker}-fork"),
            format!("{marker}-internal"),
            format!("{marker}-private"),
        ]
    );
    assert_eq!(
        option_count(&member_view, "availableTypes", "contributed"),
        1
    );
    assert_eq!(option_count(&member_view, "availableTypes", "templates"), 1);
    assert_eq!(option_count(&member_view, "availableTypes", "forks"), 1);

    let (status, _, filtered) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/repositories?q=template&language=typescript&type=templates"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(repo_names(&filtered), vec![format!("{marker}-private")]);
    assert_eq!(filtered["filters"]["query"], "template");
    assert_eq!(filtered["filters"]["language"], "TypeScript");
    assert_eq!(filtered["filters"]["repositoryType"], "templates");
    assert_eq!(filtered["total"], 1);
    assert_eq!(filtered["tabCounts"]["repositories"], 4);
    assert_eq!(option_count(&filtered, "availableTypes", "templates"), 1);

    let (status, _, admin_view) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/repositories?type=admin"),
        Some(&collaborator_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(repo_names(&admin_view), vec![format!("{marker}-private")]);
    assert_eq!(admin_view["items"][0]["canAdmin"], true);
    assert_eq!(admin_view["items"][0]["contributedByViewer"], true);

    let (status, _, page_two) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/repositories?sort=stars-desc&page=2&pageSize=1"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(page_two["total"], 4);
    assert_eq!(page_two["page"], 2);
    assert_eq!(page_two["pageSize"], 1);
    assert_eq!(repo_names(&page_two), vec![format!("{marker}-fork")]);

    let (status, _, normalized) = get_json(
        app.clone(),
        &format!(
            "/api/orgs/{marker}/repositories?q=%20%20&type=sources&language=RUST&page=-9&pageSize=0"
        ),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(normalized["filters"]["query"], Value::Null);
    assert_eq!(normalized["filters"]["language"], "Rust");
    assert_eq!(normalized["page"], 1);
    assert_eq!(normalized["pageSize"], 1);
    assert_eq!(normalized["tabCounts"]["repositories"], 4);

    let (status, _, invalid) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/repositories?type=secrets&sort=panic&density=wide"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid["error"]["code"], "validation_failed");
    let invalid_text = invalid.to_string();
    assert!(!invalid_text.contains("DATABASE_URL"));
    assert!(!invalid_text.contains("SESSION_SECRET"));
    assert!(!invalid_text.contains("stack backtrace"));
}

#[tokio::test]
async fn organization_repositories_hide_private_organizations_from_non_members() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping private organization repositories scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orgrepohidden{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Hidden Org".to_owned(),
            description: None,
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    sqlx::query("UPDATE organizations SET profile_visibility = 'private' WHERE id = $1")
        .bind(org.id)
        .execute(&pool)
        .await
        .expect("organization should be private");
    create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("{marker}-public"),
            description: Some("public but org hidden".to_owned()),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");

    let (status, _, hidden) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/repositories"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(hidden["error"]["code"], "not_found");

    let (status, _, owner_view) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/repositories"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(owner_view["total"], 1);
    assert_eq!(owner_view["viewerState"]["canAdmin"], true);
}
