# Structure Outline: projects-004 Projects v2 Custom Fields and Iterations

**Ticket**: `projects-004`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-shell.jsx`, existing Projects list/workspace/layout outlines in `.qrspi/projects-001/structure.md`, `.qrspi/projects-002/structure.md`, `.qrspi/projects-003/structure.md`, `crates/api/migrations/202605060006_projects_v2_foundation.up.sql`, `crates/api/migrations/202605060007_projects_workspace_read.up.sql`, `crates/api/src/domain/projects.rs`, `crates/api/src/routes/projects.rs`, `web/src/components/ProjectWorkspacePage.tsx`, and Projects custom-field docs under `target-docs/content/issues/planning-and-tracking-with-projects/`.
**Date**: 2026-05-06

## Phase 1: Field Settings Read Contract - custom fields are administrable from one screen-ready API

**Done**: [x]

**Scope**: Add the authenticated read contract for `/orgs/{org}/projects/{number}/settings/fields` and owner-project equivalents. The API returns project metadata, built-in fields, custom fields, single-select options, iteration settings, field usage counts, limits, and viewer capabilities without changing fields yet.

**Key changes**:
- `crates/api/migrations/`: add additive tables/columns where needed for durable `project_field_options`, `project_iterations`, `project_iteration_breaks`, option colors/positions, iteration dates/durations, deleted-field metadata, cache invalidation markers, and indexes by project/field/position/date.
- `crates/api/src/domain/projects.rs`: add `ProjectFieldSettings`, `ProjectFieldDetail`, `ProjectFieldOption`, `ProjectIteration`, `ProjectIterationBreak`, field limit metadata, and permission/capability DTOs; decode existing `project_fields.settings` without breaking workspace reads.
- `crates/api/src/routes/projects.rs`: expose `GET /api/projects/{project_id}/settings/fields` plus owner/number convenience lookup if useful, enforcing project read/admin privacy and hiding private linked item counts.
- `web/src/lib/api.ts`: add typed field-settings DTOs and signed-cookie fetch helpers without JS-side auth.
- `crates/api/tests/projects_field_settings_contract.rs`: cover read shape, built-in/custom field ordering, option and iteration decoding, project field limits, private/read-only denial, hidden linked resource counts, and no-secret error envelopes.

**Verification**: focused Rust contract tests, `cargo fmt --all`, `cargo check -p opengithub-api --tests`, web TypeScript for DTOs, mandatory Editorial banned-value scan, then `make check && make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Fields Settings Page - field list and detail pane render without dead controls

**Done**: [x]

**Scope**: Render the Projects settings Fields page using the Phase 1 read contract. The page shows the left settings sidebar, field list, selected field detail pane, New field action, built-in read-only fields, custom field metadata, and honest permission-disabled states.

**Key changes**:
- `web/src/app/[owner]/projects/[number]/settings/fields/page.tsx` and `web/src/app/orgs/[org]/projects/[number]/settings/fields/page.tsx`: load field settings through signed-cookie helpers and route forbidden/not-found states honestly.
- `web/src/components/ProjectFieldSettingsPage.tsx`: add the Editorial settings surface with project breadcrumb/header, settings sidebar, field list, selected detail pane, type badges, usage counts, field limit copy, and stable responsive layout.
- `web/src/lib/navigation.ts`: add stable href builders for project settings, fields, individual selected field query state, and workspace return links.
- Controls must navigate, open a real menu/dialog, submit a mutation, or be permission-disabled with explanatory copy. No `href="#"`, inert handlers, or GitHub visual tokens.
- `web/tests/project-field-settings-page.test.tsx`: cover owner/org rendering, selected field state, built-in field read-only behavior, New field dialog opening, settings sidebar links, permission-disabled controls, no dead links, unsafe-markup guardrails, and Editorial token usage.

**Verification**: focused Vitest, web TypeScript, focused Biome, mandatory Editorial banned-value scan, Playwright smoke when seeded data is available saving `ralph/screenshots/build/projects-004-phase2-field-settings.jpg`, then `make check && make test`.

---

## Phase 3: Field Create, Rename, and Delete - custom field lifecycle mutates durable project schema

**Done**: [ ]

**Scope**: Make New field, Rename, and Delete real for custom date/text/number/single-select/iteration fields. Authorized users can create fields up to limits, rename custom fields, and delete fields after confirmation; deletion removes project item field values but never deletes linked issues or pull requests.

**Key changes**:
- `crates/api/src/domain/projects.rs`: implement field lifecycle helpers with project write/admin permission checks, field limit enforcement, supported type validation, normalized unique names, built-in field protection, stale `expectedUpdatedAt`, field value cleanup, view/filter cache invalidation, item events, and audit events.
- `crates/api/src/routes/projects.rs`: expose `POST /api/projects/{project_id}/fields`, `PATCH /api/projects/{project_id}/fields/{field_id}`, and `DELETE /api/projects/{project_id}/fields/{field_id}` with standard validation/conflict/not-found envelopes.
- `web/src/app/api/projects/[projectId]/fields/route.ts` and `web/src/app/api/projects/[projectId]/fields/[fieldId]/route.ts`: same-origin cookie-forwarding proxies to Rust.
- `ProjectFieldSettingsPage.tsx`: wire New field dialog, name input, type selector, Save changes, Rename, Delete confirmation, pending states, inline errors, success redirects, and read-only disabled states.
- Tests cover create for each supported type, duplicate/blank/limit validation, rename conflicts, stale updates, built-in deletion denial, custom deletion value cleanup, audit/cache side effects, proxy cookie forwarding, UI success/error states, and no local-only fake updates.

