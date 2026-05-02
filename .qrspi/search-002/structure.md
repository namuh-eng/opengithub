# Structure Outline: search-002 Global Search Modal, Suggestions, and Saved Searches

**Ticket**: `search-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og-shell.jsx`, `design/project/wf-search.jsx`, `.qrspi/search-001/structure.md`, current `crates/api/src/domain/search.rs`, current `crates/api/src/routes/search.rs`, current `web/src/components/AppHeader.tsx`, current `web/src/components/SearchResultsPage.tsx`, and current `web/src/lib/navigation.ts`.
**Date**: 2026-05-02

## Phase 1: Suggestion and Saved-Search API Contract - modal data is real and permission-aware

**Done**: [x]

**Scope**: Add the backend contract that powers the global search modal without changing the existing `/api/search` result contract. The endpoint should return categorized suggestions for repositories, organizations, users, teams, code/path jumps, query qualifier completions, recent searches, and saved searches. Private repository/code suggestions must only appear for viewers with access.

**Key changes**:
- `crates/api/migrations/`: add `saved_searches`, `recent_searches`, and optional `search_telemetry_events` tables with user ownership, uniqueness, indexes for recency/query lookup, and delete-safe foreign keys.
- `crates/api/src/domain/search.rs`: add a `SearchSuggestionDashboard` contract plus helpers for token parsing, qualifier detection (`repo:`, `org:`, `user:`, `language:`, `path:`, `symbol:`, `is:`, `state:`), permission-filtered repository/code/user/org suggestions, recent-search recording, and saved-search listing.
- `crates/api/src/routes/search.rs`: add `GET /api/search/suggestions?q=...&scope=...` and keep errors in the standard envelope.
- `crates/api/tests/search_suggestions_contract.rs`: cover empty-query defaults, typed-query suggestions, qualifier completions, direct repository/code jumps, saved-search rows, private suggestion filtering, pagination/limit clamps, and no leakage of private names to outsiders.

**Verification**: focused `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false cargo test --test search_suggestions_contract -- --nocapture`, then `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make test`. Browser smoke is optional for this API-only phase.

---

## Phase 2: Editorial Command Modal - header search opens the full search experience

**Done**: [ ]

**Scope**: Replace the desktop header's small suggestion popover with a full Editorial command-style search modal while preserving the existing mobile drawer and basic submit behavior. Clicking the header search or pressing the global shortcut opens the modal, focuses the combobox, and renders API-backed categorized suggestions.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed `getSearchSuggestionsFromCookie` helpers and DTOs matching the Phase 1 contract.
- `web/src/components/AppHeader.tsx`: wire the search trigger and keyboard shortcut to a modal component instead of a local-only popover, while retaining `/search?q=...&type=repositories` submit fallback.
- `web/src/components/GlobalSearchModal.tsx`: render the Editorial modal with Search heading, combobox, syntax tips link, feedback button, grouped suggestion rows, keyboard hints, Escape close, outside-click close, loading/error states, and no Copilot chat rows.
- `web/tests/app-shell.test.tsx` and `web/tests/e2e/app-shell.spec.ts`: cover opening from click and shortcut, focus management, Escape close, API-backed suggestion rendering, submit fallback, no inert links/buttons, and desktop/mobile layout stability.

**Verification**: focused `cd web && npx vitest run tests/app-shell.test.tsx`, focused Playwright app-shell smoke with a saved screenshot at `ralph/screenshots/build/search-002-phase2-command-modal.jpg`, then full `make check`, `make test`, and `make test-e2e` with the repo's TEST_DATABASE_URL environment.

---

## Phase 3: Autocomplete, Query Builder, and Direct Jumps - modal interactions mutate query or navigate

**Done**: [ ]

**Scope**: Make modal suggestions interactive: typed input updates suggestions in place, qualifier rows autocomplete the current token, scoped-search rows submit to `/search`, and direct jump rows navigate to their concrete target. The query-builder surface should let users add common qualifiers without losing the typed query.

