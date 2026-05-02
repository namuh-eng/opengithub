# Structure Outline: search-003 Code Search Results, Facets, and Expansion

**Ticket**: `search-003`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/wf-search.jsx`, `.qrspi/search-001/structure.md`, `.qrspi/search-002/structure.md`, current `crates/api/src/domain/search.rs`, current `crates/api/src/routes/search.rs`, current `web/src/components/SearchResultsPage.tsx`, current `web/src/app/search/page.tsx`, and target search docs under `target-docs/content/rest/search/search.md`.
**Date**: 2026-05-02

## Phase 1: Code Search API Facets - results include counts, facets, chips, and bounded query errors

**Done**: [x]

**Scope**: Extend the existing permission-aware search contract for `type=code` without breaking other result types. The API should return type counts, language/path facets, active query chips, timing metadata, and structured validation errors for invalid qualifiers or oversized queries.

**Key changes**:
- `crates/api/src/domain/search.rs`: add a code-search response contract that wraps the current `ListEnvelope<SearchResult>` with `type_counts`, `facets`, `active_chips`, `query_duration_ms`, and parser diagnostics for qualifiers such as `repo:`, `org:`, `user:`, `language:`, `path:`, `symbol:`, `is:`, and `archived:`.
- `crates/api/src/routes/search.rs`: preserve `/api/search?q=...&type=code` compatibility while allowing the richer code response for the page, including page/per-page clamps and standard error envelopes.
- `crates/api/tests/search_code_results_contract.rs`: seed public and private indexed code documents and verify permission filtering, count/facet accuracy, active-chip parsing, invalid qualifier errors, query length caps, and deterministic ranking.
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed helpers for the richer code-search contract while leaving existing global search DTOs usable by other tabs.

**Verification**: focused `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false cargo test --test search_code_results_contract -- --nocapture`, then `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Code Results Shell - `/search?type=code` becomes the two-pane code workspace

**Done**: [x]

**Scope**: Replace the generic code tab rendering with the dedicated Editorial two-pane code search layout from `wf-search.jsx`: left filter rail, main code-result pane, header counts, query chips, Save button, type menu, and view controls. All controls must be real links/buttons with concrete behavior or disabled with clear state.

**Key changes**:
- `web/src/components/SearchResultsPage.tsx`: branch `type=code` to a dedicated code-search layout while preserving the existing shared shell for repositories, users, organizations, commits, issues, pull requests, and discussions.
- `web/src/components/CodeSearchResultsPage.tsx`: new component for the code search workspace with result type counts, language/path facet links, active chips with removable qualifier links, advanced owner/symbol/archive controls, Save search entry point, and dense grouped file-result cards.
- `web/src/lib/navigation.ts`: add URL builders for adding/removing qualifiers, switching result types, preserving page/sort/view state, and linking line anchors into repository blob pages.
- `web/tests/search-code-results.test.tsx`: cover the code workspace structure, facet/chip URLs, type-switch links, disabled states, and no dead controls.

**Verification**: focused `cd web && npx vitest run tests/search-code-results.test.tsx`, then full `make check` and `make test` with the repo's TEST_DATABASE_URL environment. Save a browser screenshot only if the phase adds enough UI for a meaningful smoke.

---

## Phase 3: Snippet Groups and Expansion - file cards expose line anchors and hidden matches

**Done**: [ ]

**Scope**: Make individual code results behave like grouped file matches. Each card should show repository/path/language context, line-number anchors, highlighted query terms, collapse/expand state, and a working "Show N more matches" action that reveals additional snippets without losing the URL-backed search state.

**Key changes**:
- `crates/api/src/domain/search.rs`: enrich code result snippets with grouped match lines, total match count per file, hidden-match counts, and stable blob hrefs with `#L{line}` anchors.
- `web/src/components/CodeSearchResultsPage.tsx`: render grouped snippet tables with mono line numbers, highlighted fragments, collapse buttons, "Show N more matches", and accessible file/path links.
- `web/tests/search-code-results.test.tsx`: assert highlighted fragments, line-anchor hrefs, collapse behavior, expansion behavior, keyboard reachability, and no layout shift from expanded snippets.
- `web/tests/e2e/search-code.spec.ts`: signed-in smoke for searching a seeded code marker, opening a line anchor, expanding hidden matches, and returning to search state.

**Verification**: focused Rust contract and Vitest coverage, focused Playwright smoke saving `ralph/screenshots/build/search-003-phase3-snippet-expansion.jpg`, then `make check`, `make test`, and `make test-e2e`.

---

## Phase 4: Facet Interactions, Advanced Controls, and Saved Search - filters mutate URL-backed results

**Done**: [ ]

**Scope**: Complete the interactive controls around code results. Clicking language/path facets adds qualifiers and refreshes results, deleting chips removes qualifiers, advanced owner/symbol/exclude-archived controls update the URL, type menu preserves the query, view controls are stateful, and Save uses the `search-002` saved-search API instead of a placeholder.

**Key changes**:
- `web/src/components/CodeSearchResultsPage.tsx`: add working facet links, removable chips, advanced filter form, sort/view controls, Save search dialog integration, inline success/error feedback, and empty/error states that preserve the typed query.
- `web/src/app/search/page.tsx`: parse and forward sort/view/advanced query params needed by the code workspace.
- `crates/api/src/domain/search.rs`: apply supported code-search qualifiers to prepared SQL and return validation errors for unsupported regex or malformed qualifiers without leaking private metadata.
- `web/tests/e2e/search-code.spec.ts`: cover language/path facet clicks, chip removal, advanced owner/symbol/archive controls, type switching, saved-search creation/opening, invalid qualifier recovery, and pagination preservation.

**Verification**: focused Rust code-search contract, focused Vitest, focused Playwright smoke saving `ralph/screenshots/build/search-003-phase4-facets-saved-search.jpg`, then full `make check`, `make test`, and `make test-e2e`.

---

## Phase 5: Code Search Guardrails and QA Handoff - finish search-003

**Done**: [ ]

**Scope**: Harden accessibility, privacy, responsive behavior, performance bounds, visual compliance, and bookkeeping. Mark `search-003.build_pass=true` only after code search has real API data, working filters, real result links, and verified no-dead-control coverage.

**Key changes**:
- `crates/api/tests/search_code_results_contract.rs`: final coverage for private repository/code redaction, default-branch-only indexed results, file-size/search-term constraints, facet count consistency, invalid qualifier envelopes, pagination limits, and absence of stack/secret leakage.
- `web/tests/search-code-results.test.tsx`: final accessibility, keyboard, responsive no-overflow, no `href="#"`, no empty button handler, disabled-state, and Editorial token guardrails.
- `web/tests/e2e/search-code.spec.ts`: desktop and mobile signed-in smoke for code result search, type counts, facet add/remove, expansion/collapse, line-anchor navigation, saved-search creation, invalid query error, and empty state CTA behavior.
- `ralph/screenshots/build/`: save final desktop and mobile screenshots for the code search page.
- Mandatory Editorial banned-value scan before commit: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`, `build-progress.txt`, and `prd.json`: record verification evidence and set `search-003.build_pass=true` only after the final phase passes; leave `qa_pass=false`.

**Verification**: `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make test && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session make test-e2e`; browser smoke proves every code-search button, link, facet, chip, expansion, form, empty state, and error state has a concrete action; mandatory Editorial banned-value scan returns zero matches.
