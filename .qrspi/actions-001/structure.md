# Structure Outline: Repository Actions All-Workflows Run List

**Ticket**: `actions-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/og-screens-4.jsx`, `target-docs/onboarding-flow.md`, current `crates/api/src/domain/actions.rs`, current `crates/api/src/routes/actions.rs`, current `crates/api/migrations/202604300004_automation_delivery.up.sql`, current `web/src/app/[owner]/[repo]/actions/page.tsx`, and current repository shell/navigation tests.
**Date**: 2026-05-02

## Phase 1: Actions Dashboard Read Contract - workflows, runs, and summaries are queryable

**Done**: [x]

**Scope**: Build the permission-aware backend contract that the `/actions` page needs before replacing the placeholder UI. A repository reader can fetch the Actions rail, recent runs, available filter options, run/job summary counts, and empty-workflow state from real tables.

**Key changes**:
- `crates/api/migrations/*_actions_dashboard.*.sql`: add only missing support tables/columns needed for the list contract, such as run display titles, optional pull request/commit references, pinned workflow order, and per-user recent Actions filter telemetry; preserve the existing `actions_workflows`, `workflow_runs`, `workflow_jobs`, and `workflow_steps` contracts.
- `crates/api/src/domain/actions.rs`: add `ActionsDashboard`, `ActionsWorkflowRailItem`, `ActionsRunListItem`, `ActionsRunFilters`, `ActionsRunFilterOptions`, and helpers that join workflows, runs, users, refs, job counts, and repository visibility into a stable browser DTO.
- `crates/api/src/routes/actions.rs`: add or extend `GET /api/repos/{owner}/{repo}/actions/dashboard` and/or `GET /api/repos/{owner}/{repo}/actions/runs` query params for `q`, `workflow`, `event`, `status`, `branch`, `actor`, `page`, and `pageSize`; return standard list/error envelopes and no stack/cookie leakage.
- `crates/api/tests/api_repository_actions_dashboard_contract.rs`: seed public/private repositories, active/disabled workflows, multiple run statuses/conclusions, actors, branches, and jobs; assert permission filtering, pagination clamps, status vocabulary, filter option shape, and empty repository response.

