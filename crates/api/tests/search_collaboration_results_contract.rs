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
            create_repository, CreateRepository, RepositoryOwner, RepositoryVisibility,
        },
        search::{upsert_search_document, SearchDocumentKind, UpsertSearchDocument},
    },
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use url::{form_urlencoded, Url};
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

async fn create_user(pool: &PgPool, label: &str) -> User {
    upsert_user_by_email(
        pool,
        &format!("{label}-{}@opengithub.local", Uuid::new_v4()),
        Some(label),
        None,
    )
    .await
    .expect("user should upsert")
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

fn assert_json(headers: &HeaderMap) {
    assert!(headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json")));
}

fn encode_query_component(value: &str) -> String {
    form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

#[tokio::test]
async fn issue_search_returns_facets_snippets_counts_and_private_redaction() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping collaboration search scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "search004-owner").await;
    let assignee = create_user(&pool, "search004-assignee").await;
    let outsider = create_user(&pool, "search004-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let marker = format!("collab{}", Uuid::new_v4().simple());

    let public_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("public-{marker}"),
            description: Some(format!("Public collaboration search {marker}")),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("public repo should create");
    let private_repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("private-{marker}"),
            description: Some(format!("Private collaboration search {marker}")),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("private repo should create");

    let milestone_id: Uuid = sqlx::query_scalar(
        "INSERT INTO milestones (repository_id, title, created_by_user_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(public_repo.id)
    .bind(format!("Release {marker}"))
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("milestone should insert");
    let label_id: Uuid = sqlx::query_scalar(
        "INSERT INTO labels (repository_id, name, color, description) VALUES ($1, $2, 'b66a45', $3) RETURNING id",
    )
    .bind(public_repo.id)
    .bind(format!("bug-{marker}"))
    .bind("Customer-visible defect")
    .fetch_one(&pool)
    .await
    .expect("label should insert");
    let public_issue_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO issues (repository_id, number, title, body, state, author_user_id, milestone_id)
        VALUES ($1, 41, $2, $3, 'open', $4, $5)
        RETURNING id
        "#,
    )
    .bind(public_repo.id)
    .bind(format!("Router panic {marker}"))
    .bind(format!(
        "The router emits {marker} when search snippets render."
    ))
    .bind(owner.id)
    .bind(milestone_id)
    .fetch_one(&pool)
    .await
    .expect("issue should insert");
    sqlx::query("INSERT INTO issue_labels (issue_id, label_id) VALUES ($1, $2)")
        .bind(public_issue_id)
        .bind(label_id)
        .execute(&pool)
        .await
        .expect("issue label should insert");
    sqlx::query(
        "INSERT INTO issue_assignees (issue_id, user_id, assigned_by_user_id) VALUES ($1, $2, $3)",
    )
    .bind(public_issue_id)
    .bind(assignee.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("issue assignee should insert");
    sqlx::query(
        "INSERT INTO comments (repository_id, issue_id, author_user_id, body) VALUES ($1, $2, $3, $4)",
    )
    .bind(public_repo.id)
    .bind(public_issue_id)
    .bind(assignee.id)
    .bind(format!("I can reproduce {marker}."))
    .execute(&pool)
    .await
    .expect("comment should insert");
    sqlx::query(
        "INSERT INTO reactions (repository_id, issue_id, user_id, content) VALUES ($1, $2, $3, 'eyes')",
    )
    .bind(public_repo.id)
    .bind(public_issue_id)
    .bind(assignee.id)
    .execute(&pool)
    .await
    .expect("reaction should insert");

    let private_issue_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO issues (repository_id, number, title, body, state, author_user_id)
        VALUES ($1, 7, $2, $3, 'open', $4)
        RETURNING id
        "#,
    )
    .bind(private_repo.id)
    .bind(format!("Private router panic {marker}"))
    .bind(format!("Private body {marker}"))
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("private issue should insert");

    for (repo, issue_id, number, visibility, title) in [
        (
            &public_repo,
            public_issue_id,
            41,
            RepositoryVisibility::Public,
            format!("Router panic {marker}"),
        ),
        (
            &private_repo,
            private_issue_id,
            7,
            RepositoryVisibility::Private,
            format!("Private router panic {marker}"),
        ),
    ] {
        upsert_search_document(
            &pool,
            owner.id,
            UpsertSearchDocument {
                repository_id: Some(repo.id),
                owner_user_id: Some(owner.id),
                owner_organization_id: None,
                kind: SearchDocumentKind::Issue,
                resource_id: format!("{}:{number}", repo.id),
                title,
                body: Some(format!("Search body {marker}")),
                path: None,
                language: None,
                branch: None,
                visibility,
                metadata: json!({
                    "number": number,
                    "state": "open",
                    "href": format!("/{}/{}/issues/{number}", repo.owner_login, repo.name),
                }),
            },
        )
        .await
        .expect("issue document should persist");
        assert_ne!(issue_id, Uuid::nil());
    }

    let (status, headers, body) = get_json(
        app.clone(),
        &format!("/api/search?q={marker}%20state:open&type=issues&pageSize=10"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_json(&headers);
    assert_eq!(body["page"], 1);
    assert_eq!(body["pageSize"], 10);
    assert_eq!(body["total"], 2);
    assert_eq!(body["typeCounts"][0]["resultType"], "issues");
    assert_eq!(body["activeChips"][0]["qualifier"], "state");
    assert_eq!(body["facets"]["states"][0]["value"], "open");
    assert!(body["facets"]["labels"]
        .as_array()
        .expect("labels facet")
        .iter()
        .any(|item| item["value"] == format!("bug-{marker}")));
    let public_item = body["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["repository"]["name"] == public_repo.name)
        .expect("public issue should be present");
    assert_eq!(public_item["type"], "issues");
    assert_eq!(public_item["number"], 41);
    assert_eq!(
        public_item["href"],
        format!(
            "/{}/{}/issues/41",
            public_repo.owner_login, public_repo.name
        )
    );
    assert_eq!(public_item["labels"][0]["name"], format!("bug-{marker}"));
    assert_eq!(public_item["assignees"][0]["id"], assignee.id.to_string());
    assert_eq!(
        public_item["milestone"]["title"],
        format!("Release {marker}")
    );
    assert_eq!(public_item["commentCount"], 1);
    assert_eq!(public_item["interactionCount"], 1);
    assert!(!public_item["snippets"]
        .as_array()
        .expect("snippets")
        .is_empty());

    for (qualifier, expected_chip) in [
        (format!("label:bug-{marker}"), "label"),
        (
            format!("assignee:{}", assignee.username.as_deref().unwrap()),
            "assignee",
        ),
        (format!("milestone:\"Release {marker}\""), "milestone"),
        ("comments:>0".to_owned(), "comments"),
        ("interactions:1".to_owned(), "interactions"),
    ] {
        let encoded = encode_query_component(&format!("{marker} {qualifier}"));
        let (filter_status, _headers, filter_body) = get_json(
            app.clone(),
            &format!("/api/search?q={encoded}&type=issues&pageSize=10"),
            Some(&owner_cookie),
        )
        .await;
        assert_eq!(filter_status, StatusCode::OK);
        assert_eq!(filter_body["total"], 1, "filter {qualifier} should match");
        assert!(filter_body["activeChips"]
            .as_array()
            .expect("active chips")
            .iter()
            .any(|chip| chip["qualifier"] == expected_chip));
    }

    let missing_label_query = encode_query_component(&format!("{marker} label:not-{marker}"));
    let (_status, _headers, missing_label_body) = get_json(
        app.clone(),
        &format!("/api/search?q={missing_label_query}&type=issues&pageSize=10"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(missing_label_body["total"], 0);

    let invalid_range_query = encode_query_component(&format!("{marker} comments:many"));
    let (_status, _headers, invalid_range_body) = get_json(
        app.clone(),
        &format!("/api/search?q={invalid_range_query}&type=issues&pageSize=10"),
        Some(&owner_cookie),
    )
    .await;
    assert!(invalid_range_body["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .any(|diagnostic| diagnostic["code"] == "invalid_comments_qualifier"));

    let (outsider_status, _headers, outsider_body) = get_json(
        app,
        &format!("/api/search?q={marker}&type=issues&pageSize=10"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(outsider_status, StatusCode::OK);
    assert_eq!(outsider_body["total"], 1);
    let rendered = outsider_body.to_string();
    assert!(rendered.contains(&public_repo.name));
    assert!(!rendered.contains(&private_repo.name));
}

#[tokio::test]
async fn pull_request_search_alias_returns_pr_rows_and_sort_metadata() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping collaboration search scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "search004-pr-owner").await;
    let reviewer = create_user(&pool, "search004-pr-reviewer").await;
    let cookie = cookie_header(&pool, &config, &owner).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let marker = format!("prcollab{}", Uuid::new_v4().simple());
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("pulls-{marker}"),
            description: Some(format!("Pull search {marker}")),
            visibility: RepositoryVisibility::Public,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repo should create");

    let issue_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO issues (repository_id, number, title, body, state, author_user_id)
        VALUES ($1, 88, $2, $3, 'open', $4)
        RETURNING id
        "#,
    )
    .bind(repo.id)
    .bind(format!("PR issue {marker}"))
    .bind(format!("Underlying issue {marker}"))
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("issue should insert");
    let pull_request_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO pull_requests (
            repository_id, issue_id, number, title, body, state, author_user_id,
            head_ref, base_ref, head_repository_id, base_repository_id
        )
        VALUES ($1, $2, 88, $3, $4, 'open', $5, 'feature/search', 'main', $1, $1)
        RETURNING id
        "#,
    )
    .bind(repo.id)
    .bind(issue_id)
    .bind(format!("Teach search pullrequests alias {marker}"))
    .bind(format!("Pull request body {marker}"))
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("pull request should insert");
    sqlx::query(
        "INSERT INTO comments (repository_id, pull_request_id, author_user_id, body) VALUES ($1, $2, $3, $4)",
    )
    .bind(repo.id)
    .bind(pull_request_id)
    .bind(reviewer.id)
    .bind(format!("Review comment {marker}"))
    .execute(&pool)
    .await
    .expect("pull comment should insert");
    upsert_search_document(
        &pool,
        owner.id,
        UpsertSearchDocument {
            repository_id: Some(repo.id),
            owner_user_id: Some(owner.id),
            owner_organization_id: None,
            kind: SearchDocumentKind::PullRequest,
            resource_id: format!("{}:88", repo.id),
            title: format!("Teach search pullrequests alias {marker}"),
            body: Some(format!("Pull request body {marker}")),
            path: None,
            language: None,
            branch: Some("feature/search".to_owned()),
            visibility: RepositoryVisibility::Public,
            metadata: json!({
                "number": 88,
                "state": "open",
                "href": format!("/{}/{}/pull/88", repo.owner_login, repo.name),
            }),
        },
    )
    .await
    .expect("pull request search document should persist");

    let (status, _headers, body) = get_json(
        app,
        &format!("/api/search?q={marker}&type=pullrequests&sort=most_commented"),
        Some(&cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["sort"]["selected"], "most_commented");
    assert!(body["sort"]["options"]
        .as_array()
        .expect("sort options")
        .iter()
        .any(|option| option["value"] == "least_recently_updated"));
    assert_eq!(body["items"][0]["type"], "pull_requests");
    assert_eq!(
        body["items"][0]["href"],
        format!("/{}/{}/pull/88", repo.owner_login, repo.name)
    );
    assert_eq!(body["items"][0]["headRef"], "feature/search");
    assert_eq!(body["items"][0]["baseRef"], "main");
    assert_eq!(body["items"][0]["commentCount"], 1);
    assert_eq!(body["typeCounts"][1]["resultType"], "pull_requests");
}
