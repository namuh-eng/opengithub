# Structure Outline: projects-003 Projects v2 Board and Roadmap Layouts

**Ticket**: `projects-003`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-shell.jsx`, `ralph/screenshots/inspect/projects-view-menu-board.jpg`, `ralph/screenshots/inspect/projects-roadmap-table.jpg`, existing Projects list/workspace contracts from `.qrspi/projects-001/structure.md` and `.qrspi/projects-002/structure.md`, `crates/api/src/domain/projects.rs`, `crates/api/src/routes/projects.rs`, `web/src/components/ProjectWorkspacePage.tsx`, and Projects docs under `target-docs/content/issues/planning-and-tracking-with-projects/customizing-views-in-your-project/`.
**Date**: 2026-05-06

## Phase 1: Layout-Aware Workspace Read Contract - table, board, and roadmap views are distinguishable

**Done**: [x]

**Scope**: Extend the existing Projects workspace read API so saved views can declare `table`, `board`, or `roadmap` layout and return screen-ready configuration for layout switching without changing item values yet. This phase makes layout metadata, eligible fields, persisted settings, keyboard hints, and permission-gated controls testable through one read contract.

**Key changes**:
- `crates/api/migrations/`: add additive layout metadata where absent, including view layout enum/configuration keys, `project_board_column_settings`, `project_roadmap_settings`, indexes by project/view/field, and compatible defaults for existing table views.
- `crates/api/src/domain/projects.rs`: extend `ProjectWorkspaceView` and related DTOs with `layout`, `layoutChoices`, `boardConfig`, `roadmapConfig`, eligible column/swimlane/date/marker fields, zoom options, viewer layout capabilities, and validation helpers for layout-specific settings.
- `crates/api/src/routes/projects.rs`: keep `GET /api/projects/{project_id}/workspace` as the canonical read endpoint while validating `view`, preserving filter/slice/sort state, and returning layout-specific unavailable reasons instead of leaking private linked items.
- `web/src/lib/api.ts`: add typed board and roadmap DTOs plus enum-safe layout fields used by the upcoming UI phases.
- `crates/api/tests/projects_workspace_layout_contract.rs`: cover default table compatibility, board/roadmap view reads, eligible field filtering, private linked item hiding, invalid layout config fallback, permission flags, and no-secret error envelopes.

**Verification**: focused Rust contract tests, `cargo fmt --all`, `cargo check -p opengithub-api --tests`, web TypeScript for DTOs, mandatory Editorial banned-value scan, then `make check && make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Layout Switching and View Menu - authorized users can persist active layout settings

**Done**: [x]

**Scope**: Make the Project workspace View menu operational for layout selection. Authorized users can switch between Table, Board, and Roadmap, save layout settings to the selected view, and keep existing filter/slice/sort state; read-only viewers can inspect choices but cannot persist changes.

**Key changes**:
- `crates/api/src/domain/projects.rs`: add `update_project_view_layout` with permission checks, stale `expectedUpdatedAt`, supported layout enum validation, compatible field validation, and preservation of existing table state.
- `crates/api/src/routes/projects.rs`: expose `PATCH /api/projects/{project_id}/views/{view_id}/layout` returning the refreshed workspace and standard validation/conflict envelopes.
- `web/src/app/api/projects/[projectId]/views/[viewId]/layout/route.ts`: same-origin cookie-forwarding proxy to the Rust API.
- `web/src/components/ProjectWorkspacePage.tsx`: replace disabled layout controls with an Editorial View menu showing Table, Board, Roadmap, keyboard hints `t`, `b`, `r`, Fields, Column by, Swimlanes, Sort by, Field sum, and Slice by rows; submit real PATCH requests with pending, success, error, and canonical redirect states.
- `web/tests/project-workspace-page.test.tsx`: cover layout menu opening, keyboard hint copy, Table/Board/Roadmap PATCH payloads, disabled read-only controls, stale/server errors, URL state preservation, and no dead links or inert handlers.

**Verification**: focused Rust contract tests, focused Vitest, web TypeScript, focused Biome, mandatory Editorial banned-value scan, then `make check && make test`; run a focused Playwright smoke when seeded DB credentials are available.

---

## Phase 3: Board Layout and Column Moves - cards render and column changes persist

**Done**: [x]

**Scope**: Render a horizontally scrollable Editorial board from a single-select/status/iteration column field, including item cards, empty columns, column limits, swimlane grouping, and real card moves that update the backing project item field value.

