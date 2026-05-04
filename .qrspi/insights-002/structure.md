# Structure Outline: insights-002 Repository Contributors

**Ticket**: `insights-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-shell.jsx`, existing `RepositoryInsightsShell`, `RepositoryPulsePage`, `RepositoryPulsePeriodSelector`, Pulse API/cache contract from `insights-001`, commit history filters from `commits-001`, and branch/default-ref contracts from `branches-001`.
**Date**: 2026-05-05

## Phase 1: Contributors API Contract and Weekly Rollup Cache - screen-ready analytics data

**Done**: [x]

**Scope**: Add an authenticated read contract for `GET /api/repos/{owner}/{repo}/graphs/contributors` that returns default-branch contributor analytics without replacing the placeholder UI yet. The response should include repository/default-branch metadata, normalized period and date-range bounds, commit-threshold/line-count omission metadata, repository-wide weekly commit totals, contributor weekly rows, per-contributor totals, profile and commit-history hrefs, and cache freshness metadata.

**Key changes**:
- `crates/api/migrations/*_repository_contributors_insights.*.sql`: add only missing narrow storage for `repository_contributors_weekly` or a contributors-specific `repository_insight_snapshots` cache key; include repository/default-branch/period bounds, weekly buckets, totals, and bounded recomputation metadata.
- `crates/api/src/domain/repositories.rs`: add `RepositoryContributorsView`, `RepositoryContributorsRepository`, `RepositoryContributorsPeriod`, `RepositoryContributorsThreshold`, `RepositoryContributorsWeek`, `RepositoryContributorRow`, `RepositoryContributorWeek`, `RepositoryContributorSnapshot`, and `RepositoryContributorsQuery`.
- `crates/api/src/domain/repositories.rs`: add `repository_contributors_for_actor_by_owner_name(pool, actor_user_id, owner, repo, query)` that preserves existing public/private visibility behavior, resolves the default branch through `repository_git_refs`, excludes merge/empty commits, normalizes `period` to the supported chart windows, computes weekly buckets, omits additions/deletions over the commit threshold, and writes bounded snapshot/rollup telemetry.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/graphs/contributors`, validate period/range values, return standard auth/privacy/error envelopes, and avoid leaking private repository analytics to unauthorized users.
- `crates/api/tests/repository_contributors_contract.rs`: seed users, repositories, refs, commits, commit parents, `commit_file_changes`, permissions, bot/unmatched authors, and threshold-size histories; assert default-branch scoping, merge/empty commit exclusion, weekly aggregation, line-count omission, href shape, cache persistence, private repository privacy, invalid query handling, and no secret leakage.

**Verification**: focused Rust contract tests against `opengithub_identity_test`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and `DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Contributors Page and Insights Shell Integration - default analytics render

**Done**: [x]

**Scope**: Implement `/{owner}/{repo}/graphs/contributors` using the existing Editorial Insights shell with Contributors selected. The default page should render the default-branch scope, threshold/line-count explanatory copy, period control affordance, chart action buttons, accessible repository-wide commits-over-time chart, range slider structure, top contributor sections, per-contributor bar charts, and always-available data tables with no inert controls.

**Key changes**:
- `web/src/lib/api.ts`: add typed Contributors DTOs and signed-cookie server fetch helper for the Rust endpoint.
- `web/src/app/[owner]/[repo]/graphs/contributors/page.tsx`: fetch repository and Contributors data server-side, preserve URL search params, and render repository-scoped unavailable/forbidden/empty states inside `RepositoryInsightsShell`.
- `web/src/components/RepositoryContributorsPage.tsx`: new Editorial page using `.card`, `.chip`, `.btn`, `.tabs`, `.list-row`, `.av`, `.kbd`, `.t-*`, and `var(--*)` tokens only; render chart regions with table fallbacks rather than external chart libraries.
- `web/src/lib/navigation.ts`: add contributors href helpers for period/range URLs, profile links, author-filtered commit-history links, and range-preserving route generation.
- `web/tests/repository-contributors-page.test.tsx` and `web/tests/e2e/repository-contributors.spec.ts`: cover default render, Insights sidebar active state, default branch scope, threshold message, main chart/table fallback, contributor sections, concrete profile/commit links, no `href="#"`, no inert handlers, and a saved desktop screenshot.

**Verification**: focused Vitest and Playwright smoke for `/graphs/contributors`, then `make check && make test`; browser smoke saves `ralph/screenshots/build/insights-002-phase2-contributors-overview.jpg`.

---

## Phase 3: Period Controls, Range Sliders, Data Table, and Chart Actions

**Done**: [x]

