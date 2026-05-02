# Structure Outline: profiles-002 Profile Repository and Star Tabs

**Ticket**: `profiles-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/wf-profiles.jsx`, `design/project/og-screens-2.jsx`, current `web/src/components/UserProfilePage.tsx`, current `web/src/lib/api.ts`, current `crates/api/src/domain/profiles.rs`, current `crates/api/src/routes/users.rs`, existing repository metadata/star tables, and `target-docs/`.
**Date**: 2026-05-02

## Phase 1: Profile Repository List API Contract - owned repositories, filters, sort, and tab counts

**Done**: [x]

**Scope**: Add a real Rust/Postgres read contract for `/{user}?tab=repositories`. The endpoint must resolve usernames case-insensitively, honor public/private repository visibility for anonymous and signed-in viewers, support URL-backed query parameters, and return dense repository rows with badges and metadata.

**Key changes**:
- `crates/api/src/domain/profiles.rs`: introduce `ProfileRepositoryList`, `ProfileRepositoryListItem`, `ProfileRepositoryFilters`, `ProfileRepositoryBadgeState`, language/type/sort enums, pagination metadata, and helpers for owner lookup, viewer permission filtering, type filtering, language aggregation, search text matching, and deterministic sort.
- `crates/api/src/routes/users.rs`: add `GET /api/users/{username}/repositories` with query params `q`, `type`, `language`, `sort`, `page`, and `pageSize`; return the standard list envelope plus `filters`, `availableLanguages`, `availableTypes`, and `tabCounts`.
- Reuse existing `repositories`, `repository_permissions`, `repository_languages`, `repository_stars`, `repository_forks`, issue/PR count sources, license metadata, and fork/source fields where available. Add only additive migration fields if a required badge/metadata value is missing.
- `crates/api/tests/profile_repositories_contract.rs`: seed source, fork, archived, mirror/template, public/private, and multi-language repositories; verify permission boundaries, search matching, type/language/sort behavior, pagination clamps, counts, and error envelopes.
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed DTOs and server fetch helpers for repository-tab data without changing the overview contract.