**Key changes**:
- `crates/api/src/domain/projects.rs`: add board projection helpers that group visible items into columns and optional swimlanes, compute item counts, enforce column-limit warnings, and reuse Phase 4 `projects-002` field mutation rules for move writes.
- `crates/api/src/routes/projects.rs`: expose `PATCH /api/projects/{project_id}/items/{item_id}/board-position` or extend the existing item-field endpoint with board move metadata; validate target column option, target swimlane compatibility, manual position, archived items, permissions, and audit/item-event side effects.
- `web/src/components/ProjectWorkspacePage.tsx`: add Board rendering mode with scrollable columns, count/limit headers, cards with title/repo/labels/assignees/field chips, Add item affordances wired to the existing item-add flows, empty-column toggles, and accessible Move-to-column controls where drag-and-drop is not yet available.
- `web/src/lib/api.ts` and same-origin route handlers: add typed board move helper forwarding signed cookies.
- Tests cover board grouping, empty columns, over-limit warning chips, swimlane sections, move payloads, permission denial, invalid target column errors, refreshed card placement, add-item reuse, and mobile horizontal overflow behavior.

**Verification**: focused Rust contract tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-003-phase3-board-layout.jpg` when the test DB is healthy, then `make check && make test`; run `make test-e2e` when stable.

---

## Phase 4: Roadmap Layout and Timeline Controls - date and iteration timelines render with saved zoom/markers

**Done**: [x]

**Scope**: Render the Roadmap layout from selected start/target date or iteration fields and make roadmap settings real. Users can choose date fields, marker sets, and Month/Quarter/Year zoom, with grouped rows aligned to a timeline while existing filters/sort/slice remain active.

**Key changes**:
- `crates/api/src/domain/projects.rs`: add roadmap projection helpers for start/end dates, iteration spans, milestone and item-date markers, zoom buckets, grouped row lanes, missing-date warnings, and settings validation.
- `crates/api/src/routes/projects.rs`: expose `PATCH /api/projects/{project_id}/views/{view_id}/roadmap-settings` with project write/admin checks, stale update protection, date/iteration field compatibility, marker enum validation, zoom validation, audit events, and refreshed workspace responses.
- `web/src/app/api/projects/[projectId]/views/[viewId]/roadmap-settings/route.ts`: same-origin cookie-forwarding proxy.
- `web/src/components/ProjectWorkspacePage.tsx`: add Roadmap rendering mode with left grouped rows, right timeline grid, item bars, marker selector, date-field selector, Month/Quarter/Year segmented control, pane resize affordances with stable responsive bounds, and honest disabled states for unsupported direct bar dragging.
- Tests cover date-field selection, marker toggles, zoom PATCH payloads, timeline bucket labels, missing-date rows, iteration bars, grouped row alignment, permission-disabled controls, and no text overlap on mobile.

**Verification**: focused Rust contract tests, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/projects-003-phase4-roadmap-layout.jpg` when possible, then `make check && make test`; run `make test-e2e` when stable.

---

## Phase 5: API Docs, Browser Smoke, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `projects-003` only after layout read/write contracts, layout switching, board moves, roadmap settings, docs, screenshots, and QA handoff are verified. Do not implement Projects insights charts, field administration, workflow automation, project access settings, exports, or full pointer-based drag-and-drop beyond accessible move controls.

**Key changes**:
- `web/src/lib/api-docs.ts`: document workspace layout reads, `PATCH /api/projects/{project_id}/views/{view_id}/layout`, board position/field updates, and roadmap settings endpoints, including permissions, validation rules, stale conflicts, layout compatibility, side effects, and standard errors.
- `web/tests/e2e/projects-board-roadmap.spec.ts`: final desktop/mobile smoke for layout switching, board column views, column move, over-limit warning, swimlane grouping, roadmap field/marker/zoom controls, URL state preservation, read-only disabled state, no dead controls, and no horizontal overflow outside intended board/timeline scrollers.
- `ralph/screenshots/build/`: save final evidence screenshots for View menu, board default, board over-limit/swimlane, roadmap month/quarter/year zoom, permission-disabled state, and mobile.
- `qa-hints.json`: append QA targets for large boards, many swimlanes, hidden private linked repositories, concurrent column moves, stale layout saves, invalid date ranges, timezone/date-boundary rendering, keyboard layout shortcuts, and mobile timeline scrolling.
- `build-progress.txt`, `.qrspi/projects-003/structure.md`, and `prd.json`: record evidence and set `projects-003.build_pass=true` only after all phases pass; leave `qa_pass=false`.

**Verification**: focused Rust/Vitest/Playwright checks, `make check`, `make test`, `make test-e2e` when local DB/dev servers are healthy, browser screenshots under `ralph/screenshots/build/`, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan with zero matches.
