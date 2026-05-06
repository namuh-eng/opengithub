# Structure Outline: projects-005 Project Items, Draft Issues, Archive, and Side Panel

**Ticket**: `projects-005`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-shell.jsx`, existing Projects outlines in `.qrspi/projects-001/structure.md`, `.qrspi/projects-002/structure.md`, `.qrspi/projects-003/structure.md`, `.qrspi/projects-004/structure.md`, `crates/api/src/domain/projects.rs`, `crates/api/src/routes/projects.rs`, `web/src/components/ProjectWorkspacePage.tsx`, `web/src/lib/api-docs.ts`, and Projects item docs under `target-docs/content/issues/planning-and-tracking-with-projects/`.
**Date**: 2026-05-06

## Phase 1: Item Detail Read Contract - side panel data is screen-ready

**Done**: [ ]

**Scope**: Extend the existing Projects workspace item model with a dedicated read contract for item side panels and archive views. This phase makes draft, linked issue, and linked PR metadata testable without adding new mutations.

**Key changes**:
- `crates/api/migrations/`: add only missing additive metadata for draft body/activity/comments, archived-by/restored-by timestamps, item source sync markers, and indexes for active vs archived item lookups.
- `crates/api/src/domain/projects.rs`: add `ProjectItemDetail`, `ProjectItemActivity`, `ProjectArchivedItem`, source repository summaries, draft issue edit metadata, linked issue/PR sync metadata, and viewer capabilities for edit/convert/archive/remove/restore/comment.
- `crates/api/src/routes/projects.rs`: expose `GET /api/projects/{project_id}/items/{item_id}` and `GET /api/projects/{project_id}/items/archived` with project privacy, linked repository visibility filtering, pagination, item-type filters, and no-secret envelopes.
- `web/src/lib/api.ts`: add typed item-detail and archived-item DTOs plus signed-cookie helpers.
- `crates/api/tests/projects_items_contract.rs`: cover draft detail reads, linked issue/PR detail reads, hidden private linked resources, archived listing filters, permission flags, and no-secret errors.

**Verification**: focused Rust contract tests, `cargo fmt --all`, `cargo check -p opengithub-api --tests`, web TypeScript for DTOs, mandatory Editorial banned-value scan, then `make check && make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Side Panel Shell - item rows open real detail panels

**Done**: [ ]

**Scope**: Make table, board, and roadmap item rows/cards open an Editorial side panel backed by Phase 1 data. The panel shows title, type icon, source repository/link, draft body, field editors in read-only mode where needed, activity/comments, and permission-aware actions without dead controls.

**Key changes**:
- `web/src/components/ProjectWorkspacePage.tsx`: add item deep-link state, side panel drawer, row/card click/focus behavior, close button, source links, field summary, comments/activity sections, Archive/Remove/Convert actions as real disabled or wired controls, and mobile-safe panel layout.
- `web/src/app/[owner]/projects/[number]/items/[itemId]/page.tsx` and `web/src/app/orgs/[org]/projects/[number]/items/[itemId]/page.tsx`: support direct item URLs that render the workspace context plus the open panel.
- `web/src/lib/navigation.ts`: add stable item, archived-items, and side-panel href builders preserving project owner/org routes and view/query state.
- Controls must navigate, submit to a real route, open a real menu/dialog, or be permission-disabled with explanatory copy. Use Editorial primitives/tokens only.
- `web/tests/project-workspace-page.test.tsx`: cover item row opening, direct item URL rendering, panel close/navigation, draft vs issue vs PR metadata, source links, permission-disabled controls, no dead links, mobile text fit, and banned visual guardrails.

**Verification**: focused Vitest, web TypeScript, focused Biome, mandatory Editorial banned-value scan, Playwright smoke when seeded data is available saving `ralph/screenshots/build/projects-005-phase2-item-panel.jpg`, then `make check && make test`.

---

## Phase 3: Draft Issue Editing and Comments - project-only drafts mutate without repository notifications

**Done**: [ ]

**Scope**: Make draft issue title/body edits and draft comments/activity real from the side panel. Draft changes update project-only data and item events while deliberately avoiding repository issue notifications until conversion.

**Key changes**:
- `crates/api/src/domain/projects.rs`: implement draft title/body update, draft comment create/update/delete where supported, activity event writes, stale `expectedUpdatedAt` checks, archived item denial, project write/admin permissions, and no mention-notification side effects for drafts.
- `crates/api/src/routes/projects.rs`: expose `PATCH /api/projects/{project_id}/items/{item_id}/draft`, `POST /api/projects/{project_id}/items/{item_id}/comments`, `PATCH /api/projects/{project_id}/items/{item_id}/comments/{comment_id}`, and `DELETE /api/projects/{project_id}/items/{item_id}/comments/{comment_id}`.
- `web/src/app/api/projects/[projectId]/items/[itemId]/draft/route.ts` and comment proxy routes: forward signed cookies to Rust.
- `ProjectWorkspacePage.tsx`: wire draft title/body editor, comment composer, comment edit/delete controls, pending/error/success states, and refreshed panel/workspace rows.
- Tests cover draft validation, stale edits, archived denial, read-only denial, event records, absence of repository notifications, UI success/error states, and no local-only fake updates.

