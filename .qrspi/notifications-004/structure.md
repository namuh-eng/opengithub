# Structure Outline: notifications-004 Repository Watch and Thread Notification Subscriptions

**Ticket**: `notifications-004`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-shell.jsx` repository action bar, `design/project/og-screens-2.jsx`, `design/project/og-screens-3.jsx`, existing repository watch route in `crates/api/src/routes/repositories.rs`, current `repository_watches` schema, generic notification thread/subscription model from `notifications-002`, issue/PR subscription state in `crates/api/src/domain/issues.rs` and `crates/api/src/domain/pulls.rs`, `RepositoryHeaderActions`, issue/PR sidebar notification cards, and target docs under `target-docs/content/rest/activity/watching.md`, `target-docs/content/rest/activity/notifications.md`, and `target-docs/content/subscriptions-and-notifications/`.
**Date**: 2026-05-04

## Phase 1: Repository Watch Settings API Contract

**Done**: [x]

**Scope**: Replace the current boolean repository watch mutation with a full repository-level notification settings contract. Signed-in readers can load and save watch levels (`participating`, `all`, `ignore`, `custom`) plus custom event filters. Repository watch count continues to represent users with an active non-ignore watch state, and private repositories must not leak to unauthorized users.

**Key changes**:
- `crates/api/migrations/`: extend `repository_watches` from the current `reason`-only model to durable `level`, `custom_events`, `ignored_at`, and metadata fields, or add a companion table if that is safer for existing dashboard feed reads. Preserve current rows as `participating`.
- `crates/api/src/domain/repositories.rs`: add DTOs for `RepositoryWatchSettings`, `RepositoryWatchLevel`, custom event categories (`issues`, `pull_requests`, `releases`, `discussions`, `actions`, `security_alerts`, `repository_invitations`), warning copy for ignore, and update helpers with validation.
- `crates/api/src/routes/repositories.rs`: add `GET /api/repos/{owner}/{repo}/watch` and `PATCH /api/repos/{owner}/{repo}/watch` while keeping existing `PUT/DELETE /watch` compatible as participating/unwatch aliases until the frontend is migrated.
- Update repository overview DTOs so `viewerState` exposes a normalized watch label and selected custom events without exposing private repository data to unauthenticated or unauthorized callers.
- Fanout preparation: add small domain helpers that answer "should this user receive this repository event?" using repository watch level, custom events, permissions, organization/team visibility, and ignore semantics. Do not wire every event producer yet.
- Tests: add/extend Rust repository watch contract coverage for read gates, default participating migration behavior, all/custom/ignore saves, invalid event validation, count semantics, compatibility `PUT/DELETE`, and error envelopes.

**Verification**: focused Rust repository watch contract, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and `DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Repository Watch Menu

**Done**: [x]

**Scope**: Replace the simple Watch/Unwatch button in the repository header with the Editorial watch menu described by the PRD. The menu must show current count, radio choices, concise descriptions, keyboard hints, an ignore confirmation warning, and custom event checkboxes backed by the Phase 1 API.

**Key changes**:
- `web/src/lib/api.ts`: add typed repository watch settings fetch/update helpers plus same-origin route handlers under `web/src/app/[owner]/[repo]/actions/watch` that forward authenticated `GET/PATCH/PUT/DELETE` calls to Rust.
- `web/src/components/RepositoryHeaderActions.tsx`: split watch behavior into a dedicated client component with a menu/popover, radio options for Participating and @mentions, All Activity, Ignore, and Custom, event checkboxes for the custom state, save/cancel flows, pending/error/success state, and rollback from server-confirmed responses.
- Keep the Editorial system locked: `.btn`, `.chip`, `.card`, `.input`, `.t-label`, `.t-xs`, `var(--ink-*)`, `var(--line)`, `var(--accent)`, semantic warning chip for ignore, no GitHub colors, no Primer/Octicons, no dead `href="#"`.
- Preserve header layout across repository Code, Issues, Pulls, Actions, Releases, Packages, Settings, and mobile widths; the watch menu must not resize the action bar or overlap tabs.
- Tests: extend repository overview Vitest and Playwright smoke for opening/closing the menu, keyboard traversal, each radio option, custom checkbox save, ignore warning, count/label reconciliation after reload, unauthenticated sign-in affordance where relevant, no inert controls, and mobile no-overflow.

**Verification**: focused repository header Vitest, focused repository-code Playwright smoke with screenshot under `ralph/screenshots/build/notifications-004-phase2-watch-menu.jpg`, mandatory Editorial banned-value scan, then `make check && make test`.

---

## Phase 3: Thread-Level Customize Dialog for Issues and Pull Requests

**Done**: [x]

