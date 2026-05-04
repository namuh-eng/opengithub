# Structure Outline: insights-004 Repository Network and Forks

**Ticket**: `insights-004`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-shell.jsx`, existing `RepositoryInsightsShell`, Insights analytics contracts from `insights-001` through `insights-003`, fork creation/storage contracts from `repo-003`, existing `repository_forks` storage, and repository/profile navigation patterns from `profiles-001`, `orgs-001`, and `pulls` fork comparison flows.
**Date**: 2026-05-05

## Phase 1: Network API Contract and Projection Cache - screen-ready fork graph data

**Done**: [x]

**Scope**: Add authenticated read contracts for the repository Network and Forks data without replacing the placeholder UI yet. `GET /api/repos/{owner}/{repo}/network` should return the 50 most recently pushed readable forks, upstream repository metadata, projection freshness, graph/tree hrefs, and daily update copy. `GET /api/repos/{owner}/{repo}/forks` should return filterable fork rows plus saved-default metadata.

**Key changes**:
- `crates/api/migrations/*_repository_network_forks.*.sql`: add only missing narrow storage for `repository_network_forks` projection rows and `saved_fork_filter_defaults`; include source/fork repository ids, pushed-at projection timestamps, star/fork/issue/PR counts, classification flags, cache freshness, actor/default keys, and unique constraints.
- `crates/api/src/domain/repositories.rs`: add `RepositoryNetworkView`, `RepositoryNetworkForkNode`, `RepositoryForksView`, `RepositoryForkRow`, `RepositoryForkFilters`, `RepositoryForkDefaults`, `RepositoryForkSort`, `RepositoryForkType`, and freshness metadata types.
- `crates/api/src/domain/repositories.rs`: add `repository_network_for_actor_by_owner_name(pool, actor_user_id, owner, repo)` and `repository_forks_for_actor_by_owner_name(pool, actor_user_id, owner, repo, filters)` that preserve public/private visibility, include only readable forks, classify active/inactive/archived/starred forks, sort deterministically, and upsert bounded projection rows from existing repository/fork data.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/network` and `GET /api/repos/{owner}/{repo}/forks`, validate period/type/sort query values, return standard auth/privacy/error envelopes, and avoid leaking private fork metadata.
- `crates/api/tests/repository_network_contract.rs`: seed source repositories, forks, unreadable private forks, stars, issues, pull requests, refs/commits, and saved defaults; assert projection limits, readable-only behavior, filter/sort results, href shape, freshness metadata, private repository privacy, invalid query handling, and no secret leakage.

**Verification**: focused Rust contract tests against `opengithub_identity_test`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and `DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Network Page and Insights Shell Integration - default graph render

**Done**: [x]

**Scope**: Implement `/{owner}/{repo}/network` using the existing Editorial Insights shell with Network selected. The page should show the Network graph heading, explanatory copy about recent commits to the repository network, daily update/freshness note, a graph/list representation of recent forks, and real links into fork repositories, tree views, and the Forks page.

**Key changes**:
- `web/src/lib/api.ts`: add typed Network DTOs and signed-cookie server fetch helper for the Rust endpoint.
- `web/src/app/[owner]/[repo]/network/page.tsx`: fetch repository and Network data server-side, render unavailable/empty states inside `RepositoryInsightsShell`, and preserve repository context.
- `web/src/components/RepositoryNetworkPage.tsx`: new Editorial page using `.card`, `.chip`, `.btn`, `.list-row`, `.av`, `.t-*`, and `var(--*)` tokens only; render upstream/fork relationship rows, compact graph rails or nested list structure, daily update note, freshness chips, and no dead controls.
- `web/src/lib/navigation.ts`: add network and fork tree href helpers that preserve slash-containing refs and repository owner/name encoding.
- `web/tests/repository-network-page.test.tsx` and `web/tests/e2e/repository-network.spec.ts`: cover default render, Insights sidebar active state, graph/list rows, fork owner/repository/tree links, empty state, no `href="#"`, no inert handlers, no banned GitHub values/imports, and a saved desktop screenshot.

**Verification**: focused Vitest and Playwright smoke for `/network`, then `make check && make test`; browser smoke saves `ralph/screenshots/build/insights-004-phase2-network.jpg`.

---

## Phase 3: Forks Page Filters, Sorting, and Saved Defaults

**Done**: [x]

**Scope**: Implement `/{owner}/{repo}/forks` with URL-backed Period, Repository type, and Sort menus plus a real Save defaults flow. Filter changes should update list content and URL state; the defaults button should save actor-scoped defaults only when state differs.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: finish period/type/sort normalization, saved-default comparison, and write support through `save_repository_fork_defaults(pool, actor_user_id, repository_id, filters)`.
- `crates/api/src/routes/repositories.rs`: add `PUT /api/repos/{owner}/{repo}/forks/defaults` with auth, validation, standard error envelopes, and no repository mutation beyond actor defaults.
- `web/src/lib/api.ts`: add Forks DTOs, fork list fetch helper, and saved-defaults mutation helper.
- `web/src/app/[owner]/[repo]/forks/page.tsx` and `web/src/components/RepositoryForksPage.tsx`: render Switch to tree view, Period/Repository type/Sort menus, Defaults Saved/Save defaults action, active filter chips, and dense fork rows with avatars, names, metrics, created/updated times, and concrete links.
- `web/src/components/RepositoryForkFilters.tsx`: client controls for accessible menus, Escape/outside-click handling, selected-state affordance, pending/error/success feedback for saving defaults, and no fake buttons.
- Extend Rust, Vitest, and Playwright coverage for query filters, sort ordering, saved defaults, keyboard menu behavior, action feedback, no dead controls, and mobile no-overflow.

**Verification**: focused contract/unit/browser interaction tests, screenshot `ralph/screenshots/build/insights-004-phase3-forks-filters.jpg`, then `make check && make test`; run `make test-e2e` or direct Playwright when local servers are stable.

---

## Phase 4: Fork Metadata Edge Cases and Link-Complete Drilldowns

**Done**: [ ]

**Scope**: Harden Network/Forks for repositories with no forks, unreadable private forks, archived/stale forks, slash-containing default branches, long owner/repository names, ties in metrics, and users without saved defaults. Every visible fork row, metric, owner avatar/name, repository name, and tree/network action should navigate to a real page or present a truthful unavailable state.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: add deterministic tie-breaks, active/inactive period bounds, archived/starred classification, metric counts from existing stars/issues/pulls/forks tables, missing default-branch recovery, and readable-only projection behavior.
- `web/src/components/RepositoryNetworkPage.tsx` and `web/src/components/RepositoryForksPage.tsx`: render no-forks recovery links, stale/archived/starred chips, long-label wrapping, compact mobile rows, safe relative time text, and concrete owner/repository/tree/issue/PR destinations.
- `web/tests/repository-network-page.test.tsx` and `web/tests/repository-forks-page.test.tsx`: assert edge-case rendering, long names do not overflow, private forks are omitted, semantic chips are used, no unsafe HTML, no banned GitHub values/imports, and no inert inline handlers.
- `web/tests/e2e/repository-network.spec.ts`: exercise Network to Forks navigation, filter combinations, saved defaults, fork row links, empty state, and mobile layout.
- `crates/api/tests/repository_network_contract.rs`: add coverage for empty public repositories, unreadable private forks, archived/stale forks, starred forks, metric ties, slash-containing branches, saved-default absence, invalid defaults payloads, and no secret leakage.

**Verification**: focused Rust edge-case tests, focused Vitest, focused Playwright smoke with screenshot `ralph/screenshots/build/insights-004-phase4-edge-cases.jpg`, then `make check && make test`.

---

## Phase 5: API Docs, Browser Evidence, QA Handoff, and Build Pass

**Done**: [ ]

**Scope**: Finish `insights-004` only after the Network API, Forks API, default Network page, filterable Forks page, saved defaults, edge cases, docs, screenshots, and QA handoff are verified. Mark `build_pass=true` only in this phase and leave `qa_pass=false`.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document `GET /api/repos/{owner}/{repo}/network`, `GET /api/repos/{owner}/{repo}/forks`, and `PUT /api/repos/{owner}/{repo}/forks/defaults`, including auth/privacy, readable-only fork projection, 50-fork network limit, period/type/sort filters, saved-default behavior, freshness metadata, and error envelopes.
- `crates/api/tests/repository_network_contract.rs`: add final coverage for anonymous/private/public access, readable-only forks, period/type/sort combinations, saved defaults, projection freshness, invalid input, and no secret leakage.
- `web/tests/repository-network-page.test.tsx` and `web/tests/repository-forks-page.test.tsx`: add final assertions for accessible names, keyboard focus order, menu controls, semantic chips, Editorial token usage, no banned GitHub colors/imports, no `href="#"`, no inert click handlers, and no unsafe HTML.
- `web/tests/e2e/repository-network.spec.ts`: full signed-session browser sweep for Insights navigation, Network graph/list, Forks filters, saved defaults, row links, empty states, private fork omission, and mobile no-overflow.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/insights-004/structure.md`, and `prd.json`: record verification evidence and set `insights-004.build_pass=true` only after every phase passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/insights-004-final-*.jpg`.

**Verification**: focused contract/unit/E2E tests, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full `DB_SSL=false CARGO_INCREMENTAL=0 make test`, `make test-e2e` or direct Playwright equivalent if the wrapper stalls, and mandatory Editorial banned-value scan: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