**Verification**: focused Rust contract tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-005-phase3-draft-edit.jpg` when possible, then `make check && make test`; run `make test-e2e` when local DB/dev servers are healthy.

---

## Phase 4: Draft Conversion to Repository Issue - drafts become normal linked issues

**Done**: [ ]

**Scope**: Convert a draft project item into a repository issue by choosing a readable/writable repository plus optional labels, assignees, milestone, and field defaults. After conversion, the project item points to the new issue and gains normal issue URL/timeline/notification behavior.

**Key changes**:
- `crates/api/src/domain/projects.rs`: implement conversion helpers with repository write checks, project write/admin checks, repository picker search, issue number allocation, label/assignee/milestone validation, draft body transfer, field value retention, timeline events, notification fanout, audit rows, and idempotency/stale guards.
- `crates/api/src/routes/projects.rs`: expose `GET /api/projects/{project_id}/conversion-targets` and `POST /api/projects/{project_id}/items/{item_id}/convert-to-issue`.
- `web/src/app/api/projects/[projectId]/conversion-targets/route.ts` and convert proxy route: forward signed cookies to Rust.
- `ProjectWorkspacePage.tsx`: add Convert to issue dialog with repository search, labels/assignees/milestone selectors, validation feedback, success redirect to the linked issue state, and disabled states for non-drafts or read-only viewers.
- Tests cover conversion success, forbidden repositories, invalid labels/milestones, duplicate submits/idempotency, field preservation, timeline/audit/notification side effects, panel refresh, and source URL update.

**Verification**: focused Rust contract tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-005-phase4-draft-convert.jpg`, then `make check && make test`; run `make test-e2e` when stable.

---

## Phase 5: Archive, Restore, Remove, and Linked Sync - item lifecycle stays consistent

**Done**: [ ]

**Scope**: Complete item lifecycle controls. Archiving removes items from active table/board/roadmap views while preserving history; restore returns them; remove detaches safely; linked issue/PR metadata changes stay synchronized with project fields and side-panel reads.

**Key changes**:
- `crates/api/src/domain/projects.rs`: harden archive/restore/remove helpers, archived listing filters, restore position placement, linked issue/PR metadata sync for title/state/labels/assignees/milestone, item events, audit rows, and permission checks across project and backing repository.
- `crates/api/src/routes/projects.rs`: expose `PATCH /api/projects/{project_id}/items/{item_id}/archive`, `PATCH /api/projects/{project_id}/items/{item_id}/restore`, and ensure existing `DELETE /api/projects/{project_id}/items/{item_id}` behavior is documented and compatible.
- `web/src/app/api/projects/[projectId]/items/[itemId]/archive/route.ts` and restore proxy route: forward signed cookies to Rust.
- `ProjectWorkspacePage.tsx`: wire Archive, Restore, Remove confirmations, archived-items page link, archived list page with filters, restore controls, and active workspace refresh.
- Tests cover archive hiding active views, restore placement, remove semantics, linked issue/PR sync into side panel and table fields, read-only/archived denials, audit/event evidence, and no data loss for linked resources.

**Verification**: focused Rust contract tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-005-phase5-archive-restore.jpg`, then `make check && make test`; run `make test-e2e` when stable.

---

## Phase 6: API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `projects-005` only after item detail reads, side-panel UI, draft editing/comments, draft conversion, archive/restore/remove, linked sync, docs, screenshots, and QA handoff are verified. Do not implement Projects workflow automation, insights charts, access settings, exports, GraphQL automation hooks, or GitHub visual styling as part of this feature.

**Key changes**:
- `web/src/lib/api-docs.ts`: document item detail reads, archived item lists, draft edits, item comments, conversion targets, convert-to-issue, archive, restore, remove, linked sync side effects, auth/privacy, permissions, stale conflicts, and standard errors.
- `web/tests/e2e/projects-items-side-panel.spec.ts`: final desktop/mobile smoke covering row/card panel open, draft edit/comment, convert to issue, linked issue field sync, archive, archived list restore, remove confirmation, no dead controls, and bounded overflow.
- `ralph/screenshots/build/`: save final evidence screenshots for item panel, draft editor, convert dialog, archived item list, restore confirmation, linked issue sync, permission-disabled state, and mobile.
- `qa-hints.json`: append QA targets for concurrent draft edits, conversion duplicate submits, private repository conversion targets, label/assignee permission boundaries, stale archive/restore, large archived lists, linked issue/PR sync races, notification fanout, audit rows, and mobile side-panel scrolling.
- `build-progress.txt`, `.qrspi/projects-005/structure.md`, and `prd.json`: record evidence and set `projects-005.build_pass=true` only after all phases pass; leave `qa_pass=false`.

**Verification**: focused Rust/Vitest/Playwright checks, `make check`, `make test`, `make test-e2e` when local DB/dev servers are healthy, browser screenshots under `ralph/screenshots/build/`, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan with zero matches.
