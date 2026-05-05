# Structure Outline: discussions-006 Discussion Polls

**Ticket**: `discussions-006`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, `design/project/og-screens-3.jsx`, `design/project/og-screens-4.jsx`, `design/project/og-shell.jsx`, existing `discussions-001` category list/read contracts, `discussions-002` poll creation contracts, `discussions-003` detail timeline/comment/reaction/subscription contracts, `discussions-004` category format administration, `discussions-005` moderation/management guardrails, and Editorial list/detail/sidebar primitives.
**Date**: 2026-05-06

## Existing Baseline

`discussions-001` through `discussions-005` already provide repository discussion lists, category-scoped creation, poll-category creation fields, read-only poll detail rendering, timeline comments/replies/reactions, answer/state/sidebar controls, category administration, and moderation/management. `discussions-006` should make poll categories and poll discussions operational: category list/detail filtering, poll voting constraints, result visibility, vote updates, comments/reactions continuity, and poll-specific moderation compatibility. It must preserve signed Rust-session auth, repository privacy, existing discussion comments/reactions, and the Editorial visual system. Do not implement organization-wide poll policy, poll search indexing, advanced analytics exports, or non-discussion survey tooling here.

## Phase 1: Poll Category Read Contract - polls list and detail metadata

**Done**: [ ]

**Scope**: Make `/{owner}/{repo}/discussions/categories/polls` and poll discussion detail reads first-class. The polls category page should filter to poll discussions, expose poll row metadata, and detail DTOs should include enough poll policy/result state for later voting UI without accepting votes yet.

**Key changes**:
- `crates/api/src/domain/discussions.rs`: extend list/category DTOs with `categoryQualifier`, `pollSummary`, `viewerCanVote`, `resultsVisible`, `viewerVoteOptionIds`, and poll-specific unavailable reasons derived from repository privacy, auth, category format, archived/locked state, and poll policy.
- `crates/api/src/routes/repositories.rs`: preserve existing discussion list/category routes while ensuring `category=polls` and `/discussions/categories/polls` use poll category format rather than title-only matching.
- `web/src/lib/api.ts`: add typed poll summary/detail fields without breaking existing Discussion list/detail consumers.
- `web/src/components/RepositoryDiscussionsPage.tsx`: render the Polls category title, category qualifier chip, poll emoji on each row, Polls description, upvote/comment counts, empty state CTA to the poll composer, and concrete row links.
- Tests: cover category filtering, private repository privacy, anonymous result visibility metadata, deleted/closed/locked poll row state, no normal discussions in the Polls category, and no GitHub visual values.

