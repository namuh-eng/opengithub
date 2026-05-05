# Structure Outline: discussions-001 Repository Discussions List

**Ticket**: `discussions-001`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-2.jsx`, `design/project/og-screens-4.jsx`, `design/project/og-shell.jsx`, existing repository shell/navigation patterns in `web/src/lib/navigation.ts`, repository header/tab patterns from `repo-003`, issue list filtering patterns from `issues-002`, notification/activity contracts from `notifications-001`, label contracts from `settings-006` and `label-management` work, and search empty-state behavior from `search-003`/`search-004`.
**Date**: 2026-05-05

## Existing Baseline

Repository tabs already include a Discussions destination, organization policy exposes a repository discussions toggle, and search keeps a Discussions tab with an explicit not-yet-indexed state. `discussions-001` should add the first real repository Discussions read surface with category navigation, filtering, pinned discussions, and upvote writes. It must use the Editorial design system, not GitHub Primer styling, and it should leave creation/detail/moderation/poll/category-management flows to later `discussions-*` features.

## Phase 1: Discussion List API Contract and Persistence - screen-ready list data

**Done**: [x]

**Scope**: Add repository-owned discussion list persistence and authenticated/readable API contracts without rendering the page yet. The API should support repository read permission, private repository 404 privacy, category-scoped reads, query-backed filters, sorting, pinned rows, labels, category rail data, helpful contributors, community links, and viewer-specific vote state.

**Key changes**:
- `crates/api/migrations/*_repository_discussions.*.sql`: add `discussion_categories`, `discussions`, `discussion_labels`, `discussion_comments` summary fields where needed, `discussion_votes`, `discussion_pins`, `repository_community_links`, saved query/activity rows if the existing tables do not already cover them, with indexes by repository/category/state/updated/vote count/comment count.
- `crates/api/src/domain/discussions.rs`: define DTOs such as `RepositoryDiscussionsView`, `DiscussionRow`, `PinnedDiscussionCard`, `DiscussionCategorySummary`, `DiscussionLabelSummary`, `HelpfulContributorSummary`, `CommunityLinkSummary`, `DiscussionViewer`, `DiscussionFilterState`, and `DiscussionSort`.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/discussions` and `GET /api/repos/{owner}/{repo}/discussions/categories/{slug}`, normalize `q`, `label`, `state`, `answered`, `locked`, `pinned`, `sort`, `page`, and `page_size`, and return standard validation envelopes for malformed filters.
- Seed/default helpers: create default categories for new repositories when discussions are enabled, enforce organization/repository discussions policy, and avoid returning category/pinned private details to unauthorized callers.
- `crates/api/tests/repository_discussions_contract.rs`: seed repository permissions, categories, labels, pinned discussions, answered/locked/closed rows, comments, votes, community links, and helpful contributors; verify DTO shape, filter/sort behavior, category scope, private repository privacy, disabled policy state, pagination, and no session/OAuth/env leakage.
- `web/src/lib/api.ts` and server fetch helpers: add typed Discussions DTOs and cookie-backed fetchers without adding client-side auth.

**Verification**: focused Rust contract tests against `TEST_DATABASE_URL`, `cd web && npx tsc --noEmit --pretty false`, focused Biome if web types are touched, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and same-env `make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Discussions List Page - repository tab, pinned cards, filters, and rows

**Done**: [x]

**Scope**: Implement `/{owner}/{repo}/discussions` inside the repository workspace with the Discussions tab active. The page should render Phase 1 data with URL-backed query controls, pinned cards, dense discussion rows, category rail, helpful contributors, community links, empty states, and concrete New discussion/detail/category links.

**Key changes**:
- `web/src/app/[owner]/[repo]/discussions/page.tsx`: server-fetch repository metadata and discussion list data, preserve URL query params, render unavailable/disabled/private states, and avoid leaking private repository metadata.
- `web/src/components/RepositoryDiscussionsPage.tsx`: render heading, query-builder search input defaulting to `is:open`, Pinned Discussions cards, status/filter/sort controls, New discussion link, discussion rows with category/label/state/comment/upvote metadata, right category rail, helpful contributors, and community links using Editorial primitives/tokens only.
- `web/src/components/RepositoryDiscussionFilters.tsx`: client URL controls for search, state, answered/unanswered, locked, pinned, labels, sort, and page with accessible menus, Apply/Clear behavior, outside-click/Escape handling, and no inert controls.
- `web/src/lib/navigation.ts`: add discussion list/category/detail/new href helpers that safely encode owners, repositories, category slugs, and discussion numbers.
- `web/tests/repository-discussions-page.test.tsx`: cover active repository Discussions tab, default `is:open` query, pinned cards, filter href/query composition, label/status chips, row/category/new links, disabled/empty states, no `href="#"`, no unsafe HTML, long title wrapping, mobile no-overflow, and Editorial banned-value guardrails.
- `web/tests/e2e/repository-discussions.spec.ts`: focused signed-session browser smoke for list filters, row/category navigation, empty category CTA, no dead controls, and screenshot `ralph/screenshots/build/discussions-001-phase2-list.jpg`.

**Verification**: focused Vitest and focused Playwright smoke, `cd web && npx tsc --noEmit --pretty false`, focused Biome checks, mandatory Editorial banned-value scan, then full `make check && make test`.

---

## Phase 3: Upvote Writes and Query State - optimistic controls backed by Rust

**Done**: [ ]

**Scope**: Wire discussion upvote buttons to real authenticated writes and preserve query/filter state through server-confirmed navigation. Anonymous or signed-out viewers should see a sign-in affordance; authenticated viewers should get optimistic count changes that reconcile with the API response.

**Key changes**:
- `PUT /api/repos/{owner}/{repo}/discussions/{discussion_number}/vote` and `DELETE /api/repos/{owner}/{repo}/discussions/{discussion_number}/vote`: enforce auth, repository read permission, disabled/archive guardrails, one vote per user, idempotent create/delete semantics, activity events, and notification fanout where subscribed.
- `crates/api/src/domain/discussions.rs`: add vote mutation helpers, vote-count recalculation, viewer-voted metadata, activity-event payloads, and error redaction.
- `web/src/app/[owner]/[repo]/discussions/actions/route.ts` or server action proxy: forward vote mutations with signed cookies and standardized error envelopes.
- `web/src/components/RepositoryDiscussionVoteButton.tsx`: optimistic upvote/unvote button with pending/success/error states, keyboard support, sign-in fallback, server reconciliation, and count rollback on error.
- Query-state handling: keep search/filter/sort/category qualifiers in URL params after voting and pagination; persist saved discussion query state only if the existing saved-state pattern is already present.
- Extend Rust, Vitest, and Playwright coverage for authenticated vote/unvote, idempotency, signed-out denial, private repository privacy, disabled discussions, optimistic rollback, no dead controls, and screenshot `ralph/screenshots/build/discussions-001-phase3-votes.jpg`.

**Verification**: focused Rust vote contracts, focused Vitest, focused Playwright smoke, mandatory Editorial banned-value scan, then `make check && make test`.

---

## Phase 4: Category Route and Rail Behavior - scoped list, category-specific empty states

**Done**: [ ]

**Scope**: Implement `/{owner}/{repo}/discussions/categories/{slug}` as a category-scoped variant of the list page. Category links should compose category qualifiers with the current query, update the page heading, show category metadata, and produce a category-specific empty state with a working New discussion CTA.

**Key changes**:
- `web/src/app/[owner]/[repo]/discussions/categories/[slug]/page.tsx`: server-fetch category-scoped list data, reject unknown categories with a repository-safe not-found state, and preserve query/filter/sort URL state.
- `RepositoryDiscussionsPage`: accept optional selected category metadata, render category heading/description/emoji, category qualifier chip, active rail item, and empty states that link to `/{owner}/{repo}/discussions/new?category={slug}`.
- API/category filtering: ensure the Phase 1 category endpoint and general endpoint share the same normalization, pagination, permission, and disabled-state behavior.
- Helpful contributor and community rail: make category route keep the same sidebar data while marking category-specific counts and avoiding layout shifts on mobile.
- Extend tests for category route heading, active rail state, unknown category 404/privacy behavior, query composition, empty category CTA, long category names, and mobile no-overflow.
- Save browser screenshot `ralph/screenshots/build/discussions-001-phase4-category.jpg`.

**Verification**: focused Rust category contract where needed, focused Vitest, focused Playwright category smoke, mandatory Editorial banned-value scan, then `make check && make test`; run full `make test-e2e` when the local wrapper is stable.

---

## Phase 5: API Docs, Browser Evidence, QA Handoff, and Build-Pass Bookkeeping

**Done**: [ ]

**Scope**: Finish `discussions-001` only after the list, category route, filters, pinned cards, category rail, upvote writes, docs, screenshots, QA handoff, and PRD bookkeeping are complete. Do not expand into discussion creation, detail timelines, comments, answer marking, polls, moderation, or category management.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document discussion list/category/vote endpoints with auth/privacy gates, filters, sorting, pagination, disabled discussions policy, pinned rows, category rail data, vote semantics, activity/notification side effects, and no-secret error envelopes.
- Final Rust tests: cover private repositories, disabled discussions, malformed filters, category slugs, label/status/search composition, pinned-card limits, helpful contributor calculation, duplicate votes, vote removal, archived repositories, notification/activity writes, and absence of session/OAuth/env/storage leaks.
- Final frontend tests: cover keyboard traversal through search/filter/sort/vote/category rail controls, active tab/rail state, optimistic vote reconciliation, empty category CTA, no unsafe HTML, no `href="#"`, no inert click handlers, long text wrapping, mobile no-overflow, and Editorial token compliance.
- `web/tests/e2e/repository-discussions.spec.ts`: full signed-session browser sweep for list filters, pinned cards, category route, row/detail destinations, vote/unvote, signed-out vote affordance if practical, empty states, and mobile layout.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/discussions-001/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `discussions-001.build_pass=true` only after final verification passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/discussions-001-final-*.jpg`.

**Verification**: focused contract/unit/E2E checks, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full same-env `make test`, full same-env `make test-e2e` or direct Playwright equivalent if the wrapper stalls, JSON validation for `prd.json` and `qa-hints.json`, and mandatory Editorial banned-value scan: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
