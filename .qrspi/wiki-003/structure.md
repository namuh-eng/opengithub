# Structure Outline: wiki-003 Wiki History, Revision Compare, and Revert

**Ticket**: `wiki-003`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, `design/project/og-shell.jsx`, `target-docs/content/communities/documenting-your-project-with-wikis/viewing-a-wikis-history-of-changes.md`, existing `wiki-001`/`wiki-002` structures and implementation, `crates/api/src/domain/wiki.rs`, `crates/api/src/routes/repositories.rs`, `web/src/components/RepositoryWikiPage.tsx`, `web/src/components/RepositoryWikiEditor.tsx`, and `web/src/lib/navigation.ts`.
**Date**: 2026-05-06

## Existing Baseline

`wiki-001` completed the repository wiki reader with sanitized Markdown, page routes, page list/TOC, clone metadata, and history hrefs. `wiki-002` completed page index, create/edit/preview/save, local bare Git publishing, revision rows, image references, activity/audit events, and editor docs. `wiki-003` should build on those contracts to expose revision history, revision snapshot reads, two-revision diffs, and permissioned revert commits. Keep Rust API/session authority, Editorial UI primitives, and local wiki Git metadata. Do not introduce GitHub visual styling, Primer/Octicons, JS-side auth, external GitHub APIs, or a new client-side Markdown renderer.

## Phase 1: History Read Contract and Editorial History Page - revisions are browsable

**Done**: [x]

**Scope**: Add a screen-ready history API and render `/{owner}/{repo}/wiki/_history` plus page-scoped history routes. Users should see revision rows with author, relative time, commit message, short SHA links, selectable checkboxes, and older/newer pagination without enabling compare or revert yet.

**Key changes**:
- `crates/api/src/domain/wiki.rs`: add DTOs such as `WikiHistoryView`, `WikiHistoryRevisionRow`, `WikiHistoryPagination`, and `WikiHistoryScope`, reusing `WikiRepositorySummary`, `WikiViewer`, and `WikiAuthor`.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/wiki/_history` and page-scoped history reads such as `GET /api/repos/{owner}/{repo}/wiki/{slug}/_history`; validate bounded `page`/`pageSize` and preserve repository-safe private/disabled/missing-page behavior.
- `web/src/lib/api.ts`: add signed-cookie fetch helpers and typed DTOs for wiki history.
- `web/src/lib/navigation.ts`: add history href helpers for all-pages history, page history, revision snapshot links, and compare links.
- `web/src/app/[owner]/[repo]/wiki/_history/page.tsx` and supported page-history route shape: server-fetch history and keep the Wiki tab active.
- `web/src/components/RepositoryWikiHistoryPage.tsx`: render Editorial `History` heading, page scope context, revision list rows, checkbox labels tied to commit messages, author avatar/name, committed relative time, short SHA link, and Newer/Older controls with preserved scope.
- Tests: focused Rust history contract for pagination, permissions, page scope, missing/disabled/private states, and no-secret envelopes; focused Vitest for row metadata, accessible checkbox labels, concrete SHA/pagination links, empty state, no `href="#"`, no inert buttons, and Editorial banned-value guardrails.
- Browser smoke: load a seeded wiki history page, select one row, follow a SHA link target if Phase 2 route exists or verify concrete hrefs only, page older/newer when available, and save `ralph/screenshots/build/wiki-003-phase1-history-list.jpg`.

**Verification**: focused Rust history contract, focused Vitest, `cd web && npx tsc --noEmit --pretty false`, focused Biome, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 2: Revision Snapshot Read - old page content renders by SHA

**Done**: [ ]

**Scope**: Add revision detail routes so selecting a short SHA opens the wiki page as it existed at that commit. The snapshot should reuse the reader layout while making the historical state unmistakable and read-only.

**Key changes**:
- `crates/api/src/domain/wiki.rs`: add `WikiRevisionView` or extend `RepositoryWikiView` with immutable revision context, selected commit metadata, previous/next revision links, and source page href.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/wiki/{slug}/_revision/{revision}` or a Next-compatible public route backed by `GET /api/repos/{owner}/{repo}/wiki/{slug}/revisions/{revision}`; resolve by revision id, full oid, or unique short oid.
- Rendering behavior: return sanitized HTML/outline for the selected revision markdown, not latest page content; do not expose edit/save controls on snapshots.
- `web/src/app` revision route and `RepositoryWikiRevisionPage`: render the historical banner, commit message, author, timestamp, short SHA, page title, sanitized Markdown, and links back to latest page/history.
- Tests: cover SHA/id resolution, ambiguous/unknown revision errors, disabled/private behavior, historical content differing from latest content, no edit/revert controls on read-only snapshots, unsafe Markdown sanitization, concrete navigation links, and mobile no-overflow.
- Browser smoke: open history, click a short SHA, verify historical content and read-only banner, return to latest/history, and save `ralph/screenshots/build/wiki-003-phase2-revision-snapshot.jpg`.

**Verification**: focused Rust revision contract, focused Vitest, browser smoke, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 3: Two-Revision Compare - selected revisions produce a real diff

**Done**: [ ]

