# Structure Outline: settings-004 Repository Webhooks

**Ticket**: `settings-004`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/wf-settings.jsx` (`Webhooks_A`, `Webhooks_B`), existing repository settings shell in `web/src/components/RepositorySettingsShell.tsx`, current hooks placeholder in `web/src/app/[owner]/[repo]/settings/hooks/page.tsx`, existing webhook foundation in `crates/api/migrations/202604300004_automation_delivery.up.sql`, `crates/api/src/domain/webhooks.rs`, `crates/api/tests/automation_delivery_foundation.rs`, repository settings audit patterns from `settings-001` through `settings-003`, and GitHub webhook docs in `target-docs/content/webhooks/`.
**Date**: 2026-05-03

## Phase 1: Webhook Settings API Contract - hooks, deliveries, validation, and audit events

**Done**: [x]

**Scope**: Extend the existing Rust/Postgres webhook foundation into an admin-only repository settings API under `/api/repos/{owner}/{repo}/settings/hooks`. This phase should preserve the current `webhooks` / `webhook_deliveries` tables and domain helpers while adding the missing settings contract: content type, SSL verification, write-only secret metadata, event selection mode, delivery summaries, ping creation, manual redelivery records, retention metadata, and audit events.

**Key changes**:
- `crates/api/migrations/`: add additive columns/tables as needed for `content_type`, `ssl_verify`, secret write-only metadata/hash, event selection mode, disabled reason, delivery GUID, request/response headers, body storage keys, duration, redelivery linkage, last delivery summary, and delivery retention/attempt metadata. Keep existing `webhooks.url`, `events`, `active`, and existing delivery status values compatible.
- Add DTOs for `RepositoryWebhookSettings`, `RepositoryWebhookSummary`, `RepositoryWebhookDetail`, `WebhookDeliverySummary`, `WebhookDeliveryDetail`, `WebhookEventDefinition`, `WebhookMutation`, `WebhookPingResult`, and structured validation errors.
- `GET /api/repos/{owner}/{repo}/settings/hooks`: repository admins can list/edit hooks; non-admin readers should receive a non-leaky forbidden response; anonymous/private repository access follows existing repository privacy behavior.
- `GET /api/repos/{owner}/{repo}/settings/hooks/{hook_id}` and `/deliveries/{delivery_id}`: return hook detail and delivery request/response panels without returning plaintext secrets or unbounded bodies.
- Mutation endpoints: create/update/delete hooks, ping a hook, and redeliver a delivery. Validate HTTPS payload URLs for create/update, supported content types, non-empty event selections, valid individual event names, secret length bounds, SSL verification boolean, active flag, missing hooks/deliveries, and redelivery ownership.
- Successful writes insert `repository_settings_audit_events` with changed fields and before/after metadata while never storing plaintext secrets in audit JSON.
- Create a queued `ping` delivery on hook creation and manual ping, and create a new delivery linked to the original for redelivery.
- Add `crates/api/tests/repository_webhook_settings_contract.rs` covering admin-only access, create/update/delete, HTTPS validation, content type/event validation, secret write-only behavior, ping/redelivery rows, delivery detail redaction, audit events, private repository privacy, and structured errors without stack/env leakage.

**Verification**: focused `repository_webhook_settings_contract` against `TEST_DATABASE_URL`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check && DB_SSL=false make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Webhooks List and Detail Shell - empty state, hook rows, and delivery panels

**Done**: [x]

**Scope**: Replace the `/[owner]/[repo]/settings/hooks` placeholder with a real Editorial Webhooks settings surface backed by the Phase 1 API. The page must cover the empty state, hook list, hook detail, and recent delivery detail panels without GitHub/Primer visual regression.

**Key changes**:
- `web/src/lib/api.ts` and server-session helpers: add typed webhook settings DTOs and cookie-backed fetch helpers preserving forbidden/unavailable states.
- `web/src/app/[owner]/[repo]/settings/hooks/page.tsx`: fetch settings server-side and render concrete content inside `RepositorySettingsShell`.
- Add `web/src/app/[owner]/[repo]/settings/hooks/[hookId]/page.tsx` for hook detail and recent deliveries, plus URL-backed selection for a specific delivery when useful.
- Add `web/src/components/RepositoryWebhookSettingsPage.tsx` and supporting detail components with hook count summary, empty state with working Add webhook CTA, hook list rows showing payload URL, active/state chip, event summary, latest delivery status, updated time, and concrete Edit/Delete/Test controls.
- Hook detail should include an Editorial tab layout for configuration and Recent deliveries, GUID rows, event/status/duration, request headers/body, response headers/body, and Redeliver controls.
- Non-admin/forbidden states must not leak private hook URLs or secrets; unavailable states must offer a concrete retry/navigation path.
- Use only Editorial primitives and tokens: `.card`, `.btn`, `.chip`, `.input`, `.tabs`, `.list-row`, `.t-label`, `.t-mono-sm`, `var(--ink-*)`, `var(--line)`, `var(--accent)`, and semantic chips. Do not introduce GitHub colors, Octicons, Primer imports, nested cards, or dead `href="#"` links.
- Add focused `web/tests/repository-webhook-settings-page.test.tsx` coverage for empty state, hook rows, detail delivery panels, forbidden/unavailable states, no inert anchors/buttons, and Editorial primitive/token usage.

**Verification**: focused Vitest for the page, mandatory Editorial banned-value scan, then `make check && make test`. Save a browser screenshot if the E2E seed already exposes webhook fixture data.

---

## Phase 3: Add/Edit/Delete/Test Webhook Mutations - confirmed forms and no dead controls

**Done**: [x]

**Scope**: Wire every visible webhook settings action to real server-confirmed writes. Admins must be able to create, edit, delete, ping-test, and redeliver hooks from the UI with inline success/error feedback, while invalid inputs never update local UI state optimistically.

**Key changes**:
- Add same-origin Next.js route handlers or server actions under the hooks settings route that forward authenticated mutations to the Rust API without adding JS-side auth.
- Implement Add/Edit webhook form: Payload URL, Content type radio/select, Secret password field with write-only helper text, SSL verification checkbox, event selection radios (`push`, `everything`, `selected`), individual event checkbox grid, Active checkbox, and Add/Update buttons.
- Event checkboxes render only when individual selection is active. Selected-event validation should happen client-side for fast feedback and server-side for authority.
- Implement delete confirmation requiring hook URL or hook identifier confirmation; delete must refresh from server-confirmed state.
- Implement Test delivery/Ping action and delivery Redeliver action with loading states, delivery result links, and server-confirmed delivery row creation.
- Extend Rust contract tests for update/delete/ping/redelivery audit rows, duplicate redelivery linkage, invalid URL downgrade attempts, retained secret behavior when the secret field is blank on edit, and secret replacement when provided.
- Extend Vitest coverage for forms, validation, write-only secret semantics, delete confirmation, ping/redeliver actions, disabled non-admin actions, and no local-only state changes.
- Add `web/tests/e2e/repository-settings-hooks.spec.ts`: seed an admin repository, create a webhook, verify redirect/detail/ping delivery, edit events/active flag, redeliver a seeded failed delivery, delete a disposable hook, reload to verify persistence, and save `ralph/screenshots/build/settings-004-phase3-hooks-mutations.jpg`.

**Verification**: focused Rust contract, focused Vitest, focused Playwright smoke, mandatory Editorial banned-value scan, then `make check && make test`; run `make test-e2e` when local database/dev servers are stable.

---

## Phase 4: Delivery Worker Integration - signing, retries, bounded payloads, and event sources

**Done**: [x]

**Scope**: Make webhook delivery operational beyond settings rows. Repository events should enqueue deliveries for subscribed active hooks, workers should deliver signed HTTP requests with bounded timeouts/retries, and delivery history should record request/response metadata without leaking secrets or unbounded bodies.

**Key changes**:
- Add or extend a Rust worker/job module for `webhook-delivery` leases that reads queued deliveries, builds payloads, signs with the hook secret using an HMAC header, sends HTTP requests with a bounded timeout, records response status/duration/headers/body excerpts or S3 storage keys, and schedules bounded retries.
- Enqueue webhook deliveries from existing repository activity sources where available: push/Git receive-pack, issue events, pull request events, release/package/workflow events, and manual ping. If some event families are not implemented yet, add explicit event-source adapters with tests and QA hints rather than fake data.
- Enforce payload size limits before storing/sending; large request/response bodies should use S3-modeled storage keys consistent with the configured AWS storage contract.
- Preserve event authorization: only active hooks subscribed to the event get queued; inactive hooks do not receive deliveries; deleted hooks cascade delivery history according to schema decisions.
- Add delivery status transitions for queued, delivered, failed, and retryable attempts if the existing status set is insufficient; keep API responses backward-compatible where possible.
- Add observability fields or structured logs for hook id, delivery id, event, attempt count, duration, terminal status, and redacted error category.
- Extend Rust tests for event subscription filtering, inactive hook skipping, signature generation, timeout/retry scheduling, payload-size handling, response redaction, retention bounds, and redelivery preserving original linkage.
- Add frontend assertions that delivery panels show worker-recorded status, retry count, duration, and redelivery lineage.

**Verification**: focused worker/domain tests, focused webhook settings tests, then `make check && make test`. Run focused Playwright for hook detail delivery history when seeded worker data is available.

---

## Phase 5: Guardrails, API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [x]

**Scope**: Finish `settings-004` as a complete vertical slice only after API, UI, mutation flows, worker delivery behavior, docs, screenshots, and QA handoff are verified. This phase should not add unrelated repository settings areas such as Pages, Actions secrets, protected tags, or security analysis.

**Key changes**:
- Finalize API docs in `web/src/lib/api-docs.ts` for webhook list/detail, create/update/delete, ping, delivery detail, and redelivery endpoints, including auth/visibility, write-only secret behavior, supported events, validation errors, and delivery status semantics.
- Extend `qa-hints.json` with deeper QA targets: real external receiver success/failure, TLS/SSL verification behavior, HMAC signature verification, retry backoff, payload/body truncation, concurrent admin edits, secret rotation, event subscription edge cases, private repository leakage checks, and retention cleanup.
- Ensure every visible button/link/form has concrete behavior or an accessible disabled state; verify keyboard navigation through Add/Edit/Delete, event selection, delivery detail panels, ping, and redelivery controls.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/settings-004-final-hooks-*.jpg` for empty/list/detail/admin and forbidden/read-only states.
- Run the mandatory Editorial banned-value scan before commit and fix any touched-file regressions.
- Update `build-progress.txt`, `.qrspi/settings-004/structure.md`, and `prd.json`; set `settings-004.build_pass=true` only after all implementation phases are complete and verified; leave `qa_pass=false`.

**Verification**: focused Rust/Vitest/Playwright checks, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, full same-env `make test-e2e` when available, browser smoke screenshots under `ralph/screenshots/build/`, and mandatory Editorial banned-value scan with zero matches.
