# Structure Outline: orgs-002 Organization Repositories and People

**Ticket**: `orgs-002`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `design/project/Prototype.html`, `design/project/og.css`, `design/project/wf-profiles.jsx`, `.qrspi/orgs-001/structure.md`, current `crates/api/src/domain/organizations.rs`, current `web/src/components/OrganizationProfilePage.tsx`, and the profile repository-tab patterns in `crates/api/src/domain/profiles.rs` / `web/src/components/ProfileRepositoryTabs.tsx`.
**Date**: 2026-05-03

## Phase 1: Organization Repository List API Contract - filters, density state, pagination, and visibility

**Done**: [x]

**Scope**: Add the Rust read contract for organization-owned repositories so `/api/orgs/{org}/repositories` can serve the repositories tab and `/orgs/{org}/repositories` page from real Postgres data. The contract must preserve org visibility rules from `orgs-001`, expose URL-backed filter state, and return a standard list envelope with repository rows, available filter counts, tab counts, and viewer state. This phase writes no org data; repository open/visit writes stay with existing repository page behavior.

**Key changes**:
- `crates/api/src/domain/organizations.rs`: add `OrganizationRepositoryList`, `OrganizationRepositoryListItem`, `OrganizationRepositoryFilters`, `OrganizationRepositoryFilterOption`, and `OrganizationRepositoryListQuery` DTOs or reuse compatible profile repository DTOs only if they keep org-specific filter names clear.
- `crates/api/src/domain/organizations.rs`: add `organization_repositories(pool, org_slug, viewer_user_id, query)` with case-insensitive org lookup, private-org 404 behavior, member/admin visibility, repository permission checks, bounded pagination, `q` search over name/description/topic, `language`, `sort`, `density`, and repository type filters.
- Supported repository type filters: `all`, `contributed`, `admin`, `public`, `sources`, `forks`, `archived`, and `templates`. `contributed` and `admin` should degrade to zero-count filters for anonymous users rather than leaking private membership.
- `crates/api/src/routes/organizations.rs`: add `GET /api/orgs/:org/repositories` with optional signed-cookie viewer detection and standard error envelopes for invalid filters, not-found/private orgs, and database failures.
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed org repository list DTOs and server fetch helpers for the later UI phase.
- `crates/api/tests/organization_repositories_contract.rs`: cover public list reads, private/internal redaction, member-visible repositories, search/language/type/sort filters, filter-count correctness, pagination clamps, invalid filter envelopes, and no leakage of stack traces/secrets.

**Verification**: focused `organization_repositories_contract` against `TEST_DATABASE_URL`, then same-env `make check && make test`. Browser smoke is optional because this phase is API-first.

---

## Phase 2: Editorial Organization Repository Tab Shell - render the real list

**Done**: [x]

**Scope**: Replace the placeholder `Repositories for {org}` secondary tab with a real Editorial repository list when `tab=repositories`, and add the canonical `/orgs/{org}/repositories` route. The UI should use the org header/tabs from `OrganizationProfilePage`, fetch the Phase 1 repository contract, and render repository rows with the same data density expected by the PRD.

**Key changes**:
- `web/src/app/orgs/[org]/page.tsx`: when `tab=repositories`, fetch the org profile plus `getOrganizationRepositories` with current search params and render the real repository tab instead of `SecondaryTab`.
- `web/src/app/orgs/[org]/repositories/page.tsx`: add a canonical repository-list route that renders the same shell and preserves incoming `q`, `type`, `language`, `sort`, `density`, and `page` params.
- `web/src/components/OrganizationRepositoriesPage.tsx` or a narrowly named subcomponent: render search input, type/language/sort controls, All/Contributed by me/Admin access/Public/Sources/Forks/Archived/Templates chips, comfortable/compact icon buttons, active filter chips, pagination summary, and repository rows with name, visibility, description, topics, language/license, fork/star/issue/PR counts, updated date, and concrete row links.
- `web/src/lib/navigation.ts`: add org repository-list URL helpers for tab and canonical routes, including active-filter chip removal and density toggles.
- `web/tests/organization-repositories-page.test.tsx`: cover Editorial header/tabs, row content, concrete links, display density state, empty state, no inert anchors/buttons, and active filter chip hrefs.
- `web/tests/e2e/organization-repositories.spec.ts`: focused browser smoke for the seeded org repository list, row navigation, tab/canonical route parity, and screenshot `ralph/screenshots/build/orgs-002-phase2-repositories-shell.jpg`.

**Verification**: focused Vitest and focused Playwright smoke, then `make check && make test`. Run full `make test-e2e` if the local test database is available.

---

## Phase 3: URL-Backed Repository Filters and Pagination - make every control concrete

**Done**: [x]

**Scope**: Finish repository-list interactions so filters compose predictably and every control mutates URL state without client-only placeholders. Search, type chips, language, sort, density, and Previous/Next pagination must preserve each other. Compact density changes row spacing only, not data or API visibility.

