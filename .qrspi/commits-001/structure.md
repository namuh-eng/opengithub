# Structure Outline: commits-001 Repository Commit History

**Ticket**: `commits-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og-screens-2.jsx`, `design/project/og-shell.jsx`, `ralph/screenshots/inspect/commits-list.jpg`, `ralph/screenshots/inspect/commits-branch-selector.jpg`, `ralph/screenshots/inspect/commits-user-selector.jpg`, `ralph/screenshots/inspect/commits-date-picker.jpg`, current `crates/api/src/domain/repositories.rs`, current `crates/api/src/routes/repositories.rs`, current `web/src/components/RepositoryPathViews.tsx`, and current `web/src/app/[owner]/[repo]/commits/[ref]/[[...path]]/page.tsx`.
**Date**: 2026-05-04

## Phase 1: Commit List Contract - API returns screen-ready grouped history

**Done**: [ ]

**Scope**: Upgrade the existing authenticated commit-history endpoint from a thin list of commits into a repository commit-list view that resolves the requested branch/tag/ref, supports path-scoped history, groups rows by commit date, and returns the metadata the screen needs. This phase is API-first and keeps the current UI functional.

**Key changes**:
- `crates/api/migrations/*_commit_history_metadata.*.sql`: add only missing narrow metadata, such as commit PR links, commit status summary rows, and recent-visit/filter telemetry storage if those tables do not already exist.
- `crates/api/src/domain/repositories.rs`: replace or extend `RepositoryCommitHistoryItem` with `RepositoryCommitHistoryView`, `RepositoryCommitGroup`, `RepositoryCommitListItem`, `RepositoryCommitAuthorOption`, `RepositoryCommitStatusSummary`, `RepositoryCommitVerificationSummary`, and resolved-ref/filter metadata.
- `crates/api/src/domain/repositories.rs`: make `repository_commit_history_for_actor_by_owner_name` honor `ref`, `path`, `author`, `until`/`before`, `page`, and `page_size`; validate refs through `repository_git_refs`; preserve public/private repository permission behavior.
- `crates/api/src/routes/repositories.rs`: normalize commit-list query params and return standard `401`/`403`/`404`/`422`/database envelopes without leaking stack traces or private ref names.
- `crates/api/tests/repository_commit_history_contract.rs`: seed real repositories, refs, commits, authors, PR links, signature summaries, status summaries, and path snapshots; assert grouping, ref resolution, author/date/path filters, pagination, privacy, and redaction.