**Verification**: focused Rust contract tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-004-phase3-field-lifecycle.jpg`, then `make check && make test`; run `make test-e2e` when local DB/dev servers are healthy.

---

## Phase 4: Single-Select Options and Field Values - option rows and workspace values stay in sync

**Done**: [ ]

**Scope**: Add full single-select option administration and ensure option changes remain compatible with Projects workspace field values, board columns, filters, and item cells. Authorized users can add, rename, recolor, reorder, and delete options with visible impact on existing item values.

**Key changes**:
- `crates/api/src/domain/projects.rs`: implement option mutation helpers for single-select/status-compatible fields with color token validation, position updates, duplicate name checks, deletion behavior for existing `project_item_field_values`, board column setting reconciliation, filter/view cache invalidation, item events, and audit events.
- `crates/api/src/routes/projects.rs`: expose `POST /api/projects/{project_id}/fields/{field_id}/options`, `PATCH /api/projects/{project_id}/fields/{field_id}/options/{option_id}`, `PATCH /api/projects/{project_id}/fields/{field_id}/options/reorder`, and `DELETE /api/projects/{project_id}/fields/{field_id}/options/{option_id}`.
- `web/src/app/api/projects/[projectId]/fields/[fieldId]/options/*`: same-origin proxies for create/update/reorder/delete.
- `ProjectFieldSettingsPage.tsx`: render option rows with Editorial color swatches, inline name/color editing, add option, reorder controls, delete confirmation, empty state CTA, pending/error states, and disabled states for non-single-select fields.
- Tests cover option CRUD, reorder persistence, color validation, value cleanup/retention decisions, board column synchronization, filter cache invalidation, workspace value rendering after option changes, keyboard-safe row controls, and mobile text fit.

**Verification**: focused Rust contract tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-004-phase4-single-select-options.jpg`, then `make check && make test`; run `make test-e2e` when stable.

---

## Phase 5: Iteration Cycles, Breaks, and Filters - iteration fields manage real date ranges

**Done**: [ ]

**Scope**: Make iteration field settings real. Creating an iteration field seeds three default iterations; users can edit start date, duration, unit, generated future cycles, individual iteration names/date ranges, insert breaks, and use `@current`, `@previous`, `@next`, comparisons, and ranges in project filters.

**Key changes**:
- `crates/api/src/domain/projects.rs`: implement iteration schedule generation, date/duration/unit validation, non-overlap checks, individual iteration update/delete rules, break insertion/removal, default three-cycle seeding, item value preservation rules, and filter token evaluation for relative/current/range operators.
- `crates/api/src/routes/projects.rs`: expose `PATCH /api/projects/{project_id}/fields/{field_id}/iterations/settings`, `POST /api/projects/{project_id}/fields/{field_id}/iterations`, `PATCH /api/projects/{project_id}/fields/{field_id}/iterations/{iteration_id}`, and break create/delete endpoints.
- `ProjectFieldSettingsPage.tsx`: add Starts on date picker, duration stepper with days/weeks selector, Add iteration, More options for custom dates, Insert break affordance, iteration list editing, validation copy, and read-only states.
- `ProjectWorkspacePage.tsx` and filter helpers: recognize iteration values and filter tokens in table/board/roadmap views without regressing projects-002/003 behavior.
- Tests cover default iteration generation, date math across timezones, break insertion, invalid overlap/range errors, filter tokens `@current`, `@previous`, `@next`, comparison operators, ranges, workspace row filtering, roadmap compatibility, and no text overlap.

**Verification**: focused Rust contract tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-004-phase5-iteration-settings.jpg`, then `make check && make test`; run `make test-e2e` when stable.

---

## Phase 6: API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `projects-004` only after field settings reads, field lifecycle mutations, single-select option administration, iteration schedules/breaks, filter integration, docs, screenshots, and QA handoff are verified. Do not implement project workflow automation, access settings, exports, insights charts, formulas, full drag-and-drop option reordering, or GitHub visual styling as part of this feature.

**Key changes**:
- `web/src/lib/api-docs.ts`: document field settings reads, field create/rename/delete, option CRUD/reorder/delete, iteration settings, iteration/break endpoints, auth/privacy, permissions, field limits, deletion side effects, cache invalidation, and standard errors.
- `web/tests/e2e/projects-field-settings.spec.ts`: final desktop/mobile smoke for org and user project field settings covering create, rename, delete confirmation, option add/recolor/reorder/delete, iteration schedule edits, break insertion, workspace filter reflection, read-only disabled state, no dead controls, and no unintended horizontal overflow.
- `ralph/screenshots/build/`: save final evidence screenshots for field list, create dialog, single-select option rows, iteration schedule, delete confirmation, filter reflection, permission-disabled state, and mobile.
- `qa-hints.json`: append QA targets for large option lists, concurrent field deletes/renames, stale option reorders, item value cleanup, hidden private linked repositories, timezone/date-boundary iteration filters, overlapping breaks, field limit edge cases, permissions, audit rows, and project view cache invalidation.
- `build-progress.txt`, `.qrspi/projects-004/structure.md`, and `prd.json`: record evidence and set `projects-004.build_pass=true` only after all phases pass; leave `qa_pass=false`.

**Verification**: focused Rust/Vitest/Playwright checks, `make check`, `make test`, `make test-e2e` when local DB/dev servers are healthy, browser screenshots under `ralph/screenshots/build/`, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan with zero matches.
