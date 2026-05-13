use opengithub_api::{
    api_types::{error_response, ListEnvelope},
    domain::{
        identity::upsert_user_by_email,
        repositories::{
            create_repository, CreateRepository, RepositoryOwner, RepositoryVisibility,
        },
        search::{
            search_documents, upsert_search_document, SearchDocumentKind, SearchError, SearchQuery,
            UpsertSearchDocument,
        },
    },
    middleware::request_log::{record_api_request_log, RequestLogInput},
};
use serde_json::json;
use sqlx::{PgPool, Row};
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

async fn user_repository_fixture(
    pool: &PgPool,
    prefix: &str,
    visibility: RepositoryVisibility,
) -> (Uuid, Uuid) {
    let unique = Uuid::new_v4();
    let owner = upsert_user_by_email(
        pool,
        &format!("{prefix}-owner-{unique}@opengithub.local"),
        Some("Search Owner"),
        None,
    )
    .await
    .expect("owner should upsert");

    let repository = create_repository(
        pool,
        CreateRepository {
            owner: RepositoryOwner::User { id: owner.id },
            name: format!("{prefix}-{unique}"),
            description: None,
            visibility,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");

    (owner.id, repository.id)
}

#[tokio::test]
async fn search_documents_rank_results_and_enforce_repository_visibility() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres schema contract scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let (owner_id, private_repository_id) =
        user_repository_fixture(&pool, "search-private", RepositoryVisibility::Private).await;
    let (public_owner_id, public_repository_id) =
        user_repository_fixture(&pool, "search-public", RepositoryVisibility::Public).await;
    let private_marker = format!("retry-{}", Uuid::new_v4().simple());
    let public_marker = format!("guide-{}", Uuid::new_v4().simple());
    let outsider = upsert_user_by_email(
        &pool,
        &format!("search-outsider-{}@opengithub.local", Uuid::new_v4()),
        Some("Search Outsider"),
        None,
    )
    .await
    .expect("outsider should upsert");

    let private_doc = upsert_search_document(
        &pool,
        owner_id,
        UpsertSearchDocument {
            repository_id: Some(private_repository_id),
            owner_user_id: Some(owner_id),
            owner_organization_id: None,
            kind: SearchDocumentKind::Code,
            resource_id: format!("code-private-{private_repository_id}"),
            title: format!("Billing webhook {private_marker} worker"),
            body: Some(format!(
                "Retry failed webhook deliveries with bounded backoff for {private_marker}."
            )),
            path: Some("crates/api/src/jobs/webhooks.rs".to_owned()),
            language: Some("rust".to_owned()),
            branch: Some("main".to_owned()),
            visibility: RepositoryVisibility::Private,
            metadata: json!({ "sha": "abc123" }),
        },
    )
    .await
    .expect("owner should index private repository document");
    assert_eq!(private_doc.repository_id, Some(private_repository_id));

    upsert_search_document(
        &pool,
        public_owner_id,
        UpsertSearchDocument {
            repository_id: Some(public_repository_id),
            owner_user_id: Some(public_owner_id),
            owner_organization_id: None,
            kind: SearchDocumentKind::Code,
            resource_id: format!("code-public-{public_repository_id}"),
            title: format!("Public webhook {public_marker} guide"),
            body: Some(format!(
                "Document webhook setup for public repositories with {public_marker}."
            )),
            path: Some("docs/webhooks.md".to_owned()),
            language: Some("markdown".to_owned()),
            branch: Some("main".to_owned()),
            visibility: RepositoryVisibility::Public,
            metadata: json!({}),
        },
    )
    .await
    .expect("owner should index public repository document");

    let forbidden = upsert_search_document(
        &pool,
        outsider.id,
        UpsertSearchDocument {
            repository_id: Some(private_repository_id),
            owner_user_id: Some(owner_id),
            owner_organization_id: None,
            kind: SearchDocumentKind::Code,
            resource_id: format!("blocked-{private_repository_id}"),
            title: "Blocked".to_owned(),
            body: None,
            path: None,
            language: None,
            branch: None,
            visibility: RepositoryVisibility::Private,
            metadata: json!({}),
        },
    )
    .await;
    assert!(matches!(
        forbidden,
        Err(SearchError::RepositoryAccessDenied)
    ));

    let owner_results = search_documents(
        &pool,
        SearchQuery {
            actor_user_id: owner_id,
            query: private_marker.clone(),
            kind: Some(SearchDocumentKind::Code),
            page: 1,
            page_size: 10,
        },
    )
    .await
    .expect("owner search should succeed");
    assert_eq!(owner_results.total, 1);
    assert_eq!(
        owner_results.items[0].document.repository_id,
        Some(private_repository_id)
    );
    assert!(owner_results.items[0].rank > 0.0);

    let outsider_results = search_documents(
        &pool,
        SearchQuery {
            actor_user_id: outsider.id,
            query: public_marker.clone(),
            kind: Some(SearchDocumentKind::Code),
            page: 1,
            page_size: 10,
        },
    )
    .await
    .expect("outsider search should succeed");
    assert_eq!(outsider_results.total, 1);
    assert_eq!(
        outsider_results.items[0].document.repository_id,
        Some(public_repository_id)
    );
}

#[tokio::test]
async fn request_logs_tokens_and_api_envelopes_keep_contracts_stable() {
    let Some(pool) = database_pool().await else {
        eprintln!(
            "skipping Postgres schema contract scenario; set TEST_DATABASE_URL or DATABASE_URL"
        );
        return;
    };

    let user = upsert_user_by_email(
        &pool,
        &format!("contracts-{}@opengithub.local", Uuid::new_v4()),
        Some("Contract User"),
        None,
    )
    .await
    .expect("user should upsert");

    let log_id = record_api_request_log(
        &pool,
        RequestLogInput {
            request_id: Some("req-contract".to_owned()),
            actor_user_id: Some(user.id),
            method: "GET".to_owned(),
            path: "/api/search".to_owned(),
            status: 200,
            duration_ms: 17,
            user_agent: Some("schema-contract-test".to_owned()),
            metadata: json!({
                "accept": "application/json",
                "contentType": null,
                "method": "GET"
            }),
        },
    )
    .await
    .expect("request log should insert");

    let log_row = sqlx::query("SELECT path, metadata FROM api_request_logs WHERE id = $1")
        .bind(log_id)
        .fetch_one(&pool)
        .await
        .expect("request log should fetch");
    assert_eq!(log_row.get::<String, _>("path"), "/api/search");
    let metadata = log_row.get::<serde_json::Value, _>("metadata");
    assert!(
        metadata.get("authorization").is_none(),
        "request logs should not persist secret-bearing headers"
    );

    let token_marker = Uuid::new_v4().simple().to_string();
    let token_prefix = format!("oghp_{token_marker}");
    let token_hash = format!("sha256:{token_marker}");

    sqlx::query(
        r#"
        INSERT INTO personal_access_tokens (user_id, resource_owner_user_id, name, prefix, token_hash, scopes)
        VALUES ($1, $1, 'local automation', $2, $3, ARRAY['repo', 'workflow'])
        "#,
    )
    .bind(user.id)
    .bind(&token_prefix)
    .bind(&token_hash)
    .execute(&pool)
    .await
    .expect("personal access token hash should insert");
    let duplicate = sqlx::query(
        r#"
        INSERT INTO personal_access_tokens (user_id, resource_owner_user_id, name, prefix, token_hash, scopes)
        VALUES ($1, $1, 'duplicate', $2, $3, ARRAY[]::text[])
        "#,
    )
    .bind(user.id)
    .bind(&token_prefix)
    .bind(format!("sha256:other-{token_marker}"))
    .execute(&pool)
    .await;
    assert!(duplicate.is_err(), "token prefixes must be unique");

    let envelope = ListEnvelope {
        items: vec!["one"],
        total: 1,
        page: 1,
        page_size: 30,
    };
    assert_eq!(json!(envelope)["pageSize"], 30);

    let (status, body) = error_response(
        axum::http::StatusCode::UNPROCESSABLE_ENTITY,
        "validation_failed",
        "bad input",
    );
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body.0.error.code, "validation_failed");

    let plan = sqlx::query(
        "EXPLAIN SELECT * FROM api_request_logs WHERE path ILIKE '%search%' ORDER BY created_at DESC LIMIT 5",
    )
    .fetch_all(&pool)
    .await
    .expect("representative query plan should compile");
    assert!(!plan.is_empty());
}