**Verification**: focused Rust contract tests against `opengithub_identity_test`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and `DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Commit History Page - default branch history renders real data

**Done**: [ ]

**Scope**: Replace the minimal `RepositoryCommitHistoryView` with the full Editorial commit-history page for `/{owner}/{repo}/commits/{branch}`. The default view should render the repository shell, Commit history heading, compact toolbar, date-grouped rows, verified badges, status buttons, short SHA buttons, browse-at-commit links, and row overflow controls with no inert buttons.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed commit-history view DTOs and fetch helpers for the Phase 1 contract while forwarding signed session cookies.
- `web/src/app/[owner]/[repo]/commits/[ref]/[[...path]]/page.tsx`: fetch repository commit history server-side, preserve URL search params, and render repository-scoped loading/error/empty states.
- `web/src/components/RepositoryCommitHistoryPage.tsx`: new Editorial screen component using `.card`, `.btn`, `.chip`, `.input`, `.tabs`, `.list-row`, `.av`, `.t-*`, `var(--*)` tokens, and repo-shell spacing.
- `web/src/lib/navigation.ts`: add commit-list, commit-detail, browse-at-commit, status-summary, filter, pagination, and clear-filter href helpers.
- `web/tests/repository-commit-history-page.test.tsx` and `web/tests/e2e/repository-commits.spec.ts`: cover default branch render, grouped rows, subject/SHA/detail links, browse links, status/verification affordances, empty state, no dead controls, and a saved desktop screenshot.

**Verification**: focused Vitest and Playwright smoke for commit history, then `make check && make test`; browser smoke saves `ralph/screenshots/build/commits-001-phase2-default-history.jpg`.

---

## Phase 3: Branch and Tag Selector - ref changes drive commit history

**Done**: [ ]

**Scope**: Make the branch selector dialog functional on commit-history pages. Searching branches/tags, switching tabs, and selecting a ref should reload the same commit-history page for that ref while preserving author/date/path filters when valid.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: reuse or extend the existing repository refs contract with active-ref metadata, branch/tag totals, default badge state, searchable names, and commit-history hrefs.
- `web/src/components/RepositoryCommitRefSelector.tsx`: accessible dialog or popover with Find a branch input, Branches/Tags tabs, selected/default badges, radio rows, View all branches link, loading/empty states, and URL-backed selection.
- `web/src/components/RepositoryCommitHistoryPage.tsx`: wire the selector into the compact toolbar and ensure filter/query state survives ref changes.
- Tests cover branch search, tag search, default badge rendering, active ref indication, preserving filters, invalid-ref recovery, keyboard navigation, mobile wrapping, and no horizontal overflow.

**Verification**: focused API/ref tests, focused Vitest selector tests, Playwright branch/tag selection flows, and screenshot `ralph/screenshots/build/commits-001-phase3-ref-selector.jpg`.

---

## Phase 4: Author and Date Filters - toolbar filters are URL-backed and reversible

**Done**: [ ]

**Scope**: Implement the All users author selector and All time date filter. Selecting an author or date updates the URL, reloads grouped commit rows, preserves the selected ref/path, and Clear returns to the unfiltered history.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: add author option aggregation, author filter validation by login/user id, date bounds (`until` or `before`), stable ordering, and bounded pagination under combined filters.
- `web/src/components/RepositoryCommitAuthorSelector.tsx`: searchable author selector with avatars/logins/counts, selected state, clear action, and accessible empty state.
- `web/src/components/RepositoryCommitDateFilter.tsx`: Editorial date popover/input with All time, date apply, clear, validation feedback, and URL helper integration.
- `web/src/components/RepositoryCommitHistoryPage.tsx`: show active filter chips, no-results state, and filter-preserving pagination.
- Tests cover author/date query round-trips, invalid dates, no-results recovery, filter chips, combined ref/path/author/date state, and mobile no-overflow.

**Verification**: targeted Rust filter tests, Vitest toolbar tests, Playwright author/date/clear flows, and screenshot `ralph/screenshots/build/commits-001-phase4-filtered-history.jpg`.

---

## Phase 5: Final Guardrails and QA Handoff - finish commits-001

**Done**: [ ]

**Scope**: Harden commit-history behavior, docs, visual compliance, browser evidence, and bookkeeping. Mark `commits-001` complete only after the page has real API data, working ref/author/date filters, grouped rows, live row actions, saved screenshots, and QA handoff notes.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document `GET /api/repos/{owner}/{repo}/commits` with ref/path/author/date/pagination params, auth/privacy behavior, grouped response shape, status/check/signature fields, and error envelopes.
- `crates/api/tests/repository_commit_history_contract.rs`: add final coverage for public/private/anonymous access, malformed refs, missing paths, pagination bounds, count consistency, PR/status/signature joins, recent-visit telemetry if implemented, and no secret leakage.
- `web/tests/repository-commit-history-page.test.tsx`: assert accessible names, keyboard-focusable controls, semantic chips, Editorial token usage, no banned GitHub colors/imports, no `href="#"`, and no placeholder handlers.
- `web/tests/e2e/repository-commits.spec.ts`: full signed-session sweep for default list, branch/tag switch, author filter, date filter, row detail links, status summary, browse-at-commit link, empty state, pagination, and mobile no-overflow.
- `qa-hints.json`, `build-progress.txt`, and `prd.json`: record verification evidence and set `commits-001.build_pass=true` only after every phase passes; leave `qa_pass=false`.

**Verification**: focused contract/unit/E2E tests, full `make check`, full `make test`, `make test-e2e` or direct Playwright equivalent if the wrapper stalls, and mandatory Editorial banned-value scan: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`. Save final screenshots under `ralph/screenshots/build/`.
