use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use opengithub_api::{
    domain::{
        identity::upsert_user_by_email,
        repositories::{
            create_repository, CreateRepository, RepositoryOwner, RepositoryVisibility,
        },
        webhooks::{
            create_webhook, enqueue_repository_webhook_event, CreateWebhook, DeliveryStatus,
        },
    },
    jobs::webhooks::{
        run_webhook_delivery_once_with_config, signature_header, WebhookDeliveryWorkerConfig,
    },
};
use serde_json::json;
use sqlx::{PgPool, Row};
use tokio::net::TcpListener;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, Clone, Default)]
struct ReceivedRequest {
    headers: HeaderMap,
    body: Vec<u8>,
}

#[derive(Clone)]
struct ReceiverState {
    requests: Arc<Mutex<Vec<ReceivedRequest>>>,
    status: StatusCode,
}

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

async fn repository_fixture(pool: &PgPool, prefix: &str) -> (Uuid, Uuid) {
    let unique = Uuid::new_v4();
    let owner = upsert_user_by_email(
        pool,
        &format!("{prefix}-owner-{unique}@opengithub.local"),
        Some("Webhook Worker Owner"),
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
            visibility: RepositoryVisibility::Private,
            default_branch: Some("main".to_owned()),
            created_by_user_id: owner.id,
        },
    )
    .await
    .expect("repository should create");

    (owner.id, repository.id)
}

async fn start_receiver(status: StatusCode) -> (SocketAddr, Arc<Mutex<Vec<ReceivedRequest>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let state = ReceiverState {
        requests: Arc::clone(&requests),
        status,
    };
    let app = Router::new()
        .route("/hook", post(record_request))
        .with_state(state);
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("receiver should bind");
    let addr = listener.local_addr().expect("receiver address");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("receiver should run");
    });
    (addr, requests)
}

async fn record_request(
    State(state): State<ReceiverState>,
    request: Request<Body>,
) -> impl IntoResponse {
    let headers = request.headers().clone();
    let body = to_bytes(request.into_body(), usize::MAX)
        .await
        .expect("receiver body should read")
        .to_vec();
    state
        .requests
        .lock()
        .expect("receiver lock")
        .push(ReceivedRequest { headers, body });
    (state.status, "receiver recorded")
}

