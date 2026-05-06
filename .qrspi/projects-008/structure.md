# Structure Outline: projects-008 Project Insights Charts and Sharing

**Ticket**: `projects-008`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-shell.jsx`, existing Projects outlines in `.qrspi/projects-001/structure.md` through `.qrspi/projects-007/structure.md`, repository Insights chart components/tests, `crates/api/src/domain/projects.rs`, `crates/api/src/routes/projects.rs`, `web/src/components/ProjectWorkspacePage.tsx`, `web/src/components/ProjectSettingsPage.tsx`, `web/src/lib/navigation.ts`, and `web/src/lib/api-docs.ts`.
**Date**: 2026-05-06

## Phase 1: Insights Read Contract - default burn-up and chart data are inspectable

**Done**: [x]

**Scope**: Add the authenticated Rust read contract for Project Insights. The response exposes project context, chart navigation, default Burn up chart, matching item count, range state, accessible data-table rows, custom chart summaries, latest project status, viewer capabilities, and privacy-filtered source items without mutation UI yet.

**Key changes**:
- `crates/api/migrations/`: add only missing additive chart tables, such as `project_charts`, `project_chart_revisions`, `project_chart_series_cache`, chart sharing/visibility metadata, and indexes on project items, status/date fields, chart caches, and audit events.
- `crates/api/src/domain/projects.rs`: add `ProjectInsights`, `ProjectInsightsChart`, `ProjectInsightsSeries`, `ProjectInsightsPoint`, `ProjectInsightsDataRow`, `ProjectInsightsRange`, `ProjectInsightsFilter`, `ProjectInsightsCapabilities`, and chart summary DTOs.
- `crates/api/src/routes/projects.rs`: expose `GET /api/projects/{project_id}/insights` with query params for `chart`, `range`, `start`, `end`, `filter`, and `table`, enforcing project visibility, item privacy, repository permission filtering, and standard error envelopes.
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed Insights DTOs plus signed-cookie helpers for user and organization project-number routes.
- `crates/api/tests/projects_insights_contract.rs`: cover default burn-up seeding, item filtering, date/range bounds, custom chart summaries, data-table shape, status update exposure, read-only capabilities, private item filtering, and no-secret forbidden/not-found errors.

**Verification**: focused Rust contract tests, `cargo fmt --all`, `cargo check -p opengithub-api --tests`, web TypeScript for DTOs, mandatory Editorial banned-value scan, then `make check && make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Insights Page - default charts render with real project context

**Done**: [x]

**Scope**: Build `/orgs/{org}/projects/{number}/insights` and user-project equivalents using the Phase 1 read contract. The page switches from the workspace to an Insights header, selected Insights nav item, Return to project view link, left chart sidebar, Burn up chart surface, filter bar, range controls, chart action buttons, and latest status summary using Editorial primitives only.

**Key changes**:
- `web/src/app/[owner]/projects/[number]/insights/page.tsx` and `web/src/app/orgs/[org]/projects/[number]/insights/page.tsx`: load Insights through signed-cookie helpers and render forbidden/not-found/closed states consistently with existing Projects pages.
- `web/src/components/ProjectInsightsPage.tsx`: add breadcrumb/header, Return to project view link, Insights-selected project nav, Default charts and Custom charts sidebar sections, Burn up chart card, accessible chart region, range links, Custom range button, chart actions, status summary, and read-only disabled states.
- `web/src/lib/navigation.ts`: add stable href builders for project insights, selected chart, ranges, custom range, data table, custom chart create/edit panels, and workspace return.
- Controls must be real links/forms, open route-backed panels/dialogs, or be disabled with clear capability state. No `href="#"`, inert handlers, GitHub colors, Primer imports, or Octicons.
- `web/tests/project-insights-page.test.tsx`: cover user/org rendering, return link, selected Insights nav, default/custom chart sidebar, chart region semantics, range links, disabled mutation controls for readers, no dead controls, mobile wrapping, and Editorial guardrails.

**Verification**: focused Vitest, web TypeScript, focused Biome, mandatory Editorial banned-value scan, Playwright smoke when seeded data is available saving `ralph/screenshots/build/projects-008-phase2-insights-page.jpg`, then `make check && make test`.

---

## Phase 3: Filters, Ranges, Custom Range, and Data Table - chart exploration recomputes from project items

**Done**: [x]

**Scope**: Make chart exploration real. Filter changes update URL/query state and matching count, range links recompute the Burn up window, custom range validates date selection, and View as data table toggles an accessible table without losing project context.

**Key changes**:
- `crates/api/src/domain/projects.rs`: implement filter parsing for supported project/item qualifiers, range normalization for 2 weeks, 1 month, 3 months, Max, and custom dates, zero-filled chart buckets, burn-up completed/total series calculation, and stable data-table rows.
- `crates/api/src/routes/projects.rs`: validate Insights query params and return structured 400/422 errors for invalid filters, invalid ranges, excessive windows, and unsupported chart requests.
- `ProjectInsightsPage.tsx`: wire filter form, range links, custom range dialog, View as data table toggle, chart action menu, empty/no-match states, pending/error states, and refreshed server responses without client-only fake chart data.
- Tests cover filter recomputation, range URLs, custom date validation, no-match chart/table states, chart keyboard labels, table toggle persistence, excessive query bounds, and mobile overflow.

