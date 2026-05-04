# Structure Outline: branches-001 Repository Branches Page

**Ticket**: `branches-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, current placeholder `web/src/app/[owner]/[repo]/branches/page.tsx`, existing branch settings contract in `crates/api/src/domain/repositories.rs`, existing repository refs contract, existing commit history/detail contracts from `commits-001` and `commits-002`, and existing branch protection/ruleset settings UI.
**Date**: 2026-05-05

## Phase 1: Branch Directory API Contract - screen-ready branch metadata

**Done**: [x]

**Scope**: Add an authenticated read contract for `GET /api/repos/{owner}/{repo}/branches` that returns the branch-directory view needed by the page without changing the existing settings/branches contract. The response should classify default, active, stale, and all branches; support search and pagination; and include row metadata for navigation, protection, checks, ahead/behind, linked PRs, and viewer actions.

**Key changes**:
- `crates/api/migrations/*_branch_directory_metadata.*.sql`: add only missing narrow metadata such as `branch_activity_snapshots` fields and optional branch search/recent-visit telemetry if the current schema does not already cover them.
- `crates/api/src/domain/repositories.rs`: add `RepositoryBranchesView`, `RepositoryBranchDirectoryRow`, `RepositoryBranchClassificationCounts`, `RepositoryBranchLatestCommitSummary`, `RepositoryBranchCheckSummary`, `RepositoryBranchProtectionSummary`, `RepositoryBranchPullRequestSummary`, `RepositoryBranchCapabilities`, and `RepositoryBranchesQuery`.
- `crates/api/src/domain/repositories.rs`: add `repository_branches_for_actor_by_owner_name(pool, actor_user_id, owner, repo, query)` that preserves repository privacy, resolves branch refs from `repository_git_refs`, joins the latest commit/author/status/protection/linked PR data, computes ahead/behind relative to default branch from commit ancestry when available, and records bounded search telemetry.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/branches`, normalize `tab`, `q`, `page`, and `pageSize`, and return standard `401`/`403`/`404`/`422`/database envelopes without leaking private refs or stack/session details.
- `crates/api/tests/repository_branches_contract.rs`: seed public/private repositories, branch refs, commits, status summaries, open/draft PRs, rules/rulesets, stale timestamps, and permissions; assert classification counts, search, pagination, default branch row, protection summaries, action capability flags, privacy, and redaction.

**Verification**: focused Rust contract tests against `opengithub_identity_test`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and `DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Branches Overview Page - default and active branches render real data

**Done**: [x]

**Scope**: Replace the placeholder `/{owner}/{repo}/branches` page with a real Editorial repository branches screen backed by the Phase 1 API. The default Overview tab should render the repository shell, Branches heading, tablist, search input, default branch section, active branches table, row metadata, empty states, and concrete destinations with no inert controls.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed branch-directory DTOs and signed-cookie fetch helpers for the new Rust endpoint.
- `web/src/app/[owner]/[repo]/branches/page.tsx`: fetch repository plus branch directory server-side, preserve URL search params, and render repository-scoped unavailable/forbidden/empty states inside `AppShell`.
- `web/src/components/RepositoryBranchesPage.tsx`: new Editorial component using `.card`, `.btn`, `.chip`, `.input`, `.tabs`, `.list-row`, `.av`, `.t-*`, and `var(--*)` tokens. Rows include branch tree links, latest author/time, status-check link, ahead/behind counts, linked PR link, protected badge/rules link, copy branch-name button, and action menu trigger.
- `web/src/lib/navigation.ts`: add branch directory, branch tab/search/pagination, tree-at-branch, commit-history-at-branch, PR, branch protection/rules, and branch activity href helpers. Encode slash-containing branch names safely.
- `web/tests/repository-branches-page.test.tsx` and `web/tests/e2e/repository-branches.spec.ts`: cover the overview render, default/active sections, row destinations, protected badges, copy feedback, empty state, no `href="#"`, no placeholder handlers, and a saved desktop screenshot.

**Verification**: focused Vitest and Playwright smoke for the overview page, then `make check && make test`; browser smoke saves `ralph/screenshots/build/branches-001-phase2-overview.jpg`.

---

## Phase 3: Tabs, Search, Pagination, and Row Actions - branch list interactions are URL-backed

**Done**: [x]

**Scope**: Make Overview, Active, Stale, and All tabs functional, make search case-insensitive and URL-backed, and wire every visible row action to real behavior or a truthful disabled state. Search and tab changes should preserve repository context and pagination should be stable across filtered result sets.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: finish tab-specific result shaping for active/stale/all, stable branch ordering, stale cutoff metadata, bounded `q` validation, pagination totals, and no-results recovery metadata.
- `web/src/components/RepositoryBranchesPage.tsx`: add tab/search form handling, active filter chips, clear search, pagination controls, responsive table-to-list layout, and action menus with concrete `Activity`, `View rules`, `Copy branch name`, `Open tree`, and `Open commits` actions.
- Same-origin route handlers are not required for read-only tab/search changes, but copy and menus must use client components with accessible loading/success/error states where applicable.
- Hide destructive delete/restore/rename actions for read-only viewers. If admin-only branch mutations are deferred, render them as disabled menu items with policy/protection copy and no fake-success handlers.
- Extend `web/tests/repository-branches-page.test.tsx` and `web/tests/e2e/repository-branches.spec.ts` for all tabs, search query round trips, pagination, copied branch names, slash-containing branch refs, stale empty states, menu keyboard behavior, and mobile no-overflow.

**Verification**: focused Rust tab/search tests, focused Vitest interaction tests, focused Playwright tab/search/menu flows, screenshot `ralph/screenshots/build/branches-001-phase3-filtered.jpg`, then `make check && make test`.

---

## Phase 4: Branch Activity and Rules Drill-Downs - row destinations are useful

**Done**: [x]

**Scope**: Ensure branch row destinations do not land on placeholders. Activity should open a repository-scoped branch activity/read surface, and View rules should land on the applicable branch protection/ruleset context rather than a generic dead link. This phase should stay read-oriented except for existing settings contracts already implemented by `settings-004`.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: add a bounded branch activity DTO or extend the branch-directory response with enough activity detail for a drill-down route: recent commits, recent PRs, protection events, check summaries, and last push metadata.
- `crates/api/src/routes/repositories.rs`: add `GET /api/repos/{owner}/{repo}/branches/{branch}/activity` or a query-safe equivalent that handles slash-containing branches, missing branches, public/private privacy, and malformed branch names.
- `web/src/app/[owner]/[repo]/branches/[...branch]/page.tsx` or equivalent route: render an Editorial branch activity page/panel with branch summary, recent commits, linked PRs, protection/rules links, compare/history/tree links, and recovery links back to `/branches`.
- `web/src/components/RepositoryBranchesPage.tsx`: point Activity and View rules actions at the new activity route and existing `/{owner}/{repo}/settings/branches` anchors or filtered rules URLs when the viewer can read them; keep read-only public users on non-leaky public-safe explanations.
- Rust, Vitest, and Playwright coverage for activity privacy, missing branch recovery, rules/protection links, action menu destinations, no secret leakage, and screenshot `ralph/screenshots/build/branches-001-phase4-activity.jpg`.

**Verification**: focused Rust activity tests, focused Vitest, focused Playwright row-action smoke, then `make check && make test`; run `make test-e2e` or direct Playwright when local servers are stable.

---

## Phase 5: Guardrails, API Docs, Browser Evidence, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `branches-001` only after the API, UI, search/tabs, row actions, activity/rules destinations, docs, screenshots, and QA handoff are verified. Mark `build_pass=true` only in this phase and leave `qa_pass=false`.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document `GET /api/repos/{owner}/{repo}/branches` and branch activity, including auth/privacy, tabs, search, pagination, response shape, branch protection summaries, linked PR/status metadata, telemetry, and error envelopes.
- `web/tests/repository-branches-page.test.tsx`: add final assertions for accessible names, keyboard focus order, semantic chips, Editorial token usage, no banned GitHub colors/imports, no dead anchors, no inert handlers, and no unsafe HTML rendering.
- `web/tests/e2e/repository-branches.spec.ts`: full signed-session browser sweep for overview, active/stale/all tabs, search, pagination, branch tree links, PR links, protected rules links, activity links, copy feedback, read-only permission state, and mobile no-overflow.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/branches-001/structure.md`, and `prd.json`: record verification evidence and set `branches-001.build_pass=true` only after every phase passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/branches-001-final-*.jpg`.

**Verification**: focused contract/unit/E2E tests, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full `DB_SSL=false CARGO_INCREMENTAL=0 make test`, `make test-e2e` or direct Playwright equivalent if the wrapper stalls, and mandatory Editorial banned-value scan: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
