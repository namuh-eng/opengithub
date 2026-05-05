# Structure Outline: projects-002 Projects v2 Table Workspace

**Ticket**: `projects-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-shell.jsx`, `ralph/screenshots/inspect/projects-roadmap-table.jpg`, existing Projects list/copy contract from `.qrspi/projects-001/structure.md`, `crates/api/migrations/202605060006_projects_v2_foundation.up.sql`, `crates/api/src/domain/projects.rs`, `crates/api/src/routes/projects.rs`, `web/src/components/ProjectsListPage.tsx`, and Projects docs under `target-docs/content/issues/planning-and-tracking-with-projects/`.
**Date**: 2026-05-06

## Phase 1: Workspace Read Contract - project table data is screen-ready

**Done**: [x]

**Scope**: Add the Rust/Postgres read contract for `/{owner}/projects/{number}/views/{view}` and `/orgs/{org}/projects/{number}/views/{view}` table workspaces. This phase makes project metadata, saved views, visible fields, item rows, field values, grouping/slicing/sorting/filter state, and viewer capabilities testable before adding the UI.

**Key changes**:
- `crates/api/migrations/`: add additive workspace columns/tables only where the Projects foundation is incomplete, including `project_item_field_values`, persisted `project_view_state` or equivalent view configuration metadata, saved sorts/group/slice state, item timeline events, and indexes for project item lookup/filtering.
- `crates/api/src/domain/projects.rs`: add DTOs for `ProjectWorkspace`, `ProjectWorkspaceView`, `ProjectWorkspaceField`, `ProjectWorkspaceItem`, typed field values, grouped sections, slice options, filter tokens, unsaved view metadata, and viewer edit capabilities.
- `crates/api/src/routes/projects.rs`: expose `GET /api/projects/{project_id}/workspace` plus owner/number convenience routes if useful, with `view`, `q`, `sort`, `group`, `slice`, `page`, and `pageSize` validation.
- Preserve Projects privacy and permissions from `projects-001`: anonymous/public reads only see public data, private projects require a session/PAT, and repository-linked issue/PR rows are hidden when the viewer cannot read the backing repository.
- `web/src/lib/api.ts`: add typed workspace DTOs and signed-cookie helpers without introducing JS-side auth.
- `crates/api/tests/projects_workspace_contract.rs`: cover table shape, field ordering, item rows for draft/issue/PR items, field value decoding, private denial, repository visibility filtering, invalid view/query errors, and no-secret error envelopes.

**Verification**: focused Rust contract tests, `cargo fmt --all`, `cargo check -p opengithub-api --tests`, web TypeScript for DTOs, mandatory Editorial banned-value scan, then `make check && make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Workspace Shell - saved views and table render without dead controls

**Done**: [x]

**Scope**: Render the Projects table workspace with the Editorial design system. The page should show the compact breadcrumb/header, project title, View/Insights/Settings controls, saved-view tabs, plus-view affordance, side slice rail, filter bar, matching item count, view configuration button, high-density table, grouped headers, and bottom add row shell using the Phase 1 read contract.

**Key changes**:
- `web/src/app/[owner]/projects/[number]/views/[view]/page.tsx` and `web/src/app/orgs/[org]/projects/[number]/views/[view]/page.tsx`: load workspace data through signed-cookie helpers and route not-found/forbidden states honestly.
- `web/src/components/ProjectWorkspacePage.tsx`: new shared Editorial workspace surface with tabs, filter form, visible field columns, row icons, repository/name/number links, labels, assignees, status chips, grouped section headers, sticky-ish header/omnibar layout, and mobile fallback.
- `web/src/lib/navigation.ts`: add stable workspace href builders for user and organization project routes, view tabs, filter/sort/group/slice query changes, and item deep links.
- All visible controls must either navigate, submit to a real route, open a non-placeholder dialog/menu, or be permission-disabled with explanatory copy. No `href="#"` or inert handlers.
- `web/tests/project-workspace-page.test.tsx`: cover user/org workspace rendering, saved view tabs, field columns, draft/issue/PR rows, grouped headers, filter URL state, disabled Insights/Settings handoffs where not yet built, no dead links, and Editorial token/primitives usage.

**Verification**: focused Vitest, web TypeScript, focused Biome, mandatory Editorial banned-value scan, Playwright smoke when seeded data is available saving `ralph/screenshots/build/projects-002-phase2-workspace.jpg`, then `make check && make test`.

---

## Phase 3: View State Editing - filters, sorting, grouping, slicing, and field visibility persist or show unsaved state

**Done**: [x]

**Scope**: Make workspace view controls functional. Changing filters, sort, grouping, slicing, or hidden fields updates the URL and marks the view as unsaved until the viewer saves it; authorized saves persist to the selected project view, while read-only viewers keep local URL state only.

**Key changes**:
- `crates/api/src/domain/projects.rs`: implement view-state mutation helpers with validation for supported fields, field type compatibility, sort direction, group/slice field eligibility, hidden field IDs, and stale view version checks.
- `crates/api/src/routes/projects.rs`: expose `PATCH /api/projects/{project_id}/views/{view_id}/state` with signed-session/PAT auth, write/admin permission checks, audit events, and standard validation errors.
- `web/src/app/api/projects/[projectId]/views/[viewId]/state/route.ts`: same-origin proxy forwarding signed cookies to Rust.
- `ProjectWorkspacePage.tsx`: add View configuration dialog/menu for visible fields, grouping, slicing, sorting, save/revert actions, unsaved indicator, and permission-disabled controls.
- Extend filters to support the first practical qualifier subset from the docs: text terms, `is:open`, `is:closed`, `is:issue`, `is:pr`, `is:draft`, `repo:owner/name`, `assignee:@me|login`, `label:name`, `no:assignee`, `no:label`, and field equality for single-select/status/date/text/number.
- Tests cover server-confirmed saves, read-only denial, invalid field IDs, stale updates, unsaved/revert behavior, URL preservation, and field-click-to-filter tokens.

**Verification**: focused Rust contract, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-002-phase3-view-state.jpg`, then `make check && make test`; run `make test-e2e` when the local DB/dev servers are healthy.

