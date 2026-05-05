# Structure Outline: projects-001 Projects v2 List Surfaces

**Ticket**: `projects-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-shell.jsx`, `ralph/screenshots/inspect/projects-org-list.jpg`, existing organization/profile and repository shells, `target-docs/content/issues/planning-and-tracking-with-projects/creating-projects/`, and `target-docs/content/issues/planning-and-tracking-with-projects/finding-your-projects.md`.
**Date**: 2026-05-06

## Phase 1: Projects List API Contract - account, organization, and repository lists are readable

**Done**: [ ]

**Scope**: Add the Rust/Postgres contract for Projects v2 list pages across users, organizations, and repositories. This phase makes project rows, templates, counts, permissions, and repository linkage testable through API responses while keeping UI changes minimal.

**Key changes**:
- `crates/api/migrations/`: add additive Projects v2 tables if absent: `projects`, `project_repositories`, `project_templates`, `project_permissions`, `project_status_updates`, `project_views`, `project_fields`, `project_workflows`, `project_items`, plus indexes for owner scope, state, updated time, templates, and repository links.
- `crates/api/src/domain/projects.rs`: add DTOs for `ProjectList`, `ProjectRow`, `ProjectTemplateRow`, `ProjectCounts`, `ProjectStatusSummary`, `ProjectPermissions`, `ProjectSort`, and `ProjectListScope`.
- `crates/api/src/routes/projects.rs`: expose `GET /api/users/{username}/projects`, `GET /api/orgs/{org}/projects`, and `GET /api/repos/{owner}/{repo}/projects` with `q`, `state`, `tab`, `sort`, `page`, and `pageSize` validation.
- `web/src/lib/api.ts`: add typed project list DTOs and signed-cookie helpers for user/org/repo projects surfaces.
- `crates/api/tests/projects_list_contract.rs`: cover public/private owner visibility, org policy-disabled states, repository-linked projects, templates tab, open/closed counts, sort enum validation, search matching, and no-secret error envelopes.

**Verification**: focused Rust contract tests, `cargo fmt --all`, `cargo check -p opengithub-api --tests`, then `make check && make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Projects List Pages - render the three list surfaces

**Done**: [ ]

**Scope**: Replace current Projects placeholders with Editorial list pages for `/{user}?tab=projects`, `/orgs/{org}/projects`, and `/{owner}/{repo}/projects`, backed by the Phase 1 API. Rows must navigate to project workspaces and all empty-state CTAs must be concrete or permission-disabled.

**Key changes**:
- `web/src/components/ProjectsListPage.tsx`: shared Editorial list surface with Welcome banner, Projects/Templates tabs, open/closed count tabs, dense rows, status chips, metadata, row menus, and copy buttons.
- `web/src/app/orgs/[org]/projects/page.tsx`: render inside the organization profile shell with the Projects tab active and count labels preserved.
- `web/src/app/[owner]/[repo]/projects/page.tsx`: render inside the repository shell with Projects tab active and repository-linked copy.
- Add or wire the profile `?tab=projects` path for user-owned project lists using existing profile navigation patterns.
- `web/tests/projects-list-page.test.tsx`: cover org/repo/user render states, active tab labels, row hrefs, status chips, templates tab, empty/disabled states, no `href="#"`, no inert handlers, and Editorial token/primitives usage.

**Verification**: focused Vitest, web TypeScript, focused Biome, mandatory Editorial banned-value scan, then `make check && make test`. Save a screenshot if seeded data already exposes project rows.

---

## Phase 3: Query, Tabs, Sort, and URL State - list controls drive server-confirmed results

**Done**: [ ]

**Scope**: Make search, open/closed tabs, templates tab, and sort menu functional and URL-backed across all three surfaces. This phase is complete when a browser reload preserves list state and no list-control state is local-only.

**Key changes**:
- Extend list route pages to parse `q`, `state`, `tab`, `sort`, and pagination from search params and pass them to cookie-backed API helpers.
- `ProjectsListPage.tsx`: add accessible search form, active filter chips, open/closed tabs, templates tab, sort menu with selected radio state, pagination, and clear-filter behavior.
- `web/src/lib/navigation.ts`: add stable href builders for user, organization, repository project lists and project workspace rows.
- Extend Rust tests for title/description/status search, tab/state count consistency, sort stability, pagination bounds, and repository project link filtering.
- Extend Vitest and Playwright coverage for typing search, switching tabs, selecting sort, preserving URL state, opening row links, and avoiding horizontal overflow.

**Verification**: focused Rust tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-001-phase3-list-controls.jpg`, then `make check && make test`; run `make test-e2e` when the local test DB is available.

---

## Phase 4: Copy Project Flow - permissioned copies clone views, fields, workflows, and optional drafts

**Done**: [ ]

**Scope**: Wire the visible copy-project action to a real mutation. Authorized users can copy a project from list rows or templates, choose whether to include draft issues, and land on the new project workspace stub; unauthorized viewers see a permission error without local-only creation.

**Key changes**:
- `crates/api/src/domain/projects.rs`: implement `copy_project_for_actor` that clones project metadata, repository links, views, fields/options, workflows, and optionally draft items; exclude linked issues/PRs unless explicitly modeled by later project item phases.
- `crates/api/src/routes/projects.rs`: expose `POST /api/projects/{project_id}/copies` and normalize validation/permission errors with standard envelopes.
- Add audit events and recent-visit rows for successful copies; enforce closed/deleted/source visibility and organization project policy constraints.
- `web/src/app/api/projects/[projectId]/copies/route.ts`: same-origin proxy forwarding signed cookies to Rust without JS-side auth.
- `ProjectsListPage.tsx`: add copy dialog with prefilled `[COPY]` title, Include draft issues checkbox, Cancel, Copy submit, pending/success/error states, and redirect to the returned workspace href.
- Tests cover copy payloads, permission denial, include/exclude draft semantics, cloned view/field/workflow rows, audit rows, dialog validation, server errors, no optimistic fake rows, and no dead controls.

**Verification**: focused Rust contract, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-001-phase4-copy-flow.jpg`, then `make check && make test`; run `make test-e2e` when stable.

---

## Phase 5: API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `projects-001` only after list contracts, UI states, URL-backed controls, copy flow, docs, screenshots, and QA handoff are verified. Do not implement the project table workspace, board/roadmap views, project fields settings, item side panel, workflows, or insights in this feature.

**Key changes**:
- `web/src/lib/api-docs.ts`: document Projects list and copy endpoints, including owner/repository scope, auth/privacy, query params, templates, counts, sort enums, copy semantics, permissions, audit behavior, and standard errors.
- `web/tests/e2e/projects-list.spec.ts`: final desktop/mobile smoke for org, repo, and user projects lists; search, tabs, sort, templates, row navigation, copy success/error, empty state, forbidden state, no dead controls, and no horizontal overflow.
- `ralph/screenshots/build/`: save final evidence screenshots for org list, repo list, user list, templates tab, copy dialog/success, forbidden/disabled state, and mobile.
- `qa-hints.json`: append QA targets for large project counts, private organization leakage, stale URL filters, copy concurrency, draft issue inclusion, permission boundaries, and project policy disabled states.
- `build-progress.txt`, `.qrspi/projects-001/structure.md`, and `prd.json`: record evidence and set `projects-001.build_pass=true` only after all phases pass; leave `qa_pass=false`.

**Verification**: focused Rust/Vitest/Playwright checks, `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, `make test-e2e` when local DB/dev servers are healthy, browser screenshots under `ralph/screenshots/build/`, and mandatory Editorial banned-value scan with zero matches.