**Scope**: Make Contributors controls URL-backed and action-complete. Period options and start/end range sliders should constrain visible weekly data, View as data table should open a real accessible table panel, and chart action/export buttons should either download/copy real CSV data or be omitted. No placeholder menus, fake exports, or dead buttons.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: finish period/range normalization, inclusive weekly bucket clipping, invalid range errors, and cache-key partitioning for period plus range.
- `web/src/components/RepositoryContributorsControls.tsx`: client controls for Period menu, range start/end inputs or sliders with stable dimensions, Apply/Clear links, Escape/outside-click handling, and selected-state affordances.
- `web/src/components/RepositoryContributorsDataTable.tsx`: accessible table panel for repository-wide weekly values and contributor weekly values, with keyboard-visible close/toggle state and no unsafe HTML.
- `web/src/components/RepositoryContributorsPage.tsx`: wire chart action buttons to real CSV download/copy behavior, render active filter chips, preserve range in contributor commit links, and keep mobile layout free of overflow.
- Extend Rust, Vitest, and Playwright coverage for each period, range clipping, invalid ranges, data-table toggles, CSV export/copy behavior, keyboard menu behavior, no dead controls, and mobile no-overflow.

**Verification**: focused contract/unit/browser interaction tests, screenshot `ralph/screenshots/build/insights-002-phase3-controls.jpg`, then `make check && make test`; run `make test-e2e` or direct Playwright when local servers are stable.

---

## Phase 4: Contributor Edge Cases and Link-Complete Drilldowns

**Done**: [x]

**Scope**: Harden contributor analytics for repositories with no commits, merge-heavy histories, bot-only activity, unmatched/deleted authors, slash-containing default branches, very large histories, and public/private permission differences. Every contributor avatar/login/commit total/chart row should navigate to a real page or present a truthful unavailable state.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: add bounded rollup logic for large histories, bot detection, unmatched authors, deleted users, empty repositories, missing default branch recovery, and merge/empty commit exclusion.
- `web/src/components/RepositoryContributorsPage.tsx`: render bot/unmatched chips, empty repository recovery, line-count omission banner, long-login wrapping, compact mobile charts/tables, and concrete profile/commit-history destinations.
- `web/tests/repository-contributors-page.test.tsx`: assert edge-case rendering, table fallback values, long labels/range text do not overflow, no unsafe HTML, no banned GitHub values/imports, no inert inline handlers, and semantic chips.
- `web/tests/e2e/repository-contributors.spec.ts`: exercise contributor profile links, commit-count links, data table, range controls, empty repository state, threshold omission banner, and mobile layout.
- `crates/api/tests/repository_contributors_contract.rs`: add coverage for merge commit exclusion, empty commits, threshold line omission, bot/unmatched authors, slash-containing default branch hrefs, public/private privacy, and snapshot refresh.

**Verification**: focused Rust edge-case tests, focused Vitest, focused Playwright smoke with screenshot `ralph/screenshots/build/insights-002-phase4-edge-cases.jpg`, then `make check && make test`.

---

## Phase 5: API Docs, Browser Evidence, QA Handoff, and Build Pass

**Done**: [ ]

**Scope**: Finish `insights-002` only after the Contributors API, default page, period/range controls, chart data-table/export actions, contributor drilldowns, edge cases, docs, screenshots, and QA handoff are verified. Mark `build_pass=true` only in this phase and leave `qa_pass=false`.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document `GET /api/repos/{owner}/{repo}/graphs/contributors`, including auth/privacy, default branch scope, period and range query params, merge/empty commit exclusion, threshold line-count omission, weekly response shape, contributor rows, cache behavior, and error envelopes.
- `crates/api/tests/repository_contributors_contract.rs`: add final coverage for anonymous/private/public access, supported periods/ranges, empty repositories, large rollups, deleted/missing actors, snapshot refresh, invalid input, and no secret leakage.
- `web/tests/repository-contributors-page.test.tsx`: add final assertions for accessible names, keyboard focus order, semantic chips, Editorial token usage, no banned GitHub colors/imports, no `href="#"`, no inert click handlers, no unsafe HTML, and table accessibility.
- `web/tests/e2e/repository-contributors.spec.ts`: full signed-session browser sweep for Insights navigation, period/range controls, chart actions, data table, contributor links, commit-total links, empty states, threshold state, read-only permission state, and mobile no-overflow.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/insights-002/structure.md`, and `prd.json`: record verification evidence and set `insights-002.build_pass=true` only after every phase passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/insights-002-final-*.jpg`.

**Verification**: focused contract/unit/E2E tests, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full `DB_SSL=false CARGO_INCREMENTAL=0 make test`, `make test-e2e` or direct Playwright equivalent if the wrapper stalls, and mandatory Editorial banned-value scan: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
