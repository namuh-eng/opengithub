# Structure Outline: discussions-005 Discussion Moderation and Management

**Ticket**: `discussions-005`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, `design/project/og-screens-3.jsx`, `design/project/og-screens-4.jsx`, `design/project/og-shell.jsx`, existing `discussions-001` list/pin reads, `discussions-002` creation contracts, `discussions-003` detail timeline/comment/state controls, `discussions-004` category administration, issue detail/comment/timeline patterns, repository permission helpers, notification/audit helpers, and Editorial sidebar/action-menu primitives.
**Date**: 2026-05-06

## Existing Baseline

`discussions-001` through `discussions-004` already provide repository discussion lists, pinned-card reads, category-scoped creation, YAML forms/polls, detail timelines, comments/replies/reactions/subscriptions, answer/close/sidebar metadata controls, and category administration. `discussions-005` should complete maintainer moderation and management: global/category pin lifecycle, lock/unlock with reaction policy, transfer/delete, richer close/reopen and recategorization management, and issue-to-discussion conversion. It must preserve server-side permission enforcement and Editorial design, and should not expand into poll voting, organization-wide discussion policy, repository search indexing, or general issue migration tooling beyond conversion to discussions.

## Phase 1: Moderation Contract - pin, lock, close, and recategorize foundations

**Done**: [x]

**Scope**: Add the authenticated moderation API surface and persistence needed for maintainer controls without shipping the full UI yet. Triage/write/admin users should be able to pin/unpin globally or within a category, customize pinned copy, lock/unlock with optional reactions, close/reopen with stable reasons, and recategorize eligible non-poll discussions.

**Key changes**:
- `crates/api/migrations/*_repository_discussion_moderation.*.sql`: add or extend `discussion_pins` with `pin_scope`, `category_id`, custom title/body, and unique position constraints; add `discussion_locks` or lock metadata columns for allow-reactions policy; add moderation-specific event/audit indexes if existing `discussion_activity_events` and `audit_events` need stronger lookup support.
- `crates/api/src/domain/discussions.rs`: add DTOs such as `DiscussionModerationPanel`, `DiscussionPinTarget`, `PinDiscussionRequest`, `UpdatePinnedDiscussionRequest`, `LockDiscussionRequest`, `CloseDiscussionRequest`, and `RecategorizeDiscussionRequest`.
- `crates/api/src/routes/repositories.rs`: register `PUT/PATCH/DELETE /api/repos/{owner}/{repo}/discussions/{discussion_number}/pin`, `PUT/DELETE /lock`, `PUT /state`, and `PATCH /category` or extend existing state/metadata routes with the richer moderation contract.
- Domain validation: enforce triage-or-greater permission, private repository privacy, archived/disabled guardrails, locked-comment behavior, four-global and four-category pin limits, no duplicate pin targets, category pin target validity, non-poll recategorization, form/poll compatibility, close reason validation, and no-secret error envelopes.
- Side effects: write `discussion_activity_events`, `audit_events`, notifications, updated `discussion_pins` ordering, and refreshed list/detail DTOs used by `discussions-001` and `discussions-003`.
- Tests: cover permission denial, pin limits, global/category pin uniqueness, pinned customization, unpin reorder behavior, lock allow-reactions behavior, close/reopen reasons, recategorization eligibility, archived/disabled denial, activity/audit rows, notification rows, and absence of session/OAuth/env leakage.

