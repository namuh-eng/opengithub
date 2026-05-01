# Structure Outline: prs-004 Pull Request Conversation Detail

**Ticket**: `prs-004`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og-screens-3.jsx`, `.qrspi/prs-001/structure.md`, `.qrspi/prs-002/structure.md`, `.qrspi/prs-003/structure.md`, `.qrspi/issues-004/structure.md`, current `crates/api/src/domain/pulls.rs`, current `crates/api/src/routes/pulls.rs`, current `web/src/app/[owner]/[repo]/pull/[number]/page.tsx`, current `web/src/components/RepositoryIssueDetailPage.tsx`, `web/src/components/MarkdownBody.tsx`, and `web/src/components/MarkdownEditor.tsx`.
**Date**: 2026-05-01

## Phase 1: Detail Read Model - header, tabs, original body, and sidebar shell

**Done**: [x]

**Scope**: Replace the placeholder `/{owner}/{repo}/pull/{number}` page with a real permission-aware pull request detail read surface. The API returns title, number, state, draft/merged/closed flags, author, base/head refs, commit/file/comment/review/check counts, repository context, viewer permission, labels, milestone, assignees, requested reviewers, linked issues, participants, and subscription defaults. The UI renders the Editorial PR conversation shell with repository tabs, Conversation/Commits/Checks/Files changed tabs, original body card, right metadata sidebar, and no inert controls.

**Key changes**:
- `crates/api/src/domain/pulls.rs`: add `PullRequestDetailView`, `PullRequestDetailRepository`, `PullRequestDetailAuthor`, `PullRequestDetailSidebar`, and summary DTOs that compose the existing `PullRequest`, backing `Issue`, author, labels, milestone, assignees, review requests, latest reviews, checks summary, snapshot commits/files, linked issues, participants, and viewer permission.
- `crates/api/src/routes/pulls.rs`: make `GET /api/repos/:owner/:repo/pulls/:number` return the screen-ready detail view; allow anonymous reads for public repositories consistently with the pull list and issue detail contracts while keeping private redaction.
- `web/src/lib/api.ts`: add typed PR detail DTOs, `repositoryPullRequestPath`, `getRepositoryPullRequestFromCookie`, and preserved standard error-envelope handling.
- `web/src/app/[owner]/[repo]/pull/[number]/page.tsx`: fetch repository metadata and the detail read model server-side, render inside `RepositoryShell`, and keep unavailable states explicit.
- `web/src/components/RepositoryPullRequestDetailPage.tsx`: new Editorial component using `.btn`, `.chip`, `.card`, `.tabs`, `.list-row`, `.av`, `.t-*`, `MarkdownBody`, and tokenized semantic colors only.
- Tests: Rust detail read contract for public/anonymous, private denial, draft/open/closed/merged state chips, sidebar metadata, counts, and redaction; Vitest rendering coverage; Playwright detail-load smoke screenshot `ralph/screenshots/build/prs-004-phase1-detail-read.jpg`.

**Verification**: focused Rust PR detail-read contract, focused `repository-pull-request-detail` Vitest, focused Playwright public and signed-session detail smoke, mandatory Editorial banned-value scan, then `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test make check && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test make test`.

---

## Phase 2: Conversation Timeline and Comments - load events and create comments

**Done**: [x]

**Scope**: Turn the Conversation tab into a real pull request timeline. Opened/commented/committed/review-requested/label/base-change/force-push/ready-for-review/review events render in chronological order, existing comments show Markdown, and signed-in users can post comments with Write/Preview, success/error feedback, and reload persistence. Anonymous public readers can read but see a concrete sign-in CTA for commenting.

**Key changes**:
- `crates/api/src/domain/pulls.rs`: replace the raw `Vec<TimelineEvent>` PR timeline with a normalized `PullRequestTimelineItem` view that joins actors, comments, review rows, review-request metadata, commit/file snapshot metadata, labels, and safe event metadata.
- `crates/api/src/routes/pulls.rs`: keep `GET /pulls/:number/timeline` readable for public repositories, ensure `POST /pulls/:number/comments` returns the created timeline item or updated timeline fragment, and preserve `422 validation_failed` for blank comments.
- `web/src/app/[owner]/[repo]/pull/[number]/comments/route.ts`: add a same-origin cookie-forwarding comment route.
- `web/src/components/PullRequestTimeline.tsx` and `web/src/components/PullRequestCommentComposer.tsx`: reuse the issue-detail timeline/editor patterns with PR-specific event copy, status icons, author badges, bot/comment cards, Write/Preview tabs, Markdown preview, attachment affordance only when supported, and no placeholder toolbar buttons.
- `web/src/components/RepositoryPullRequestDetailPage.tsx`: render the Phase 2 timeline and signed-in/anonymous composer states under the Conversation tab.
- Tests: Rust timeline/comment contract including public read, private denial, event ordering, comment validation, and actor joins; Vitest timeline/comment/preview/error coverage; Playwright signed-session comment-create smoke screenshot `ralph/screenshots/build/prs-004-phase2-comment.jpg`.

**Verification**: focused API timeline/comment tests, focused frontend tests, focused Playwright comment-create flow, `make check`, `make test`, and `make test-e2e` if shared repository navigation is touched.

---

## Phase 3: Reviews, Review Requests, Sidebar Metadata, and Notifications

**Done**: [x]

**Scope**: Wire the non-merge conversation actions that reviewers expect. Authorized users can request/remove reviewers, mark draft/ready when permitted, edit supported labels/assignees/milestone metadata, and subscribe/unsubscribe from notifications. Review status summaries and requested-reviewer rows update the header/sidebar/timeline and persist through reload. Unsupported Projects/stacked-PR fields must show honest empty/disabled states instead of inert controls.

**Key changes**:
- `crates/api/migrations/*_pull_request_detail_interactions.*.sql`: add narrow missing tables/columns only if absent, such as PR subscriptions, draft transition metadata, review-request dismissal timestamps, or PR metadata audit helpers; reuse `issue_subscriptions`, `pull_request_review_requests`, `pull_request_reviews`, `timeline_events`, `notifications`, and `audit_events` where possible.
- `crates/api/src/domain/pulls.rs`: add permission-checked helpers for review request add/remove, draft/ready transitions, metadata updates backed by the issue tables, subscription toggle/read state, participant derivation, notification fanout, and timeline events.
- `crates/api/src/routes/pulls.rs`: add scoped endpoints for `/review-requests`, `/draft`, `/metadata`, and `/subscription` using `RestJson` plus standard 401/403/404/422 envelopes.
- `web/src/app/[owner]/[repo]/pull/[number]/*/route.ts`: add same-origin cookie-forwarding routes for the client mutations.
- `web/src/components/PullRequestMetadataSidebar.tsx`: reviewer, assignee, label, milestone, linked-issue, participant, notification, and check-summary sections with accessible Editorial popovers and live save/error states.
- Tests: Rust interaction contract for permissions, invalid IDs, idempotency, timeline/notification side effects, and private redaction; Vitest sidebar/menu/action coverage; Playwright review-request/subscription/metadata smoke screenshot `ralph/screenshots/build/prs-004-phase3-sidebar-actions.jpg`.

**Verification**: focused Rust interaction tests, focused Vitest, focused Playwright signed-session action flow, standard `make check`, `make test`, and mandatory Editorial banned-value scan.

---

## Phase 4: Mergeability Box and State Actions - close, reopen, ready, and merge rules

**Done**: [x]

**Scope**: Make the merge box real. The page computes mergeability from draft state, open/closed/merged state, compare status, conflicts/no-diff, checks summary, review requirements, branch protection/ruleset placeholders, stacked-PR blockers where modeled, and viewer permission. Authorized users can close/reopen, mark ready for review, and merge only when the rules allow; blocked cases show explicit reasons.

**Key changes**:
- `crates/api/migrations/*_pull_request_mergeability.*.sql`: add merge metadata/rule tables only if current schema lacks a narrow place for required reviews/checks/branch protection snapshots; prefer reusing `pull_request_checks_summary`, `pull_request_reviews`, and existing branch/ref metadata.
- `crates/api/src/domain/pulls.rs`: add `PullRequestMergeability`, `PullRequestMergeBlocker`, `MergeMethod`, and helpers to compute blockers; harden `update_pull_request_state` so merge requests validate permissions, draft state, checks, reviews, duplicate merge, and merge commit availability before mutating state.
- `crates/api/src/routes/pulls.rs`: add or extend state/merge endpoints so close/reopen/ready/merge return the updated detail view or mergeability fragment with standard blocked-reason envelopes.
- `web/src/app/[owner]/[repo]/pull/[number]/state/route.ts` and `/merge/route.ts`: forward client actions to Rust with signed cookies.
- `web/src/components/PullRequestMergeBox.tsx`: render ready/blocked/merged/closed states, merge method selector, close/reopen/ready actions, explicit blocker list, success/error feedback, and no dead buttons.
- Tests: Rust mergeability contract for draft blocks, no-diff/conflict placeholders, checks pass/fail/pending, reviews approved/changes requested/missing, branch protection/ruleset placeholders, close/reopen/merge idempotency, unauthorized denial, timeline/audit/search updates; Vitest merge-box states; Playwright merge-blocked and merge-success smoke screenshot `ralph/screenshots/build/prs-004-phase4-mergeability.jpg`.

**Verification**: focused Rust mergeability tests, focused merge-box Vitest, focused Playwright merge/state flow, `make check`, `make test`, and `make test-e2e`.

---

## Phase 5: Detail Guardrails and QA Handoff - finish prs-004

**Done**: [x]

**Scope**: Harden the full PR conversation feature and mark `prs-004` complete only when the detail read model, timeline/comments, review requests, notifications, supported metadata actions, close/reopen/ready/merge actions, tabs, mobile layout, and browser evidence are all verified.

**Key changes**:
- `crates/api/tests/api_pull_request_detail_contract.rs`: final matrix for anonymous public reads, private denial/redaction, detail shape, timeline ordering, comments, review requests, draft/ready transitions, metadata edits, subscriptions, merge blockers, close/reopen/merge, notifications, audit events, and standard error envelopes.
- `web/tests/repository-pull-request-detail.test.tsx`: final coverage for header/tabs/sidebar/timeline/composer/merge box accessibility, Write/Preview tabs, no `href="#"`, no empty handlers, no placeholder controls, and API error displays.
- `web/tests/e2e/repository-pull-request-detail.spec.ts`: signed-session sweep for detail load, conversation tabs, comment create, review request, subscription toggle, metadata edit, close/reopen, ready/merge blocker states, successful merge where seeded, anonymous public read, auth-gated write CTA, mobile no-overflow, and desktop/mobile screenshots.
- `web/src/app/[owner]/[repo]/pull/[number]/files/page.tsx`: ensure the Files changed tab links to the existing PR diff/snapshot surface or an honest live route from `prs-003`; no dead Files changed tab.
- Mandatory Editorial banned-value scan before commit: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`, `build-progress.txt`, and `prd.json`: record verification evidence and set `prs-004.build_pass=true` only after every phase is complete; leave `qa_pass=false`.

**Verification**: `.scratch/prs-004-detail-scenario.sh` or equivalent TEST_DATABASE_URL-backed API/browser scenario, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test make check`, `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test make test`, and `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test make test-e2e`; browser smoke proves every visible tab, composer, sidebar action, notification control, close/reopen/ready/merge control, and detail row link is live and saves final screenshots.
