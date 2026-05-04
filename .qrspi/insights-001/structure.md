# Structure Outline: insights-001 Repository Pulse

**Ticket**: `insights-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, `design/project/og-shell.jsx`, current placeholder `web/src/app/[owner]/[repo]/pulse/page.tsx`, existing repository shell/navigation contracts, existing commit history/detail contracts from `commits-001` and `commits-002`, existing pull request/issue/release read contracts, and branch-directory patterns from `branches-001`.
**Date**: 2026-05-05

## Phase 1: Pulse API Contract and Snapshot Cache - screen-ready activity aggregates

**Done**: [x]

**Scope**: Add an authenticated read contract for `GET /api/repos/{owner}/{repo}/pulse` that computes the Pulse summary for one normalized period without replacing the placeholder UI yet. The response should include repository metadata, period bounds, overview counts, linked metric destinations, a natural-language summary payload, top committers, releases, merged PRs, issue activity, and cache freshness metadata.

**Key changes**:
- `crates/api/migrations/*_repository_pulse_insights.*.sql`: add only missing narrow tables/columns for `repository_insight_snapshots` and optional `recent_insight_views`; include a stable period/cache key and JSON snapshot payload if the current schema does not already cover it.
- `crates/api/src/domain/repositories.rs`: add `RepositoryPulseView`, `RepositoryPulseRepository`, `RepositoryPulsePeriod`, `RepositoryPulseMetric`, `RepositoryPulseSummary`, `RepositoryPulseCommitter`, `RepositoryPulseActivityItem`, `RepositoryPulseSnapshot`, and `RepositoryPulseQuery`.
- `crates/api/src/domain/repositories.rs`: add `repository_pulse_for_actor_by_owner_name(pool, actor_user_id, owner, repo, query)` that preserves public/private visibility, normalizes `period` to `24h`, `3d`, `1w`, or `1m`, computes inclusive date bounds, aggregates commits/files/additions/deletions/authors, PR/issue/release activity, and writes or refreshes bounded snapshot/view telemetry.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/pulse`, validate `period`, return standard auth/privacy/error envelopes, and avoid leaking private repository counts to unauthorized users.
- `crates/api/tests/repository_pulse_contract.rs`: seed repositories, permissions, commits, `commit_file_changes`, users, pull requests, issues, releases, and activity events; assert period normalization, aggregate counts, top committer ordering, filtered metric hrefs, release/PR/issue activity shape, snapshot persistence, private repository privacy, and no secret leakage.

**Verification**: focused Rust contract tests against `opengithub_identity_test`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and `DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Insights Shell and Pulse Overview - default page renders real data

**Done**: [x]

**Scope**: Replace the placeholder `/{owner}/{repo}/pulse` route with a real Editorial Insights page backed by the Phase 1 API. The default period should show the Insights sidebar, Pulse heading/date range, overview cards, linked counts, summary sentence, top committers chart, and compact activity lists with no inert controls.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed Pulse DTOs and signed-cookie fetch helper for the Rust endpoint.
- `web/src/app/[owner]/[repo]/pulse/page.tsx`: fetch repository plus Pulse data server-side, preserve URL search params, and render repository-scoped unavailable/forbidden/empty states inside the existing app/repository shell.
- `web/src/components/RepositoryInsightsShell.tsx`: new shared Editorial Insights layout with left sidebar links for Pulse, Contributors, Community standards, Commits, Code frequency, Dependency graph, Network, Forks, Actions usage metrics, and Actions performance metrics. Use `.card`, `.chip`, `.btn`, `.tabs`, `.list-row`, `.av`, `.t-*`, and `var(--*)` tokens only.
- `web/src/components/RepositoryPulsePage.tsx`: render exact date range, overview metric cards, linked counts for merged/open PRs and closed/new issues, summary sentence, accessible Top Committers bars with table fallback, avatar/profile links, release list, merged PR list, and empty states that link to existing commit history/issues/pulls routes.
- `web/src/lib/navigation.ts`: add Pulse period, filtered PR/issue, commit range, release, profile, and Insights-sidebar href helpers. Preserve the active repository tab for `/pulse`, `/graphs/*`, `/network`, and `/forks`.
- `web/tests/repository-pulse-page.test.tsx` and `web/tests/e2e/repository-pulse.spec.ts`: cover default render, sidebar links, overview cards, chart/table fallback, activity destinations, empty states, no `href="#"`, no placeholder handlers, and a saved desktop screenshot.

**Verification**: focused Vitest and Playwright smoke for `/pulse`, then `make check && make test`; browser smoke saves `ralph/screenshots/build/insights-001-phase2-pulse-overview.jpg`.

---

## Phase 3: Period Selector and Metric Navigation - query state drives aggregates

**Done**: [x]

**Scope**: Make the Period control functional for `24h`, `3d`, `1w`, and `1m`, with URL-backed state and metric cards that navigate to real filtered issue or pull request pages. Changing periods should reload the server-backed Pulse metrics, keep the exact date range visible, and preserve repository context.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: finish period-specific aggregation, cache invalidation/freshness metadata, bounded empty-period recovery, and validation errors for unsupported period values.
- `web/src/components/RepositoryPulsePeriodSelector.tsx`: client component with accessible menu button, four concrete period links, selected state, Escape/outside-click close behavior, and no fake loading states.
- `web/src/components/RepositoryPulsePage.tsx`: render active period chip, exact range copy, metric-card hrefs for filtered PR/issue lists, clear recovery links for empty periods, and stable responsive grid dimensions.
- `web/src/lib/navigation.ts`: add helpers for period-preserving Pulse URLs and filtered issue/PR URLs such as merged PRs, open PRs, closed issues, and new issues over the selected range.
- Extend Rust, Vitest, and Playwright coverage for all period values, invalid period validation, URL round trips, metric-card destinations, menu keyboard behavior, empty-period recovery, and mobile no-overflow.

**Verification**: focused contract/unit/browser interaction tests, screenshot `ralph/screenshots/build/insights-001-phase3-period-selector.jpg`, then `make check && make test`.

---

## Phase 4: Committer Chart, Activity Links, and Bounded Edge Cases

**Done**: [ ]

**Scope**: Harden the interactive Pulse content so every chart, avatar, activity item, and commit range destination either opens a real page or presents a truthful unavailable state. This phase should cover repositories with no commits, bot-only activity, deleted release authors, very large file-change counts, and public/private permission differences.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: add bounded rollup logic for large commit/file-change sets, detached/unmatched commit authors, deleted users, bot markers when available, release/pr/issue activity ordering, and stable commit range href metadata.
- `web/src/components/RepositoryPulsePage.tsx`: add accessible chart/table toggle or always-visible table fallback, sorted committer rows with profile and commit-filter links, activity rows with concrete release/PR/issue/commit-range links, unavailable author labels, and semantic chips for activity state.
- Same-origin mutation route handlers are not required because Pulse is read-only, but any copy/share/export affordance introduced must be fully wired or omitted.
- `web/tests/repository-pulse-page.test.tsx`: assert table fallback values, long names/range text do not overflow, activity ordering, no unsafe HTML, no banned GitHub values/imports, and no inert inline handlers.
- `web/tests/e2e/repository-pulse.spec.ts`: exercise chart/table navigation, avatar/profile links, commit-range links, release/PR/issue links, empty repository recovery, and mobile layout.

**Verification**: focused Rust edge-case tests, focused Vitest, focused Playwright smoke with screenshot `ralph/screenshots/build/insights-001-phase4-activity-links.jpg`, then `make check && make test`; run `make test-e2e` or direct Playwright when local servers are stable.

---

## Phase 5: API Docs, Browser Evidence, QA Handoff, and Build Pass

**Done**: [ ]

**Scope**: Finish `insights-001` only after the Pulse API, Insights shell, overview metrics, period selector, metric links, top committers chart/table, activity links, docs, screenshots, and QA handoff are verified. Mark `build_pass=true` only in this phase and leave `qa_pass=false`.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document `GET /api/repos/{owner}/{repo}/pulse`, including auth/privacy, period values, date bounds, snapshot caching, response shape, metric hrefs, top committers, activity lists, and error envelopes.
- `crates/api/tests/repository_pulse_contract.rs`: add final coverage for public/private/anonymous access, each period, no-activity repositories, large rollups, deleted/missing actors, snapshot refresh, invalid input, and no secret leakage.
- `web/tests/repository-pulse-page.test.tsx`: add final assertions for accessible names, keyboard focus order, semantic chips, Editorial token usage, no banned GitHub colors/imports, no `href="#"`, no inert click handlers, no unsafe HTML, and table fallback accessibility.
- `web/tests/e2e/repository-pulse.spec.ts`: full signed-session browser sweep for sidebar navigation, period menu, overview metric links, top committer links, table fallback, release/PR/issue links, commit-range links, empty states, read-only permission state, and mobile no-overflow.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/insights-001/structure.md`, and `prd.json`: record verification evidence and set `insights-001.build_pass=true` only after every phase passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/insights-001-final-*.jpg`.

**Verification**: focused contract/unit/E2E tests, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full `DB_SSL=false CARGO_INCREMENTAL=0 make test`, `make test-e2e` or direct Playwright equivalent if the wrapper stalls, and mandatory Editorial banned-value scan: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