**Verification**: focused Rust moderation contract tests against `TEST_DATABASE_URL`, `cargo fmt --all --check`, `cargo check -p opengithub-api --tests`, `cd web && npx tsc --noEmit --pretty false` for DTO compatibility, focused Biome for touched web files, then full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and same-env `make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Moderator Sidebar - pinning, locking, close, and category controls

**Done**: [x]

**Scope**: Render the maintainer-facing discussion sidebar and action menus on `/{owner}/{repo}/discussions/{number}` using the Phase 1 APIs. Every button in the sidebar/action menu should either perform the real mutation or be visibly unavailable with a concrete reason.

**Key changes**:
- `web/src/components/RepositoryDiscussionDetailPage.tsx`: add a moderator controls card in the sidebar with Pin discussion, Pin to category, Edit pinned discussion, Unpin, Lock conversation, Close/Reopen, and Change category controls while preserving the existing answer/label/subscription controls.
- `RepositoryDiscussionModerationPanel`: implement accessible menus/dialogs for pin target selection, pinned title/body preview, lock allow-reactions checkbox, close reason selection, category picker, pending/error/success states, optimistic refresh from server responses, and no inert controls.
- Same-origin Next.js proxy routes under `web/src/app/api/repos/[owner]/[repo]/discussions/[number]/...` for pin, lock, state, and category mutations where client interactivity needs cookie-forwarding.
- Use only Editorial primitives and tokens: `.card`, `.btn`, `.chip`, `.input`, `.tabs`, `.list-row`, `.t-*`, `var(--accent)`, `var(--ok)`, `var(--warn)`, and `var(--err)`.
- Tests: cover moderator panel rendering, pin/update/unpin payloads, lock/unlock allow-reactions payloads, close/reopen reason payloads, category change payloads, server error display, unavailable reader states, long pinned-copy wrapping, no `href="#"`, no inert click handlers, no unsafe HTML, mobile no-overflow, and mandatory Editorial banned-value guardrails.
- `web/tests/e2e/repository-discussions.spec.ts`: add focused browser smoke for opening each moderation dialog/menu, submitting one pin, locking/unlocking, closing/reopening, changing category, and screenshot `ralph/screenshots/build/discussions-005-phase2-moderator-sidebar.jpg` when a usable seeded database is available.

**Verification**: focused Vitest detail-page tests, focused Playwright moderation smoke when DB credentials allow seeding, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 3: Transfer and Delete - destructive discussion management with tombstones

**Done**: [x]

**Scope**: Add safe destructive management for discussions. Maintainers should be able to transfer a discussion only to an allowed repository under the same owner constraints and delete a discussion only after explicit confirmation that records a tombstone/audit trail.

**Key changes**:
- API routes: add `GET /api/repos/{owner}/{repo}/discussions/{discussion_number}/transfer-targets`, `POST /transfer`, and `DELETE /discussion` or route equivalents under the existing discussion detail namespace.
- Domain validation: enforce admin/write permission as appropriate, same-owner transfer constraints, destination repository discussion-enabled state, category compatibility at destination, private repository privacy, archived/disabled guardrails, locked-state semantics, explicit confirmation string for delete, and idempotent not_found/deleted envelopes.
- Persistence side effects: update `repository_id`, `category_id`, discussion number mapping or destination numbering strategy atomically on transfer; write source/destination timeline events; move subscriptions/notifications safely; create deletion tombstones without leaking deleted body/comment contents; preserve audit rows for both operations.
- `RepositoryDiscussionDetailPage`: add Transfer discussion repository picker and Delete discussion confirmation dialog with explicit warnings, disabled/confirm states, destination/category selectors, concrete success navigation, and no dead destructive buttons.
- Tests: cover allowed/denied transfer targets, same-owner constraints, destination disabled/archived denial, category compatibility, event rows, notification updates, tombstone creation, delete confirmation validation, private repository privacy, no deleted-content leakage, and UI destructive confirmation behavior.
- Save screenshot `ralph/screenshots/build/discussions-005-phase3-transfer-delete.jpg` when seeded browser smoke is available.

**Verification**: focused Rust transfer/delete contracts, focused Vitest destructive-dialog tests, focused Playwright transfer/delete smoke when DB credentials allow, mandatory Editorial banned-value scan, then `make check && make test`; run `make test-e2e` if the wrapper is stable.

---

## Phase 4: Issue Conversion - convert issue detail into discussion

**Done**: [x]

**Scope**: Implement issue-to-discussion conversion from `/{owner}/{repo}/issues/{number}`. A maintainer should choose a discussion category, convert the issue title/body/comments metadata into a new discussion, link both timelines, close or mark the source issue as converted, and navigate to the new discussion.

**Key changes**:
- API routes: add `GET /api/repos/{owner}/{repo}/issues/{issue_number}/convert-to-discussion` and `POST /convert-to-discussion` with eligible category metadata and conversion status.
- Domain conversion logic: validate triage/write/admin permission, issue existence/privacy, not already converted, destination category eligibility, poll/form incompatibility, archived/disabled repository guardrails, comment copy/metadata strategy, author attribution, attachment references, and duplicate submission idempotency.
- Persistence side effects: insert a new discussion, initial body/comment records, converted-comment metadata where supported, `issue_timeline_events`, `discussion_activity_events`, notifications, audit events, and a durable link from source issue to destination discussion.
- `web/src/components/RepositoryIssueDetailPage.tsx` or existing issue detail/sidebar component: add Convert to discussion dialog with category picker, comment-copy summary, explicit warning about issue state/linking, submit feedback, and success navigation to `/{owner}/{repo}/discussions/{number}`.
- Tests: cover conversion metadata read, successful conversion, duplicate conversion denial, non-maintainer denial, disabled discussions denial, category eligibility, copied title/body/comments metadata, source issue event, destination discussion event, notification/audit rows, no secret leakage, and UI dialog/no-dead-control behavior.
- Save screenshot `ralph/screenshots/build/discussions-005-phase4-issue-conversion.jpg`.

**Verification**: focused Rust issue-conversion contracts, focused Vitest issue-detail conversion tests, focused Playwright issue conversion smoke when seeded DB works, mandatory Editorial banned-value scan, then `make check && make test`; run direct Playwright equivalent if `make test-e2e` stalls.

---

## Phase 5: API Docs, Browser Evidence, QA Handoff, and Build-Pass Bookkeeping

**Done**: [x]

**Scope**: Finish `discussions-005` only after moderation/management contracts, discussion sidebar controls, destructive transfer/delete, issue conversion, docs, screenshots, QA handoff, and PRD bookkeeping are complete. Do not implement poll voting, organization-wide discussion policy, repository-wide discussion search indexing, or unrelated issue migration features here.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document discussion pin/update/unpin, lock/unlock, close/reopen, recategorize, transfer-targets/transfer, delete/tombstone, and issue convert-to-discussion endpoints with auth/privacy gates, validation envelopes, side effects, destructive confirmation, limits, and no-secret guarantees.
- Final Rust tests: cover private repository privacy, archived/disabled repositories, malformed ids/slugs, pin target limits, lock reaction policy, close reasons, recategorization, transfer constraints, delete tombstones, issue conversion idempotency, activity/audit rows, notification fanout, and absence of session/OAuth/env/storage leaks.
- Final frontend tests: cover keyboard traversal through moderator/sidebar menus, dialogs, destructive confirmations, category/repository pickers, issue conversion, server error display, long content wrapping, mobile no-overflow, no unsafe HTML, no `href="#"`, no inert buttons, and Editorial token compliance.
- `web/tests/e2e/repository-discussions.spec.ts`: full signed-session browser sweep for pin/edit/unpin, category pin, lock/unlock with reactions, close/reopen, recategorize, transfer, delete confirmation, issue conversion, reader denial, and mobile layout.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/discussions-005/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `discussions-005.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/discussions-005-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
