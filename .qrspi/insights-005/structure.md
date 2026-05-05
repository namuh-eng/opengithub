# Structure Outline: insights-005 Repository Dependency Graph

**Ticket**: `insights-005`
**Design**: `build-spec.md`, `BUILD_GUIDE.md`, `prd.json`, `web/AGENTS.md`, `target-docs/content/code-security/concepts/supply-chain-security/about-the-dependency-graph.md`, `target-docs/content/code-security/concepts/supply-chain-security/dependency-graph-data.md`, `target-docs/content/code-security/how-tos/secure-your-supply-chain/secure-your-dependencies/exploring-the-dependencies-of-a-repository.md`, existing `RepositoryInsightsShell`, existing Network/Forks Insights patterns from `insights-004`, repository contents contracts from `repo-005`, package registry contracts from `packages-001`, and security feature settings from `settings-006`.
**Date**: 2026-05-05

## Phase 1: Dependencies API Contract and Extraction Storage - screen-ready dependency rows

**Done**: [x]

**Scope**: Add the repository-owned dependency graph read model and default Dependencies API without rendering the page yet. `GET /api/repos/{owner}/{repo}/network/dependencies` should return supported manifest/lockfile metadata, package rows, direct/transitive relationship state, ecosystem counts, advisory/detail hrefs, dependency-graph availability, and export affordance metadata. Extraction should index existing repository file contents from the default branch for the first supported ecosystems instead of using mock rows.

