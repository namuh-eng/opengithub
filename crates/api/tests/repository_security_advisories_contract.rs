use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
};
use chrono::{Duration, Utc};
use opengithub_api::{
    auth::session,
    config::{AppConfig, AuthConfig},
    domain::{
        identity::{upsert_session, upsert_user_by_email, User},
        permissions::RepositoryRole,
        repositories::{
            create_repository, grant_repository_permission, CreateRepository, RepositoryOwner,
            RepositoryVisibility,
        },
        repository_security::{
            create_repository_security_advisory_for_actor_by_owner_name,
            publish_repository_security_advisory_for_actor_by_owner_name,
            repository_security_advisories_for_actor_by_owner_name,
            repository_security_advisory_detail_for_actor_by_owner_name,
            RepositorySecurityAdvisoriesQuery, RepositorySecurityAdvisoryCreate,
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

    let pool = match opengithub_api::db::test_pool_options()
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("skipping repository advisories scenario; database connect failed: {error}");
            return None;
        }
    };
    if let Err(error) = MIGRATOR.run(&pool).await {
        let has_advisory_tables = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT to_regclass('public.repository_security_advisories') IS NOT NULL
               AND to_regclass('public.repository_security_advisory_credits') IS NOT NULL
               AND to_regclass('public.repository_security_advisory_events') IS NOT NULL
            "#,
        )
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if !has_advisory_tables {
            eprintln!("skipping repository advisories scenario; migration failed: {error}");
            return None;
        }
        eprintln!(
            "continuing repository advisories scenario with pre-applied schema after migration warning: {error}"
        );
    }
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
    let suffix = Uuid::new_v4().simple();
    let user = upsert_user_by_email(
        pool,
        &format!("{label}-{suffix}@opengithub.local"),
        Some(label),
        Some(&format!("https://avatars.opengithub.local/{label}.png")),
    )
    .await
    .expect("user should upsert");
    sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
        .bind(format!("{label}-{suffix}"))
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

async fn get_json(app: axum::Router, uri: &str, cookie: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(builder.body(Body::empty()).expect("request should build"))
        .await
        .expect("request should run");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    (
        status,
        serde_json::from_slice(&bytes).expect("response should be json"),
    )
}

async fn patch_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method("PATCH")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(
            builder
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should run");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    (
        status,
        serde_json::from_slice(&bytes).expect("response should be json"),
    )
}

async fn post_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = app
        .oneshot(
            builder
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should run");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    (
        status,
        serde_json::from_slice(&bytes).expect("response should be json"),
    )
}

