# Structure Outline: releases-002 Release Management Forms and Assets

**Ticket**: `releases-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, existing release read/mutation work in `.qrspi/releases-001/structure.md`, `crates/api/src/domain/releases.rs`, `crates/api/src/routes/repositories.rs`, `web/src/components/RepositoryReleaseManager.tsx`, `web/src/app/[owner]/[repo]/releases/actions/route.ts`, and release references in `target-docs/content/repositories/releasing-projects-on-github/` plus `target-docs/content/rest/releases/`.
**Date**: 2026-05-03

## Existing Baseline

`releases-001` already introduced release read endpoints, release list/detail/tag pages, reactions, download metadata, and a compact inline management component for create/update/publish/delete plus asset metadata. `releases-002` should keep that baseline but complete the dedicated GitHub-like release management workflow from the PRD: `/releases/new`, `/releases/edit/{id}`, generated release notes, tag/target/previous-tag selectors, real upload-intent semantics, explicit latest/draft/prerelease options, and final docs/smoke bookkeeping.

## Phase 1: Management API Hardening - edit context, generated notes, latest policy, and asset upload intents

**Done**: [ ]

**Scope**: Extend the Rust/Postgres release write contract so the dedicated form can load everything it needs and submit all mutations without local-only behavior.

**Key changes**:
- Extend `crates/api/src/domain/releases.rs` with a release management context DTO containing writable permission state, available tags, branches/refs, default target, previous tag candidates, latest-release policy options, immutable/draft constraints, and upload limits.
- Add endpoints under `/api/repos/{owner}/{repo}/releases/manage`: new-release context, edit-release context by release id, generated notes preview, and asset upload intent creation. Keep the existing create/update/publish/delete endpoints but widen `ReleaseMutation` only as needed for latest policy and optional delete-tag intent.
- Generate release notes from commits and merged pull requests between previous tag and target ref, with bounded Markdown output, contributor summaries, and deterministic empty-range messaging. Do not call GitHub APIs.
- Add upload-intent metadata for local/S3-pluggable asset storage: asset name, content type, byte size, checksum, storage key, upload URL or local handoff token, status, and expiry. API responses must never expose raw S3 keys when a signed URL is enough.
- Enforce write permissions, archived repository blocking, immutable release constraints, duplicate tag checks, invalid target refs, invalid previous-tag ranges, asset size/name/content-type validation, and audit rows with storage keys and secrets redacted.
- Queue release/webhook/activity events for publish, delete, asset upload completion, and generated-notes usage where existing webhook delivery patterns support it.
- Add `crates/api/tests/repository_release_management_contract.rs` covering context loading, write gates, generated notes, latest policy, draft/publish semantics, delete-tag opt-in behavior, upload intent validation, storage-key redaction, audit rows, webhook queueing, private repository redaction, and immutable release restrictions.

**Verification**: focused `repository_release_management_contract`, existing `repository_releases_contract`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check && DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Dedicated Editorial Release Form Pages - new/edit routes, selectors, preview, and validation

**Done**: [ ]

**Scope**: Replace the compact inline manager as the primary authoring surface with dedicated Editorial pages at `/{owner}/{repo}/releases/new` and `/{owner}/{repo}/releases/edit/{id}` while keeping read-only release pages clean.

**Key changes**:
- Add App Router pages for `/releases/new` and `/releases/edit/[id]` that fetch the management context server-side through cookie-backed helpers in `web/src/lib/api.ts` and `web/src/lib/server-session.ts`.
- Add `RepositoryReleaseFormPage` and focused subcomponents for tag selection/new tag entry, target branch/ref dropdown, previous tag selector, title, Markdown editor, Preview tab, generated-notes action, latest policy options, draft/prerelease checkboxes, asset upload list, and danger actions.
- Update release list/detail pages so `New release`, `Edit`, `Publish draft`, and `Delete` navigate to or invoke the dedicated management routes; remove or shrink any duplicated inline form behavior that would create two competing authoring experiences.
- Use only Editorial primitives and tokens: `.card`, `.btn`, `.chip`, `.input`, `.tabs`, `.list-row`, `.kbd`, `.t-*`, `var(--ink-*)`, `var(--line)`, and semantic chips. Keep controls dense but calm, with no GitHub palette, Primer imports, Octicons, nested cards, or inert placeholders.
- Provide unavailable/forbidden states for non-writers and private repositories, accessible disabled states for immutable releases, inline validation callouts, keyboard focus order, and responsive layout without text overlap.
- Add focused Vitest coverage for new/edit rendering, permission-gated controls, generated-notes preview state, Markdown preview sanitization, selectors, latest/draft/prerelease options, destructive confirmation UI, no dead controls, and Editorial banned-value guardrails.

**Verification**: focused release form Vitest, `cd web && npx tsc --noEmit --pretty false`, mandatory Editorial banned-value scan, then full `make check && make test`. Add a focused Playwright smoke if the seeded E2E repository can exercise writer permissions.

---

## Phase 3: Server-Confirmed Create/Edit/Publish/Delete Flow - no dead authoring controls

**Done**: [ ]

**Scope**: Wire every visible form action to the Rust API with rollback-on-error semantics and browser-proven user flows.

**Key changes**:
- Extend the same-origin Next.js release action route to forward management context, generated-notes, create, update, publish, save-draft, delete, and optional delete-tag actions with cookie-backed auth and standardized error envelopes.
- Implement client-backed form state so submit buttons show pending states, success redirects to the returned release URL, rejected mutations preserve form data and show server errors, and no UI state claims success until the API returns the updated release.
- Support separate `Save draft`, `Publish release`, `Update release`, `Publish draft`, and `Delete release` paths. Delete must require typed or explicit confirmation and must not delete the git tag unless the delete-tag option is selected and accepted by the API.
- Make generated notes insert or replace editor content only after the server returns a preview; users must be able to inspect and edit the result before publishing.
- Ensure latest-release options are stored and reflected in list/detail badges after publish/update.
- Add Rust tests for action transitions where needed, plus Vitest and Playwright coverage for create draft, publish release, edit existing release, generated notes, delete without tag deletion, duplicate tag rejection, invalid target rejection, non-writer gating, and mobile no-overflow.
- Save browser smoke screenshots under `ralph/screenshots/build/releases-002-phase3-*.jpg`.

**Verification**: focused Rust contracts, focused Vitest, focused Playwright `repository-release-management.spec.ts`, mandatory Editorial banned-value scan, then `make check && make test`; run full `make test-e2e` when the local database can run the seeded release flow.

---

## Phase 4: Asset Upload Lifecycle - upload intents, progress rows, completion, removal, and download integrity

**Done**: [ ]

**Scope**: Upgrade asset handling from metadata-only creation to a real upload lifecycle that supports drag-and-drop form rows, progress/success/error states, removal before publish, and existing release asset management.

**Key changes**:
- Add API support for upload completion and cancellation against the Phase 1 upload-intent contract, persisting `release_assets` only after validated completion or marking pending rows safely for drafts.
- Use local storage in development and S3 signed upload semantics when provider env is configured; keep the storage abstraction compatible with existing Pages/package artifact patterns.
- Add UI drag-and-drop/file-picker behavior for assets with progress rows, byte sizes, content types, checksum/status metadata, retry/cancel/remove controls, and clear messaging for unsupported or oversized files.
- Allow draft assets to be removed before publish and existing release assets to be deleted through server-confirmed actions; downloaded assets should continue to use authorized download metadata and counters from `releases-001`.
- Enforce private repository permissions, immutable release limits, storage-key redaction, and audit rows for upload complete/cancel/delete.
- Add tests for upload intent -> completion -> asset visible, cancellation, deletion, oversized file rejection, invalid content type, stale intent expiry, storage-key redaction, private asset access, and download counter integrity.
- Add Playwright coverage for drag/drop or file input upload, progress row transitions, remove before publish, remove existing asset, no dead controls, and mobile no-overflow. Save `ralph/screenshots/build/releases-002-phase4-assets-*.jpg`.

**Verification**: focused Rust storage/asset tests, focused Vitest, focused Playwright asset smoke, mandatory Editorial banned-value scan, then `make check && make test`; full `make test-e2e` if seeded storage and database state are stable.

---

## Phase 5: API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `releases-002` only after the dedicated management pages, API docs, screenshots, QA hints, and PRD bookkeeping are complete. Do not expand into packages, marketplace publishing, release discussions, security advisories, or organization-level release policies.

**Key changes**:
- Extend `/docs/api` with release management context, generated notes, create/update/publish/delete, latest policy, delete-tag opt-in, upload intent/complete/cancel, asset deletion, validation errors, permission gates, audit behavior, webhook/activity side effects, and local/S3 storage notes.
- Add final Playwright/browser smoke for `/releases/new`, `/releases/edit/{id}`, generated notes, draft save, publish, update, delete, asset upload/remove, forbidden state, immutable state, empty state, and mobile layout. Confirm no visible button/link/form is inert.
- Save final screenshots under `ralph/screenshots/build/releases-002-final-*.jpg`.
- Append honest `qa-hints.json` entries for real S3 signed uploads, large asset limits, upload interruption/retry, generated-notes accuracy across merge commits, concurrent release edits, immutable release policy, delete-tag safety, private repository leakage, webhook delivery, Markdown XSS, and accessibility traversal.
- Update `build-progress.txt`, `.qrspi/releases-002/structure.md`, and `prd.json`; set `releases-002.build_pass=true` only after all phases pass. Leave `qa_pass=false`.
- Run the mandatory Editorial banned-value scan before committing and fix any touched-file regressions.

**Verification**: focused Rust/Vitest/Playwright checks, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, same-env `make test`, full same-env `make test-e2e` when available, browser smoke screenshots under `ralph/screenshots/build/`, and mandatory Editorial banned-value scan with zero matches.
