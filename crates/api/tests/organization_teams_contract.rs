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
            create_organization, create_repository, repository_permission_for_user,
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
        .map_err(|error| eprintln!("organization teams DB connect failed: {error}"))
        .ok()?;
    if let Err(error) = MIGRATOR.run(&pool).await {
        eprintln!("organization teams migration warning: {error}");
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

async fn post_json(
    app: axum::Router,
    uri: &str,
    cookie: Option<&str>,
    body: Value,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder()
        .method(Method::POST)
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

fn team_slugs(body: &Value) -> Vec<String> {
    body["items"]
        .as_array()
        .expect("items should be an array")
        .iter()
        .map(|item| {
            item["slug"]
                .as_str()
                .expect("slug should be string")
                .to_owned()
        })
        .collect()
}

async fn insert_org_member(pool: &PgPool, organization_id: Uuid, user_id: Uuid, role: &str) {
    sqlx::query(
        "INSERT INTO organization_memberships (organization_id, user_id, role) VALUES ($1, $2, $3)",
    )
    .bind(organization_id)
    .bind(user_id)
    .bind(role)
    .execute(pool)
    .await
    .expect("organization membership should insert");
}

async fn insert_team(
    pool: &PgPool,
    organization_id: Uuid,
    slug: &str,
    name: &str,
    visibility: &str,
    parent_team_id: Option<Uuid>,
) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO teams (
            organization_id, slug, name, description, visibility, parent_team_id,
            notifications_enabled
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(organization_id)
    .bind(slug)
    .bind(name)
    .bind(format!("{name} coordinates product work."))
    .bind(visibility)
    .bind(parent_team_id)
    .bind(visibility == "visible")
    .fetch_one(pool)
    .await
    .expect("team should insert")
}

#[tokio::test]
async fn organization_teams_directory_authorizes_filters_and_redacts() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization teams scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orgteams{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let admin = create_user(&pool, &format!("{marker}-admin")).await;
    let member = create_user(&pool, &format!("{marker}-member")).await;
    let secret_member = create_user(&pool, &format!("{marker}-secret")).await;
    let outsider = create_user(&pool, &format!("{marker}-outsider")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let admin_cookie = cookie_header(&pool, &config, &admin).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let secret_cookie = cookie_header(&pool, &config, &secret_member).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Teams Guild".to_owned(),
            description: Some("Team directory contract".to_owned()),
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    insert_org_member(&pool, org.id, admin.id, "admin").await;
    insert_org_member(&pool, org.id, member.id, "member").await;
    insert_org_member(&pool, org.id, secret_member.id, "member").await;

    let visible = insert_team(
        &pool,
        org.id,
        &format!("{marker}-platform"),
        "Platform",
        "visible",
        None,
    )
    .await;
    let child = insert_team(
        &pool,
        org.id,
        &format!("{marker}-frontend"),
        "Frontend",
        "visible",
        Some(visible),
    )
    .await;
    let secret = insert_team(
        &pool,
        org.id,
        &format!("{marker}-security"),
        "Security",
        "secret",
        None,
    )
    .await;
    sqlx::query("INSERT INTO team_memberships (team_id, user_id, role) VALUES ($1, $2, 'member'), ($3, $4, 'member'), ($5, $4, 'maintainer')")
        .bind(visible)
        .bind(member.id)
        .bind(child)
        .bind(secret_member.id)
        .bind(secret)
        .execute(&pool)
        .await
        .expect("team memberships should insert");
    let repo = create_repository(
        &pool,
        CreateRepository {
            owner: RepositoryOwner::Organization { id: org.id },
            name: format!("{marker}-repo"),
            description: Some("repository permission count".to_owned()),
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");
    sqlx::query("INSERT INTO repository_team_permissions (repository_id, team_id, role) VALUES ($1, $2, 'write'), ($1, $3, 'read')")
        .bind(repo.id)
        .bind(visible)
        .bind(child)
        .execute(&pool)
        .await
        .expect("team repository permissions should insert");
    sqlx::query(
        r#"
        INSERT INTO organization_team_mentions (
            organization_id, team_id, source_kind, source_id, mentioned_by_user_id,
            notification_status
        )
        VALUES ($1, $2, 'issue', $3, $4, 'sent')
        "#,
    )
    .bind(org.id)
    .bind(visible)
    .bind(Uuid::new_v4())
    .bind(member.id)
    .execute(&pool)
    .await
    .expect("team mention should insert");
    let inherited_permission = repository_permission_for_user(&pool, repo.id, secret_member.id)
        .await
        .expect("inherited repository permission should load")
        .expect("child team membership should inherit parent repository permission");
    assert_eq!(inherited_permission.role.as_str(), "write");
    assert_eq!(inherited_permission.source, "team");
    sqlx::query(
        r#"
        INSERT INTO organization_invitations (
            organization_id, invited_email, role, token_hash, invited_by_user_id, expires_at
        )
        VALUES ($1, $2, 'member', $3, $4, now() + INTERVAL '7 days')
        "#,
    )
    .bind(org.id)
    .bind(format!("{marker}-private-invite@opengithub.local"))
    .bind(format!("sha256:{marker}-token"))
    .bind(owner.id)
    .execute(&pool)
    .await
    .expect("private invitation should insert");

    let (status, headers, anonymous) =
        get_json(app.clone(), &format!("/api/orgs/{marker}/teams"), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_json(&headers);
    assert_eq!(anonymous["error"]["code"], "not_authenticated");

    let (status, _, outsider_forbidden) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(outsider_forbidden["error"]["code"], "forbidden");

    let (status, _, member_view) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams?visibility=all&page=0&pageSize=500"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(member_view["page"], 1);
    assert_eq!(member_view["pageSize"], 100);
    assert_eq!(member_view["total"], 2);
    assert_eq!(
        team_slugs(&member_view),
        vec![format!("{marker}-frontend"), format!("{marker}-platform")]
    );
    assert_eq!(member_view["counts"]["visible"], 2);
    assert_eq!(member_view["counts"]["secret"], 0);
    assert_eq!(member_view["counts"]["memberTeams"], 1);
    assert_eq!(member_view["viewerState"]["role"], "member");
    assert_eq!(member_view["viewerState"]["canAdminTeams"], false);
    assert_eq!(member_view["viewerState"]["canCreateTeam"], true);
    assert_eq!(
        member_view["items"][0]["parent"]["slug"],
        format!("{marker}-platform")
    );
    assert_eq!(member_view["items"][1]["memberCount"], 1);
    assert_eq!(member_view["items"][1]["repositoryCount"], 1);
    assert_eq!(member_view["items"][1]["childTeamCount"], 1);
    assert_eq!(
        member_view["items"][1]["viewerCapabilities"]["canManage"],
        false
    );
    assert_eq!(
        member_view["items"][1]["viewerCapabilities"]["isMember"],
        true
    );
    assert_eq!(member_view["items"][1]["mentionable"], true);

    let (status, _, visible_detail) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams/{marker}-platform"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(visible_detail["team"]["slug"], format!("{marker}-platform"));
    assert_eq!(
        visible_detail["members"][0]["login"],
        format!("{marker}-member")
    );
    assert_eq!(visible_detail["repositories"][0]["role"], "write");
    assert_eq!(
        visible_detail["childTeams"][0]["slug"],
        format!("{marker}-frontend")
    );
    assert_eq!(visible_detail["mentionState"]["notificationsEnabled"], true);
    assert_eq!(
        visible_detail["mentionState"]["recentMentions"][0]["notificationStatus"],
        "sent"
    );

    let (status, _, child_detail) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams/{marker}-frontend"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        child_detail["hierarchy"]["parentChain"][0]["slug"],
        format!("{marker}-platform")
    );
    assert!(child_detail["repositories"]
        .as_array()
        .unwrap()
        .iter()
        .any(|repository| repository["inherited"] == true
            && repository["sourceTeamSlug"] == format!("{marker}-platform")
            && repository["role"] == "write"));

    let (status, _, secret_hidden) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams/{marker}-security"),
        Some(&member_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(secret_hidden["error"]["code"], "not_found");

    let member_text = member_view.to_string();
    assert!(!member_text.contains(&format!("{marker}-security")));
    assert!(!member_text.contains("private-invite"));
    assert!(!member_text.contains("token"));

    let (status, _, secret_member_view) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams?visibility=secret"),
        Some(&secret_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        team_slugs(&secret_member_view),
        vec![format!("{marker}-security")]
    );
    assert_eq!(
        secret_member_view["items"][0]["viewerCapabilities"]["isMember"],
        true
    );
    assert_eq!(
        secret_member_view["items"][0]["notificationsEnabled"],
        false
    );

    let (status, _, owner_view) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams?q=security"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(team_slugs(&owner_view), vec![format!("{marker}-security")]);
    assert_eq!(owner_view["counts"]["total"], 3);
    assert_eq!(owner_view["counts"]["secret"], 1);
    assert_eq!(owner_view["viewerState"]["canAdminTeams"], true);
    assert_eq!(
        owner_view["items"][0]["viewerCapabilities"]["canManage"],
        true
    );

    let (status, _, owner_member_filter) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams?visibility=member"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(owner_member_filter["total"], 0);
    assert_eq!(owner_member_filter["counts"]["memberTeams"], 0);
    assert!(owner_member_filter["items"].as_array().unwrap().is_empty());

    let (status, _, admin_page) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams?page=2&pageSize=1"),
        Some(&admin_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(admin_page["total"], 3);
    assert_eq!(admin_page["page"], 2);
    assert_eq!(admin_page["pageSize"], 1);
    assert_eq!(admin_page["parentOptions"].as_array().unwrap().len(), 2);

    let (status, _, invalid) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams?visibility=buried"),
        Some(&owner_cookie),
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
async fn organization_teams_empty_state_and_private_org_privacy() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization teams empty-state scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orgteamsempty{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let outsider = create_user(&pool, &format!("{marker}-outsider")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let outsider_cookie = cookie_header(&pool, &config, &outsider).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Empty Teams".to_owned(),
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
        .expect("organization should become private");

    let (status, _, hidden) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams"),
        Some(&outsider_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(hidden["error"]["code"], "not_found");

    let (status, _, owner_view) = get_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams"),
        Some(&owner_cookie),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(owner_view["total"], 0);
    assert_eq!(
        owner_view["emptyState"]["newTeamHref"],
        format!("/orgs/{marker}/teams/new")
    );
    assert_eq!(
        owner_view["emptyState"]["learnMoreHref"],
        "/docs/api#organization-teams"
    );
    let column_titles: Vec<_> = owner_view["emptyState"]["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|column| column["title"].as_str().unwrap())
        .collect();
    assert_eq!(
        column_titles,
        vec![
            "Flexible repository access",
            "Request-to-join teams",
            "Team mentions"
        ]
    );
}

#[tokio::test]
async fn organization_team_create_validates_policy_parent_rules_and_audits() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping organization team create scenario; set TEST_DATABASE_URL");
        return;
    };

    let config = app_config();
    let marker = format!("orgteamcreate{}", Uuid::new_v4().simple());
    let owner = create_user(&pool, &format!("{marker}-owner")).await;
    let member = create_user(&pool, &format!("{marker}-member")).await;
    let blocked_member = create_user(&pool, &format!("{marker}-blocked")).await;
    let owner_cookie = cookie_header(&pool, &config, &owner).await;
    let member_cookie = cookie_header(&pool, &config, &member).await;
    let blocked_cookie = cookie_header(&pool, &config, &blocked_member).await;
    let app = opengithub_api::build_app_with_config(Some(pool.clone()), config);
    let org = create_organization(
        &pool,
        CreateOrganization {
            slug: marker.clone(),
            display_name: "Create Teams".to_owned(),
            description: Some("Team creation contract".to_owned()),
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("organization should create");
    insert_org_member(&pool, org.id, member.id, "member").await;
    let locked_org = create_organization(
        &pool,
        CreateOrganization {
            slug: format!("{marker}-locked"),
            display_name: "Locked Teams".to_owned(),
            description: None,
            owner_user_id: owner.id,
        },
    )
    .await
    .expect("locked organization should create");
    insert_org_member(&pool, locked_org.id, blocked_member.id, "member").await;
    sqlx::query(
        r#"
        INSERT INTO organization_policy_settings (organization_id, members_can_create_teams)
        VALUES ($1, false)
        ON CONFLICT (organization_id)
        DO UPDATE SET members_can_create_teams = false
        "#,
    )
    .bind(locked_org.id)
    .execute(&pool)
    .await
    .expect("policy should update");
    let parent = insert_team(
        &pool,
        org.id,
        &format!("{marker}-parent"),
        "Parent",
        "visible",
        None,
    )
    .await;
    let secret_parent = insert_team(
        &pool,
        org.id,
        &format!("{marker}-secret-parent"),
        "Secret Parent",
        "secret",
        None,
    )
    .await;

    let (status, headers, created) = post_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams"),
        Some(&member_cookie),
        json!({
            "name": "Release Infrastructure!",
            "description": "Owns release trains.",
            "parentTeamId": parent,
            "visibility": "visible",
            "notificationsEnabled": false
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_json(&headers);
    assert_eq!(created["team"]["slug"], "release-infrastructure");
    assert_eq!(created["team"]["parent"]["id"], parent.to_string());
    assert_eq!(created["team"]["notificationsEnabled"], false);
    assert_eq!(
        created["destinationHref"],
        format!("/orgs/{marker}/teams/release-infrastructure")
    );
    let membership_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM team_memberships
        JOIN teams ON teams.id = team_memberships.team_id
        WHERE teams.organization_id = $1
          AND teams.slug = 'release-infrastructure'
          AND team_memberships.user_id = $2
          AND team_memberships.role = 'maintainer'
        "#,
    )
    .bind(org.id)
    .bind(member.id)
    .fetch_one(&pool)
    .await
    .expect("membership count should load");
    assert_eq!(membership_count, 1);
    let audit_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM organization_audit_events
        WHERE organization_id = $1
          AND event_type = 'organization.team.create'
          AND metadata->>'slug' = 'release-infrastructure'
          AND metadata::text NOT LIKE '%Owns release trains%'
        "#,
    )
    .bind(org.id)
    .fetch_one(&pool)
    .await
    .expect("audit count should load");
    assert_eq!(audit_count, 1);

    let (status, _, duplicate) = post_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams"),
        Some(&owner_cookie),
        json!({
            "name": "Release Infrastructure",
            "visibility": "visible",
            "notificationsEnabled": true
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(duplicate["error"]["code"], "conflict");

    let (status, _, secret_nested) = post_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams"),
        Some(&owner_cookie),
        json!({
            "name": "Private Child",
            "parentTeamId": parent,
            "visibility": "secret",
            "notificationsEnabled": true
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(secret_nested["error"]["code"], "validation_failed");

    let (status, _, secret_parent_error) = post_json(
        app.clone(),
        &format!("/api/orgs/{marker}/teams"),
        Some(&owner_cookie),
        json!({
            "name": "Visible Child",
            "parentTeamId": secret_parent,
            "visibility": "visible",
            "notificationsEnabled": true
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(secret_parent_error["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Secret teams cannot be used as parent"));

    let (status, _, blocked) = post_json(
        app.clone(),
        &format!("/api/orgs/{}-locked/teams", marker),
        Some(&blocked_cookie),
        json!({
            "name": "Blocked Member Team",
            "visibility": "visible",
            "notificationsEnabled": true
        }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(blocked["error"]["code"], "policy_locked");
    assert_eq!(blocked["details"]["field"], "membersCanCreateTeams");
    assert_eq!(
        blocked["details"]["settingsHref"],
        format!("/organizations/{marker}-locked/settings/member_privileges")
    );
}