**Verification**: focused `TEST_DATABASE_URL=postgresql://namuh@localhost:55432/opengithub_identity_test DB_SSL=false cargo test --test profile_repositories_contract -- --nocapture`, then same-env `make check` and `make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Repository Tab Shell - render owned repositories with dense result rows

**Done**: [x]

**Scope**: Replace the current secondary-tab placeholder for `tab=repositories` with a full Editorial repository results surface while preserving the profile identity column and tabs from `profiles-001`. The UI must use `og.css` tokens and primitives, not GitHub Primer colors or chrome.

**Key changes**:
- `web/src/app/[owner]/page.tsx`: when `tab=repositories`, fetch the new repository list with `searchParams`, pass data into the profile page, and preserve anonymous public access.
- `web/src/components/UserProfilePage.tsx`: route the selected tab to a concrete repository-tab component instead of `SecondaryTab`.
- `web/src/components/ProfileRepositoryTabs.tsx`: add a server/client split as needed for the repository tab shell, including filter row, result count summary, active filter chips, repository row component, language dots using data-provided colors only when safe, and concrete repository links to the Code tab.
- Repository rows show name, visibility, archived/fork/template/mirror badges, fork source when present, description, primary language, star count, fork count, license, issue/PR counts if available, and updated date with stable mobile wrapping.
- `web/tests/user-profile-repositories.test.tsx`: assert repository row rendering, badges, metadata, empty state, filter form values, concrete links, no `href="#"`, and Editorial primitive/token usage.

**Verification**: focused Vitest for repository tab rendering, mandatory Editorial banned-value scan, then `make check` and `make test`. Save a browser screenshot if the seeded profile E2E fixture is already stable.

---

## Phase 3: URL-Backed Repository Filters - search, type, language, sort, clear, and recent visits

**Done**: [ ]

**Scope**: Make every repository-tab control concrete. Typing search terms, selecting Type, selecting Language, selecting Sort, and clearing filters must update URL state, refetch data, and never rely on dead handlers. Opening a repository row may write a recent visit if an authenticated viewer is present.

**Key changes**:
- `web/src/components/ProfileRepositoryTabs.tsx`: implement search submission, type/language/sort controls, active chips with remove links, clear filters CTA, loading/disabled states, keyboard focus behavior, and mobile-safe control wrapping.
- `web/src/lib/navigation.ts`: add helpers for profile repository tab query construction/removal so filters preserve owner and tab context.
- Add or reuse a same-origin route/API helper for authenticated recent-visit writes when a repository row is opened; keep the row link navigation concrete even if recent-visit write fails.
- Extend `crates/api/tests/profile_repositories_contract.rs` for query-state combinations, unsupported filter validation, and recent-visit write behavior if a new endpoint is needed.
- Extend Vitest and Playwright coverage for search input, type/language/sort changes, chip removal, clear filters, row navigation, and no dead controls.

**Verification**: focused Rust tests if backend changed, focused Vitest, focused Playwright profile repository smoke saving `ralph/screenshots/build/profiles-002-phase3-repository-filters.jpg`, then `make check`, `make test`, and `make test-e2e` if the local test database is reachable.

---

## Phase 4: Starred Repositories Tab - starred-list API, stars-specific sort, and shared row UI

**Done**: [ ]

**Scope**: Add `/{user}?tab=stars` using the same profile identity shell and repository row component, but source rows from repositories the user starred. The Stars tab supports search, language, and sort options `recently-starred`, `recently-active`, and `most-stars`.

**Key changes**:
- `crates/api/src/domain/profiles.rs`: add `starred_repositories` list helper that joins `repository_stars`, repository metadata, owner data, language summaries, and permission checks. Include `starredAt` on rows and keep private starred repositories hidden unless viewer permissions allow them.
- `crates/api/src/routes/users.rs`: add `GET /api/users/{username}/stars` with `q`, `language`, `sort`, `page`, and `pageSize` query params; return the same list shape where possible with `mode: "stars"` and stars-specific sort metadata.
- `web/src/lib/api.ts`: add typed starred repository helpers while sharing repository-list row types when practical.
- `ProfileRepositoryTabs`: support `mode="stars"`, stars-specific copy and sort menu, `starredAt` metadata, clear filters, empty state, and shared row rendering.
- `crates/api/tests/profile_starred_repositories_contract.rs`, `web/tests/user-profile-repositories.test.tsx`, and Playwright coverage: verify recent-starred/recent-active/most-stars ordering, private redaction, language/search filters, row navigation, and empty-state clear CTA.

**Verification**: focused Rust starred-repositories contract, focused Vitest, focused Playwright stars smoke saving `ralph/screenshots/build/profiles-002-phase4-stars.jpg`, then `make check`, `make test`, and `make test-e2e`.

---

## Phase 5: Profile Repository Tabs Guardrails and QA Handoff - finish profiles-002

**Done**: [ ]

**Scope**: Harden privacy, accessibility, URL-state behavior, responsive layout, visual consistency, and bookkeeping. Mark `profiles-002.build_pass=true` only after repository and star tabs are backed by real APIs, filters are URL-backed, rows navigate to concrete repository pages, empty states have working clear actions, and no visible controls are dead.

**Key changes**:
- Rust contracts: final coverage for missing users, private profiles, private repositories, viewer permissions, type/language/sort validation, pagination bounds, tab counts, fork/source metadata, archived/template/mirror badges, starred timestamp ordering, and no stack trace/secret leakage in errors.
- Frontend unit tests: final coverage for repository and stars tabs, filter-row accessibility, active chips, no inert anchors/buttons, long names/descriptions, mobile wrapping, no horizontal overflow-prone fixed widths, and private-profile suppression.
- Playwright: signed-in and anonymous desktop/mobile smoke for `/{user}?tab=repositories` and `/{user}?tab=stars`; exercise search, type, language, sort, clear filters, empty state, row navigation, and screenshots.
- `qa-hints.json`: record deeper QA targets such as fork/source edge cases, repository permission visibility, very large repository counts, and recent-visit write failures.
- `build-progress.txt`, `.qrspi/profiles-002/structure.md`, and `prd.json`: record verification evidence, mark all phases done, and set `profiles-002.build_pass=true`; leave `qa_pass=false`.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.

**Verification**: `TEST_DATABASE_URL=postgresql://namuh@localhost:55432/opengithub_identity_test DB_SSL=false make check`, same-env `make test`, full same-env `make test-e2e` when the local test database is reachable, focused browser smoke screenshots under `ralph/screenshots/build/`, and mandatory Editorial banned-value scan with zero matches.
