# Structure Outline: commits-002 Repository Commit Detail

**Ticket**: `commits-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og-screens-2.jsx`, `design/project/og-shell.jsx`, current `crates/api/src/domain/repositories.rs`, current `crates/api/src/routes/repositories.rs`, current `crates/api/src/domain/pulls.rs` diff-review patterns, current `web/src/components/RepositoryCommitHistoryPage.tsx`, and current `web/src/components/PullRequestFilesChangedPage.tsx`.
**Date**: 2026-05-04

## Phase 1: Commit Detail Contract and Summary Page

**Done**: [x]

**Scope**: Add an authenticated, screen-ready commit-detail read contract and render the top summary shell for `/{owner}/{repo}/commit/{sha}`. This phase makes commit-history subject/SHA links land on a real page with author, SHA, verification, status, parent, PR, branch, and browse controls, while the diff body can still show a bounded loading/next-phase state.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: add `RepositoryCommitDetailView`, `RepositoryCommitDetailRepository`, `RepositoryCommitDetailCommit`, `RepositoryCommitDetailParent`, `RepositoryCommitDetailBranchLink`, `RepositoryCommitDetailPullRequestLink`, `RepositoryCommitDetailStatusSummary`, and `RepositoryCommitDetailVerificationSummary`.
- `crates/api/src/domain/repositories.rs`: add `repository_commit_detail_for_actor_by_owner_name(pool, actor, owner, repo, sha)` with repository visibility checks, abbreviated/full SHA resolution, parent/branch/PR/status/signature joins, recent-visit recording if supported, and redacted not-found/private errors.
- `crates/api/src/routes/repositories.rs`: add `GET /api/repos/{owner}/{repo}/commits/{sha}` or `GET /api/repos/{owner}/{repo}/commit/{sha}` consistently with existing route style, returning standard auth/error envelopes.
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed commit-detail DTOs and signed-cookie fetch helper.
- `web/src/app/[owner]/[repo]/commit/[sha]/page.tsx` and `web/src/components/RepositoryCommitDetailPage.tsx`: render the Editorial summary header using existing repository shell primitives, concrete Browse files/status/parent/PR links, a copy-SHA control with feedback, and a diff placeholder that points to the next phase without dead buttons.
- `crates/api/tests/repository_commit_detail_contract.rs`, `web/tests/repository-commit-detail-page.test.tsx`, and `web/tests/e2e/repository-commit-detail.spec.ts`: cover auth/privacy, SHA resolution, metadata shape, summary render, concrete links, copy feedback, and no dead controls.

**Verification**: focused Rust contract, focused Vitest, focused Playwright smoke with screenshot `ralph/screenshots/build/commits-002-phase1-summary.jpg`, then `make check && make test`.

---

## Phase 2: Diff File Tree and Unified Diff Rendering

**Done**: [x]

**Scope**: Extend the commit-detail contract and page so the diff area renders real changed files, a collapsible file tree, per-file stats, and unified diff rows with old/new line numbers. This phase makes the primary commit review surface useful end-to-end for text changes.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: add `RepositoryCommitDetailFile`, `RepositoryCommitDetailFileTreeNode`, `RepositoryCommitDetailHunk`, and `RepositoryCommitDetailLine` using existing `commit_file_changes` data and PR diff-review helpers where practical.
- `crates/api/src/domain/repositories.rs`: include diff summary totals, file anchors, language/path metadata, bounded hunk rows, and binary/large-file flags in `RepositoryCommitDetailView`.
- `web/src/components/RepositoryCommitDetailPage.tsx`: render a two-pane Editorial layout with file tree sidebar, changed-file rows, sticky diff toolbar, file anchors, per-file more menus with concrete Raw/View file links, and unified diff grid styling with tokenized colors.
- `web/src/lib/navigation.ts`: add commit-file anchor, raw-at-commit, and blob-at-commit helpers if existing helpers are insufficient.
- Tests cover file tree nesting, anchor destinations, diff rows, additions/deletions, binary/large placeholders, Raw/View file links, keyboard-focusable file navigation, and no horizontal overflow.

**Verification**: focused contract and component tests, Playwright diff render smoke with screenshot `ralph/screenshots/build/commits-002-phase2-diff.jpg`, then `make check && make test`.

---

## Phase 3: Diff Filter, In-Page Search, and Focus Behavior