**Verification**: focused Rust chart/filter contract tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-008-phase3-chart-exploration.jpg` when possible, then `make check && make test`; run `make test-e2e` when local DB/dev servers are healthy.

---

## Phase 4: Custom Chart Mutations - authorized users create, edit, and delete shared charts

**Done**: [x]

**Scope**: Add real custom chart lifecycle. Writers/admins can create charts, edit title/description/filter/type/X/Y/grouping/visibility, delete charts, and see updated sidebar entries; readers can view shared charts but cannot mutate them.

**Key changes**:
- `crates/api/src/domain/projects.rs`: implement custom chart helpers with project write checks, chart ownership/visibility rules, field compatibility validation, stale `expectedUpdatedAt` conflicts, revision rows, cache invalidation/snapshots, and audit events.
- `crates/api/src/routes/projects.rs`: expose `POST /api/projects/{project_id}/charts`, `PATCH /api/projects/{project_id}/charts/{chart_id}`, and `DELETE /api/projects/{project_id}/charts/{chart_id}`.
- `web/src/app/api/projects/[projectId]/charts/route.ts` and `charts/[chartId]/route.ts`: forward signed cookies to Rust and preserve standard JSON envelopes.
- `ProjectInsightsPage.tsx`: add create/edit/delete chart dialogs or route-backed panels, chart type selector, filter editor, X/Y/grouping selectors from eligible project fields, visibility-to-viewers toggle, confirmation states, conflict copy, and refreshed chart navigation.
- Tests cover create/edit/delete persistence, reader denial, invalid field/type combinations, stale edit rejection, private chart visibility, audit rows, UI payloads, and no client-only fake updates.

**Verification**: focused Rust mutation contract tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-008-phase4-custom-charts.jpg` when possible, mandatory Editorial banned-value scan, then `make check && make test`.

---

## Phase 5: Sharing, Status Surfacing, and Cross-Project Integration - charts become shareable project artifacts

**Done**: [x]

**Scope**: Finish integration points around charts and status. Shared charts are readable to project viewers, chart links preserve selected project/range/filter state, project status updates surface in lists and side panels where existing Projects pages already render context, and chart mutations leave audit/cache evidence.

**Key changes**:
- `crates/api/src/domain/projects.rs`: add chart share-link metadata, viewer-safe chart lookup by stable id/slug, cache refresh snapshots after relevant item/status changes, and status/chart summary helpers for project list and item side-panel surfaces.
- `crates/api/src/routes/projects.rs`: expose viewer-safe chart share reads through the existing Insights endpoint or `GET /api/projects/{project_id}/charts/{chart_id}` with the same auth/privacy contract.
- `ProjectWorkspacePage.tsx`, project list components, and relevant settings/status panels: surface latest status and chart/Insights links without introducing unrelated Insights analytics or repository Insights behavior.
- `ProjectInsightsPage.tsx`: add copy/share link controls, viewer-visible shared chart badges, stale-cache timestamp copy, and cache-refresh/error states.
- Tests cover shared chart read permissions, private chart denial, status visibility in project lists/side panels, cache refresh metadata, copied link shape, audit rows, and mobile layout.

**Verification**: focused Rust integration contract tests, focused Vitest, focused browser smoke saving `ralph/screenshots/build/projects-008-phase5-sharing-status.jpg` when possible, then `make check && make test`; run `make test-e2e` when local DB/dev servers are healthy.

---

## Phase 6: API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `projects-008` only after Insights reads, Editorial chart UI, filter/range/table exploration, custom chart mutations, sharing/status integration, docs, screenshots, and QA handoff are verified. Do not implement repository Insights, project exports, billing, full analytics dashboards, Highcharts as a dependency unless explicitly installed and justified, or GitHub visual styling as part of this feature.

**Key changes**:
- `web/src/lib/api-docs.ts`: document Project Insights reads, default Burn up chart, filter/range/custom-range query contract, data-table fallback, custom chart create/edit/delete, chart sharing, status surfacing, permissions, privacy, cache snapshots, stale conflicts, audit/log side effects, and standard errors.
- `web/tests/e2e/projects-insights.spec.ts`: final signed-in browser smoke for org and user project Insights covering default chart, filter, range, custom range, data-table toggle, custom chart create/edit/delete, shared chart read-only view, no dead controls, desktop/mobile screenshots, and bounded overflow.
- `ralph/screenshots/build/`: save final evidence screenshots for default Burn up, filter/range, custom range, data table, custom chart form, shared read-only chart, status surfacing, and mobile.
- `qa-hints.json`: append QA targets for large projects, empty/no-date projects, unusual filter syntax, long chart titles, concurrent chart edits, stale cache refreshes, deleted field references, private repository item filtering, reader/admin role drift, shared chart privacy, audit rows, and mobile chart accessibility.
- `build-progress.txt`, `.qrspi/projects-008/structure.md`, and `prd.json`: record evidence and set `projects-008.build_pass=true` only after all phases pass; leave `qa_pass=false`.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.

**Verification**: focused Rust/Vitest/Playwright checks, `make check`, `make test`, `make test-e2e` when local DB/dev servers are healthy, browser screenshots under `ralph/screenshots/build/`, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan with zero matches.
