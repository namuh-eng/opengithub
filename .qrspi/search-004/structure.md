# Structure Outline: search-004 Issue and Pull Request Search Results

**Ticket**: `search-004`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/og-screens-3.jsx`, `design/project/og-shell.jsx`, `.qrspi/search-002/structure.md`, `.qrspi/search-003/structure.md`, current `crates/api/src/domain/search.rs`, current `crates/api/src/routes/search.rs`, current `web/src/components/SearchResultsPage.tsx`, current `web/src/app/search/page.tsx`, current `web/src/lib/navigation.ts`, and `target-docs/content/rest/search/search.md`.
**Date**: 2026-05-02

## Phase 1: Collaboration Search API Contract - issues and pull requests return facets, counts, snippets, sort metadata, and permission-safe rows

**Done**: [x]

**Scope**: Add a richer response contract for `type=issues` and `type=pull_requests` while preserving the existing `/api/search` list envelope for other result types. The API should expose result-type counts, active qualifier chips, advanced facet groups, sort state, query timing, highlighted text-match snippets, pagination, and dense issue/PR row metadata from real indexed issue and pull request documents. Private repository metadata must remain invisible to viewers without access.

**Key changes**:
- `crates/api/src/domain/search.rs`: introduce a collaboration-search query/response contract parallel to `CodeSearchResponse`, with `type_counts`, `facets`, `active_chips`, `sort`, `query_duration_ms`, `diagnostics`, and row metadata for repository, title, number, state, close reason, labels, author, assignees, milestone, linked PR presence, comment count, interaction count, opened/updated/closed timestamps, and match snippets.
- `crates/api/src/routes/search.rs`: route `type=issues`, `type=issue`, `type=pull_requests`, `type=pulls`, and `type=pullrequests` through the collaboration-search path, normalize aliases for UI compatibility, apply page/pageSize clamps, and keep errors in the standard envelope.
- `crates/api/tests/search_collaboration_results_contract.rs`: seed real users, repositories, issues, pull requests, labels, comments, reactions/interactions, milestones, assignees, linked PR relationships, public/private permissions, and verify result shapes, facet counts, highlighted snippets, alias normalization, pagination, and private result redaction.
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed `CollaborationSearchResponse`, query DTOs, and server fetch helpers without breaking `searchGlobal` consumers.

**Verification**: focused `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false cargo test --test search_collaboration_results_contract -- --nocapture`, then same-env `make check && make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Collaboration Results Shell - `/search?type=issues` and `/search?type=pull_requests` use the shared two-pane search workspace

**Done**: [x]

**Scope**: Replace the generic issue/PR result rendering with a dedicated Editorial collaboration-search layout that matches the code-search shell: left rail, result-type switching, active scope chips, count/timing header, Save button, Sort menu, view options, and dense rows. The page must keep the Editorial design system locked: tokens, `.btn`, `.chip`, `.card`, `.input`, `.tabs`, and type ramp only.

**Key changes**:
- `web/src/components/SearchResultsPage.tsx`: branch issue and pull request result types to a new shared collaboration-search page while preserving repositories/users/orgs/commits/discussions generic behavior.
- `web/src/components/CollaborationSearchResultsPage.tsx`: render the two-pane shell with Type menu links for Code, Issues, Pull requests, Discussions, Commits, Packages, and Wikis; header count/timing; active chips; Save link/dialog entry point; sort button/menu; view controls; and dense row cards for issues/PRs.
- `web/src/lib/navigation.ts`: add URL builders for switching result types, preserving query/type/sort/page/view, and issue/PR detail links.
- `web/tests/search-collaboration-results.test.tsx`: assert shell structure, result-type links, issue and PR row metadata, label chips, snippets, no dead links/buttons, and Editorial token usage.

**Verification**: focused `cd web && npx vitest run tests/search-collaboration-results.test.tsx`, then full `make check` and `make test` with the repo's TEST_DATABASE_URL environment. Save a browser screenshot only if the phase adds enough UI for meaningful visual smoke.

---

## Phase 3: Advanced Facets and Qualifier Controls - filters mutate URL-backed collaboration search results

**Done**: [ ]

**Scope**: Make the left rail filters real. Owner, state, close reason, linked pull request, author, assignee, mentioned user, mentioned team, commenter, involved user, label, milestone, number of comments, number of interactions, and advanced search controls should add/remove qualifiers and refresh results through concrete URLs. Empty/unsupported facet values should be disabled or absent, never inert.

