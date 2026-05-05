# Structure Outline: discussions-003 Discussion Detail Timeline

**Ticket**: `discussions-003`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, `design/project/og-screens-4.jsx`, `design/project/og-shell.jsx`, existing `discussions-001` list/category/vote contracts, `discussions-002` creation/form/poll contracts, issue/PR timeline patterns in `crates/api/src/domain/issues.rs`, `crates/api/src/routes/issues.rs`, `crates/api/src/routes/pulls.rs`, repository navigation helpers in `web/src/lib/navigation.ts`, Markdown sanitization/comment composer patterns from issue and PR detail surfaces, notification fanout from `notifications-001`, and Editorial repository shell components.
**Date**: 2026-05-05

## Existing Baseline

`discussions-001` already provides repository Discussions list/category reads and discussion upvotes. `discussions-002` already provides chooser, composer, YAML form, poll metadata, and create-side effects. `discussions-003` should add the real discussion detail experience: read timeline, comments, nested replies, reactions, answer state, subscription controls, sidebar metadata, and moderation state changes. It must keep the Editorial design system and should not expand into category administration, poll voting, repository-wide discussion search indexing, or organization discussion policy settings.

## Phase 1: Detail Contract and Read-Only Timeline - screen-ready discussion page data

**Done**: [x]

**Scope**: Add the authenticated API and DTOs needed to render one discussion detail page without enabling mutations yet. The view should include repository/read permission checks, private repository privacy, title/status/category metadata, author and collaborator badges, sanitized Markdown body, initial comment, timeline comments, nested replies, reactions summary, poll/form answer summaries, sidebar data, viewer permissions, and stable sort controls.

**Key changes**:
- `crates/api/migrations/*_repository_discussion_detail.*.sql`: add any missing `discussion_comment_replies`, `discussion_answers`, `discussion_reactions`, `discussion_subscriptions`, and `discussion_events` columns/indexes needed by detail reads, preserving existing `discussion_comments`, `discussion_votes`, labels, polls, form answers, and notifications.
- `crates/api/src/domain/discussions.rs`: add DTOs such as `RepositoryDiscussionDetailView`, `DiscussionDetailSummary`, `DiscussionTimelineItem`, `DiscussionCommentView`, `DiscussionReplyView`, `DiscussionReactionSummary`, `DiscussionAnswerSummary`, `DiscussionSidebarView`, `DiscussionSubscriptionState`, `DiscussionDetailViewer`, and `DiscussionCommentSort`.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/discussions/{discussion_number}` with `sort=oldest|newest|top`, `page`, and `page_size`; enforce repository read permission, disabled/archived state visibility, private repository 404 privacy, and stable validation envelopes.
- Markdown and sanitization: reuse existing sanitized Markdown rendering for discussion bodies/comments/replies and ensure stored form answers, poll text, event payloads, and error responses never leak session/OAuth/env/storage secrets.
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed detail DTOs and signed-cookie fetch helpers without adding client-side auth.
- `crates/api/tests/repository_discussions_contract.rs`: extend seeded scenarios for detail reads, timeline ordering, nested replies, answer summary, reaction aggregates, form answers, poll metadata, private repository privacy, malformed sort validation, and no-secret error envelopes.

**Verification**: focused Rust detail contract tests against `TEST_DATABASE_URL`, `cd web && npx tsc --noEmit --pretty false`, focused Biome for touched web files, then full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and same-env `make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Detail Page - timeline, composer shell, and sidebar

**Done**: [ ]

**Scope**: Implement `/{owner}/{repo}/discussions/{number}` as a read-oriented Editorial detail page backed by Phase 1 data. Users should be able to navigate from list rows to the detail page, sort timeline comments, inspect Markdown content safely, copy/open permalinks, and see non-dead composer controls with submit disabled until Phase 3 writes land.

**Key changes**:
- `web/src/app/[owner]/[repo]/discussions/[number]/page.tsx`: server-fetch repository metadata plus discussion detail data, preserve sort/page URL params, render unavailable/private/not-found states safely, and keep the Discussions tab active.
- `web/src/components/RepositoryDiscussionDetailPage.tsx`: render title, discussion number, Open/Closed/Answered status, category breadcrumb, author metadata, upvote affordance, sanitized body, form-answer/poll summaries, timeline comments/replies, answer card when present, sort links, and right sidebar for Category, Labels, Participants, Notifications, and Events using Editorial primitives/tokens only.
- `web/src/components/RepositoryDiscussionTimeline.tsx`: render comments, replies, event rows, reaction summaries, permalink anchors, edited markers, collapsed deleted/moderated states, and no unsafe HTML.
- `web/src/components/RepositoryDiscussionReplyComposer.tsx`: render write/preview tabs, saved-replies/attachment controls, validation copy, and sign-in/permission affordances with actual disabled states until Phase 3 enables submission.
- `web/src/lib/navigation.ts`: ensure `repositoryDiscussionDetailHref`, comment anchor helpers, sort query helpers, and list back-links safely encode owner/repo/number/comment ids.
- `web/tests/repository-discussion-detail-page.test.tsx`: cover detail metadata, timeline sorting links, sidebar data, answer card, permalink anchors, composer disabled/signed-out states, active navigation, no `href="#"`, no inert controls, long Markdown wrapping, mobile no-overflow, unsafe HTML stripping, and banned Editorial visual guardrails.
- `web/tests/e2e/repository-discussions.spec.ts`: add focused browser smoke for list-to-detail navigation, sort links, answer anchor, sidebar, no dead controls, and screenshot `ralph/screenshots/build/discussions-003-phase2-detail.jpg` when a usable database URL is available.

