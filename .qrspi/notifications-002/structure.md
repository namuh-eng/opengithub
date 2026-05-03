# Structure Outline: notifications-002 Notification Triage Actions

**Ticket**: `notifications-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/og-screens-1.jsx`, existing inbox code in `crates/api/src/domain/notifications.rs`, `crates/api/src/routes/notifications.rs`, `web/src/components/NotificationsInboxPage.tsx`, `web/src/lib/api.ts`, `web/tests/notifications-inbox.test.tsx`, `crates/api/tests/api_notifications_inbox_contract.rs`, and notification triage docs in `target-docs/content/subscriptions-and-notifications/how-tos/viewing-and-triaging-notifications/`.
**Date**: 2026-05-03

## Phase 1: Persisted Read/Unread and Saved State

**Done**: [ ]

**Scope**: Add the first server-backed triage mutations beyond the existing single mark-read endpoint: mark read, mark unread, save, and unsave for one notification. The inbox should reflect saved counts and saved-folder filtering from persisted data, while row-level controls provide server-confirmed success/error feedback.

**Key changes**:
- `crates/api/migrations/`: add `notifications.saved boolean NOT NULL DEFAULT false`, `notifications.saved_at timestamptz`, and indexes for `(user_id, saved, updated_at DESC)` without changing existing notification identity.
- `crates/api/src/domain/notifications.rs`: replace hardcoded saved filtering with real `notifications.saved`, add a `NotificationTriageAction` enum or focused helpers for `read`, `unread`, `save`, and `unsave`, and return a compact row/count response suitable for optimistic UI reconciliation.
- `crates/api/src/routes/notifications.rs`: expose authenticated PATCH routes such as `/api/notifications/:id/read`, `/unread`, `/save`, and `/unsave` with the existing error envelope and ownership checks.
- `web/src/lib/api.ts`: add typed notification action response helpers and same-origin action calls that preserve cookies.
- `web/src/components/NotificationsInboxPage.tsx`: introduce a small client island for row actions using Editorial icon buttons/tooltips, inline pending/success/error state, and rollback after failed mutations while preserving row open navigation.
- Tests: extend `api_notifications_inbox_contract`, `web/tests/notifications-inbox.test.tsx`, and focused Playwright notifications smoke for read/unread/save/unsave, saved folder counts, no inert row controls, and mobile no-overflow.

**Verification**: focused Rust notifications contract, focused Vitest notifications tests, focused Playwright notification smoke with screenshot under `ralph/screenshots/build/`, mandatory Editorial banned-value scan, then `make check && make test`.

---

## Phase 2: Done and Move-to-Inbox Lifecycle

**Done**: [ ]

**Scope**: Implement GitHub-style Done triage. Marking Done should remove rows from Inbox, show them in Done, preserve read/saved state, and allow Move to inbox to restore the notification to the active inbox.

**Key changes**:
- `crates/api/migrations/`: add `notifications.done_at timestamptz` plus indexes for active inbox (`done_at IS NULL`) and done-folder queries.
- `crates/api/src/domain/notifications.rs`: make `folder=inbox` exclude done rows by default, make `folder=done` include only done rows, add `done` and `inbox` mutation helpers, and update folder facet counts to use real persisted inbox/saved/done totals.
- `crates/api/src/routes/notifications.rs`: expose PATCH `/api/notifications/:id/done` and `/api/notifications/:id/inbox`, returning updated folder/count state.
- `web/src/components/NotificationsInboxPage.tsx`: row Done and Move to inbox controls should optimistically remove or restore rows for the current folder, update selected counts, and show an inline toast on rollback.
- Tests: cover done rows disappearing from inbox, appearing in Done, Move to inbox restoring visibility, unread count remaining correct, saved+done overlap, and direct notification open still marking read.

**Verification**: focused Rust and Vitest notification tests, Playwright smoke for Done and Move to inbox from `/notifications` and `/notifications?folder=done`, mandatory banned-value scan, then `make check && make test`.

---

## Phase 3: Subscribe and Unsubscribe Triage

**Done**: [ ]

**Scope**: Connect row-level Subscribe/Unsubscribe actions to repository/thread subscription state. Unsubscribe should remove the notification from the inbox and suppress future thread notifications until participation, mention, team mention, or review request re-subscribes the user.