**Scope**: Wire two-row selection and compare navigation. The compare page should show an Editorial file/page diff with added and removed lines for the selected revisions, and it should stay read-only except for users who can edit and will receive revert controls in Phase 4.

**Key changes**:
- `crates/api/src/domain/wiki.rs`: add `WikiCompareView`, `WikiCompareRevisionSummary`, `WikiDiffFile`, `WikiDiffHunk`, and `WikiDiffLine` DTOs; use existing `wiki_diff_cache` when fresh and populate it when missing.
- Diff behavior: compare older/base to newer/head, normalize revision ordering, support page-specific diffs, bound diff size, and return structured validation errors when the two revisions are identical, unrelated, or unavailable.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/wiki/_compare?base=...&head=...` and page-scoped compare if needed.
- `RepositoryWikiHistoryPage`: make exactly two selected rows enable `Compare Revisions`, preserve selection in URL/query state when practical, and keep one/three-plus selections disabled with accessible explanation.
- `web/src/components/RepositoryWikiComparePage.tsx`: render base/head metadata, page path, diff stats, line-numbered additions/removals/context, and read-only unavailable states using Editorial tokens.
- Tests: cover compare button state, URL generation, base/head ordering, diff line rendering, cached/uncached diff contract, validation errors, no dead controls, keyboard selection flow, long lines wrapping/scrolling, and Editorial guardrails.
- Browser smoke: select two revisions, navigate to compare, verify added/removed lines and no dead buttons, and save `ralph/screenshots/build/wiki-003-phase3-compare.jpg`.

**Verification**: focused Rust compare contract, focused Vitest, browser smoke, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 4: Permissioned Revert - revert creates a new wiki commit

**Done**: [ ]

**Scope**: Add the mutation that restores older content from a compare view for users with wiki edit permission. Successful revert should create a new revision/git commit, record a revert event, invalidate rendered caches, write audit/activity/notification rows where enabled, and redirect to page history or latest page.

**Key changes**:
- `crates/api/migrations/*_repository_wiki_revert.*.sql`: add `wiki_revert_events` if it does not already exist, with repository/page/base/head/restored revision references, actor, commit id, and timestamp indexes.
- `crates/api/src/domain/wiki.rs`: add `WikiRevertRequest`, `WikiRevertResult`, and a revert function that validates edit permission, enabled wiki, non-archived repository, expected head revision, and same-page revision compatibility.
- Publishing behavior: create a new `wiki_git_commits` row and `wiki_page_revisions` row whose commit message records the reverted revision/short SHA; refresh `wiki_pages.latest_revision_id`, rendered cache, diff cache as needed, repository activity, audit events, and notifications.
- `crates/api/src/routes/repositories.rs`: register `POST /api/repos/{owner}/{repo}/wiki/reverts` or page-scoped equivalent; return structured 401/403/404/409/422 errors without stack traces or secrets.
- `RepositoryWikiComparePage`: show `Revert Changes` only when `viewer.canEditWiki`; submit with pending/error/success states and redirect to returned href. Readers can view diffs but never see revert controls.
- Tests: cover editor success, reader denial, archived/disabled denial, stale head conflict, cross-page revision rejection, revert event rows, audit/activity/notification rows, cache refresh, commit message contents, no secret leakage, UI hidden control for readers, and error display.
- Browser smoke: compare two revisions as an editor, click Revert Changes, verify redirect and a new history row with revert message, then re-open compare as reader/stub state to verify no revert control; save `ralph/screenshots/build/wiki-003-phase4-revert.jpg`.

**Verification**: focused Rust revert contract, focused Vitest, browser smoke, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 5: Docs, E2E, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `wiki-003` only after history, revision snapshots, compare, revert, docs, screenshots, and QA handoff are complete.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document wiki history, revision reads, compare, revert, permissions, pagination, diff cache, audit/activity/revert events, notifications, stale conflicts, and no-secret error envelopes.
- Final Rust tests: cover all-pages and page-scoped history, pagination, SHA resolution, compare ordering/cache behavior, revert success/failures, private/disabled/archived repositories, permission boundaries, cache invalidation, audit/activity/notification rows, and no credential/env leakage.
- Final frontend tests: cover History row accessibility, two-row selection, compare navigation, revision snapshot banner, diff rendering, permissioned revert visibility/action, error states, no placeholder hrefs, no inert handlers, long commit messages/diff lines, keyboard traversal, mobile no-overflow, and Editorial token compliance.
- `web/tests/e2e/repository-wiki-history.spec.ts`: signed-session sweep for history, revision snapshot, compare, revert when practical, final screenshots, and mobile no-overflow.
- `qa-hints.json`: append targets for large histories, ambiguous short SHAs, concurrent reverts, diff cache staleness, huge diffs, private repository leakage, revert notification fanout, local bare Git contents after revert, keyboard-only compare selection, and pagination edge cases.
- `build-progress.txt`, `.qrspi/wiki-003/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `wiki-003.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/wiki-003-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, JSON validation for `prd.json` and `qa-hints.json`, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, and mandatory Editorial banned-value scan:

```bash
rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'
```