**Scope**: Upgrade the issue and pull request sidebar notification cards from basic Subscribe/Unsubscribe buttons to real thread-level subscription controls. Users can subscribe, unsubscribe, and customize state-change events for the specific issue/PR without mutating repository-wide watch settings.

**Key changes**:
- `crates/api/migrations/`: extend `issue_subscriptions`, `pull_request_subscriptions`, or the generic `notification_subscriptions` table with thread-level custom event preferences for `closed`, `reopened`, `merged`, and related state changes. Prefer converging issue/PR-specific tables with the generic notification thread model where it avoids duplicated fanout logic.
- `crates/api/src/domain/issues.rs` and `crates/api/src/domain/pulls.rs`: extend detail DTO subscription state with `subscribed`, `reason`, `customEvents`, and `canCustomize`; add update helpers for subscribe, unsubscribe, and customize requests. Preserve participation/mention reactivation rules.
- Route layer: expose authenticated issue and pull request subscription update endpoints that accept `subscribed` plus optional `customEvents`, return normalized subscription state, and enforce repository read permission.
- `RepositoryIssueDetailPage` and `RepositoryPullRequestDetailPage`: replace the static sidebar card with an Editorial Notifications card that includes Subscribe/Unsubscribe, Customize dialog, event checkboxes, inline success/error feedback, disabled unauthenticated sign-in CTA, and confirmed server reload behavior.
- Tests: extend issue and PR Rust contracts, Vitest detail-page tests, and Playwright detail smokes for subscribe/unsubscribe/customize, reload persistence, closed/reopened/merged selections, unauthorized redaction, focus trap/Escape close, and no dead controls.

**Verification**: focused issue and PR subscription Rust contracts, focused issue/PR detail Vitest, focused Playwright issue and PR detail smokes with screenshots, mandatory banned-value scan, then `make check && make test`.

---

## Phase 4: Notification Fanout Integration for Watch Levels and Thread Events

**Done**: [x]

**Scope**: Wire repository watch levels, custom repository event categories, ignore, and thread-level custom events into notification creation paths. This phase proves future notifications are delivered or suppressed according to saved settings for issues, pull requests, releases, Actions runs, security alerts, repository invitations, comments, assignments, mentions, review requests, and state changes already supported by the app.

**Key changes**:
- Add a centralized notification audience resolver in `crates/api/src/domain/notifications.rs` or a new focused domain module. It should combine repository permissions, repository watch settings, thread subscription overrides, participation, direct mentions, team mentions, review requests, author exclusion, ignore suppression, and custom event filters.
- Update event producers in issues, pulls, releases, Actions, repository invitations/access, and security/code scanning surfaces where present to call the resolver before `create_notification`.
- Ensure thread-level unsubscribe wins over repository `all/custom` until participation/mention/review-request reactivation occurs; repository `ignore` suppresses repository-watch delivery but cannot suppress direct participation/mention security-critical reactivation if the product contract requires it.
- Add audit or diagnostic metadata only where already established; do not store plaintext secret values, private team lists, or inaccessible repository names in notification rows.
- Tests: add Rust fanout contract coverage for all watch levels, custom event inclusion/exclusion, ignore warning effect, issue/PR thread custom state-change events, mention/review-request reactivation, team mention membership, private repository leakage, and duplicate recipient de-duping.

**Verification**: focused notification fanout Rust contracts, existing issue/PR/release/Actions contract reruns where changed, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and `DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional unless UI state changes are touched in this phase.

---

## Phase 5: API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `notifications-004` only after repository watch menu, issue/PR thread subscription customization, fanout integration, docs, screenshots, and QA hints are verified. This phase flips only `notifications-004.build_pass` and leaves `qa_pass=false`.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document repository watch read/update endpoints, compatibility aliases, watch levels, custom event categories, ignore behavior, issue/PR subscription customize endpoints, fanout precedence, permission rules, response shapes, and error envelopes.
- `qa-hints.json`: append deeper QA targets for concurrent watch saves, ignore-vs-mention precedence, custom event gaps, issue/PR thread overrides, team mentions, review request reactivation, private repository leakage, keyboard traversal, screen reader labels, and mobile menu/dialog placement.
- Broaden Playwright coverage across repository overview, issue detail, and PR detail. Verify every menu item, checkbox, confirmation, dialog, and CTA is live; save final desktop/mobile screenshots under `ralph/screenshots/build/notifications-004-final-*.jpg`.
- Run the mandatory Editorial banned-value scan before commit and fix any touched-file regressions.
- Update `build-progress.txt`, `.qrspi/notifications-004/structure.md`, and `prd.json`; set `notifications-004.build_pass=true` only after all implementation phases are complete and verified.

**Verification**: focused Rust/Vitest/Playwright checks, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, same-env `make test`, full or focused `make test-e2e` when the local migration state allows, final browser screenshots, and mandatory Editorial banned-value scan with zero matches.