---

## Phase 4: Inline Cell Editing - field edits sync to project items and linked resources

**Done**: [x]

**Scope**: Wire table cells to real permissioned edits. Authorized viewers can edit supported project field values inline; mapped fields update linked issue/PR metadata when applicable, draft issue fields update the draft item, and all successful edits create timeline/audit/notification records.

**Key changes**:
- `crates/api/src/domain/projects.rs`: implement field value validation and mutation for single-select/status, iteration, date, text, number, assignees, labels, milestone, repository, and draft title/body where supported.
- `crates/api/src/routes/projects.rs`: expose `PATCH /api/projects/{project_id}/items/{item_id}/fields/{field_id}` and normalize validation, permission, archived item, hidden repository, and unsupported field errors.
- Preserve linked issue/PR ownership rules: issue/PR metadata edits must use the backing repository permission model, while project-only custom fields require project write/admin permission.
- `web/src/app/api/projects/[projectId]/items/[itemId]/fields/[fieldId]/route.ts`: same-origin proxy forwarding signed cookies to Rust.
- `ProjectWorkspacePage.tsx`: add accessible inline editors for each field type, optimistic pending state only after submission starts, success/error feedback, and rollback on server errors.
- Tests cover all supported field types, linked issue/PR sync side effects, draft item edits, permission denial, archived item denial, timeline/audit writes, notifications, editor keyboard behavior, and no local-only fake edits.

**Verification**: focused Rust contract, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-002-phase4-inline-edit.jpg`, then `make check && make test`; run `make test-e2e` when stable.

---

## Phase 5: Add Row and Reordering - workspace can add items and persist manual order

**Done**: [x]

**Scope**: Make the bottom add row and manual ordering real. Users can paste issue/PR URLs, search repositories with `#`, bulk add existing issues/PRs, create draft issues, open the issue creation modal where available, and reorder rows within the table or grouped sections.

**Key changes**:
- `crates/api/src/domain/projects.rs`: implement item add helpers for linked issues, linked pull requests, draft issues, bulk adds, duplicate handling, item limit validation, filtered-metadata defaults, and manual position assignment.
- `crates/api/src/routes/projects.rs`: expose `POST /api/projects/{project_id}/items`, `POST /api/projects/{project_id}/items/bulk`, `PATCH /api/projects/{project_id}/items/{item_id}/position`, and `DELETE /api/projects/{project_id}/items/{item_id}` if remove controls are visible.
- Add repository/item search support needed by the omnibar, scoped to readable repositories and project permissions.
- `ProjectWorkspacePage.tsx`: implement the add-row combobox/dialog, paste URL parsing, draft issue creation form, bulk-add repository picker, row reorder controls, grouped-row move semantics where the grouped field can be updated, and honest disabled states for unsupported drag interactions.
- Tests cover linked item add, draft creation, duplicate handling, filtered metadata defaults, row reorder persistence, grouped move value updates, remove behavior if exposed, audit/timeline/notification rows, no dead omnibar controls, and error feedback.

**Verification**: focused Rust contract, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-002-phase5-add-row-reorder.jpg`, then `make check && make test`; run `make test-e2e` when stable.

---

## Phase 6: API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [x]

**Scope**: Finish `projects-002` only after workspace read/write contracts, table UI, view-state persistence, inline edits, add-row flows, row ordering, docs, screenshots, and QA handoff are verified. Do not implement board layout, roadmap layout, insights charts, project field settings administration, workflows automation, project access settings, or exports in this feature.

**Key changes**:
- `web/src/lib/api-docs.ts`: document Projects workspace endpoints, view state patching, item field patching, item add/bulk/reorder/remove endpoints, auth/privacy, permissions, validation rules, side effects, and standard errors.
- `web/tests/e2e/projects-workspace.spec.ts`: final desktop/mobile smoke for user and organization project workspaces covering saved view navigation, filters, field visibility, grouping/slicing, inline edits, add row, draft issue creation, linked issue/PR add, reorder, permission-denied states, no dead controls, and no horizontal overflow.
- `ralph/screenshots/build/`: save final evidence screenshots for default table, grouped view, slice rail, view config dialog, inline editor, add-row dialog, unsaved state, permission-disabled state, and mobile.
- `qa-hints.json`: append QA targets for large virtualized tables, hidden private linked repositories, stale view-state conflicts, concurrent cell edits, duplicate linked items, grouped-row moves, URL paste parsing, filtered metadata defaults, permission boundaries, and notification/timeline side effects.
- `build-progress.txt`, `.qrspi/projects-002/structure.md`, and `prd.json`: record evidence and set `projects-002.build_pass=true` only after all phases pass; leave `qa_pass=false`.

**Verification**: focused Rust/Vitest/Playwright checks, `make check`, `make test`, `make test-e2e` when local DB/dev servers are healthy, browser screenshots under `ralph/screenshots/build/`, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan with zero matches.