**Verification**: focused Rust Actions dashboard contract tests, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check`, and same-env `make test`. Browser smoke is optional in this API-first phase.

---

## Phase 2: Editorial Actions Page - real rail and run list replace the placeholder

**Done**: [ ]

**Scope**: Replace `RepositoryFeaturePage` on `/{owner}/{repo}/actions` with the Editorial all-workflows screen. The page fetches the Phase 1 contract server-side, renders a left Actions rail, dense run rows, status chips/icons, run counts, and the no-workflows template cards with a working New workflow CTA.

**Key changes**:
- `web/src/lib/api.ts`: add typed `getRepositoryActionsDashboard({ owner, repo, searchParams })` with cookie forwarding, error envelope handling, and DTOs matching Phase 1.
- `web/src/app/[owner]/[repo]/actions/page.tsx`: fetch real data and render an Actions-specific component while preserving repository tab activation and auth/permission errors.
- `web/src/components/RepositoryActionsPage.tsx`: new client component using `.btn`, `.chip`, `.card`, `.input`, `.list-row`, `.t-*`, `var(--*)` tokens, and the Editorial layout language; include workflow rail, pinned workflow indicators, Show more workflows, run rows, branch pills, actor avatars, duration/live state, kebab menu, and empty template CTA.
- `web/tests/repository-actions.test.tsx`: cover rail rendering, row navigation to `/actions/runs/{run_id}`, empty template CTA, status/conclusion presentation, accessible names, no `href="#"`, no inert buttons, and Editorial-token-only styling.
- `web/tests/e2e/repository-actions.spec.ts`: signed-session browser smoke for a seeded repository with workflows/runs and a no-workflows repository; save `ralph/screenshots/build/actions-001-phase2-actions-list.jpg`.

**Verification**: focused Actions Vitest, focused Actions Playwright smoke, `make check`, `make test`, and focused same-env `make test-e2e`.

---

## Phase 3: Filters, Workflow Scoping, and Recent View Telemetry - URL-backed controls work

**Done**: [ ]

**Scope**: Make every filter control interactive and data-backed. Typing in the run filter updates the URL and result list; Workflow/Event/Status/Branch/Actor controls open searchable select panels; selecting a rail workflow scopes the list; recent filter/view telemetry is persisted for signed-in users without blocking the page.

**Key changes**:
- `crates/api/src/domain/actions.rs`: add validated filter parsing, status normalization for the full PRD vocabulary (`action_required`, `cancelled`, `completed`, `failure`, `in_progress`, `neutral`, `queued`, `skipped`, `stale`, `success`, `timed_out`, `waiting`), searchable option queries, and best-effort telemetry write helpers.
- `crates/api/src/routes/actions.rs`: accept filter params consistently on dashboard/list routes and add a narrow `POST /api/repos/{owner}/{repo}/actions/recent-view` endpoint for recent filter telemetry with read permission.
- `web/src/components/RepositoryActionsPage.tsx`: add debounced search form, URL-preserving filter panel state, searchable select-panel dialogs, selected filter chips with remove actions, workflow rail scoping, invalid-filter recovery links, and loading/empty states that do not dead-end.
- `web/tests/repository-actions.test.tsx`: cover search URL updates, each filter panel, status vocabulary, workflow scoping, clear filters, invalid query recovery, and telemetry payload shape.
- `web/tests/e2e/repository-actions.spec.ts`: smoke search/filter/workflow-scope flows and save `ralph/screenshots/build/actions-001-phase3-filters.jpg`.

**Verification**: focused Rust filter/telemetry tests, focused Actions Vitest, focused Actions Playwright smoke, `make check`, `make test`, and focused same-env `make test-e2e`.

---

## Phase 4: Management Navigation and API Docs - every Actions link has a real destination

**Done**: [ ]

**Scope**: Finish the all-workflows page navigation contract. Management links for Caches, Deployments, Attestations, Usage metrics, and Performance metrics resolve to real repository workspace pages or documented not-yet-implemented states, and API docs describe the Actions dashboard/run-list contract.

**Key changes**:
- `web/src/app/[owner]/[repo]/actions/{caches,deployments,attestations,usage,performance}/page.tsx`: add thin repository workspace pages with concrete back-links, honest unavailable/empty states, and no placeholder click handlers.
- `web/src/components/RepositoryActionsPage.tsx`: wire Management links, run row kebab actions, Show more workflows behavior, and New workflow CTA to concrete destinations or gated future-feature surfaces.
- `web/src/lib/api-docs.ts` and `/docs/api`: document Actions dashboard, workflows list/create, workflow runs list/create/read/update, filter params, status vocabulary, permissions, and error envelopes.
- `web/tests/api-docs.test.tsx`, `web/tests/repository-actions.test.tsx`, and navigation route tests: assert docs anchors, management links, active repository tab behavior, and no dead links.
- `web/tests/e2e/repository-actions.spec.ts`: browser-smoke management navigation and docs link from the Actions page.

**Verification**: focused docs/navigation Vitest, focused Actions Playwright management smoke, `make check`, `make test`, and same-env `make test-e2e`.

---

## Phase 5: Final Actions Guardrails and QA Handoff - complete actions-001 safely

**Done**: [ ]

**Scope**: Harden the full Actions list surface before setting `actions-001.build_pass=true`. Validate permissions, empty states, filter combinations, mobile layout, visual compliance, browser evidence, QA hints, and bookkeeping.

**Key changes**:
- `crates/api/tests/api_repository_actions_dashboard_contract.rs`: final matrix for anonymous/private access, public repository readers, disabled workflows, zero workflows, all run statuses/conclusions, branch/actor/event filters, pagination bounds, and response redaction.
- `web/tests/repository-actions.test.tsx` and `web/tests/e2e/repository-actions.spec.ts`: final dead-control sweep, row navigation, filter combinations, management links, no horizontal overflow, keyboard/focus behavior, and screenshots `ralph/screenshots/build/actions-001-phase5-final-desktop.jpg` and `ralph/screenshots/build/actions-001-phase5-final-mobile.jpg`.
- Mandatory Editorial banned-value scan before commit: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`: append honest notes for real runner execution not being part of this feature, seeded status/check data limits, filter edge cases, and management pages that intentionally remain thin.
- `prd.json`: set only `actions-001.build_pass` to `true` after every phase passes; leave `qa_pass=false`.
- `build-progress.txt`: append final summary, verification evidence, changed files, and known risks.

**Verification**: focused Actions Rust contract tests, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session make test-e2e`, and mandatory Editorial banned-value scan using the local compatible `rg -n -e` form.