**Done**: [x]

**Scope**: Make the diff controls interactive: Filter files narrows the file tree and visible diff files, Search within code highlights matches in visible diff content, and selecting a file tree row scrolls/focuses the matching diff file without layout jumps.

**Key changes**:
- `web/src/components/RepositoryCommitDetailPage.tsx`: split client-side diff controls into focused subcomponents such as `RepositoryCommitFileTree`, `RepositoryCommitDiffSearch`, and `RepositoryCommitDiffFile`.
- `web/src/components/RepositoryCommitDetailPage.tsx`: add filter/search state, accessible empty states, highlight markup that does not use unsafe HTML, active-file focus management, and stable sidebar/main grid constraints for desktop and mobile.
- `web/tests/repository-commit-detail-page.test.tsx`: assert file filtering, clear filters, code search highlighting, empty states, selected-file focus labels, mobile wrapping, and no placeholder handlers.
- `web/tests/e2e/repository-commit-detail.spec.ts`: exercise filter/search/file selection in a signed-in browser session and save `ralph/screenshots/build/commits-002-phase3-search-filter.jpg`.

**Verification**: focused Vitest and Playwright interaction tests, then `make check && make test`.

---

## Phase 4: Context Expansion and Bounded Diff Edge Cases

**Done**: [ ]

**Scope**: Add working Expand all lines controls and harden large, binary, renamed, deleted, and generated-file diff cases. This phase ensures every visible diff affordance either performs the real API-backed action or is explicitly disabled with a truthful reason.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: add bounded context-window support to the commit-detail query, including path and hunk identifiers, maximum context limits, and stable error responses for invalid ranges.
- `crates/api/src/routes/repositories.rs`: add a narrow context-lines endpoint or query mode for `GET /api/repos/{owner}/{repo}/commits/{sha}` that returns extra lines for one file/hunk without leaking private repository data.
- `web/src/app/api/repos/[owner]/[repo]/commits/[sha]/context/route.ts` or equivalent same-origin proxy: forward context expansion requests with the current session cookie.
- `web/src/components/RepositoryCommitDetailPage.tsx`: wire Expand all lines buttons, loading/error feedback, binary/large diff placeholders, renamed/deleted file labels, and Raw/View file actions.
- Tests cover successful context expansion, invalid range errors, auth denial, binary/large placeholders, raw/blob navigation, repeated rapid clicks, and no secret/stack leakage.

**Verification**: focused Rust/API tests, focused Vitest, Playwright context-expansion smoke with screenshot `ralph/screenshots/build/commits-002-phase4-expanded-context.jpg`, then `make check && make test`.

---

## Phase 5: API Docs, Browser QA Handoff, and Build Pass

**Done**: [ ]

**Scope**: Finish `commits-002` with API documentation, final browser evidence, QA handoff notes, full regression checks, and `prd.json` bookkeeping. This phase only marks the feature complete after the summary, diff, filters, search, expansion, copy, browse, parent, status, and raw/view controls are all live.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document the commit-detail endpoint, context expansion contract, query params, grouped diff response shape, auth/privacy behavior, error envelopes, binary/large-file handling, and redaction rules.
- `crates/api/tests/repository_commit_detail_contract.rs`: add final coverage for public/private/anonymous access, malformed SHA, abbreviated SHA ambiguity, parentless root commits, merge commits with multiple parents, missing files, binary/large diffs, status/signature/PR joins, and no secret leakage.
- `web/tests/repository-commit-detail-page.test.tsx`: add final assertions for accessible names, semantic chips, Editorial token usage, no banned GitHub colors/imports, no `href="#"`, and no inert click handlers.
- `web/tests/e2e/repository-commit-detail.spec.ts`: full signed-session sweep for summary links, copy SHA, browse files, parent link, status link, file tree, filter, code search, context expansion, raw/view file links, empty/binary states, mobile no-overflow, and saved final screenshots.
- `qa-hints.json`, `build-progress.txt`, and `prd.json`: record verification evidence and set `commits-002.build_pass=true` only after every phase passes; leave `qa_pass=false`.

**Verification**: focused contract/unit/E2E tests, full `make check`, full `make test`, `make test-e2e` or direct Playwright equivalent if the wrapper stalls, and mandatory Editorial banned-value scan: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`. Save final screenshots under `ralph/screenshots/build/`.
