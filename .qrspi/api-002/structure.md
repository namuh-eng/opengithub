# api-002 — Webhook catalog with signed deliveries

## Actual implementation structure

- **Schema**: `crates/api/migrations/202605030042_webhook_catalog.*.sql` extends the automation-delivery foundation to the PRD shape: `scope_type`, `scope_id`, `content_type`, `secret_ciphertext`, `ssl_verify`, signed request headers/body, response headers/body, duration, redelivery linkage, and nullable repository scope for org hooks.
- **Backend domain**: `crates/api/src/domain/webhooks.rs` owns the supported event catalog, URL/event validation, repo/org admin authorization, HMAC-SHA256 over raw request bytes, delivery persistence, outbound POST dispatch, redelivery, and 3-attempt exponential retry metadata.
- **Backend routes**: `crates/api/src/routes/webhooks.rs` exposes authenticated repo and org webhook catalog/create/toggle/redeliver/dispatch endpoints; `crates/api/src/main.rs` starts a periodic due-delivery worker when DB is available.
- **Next API bridge**: `web/src/app/api/repos/[owner]/[repo]/hooks/**` and `web/src/app/api/orgs/[org]/hooks/**` forward client mutations to the Rust API with cookie auth.
- **UI**: `web/src/components/WebhooksSettingsPage.tsx` powers `/<owner>/<repo>/settings/hooks` and `/organizations/<org>/settings/hooks` with Editorial cards/table/form/event checklist/recent delivery viewer/redelivery controls and no dead buttons.
- **Tests**: `crates/api/tests/api_webhooks_contract.rs` covers supported events and HMAC raw-body signing; `web/tests/webhooks-settings-page.test.tsx` covers catalog/table/viewer, create payloads, and active toggles.

## Verification evidence

- `cd web && npx tsc --noEmit --pretty false` passed.
- Focused `cd web && npx vitest run tests/webhooks-settings-page.test.tsx` passed (2 tests).
- Focused `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo test -p opengithub-api --test api_webhooks_contract` passed (2 tests).
- Mandatory Editorial banned-value scan on touched web files passed with zero matches.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 make check` passed: Cargo check, web typecheck, Clippy, Biome.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 make test` passed: Cargo tests and 269 web tests.

## QA focus

- Exercise a real local receiver to verify `X-GitHub-Event`, UUID `X-GitHub-Delivery`, and `X-Hub-Signature-256` against captured raw bytes.
- Verify org-admin and repo-admin authorization with non-admin users.
- Verify retry timing with a receiver returning 5xx, then redelivery from the Recent deliveries panel.
