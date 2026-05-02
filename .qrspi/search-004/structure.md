# Structure Outline: search-004 Issue and Pull Request Search Results

**Ticket**: `search-004`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/wf-search.jsx`, `.qrspi/search-001/structure.md`, `.qrspi/search-002/structure.md`, current `crates/api/src/domain/search.rs`, current issue/PR domains and routes, current `/search` page, and existing search result tests.
**Date**: 2026-05-02

## Phase 1: Collaboration Search API Contract - issues and PRs expose filters, facets, and sort

**Done**: [x]

**Scope**: Add a richer permission-aware contract for `/api/search?q=...&type=issues|pull_requests` without changing repository/code/commit behavior. Results should include issue/PR rows plus state/label/assignee/reviewer/milestone facets, active query chips, sort options, type counts, and query timing.

**Key changes**:
- `crates/api/src/domain/search.rs`: add collaboration-search query/response DTOs, parse issue/PR qualifiers (`state:`, `is:`, `label:`, `author:`, `assignee:`, `reviewer:`, `milestone:`), apply supported filters against indexed metadata, and expose removable chips/facets.
- `crates/api/src/routes/search.rs`: branch issue and pull request searches to the richer contract and pass through `sort`, pagination, and standard error envelopes.
- `crates/api/tests/search_collaboration_results_contract.rs`: seed real indexed issue/PR documents and verify result shape, private filtering, state/label filters, sort, facets, chips, and stable detail links.

**Verification**: focused `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false cargo test --test search_collaboration_results_contract -- --nocapture`, then `make check` and `make test` with the same DB environment.

---

## Phase 2: Editorial Collaboration Results Shell - `/search` renders issue/PR workspaces

**Done**: [x]

**Scope**: Render `type=issues` and `type=pull_requests` with the same calm two-pane search workspace as code search while avoiding CodeSearchResultsPage overlap. The rail shows state and advanced facets; the main pane shows counts, timing, sort menu links, Save entry point, snippets, labels, assignees/reviewers, milestones, state chips, and pagination.

**Key changes**:
- `web/src/lib/api.ts` / `web/src/lib/server-session.ts`: add typed collaboration-search helpers.
- `web/src/app/search/page.tsx`: route issue/PR tabs to the collaboration helper while preserving generic tabs and the code-search branch.
- `web/src/components/CollaborationSearchResultsPage.tsx`: new component for issue/PR results with Editorial tokens, concrete facet/sort/type-switch/pagination links, and accessible empty/error states.
- `web/tests/search-collaboration-results.test.tsx`: cover issue/PR rendering, state/label/assignee/reviewer/milestone metadata, filter and sort URLs, pagination preservation, and no dead controls.

**Verification**: focused Vitest for collaboration results, mandatory Editorial banned-value scan, then `make check` and `make test`.

---

## Phase 3: Advanced Facets and Snippets - filters and dense rows match PRD behavior

**Done**: [ ]

**Scope**: Deepen issue/PR facets for close reason, linked pull request, mentioned/commenter/involved users or teams, comment/interaction counts, and highlighted snippets as indexing metadata becomes available.

**Key changes**:
- Extend indexed issue/PR metadata during mutation paths with comment counts, interaction counts, linked PR hints, milestones, assignees, reviewers, close reasons, and participant fields.
- Apply the additional qualifiers in the collaboration search SQL and return validation diagnostics for unsupported or malformed filters.
- Extend UI rows with highlighted title/body snippets and richer participant chips.
- Add Rust and Vitest coverage for the additional qualifiers and snippet highlighting.

**Verification**: focused Rust + Vitest, then `make check` and `make test`.

---

## Phase 4: Browser Smoke and Pagination Guardrails - real navigation stays URL-backed

**Done**: [ ]

**Scope**: Exercise signed-in browser flows for issue/PR search tabs, type switching, sort and facet links, pagination preservation, empty states, and result detail navigation.

**Key changes**:
- `web/tests/e2e/search-collaboration.spec.ts`: seed searchable issues/PRs, visit both tabs, add/remove filters, change sort, paginate, open issue/PR detail links, and save screenshots.
- Ensure every control is a concrete link/button or explicitly disabled with accessible text.
- Verify no horizontal overflow on desktop and mobile viewports.

**Verification**: focused Playwright smoke, `make test-e2e` when stable, and screenshots under `ralph/screenshots/build/`.

---

## Phase 5: Search-004 Guardrails and QA Handoff - finish issue/PR search

**Done**: [ ]

**Scope**: Harden accessibility, privacy, responsive behavior, performance bounds, visual compliance, and bookkeeping. Mark `search-004.build_pass=true` only after all phases pass.

**Key changes**:
- Final Rust coverage for private repository redaction, invalid query envelopes, pagination limits, sort stability, and absence of stack/secret leakage.
- Final frontend no-dead-control, keyboard, responsive, and Editorial-token tests.
- Mandatory Editorial banned-value scan: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- Update `build-progress.txt`, `qa-hints.json`, and `prd.json` with evidence; leave `qa_pass=false`.

**Verification**: `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make test && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session make test-e2e`; mandatory Editorial banned-value scan returns zero matches.