**Key changes**:
- Backend: harden query normalization for empty search, oversized query, unsupported type, unsupported sort, invalid page/pageSize, and language names with unusual casing.
- Backend: add deterministic sorting for `updated-desc`, `name-asc`, `stars-desc`, and any org-specific sort needed by the UI; ensure filter counts are computed from the visible repository set and not the current page only.
- Frontend: make filter chips, select controls, search submit, density icon buttons, clear filters, and pagination links use the same navigation helper and preserve current state correctly.
- Frontend: ensure compact/comfortable row layouts have stable dimensions, no text overlap, and no GitHub palette values.
- `crates/api/tests/organization_repositories_contract.rs`: extend for composed filters, pagination preservation, admin/contributed visibility, count consistency, and invalid-query redaction.
- `web/tests/organization-repositories-page.test.tsx` and `web/tests/e2e/organization-repositories.spec.ts`: extend for combined filters, chip removal, density toggles, pagination, no dead controls, row navigation, and desktop/mobile no-overflow screenshots.

**Verification**: focused Rust contract, focused Vitest, focused Playwright, then `make check && make test`. Run full `make test-e2e` when database state is healthy.

---

## Phase 4: Organization People List API and Editorial People Page - public members and pagination

**Done**: [ ]

**Scope**: Add a real people list for `/api/orgs/{org}/people`, `/orgs/{org}?tab=people`, and `/orgs/{org}/people`. Signed-out viewers see public members only when the organization allows public members; members/admins can see internal membership rows. This phase is read-only and defers pending invitations, outside collaborators, role edits, and admin management actions to later settings/admin features.

**Key changes**:
- `crates/api/src/domain/organizations.rs`: add `OrganizationPeopleList`, `OrganizationPeopleListItem`, `OrganizationPeopleFilters`, and `organization_people(pool, org_slug, viewer_user_id, query)` with bounded pagination, search over login/display name, public-member visibility rules, viewer role state, and stable role ordering.
- `crates/api/src/routes/organizations.rs`: add `GET /api/orgs/:org/people` with optional signed-cookie viewer detection and standard envelopes.
- `web/src/lib/api.ts`, `web/src/lib/server-session.ts`, and `web/src/lib/navigation.ts`: add typed people list helpers and URL builders for people search/page preservation.
- `web/src/components/OrganizationPeoplePage.tsx`: render org header/tabs, a left `Organization permissions` side nav, Members subtab, search input, member rows with avatar/name/username, role chips only when visible, concrete profile links, and Previous/Next pagination.
- `web/src/app/orgs/[org]/page.tsx` and `web/src/app/orgs/[org]/people/page.tsx`: render the real people page for `tab=people` and canonical people route.
- `crates/api/tests/organization_people_contract.rs`, `web/tests/organization-people-page.test.tsx`, and `web/tests/e2e/organization-people.spec.ts`: cover public-member visibility, member/admin role visibility, private org 404 behavior, pagination/search preservation, concrete profile links, side nav links, and screenshots.

**Verification**: focused people Rust contract, focused Vitest, focused Playwright smoke, then `make check && make test`; run `make test-e2e` if local DB and dev servers are stable.

---

## Phase 5: Final Privacy, Visual, and QA Guardrails - finish orgs-002

**Done**: [ ]

**Scope**: Lock the organization repository and people surfaces against regressions and mark `orgs-002.build_pass=true` only after API contracts, Editorial UI, URL-backed interactions, visibility rules, browser smoke, and QA handoff all pass. This phase should not add new product scope.

**Key changes**:
- Rust tests: final anonymous, signed-in non-member, member, admin/owner, private org, private repository, internal repository, public-member-hidden, and invalid query coverage for both repository and people endpoints.
- Frontend tests: final accessibility, keyboard focus, no-dead-control, responsive truncation/no-overflow, empty-state, pagination, density, and active-filter coverage.
- E2E: desktop and mobile browser smoke for `/orgs/{org}?tab=repositories`, `/orgs/{org}/repositories`, `/orgs/{org}?tab=people`, and `/orgs/{org}/people`; save final screenshots under `ralph/screenshots/build/`.
- `web/src/lib/api-docs.ts`: document `GET /api/orgs/{org}/repositories` and `GET /api/orgs/{org}/people` with auth/visibility notes and response examples.
- Mandatory Editorial banned-value scan before commit: `rg -nE '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
- `qa-hints.json`, `build-progress.txt`, and `prd.json`: record verification evidence and set `orgs-002.build_pass=true` only after the final phase passes; leave `qa_pass=false`.

**Verification**: `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make check && TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false make test && TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false SESSION_SECRET=playwright-session-secret-with-enough-entropy SESSION_COOKIE_NAME=og_session make test-e2e`; browser smoke proves every visible organization repository/people control has concrete behavior; mandatory Editorial banned-value scan returns zero matches.