#[tokio::test]
async fn repository_security_advisories_hide_drafts_and_return_detail_metadata() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping repository advisories scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let owner = create_user(&pool, "advisory-owner").await;
    let reader = create_user(&pool, "advisory-reader").await;
    let outsider = create_user(&pool, "advisory-outsider").await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let reader_cookie = cookie_header(&pool, &config, &reader).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let repository = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("advisories-{}", Uuid::new_v4().simple()),
            description: Some("Repository advisories contract".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    grant_repository_permission(
        &pool,
        repository.id,
        reader.id,
        RepositoryRole::Read,
        "direct",
    )
    .await
    .expect("reader permission should grant");

    let published_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO repository_security_advisories (
            repository_id, advisory_identifier, ghsa_id, cve_id, severity, status, title, summary,
            package_ecosystem, package_name, vulnerable_range, affected_versions, patched_versions,
            cvss_vector, cvss_score, cvss_metrics, markdown_summary, markdown_details,
            author_user_id, advisory_href, published_at
        )
        VALUES (
            $1, 'GHSA-advisory-visible', 'GHSA-advisory-visible', 'CVE-2026-10001',
            'high', 'published', 'Visible repository advisory',
            'Patch the affected package before enabling untrusted markdown.',
            'npm', 'opengithub-markdown', '< 2.0.0', '< 2.0.0', '2.0.0',
            'CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:N', 9.1,
            '{"attackVector":"network","privilegesRequired":"none"}'::jsonb,
            'Visible summary', '## Impact\n\n<script>alert("x")</script>\n\nUse patched versions.',
            $2, '/advisories/GHSA-advisory-visible', now() - interval '1 day'
        )
        RETURNING id
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .fetch_one(&pool)
    .await
    .expect("published advisory should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_security_advisories (
            repository_id, advisory_identifier, ghsa_id, severity, status, title, summary,
            package_ecosystem, package_name, vulnerable_range, advisory_href, author_user_id
        )
        VALUES (
            $1, 'GHSA-advisory-draft', 'GHSA-advisory-draft', 'critical', 'draft',
            'Private draft advisory', 'Draft details stay private.', 'cargo', 'secret-crate',
            '< 9.9.9', '/advisories/GHSA-advisory-draft', $2
        )
        "#,
    )
    .bind(repository.id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("draft advisory should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_security_advisory_cwes (advisory_id, cwe_id, name, href)
        VALUES ($1, 'CWE-79', 'Improper Neutralization of Input During Web Page Generation', 'https://cwe.mitre.org/data/definitions/79.html')
        "#,
    )
    .bind(published_id)
    .execute(&pool)
    .await
    .expect("cwe should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_security_advisory_credits (advisory_id, user_id, login, avatar_url, credit_type)
        VALUES ($1, $2, $3, $4, 'reporter')
        "#,
    )
    .bind(published_id)
    .bind(reader.id)
    .bind(reader.username.as_deref().expect("reader username"))
    .bind(reader.avatar_url.as_deref())
    .execute(&pool)
    .await
    .expect("credit should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_security_advisory_collaborators (advisory_id, user_id, login, avatar_url, role, invited_by_user_id)
        VALUES ($1, $2, $3, $4, 'author', $2)
        "#,
    )
    .bind(published_id)
    .bind(owner.id)
    .bind(owner.username.as_deref().expect("owner username"))
    .bind(owner.avatar_url.as_deref())
    .execute(&pool)
    .await
    .expect("collaborator should insert");
    sqlx::query(
        r#"
        INSERT INTO repository_security_advisory_events (advisory_id, actor_user_id, event_type, message)
        VALUES ($1, $2, 'published', 'Published advisory GHSA-advisory-visible')
        "#,
    )
    .bind(published_id)
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("event should insert");

    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let owner_login = owner.username.as_deref().expect("owner username");
    let base = format!(
        "/api/repos/{owner_login}/{}/security/advisories",
        repository.name
    );

    let (anonymous_status, anonymous_body) = get_json(app.clone(), &base, None).await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["error"]["code"], "not_authenticated");

    let (outsider_status, outsider_body) =
        get_json(app.clone(), &base, Some(&outsider_cookie)).await;
    assert_eq!(outsider_status, StatusCode::NOT_FOUND);
    assert!(!outsider_body.to_string().contains("Private draft advisory"));

    let (reader_status, reader_body) = get_json(app.clone(), &base, Some(&reader_cookie)).await;
    assert_eq!(reader_status, StatusCode::OK, "{reader_body}");
    assert_eq!(reader_body["viewer"]["canWrite"], false);
    assert_eq!(reader_body["counts"]["published"], 1);
    assert_eq!(reader_body["counts"]["draft"], Value::Null);
    assert_eq!(reader_body["advisories"].as_array().expect("rows").len(), 1);
    assert_eq!(
        reader_body["advisories"][0]["ghsaId"],
        "GHSA-advisory-visible"
    );
    assert!(!reader_body.to_string().contains("GHSA-advisory-draft"));
    assert!(!reader_body.to_string().contains("test-session-secret"));

    let (owner_status, owner_body) = get_json(
        app.clone(),
        &format!("{base}?state=all"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(owner_status, StatusCode::OK, "{owner_body}");
    assert_eq!(owner_body["counts"]["draft"], 1);
    assert_eq!(owner_body["advisories"].as_array().expect("rows").len(), 2);
    assert!(owner_body.to_string().contains("GHSA-advisory-draft"));

    let (filter_status, filter_body) = get_json(
        app.clone(),
        &format!("{base}?severity=critical"),
        Some(&reader_cookie),
    )
    .await;
    assert_eq!(filter_status, StatusCode::OK);
    assert_eq!(filter_body["advisories"].as_array().expect("rows").len(), 0);

    let (invalid_status, invalid_body) = get_json(
        app.clone(),
        &format!("{base}?severity=urgent"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_body["error"]["code"], "validation_failed");

    let detail_uri = format!("{base}/GHSA-advisory-visible");
    let (detail_status, detail_body) =
        get_json(app.clone(), &detail_uri, Some(&reader_cookie)).await;
    assert_eq!(detail_status, StatusCode::OK, "{detail_body}");
    assert_eq!(detail_body["advisory"]["cveId"], "CVE-2026-10001");
    assert_eq!(detail_body["advisory"]["package"]["ecosystem"], "npm");
    assert_eq!(detail_body["advisory"]["cvss"]["score"], 9.1);
    assert_eq!(detail_body["advisory"]["cwes"][0]["id"], "CWE-79");
    assert_eq!(detail_body["credits"][0]["creditType"], "reporter");
    assert_eq!(detail_body["collaborators"][0]["role"], "author");
    assert_eq!(detail_body["timeline"][0]["eventType"], "published");
    let html = detail_body["markdown"]["detailsHtml"]
        .as_str()
        .expect("html");
    assert!(html.contains("Use patched versions"));
    assert!(!html.contains("<script"));
    assert!(!detail_body.to_string().contains("google-client-secret"));

    let mutation = json!({
        "title": "Updated repository advisory",
        "summary": "Updated maintainer-safe summary.",
        "detailsMarkdown": "## Remediation\n\nUpgrade immediately. <script>alert(\"x\")</script>",
        "cveId": "CVE-2026-20002",
        "severity": "critical",
        "packageEcosystem": "cargo",
        "packageName": "opengithub-markdown",
        "affectedVersions": "< 3.0.0",
        "patchedVersions": ">= 3.0.0",
        "cvssVector": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H",
        "cvssScore": 9.8,
        "cvssMetrics": { "attackVector": "network", "availability": "high" },
        "cwes": [{ "id": "CWE-862", "name": "Missing Authorization", "href": null }],
        "credits": [{ "login": "security-reporter", "creditType": "analyst" }],
        "collaborators": [{ "login": "jaeyun", "role": "author" }]
    });
    let (reader_patch_status, _) = patch_json(
        app.clone(),
        &detail_uri,
        Some(&reader_cookie),
        mutation.clone(),
    )
    .await;
    assert_eq!(reader_patch_status, StatusCode::FORBIDDEN);
    let (owner_patch_status, owner_patch_body) =
        patch_json(app.clone(), &detail_uri, Some(&owner_cookie), mutation).await;
    assert_eq!(owner_patch_status, StatusCode::OK, "{owner_patch_body}");
    assert_eq!(
        owner_patch_body["advisory"]["title"],
        "Updated repository advisory"
    );
    assert_eq!(owner_patch_body["advisory"]["severity"], "critical");
    assert_eq!(owner_patch_body["advisory"]["cveId"], "CVE-2026-20002");
    assert_eq!(owner_patch_body["advisory"]["cwes"][0]["id"], "CWE-862");
    assert_eq!(owner_patch_body["credits"][0]["creditType"], "analyst");
    assert_eq!(
        owner_patch_body["timeline"]
            .as_array()
            .expect("timeline")
            .last()
            .expect("edited event")["eventType"],
        "edited"
    );
    assert!(!owner_patch_body["markdown"]["detailsHtml"]
        .as_str()
        .expect("updated html")
        .contains("<script"));

    let draft_detail = format!("{base}/GHSA-advisory-draft");
    let (reader_draft_status, reader_draft_body) =
        get_json(app.clone(), &draft_detail, Some(&reader_cookie)).await;
    assert_eq!(reader_draft_status, StatusCode::NOT_FOUND);
    assert!(!reader_draft_body
        .to_string()
        .contains("Private draft advisory"));
    let (owner_draft_status, owner_draft_body) =
        get_json(app.clone(), &draft_detail, Some(&owner_cookie)).await;
    assert_eq!(owner_draft_status, StatusCode::OK, "{owner_draft_body}");
    assert_eq!(owner_draft_body["viewer"]["canPublish"], true);

    let direct = repository_security_advisories_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
        RepositorySecurityAdvisoriesQuery {
            state: Some("all"),
            severity: None,
            query: Some("markdown"),
            sort: Some("recently_published"),
            page: Some(1),
            page_size: Some(10),
        },
    )
    .await
    .expect("direct advisories should load")
    .expect("repository should exist");
    assert_eq!(direct.advisories.len(), 1);
    let direct_detail = repository_security_advisory_detail_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
        "GHSA-advisory-visible",
    )
    .await
    .expect("direct detail should load")
    .expect("advisory should exist");
    assert_eq!(direct_detail.advisory.id, published_id);

    let create_body = json!({
        "title": "Draft advisory lifecycle",
        "summary": "Maintainers can prepare draft advisories privately.",
        "detailsMarkdown": "## Draft\n\nUse patched package versions.",
        "severity": "high",
        "packageEcosystem": "cargo",
        "packageName": "opengithub-api",
        "affectedVersions": "< 4.0.0",
        "patchedVersions": ">= 4.0.0",
        "cvssScore": 8.2,
        "cwes": [],
        "credits": [],
        "collaborators": []
    });
    let (invalid_create_status, invalid_create_body) = post_json(
        app.clone(),
        &base,
        Some(&owner_cookie),
        json!({ "title": "" }),
    )
    .await;
    assert_eq!(invalid_create_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid_create_body["error"]["code"], "validation_failed");
    let (reader_create_status, _) = post_json(
        app.clone(),
        &base,
        Some(&reader_cookie),
        create_body.clone(),
    )
    .await;
    assert_eq!(reader_create_status, StatusCode::FORBIDDEN);
    let (create_status, create_response) =
        post_json(app.clone(), &base, Some(&owner_cookie), create_body).await;
    assert_eq!(create_status, StatusCode::CREATED, "{create_response}");
    let created_ghsa = create_response["advisory"]["ghsaId"]
        .as_str()
        .expect("created ghsa");
    assert!(created_ghsa.starts_with("GHSA-local-"));
    assert_eq!(create_response["advisory"]["state"], "draft");
    assert_eq!(create_response["viewer"]["canPublish"], true);
    assert!(!create_response.to_string().contains("google-client-secret"));

    let created_detail_uri = format!("{base}/{created_ghsa}");
    let (reader_created_status, reader_created_body) =
        get_json(app.clone(), &created_detail_uri, Some(&reader_cookie)).await;
    assert_eq!(reader_created_status, StatusCode::NOT_FOUND);
    assert!(!reader_created_body
        .to_string()
        .contains("Draft advisory lifecycle"));
    let (publish_status, publish_body) = post_json(
        app.clone(),
        &format!("{created_detail_uri}/publish"),
        Some(&owner_cookie),
        json!({}),
    )
    .await;
    assert_eq!(publish_status, StatusCode::OK, "{publish_body}");
    assert_eq!(publish_body["advisory"]["state"], "published");
    assert!(publish_body["advisory"]["publishedAt"].is_string());
    assert_eq!(
        publish_body["timeline"]
            .as_array()
            .expect("timeline")
            .last()
            .expect("published event")["eventType"],
        "published"
    );
    assert_eq!(publish_body["viewer"]["canPublish"], false);
    let (reader_published_status, reader_published_body) =
        get_json(app.clone(), &created_detail_uri, Some(&reader_cookie)).await;
    assert_eq!(
        reader_published_status,
        StatusCode::OK,
        "{reader_published_body}"
    );
    assert_eq!(
        reader_published_body["advisory"]["title"],
        "Draft advisory lifecycle"
    );
    let (republish_status, republish_body) = post_json(
        app.clone(),
        &format!("{created_detail_uri}/publish"),
        Some(&owner_cookie),
        json!({}),
    )
    .await;
    assert_eq!(republish_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(republish_body["error"]["code"], "validation_failed");

    let direct_created = create_repository_security_advisory_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
        RepositorySecurityAdvisoryCreate {
            title: "Direct draft advisory".to_owned(),
            summary: Some("Direct draft summary.".to_owned()),
            details_markdown: Some("Direct draft details.".to_owned()),
            cve_id: None,
            severity: Some("moderate".to_owned()),
            package_ecosystem: None,
            package_name: None,
            affected_versions: None,
            patched_versions: None,
            cvss_vector: None,
            cvss_score: None,
            cvss_metrics: None,
            cwes: Vec::new(),
            credits: Vec::new(),
            collaborators: Vec::new(),
        },
    )
    .await
    .expect("direct create should work")
    .expect("direct create should return detail");
    let direct_published = publish_repository_security_advisory_for_actor_by_owner_name(
        &pool,
        owner.id,
        owner_login,
        &repository.name,
        &direct_created.advisory.ghsa_id,
    )
    .await
    .expect("direct publish should work")
    .expect("direct publish should return detail");
    assert_eq!(direct_published.advisory.state, "published");
}