**Verification**: focused Vitest and focused Playwright detail smoke when local DB credentials allow seeding, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 3: Comment, Reply, Reaction, and Subscription Writes - authenticated interaction loop

**Done**: [ ]

**Scope**: Enable real authenticated mutations for the most common reader interactions on a discussion detail page: add top-level comments, add nested replies, react/unreact to discussion/comment/reply targets, subscribe/unsubscribe, and preserve timeline sort/query state after mutations.

**Key changes**:
- API routes: add `POST /api/repos/{owner}/{repo}/discussions/{discussion_number}/comments`, `POST /api/repos/{owner}/{repo}/discussions/{discussion_number}/comments/{comment_id}/replies`, `PUT|DELETE /api/repos/{owner}/{repo}/discussions/{discussion_number}/reactions`, target-specific comment/reply reaction routes, and `PUT|DELETE /api/repos/{owner}/{repo}/discussions/{discussion_number}/subscription`.
- `crates/api/src/domain/discussions.rs`: add mutation helpers for comment/reply creation, sanitized Markdown persistence, bounded attachment metadata, reaction idempotency, subscription state, activity events, notification fanout, and viewer-specific response DTOs.
- Next.js same-origin proxies or route handlers following existing cookie-forwarding patterns for comment, reply, reaction, and subscription mutations.
- `RepositoryDiscussionReplyComposer`: enable submit, Markdown preview, attachment metadata, pending/error/success feedback, duplicate-submit protection, and redirect/refresh to the created comment or reply anchor.
- `RepositoryDiscussionReactionMenu` and subscription control: implement concrete buttons/menus with optimistic updates, rollback on API failure, keyboard support, signed-out fallback, and no inert click handlers.
- Tests: cover authenticated comment/reply create, Markdown sanitization, attachment bounds, reaction add/remove/idempotency, subscription toggles, signed-out denial, private repository privacy, disabled discussions/archived guardrails, notification rows, activity events, browser optimistic rollback, and screenshot `ralph/screenshots/build/discussions-003-phase3-interactions.jpg`.

**Verification**: focused Rust mutation contracts, focused Vitest for composer/reaction/subscription behavior, focused Playwright write smoke when seeded DB works, mandatory Editorial banned-value scan, then `make check && make test`.

---

## Phase 4: Answer State, Close/Reopen, and Sidebar Metadata Editing - maintainer controls

**Done**: [ ]

**Scope**: Add triage-or-greater controls for answer-enabled categories and discussion moderation metadata. Maintainers should be able to mark/unmark an answer, close/reopen with reasons, edit category/labels, and expose sidebar event history without weakening reader privacy.

**Key changes**:
- API routes: add `PUT|DELETE /api/repos/{owner}/{repo}/discussions/{discussion_number}/answer`, `PUT /api/repos/{owner}/{repo}/discussions/{discussion_number}/state`, and `PATCH /api/repos/{owner}/{repo}/discussions/{discussion_number}/metadata` for category/label updates.
- Domain validation: enforce triage-or-greater permission, answer-enabled category requirement, comment-in-same-discussion requirement, close/reopen reason validation, archived/locked guardrails, category compatibility for polls/forms, label id validation, event recording, audit metadata, and notification fanout.
- UI controls: add answer mark/unmark buttons on eligible comments, highlighted Answered by card with jump-to-answer link, Close/Reopen panel, category/labels menus in the sidebar, participants/events refresh, and clear permission-denied/signed-out affordances.
- Timeline events: render answer-marked, answer-unmarked, closed, reopened, category-changed, labels-changed, pinned, locked, transferred, deleted, and moderated-comment rows using the existing event DTO shape.
- Tests: cover answer-enabled vs normal categories, answer card anchors, mark/unmark validation, close/reopen reasons, category/label edits, reader denial, maintainer success, event rows, notification fanout, no secret leakage, no dead controls, and screenshot `ralph/screenshots/build/discussions-003-phase4-moderation.jpg`.

**Verification**: focused Rust moderation contracts, focused Vitest for answer/sidebar controls, focused Playwright maintainer smoke when DB credentials allow, mandatory Editorial banned-value scan, then `make check && make test`; run `make test-e2e` if the wrapper is stable.

---

## Phase 5: API Docs, Browser Evidence, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `discussions-003` only after detail reads, comments, replies, reactions, subscriptions, answer marking, close/reopen, sidebar metadata, docs, screenshots, QA handoff, and PRD bookkeeping are complete. Do not implement category administration, poll voting, repository-wide discussion search indexing, or organization discussion policy settings here.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document discussion detail, comment/reply creation, reactions, subscriptions, answer mark/unmark, close/reopen, and metadata editing endpoints with auth/privacy gates, validation envelopes, side effects, and no-secret guarantees.
- Final Rust tests: cover private repository privacy, disabled discussions, archived repositories, malformed discussion/comment ids, timeline sort bounds, Markdown sanitization, attachment limits, nested reply bounds, reaction idempotency, subscription toggles, answer/category/label validation, close/reopen events, notification/activity rows, and absence of session/OAuth/env/storage leaks.
- Final frontend tests: cover keyboard traversal through timeline, reaction menus, composer, preview, attachments, subscription, answer controls, close/reopen, category/label menus, long content wrapping, mobile no-overflow, no unsafe HTML, no `href="#"`, no inert click handlers, and Editorial token compliance.
- `web/tests/e2e/repository-discussions.spec.ts`: full signed-session browser sweep for list-to-detail navigation, sort, comment/reply creation, reaction toggles, subscription, answer marking, close/reopen, sidebar edits, signed-out affordances, and mobile layout.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/discussions-003/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `discussions-003.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/discussions-003-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