#[tokio::test]
async fn webhook_worker_filters_signs_records_and_retries_deliveries() {
    let Some(pool) = database_pool().await else {
        eprintln!("skipping webhook worker scenario; set TEST_DATABASE_URL");
        return;
    };

    let (success_addr, success_requests) = start_receiver(StatusCode::ACCEPTED).await;
    let (failure_addr, failure_requests) = start_receiver(StatusCode::SERVICE_UNAVAILABLE).await;
    let (owner_id, repository_id) = repository_fixture(&pool, "webhook-worker").await;
    let success_url = format!("http://{success_addr}/hook");
    let failure_url = format!("http://{failure_addr}/hook");

    let push_hook = create_webhook(
        &pool,
        CreateWebhook {
            repository_id,
            actor_user_id: owner_id,
            url: success_url,
            secret_hash: Some("secret:v1:dGVzdC1zZWNyZXQ=".to_owned()),
            events: vec!["push".to_owned()],
        },
    )
    .await
    .expect("push hook should create");
    let inactive_hook = create_webhook(
        &pool,
        CreateWebhook {
            repository_id,
            actor_user_id: owner_id,
            url: format!("http://{success_addr}/hook"),
            secret_hash: None,
            events: vec!["push".to_owned()],
        },
    )
    .await
    .expect("inactive hook should create");
    sqlx::query("UPDATE webhooks SET active = false WHERE id = $1")
        .bind(inactive_hook.id)
        .execute(&pool)
        .await
        .expect("inactive hook should update");
    create_webhook(
        &pool,
        CreateWebhook {
            repository_id,
            actor_user_id: owner_id,
            url: format!("http://{success_addr}/hook"),
            secret_hash: None,
            events: vec!["issues".to_owned()],
        },
    )
    .await
    .expect("issues-only hook should create");

    let queued = enqueue_repository_webhook_event(
        &pool,
        repository_id,
        "push",
        json!({ "ref": "refs/heads/main", "after": "abc123" }),
    )
    .await
    .expect("push delivery should enqueue");
    assert_eq!(queued.len(), 1, "only the active push subscription queues");
    assert_eq!(queued[0].webhook_id, push_hook.id);

    let delivered = run_webhook_delivery_once_with_config(
        &pool,
        queued[0].delivery_id,
        "webhook-worker-success",
        &WebhookDeliveryWorkerConfig {
            timeout: Duration::from_secs(2),
            ..WebhookDeliveryWorkerConfig::default()
        },
    )
    .await
    .expect("worker should run")
    .expect("delivery should process");
    assert_eq!(delivered.status, DeliveryStatus::Delivered);
    assert_eq!(delivered.response_status, Some(202));
    assert_eq!(delivered.attempt_count, 1);

    let received = success_requests
        .lock()
        .expect("receiver requests")
        .first()
        .cloned()
        .expect("receiver should see one request");
    assert_eq!(
        received
            .headers
            .get("x-github-event")
            .expect("event header")
            .to_str()
            .expect("event header text"),
        "push"
    );
    assert_eq!(
        received
            .headers
            .get("x-hub-signature-256")
            .expect("signature header")
            .to_str()
            .expect("signature header text"),
        signature_header("test-secret", &received.body)
    );
    assert!(String::from_utf8_lossy(&received.body).contains("refs/heads/main"));

    let delivery_row = sqlx::query(
        r#"
        SELECT request_headers, request_body_excerpt, response_headers, response_body,
               duration_ms, terminal_error
        FROM webhook_deliveries
        WHERE id = $1
        "#,
    )
    .bind(delivered.id)
    .fetch_one(&pool)
    .await
    .expect("delivery should reload");
    assert!(delivery_row
        .get::<serde_json::Value, _>("request_headers")
        .to_string()
        .contains("x-hub-signature-256"));
    assert!(delivery_row
        .get::<String, _>("request_body_excerpt")
        .contains("refs/heads/main"));
    assert!(delivery_row
        .get::<serde_json::Value, _>("response_headers")
        .is_object());
    assert_eq!(
        delivery_row
            .get::<Option<String>, _>("response_body")
            .as_deref(),
        Some("receiver recorded")
    );
    assert!(delivery_row.get::<Option<i64>, _>("duration_ms").is_some());
    assert!(delivery_row
        .get::<Option<String>, _>("terminal_error")
        .is_none());

    let failing_hook = create_webhook(
        &pool,
        CreateWebhook {
            repository_id,
            actor_user_id: owner_id,
            url: failure_url,
            secret_hash: None,
            events: vec!["workflow_run".to_owned()],
        },
    )
    .await
    .expect("failing hook should create");
    let failing_delivery = enqueue_repository_webhook_event(
        &pool,
        repository_id,
        "workflow_run",
        json!({ "workflow": "ci", "status": "completed" }),
    )
    .await
    .expect("workflow delivery should enqueue")
    .into_iter()
    .find(|delivery| delivery.webhook_id == failing_hook.id)
    .expect("failing delivery should queue");

    let retryable = run_webhook_delivery_once_with_config(
        &pool,
        failing_delivery.delivery_id,
        "webhook-worker-retry",
        &WebhookDeliveryWorkerConfig {
            timeout: Duration::from_secs(2),
            max_attempts: 2,
            ..WebhookDeliveryWorkerConfig::default()
        },
    )
    .await
    .expect("retry worker should run")
    .expect("retryable delivery should process");
    assert_eq!(retryable.status, DeliveryStatus::Queued);
    assert_eq!(retryable.response_status, Some(503));
    assert_eq!(retryable.attempt_count, 1);
    assert!(retryable.next_attempt_at.is_some());
    assert_eq!(
        failure_requests.lock().expect("failure requests").len(),
        1,
        "failed receiver should be called once"
    );

    sqlx::query("UPDATE webhook_deliveries SET next_attempt_at = now() WHERE id = $1")
        .bind(failing_delivery.delivery_id)
        .execute(&pool)
        .await
        .expect("delivery retry should be due");
    sqlx::query(
        "UPDATE job_leases SET locked_until = NULL WHERE queue = 'webhook-delivery' AND lease_key = $1",
    )
    .bind(failing_delivery.delivery_id.to_string())
    .execute(&pool)
    .await
    .expect("job retry lease should release");

    let terminal = run_webhook_delivery_once_with_config(
        &pool,
        failing_delivery.delivery_id,
        "webhook-worker-retry",
        &WebhookDeliveryWorkerConfig {
            timeout: Duration::from_secs(2),
            max_attempts: 2,
            ..WebhookDeliveryWorkerConfig::default()
        },
    )
    .await
    .expect("terminal worker should run")
    .expect("terminal delivery should process");
    assert_eq!(terminal.status, DeliveryStatus::Failed);
    assert_eq!(terminal.attempt_count, 2);
    assert_eq!(terminal.response_status, Some(503));

    let completed_at = sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
        "SELECT completed_at FROM job_leases WHERE queue = 'webhook-delivery' AND lease_key = $1",
    )
    .bind(failing_delivery.delivery_id.to_string())
    .fetch_one(&pool)
    .await
    .expect("job lease should load");
    assert!(
        completed_at.is_some(),
        "terminal failures complete the job lease"
    );

    let payload_too_large = enqueue_repository_webhook_event(
        &pool,
        repository_id,
        "push",
        json!({ "body": "x".repeat(128) }),
    )
    .await
    .expect("large payload delivery should queue")
    .into_iter()
    .find(|delivery| delivery.webhook_id == push_hook.id)
    .expect("large delivery should target push hook");
    let oversized = run_webhook_delivery_once_with_config(
        &pool,
        payload_too_large.delivery_id,
        "webhook-worker-large",
        &WebhookDeliveryWorkerConfig {
            timeout: Duration::from_secs(2),
            max_payload_bytes: 16,
            ..WebhookDeliveryWorkerConfig::default()
        },
    )
    .await
    .expect("large payload worker should run")
    .expect("large payload delivery should process");
    assert_eq!(oversized.status, DeliveryStatus::Failed);
    assert_eq!(oversized.response_status, None);
    let terminal_error = sqlx::query_scalar::<_, Option<String>>(
        "SELECT terminal_error FROM webhook_deliveries WHERE id = $1",
    )
    .bind(payload_too_large.delivery_id)
    .fetch_one(&pool)
    .await
    .expect("terminal error should load");
    assert_eq!(terminal_error.as_deref(), Some("payload_too_large"));

    let invalid_event = enqueue_repository_webhook_event(
        &pool,
        repository_id,
        "not_an_event",
        json!({ "ignored": true }),
    )
    .await;
    assert!(
        invalid_event.is_err(),
        "event adapters must reject unsupported webhook events"
    );
}
