# Structure Outline: insights-003 Repository Traffic

**Ticket**: `insights-003`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-shell.jsx`, existing `RepositoryInsightsShell`, Pulse and Contributors analytics contracts from `insights-001` and `insights-002`, repository permission patterns from existing repository routes, and commit/file navigation contracts from `commits-001`, `commits-002`, and `repo-005`.
**Date**: 2026-05-05

## Phase 1: Traffic API Contract and Rollup Storage - permissioned analytics data

**Done**: [x]

**Scope**: Add an authenticated read contract for `GET /api/repos/{owner}/{repo}/graphs/traffic` that returns the 14-day traffic dashboard data for users with push access, without replacing the placeholder UI yet. The response should include repository metadata, permission state, UTC date bounds, clone and visitor daily series, total and unique summaries, referrer rows, popular content rows, cache freshness metadata, and structured 403 behavior for users without push access.

**Key changes**:
- `crates/api/migrations/*_repository_traffic_insights.*.sql`: add only missing narrow storage for `repository_traffic_daily`, `repository_referrers_daily`, `repository_popular_content_daily`, and optional `repository_insight_snapshots` cache keys; include repository/date keys, total/unique counts, safe referrer/content dimensions, and bounded freshness metadata.
- `crates/api/src/domain/repositories.rs`: add `RepositoryTrafficView`, `RepositoryTrafficRepository`, `RepositoryTrafficWindow`, `RepositoryTrafficSeriesPoint`, `RepositoryTrafficSummary`, `RepositoryTrafficReferrer`, `RepositoryTrafficContent`, `RepositoryTrafficSnapshot`, and `RepositoryTrafficQuery`.
- `crates/api/src/domain/repositories.rs`: add `repository_traffic_for_actor_by_owner_name(pool, actor_user_id, owner, repo, query)` that preserves public/private repository visibility, enforces push-or-admin access for traffic counts, computes the past 14 UTC days, aggregates clone/view/referrer/content rows, records bounded recent analytics access, and avoids leaking counts in unauthorized responses.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/graphs/traffic`, return standard auth/privacy/error envelopes, and distinguish missing repositories from permissioned 403 traffic denial without exposing traffic data.
- `crates/api/tests/repository_traffic_contract.rs`: seed owners, readers, push collaborators, private/public repositories, traffic daily rows, referrers, popular content, and audit/access rows; assert permission gates, UTC window shape, summary totals, row ordering, cache metadata, unauthorized 403 response shape, private repository privacy, and no secret leakage.

**Verification**: focused Rust contract tests against `opengithub_identity_test`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and `DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Traffic Page and Insights Shell Integration - default analytics render

**Done**: [x]

**Scope**: Implement `/{owner}/{repo}/graphs/traffic` using the existing Editorial Insights shell with Traffic selected. The default page should render permissioned traffic analytics with clone and visitor charts, summary totals, referrers, popular content, UTC/freshness notes, and no inert controls.

**Key changes**:
- `web/src/lib/api.ts`: add typed Traffic DTOs and signed-cookie server fetch helper for the Rust endpoint.
- `web/src/app/[owner]/[repo]/graphs/traffic/page.tsx`: fetch repository and Traffic data server-side, render 403/empty/unavailable states inside `RepositoryInsightsShell`, and preserve repository context.
- `web/src/components/RepositoryInsightsShell.tsx`: ensure Traffic is present and active for `/graphs/traffic` without regressing Pulse or Contributors navigation.
- `web/src/components/RepositoryTrafficPage.tsx`: new Editorial page using `.card`, `.chip`, `.btn`, `.list-row`, `.t-*`, and `var(--*)` tokens only; render compact clone and visitor line charts with accessible data-table fallbacks, summary metric cards, referrer rows, popular content rows, freshness copy, and UTC note.
- `web/src/lib/navigation.ts`: add traffic href helpers plus safe repository-content links for popular paths.
- `web/tests/repository-traffic-page.test.tsx` and `web/tests/e2e/repository-traffic.spec.ts`: cover default render, Insights sidebar active state, summary cards, chart fallback tables, referrer/content destinations, 403 empty state, no `href="#"`, no inert handlers, and a saved desktop screenshot.

**Verification**: focused Vitest and Playwright smoke for `/graphs/traffic`, then `make check && make test`; browser smoke saves `ralph/screenshots/build/insights-003-phase2-traffic-overview.jpg`.

---

## Phase 3: Chart Focus, Tooltip Data, and Safe Link Actions

**Done**: [ ]

**Scope**: Make chart point hover/focus and table interactions complete. Keyboard-focusing or hovering a clone/visitor chart point should reveal exact date, total, and unique values; referrer links should open externally with safe attributes; popular content links should open real repository paths.

**Key changes**:
- `web/src/components/RepositoryTrafficChart.tsx`: client chart component with stable dimensions, roving or native keyboard focus on points, hover/focus tooltip text, visible selected-point details, and data-table fallback for small screens and screen readers.
- `web/src/components/RepositoryTrafficPage.tsx`: wire clone and visitor chart points, summary values, referrer external anchors with `rel="noopener noreferrer"`, popular content links, and empty-state recovery links.
- `web/src/lib/navigation.ts`: add content path helpers that resolve directories/files against existing repository browse routes and preserve branch/default-ref assumptions.
- Extend Rust, Vitest, and Playwright coverage for point labels, tooltip/focus behavior, safe external rel attributes, content href generation, sorted referrers/content, no dead controls, and mobile no-overflow.

**Verification**: focused contract/unit/browser interaction tests, screenshot `ralph/screenshots/build/insights-003-phase3-chart-focus.jpg`, then `make check && make test`; run `make test-e2e` or direct Playwright when local servers are stable.

---

## Phase 4: Permission, Freshness, and Bounded Edge Cases

**Done**: [ ]

**Scope**: Harden Traffic for repositories with no traffic, sparse days, bot/internal traffic exclusions, long referrer domains, long content paths, private repositories, and users without push access. Every visible action should either navigate to a real page or show a truthful unavailable state.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: add bounded rollup logic for sparse/missing daily rows, zero-traffic repositories, long referrer/content values, daily-vs-hourly freshness labels, internal traffic exclusion metadata when available, and deterministic ordering/tie-breaks.
- `web/src/components/RepositoryTrafficPage.tsx`: render no-traffic states, 403 permission state without leaked counts, long-label wrapping, freshness/UTC notes, compact mobile tables, and semantic chips for update cadence.
- `web/tests/repository-traffic-page.test.tsx`: assert edge-case rendering, long domains/paths do not overflow, permission state hides counts, no unsafe HTML, no banned GitHub values/imports, no inert inline handlers, and semantic chip usage.
- `web/tests/e2e/repository-traffic.spec.ts`: exercise push-access view, read-only permission state, chart focus, referrer/content links, empty repository state, and mobile layout.
- `crates/api/tests/repository_traffic_contract.rs`: add coverage for zero/sparse data, sorted ties, long dimensions, private repository privacy, push-access vs read-only collaborators, snapshot refresh, and no secret leakage.

**Verification**: focused Rust edge-case tests, focused Vitest, focused Playwright smoke with screenshot `ralph/screenshots/build/insights-003-phase4-edge-cases.jpg`, then `make check && make test`.

---

## Phase 5: API Docs, Browser Evidence, QA Handoff, and Build Pass

**Done**: [ ]

**Scope**: Finish `insights-003` only after the Traffic API, default page, chart focus/tooltips, safe links, permission states, edge cases, docs, screenshots, and QA handoff are verified. Mark `build_pass=true` only in this phase and leave `qa_pass=false`.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document `GET /api/repos/{owner}/{repo}/graphs/traffic`, including auth/privacy, push-access requirement, 14-day UTC window, hourly/daily freshness, clone/visitor series, referrer rows, popular content rows, cache behavior, and error envelopes.
- `crates/api/tests/repository_traffic_contract.rs`: add final coverage for anonymous/private/public access, push/read/admin permissions, empty traffic, sparse windows, long dimensions, snapshot refresh, invalid input, and no secret leakage.
- `web/tests/repository-traffic-page.test.tsx`: add final assertions for accessible names, keyboard focus order, chart table fallback, safe external links, semantic chips, Editorial token usage, no banned GitHub colors/imports, no `href="#"`, no inert click handlers, and no unsafe HTML.
- `web/tests/e2e/repository-traffic.spec.ts`: full signed-session browser sweep for Insights navigation, chart hover/focus, summary totals, referrer external links, popular content links, empty states, read-only permission state, and mobile no-overflow.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/insights-003/structure.md`, and `prd.json`: record verification evidence and set `insights-003.build_pass=true` only after every phase passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/insights-003-final-*.jpg`.

**Verification**: focused contract/unit/E2E tests, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full `DB_SSL=false CARGO_INCREMENTAL=0 make test`, `make test-e2e` or direct Playwright equivalent if the wrapper stalls, and mandatory Editorial banned-value scan: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
