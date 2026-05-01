# Structure Outline: prs-005 Pull Request Files Changed Review

**Ticket**: `prs-005`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og-screens-3.jsx`, `target-docs/content/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/reviewing-proposed-changes-in-a-pull-request.md`, `target-docs/content/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/about-pull-request-reviews.md`, `target-docs/content/pull-requests/collaborating-with-pull-requests/reviewing-changes-in-pull-requests/commenting-on-a-pull-request.md`, `.qrspi/prs-003/structure.md`, `.qrspi/prs-004/structure.md`, current `crates/api/src/domain/pulls.rs`, current `crates/api/src/routes/pulls.rs`, current `web/src/app/[owner]/[repo]/pull/[number]/files/page.tsx`, and current `web/src/components/PullRequestFilesChangedPage.tsx`.
**Date**: 2026-05-01

## Phase 1: Diff Review Read Contract - files, hunks, viewer state, and settings

**Done**: [x]

**Scope**: Replace the current summary-only Files changed backing data with a permission-aware PR diff review contract. The Rust API returns PR header context, tabs/counts, file tree items, per-file diff hunks, old/new line positions, syntax metadata, viewed state for the signed-in viewer, pending review summary, published line comments, available commits, and current diff settings. Public repositories remain anonymously readable, but private repositories require read access and viewer-specific state is only returned for signed sessions.

**Key changes**:
- `crates/api/migrations/*_pull_request_diff_review.*.sql`: add narrow tables for `pull_request_file_hunks`, `pull_request_viewed_files`, `pull_request_review_comments`, and `pull_request_review_drafts` if absent; keep existing `pull_request_files` and `pull_request_reviews` contracts intact.
- `crates/api/src/domain/pulls.rs`: add `PullRequestDiffReviewView`, file-tree DTOs, hunk/line DTOs, diff settings DTOs, viewed-file DTOs, review-comment DTOs, and `get_pull_request_diff_review_for_actor`; derive reset semantics from file version fields such as `blob_oid`, `additions`, `deletions`, or a stored `diff_hash`.
- `crates/api/src/routes/pulls.rs`: add `GET /api/repos/:owner/:repo/pulls/:number/files` with query params for `view`, `whitespace`, `commit`, `filter`, and pagination bounds; preserve standard `401`, `403`, `404`, and `422 validation_failed` envelopes.
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed diff review DTOs and cookie-forwarding helpers separate from the existing compare summary helper.
- `crates/api/tests/api_pull_request_diff_review_contract.rs`: seed real Postgres pull requests, files, hunks, comments, and viewed rows; assert public/private access, filter behavior, split/unified settings, whitespace flags, viewed reset keys, comment joins, and standard errors.
- `.scratch/prs-005-diff-review-contract-scenario.sh`: exercise the live API and Postgres-backed scenario without mocks.

**Verification**: focused Rust diff review contract, `.scratch/prs-005-diff-review-contract-scenario.sh`, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check`, and `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make test`. Browser smoke is optional in this backend contract phase.

---

## Phase 2: Editorial Files Changed Page - sticky toolbar, file tree, diff controls, and viewed tracking

**Done**: [ ]

**Scope**: Upgrade `/{owner}/{repo}/pull/{number}/files` from a summary page into the Editorial diff review surface. The page renders the sticky PR toolbar, Conversation/Commits/Checks/Files changed tabs, additions/deletions totals, Review changes button, file filter input, commit selector, diff settings, split/unified and whitespace controls, resizable or responsive file tree, per-file headers, viewed checkboxes, expand controls, and diff rows with line numbers. Every visible control has a live action or an honest unavailable state.

**Key changes**:
- `web/src/app/[owner]/[repo]/pull/[number]/files/page.tsx`: fetch the Phase 1 diff review contract server-side instead of `getPullRequestCompare`, pass query params through, and render unavailable states consistently.
- `web/src/components/PullRequestFilesChangedPage.tsx`: replace the summary list with the Editorial diff review layout from `og-screens-3.jsx`, using `.btn`, `.chip`, `.card`, `.input`, `.tabs`, `.list-row`, `.av`, `.t-*`, and live tokens only.
- `web/src/components/PullRequestDiffToolbar.tsx`, `PullRequestFileTree.tsx`, and `PullRequestDiffFile.tsx`: split the page into focused components for toolbar controls, file navigation, responsive sidebar, file headers, hunk rows, syntax-highlighted code, and viewed-collapse state.
- `crates/api/src/routes/pulls.rs`: add `PATCH /api/repos/:owner/:repo/pulls/:number/files/viewed` for signed viewers to toggle a file viewed state, with version reset protection and idempotent responses.
- `web/src/app/[owner]/[repo]/pull/[number]/files/viewed/route.ts`: add the same-origin cookie-forwarding viewed toggle route.
- Tests: Vitest coverage for file filtering URLs, split/unified controls, whitespace links, file-tree jump targets, viewed toggle optimistic/error behavior, no dead hrefs, and accessible names; Playwright signed-session smoke saves `ralph/screenshots/build/prs-005-phase2-files-viewed.jpg`.

**Verification**: focused `pull-request-files-changed` Vitest, focused Playwright Files changed smoke, mandatory Editorial banned-value scan, then `make check`, `make test`, and `make test-e2e` because this phase changes a user-facing route.

---

## Phase 3: Inline Pending Comments - line/file comment composer and draft persistence

**Done**: [ ]

**Scope**: Make inline review commenting real without publishing comments immediately. Signed-in users can open a line or file comment composer, use Write/Preview Markdown tabs, save a pending review comment, edit/delete pending comments, and see pending comments remain private through reload until review submission. Anonymous public readers can read published comments but see a concrete sign-in CTA for adding review comments.

**Key changes**:
- `crates/api/src/domain/pulls.rs`: add helpers for pending review draft creation/update/delete, line-position validation against the stored hunk contract, Markdown rendering, and private draft visibility scoped to the draft author.
- `crates/api/src/routes/pulls.rs`: add `POST /api/repos/:owner/:repo/pulls/:number/review-comments/drafts`, `PATCH /drafts/:draft_id`, and `DELETE /drafts/:draft_id` with standard envelopes for blank bodies, invalid positions, stale files, and unauthorized access.
- `web/src/app/[owner]/[repo]/pull/[number]/files/review-comments/drafts/route.ts` and nested draft route handlers: same-origin cookie-forwarding mutations.
- `web/src/components/PullRequestInlineCommentComposer.tsx` and `PullRequestReviewThread.tsx`: render plus/comment affordances on diff rows, pending badges, Write/Preview tabs, Markdown preview, save/edit/delete feedback, and published thread rows.
- `web/src/components/PullRequestFilesChangedPage.tsx`: integrate pending/published thread rendering into diff rows without layout overlap on mobile or long code lines.
- Tests: Rust draft contract for signed-only writes, private draft visibility, line validation, edit/delete idempotency, Markdown preview, and no notification/timeline side effects before submit; Vitest composer coverage; Playwright pending-comment persistence screenshot `ralph/screenshots/build/prs-005-phase3-pending-comment.jpg`.

**Verification**: focused Rust draft tests, focused Vitest, focused Playwright pending comment flow, `make check`, `make test`, `make test-e2e`, and Editorial banned-value scan.

---

## Phase 4: Submit Review Dialog - publish comments, approvals, and requested changes

**Done**: [ ]

**Scope**: Wire the Review changes dialog end to end. Signed-in users can open the dialog, write a review summary, preview Markdown, choose Comment, Approve, or Request changes, submit with Command+Enter, or abandon/cancel without publishing drafts. Submission creates a `pull_request_reviews` row, publishes pending review comments, emits timeline events, updates review summaries and mergeability, sends notifications, writes audit/search side effects, and returns updated diff/review state.

**Key changes**:
- `crates/api/src/domain/pulls.rs`: add submit-review and abandon-review helpers that atomically move pending drafts into published `pull_request_review_comments`, create `pull_request_reviews`, attach summary body, update timeline events, notify participants/watchers, and refresh mergeability/review summaries.
- `crates/api/src/routes/pulls.rs`: add `POST /api/repos/:owner/:repo/pulls/:number/reviews` and `DELETE /api/repos/:owner/:repo/pulls/:number/reviews/draft`; validate review state values `commented`, `approved`, and `changes_requested`, author self-approval rules if required, and no-op abandon behavior.
- `web/src/app/[owner]/[repo]/pull/[number]/files/reviews/route.ts`: same-origin submit/abandon route handlers.
- `web/src/components/PullRequestSubmitReviewDialog.tsx`: dialog with Markdown summary editor, Write/Preview tabs, attachment affordance only when supported, radio choices Comment/Approve/Request changes, Cancel, Abandon review when drafts exist, Submit review, Command+Enter hint, pending count, and explicit success/error feedback.
- `web/src/components/RepositoryPullRequestDetailPage.tsx` and timeline components: ensure newly submitted review comments and review state appear on the Conversation tab after reload.
- Tests: Rust submit-review contract for publish transaction, abandon behavior, notification/timeline/audit side effects, approval/request-changes mergeability impact, and authorization; Vitest dialog/keyboard coverage; Playwright submit-review smoke screenshot `ralph/screenshots/build/prs-005-phase4-submit-review.jpg`.

**Verification**: focused Rust submit-review tests, focused Vitest, focused Playwright submit-review flow, full `make check`, `make test`, and `make test-e2e`.

---

## Phase 5: Diff Review Guardrails and QA Handoff - finish prs-005

**Done**: [ ]

**Scope**: Harden the full Files changed review feature and mark `prs-005` complete only when the diff read model, file navigation, viewed tracking, split/unified and whitespace settings, pending comments, submit/abandon review, published threads, privacy, mobile layout, and visual compliance are verified.

**Key changes**:
- `crates/api/tests/api_pull_request_diff_review_contract.rs`: final matrix for public anonymous read, private denial/redaction, signed viewed state, stale viewed reset, file filter, diff settings validation, inline draft isolation, draft edit/delete, submit review states, abandon review, published comments, timeline events, notifications, audit events, and standard error envelopes.
- `web/tests/pull-request-files-changed.test.tsx`: final coverage for toolbar controls, file tree, sticky actions, filter empty state recovery, keyboard-accessible file jumps, viewed toggles, inline composer, submit dialog, no `href="#"`, no inert buttons, and no direct banned colors/imports.
- `web/tests/e2e/pull-request-files-changed.spec.ts`: signed-session sweep for files tab load, file filter, tree jump, split/unified and whitespace controls, viewed toggle persistence, pending comment reload privacy, submit review as Comment/Approve/Request changes where seeded, abandon review, anonymous public read with auth-gated write CTA, mobile no-overflow, and desktop/mobile screenshots.
- `qa-hints.json`, `build-progress.txt`, and `prd.json`: record verification evidence and set `prs-005.build_pass=true` only after all phases are complete; leave `qa_pass=false`.
- Mandatory Editorial banned-value scan before commit: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.

**Verification**: `.scratch/prs-005-diff-review-contract-scenario.sh`, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check`, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make test`, and `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session make test-e2e`; browser smoke proves every visible Files changed toolbar control, file row, viewed checkbox, inline comment composer, pending draft, submit-review dialog action, conversation link, and mobile layout is live.