**Key changes**:
- `crates/api/src/domain/search.rs`: enrich suggestion rows with action metadata (`navigate`, `submit_search`, `replace_token`, `open_saved_search_dialog`) and normalized next-query values for qualifier completions.
- `web/src/components/GlobalSearchModal.tsx`: add highlighted row navigation, Enter/Arrow/Home/End handling, token replacement, query-builder chips, scope rows for all repositories/current repo/org/all opengithub, and URL-safe submit helpers.
- `web/src/lib/navigation.ts`: add typed builders for search modal actions and qualifier-preserving `/search` URLs.
- Tests: assert qualifier replacement (`language:` to `language:rust`), repository/code/symbol jumps, scoped searches, selected-row keyboard behavior, empty suggestions, and malformed action guardrails.

**Verification**: focused Rust suggestion contract, focused GlobalSearchModal Vitest coverage, focused Playwright smoke saving `ralph/screenshots/build/search-002-phase3-autocomplete-jumps.jpg`, then full `make check`, `make test`, and `make test-e2e`.

---

## Phase 4: Saved-Search Creation and Management - modal writes persist and validate

**Done**: [ ]

**Scope**: Implement the `Create saved search` flow from the modal. Required Name and Query fields validate client-side and server-side, successful creation persists for the viewer, duplicate names return an inline error, and saved-search rows can be opened from the modal.

**Key changes**:
- `crates/api/src/domain/search.rs`: add saved-search create/list/delete helpers with ownership checks, normalized query validation, duplicate-name handling, and recent-search write-through on submit.
- `crates/api/src/routes/search.rs`: add `POST /api/search/saved-searches` and `DELETE /api/search/saved-searches/{id}`; optionally add `POST /api/search/recent` if submit telemetry should be explicit instead of coupled to `/api/search`.
- `web/src/components/GlobalSearchModal.tsx`: add the nested `Create saved search` dialog with required Name and Query inputs, documentation link, Cancel, Create saved search, pending/success/error feedback, and refreshed saved-search rows.
- `web/src/lib/api-docs.ts`: document suggestion and saved-search endpoints with request/response examples.
- Tests: cover successful create, validation errors, duplicate names, unauthorized access, saved-search row navigation, dialog focus return, and no saved-search writes for anonymous users.

**Verification**: focused saved-search Rust contract, focused modal/API-doc Vitest coverage, focused Playwright smoke saving `ralph/screenshots/build/search-002-phase4-saved-search.jpg`, then full `make check`, `make test`, and `make test-e2e`.

---

## Phase 5: Search Modal Guardrails and QA Handoff - finish search-002

**Done**: [ ]

**Scope**: Harden the full global search modal feature across accessibility, responsive behavior, privacy boundaries, visual compliance, and bookkeeping. Mark `search-002.build_pass=true` only after every modal interaction is real and verified.

**Key changes**:
- `crates/api/tests/search_suggestions_contract.rs`: final coverage for permission boundaries, query bounds, qualifier parsing edge cases, saved-search ownership/deletion, recent-search ordering, and redaction of private metadata.
- `web/tests/global-search-modal.test.tsx`: final accessibility, focus trap, keyboard, no-dead-control, validation, and mobile no-overflow assertions.
- `web/tests/e2e/search-modal.spec.ts`: signed-in browser smoke for shortcut open, typed suggestions, direct repo/code jump, qualifier autocomplete, scoped submit to `/search`, saved-search create/open, Escape close, and mobile viewport layout.
- `ralph/screenshots/build/`: save final desktop and mobile screenshots.
- Mandatory Editorial banned-value scan before commit: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`, `build-progress.txt`, and `prd.json`: record verification evidence and set `search-002.build_pass=true` only after the final phase passes; leave `qa_pass=false`.

**Verification**: `TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make check && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false make test && TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DATABASE_URL=postgresql://postgres:postgres@localhost:5432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session make test-e2e`; browser smoke proves every modal button, row, form, shortcut, and empty state has a concrete action; mandatory Editorial banned-value scan returns zero matches.