**Key changes**:
- `crates/api/src/domain/search.rs`: parse and apply collaboration qualifiers such as `repo:`, `org:`, `user:`, `owner:`, `is:issue`, `is:pr`, `state:`, `closed:`, `reason:`, `linked:pr`, `author:`, `assignee:`, `mentions:`, `team:`, `commenter:`, `involves:`, `label:`, `milestone:`, `comments:`, and `interactions:` with structured diagnostics for malformed or unsupported ranges.
- `web/src/components/CollaborationSearchResultsPage.tsx`: render facet groups with selected state, removable active chips, advanced filter inputs, empty facet states, and accessible controls that preserve query/type/sort/page/view.
- `web/src/lib/navigation.ts`: add qualifier add/remove/toggle helpers for quoted values and numeric range qualifiers.
- `web/tests/search-collaboration-results.test.tsx` and `web/tests/e2e/search-collaboration.spec.ts`: cover facet URL generation, chip removal, state/label/milestone/assignee filters, invalid qualifier recovery, and no `href="#"` or placeholder handlers.

**Verification**: focused Rust collaboration-search contract, focused Vitest coverage, focused Playwright smoke saving `ralph/screenshots/build/search-004-phase3-facets.jpg`, then `make check`, `make test`, and `make test-e2e`.

---

## Phase 4: Sort, Save, Pagination, and Result Navigation - every advertised collaboration-search interaction is concrete

**Done**: [ ]

**Scope**: Complete the user workflow around issue/PR search. Sort menu options should update URL/results for Best match, Most/Least commented, Newest, Oldest, Recently updated, and Least recently updated. Pagination must preserve query, type, sort, view, and qualifiers. Save should reuse the `search-002` saved-search API. Clicking rows must open real issue or PR detail routes.

**Key changes**:
- `crates/api/src/domain/search.rs`: implement collaboration sort mapping, deterministic tie-breaks, comment/interaction count ordering, newest/oldest/opened/updated ordering, and page/window bounds.
- `web/src/app/search/page.tsx`: parse and forward collaboration `sort`, `view`, and advanced query params to the server helper without applying code-search-only qualifiers.
- `web/src/components/CollaborationSearchResultsPage.tsx`: add working sort menu, compact/comfortable view state, saved-search dialog integration or saved-search deep link, pagination controls, row click/detail navigation, empty-state CTAs that start a concrete adjusted search, and inline success/error feedback.
- `web/tests/e2e/search-collaboration.spec.ts`: signed-in browser smoke for issue and PR result navigation, sort changes, pagination preservation, saved-search creation/opening, compact view, and empty state recovery.

**Verification**: focused Rust contract, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/search-004-phase4-sort-save-pagination.jpg`, then full `make check`, `make test`, and `make test-e2e`.

---

## Phase 5: Collaboration Search Guardrails and QA Handoff - finish search-004

**Done**: [ ]

**Scope**: Harden accessibility, privacy, responsive behavior, performance bounds, and bookkeeping. Mark `search-004.build_pass=true` only after issue and pull request search use real API data, filters and sort are URL-backed, result rows navigate to detail pages, saved search writes persist, and no visible controls are dead.

**Key changes**:
- `crates/api/tests/search_collaboration_results_contract.rs`: final coverage for private repository redaction, issue/PR permission boundaries, query length and qualifier diagnostics, closed/merged state semantics, facet count consistency, pagination limits, sort determinism, and absence of stack traces or secret leakage.
- `web/tests/search-collaboration-results.test.tsx`: final accessibility, keyboard, no-dead-control, responsive no-overflow, label/snippet truncation, disabled-state, and Editorial visual guardrails.
- `web/tests/e2e/search-collaboration.spec.ts`: desktop and mobile signed-in smoke for issue and PR searches, type switching, facets, chips, sort, pagination, saved-search creation, row navigation, invalid query recovery, and empty states.
- `ralph/screenshots/build/`: save final desktop and mobile screenshots for issue and PR search result pages.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`, `build-progress.txt`, and `prd.json`: record verification evidence and set `search-004.build_pass=true` only after the final phase passes; leave `qa_pass=false`.

**Verification**: `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make test && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session make test-e2e`; browser smoke proves every issue/PR search button, row, facet, chip, sort option, save flow, pagination link, empty state, and error state has a concrete action; mandatory Editorial banned-value scan returns zero matches.