**Key changes**:
- `crates/api/migrations/*_repository_dependency_graph.*.sql`: add `dependency_manifests`, `dependency_packages`, `repository_dependencies`, `dependency_advisories`, `repository_dependents`, `sbom_exports`, and narrow audit/job fields needed for extraction/export; include repository, package, ecosystem, manifest path, lockfile path, relationship, license, detected-at, and artifact status indexes.
- `crates/api/src/domain/repositories.rs`: add `RepositoryDependenciesView`, `RepositoryDependencyRow`, `RepositoryDependencyManifest`, `RepositoryDependencyFilters`, `RepositoryDependencyPackage`, `RepositoryDependencyExportState`, and `RepositoryDependencyGraphAvailability` DTOs.
- `crates/api/src/domain/repositories.rs`: add `repository_dependencies_for_actor_by_owner_name(pool, actor_user_id, owner, repo, filters)` and `extract_repository_dependencies(pool, repository_id)` with deterministic parsers for `package.json`/`package-lock.json`, `Cargo.toml`/`Cargo.lock`, and `requirements.txt`; unsupported files are omitted with availability metadata.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/network/dependencies`, validate `q`, `ecosystem`, and `relationship` filters, enforce repository read permission, return structured `422 dependency_graph_unavailable` for disabled/unavailable states, and avoid leaking private repository paths.
- `crates/api/tests/repository_dependency_graph_contract.rs`: seed default-branch manifests/lockfiles and assert extraction, direct/transitive rows, ecosystem counts, search/filtering, advisory/detail href shape, disabled-state 422, private repository privacy, invalid filters, and no secret leakage.

**Verification**: focused Rust contract tests against `opengithub_identity_test`, then `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check` and `DB_SSL=false CARGO_INCREMENTAL=0 make test`. Browser smoke is optional for this API-first phase.

---

## Phase 2: Editorial Dependencies Page and Query Filters - default dependency graph UI

**Done**: [x]

**Scope**: Implement `/{owner}/{repo}/network/dependencies` using the Editorial Insights shell with Dependency graph selected and Dependencies tab active. The page should render the search query-builder input, ecosystem select-panel filter, total count, dependency rows, relationship chips, manifest/lockfile links, license copy, row options menu, empty state, and unavailable state from the Phase 1 API.

**Key changes**:
- `web/src/lib/api.ts` and `web/src/lib/server-session.ts`: add typed dependency graph DTOs plus signed-cookie fetch helper for the dependencies endpoint.
- `web/src/app/[owner]/[repo]/network/dependencies/page.tsx`: fetch repository and dependency data server-side, preserve query params, and render unavailable/empty states inside `RepositoryInsightsShell`.
- `web/src/components/RepositoryDependencyGraphPage.tsx`: add Editorial tab header for Dependencies/Dependents, summary cards, row list, manifest/lockfile links, relationship/license/ecosystem chips, and no dead controls.
- `web/src/components/RepositoryDependencyFilters.tsx`: client controls for query input, ecosystem menu, relationship chips, URL-backed Apply/Clear actions, Escape/outside-click handling, and accessible selected-state affordances.
- `web/src/lib/navigation.ts`: add dependency/dependent href helpers that encode owners, repositories, package names, ecosystems, and manifest paths safely.
- `web/tests/repository-dependency-graph-page.test.tsx` and `web/tests/e2e/repository-dependency-graph.spec.ts`: cover default render, active Insights nav state, tabs, query filtering, ecosystem filtering, row menu destinations, empty/unavailable states, no `href="#"`, no inert handlers, no unsafe HTML, no banned GitHub values/imports, and a saved desktop screenshot.

**Verification**: focused Vitest and Playwright smoke for `/network/dependencies`, screenshot `ralph/screenshots/build/insights-005-phase2-dependencies.jpg`, then `make check && make test`.

---

## Phase 3: SBOM Export Job and Download Flow - real export artifact

**Done**: [x]

**Scope**: Implement the Export SBOM button as a real server action backed by Rust API storage. `POST /api/repos/{owner}/{repo}/network/dependencies/sbom` should create an export job, produce an SPDX-style JSON artifact from indexed dependency rows, audit the export, and return a signed download URL or pending status. The UI should show pending/success/error feedback and expose the download when ready.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: add `RepositorySbomExport`, `RepositorySbomExportStatus`, `start_repository_sbom_export(pool, actor_user_id, owner, repo)`, `repository_sbom_export_status(...)`, and `repository_sbom_download(...)`; write deterministic SPDX JSON from current dependency graph rows.
- `crates/api/src/routes/repositories.rs`: register `POST /api/repos/{owner}/{repo}/network/dependencies/sbom` and `GET /api/repos/{owner}/{repo}/network/dependencies/sbom/{export_id}` with auth, permission checks, audit events, artifact expiry, and standard error envelopes.
- `web/src/app/[owner]/[repo]/network/dependencies/sbom/route.ts`: proxy signed-cookie export requests from the browser to Rust without exposing session secrets.
- `web/src/components/RepositoryDependencyExportButton.tsx`: client export control with pending state, retry/error copy, signed download link, and no fake download behavior.
- `crates/api/tests/repository_dependency_graph_contract.rs`, `web/tests/repository-dependency-graph-page.test.tsx`, and Playwright: assert export creates an artifact from real rows, writes audit metadata, handles empty graphs truthfully, downloads valid JSON, and never leaks private package data.

**Verification**: focused Rust export tests, focused Vitest export tests, focused Playwright export/download smoke with screenshot `ralph/screenshots/build/insights-005-phase3-sbom-export.jpg`, then `make check && make test`.

---

## Phase 4: Dependents API and Page Filters - package and owner scoped usage view

**Done**: [ ]

**Scope**: Implement `/{owner}/{repo}/network/dependents` with package filtering, owner filtering, repository/package count summary, warning disclosure, and dependent repository rows. Public repository dependents should be visible from indexed package usage; private repository dependents should return a truthful unavailable/empty state without exposing private consumers.

**Key changes**:
- `crates/api/src/domain/repositories.rs`: add `RepositoryDependentsView`, `RepositoryDependentPackage`, `RepositoryDependentRow`, `RepositoryDependentsFilters`, and `repository_dependents_for_actor_by_owner_name(pool, actor_user_id, owner, repo, filters)` using `dependency_packages`, `repository_dependencies`, `repository_dependents`, repository visibility, stars, forks, and package metadata.
- `crates/api/src/routes/repositories.rs`: register `GET /api/repos/{owner}/{repo}/network/dependents`, validate `package` and `owner` filters, enforce public/private visibility, and return structured unavailable states for private or unindexed packages.
- `web/src/app/[owner]/[repo]/network/dependents/page.tsx`: fetch repository and dependents data server-side, keep Dependency graph selected, and preserve query state.
- `web/src/components/RepositoryDependentsPage.tsx` and `RepositoryDependentsFilters.tsx`: render Dependencies/Dependents tabs, package menu, owner username input, warning disclosure, dependent repository rows, package/repository count summary, concrete owner/repository/package links, empty states, and mobile wrapping.
- `web/tests/repository-dependents-page.test.tsx`, `web/tests/e2e/repository-dependency-graph.spec.ts`, and Rust contract tests: cover package filter switching, owner narrowing, public-only dependents, approximate count warning, long owner/repository names, no private leaks, no dead controls, and mobile no-overflow.

**Verification**: focused Rust dependents tests, focused Vitest, focused Playwright smoke with screenshot `ralph/screenshots/build/insights-005-phase4-dependents.jpg`, then `make check && make test`.

---

## Phase 5: API Docs, Edge Cases, Browser Evidence, QA Handoff, and Build Pass

**Done**: [ ]

**Scope**: Finish `insights-005` only after dependencies extraction, dependencies page, query filters, SBOM export, dependents page, package/owner filters, unavailable states, docs, screenshots, and QA handoff are verified. Mark `build_pass=true` only in this phase and leave `qa_pass=false`.

**Key changes**:
- `web/src/lib/api-docs.ts` and `/docs/api`: document `GET /api/repos/{owner}/{repo}/network/dependencies`, `POST /api/repos/{owner}/{repo}/network/dependencies/sbom`, `GET /api/repos/{owner}/{repo}/network/dependencies/sbom/{export_id}`, and `GET /api/repos/{owner}/{repo}/network/dependents`, including auth/privacy, supported ecosystems, filters, unavailable 422 states, SBOM artifact lifecycle, dependents public-only behavior, and error envelopes.
- `crates/api/tests/repository_dependency_graph_contract.rs`: add final coverage for anonymous/private/public access, disabled feature settings, unsupported manifests, malformed manifests, duplicate packages, multiple ecosystems, vulnerable/advisory rows, empty graphs, SBOM export, dependents privacy, invalid filters, and no secret leakage.
- `web/tests/repository-dependency-graph-page.test.tsx` and `web/tests/repository-dependents-page.test.tsx`: add final assertions for accessible names, keyboard focus order, menu controls, semantic chips, Editorial token usage, no banned GitHub colors/imports, no `href="#"`, no inert click handlers, and no unsafe HTML rendering.
- `web/tests/e2e/repository-dependency-graph.spec.ts`: full signed-session browser sweep for Insights navigation, Dependencies filters, row menus, SBOM export/download, Dependents filters, warning disclosure, empty/unavailable states, and mobile no-overflow.
- `qa-hints.json`, `build-progress.txt`, `.qrspi/insights-005/structure.md`, and `prd.json`: record verification evidence and set `insights-005.build_pass=true` only after every phase passes; leave `qa_pass=false`.
- Save final desktop/mobile screenshots under `ralph/screenshots/build/insights-005-final-*.jpg`.

**Verification**: focused contract/unit/E2E tests, full `TEST_DATABASE_URL=postgresql://postgres@localhost:55432/opengithub_identity_test DB_SSL=false CARGO_INCREMENTAL=0 make check`, full `DB_SSL=false CARGO_INCREMENTAL=0 make test`, `make test-e2e` or direct Playwright equivalent if the wrapper stalls, and mandatory Editorial banned-value scan: `rg -n -e '#(0969da|1f883d|1a7f37|cf222e|82071e|f6f8fa|1f2328|d0d7de|59636e|f1aeb5|fff1f3)\b|@primer/|Octicon' web/src/ --glob '!**/og*.css'`.