**Key changes**:
- `crates/api/migrations/`: add `notification_threads` and `notification_subscriptions` if absent, keyed by subject identity with `state` values such as `subscribed`, `unsubscribed`, and `participating`; backfill existing rows from repository/subject data where possible.
- `crates/api/src/domain/notifications.rs`: resolve thread identity from `repository_id`, `subject_type`, and `subject_id`; add subscribe/unsubscribe mutations; make inbox queries hide unsubscribed threads unless the row reason is a reactivation reason; preserve repository watch compatibility for repository-level subscribed chips.
- Notification emitters in `issues.rs`, `pulls.rs`, Actions, releases, and imports: consult thread subscription state and re-subscribe on participation, direct mention, team mention, or review request.
- `crates/api/src/routes/notifications.rs`: expose PATCH `/api/notifications/:id/subscribe` and `/unsubscribe` with ownership and subject validation.
- `web/src/components/NotificationsInboxPage.tsx`: row action toggles should switch subscribed state, remove unsubscribed rows from Inbox, and show clear rollback feedback.
- Tests: cover subscription persistence, unsubscribe hiding only the target thread, reactivation reasons restoring delivery, repository watch fallback, forbidden cross-user mutation, and no metadata leakage for inaccessible private repos.

**Verification**: focused Rust subscription/delivery contract tests, focused Vitest row action tests, Playwright smoke for unsubscribe/subscribe state, mandatory banned-value scan, then `make check && make test`.

---

## Phase 4: Bulk and Group-Level Triage Toolbar

**Done**: [ ]

**Scope**: Add multi-select, group select-all, and bulk actions for Done, Unsubscribe, Save/Unsave, Mark read/unread, and Move to inbox. Bulk actions should update counts optimistically, preserve navigation for row links, and rollback only the failed rows if the API returns partial failure details.

**Key changes**:
- `crates/api/src/domain/notifications.rs`: add a bulk action input type with bounded notification IDs, action enum, per-row authorization, and a response containing updated IDs, failed IDs, fresh folder counts, and unread count.
- `crates/api/src/routes/notifications.rs`: expose `POST /api/notifications/bulk` or equivalent with validation for empty selections, duplicate IDs, excessive batch size, unsupported actions, and cross-user IDs.
- `web/src/lib/api.ts`: add typed bulk triage request/response helpers.
- `web/src/components/NotificationsInboxPage.tsx`: move selection state into the client island, add row checkboxes, visible/group select-all, sticky or inline bulk toolbar with selected count, action buttons, pending states, and accessible labels. Keep Editorial primitives and icon-style controls rather than GitHub/Primer chrome.
- Tests: cover selecting rows without navigating, group select-all, toolbar appearance/disappearance, every bulk action, partial failure rollback, empty-state recovery, keyboard traversal, and no dead controls.

**Verification**: focused Rust bulk contract tests, focused Vitest interaction tests with Testing Library user events, focused Playwright smoke for row/group/bulk flows and screenshot, mandatory banned-value scan, then `make check && make test`; run `make test-e2e` when the local migration state allows.

---

## Phase 5: API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `notifications-002` only after all single, subscription, and bulk triage actions are working end-to-end. This phase should document the mutation contract, add QA focus notes, capture final browser evidence, and flip only `notifications-002.build_pass`.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document notification inbox query fields, row action endpoints, bulk action endpoint, response shapes, error/partial-failure behavior, subscription reactivation rules, and retention notes for saved/done notifications.
- `qa-hints.json`: append focused QA targets for concurrent bulk mutations, failed-row rollback, subscription reactivation triggers, saved+done retention, keyboard selection, private repository leakage, mobile overflow, and screen reader labels for icon controls.
- `web/tests/e2e/notifications.spec.ts`: broaden smoke coverage for all row and bulk controls, all folders, selected toolbar behavior, rollback toast, no dead anchors/buttons, and save screenshots under `ralph/screenshots/build/`.
- `build-progress.txt`, `.qrspi/notifications-002/structure.md`, and `prd.json`: record verification evidence, mark Phase 5 done, and set `notifications-002.build_pass=true` only after full verification passes; leave `qa_pass=false`.

**Verification**: focused Rust notification tests, focused notification/docs Vitest tests, focused Playwright notifications smoke, mandatory Editorial banned-value scan, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, same-env `make test`, and `make test-e2e` when migration state allows. If the shared database hits the known `202605030041` migration checksum issue, document it precisely and include fresh-DB Playwright evidence.