**Verification**: focused Rust read-contract tests against `TEST_DATABASE_URL`, focused Vitest list/detail tests, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`. Browser smoke should save `ralph/screenshots/build/discussions-006-phase1-polls-category.jpg` when seeded DB credentials allow.

---

## Phase 2: Poll Vote API - one vote, change policy, and result math

**Done**: [ ]

**Scope**: Add authenticated poll voting persistence and mutation endpoints. A signed-in viewer should be able to cast one option vote, or multiple option votes only when the poll allows multiple choices; changing a vote should be accepted only when the poll policy allows it.

**Key changes**:
- `crates/api/migrations/*_repository_discussion_poll_votes.*.sql`: create or extend `discussion_poll_votes` with `poll_id`, `option_id`, `user_id`, timestamps, optional `replaced_at`, and uniqueness constraints that enforce one active vote per user/option plus one active choice for single-choice polls.
- `crates/api/src/domain/discussions.rs`: add `DiscussionPollVoteRequest`, `DiscussionPollVoteResponse`, result aggregation helpers, change-policy validation, anonymous/sign-in prompt states, archived/locked/deleted/private guardrails, and transaction-safe activity/audit rows.
- `crates/api/src/routes/repositories.rs`: register `PUT /api/repos/{owner}/{repo}/discussions/{discussion_number}/poll/vote` and optional `DELETE /poll/vote` if unvoting is supported by the policy.
- Side effects: update result counts/percentages, preserve discussion comments/reactions/subscriptions, create notifications or activity events only where product behavior expects visible poll activity, and never leak private poll result data to unauthorized viewers.
- Tests: cover anonymous denial, reader permission, single-choice replacement denial/allowance per policy, multiple-choice validation, invalid option ids, cross-poll option rejection, locked/archived/deleted denial, result percentages rounding to 100, idempotent duplicate submissions, and no secret leakage.

**Verification**: focused Rust voting contract tests against `TEST_DATABASE_URL`, `cargo fmt --all --check`, `cargo check -p opengithub-api --tests`, typed frontend compile, then full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and same-env `make test`. Browser smoke is optional for this API-first phase.

---

## Phase 3: Editorial Poll Detail UI - voting controls and result bars

**Done**: [ ]

**Scope**: Replace the current read-only poll detail block on `/{owner}/{repo}/discussions/{number}` with operational Editorial voting controls backed by Phase 2. Radio/checkbox controls, vote actions, sign-in prompts, and result bars should all be concrete and accessible.

**Key changes**:
- `web/src/components/RepositoryDiscussionDetailPage.tsx`: render poll option rows with radio controls for single-choice polls, checkboxes for multiple-choice polls, disabled/unavailable states, Vote/Update vote action, inline success/error feedback, percentage/count result bars after voting or when results are visible, and long-option wrapping.
- Same-origin Next.js proxy route under `web/src/app/api/repos/[owner]/[repo]/discussions/[number]/poll/vote/route.ts` for cookie-forwarded voting.
- `web/src/lib/api.ts`: add `voteDiscussionPoll` helper and update detail refresh handling after a vote.
- UI constraints: anonymous users see a sign-in prompt instead of a dead control; private repository unauthorized viewers never see counts; locked discussions follow the server's poll/reaction policy; comments/replies timeline remains usable below the poll.
- Tests: cover radio/checkbox behavior, disabled submit until a valid selection exists, vote payloads, update vote payloads, result bar rendering, server error display, anonymous prompt links, keyboard traversal, no `href="#"`, no inert click handlers, mobile no-overflow, and Editorial token compliance.
- `web/tests/e2e/repository-discussions.spec.ts`: add focused browser smoke for opening a poll discussion, voting, seeing result bars, changing vote where allowed, and screenshot `ralph/screenshots/build/discussions-006-phase3-poll-vote.jpg`.

**Verification**: focused Vitest detail-page poll tests, focused Playwright poll smoke when DB credentials allow, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then `make check && make test`.

---

## Phase 4: Poll Constraints, Comments, Reactions, and Moderation Compatibility

**Done**: [ ]

**Scope**: Finish the poll-specific rules that cross existing discussion systems. Poll discussions should keep comments/replies/reactions/subscriptions working, reject moves between poll and non-poll categories, and expose clear moderator/reader states without corrupting poll votes.

**Key changes**:
- `crates/api/src/domain/discussions.rs`: harden move/transfer/delete/lock/reaction/comment paths around poll discussions, including "poll discussions cannot move to non-poll categories" and "normal discussions cannot move into poll categories" wherever not already enforced.
- `RepositoryDiscussionDetailPage` and related sidebar controls: surface poll-specific unavailable reasons for category change, transfer, lock/reaction policy, and deleted/tombstoned poll state using `.chip.warn`/`.chip.err` and existing Editorial cards.
- Preserve comments/replies/reactions timeline: ensure poll votes do not create duplicate comments, reactions remain permission-aware, and subscriptions/notifications keep their existing semantics.
- Tests: cover poll category move/transfer rejection, comment/reply/reaction continuity after voting, locked poll reaction policy, moderator states, deleted/tombstoned privacy, long poll/result layout, and no unsafe HTML in poll options or comments.
- Save screenshot `ralph/screenshots/build/discussions-006-phase4-poll-constraints.jpg` when seeded browser smoke is available.

**Verification**: focused Rust compatibility tests, focused Vitest detail/sidebar tests, focused Playwright poll constraints smoke when possible, mandatory Editorial banned-value scan, then `make check && make test`; run `make test-e2e` if the wrapper is stable.

---

## Phase 5: API Docs, Browser Evidence, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `discussions-006` only after poll category reads, vote APIs, operational poll UI, cross-feature constraints, docs, screenshots, QA handoff, and PRD bookkeeping are complete.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document poll category list/detail behavior, poll creation constraints inherited from `discussions-002`, vote/unvote or vote-update endpoints, result visibility, auth/privacy gates, single vs multiple choice rules, change policy, category move/transfer incompatibility, comments/reactions continuity, activity/audit side effects, and no-secret response guarantees.
- Final Rust tests: cover private repository privacy, anonymous result suppression, malformed poll/option ids, vote replacement policy, multi-choice aggregation, locked/archived/deleted denial, category compatibility, activity/audit rows, and absence of session/OAuth/env/storage leaks.
- Final frontend tests: cover poll category page, detail voting, result bars, comments/replies/reactions after vote, sign-in prompt, server errors, keyboard traversal, long option text, mobile no-overflow, no unsafe HTML, no `href="#"`, no inert buttons, and Editorial token compliance.
- `web/tests/e2e/repository-discussions.spec.ts`: full signed-session browser sweep for poll category filtering, create poll, vote, update vote, anonymous prompt, results bars, comments/reactions, moderator constraints, and mobile layout.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/discussions-006/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `discussions-006.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/discussions-006-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
